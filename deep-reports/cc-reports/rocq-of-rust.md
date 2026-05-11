# rocq-of-rust 深度报告

## 元数据

- **run (v4 当前)**: `run-1778480785-20112`（2026-05-11T14:26:25Z UTC，单工具重跑：rocq-of-rust × 161 entries；wrapper-based oracle，gate 6 N=7 attempt 鲁棒化；host：Apple M5 / macOS aarch64 / 24 GB / 10 cpu）
- **run (v3 历史)**: `run-1778238662-69805`（2026-05-08T11:11:02Z UTC，**P12-B 重跑**：3 工具 × 146 entries）
- **封堵前对照 run**: `run-1778226613-5282`（5 道门旧 oracle，121/146 = 82.9%）
- **工具版本**：`rocq_of_rust_cli 0.1.0` @ commit `a8a76a4d`，nightly-2024-12-07 toolchain（与旧 run 同 binary）
- **通过率 v4**：124/161 = **77.0%**（仅看共享 146 entry 子集：109/146 = 74.7%，**vs v3 −1.3pp**——P15-impl 反向暴露的 gate 6 非确定性 silent skip 修复抓到 2 个原 SUCCESS 翻 FAILED）；corpus 扩展至 161 (+15 `runnable/*`)；FAILED 37 个；TIMEOUT 0
- **通过率 v3**：111/146 = 76.0%（旧 5 道门 121/146 = 82.9%，delta vs 5 道门 = −7pp）
- **时长（ms） v4**：avg 477 / median 547 / p90 641 / max 1036（N=7 attempt wrapper translate overhead 主导，~6× v3 的 ~76ms avg；仍远低于 120s timeout）
- **时长（ms） v3**：avg 76 / median 76 / p90 89 / max 170
- **时效声明**：本快照锚定上述 run id + 工具版本 + corpus + gate 6 N-attempt wrapper oracle，不构成长期承诺。rocq-of-rust 设计上几乎永远 exit 0，silent fallback 通过产物字面 marker 或 silent skip-item 表达；新版本可能引入新的 fallback 路径不带已知 marker / 不被 gate 6 抓。

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

## SUCCESS 信号 + 形式严格性（v4：6 道门 + N-attempt wrapper）

**SUCCESS 必须满足 6 道门，且 wrapper 跑 N=7 次 `rocq-of-rust translate` 后每次都必须通过所有门**（实现见 `tools/rocq-of-rust/rocq-of-rust-wrapper.sh`；N=7 在经验 P(drop fn)≈0.6 的 `thread-local-ref` 上 catch rate ≈ 99.84%）：

1. exit code = 0（每次 attempt）
2. 至少一个 `.v` 产物存在（每次 attempt）
3. 无 0-byte `.v`（每次 attempt）
4. 至少一个 `.v` > 200 字节（每次 attempt）
5. 产物不含显式 failure marker：`grep -rqE '\(\* (Error |Unexpected |Please report!|thir failed to compile|Unimplemented )' rocq_translation_*`（每次 attempt）
6. entry_fn 的 `Definition <fn>` 必须出现在某个 `.v` 产物中：`grep -rqE "^[[:space:]]*Definition[[:space:]]+$TS_ENTRY_FN[[:space:]]" rocq_translation_*`，runner 通过 `TS_ENTRY_FN` env 注入（`runner/src/exec.rs:178`）（每次 attempt 都必须命中）

任何一次 attempt 的任何门未满足 → FAILED（rc=1）+ stderr 诊断 `[rocq-oracle] FAIL (attempt i/N): ...`。

**v4（2026-05-11）与 v3 的差异**：
- v3：tool.toml inline `sh -c`，单次 translate
- v4：wrapper-based，N=7 次 translate，AND-reduce 所有 attempt 的 6 道门（N 默认 7，可由 `ROCQ_OF_RUST_N_ATTEMPTS` env 覆盖）

