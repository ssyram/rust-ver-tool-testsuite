# Verus

SMT-based spec verifier for Rust，基于 `verus! {}` 宏块写 precondition/postcondition 并由 Z3 验证。

## 简介

Verus 由 UCSD/CMU 联合出品，允许开发者在 `verus! { }` 宏块内使用类 Dafny 语法编写 Rust 函数规格，通过 Z3 做 SMT 验证。内置 `vstd` 标准库验证支持，不依赖 Cargo 做 vstd 解析（standalone binary 模式内置 `libvstd.rlib`）。

GitHub: <https://github.com/verus-lang/verus>

## 本测试集中的"前端接受"定义

本测试集筛选 Rust **前端特性覆盖广度**——把每个工具测到"工具自带后端验证器/求解器之前"为止，不看下游求解结果。

对 Verus，这条边界**理想上**应落在 **VIR/AIR 构造** 与 **Z3 SMT 求解** 之间。但实测发现 Verus 的 `--no-verify` flag **同时切断 AIR 构造与 Z3 调用**（all-or-nothing），无 `--no-z3` / `--rlimit 0` 独立切线。所以本工具能拿到的最深前端表示是 **VIR**，而非 AIR/SMT-LIB：

- **前端**（本工具检测的范围）：verus-driver 接管 HIR/MIR + `verus! {}` macro 扩展 + Verus type check + lifetime check + VIR 构造
- **后端**（本工具不检测）：simplified VIR → AIR (Verus 自家 SMT-LIB 前置形式) → SMT-LIB → Z3 求解

`--no-verify` 之后若加 `--log vir`，VIR 会落到 `.verus-log/crate.vir`，便于函数级对应性校验（每个 fn 在 VIR 里以 `(Function :name (Fun :path lib::<crate>::<fn>) ... :body ...)` 存在）。

- **判定**：exit 0 = SUCCESS（VIR 构造完成且未被 Verus 前端拒收）；exit ≠ 0 = FAILED
- **真实失败常见来源**：未加 `assume_specification` 的标准库 API（`String::from` / `Any` trait / `Box::clone` 等）整体被拒。这是 Verus 设计选择——开发者必须显式给 std 写 spec——比 kani"先编再说"的策略保守

### SUCCESS 信号（严格反映前端特性支持范围）

为了严格反映前端特性支持范围（不允许 partial），**SUCCESS = `verus --no-verify --log vir` exit 0**（VIR 构造完成）。任何 partial → FAILED。

- **partial 暴露机制**：Verus 任何 rejection（lifetime / type check / `assume_specification` 缺失 / `verus_builtin not imported` 等）→ exit ≠ 0
- **形式严格性 — 0 误报（不冤枉能力）**：✅ 形式可证。verus exit 0 ⇔ VIR 构造完成无错误
- **形式严格性 — 0 漏报（不高估能力）**：✅ 形式可证。Verus 任何 rejection 都通过 `dcx().emit` 触发 exit ≠ 0
- **漏报盲点**：无（注：`--no-verify` 同时切 AIR + Z3，最深前端到 VIR——所以"前端边界 = VIR 构造完成"）

## 安装

上游：<https://github.com/verus-lang/verus>

本测试基线：release `0.2026.05.03.8b81855`（搭配 Rust toolchain `1.95.0`）。

按上游文档自行安装（prebuilt release 或源码自行处理）。装好后把 `verus` 可执行文件路径填到 `.env` 的 `TS_VERUS_BIN`。本项目不提供安装脚本或步骤教程，避免在工具版本变迁后误导。

## 本框架配置

参见 `tool.toml`。关键参数：

- **command**：直接调用 `verus --crate-type=lib src/lib.rs`；不用 `cargo-verus`（见"已知坑"）。
- **entry_mode = "lib"**：Verus 强制要求每个处理的 crate 都包含 `use vstd::prelude::*` 和 `verus! {}`，plain Rust 文件被直接拒绝（`The verus_builtin crate was not imported`）。runner 将 `src/lib.rs` 重命名为 `src/__ts_inner.rs`，再写入 harness 作为新的 `src/lib.rs`。
- **harness 结构**：
  - `use vstd::prelude::*` + `mod __ts_inner;` + `pub use __ts_inner::*;`
  - `verus! {}` 块内含 `#[verifier::external] fn __ts_invoke()`，仅在类型层面引用 `entry_fn`，Verus 不对其做验证。
- **无 extra_cargo_deps**：verus binary 内置 vstd，不需要在 Cargo.toml 里加 vstd 依赖。

## 反作弊：`mod __ts_inner` 必须放进 `verus! {}` 块内

本测试的核心原则是测**工具自身能接受的代码范围**。Verus 的"前端"特指 `verus! {}` 宏块内对 Rust 子集的检查（parse + macro elaboration + Verus type check + lifetime check）。**块外的代码 Verus 直接透传给 stock rustc**——如果把 `mod __ts_inner;` 写在 `verus! {}` 外，块内只剩个 `__ts_invoke` 壳，inner 的真实代码就**没经过 Verus 自身前端**，等价于 cargo-check。SUCCESS 信号会退化成"rustc parses it"，对所有 entry 齐刷刷高分，丢掉 Verus 的特性子集分化信号——这就是作弊。

正确 harness（已落到 `harness.rs.tera`）：

```rust
use vstd::prelude::*;

verus! {
    mod __ts_inner;          // ← 必须在块内
    pub use __ts_inner::*;

    #[allow(dead_code)]
    #[verifier::external]
    fn __ts_invoke() {
        __ts_inner::{{ entry_fn }}();
    }
}
```

`#[verifier::external]` 是给 `__ts_invoke` 这层壳函数用的——告诉 Verus "**这个 wrapper** 不要 verify"，但 `mod __ts_inner` 仍受 Verus 前端检查（不带 `#[verifier::external]`）。所以矩阵里 verus 33% 的 SUCCESS 率才是 Verus 真实的"接受 Rust 子集"边界——它确实拒掉很多块外才合法的构造（闭包捕获 `&mut`、`Option::copied` 无 spec、`transmute` 无 spec 等）。

如果以后改 harness 一定要保住这条不变量：**`mod __ts_inner` 在 `verus! {}` 块内，且不带 `#[verifier::external]`**。

## 已知限制 / 坑

- **cargo-verus 不可用**：`cargo-verus` 作为 `RUSTC_WRAPPER` 尝试从 crates.io 重编译 vstd，但 `0.2026.05.03` 发布版 binary 与 crates.io 任何版本的 vstd 均不兼容，会 panic（`rust_verify/src/erase.rs:405`）。必须绕开，直接调用 `verus` binary。
- **工具链版本锁定**：verus binary 需要精确的 rustup 工具链版本（此版本为 `1.95.0-aarch64-apple-darwin`），首次运行会自动提示，需要 `rustup install` 安装。
- **binary 路径**：当前 `tool.toml` 指向 `/tmp/ts-tools-install/...`（临时目录）。机器重启后需重新下载或移至持久路径（如 `~/.verus/`）。
- **plain Rust 行为**：`0 verified, 0 errors` + exit 0——Verus 不对无 spec 的函数做验证，仅做类型检查，这是正确的"无 spec 即无验证"语义。

## 关联 sub-tests

本工具未派生限制集 agent，无 `examples/verus-limit/`。

plain Rust 样例预期 SUCCESS（`0 verified, 0 errors`，exit 0）。类型错误样例预期 FAILED（exit 1）。
