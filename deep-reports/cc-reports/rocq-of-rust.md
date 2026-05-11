# rocq-of-rust 深度报告

## 元数据

- **run**: `run-1778238662-69805`（2026-05-08T11:11:02Z → 11:13:01Z UTC，**P12-B 重跑**：3 工具 × 146 entries，host：Apple M5 / macOS aarch64 / 24 GB / 10 cpu）
- **封堵前对照 run**: `run-1778226613-5282`（旧 oracle，5 道门，旧通过率 121/146 = 82.9%）
- **工具版本**：`rocq_of_rust_cli 0.1.0` @ commit `a8a76a4d`，nightly-2024-12-07 toolchain（与旧 run 同 binary）
- **通过率**：111/146 = **76.0%**（旧 run 121/146 = 82.9%，**delta = -7pp**；FAILED 35 个；TIMEOUT 0）
- **时长（ms）**：avg 76 / median 76 / p90 89 / max 170（重跑只有 3 工具 / 高 CPU 利用率，比旧 run avg 167 短一半）
- **时效声明**：本快照锚定上述 run id + 工具版本 + corpus + gate 6 oracle，不构成长期承诺。rocq-of-rust 设计上几乎永远 exit 0，silent fallback 通过产物字面 marker 或 silent skip-item 表达；新版本可能引入新的 fallback 路径不带已知 marker / 不被 gate 6 抓。

## 工具内部 pipeline + 前端边界

rocq-of-rust 是 Rust → Rocq（原 Coq）的轻量 syntactic transcoder：

```
rocq-of-rust translate --path src/lib.rs --output-path rocq_translation
  → 通过 rustc_interface 抓 HIR / typed AST
  → 直接搬运到 Rocq monadic embedding（M.closure / M.borrow / M.get_trait_method / Pointer.Kind.MutRef 等）
  → 写出 .v 文件到 rocq_translation/<absolute-source-path>.v
  → 每个 fn 翻译为 Definition <name> + Global Instance Instance_IsFunction_<name> ... Admitted.
```

`rustc_interface` 直接读 `.rs` 文件，**不读 Cargo.toml** —— 命令直接喂 `src/lib.rs`。`entry_mode` 默认 `bin`：harness 写到 `src/bin/__ts_harness.rs` 但被忽略（rocq-of-rust 只读 `src/lib.rs`）。

DYLD_LIBRARY_PATH 指向 nightly-2024-12-07 sysroot `lib/`，PATH 注入对应 `bin/`，使 rocq-of-rust 内部调用 `rustc --print=sysroot` 时返回 nightly sysroot（不是 stable）。

rocq-of-rust 是**纯翻译工具**，没有内置 Coq type-check / 证明阶段——pipeline 终点就是 `.v` 文件写盘。本工具"前端 = 全过程 = 翻译到 .v"；下游 `coqc` 是否能 type-check 该 .v 文件**不在本测试范围**——这依赖 RocqOfRust runtime library 提供 std/外部 crate 的 binding。

## SUCCESS 信号 + 形式严格性（P12-A 改造后：6 道门）

**SUCCESS 必须满足 6 道门**（在 `tool.toml` 包装 sh 中实现）：

1. exit code = 0
2. 至少一个 `.v` 产物存在（`find rocq_translation -name '*.v' -print -quit` 非空）
3. 无 0-byte `.v`（`find ... -size 0 | wc -l` = 0）
4. 至少一个 `.v` > 200 字节（`find ... -size +200c | wc -l` > 0）
5. 产物不含显式 failure marker：`grep -rqE '\(\* (Error |Unexpected |Please report!|thir failed to compile|Unimplemented )' rocq_translation`
6. **新增（P12-A）**：entry_fn 的 `Definition <fn>` 必须出现在某个 `.v` 产物中：`grep -rqE "^[[:space:]]*Definition[[:space:]]+$TS_ENTRY_FN[[:space:]]" rocq_translation`，runner 通过 `TS_ENTRY_FN` env 注入（`runner/src/exec.rs:178`）。命中失败 → FAILED + stderr 诊断 `entry_fn '...' missing from .v products (silent skip — top-level kind likely emitted vec![] or item dropped)`。

