# MIRI

解释器 + UB 检测器——在 MIR 层解释执行 Rust 程序并检测未定义行为。

## 简介

MIRI（Mid-level Intermediate Representation Interpreter）由 Rust 官方维护，内置于 rustup nightly toolchain。它在 MIR 层模拟执行程序，能检测内存越界、use-after-free、未初始化内存读取、数据竞争、不合法的原始指针操作等 UB。官方仓库：<https://github.com/rust-lang/miri>

## 本测试集中的"前端接受"定义

本测试集筛选 Rust **前端特性覆盖广度**——把每个工具测到"工具自带后端验证器/求解器之前"为止。

MIRI **没有"前端 / 后端"分界**——它就是一个 MIR 解释器，工作内容就是解释执行 + UB 检测。**所以 MIRI 的"前端 = 全过程"**（与 cargo-check 同形，但比它深一层：不只类型检查，还实际执行）。

按宪法 §四 B "测必要条件 / 非语义对错"——只问"工具能不能吃下这段代码并产出预期形状的输出"。具体到 MIRI：

- **算 SUCCESS**（工具完成工作的两种形态）：
  - 程序正常终止退出（exit 0）—— 解释器完整跑完且 entry 代码无 UB
  - **解释执行完成 + MIRI 自陈在 entry 代码里检测到 UB**（stderr `Undefined Behavior` 且无 `unsupported operation`）—— MIRI 最有价值的输出，按 architecture §一 "bug detect 归 SUCCESS" / §四 B 派生
- **算 FAILED**（工具不能吃下这段代码）：
  - MIRI 自陈 unsupported（`unsupported operation: inline assembly is not supported` / `can't call foreign function 'X'` / `not available when isolation is enabled`）
  - rustc 编译失败 / cargo build 失败

判定由 `miri-strict-wrapper.sh` 实施（P35 新加，2026-05-12）。优先级：unsupported 路径 > UB-detect 路径 > 默认。

> P35 前 oracle 一律把 exit ≠ 0 算 FAILED，把"UB 检测"和"工具不支持"混桶，与宪法 §四 B 实际冲突（§四 B 只问能不能吃下不问语义对错——bug detect 是"吃下了 + 给出预期输出"）。P35 wrapper 把两者分开。

- **不产生持久化产物**：execution state 进程结束即销毁。函数级对应性概念不适用

### SUCCESS 信号（按 architecture §一 / §四 B）

**SUCCESS = MIRI 跑完工作 + 输出有意义结果**：

- exit 0（正常终止 / 无 UB）
- 或 exit ≠ 0 + `Undefined Behavior` 自陈 + 无 `unsupported operation` → wrapper 翻 SUCCESS

**FAILED = MIRI 不能吃下这段代码**：

- exit ≠ 0 + `unsupported operation` / `can't call foreign function` / `not available when isolation is enabled`

形式严格性：

- **0 误报（不冤枉能力）**：✅ 实测 + wrapper 双路区分。SUCCESS 两路（exit 0 / UB-detect）均反映工具真实工作完成
- **0 漏报（不高估能力）**：✅ 实测。FAILED 三路（unsupported / 外部 fn / isolation 阻塞）完整覆盖工具能力边界
- **漏报盲点**：无（MIRI 不存在 silent skip）

## 安装

上游：<https://github.com/rust-lang/miri>（随 rustup nightly toolchain 分发，作为 component 安装）。

本测试基线：跟随当前 `nightly` toolchain 中 `miri` component 的可用版本。

按上游文档自行安装；runner 直接调用 PATH 中的 `cargo +nightly miri`，无需为本工具配置 `TS_*` 变量。本项目不提供安装脚本或步骤教程，避免在工具版本变迁后误导。

## 本框架配置

参见 `tool.toml`。关键参数：

- `command = ["cargo", "+nightly", "miri", "run", "--bin", "__ts_harness"]`：显式指定 `+nightly` toolchain，因为 miri 只在 nightly 上可用。用 `run` 而非 `check`——miri 必须实际执行程序才能检测 UB，仅类型检查无意义。
- `timeout_secs = 300`：miri 解释执行速度远低于原生执行，复杂程序（深递归、大容器、并发）可能耗时较长，故超时适当放宽。
- harness 形态：`fn main() { <crate>::<entry>(); }` 标准 bin 入口，miri 解释执行 main 进而调用 entry。

## 已知限制 / 坑

- miri 不支持内联汇编（`asm!`）、部分 FFI 调用（无 shim 的 extern fn）、SIMD 部分指令，这类 entry 会直接 FAILED。
- 网络 / 系统调用等 OS 接口 miri 默认拒绝执行，会报 UB 或不支持错误。
- 解释执行比原生慢 10-100 倍，循环展开量大的样例需注意超时风险。
- macOS arm64 本身无特殊兼容问题，但 nightly toolchain 更新后 miri component 可能短暂不可用，需等官方同步。

## 关联 sub-tests

`examples/miri-limit/` 是 MIRI 自声明的限制集。这些 entry 故意触发 MIRI 的已知"不支持"特性（如 `inline-asm`、`ffi-unshimmed-extern`、`networking-unsupported`、`simd-bitmask-large-vector`、`uninit-memory` 等），期望 MIRI 在这些 entry 上 FAILED。