升级原因：P15-impl 反向暴露——同一 entry (`creusot-limit/thread-local-ref/read_thread_local`) 同一 binary 同一 lib.rs 跑 rocq-of-rust translate **结果非确定性**：产物可能是变体 A (含 `Definition read_thread_local`、缺 thread_local! 宏展开) 或变体 B (含宏展开、缺 fn 定义)。两种都是 partial 翻译但旧单次 oracle 在变体 A 时漏报 SUCCESS。详 [`docs/fixes/ror-gate6-fix-2026-05-11.md`](../../docs/fixes/ror-gate6-fix-2026-05-11.md)。

Gate 6 闭合的 silent path（v3 + v4 共同覆盖）：
- v3 已知场景：rocq-of-rust 在 `top_level.rs:349-390` 对部分 top-level kind（`ForeignMod` / 个别 `extern crate` 形态等）直接返回 `vec![]`，entry_fn 被 silently 丢弃但工具仍 exit 0 + 产物 size > 200 bytes（含其他 fn / pub use / 类型声明等）—— 5 道门全通过，gate 6 抓住 entry_fn 缺失。
- v4 新增覆盖：rocq-of-rust 翻译路径非确定性导致 entry_fn 在部分 run 上 silently drop —— 单次 sample 无法稳定捕获，多次 sample AND-reduce 必命中其中一次 drop。

- **partial 暴露机制**：rocq-of-rust **设计上不用 exit code 表达 partial**——几乎永远 exit 0，对所有 unsupported 用 rustc warning，不影响 exit。所以 oracle 完全靠产物 grep + 产物 shape + entry_fn 存在性检测——这是工具自身设计决定的"前端测试范围"切割方式
- **0 误报**：⚠️ 实测验证。oracle 用保守的 marker 集 + entry_fn-level grep + 多次 sample AND-reduce；用户合法代码极难误命中。Gate 6 反误报论证（详 implementation log §2.3 + [`ror-gate6-fix-2026-05-11.md`](../../docs/fixes/ror-gate6-fix-2026-05-11.md) §5）：合法 entry 必为 `examples/<feature>/<dir>/hirusttest.toml` 中 `entries = [...]` 列出的 fn 名，必为 `src/lib.rs` 顶层或嵌套模块内的 `pub fn`；rocq-of-rust 对每个 fn item 都生成 `Definition <name>`（嵌套模块内 fn 用 `^[[:space:]]*Definition` 也能匹配）—— gate 6 grep 命中；N-attempt 也命中（确定性翻译路径的产物 byte-identical）。reject 条件只在 fn item 被 silently skipped（或非确定性 drop）时成立，与"合法翻译完成"互斥。
- **0 漏报**：⚠️ 升级二度（5 道门 → 6 道门 → v4 N-attempt wrapper）。Gate 6 把 audit §3.2 标记的 "silent skip-item 类"封堵；v4 N-attempt 把非确定性翻译路径也纳入捕获。
- **漏报盲点**：
  - 上游引入新 silent fallback 路径不带已知 markers 且 entry_fn 仍被生成（理论窗口；本 corpus 0 现象）
  - rocq-of-rust 引入新的非确定性翻译路径，3 次 attempt 都恰好采到含 entry_fn 的变体——可通过把 N 增大缓解（`ROCQ_OF_RUST_N_ATTEMPTS` env 已暴露）
  - 合理 skip 类（`use` / `extern crate` / `macro_rules!` 在 `top_level.rs:349-390` 直接 `vec![]`）—— 这些不是 fn item，gate 6 不针对（`TS_ENTRY_FN` 永远是 fn 名），属合理 skip，**不算漏报**

## 实测结果

### 按 feature 类目分布

全 SUCCESS 类目 (v4)：`arc / assoc-type / box / closure / closure-adv / collections / concurrency / const / drop / enum / error / float (10/10) / gat / generic / hello / hrtb / impl-trait / int / int-width (14/14) / iter / panic / rc / refcell / runnable (15/15, 新增) / slice / trait / trait-obj / unsafe-adv / unsafe-ptr / vec`。