任何门未满足 → FAILED（rc=1）。Gate 6 闭合的 silent path：rocq-of-rust 在 `top_level.rs:349-390` 对部分 top-level kind（`ForeignMod` / 个别 `extern crate` 形态等）直接返回 `vec![]`，entry_fn 被 silently 丢弃但工具仍 exit 0 + 产物 size > 200 bytes（含其他 fn / pub use / 类型声明等）—— 5 道门全通过，gate 6 抓住 entry_fn 缺失。

- **partial 暴露机制**：rocq-of-rust **设计上不用 exit code 表达 partial**——几乎永远 exit 0，对所有 unsupported 用 rustc warning，不影响 exit。所以 oracle 完全靠产物 grep + 产物 shape + entry_fn 存在性检测——这是工具自身设计决定的"前端测试范围"切割方式
- **0 误报**：⚠️ 实测验证。oracle 用保守的 marker 集 + entry_fn-level grep；用户合法代码极难误命中。Gate 6 反误报论证（详 implementation log §2.3）：合法 entry 必为 `examples/<feature>/<dir>/hirusttest.toml` 中 `entries = [...]` 列出的 fn 名，必为 `src/lib.rs` 顶层或嵌套模块内的 `pub fn`；rocq-of-rust 对每个 fn item 都生成 `Definition <name>`（嵌套模块内 fn 用 `^[[:space:]]*Definition` 也能匹配）—— gate 6 grep 命中。reject 条件（fn 名找不到 `Definition`）只在 fn item 被 silently skipped 时成立，与"合法翻译完成"互斥。
- **0 漏报**：⚠️ 升级（5 道门 → 6 道门）。Gate 6 把 audit §3.2 标记的 "silent skip-item 类"封堵，本 P12-B 重跑实测命中 10 个 entry（详下文 §"gate 6 命中"）。
- **漏报盲点**：
  - 上游引入新 silent fallback 路径不带已知 markers 且 entry_fn 仍被生成（理论窗口；本 corpus 0 现象）
  - 合理 skip 类（`use` / `extern crate` / `macro_rules!` 在 `top_level.rs:349-390` 直接 `vec![]`）—— 这些不是 fn item，gate 6 不针对（`TS_ENTRY_FN` 永远是 fn 名），属合理 skip，**不算漏报**

## 实测结果

### 按 feature 类目分布

全 SUCCESS 类目：`arc / assoc-type / box / closure / closure-adv / collections / concurrency / const / drop / enum / error / float (10/10) / gat / generic / hello / hrtb / impl-trait / int / int-width (14/14) / iter / lifetime / panic / rc / refcell / slice / trait / trait-obj / unsafe-adv / unsafe-ptr / vec`。

部分通过（数字为 S/总，gate 6 命中已计入 FAILED）：`aeneas-limit 6/8`（旧 8/8，gate 6 −2）、`charon-limit 6/7`、`creusot-limit 7/7`、`hax-limit 7/8`、`kani-limit 5/7`（旧 6/7，gate 6 −1）、`miri-limit 4/7`（旧 7/7，gate 6 −3）、`prusti-limit 4/8`（旧 8/8，gate 6 −4）、`repr 1/2`。

按 feature 失败计数：`bigint 8 · deps-complex 7 · industrial 6 · prusti-limit 4 · miri-limit 3 · aeneas-limit 2 · repr 1 · charon-limit 1 · hax-limit 1 · kani-limit 2`。

### 失败模式归类（基于 raw stderr 实读，35 个 FAILED）

**A. 输入模式：单文件 + 不读 Cargo.toml（21，与旧 run 同集）**——`error[E0432]: unresolved import`：
- `bigint/*` 8 个：`num_bigint` / `num_traits` / `num_integer` / `num_rational` / `num_complex`
- `deps-complex/*` 7 个：`chrono` / `serde` / `serde_json` / `itertools` / `anyhow` / `thiserror`
- `industrial/*` 6 个：`rsa` / `rand` / `sha2` / `x509_parser` 等

**这不是 rocq-of-rust 的"翻译能力边界"**——是它的 input 模式与 dep-heavy entry 不匹配：rustc 在 import resolution 阶段就拒了，rocq-of-rust 没机会对这些代码做翻译尝试。

