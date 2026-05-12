# cargo-check

Baseline——验证样例本身能被 rustc 成功编译，与任何验证器无关。

## 简介

`cargo check` 是 Cargo 内置子命令，在不生成最终二进制的情况下对 crate 做完整类型检查与借用检查。由 Rust 官方维护，使用 rustc 前端。在本框架中作为"健全性基准"存在：若某工具在某 entry 上 FAILED，可先看 cargo-check 是否同样 FAILED——若是，说明是样例本身的 Rust 错误，而非工具自身的限制。官方文档：<https://doc.rust-lang.org/cargo/commands/cargo-check.html>

## 本测试集中的"前端接受"定义

本测试集筛选 Rust **前端特性覆盖广度**——把每个工具测到"工具自带后端验证器/求解器之前"为止，不看下游求解结果。

cargo-check **没有后端**：rustc 跑完整前端（parse + macro expand + name resolution + type check + borrow check + MIR build）即 exit 0，不进入 codegen / LLVM IR。所以本工具的"前端 = 全过程"。

- **判定**：exit 0 = SUCCESS；exit ≠ 0 = FAILED
- **不产生持久化产物**：MIR 在内存里构造完即丢弃，不存盘。函数级对应性概念不适用
- **矩阵中的角色**：rustc 编译基线。任何 entry 在其它工具上 FAILED 时，先看 cargo-check 是否同时 FAILED——若是，说明 entry 自身 Rust 不合法，与工具能力无关

### SUCCESS 信号（严格反映前端特性支持范围）

为了严格反映前端特性支持范围，**SUCCESS = `cargo check` exit 0**（rustc 类型 / 借用检查通过）。任何错误项 → FAILED。

- **partial 暴露机制**：rustc 自身的 error registry——任何错误 exit ≠ 0
- **形式严格性 — 0 误报（不冤枉能力）**：✅ by-design no-silent-skip。rustc 单一 exit-code 信号通路；exit 0 ⇔ type / borrow check 全部通过（rustc 设计意图，非项目层形式证明）
- **形式严格性 — 0 漏报（不高估能力）**：✅ by-design no-silent-skip。任何 check 失败 → rustc exit ≠ 0（rustc 不存在 silent skip 设计模式）
- **漏报盲点**：
  - 边界情况理论可能：rustc 内部 bug 让某种 unsafe code 漏过 check（极罕见，未在 v6 corpus 命中）
  - cargo-check 不做 verifier-style 验证，"通过" = "rustc 接受"，不蕴含 "代码正确"——baseline 角色而非工具评估对象

## 安装

上游：<https://doc.rust-lang.org/cargo/commands/cargo-check.html>（随 Rust 工具链自带）。

本测试基线：稳定版 `cargo` / `rustc`（与机器上的默认 toolchain 一致）。

按上游文档自行准备好 Rust toolchain；runner 直接调用 PATH 中的 `cargo`，无需为本工具配置 `TS_*` 变量。本项目不提供安装脚本或步骤教程，避免在工具版本变迁后误导。

## 本框架配置

参见 `tool.toml`。关键参数：

- `command = ["cargo", "check", "--bin", "__ts_harness"]`：对注入 harness 的 bin target 做类型检查，不进行链接或代码生成，速度极快。
- timeout / extra_cargo_deps / entry_mode：均使用默认值（无需显式配置）。
- harness 形态：`fn main() { <crate>::<entry>(); }` 标准 bin 入口，cargo-check 只要能通过类型检查即算 SUCCESS。

cargo-check 是矩阵中唯一不测验证能力的行，其他所有工具的 FAILED 结果应先对照此行排除"样例本身编译错误"的可能。

## 已知限制 / 坑

- macOS arm64 无平台特定限制，随 stable toolchain 正常运行。
- cargo-check 只检查编译期正确性，不执行代码，因此无法检测运行时 UB 或语义错误。

## 关联 sub-tests

cargo-check 是 baseline，不存在"自声明的工具限制"，无 `examples/cargo-check-limit/`。