部分通过（数字为 S/总，gate 6 命中已计入 FAILED）：`aeneas-limit 6/8`（v3 同；旧 8/8，gate 6 −2）、`charon-limit 6/7`、`creusot-limit 6/7`（**v4 thread-local-ref 翻 FAILED；v3 是 7/7**）、`hax-limit 7/8`、`kani-limit 5/7`（旧 6/7，gate 6 −1）、`lifetime 2/3`（**v4 thread-local 翻 FAILED；v3 是 3/3**）、`miri-limit 4/7`（旧 7/7，gate 6 −3）、`prusti-limit 4/8`（旧 8/8，gate 6 −4）、`repr 1/2`。

按 feature 失败计数 (v4)：`bigint 8 · deps-complex 7 · industrial 6 · prusti-limit 4 · miri-limit 3 · aeneas-limit 2 · repr 1 · charon-limit 1 · hax-limit 1 · kani-limit 2 · creusot-limit 1 · lifetime 1`。

### 失败模式归类（基于 raw stderr 实读，v4 = 37 个 FAILED；v3 = 35 个 FAILED）

> v4 相对 v3 增量：`creusot-limit/thread-local-ref/read_thread_local` + `lifetime/thread-local/thread_local_read`（两个 thread_local! entry 都被新 N-attempt wrapper gate 6 抓住——见 §"D. gate 6"）。其他类别无变化。


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

**D. gate 6 entry_fn silent skip-item（v4 = 12，v3 = 10）**——v3 P12-A 引入 gate 6 单次 grep 抓获 10 个；v4 N-attempt wrapper 在此基础上多抓 2 个 thread_local! 类（非确定性翻译路径 silent drop）。stderr 模板（v4）：`[rocq-oracle] FAIL (attempt i/N): entry_fn '<fn>' missing from .v products (silent skip — top-level kind likely emitted vec![] or item dropped, or rocq-of-rust non-deterministic translate path dropped entry on this attempt)`。具体 entry：

| entry | entry_fn | rocq-of-rust 行为 | gate 6 引入版本 |
| --- | --- | --- | --- |
| `aeneas-limit/mutually-recursive-traits/trigger_mutually_recursive_traits` | `trigger_mutually_recursive_traits` | exit 0 + .v 产物中无 `Definition trigger_mutually_recursive_traits`（确定性 silent skip） | v3 |
| `aeneas-limit/trait-impl-mut-param-mismatch/trigger_trait_impl_mut_param_mismatch` | `trigger_trait_impl_mut_param_mismatch` | 同上（确定性） | v3 |
| `kani-limit/float-overapprox/trigger_check_sin_cos_identity` | `trigger_check_sin_cos_identity` | 同上（确定性） | v3 |
| `miri-limit/simd-bitmask-large-vector/trigger_bitmask_over_64_elements` | `trigger_bitmask_over_64_elements` | 同上（确定性） | v3 |
| `miri-limit/soundness-not-guaranteed/trigger_safe_wrapper_hides_ub` | `trigger_safe_wrapper_hides_ub` | 同上（确定性） | v3 |
| `miri-limit/weak-memory-incomplete/relaxed_load_may_not_observe_all_stores` | `relaxed_load_may_not_observe_all_stores` | 同上（确定性） | v3 |
| `prusti-limit/loan-crosses-loop-boundary/trigger_loan_crosses_loop_boundary` | `trigger_loan_crosses_loop_boundary` | 同上（确定性） | v3 |
| `prusti-limit/ref-typed-struct-field/trigger_ref_typed_struct_field` | `trigger_ref_typed_struct_field` | 同上（确定性） | v3 |
| `prusti-limit/shallow-borrow-match-guard/trigger_shallow_borrow_match_guard` | `trigger_shallow_borrow_match_guard` | 同上（确定性） | v3 |
| `prusti-limit/spec-entailment-unsupported/trigger_spec_entailment_unsupported` | `trigger_spec_entailment_unsupported` | 同上（确定性） | v3 |
| **`creusot-limit/thread-local-ref/read_thread_local`** | `read_thread_local` | **非确定性**：30 次手测 ≈40% run 含 `Definition read_thread_local`，≈60% drop fn；N-attempt 7 次 catch rate ≈ 99.84% (= 1 − 0.4^7) | **v4** |
| **`lifetime/thread-local/thread_local_read`** | `thread_local_read` | **非确定性**：≈40% run 含 fn，≈60% drop fn；N-attempt 7 次几乎必抓 | **v4** |

