# Axis E: examples 中立性 vs 宪法 §四 原则 A

> 审计时间：2026-05-12
> 范围：`examples/` 42 个 feature 桶 / 约 161 个 entries
> 抽样模式：每 -limit/ 桶抽 1 个 + industrial/ 抽 1 + runnable/ 抽 10 + 非-limit 桶抽 5

## 1. 总览

- 候选数（违反原则 A 的样例）：**0 强候选 / 0 弱候选**
- 抽样数：约 25 / 161（含每个 -limit 桶代表 + runnable 全枚举 + 散桶抽查）
- 关键证据：
  - `examples/**/*.rs` 全文 grep verifier attribute / macro / 引用 `kani::proof|prusti::|creusot_contracts|verus!|#[ensures]|#[requires]|#[invariant]`：**0 命中**（剔除注释/doc）
  - 散落的 `kani::unwind / charon::opaque / #[pure]` 字串全部位于 `//!` 或 `//` 文档注释中，作为"限制说明"，不是代码侵入
  - `examples/**/Cargo.toml` grep verifier 依赖 `kani|prusti|creusot|verus|aeneas|hax|charon|verifast`：**0 命中**
  - 根 `Cargo.toml` 已将 `examples` 在 `workspace.exclude` 内，且各 entry 自带独立 `Cargo.toml` + `Cargo.lock`，hirusttest.toml 不被任何 Cargo.toml 引用 → A 形式定义（hirusttest 加入前后 cargo 字节级一致）满足
  - 跨 entry path 依赖 grep：仅 `industrial/{rsa,sha2,x509-parser}/*/Cargo.toml` 指向 `vendor/`（vendored crate，非 entry 间共享），entry ↔ entry 路径依赖 **0 命中**
  - `find examples -name '*.rs' -not -path '*/lib.rs' -not -path '*/main.rs'` 全部为 build artifact（`target/.../build/.../out/oid_db.rs`），无人工 helper 文件

## 2. 候选清单

无候选。下列为合规观察的代表性证据：

### 证据 A：-limit/ 桶纯净 Rust 不夹工具属性

- `examples/kani-limit/loop-unwinding/src/lib.rs:1-30`：仅 `//!` 注释引用 `#[kani::unwind]` 说明限制；函数体 `pub fn sum_to_n(n: u64) -> u64` 是纯 Rust
- `examples/prusti-limit/closure-in-pure-fn/src/lib.rs:1-24`：同样仅注释提及 `#[pure]`，代码体 `(|| n > 0)()` 是纯 Rust
- `examples/aeneas-limit/closure-if-capture/src/lib.rs`、`charon-limit/generic-to-dyn-unsize`、`creusot-limit/dyn-trait-forbidden`、`hax-limit/let-chains`、`miri-limit/inline-asm`：同一模式（限制说明在注释，源码纯净）

### 证据 B：runnable 桶符合 P21 `[runnable.<entry>]` schema

- 抽样 10 个（abs / add-two / add-u32 / bool-ops / digit-sum / enum-classify / fact / fib / gcd / max3 / parity / power / saturating / struct-norm / sub-clamped 共 15 个，全部 hirusttest.toml 形如：
  ```
  entries = ["<name>"]
  [runnable.<name>]
  inputs = [...]
  expected = [...]
  ```
- schema 完全合 P21

### 证据 C：industrial/ 三 entry 各自独立 lib crate

- `industrial/rsa/rsa-pkcs8/Cargo.toml`：`[lib] path="src/lib.rs"`，依赖 `rsa = { path="../../../../vendor/rsa" }`
- `industrial/sha2/sha256-digest`、`industrial/x509-parser/cert-parse`：同模式
- 每个 entry 自带 `src/lib.rs`、独立 Cargo.lock、独立 hirusttest.toml

### 证据 D：每 entry 一个独立 lib crate（§五 纯净性）

- 所有 entry 仅 `src/lib.rs` 一份源文件（非 build artifact 例外为 0）
- 跨 entry `mod`/`use crate::`/`use super::` 引用 grep：0 命中
- helper 全部内联在 entry 自己 lib 内（与 CLAUDE.md §"entry 之间禁止共享关键依赖"一致）

### 证据 E：hirusttest.toml 的 cargo-中立性

- 根 `Cargo.toml`：`workspace.exclude = ["examples", "tools", "runs", ".tmp", "vendor"]`
- 各 entry 的 `Cargo.toml` 不引用 `hirusttest.toml`（grep 0 命中）
- hirusttest.toml 是 runner 工具读取的信号文件，Cargo 不感知 → 删除前后 `cargo build` 的 manifest 解析、依赖图、编译输入字节级一致 → A 形式定义满足

## 3. 总结

- **强候选**（明显侵入）：**无**
- **弱候选**（边界情况）：**无**
- **抽样覆盖度评估**：
  - 高置信桶：-limit/ 全部 7 个工具桶各抽 1 代表 + grep 全 examples → 工具属性 0 命中，可推广至全部 -limit
  - 高置信桶：runnable/ 抽 10/15 + schema 一致 → 全桶合规
  - 中置信桶：industrial/ 3 entries 全审 → 合规
  - 中置信桶：非-limit/ 散桶（arc/closure/trait-obj/unsafe-adv/generic）抽样未见违规
  - 抽样未覆盖：约 130 个非-limit entry 未逐一打开。**反状态 R1 适用**：未抽到不能 generalize，但 grep 全 examples 的 verifier 标记/依赖 0 命中是全量证据，可补足"无 verifier 污染"维度的全覆盖
- **结论**：examples 当前状态对 §四 原则 A（双方都不可侵入）合规。无需向用户报告候选。