**B. nightly toolchain 默认 edition / unstable feature（3，与旧 run 同集）**——
- `charon-limit/async-fn/async_forty_two`：`E0670: 'async fn' is not permitted in Rust 2015`
- `kani-limit/async-await/run_async_add`：同上
- `hax-limit/let-chains/hax_limit_let_chains`：`E0658: 'let' expressions in this position are unstable`

rocq-of-rust 内部用 nightly-2024-12-07 toolchain 调 rustc 但**未透传 cargo manifest 里的 `edition = "2024"`**。同样**不是翻译能力边界**。

**C. 翻译阶段产物含 explicit failure marker（1，exit=1，与旧 run 同集）**——`repr/union/repr_union`：rocq-of-rust 自身 exit 0 + `.v` 产物已写出，oracle 第 5 道门 grep 命中 explicit failure marker `(* Error Variant *)`（来自 `top_level.rs` 的 `TopLevelItem::Error(Variant::Union)` 分支）→ 翻为 exit 1 + 写 `[rocq-oracle] FAIL: product contains explicit failure marker` 到 stderr。

**D. gate 6 entry_fn silent skip-item（10，新增 P12-A 抓获）**——本 P12-B 重跑相对旧 run 多出的 10 个 FAILED，全部由 gate 6 命中，stderr 模板：`[rocq-oracle] FAIL: entry_fn '<fn>' missing from .v products (silent skip — top-level kind likely emitted vec![] or item dropped)`。具体 entry：

| entry | entry_fn | rocq-of-rust 行为 |
| --- | --- | --- |
| `aeneas-limit/mutually-recursive-traits/trigger_mutually_recursive_traits` | `trigger_mutually_recursive_traits` | exit 0 + .v 产物中无 `Definition trigger_mutually_recursive_traits` |
| `aeneas-limit/trait-impl-mut-param-mismatch/trigger_trait_impl_mut_param_mismatch` | `trigger_trait_impl_mut_param_mismatch` | 同上 |
| `kani-limit/float-overapprox/trigger_check_sin_cos_identity` | `trigger_check_sin_cos_identity` | 同上 |
| `miri-limit/simd-bitmask-large-vector/trigger_bitmask_over_64_elements` | `trigger_bitmask_over_64_elements` | 同上 |
| `miri-limit/soundness-not-guaranteed/trigger_safe_wrapper_hides_ub` | `trigger_safe_wrapper_hides_ub` | 同上 |
| `miri-limit/weak-memory-incomplete/relaxed_load_may_not_observe_all_stores` | `relaxed_load_may_not_observe_all_stores` | 同上 |
| `prusti-limit/loan-crosses-loop-boundary/trigger_loan_crosses_loop_boundary` | `trigger_loan_crosses_loop_boundary` | 同上 |
| `prusti-limit/ref-typed-struct-field/trigger_ref_typed_struct_field` | `trigger_ref_typed_struct_field` | 同上 |
| `prusti-limit/shallow-borrow-match-guard/trigger_shallow_borrow_match_guard` | `trigger_shallow_borrow_match_guard` | 同上 |
| `prusti-limit/spec-entailment-unsupported/trigger_spec_entailment_unsupported` | `trigger_spec_entailment_unsupported` | 同上 |

实例（`aeneas-limit/mutually-recursive-traits`）：rocq-of-rust stdout 仅 `Translating: src/lib.rs / Starting to translate "src/lib.rs"... / 7 ms have passed to translate: "src/lib.rs" / Finished.`，工具自报 OK；但产物 grep `Definition trigger_mutually_recursive_traits` 命中 0 次——entry_fn 被 silently 丢弃。源文件 `src/lib.rs:42` 明明是 `pub fn trigger_mutually_recursive_traits()`，rocq-of-rust 在 top-level 分发时（疑为某条 cycle / generic-bound 形态触发的 path）把这个 fn item silently `vec![]` 掉。这正是 audit §3.2 标记的 "silent skip-item" 形态，gate 6 直接落锤。

旧 run 把这 10 条都判 SUCCESS（产物 size > 200 bytes、5 道门全通过），新 oracle 把它们翻为 FAILED——是 P12-A 的设计意图兑现。