实例（`aeneas-limit/mutually-recursive-traits`，v3 确定性 silent skip）：rocq-of-rust stdout 仅 `Translating: src/lib.rs / Starting to translate "src/lib.rs"... / 7 ms have passed to translate: "src/lib.rs" / Finished.`，工具自报 OK；但产物 grep `Definition trigger_mutually_recursive_traits` 命中 0 次（每次跑都 0）——entry_fn 被确定性丢弃。

实例（`creusot-limit/thread-local-ref/read_thread_local`，v4 非确定性 silent skip）：手工连跑 30 次产物 size 在 3118 (含 fn) / 2581 (含宏展开但无 fn) 之间 12:18 分布——P(drop fn) ≈ 0.6；v3 单次 oracle 采到 size 3118 时 SUCCESS、采到 2581 时 FAILED——P12-B baseline 恰好采到前者，P15-impl 反向暴露的同 entry 跑出后者。v4 N-attempt 把 P(漏抓) 从 0.4 推到 0.4^7 ≈ 0.0016 (99.84% catch rate)；N=3 catch rate 仅 93.6%，曾在重复重跑时观察到一次 flip，N=7 是稳健阈值。

旧 run 把这 12 条中的 10 条 (v3 / 12) + 2 条 (v4 / 12) 都判 SUCCESS（产物 size > 200 bytes、5 道门全通过），新 oracle 把它们翻为 FAILED——是 P12-A + P15-bug-fix 两轮设计意图兑现。详 [`docs/fixes/ror-gate6-fix-2026-05-11.md`](../../docs/fixes/ror-gate6-fix-2026-05-11.md)。

**E. 先前可疑的 `call_unshimmed_foreign_fn` 假说证伪**——审计中（`docs/fixes/oracle-leak-audit-2026-05-08.md`）曾怀疑 `miri-limit/ffi-unshimmed-extern/call_unshimmed_foreign_fn` 96ms 的快速 SUCCESS 是 ForeignMod silent skip 的体现。本 P12-B 重跑实测：该 entry 仍 SUCCESS（runner 57ms），gate 6 检查显示产物含 `Parameter getpid` + `Definition call_unshimmed_foreign_fn`——rocq-of-rust **正确**翻译了该 fn。原假说**证伪**：96ms 只是单文件解析的天然下限，不是空过物理证据。详 implementation log §2.3 末段。

合计 v4：21 + 3 + 1 + 12 = 37，与 v4 results.json 一致（其中 24 个 exit=101 来自 A+B 类，13 个 exit=1 来自 C+D 类）。v3：21 + 3 + 1 + 10 = 35。

## 与本次测试边界的关系

**6 道门 + N-attempt wrapper 抓 silent partial**：rocq-of-rust 几乎永远 exit 0，silent fallback 通过产物字面 marker 或 silent skip-item 表达。v4 实测（37 FAILED）：
- 24/37 FAILED 在 rustc 端就死（A+B 类），rocq-of-rust 没机会翻译——这些不在 rocq-of-rust 的"翻译能力"层面
- 1/37 FAILED（`repr/union`）落在第 5 道门 grep（C 类）——是真正的翻译能力边界触发（`(* Error Variant *)`）
- **12/37 FAILED 落在第 6 道门 entry_fn grep（D 类）**——silent skip-item，rocq-of-rust 把 fn item silently `vec![]` 但 exit 0 + 5 道门全通过；其中 10 个是 v3 的确定性 silent skip（top_level 分发问题），2 个是 v4 新抓的非确定性 silent skip（thread_local! 类）

124 个 SUCCESS 上 N-attempt（3 次）AND-reduce 后 marker grep 没命中任何 `Unexpected / Please report! / thir failed to compile / Unimplemented / Error` marker，且 entry_fn 都正确生成 `Definition` 三次——说明 rocq-of-rust 在它能接受 input + top-level 分发到位 + 翻译路径稳定的 entry 上**给出了包含 entry_fn 的完整、稳定翻译**。

