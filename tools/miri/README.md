# MIRI

解释器 + UB 检测器——在 MIR 层解释执行 Rust 程序并检测未定义行为。

## 简介

MIRI（Mid-level Intermediate Representation Interpreter）由 Rust 官方维护，内置于 rustup nightly toolchain。它在 MIR 层模拟执行程序，能检测内存越界、use-after-free、未初始化内存读取、数据竞争、不合法的原始指针操作等 UB。官方仓库：<https://github.com/rust-lang/miri>

## 本测试集中的"前端接受"定义

本测试集筛选 Rust **前端特性覆盖广度**——把每个工具测到"工具自带后端验证器/求解器之前"为止。

MIRI **没有"前端 / 后端"分界**——它就是一个 MIR 解释器，工作内容就是解释执行 + UB 检测。**所以 MIRI 的"前端 = 全过程"**（与 cargo-check 同形，但比它深一层：不只类型检查，还实际执行）。

按宪法 §六-2 不允许 partial（"SUCCESS = 工具完整完成它的工作单元，不允许任何 partial / silent skip / 半翻译"）："工具完成它自己的工作" = MIRI 解释执行**完整跑完且无任何中断**。具体：

- **算 SUCCESS**：程序正常终止退出（exit 0）—— 解释器完整跑完且无 UB / unsupported / panic
- **算 FAILED**：任何中断（UB 检测 abort / unsupported operation / panic / rustc 编译失败）→ exit ≠ 0

> 注：UB 检测在某些视角下可理解为"工具有效输出"，但按宪法精神（不允许 partial = 必须完整跑完）一律 FAILED——解释执行被中断 = 没完整跑完。oracle（runner 仅按 exit code 判定）与宪法一致。

- **不产生持久化产物**：execution state 进程结束即销毁。函数级对应性概念不适用

### SUCCESS 信号（严格反映前端特性支持范围）

为了严格反映前端特性支持范围（不允许 partial / 不接受中断），**SUCCESS = miri exit 0**（解释执行完整跑完且无 UB / unsupported operation）。任何中断 → FAILED。

- **partial 暴露机制**：UB / unsupported operation / panic 任意触发 → exit ≠ 0
- **形式严格性 — 0 误报（不冤枉能力）**：✅ 形式可证。miri exit 0 ⇔ 解释执行完整跑完且无 UB / unsupported operation
- **形式严格性 — 0 漏报（不高估能力）**：✅ 形式可证。任何 UB / unsupported / panic 触发 → miri exit ≠ 0
- **漏报盲点**：无（miri 不存在 silent skip）

注：UB 检测在某些视角下被理解为"工具有效输出"——但按"不允许 partial / 完整完成"精神，本测试一律 FAILED（解释执行没完整跑完）。

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