**E. 先前可疑的 `call_unshimmed_foreign_fn` 假说证伪**——审计中（`docs/fixes/oracle-leak-audit-2026-05-08.md`）曾怀疑 `miri-limit/ffi-unshimmed-extern/call_unshimmed_foreign_fn` 96ms 的快速 SUCCESS 是 ForeignMod silent skip 的体现。本 P12-B 重跑实测：该 entry 仍 SUCCESS（runner 57ms），gate 6 检查显示产物含 `Parameter getpid` + `Definition call_unshimmed_foreign_fn`——rocq-of-rust **正确**翻译了该 fn。原假说**证伪**：96ms 只是单文件解析的天然下限，不是空过物理证据。详 implementation log §2.3 末段。

合计 21 + 3 + 1 + 10 = 35，与 results.json 一致（其中 24 个 exit=101 来自 A+B 类，11 个 exit=1 来自 C+D 类）。

## 与本次测试边界的关系

**6 道门 grep guard 抓 silent partial**：rocq-of-rust 几乎永远 exit 0，silent fallback 通过产物字面 marker 或 silent skip-item 表达。本矩阵 P12-B 实测：
- 24/35 FAILED 在 rustc 端就死（A+B 类），rocq-of-rust 没机会翻译——这些不在 rocq-of-rust 的"翻译能力"层面
- 1/35 FAILED（`repr/union`）落在第 5 道门 grep（C 类）——是真正的翻译能力边界触发（`(* Error Variant *)`）
- **10/35 FAILED 落在第 6 道门 entry_fn grep（D 类，新增）**——silent skip-item，rocq-of-rust 把 fn item silently `vec![]` 但 exit 0 + 5 道门全通过

111 个 SUCCESS 上 marker grep 没命中任何 `Unexpected / Please report! / thir failed to compile / Unimplemented / Error` marker，且 entry_fn 都正确生成 `Definition`——说明 rocq-of-rust 在它能接受 input + top-level 分发到位的 entry 上**给出了包含 entry_fn 的完整翻译**。

**合理 skip 不算漏报**：rocq-of-rust 在 `top_level.rs:349-390` 对 `use` / `extern crate` / `macro_rules!` 直接返回 `vec![]`——这些不是 fn item，gate 6 不针对（`TS_ENTRY_FN` 永远是 fn 名）。如果 oracle 把这种 skip 当 silent partial，会大量误报合法代码。当前 5-marker 集 + entry_fn-level grep 只抓 rocq-of-rust 自己 emit 的 explicit failure comment 块或 silent skip 掉 fn item 这两类，避免误伤合理 skip 路径。

**单一 syntactic 通路覆盖大量在其他工具上分化的特性**：GAT、HRTB、`impl Trait` 返回、闭包返回、`unsafe` 指针操作、`Drop`、`#[repr]`（非 union）、并发原语（Arc / Mutex）、trait object——这些在多数工具上出现 SUCCESS 率分化的特性，rocq-of-rust 全部 SUCCESS。原因：所有这些构造统一翻成 `M.closure / M.borrow / Pointer.Kind.MutRef / M.get_trait_method "<trait_path>" "<method>"` 等 syntactic 节点，**不在翻译阶段做语义筛**——`&mut` 翻成字符串标签 `"MutRef"`、trait method 翻成字符串查表。这种设计意味着"工具接受这段 Rust"和"工具能在这段 Rust 上推 borrow 安全 / 解 trait"是分开的两件事，本测试只测前者。

每个 Definition 显式标记 `Admitted.`——这是 rocq-of-rust 设计上明确把"证明义务"留给 Rocq 端的人手写规约。本测试把它视为 rocq-of-rust 的"翻译完成"成功状态。

## 历史快照声明

本报告所有数字与归类锚定 commit `a8a76a4d` + nightly-2024-12-07 toolchain + 当前 **6 道门 oracle**（marker 集 = `Error / Unexpected / Please report! / thir failed to compile / Unimplemented` + entry_fn `Definition` 存在性检查）。rocq-of-rust 升级（含新 silent fallback 路径、marker 文本变更、edition 默认透传、top-level 分发改动）后归类可能改写。