**合理 skip 不算漏报**：rocq-of-rust 在 `top_level.rs:349-390` 对 `use` / `extern crate` / `macro_rules!` 直接返回 `vec![]`——这些不是 fn item，gate 6 不针对（`TS_ENTRY_FN` 永远是 fn 名）。如果 oracle 把这种 skip 当 silent partial，会大量误报合法代码。当前 5-marker 集 + entry_fn-level grep 只抓 rocq-of-rust 自己 emit 的 explicit failure comment 块或 silent skip 掉 fn item 这两类，避免误伤合理 skip 路径。

**单一 syntactic 通路覆盖大量在其他工具上分化的特性**：GAT、HRTB、`impl Trait` 返回、闭包返回、`unsafe` 指针操作、`Drop`、`#[repr]`（非 union）、并发原语（Arc / Mutex）、trait object——这些在多数工具上出现 SUCCESS 率分化的特性，rocq-of-rust 全部 SUCCESS。原因：所有这些构造统一翻成 `M.closure / M.borrow / Pointer.Kind.MutRef / M.get_trait_method "<trait_path>" "<method>"` 等 syntactic 节点，**不在翻译阶段做语义筛**——`&mut` 翻成字符串标签 `"MutRef"`、trait method 翻成字符串查表。这种设计意味着"工具接受这段 Rust"和"工具能在这段 Rust 上推 borrow 安全 / 解 trait"是分开的两件事，本测试只测前者。

每个 Definition 显式标记 `Admitted.`——这是 rocq-of-rust 设计上明确把"证明义务"留给 Rocq 端的人手写规约。本测试把它视为 rocq-of-rust 的"翻译完成"成功状态。

## 历史快照声明

本报告 v4 数字与归类锚定 commit `a8a76a4d` + nightly-2024-12-07 toolchain + **6 道门 + N-attempt wrapper oracle**（N=7，marker 集 = `Error / Unexpected / Please report! / thir failed to compile / Unimplemented` + entry_fn `Definition` 存在性检查 + 3 次产物 AND-reduce）。rocq-of-rust 升级（含新 silent fallback 路径、marker 文本变更、edition 默认透传、top-level 分发改动、翻译路径非确定性变化）后归类可能改写。

## 关键发现摘要

1. **v3 → v4 关键修复（2026-05-11，本 update）**：P15-impl 实施 ror 档 1 typecheck 自动化时，反向暴露档 0 在 `creusot-limit/thread-local-ref` / `lifetime/thread-local` 上的 gate 6 漏报。根因不是 grep 模式错——而是 rocq-of-rust 翻译 `thread_local!` 宏触发的 entry 时输出**非确定性**（≈80% 含 fn / ≈20% drop fn），单次 oracle 采样 SUCCESS / FAILED 随机切换。修复采用 wrapper-based N-attempt（N=7）AND-reduce，把 P(漏 fn)^3 推到几乎 0。详 [`docs/fixes/ror-gate6-fix-2026-05-11.md`](../../docs/fixes/ror-gate6-fix-2026-05-11.md)。
2. **v3 关键修复（2026-05-08，P12-A）**：新增 gate 6 entry_fn `Definition <fn>` 存在性 grep，抓 10 个 silent skip-item（确定性 top_level 分发问题），把通过率从 121/146 (82.9%) 降到 111/146 (76.0%)。
3. **v4 通过率（仅看共享 146 entry 子集）**：109/146 = 74.7%（v3 76.0%，−1.3pp）；全 161 entry 124/161 = 77.0%。
4. **审计启发实施暴露漏报第三案例**（前两：P12 verifast N≤40 / P13 hax-fstar 漏 mutual-rec）。本案再次验证"在实施新工具 / 新版本时反向比对老工具，是发现隐藏漏报的稳健启发"。
5. **0 误报论证不退化**：N-attempt wrapper 对确定性翻译路径（占 corpus 大头）的产物 byte-identical，3 次 AND-reduce 与 1 次结果相同，不引入误报。
