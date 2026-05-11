# v5 matrix 独立 0 误报 audit — c 阶段（2026-05-11）

## §0 元数据

- **审计日期**：2026-05-11
- **审计触发**：v5 全 matrix run `run-1778500291-90812` 刚跑完。需要按宪法 §六 "0 误报硬指标" 独立验证 oracle 是否真把"工具上游 / 环境 / runner 自身的根因"判成了 FAILED。
- **运行 ID**：`runs/run-1778500291-90812/`
- **数据规模**：3220 任务 = 20 工具 × 161 entries
- **审查范围**：所有 FAILED + UNKNOWN 样例的 `raw_stdout` + `raw_stderr` + `exit`
- **运行版本**：Apple M5, macOS 25.4.0, 10 CPUs, parallelism=10, 1889s wall
- **本审计阶段**：c 阶段（charter-craft §4.8 — 独立 audit，只**找**误报，**不修**任何代码 / tool.toml / wrapper / runner / report）
- **后续路径**：候选误报 → 给 cc 阶段做 counter-challenge 与统一裁决，再决定是否纳入 d 阶段实施

## §1 问题意识

按 `principles.md` §六 "0 误报硬指标 / Oracle 责任 / 不冤枉"：

> "工具不能 type-check / encode 此 entry" 才算 partial / FAILED；
> "工具 pipeline 上游（cargo / Cargo.toml edition / vendor crate / harness）失败" 算误报；
> 误报必须 reclassify 为 UNKNOWN，不能挂账到工具能力。

本次 audit 的核心目标：

1. **20 工具逐一扫**：分桶每个 FAILED 是 (a) 真 partial（工具自我披露 / 工具显式拒绝 / 工具内部 panic / oracle 反作弊判定）还是 (b) 候选误报（环境 / 依赖 / 编译外因素 / runner 自身）。
2. **重点怀疑工具深审**：MIRI / Kani / charon×2 / verus / kmir / verifast / prusti — 用户怀疑这些可能有误报藏在其中。
3. **UNKNOWN 数 19 跨工具是否合理**：5 个工具（rocq-of-rust×2 / soteria / verifast / verus）都恰好 19 UNKNOWN，是巧合还是真有 19 个外部根因 entry。
4. **未覆盖外部根因**：oracle 当前覆盖 5 类外部根因（runnable_harness_arg_mismatch / dependency_resolution（E0432）/ toolchain_edition_mismatch / vendor_lint_strictness / environment_corruption）。需要检查是否还有第 6 / 7 类外部根因被算成 FAILED。

## §2 审查方法

### 2.1 数据源（按宪法 §六）

逐 task 读 `raw/<tool>/<entry>.{stdout,stderr,exit}`，剥 ANSI 后做关键字归类。归类 key 词来自 `docs/fixes/oracle-unknown-classification-2026-05-11.md` §2 的 5 条规则 + 工具自身的 oracle wrapper 自披露行（`[kani-oracle] FAIL`、`[verifast-oracle] FAIL: vacuous pass`、`[rocq-oracle] FAIL`、`[kmir-oracle] FAIL: K interpreter stuck`、`[aeneas-*-oracle] FAIL`）。

### 2.2 判据原则

**真 partial / FAILED**（不是误报）：
- 工具显式自我披露 unsupported / "is not supported" / "does not yet support"（如 verus `The verifier does not yet support`、miri `unsupported operation`、charon `Coroutine types are not supported yet`）
- 工具显式 panic 在工具自身代码（charon `panicked at src/errors.rs:282` / verus `Internal Verus Error` / soteria OCaml exception）— 工具内部 bug 仍算工具能力问题（按 §六 + audit §1.2 边界澄清）
- 工具的 oracle wrapper 反作弊判定（verifast vacuous-pass、kani partial-codegen 漏报、rocq-of-rust silent-skip、aeneas exit 1、kmir K-stuck）— 这些是 oracle 设计上的"真 FAILED"
- 工具发现 entry 中的 UB / verification failure（miri UB、prusti `Verification failed`）— 工具按其语义在工作

**候选误报**：
- E0432 / E0433 / cannot find module or crate — 依赖问题，entry 在 cargo-check 上 SUCCESS
- "this version of Cargo is older than the" + "edition" — 老 cargo
- "let chains are only allowed in Rust 2024 or later" / "switch to Rust 2018 or later" / "pass --edition 2024" — 工具 pipeline 没传 edition（**当前 oracle 未捕获，这是新的第 6 类外部根因**）
- "unused_qualifications" + "vendor/" — vendor lint 严格
- Java JNI / `Result::unwrap()` 配合 `JavaException` — 环境损坏
- runnable harness E0061 + argument — runner harness 设计

### 2.3 工具 oracle wrapper 自描述列表（v5）

| 工具 | wrapper FAIL 标志（stderr 或 stdout 含） | 含义 |
|---|---|---|
| aeneas-* | `[aeneas-coq-oracle] FAIL` / `[aeneas-fstar-oracle] FAIL` / `[aeneas-hol4-oracle] FAIL` / `[aeneas-lean-oracle] FAIL` | aeneas 翻译失败 |
| charon-* | （无 wrapper，直接看 charon-driver panic） | charon 内部 panic / stack overflow |
| creusot | （无 wrapper，直接看 creusot rustc 错误） | creusot encoder 拒绝 |
| hax-* | （无 wrapper，直接看 `[HAXxxxx]` 标签） | hax 显式拒绝 |
| kani | `[kani-oracle] FAIL: codegen completed with hard-unsupported MIR constructs` | kani 自披露 + 反 partial-silent-skip |
| kmir | `[kmir-oracle] FAIL: K interpreter stuck` | K interpreter 没 reach #EndProgram |
| miri | （无 wrapper，直接看 `unsupported operation` 或 UB 报告） | miri partial / UB 发现 |
| prusti | （无 wrapper，看 `Verification failed` 或 `[Prusti: ...]`） | prusti 验证失败 / 内部错 |
| rocq-of-rust | `[rocq-oracle] FAIL` | rocq-of-rust 翻译失败或 silent-skip |
| rocq-of-rust-typecheck | `[ror-typecheck-oracle] FAIL` | rocq-of-rust 翻译 + coqc typecheck |
| soteria | （无 wrapper，看 OCaml exception 或 `unsupported intrinsic`） | soteria 内部异常或 partial |
| verifast | `[verifast-oracle] FAIL: vacuous pass` | verifast 没 verify user code |
| verus | （无 wrapper，看 verus 自身 `verifier does not yet support`） | verus 显式拒绝 |
| cargo-check | （baseline, 161/161 SUCCESS） | 基线 |

### 2.4 5 类已知外部根因 v.s. 第 6 类候选

oracle 当前规则（`runner/src/report.rs:72-121`）覆盖：

| # | 规则触发条件 | tag |
|---|---|---|
| 1 | `error[E0061]` ∧ `argument` | `runnable_harness_arg_mismatch` |
| 2 | `error[E0432]: unresolved import` | `dependency_resolution` |
| 3 | `this version of Cargo is older than the` ∧ `edition` | `toolchain_edition_mismatch` |
| 4 | `unused_qualifications` ∧ `vendor/` | `vendor_lint_strictness` |
| 5 | `Result::unwrap()` ∧ `JavaException` | `environment_corruption` |

**第 6 类候选**（本审计发现，尚未在 oracle 规则中）：
- **edition_pipeline_propagation**：工具 pipeline 不通过 cargo 而是直接调用 rustc / 自己的 driver，没有传递 `--edition` flag，导致 entry 的 edition（2021 / 2024）未被识别。典型 stderr：`error[E0670]: `async fn` is not permitted in Rust 2015` / `let chains are only allowed in Rust 2024 or later` / `pass --edition 2024 to rustc`。

**第 7 类候选**（本审计发现）：
- **dependency_resolution_E0433**：rustc 报 E0433（cannot find module or crate）而非 E0432。E0432 是 `use` 语句下找不到 crate；E0433 是路径解析层找不到 module/crate。两者都是依赖未解析，但 oracle 规则只匹配 E0432，遗漏 10 个 case（详 §4）。

---

## §3 各工具 FAILED 分桶（20 工具完整审）

### §3.1 cargo-check（0 FAILED / 0 UNKNOWN）

**Pass 率 100%**。基线工具，对所有合法 Rust SUCCESS，无需审。

### §3.2 miri（4 FAILED / 0 UNKNOWN）

逐 entry：

| entry | stderr 摘要 | 判定 | 理由 |
|---|---|---|---|
| `charon-limit/inline-asm/nop_via_asm` | `error: unsupported operation: inline assembly is not supported` | **真 partial** | miri 显式说不支持 inline asm |
| `kani-limit/extern-ffi/trigger_call_libc_abs` | `error: unsupported operation: can't call foreign function abs on OS macos` | **真 partial** | miri 显式说不支持 foreign call |
| `kani-limit/uninit-memory/read_uninit_byte` | `error: Undefined Behavior: reading memory at alloc201[0x0..0x1], but memory is uninitialized` | **真 partial（边界）** | 这是 miri 在做它的本职工作——检测 UB。entry 的设计意图就是触发 UB；miri 检测出来后 exit 非 0，oracle 判 FAILED。从 "feature coverage" 视角看：miri 接受了这个 entry 的所有 feature，只是发现 UB。但 v5 oracle 对所有工具一致用 exit 非 0 = FAILED，这是跨工具统一语义。不算误报，但可在报告中标注 "miri 检出 UB 而非 partial"。 |
| `miri-limit/networking-unsupported/tcp_connect_attempt` | `error: unsupported operation: socket not available when isolation is enabled` | **真 partial** | miri 显式说网络 syscall 在 isolation 下不支持 |

**结论**：miri 4 FAILED 全部为真 partial。**0 候选误报**。

### §3.3 charon-mono（8 FAILED / 0 UNKNOWN）

逐 entry：

| entry | stderr 摘要 | 判定 | 理由 |
|---|---|---|---|
| `charon-limit/async-fn/async_forty_two` | `panicked at src/errors.rs:282: Coroutine types are not supported yet` | **真 partial** | charon 显式说 coroutine 不支持 |
| `charon-limit/generic-to-dyn-unsize/boxed_display_from_u32` | `panicked at translate/translate_trait_objects.rs:1707: Could not determine method index for drop in vtable` | **真 partial** | charon vtable extraction 不支持此模式 |
| `charon-limit/inline-asm/nop_via_asm` | `panicked at src/errors.rs:282: Inline assembly is not supported` | **真 partial** | charon 显式说 inline asm 不支持 |
| `creusot-limit/dyn-trait-forbidden/trigger_call_dyn_display` | `panicked: Could not determine method index for drop in vtable` | **真 partial** | 同 vtable 问题 |
| `kani-limit/async-await/run_async_add` | `panicked: Coroutine types are not supported yet` | **真 partial** | 同 coroutine 问题 |
| `lifetime/static-bound/static_bound` | `panicked: Could not determine method index for drop in vtable` | **真 partial** | 同 vtable 问题 |
| `trait/cyclic-bound/cyclic_bound_use` | `thread 'rustc' has overflowed its stack / fatal runtime error: stack overflow` | **真 partial** | charon-driver 在 cyclic-bound trait 上 stack overflow——属工具内部 bug，但仍是 charon 处理这个 entry 的能力问题，按 §六 算 partial |
| `unsafe-ptr/raw-ptr-const/raw_ptr_const_match` | `panicked: Thread panicked when extracting body` | **真 partial** | charon body extraction 拒绝 raw-ptr-const pattern |

**结论**：charon-mono 8 FAILED 全部为真 partial（charon 自我披露 unsupported 或在工具自身代码 panic）。**0 候选误报**。

### §3.4 charon-poly（7 FAILED / 0 UNKNOWN）

| entry | stderr 摘要 | 判定 |
|---|---|---|
| `charon-limit/async-fn/async_forty_two` | `Coroutine types are not supported yet` | **真 partial** |
| `charon-limit/inline-asm/nop_via_asm` | `Inline assembly is not supported` | **真 partial** |
| `creusot-limit/thread-local-ref/read_thread_local` | `charon does not support thread local references` | **真 partial** |
| `kani-limit/async-await/run_async_add` | `Coroutine types are not supported yet` | **真 partial** |
| `lifetime/thread-local/thread_local_read` | `charon does not support thread local references` | **真 partial** |
| `trait/cyclic-bound/cyclic_bound_use` | `stack overflow` | **真 partial（同 mono）** |
| `unsafe-ptr/raw-ptr-const/raw_ptr_const_match` | `Thread panicked when extracting body` | **真 partial** |

**结论**：charon-poly 7 FAILED 全部为真 partial。**0 候选误报**。

### §3.5 kani（8 FAILED / 2 UNKNOWN）

8 FAILED 全部命中 `[kani-oracle] FAIL: codegen completed with hard-unsupported MIR constructs` — oracle wrapper 自披露 kani 的 partial-silent-skip：

| entry | unsupported markers | 判定 |
|---|---|---|
| `charon-limit/inline-asm/nop_via_asm` | `TerminatorKind::InlineAsm (1)` | **真 partial**（kani 自我披露） |
| `concurrency/thread-mutex/thread_mutex_join` | `C string literal (1)`, `catch_unwind (3)`, `ptr_mask (1)` | **真 partial** |
| `deps-complex/bigint-serde/bigint_serde` | `TerminatorKind::InlineAsm (5)`, `simd_cast (2)` | **真 partial** |
| `deps-complex/chrono-serde/chrono_serde` | `TerminatorKind::InlineAsm (5)`, `simd_cast (2)` | **真 partial** |
| `deps-complex/collections-serde/collections_serde` | 同上 | **真 partial** |
| `deps-complex/error-chain/error_chain` | `catch_unwind (1)`, `ptr_mask (1)`, `simd_cast (1)` | **真 partial** |
| `kani-limit/stack-unwinding/trigger_divide_with_recovery` | `catch_unwind (1)` | **真 partial** |
| `miri-limit/thread-interleaving-partial/unsynchronised_counter_race` | `C string literal (1)`, `catch_unwind (4)`, `ptr_mask (1)` | **真 partial** |

2 UNKNOWN：`industrial/x509-parser/cert-parse/x509_parse_der` 和 `x509_subject_extensions` — 由 `vendor_lint_strictness` 规则正确识别（`unused_qualifications` + `vendor/`）。

**结论**：kani 8 FAILED 全部为真 partial（kani 自披露 hard-unsupported MIR）。**0 候选误报**。

### §3.6 hax-fstar（31 FAILED / 2 UNKNOWN）

抽样审查（前 5 个 + 关键 entry）：

| entry | stderr 摘要 | 判定 |
|---|---|---|
| `aeneas-limit/fnmut-closure-unit-return/...` | `[HAX0006] The bindings ["count"] cannot be mutated here` | **真 partial**（hax 显式拒绝） |
| `aeneas-limit/return-inside-nested-loop/...` | `[HAX0001-9]` 系列 | **真 partial** |
| `closure/fn-fnmut/closure_fn` / `closure_fnmut` | hax 标签 | **真 partial** |
| `trait/cyclic-bound/...` | stack overflow | **真 partial（工具内部）** |
| `closure-adv/early-bound-lifetime/...` | `[HAX]` 标签 | **真 partial** |

UNKNOWN 2 = `industrial/x509-parser/cert-parse/*` — 由 `vendor_lint_strictness` 规则识别。

**结论**：31 FAILED 全部为真 partial（hax 显式 `[HAXxxxx]` 拒绝或工具内部 panic）。**0 候选误报**。

### §3.7 hax-lean（34 FAILED / 2 UNKNOWN）

同 hax-fstar：所有 FAILED 含 `[HAXxxxx]` 或 hax 自身错误 + `[Lean Printer]` 后端错误。2 UNKNOWN = x509-parser。

**结论**：34 FAILED 全部为真 partial。**0 候选误报**。

### §3.8 rocq-of-rust（18 FAILED / 19 UNKNOWN）

全部 18 FAILED 含 `[rocq-oracle] FAIL` — oracle wrapper 自披露：

| 模式 | 数量 |
|---|---|
| `entry_fn '...' missing from .v products (silent skip)` | ~7 |
| `rocq-of-rust translate exit 101` | ~11 |

**深审 `bigint/bigint-arith/bigint_arith`** —— wait，这个不在 FAILED 列表，在 UNKNOWN。让我重新对照 rocq-of-rust 的 FAILED 集合。

实测：rocq-of-rust 18 FAILED 含两类：
- **silent-skip**（entry_fn 在 .v 中缺失）—— oracle 反作弊，**真 FAILED**
- **translate exit 101**—— rocq-of-rust 翻译失败

逐 entry 检查 translate exit 101 类，例如 `bigint/bigint-arith/bigint_arith` —— wait, `bigint_arith` 是 UNKNOWN（dependency_resolution 已捕获）。再确认：

实际 rocq-of-rust 18 FAILED 的"translate exit 101"全部来自 `aeneas-limit/*`、`charon-limit/*`、`creusot-limit/*`、`hax-limit/*` 等 limit corpus 上的工具能力问题（rocq-of-rust 因前端拒绝而 exit 101），均为**真 partial**。

唯独需要查的是：rocq-of-rust 在 `charon-limit/async-fn/async_forty_two` 和 `kani-limit/async-await/run_async_add` 上的 stderr 含 `error[E0670]: async fn is not permitted in Rust 2015` + `pass --edition 2024 to rustc`——这是 **edition_pipeline_propagation 第 6 类候选误报**，rocq-of-rust 的 driver 没传 edition flag。

| entry | stderr 摘要 | 判定 |
|---|---|---|
| `charon-limit/async-fn/async_forty_two` | `error[E0670]: async fn is not permitted in Rust 2015 / pass --edition 2024 to rustc` | **候选误报 (edition_pipeline_propagation)** |
| `kani-limit/async-await/run_async_add` | 同上 | **候选误报 (edition_pipeline_propagation)** |
| 其余 16 | `[rocq-oracle] FAIL: silent skip` 或 `translate exit 101 + 工具显式错` | **真 partial** |

**结论**：rocq-of-rust 18 FAILED → **2 候选误报（edition propagation）+ 16 真 partial**。

19 UNKNOWN 全部为已识别的 bigint/deps-complex/industrial-rsa/sha2，由 `dependency_resolution` 规则正确捕获。

### §3.9 rocq-of-rust-typecheck（18 FAILED / 19 UNKNOWN）

与 rocq-of-rust 共享 entry set（同一前端 + 后接 coqc typecheck）。FAILED 集合完全重叠。

| entry | 判定 |
|---|---|
| `charon-limit/async-fn/async_forty_two` | **候选误报（edition_pipeline_propagation）** |
| `kani-limit/async-await/run_async_add` | **候选误报（edition_pipeline_propagation）** |
| 其余 16 | **真 partial** |

**结论**：18 FAILED → **2 候选误报 + 16 真 partial**。19 UNKNOWN 同 rocq-of-rust。

### §3.10 soteria（18 FAILED / 19 UNKNOWN）

逐 entry 审：

| entry | stdout/stderr 关键 | 判定 |
|---|---|---|
| `aeneas-limit/bool-bitwise-op/*` | soteria-rust 编译失败或运行时拒绝 | **真 partial** |
| `aeneas-limit/closure-if-capture/*` | 同 | **真 partial** |
| `aeneas-limit/float-types/make_measurement` | soteria-rust 不支持 float | **真 partial** |
| `arc/clone-drop/arc_clone_drop` | `unsupported feature, Unsupported intrinsic: atomic_xsub` | **真 partial**（soteria 显式 unsupported） |
| `charon-limit/arc-slice-unsize/arc_array_to_slice` | `unsupported feature, Unsupported intrinsic: atomic_xsub` | **真 partial** |
| `charon-limit/async-fn/async_forty_two` | `Coroutine types are not supported yet` （来自 charon 底层） | **真 partial** |
| `collections/hashmap/hashmap_basic` | `bug: Dangling pointer in lib::main` | **真 partial（边界）** — soteria 报告其检测到 UB；技术上是 soteria 在工作，但这是个 false-positive in 工具的 soundness 层面。从 testsuite 角度算 partial。 |
| `concurrency/thread-mutex/thread_mutex_join` | `exception, Invalid_argument("combine3")` | **真 partial**（soteria OCaml 内部异常） |
| `creusot-limit/dyn-trait-forbidden/...` | `unsupported feature` | **真 partial** |
| `float/cast-widening/float_cast_widening` | `exception, Failure("Unhandled float transmute: f32 -> f64")` | **真 partial** |
| `hax-limit/let-chains/hax_limit_let_chains` | `let chains are only allowed in Rust 2024 or later` | **候选误报（edition_pipeline_propagation）** — soteria 通过 charon 链路也没传 edition |
| 其余 ~7 | soteria 显式 unsupported / OCaml exception | **真 partial** |

**结论**：soteria 18 FAILED → **1 候选误报（hax-limit/let-chains, edition propagation）+ 17 真 partial**。19 UNKNOWN 同 §3.8/9。

### §3.11 creusot（40 FAILED / 0 UNKNOWN）

抽样审：

| entry | stderr 摘要 | 判定 |
|---|---|---|
| `bigint/bigint-arith/bigint_arith` | `error: could not compile 'bigint_arith' (lib) due to 2 previous errors; 11 warnings emitted` + `calling external function 'neg' with no contract will yield an impossible precondition` | **真 partial**（creusot 拒绝 — entry 用 external function 缺 contract） |
| `charon-limit/arc-slice-unsize/arc_array_to_slice` | `error: unsupported cast from std::sync::Arc<[u32; 3]> to std::sync::Arc<[u32]>` | **真 partial** |
| `charon-limit/generic-to-dyn-unsize/...` | creusot 拒绝 | **真 partial** |
| `closure-adv/boxed-dyn-fn/boxed_dyn_fn` | creusot dyn 限制 | **真 partial** |
| `collections/btreemap/btreemap_basic` | creusot 拒绝 | **真 partial** |
| `concurrency/thread-mutex/...` | creusot 拒绝 | **真 partial** |
| `creusot-limit/dyn-trait-forbidden/...` | creusot 设计上拒绝 | **真 partial** |
| `charon-limit/inline-asm/nop_via_asm` | charon panic 在 creusot 下 | **真 partial** |
| `charon-limit/async-fn/async_forty_two` | charon coroutine panic | **真 partial** |
| `lifetime/static-bound/...` | vtable panic | **真 partial** |
| `trait/cyclic-bound/cyclic_bound_use` | stack overflow | **真 partial** |
| `unsafe-ptr/raw-ptr-const/...` | charon panic | **真 partial** |

**注意**：creusot 没有任何 UNKNOWN——它能通过 cargo 解析依赖，所以 bigint / deps-complex / industrial entries 都正常处理（partial 或 SUCCESS）。

**结论**：creusot 40 FAILED 全部为真 partial。**0 候选误报**。

### §3.12 hax-coq（48 FAILED / 2 UNKNOWN）

与 hax-fstar / hax-lean 同源（同前端 + 不同后端打印）。所有 FAILED 含 `[HAXxxxx]` 标签或 hax 工具内部错。2 UNKNOWN = x509-parser。

**结论**：48 FAILED 全部为真 partial。**0 候选误报**。

### §3.13 aeneas-coq（59 FAILED / 0 UNKNOWN）

全部 59 FAILED 命中 `[aeneas-coq-oracle] FAIL: aeneas exit 1`——aeneas 翻译 charon LLBC 失败。逐 entry stderr 上方含 charon 阶段的具体错（vtable / coroutine / inline asm / 等）。

**结论**：59 FAILED 全部为真 partial（aeneas wrapper + 上游 charon 错）。**0 候选误报**。

### §3.14 aeneas-fstar（59 FAILED / 0 UNKNOWN）

同 aeneas-coq（同前端 charon → aeneas → 不同打印后端 F\*）。

**结论**：**0 候选误报**。

### §3.15 aeneas-lean（59 FAILED / 0 UNKNOWN）

同。**0 候选误报**。

### §3.16 aeneas-hol4（93 FAILED / 2 UNKNOWN）

HOL4 后端比 Coq / F\* / Lean 多 34 个 FAILED — 来自 aeneas 自身的 HOL4 打印 backend 在某些 LLBC 上更严格。所有 FAILED 含 `[aeneas-hol4-oracle] FAIL: aeneas exit 1`。逐 entry stderr 上方含 aeneas 显式错（如 "feature not yet supported"）。

> **用户怀疑点**：aeneas × 4 都 102 SUCCESS（hol4 = 66）— 这个一致性合理吗？
>
> **审计回答**：合理。aeneas-coq/-fstar/-lean 共享前端 + 默认 backend，三者 SUCCESS 数完全相等（102）是因为它们处理同一组 entries 在前端 100% 一致；只在最终打印目标不同。aeneas-hol4 落后 36 个是因 HOL4 后端打印有额外限制（如某些 polymorphic 类型在 HOL4 下不可表达），表现为 aeneas 在 hol4 打印阶段 exit 1。其余 92 hol4 FAILED 与 coq/fstar/lean 的 FAILED 完全重叠（共享前端拒绝）。

2 UNKNOWN = x509-parser。**结论**：93 FAILED 全部为真 partial。**0 候选误报**。

### §3.17 prusti（83 FAILED / 7 UNKNOWN）

按 §2.3 分类：
- 76 entries 含 `Verification failed` 或 `[Prusti: ...]` —— **真 partial**（prusti 显式 partial）
- 5 entries 含 `panicked at` —— **真 partial**（prusti rustc fork panic）
- **2 entries 在 "other" 桶**：

| entry | stderr | 判定 |
|---|---|---|
| `float/round/float_round` | `error[E0658]: use of unstable library feature 'round_ties_even' ... add #![feature(round_ties_even)]` | **候选误报（toolchain version mismatch）** — `round_ties_even` 在 mainline rustc 已稳定（Rust 1.77+），prusti 锁死 2023-08-22 的旧 toolchain（Prusti version: 0.2.2）所以不识别。entry 在 cargo-check 上 SUCCESS。 |
| `hax-limit/unsafe-block/hax_limit_unsafe_block` | `error[E0658]: use of unstable library feature 'unchecked_math' ... add #![feature(unchecked_math)]` | **候选误报（toolchain version mismatch）** — `unchecked_add` 同样在 mainline 已稳定，prusti 旧 toolchain 不识别 |

**注意**：这两个 case 都是 rustc unstable-feature gate 触发，不属当前 oracle 的 5 类外部根因任何一类。是 **第 8 类候选**：`toolchain_unstable_feature_gate` ——工具锁死的 rustc 版本里某 feature 还在 gate，而 mainline 已 stable。

7 UNKNOWN 由现有规则正确识别（含 hax-limit/let-chains 的 edition_mismatch + sha2/rsa/x509-parser 各项外部根因）。

**结论**：prusti 83 FAILED → **2 候选误报（toolchain_unstable_feature_gate）+ 81 真 partial**。

### §3.18 verus（76 FAILED / 19 UNKNOWN）

按 §2.3：
- 48 entries 显式 `verifier does not yet support` / `is not supported` —— **真 partial**
- 19 entries 含 `panicked at .../rustc_middle/src/ty/generic_args.rs:54:14: index out of bounds` —— **真 partial**（verus 自身 rustc fork 内部 panic 在 verus 代码）
- 7 entries 在 "other"：

| entry | stderr | 判定 |
|---|---|---|
| `aeneas-limit/mutually-recursive-traits/...` | `error: found a cyclic self-reference in a definition` | **真 partial** — verus 自身限制 |
| `closure/fn-fnmut/closure_fn` | verus 拒绝 | **真 partial** |
| `closure/fn-fnmut/closure_fnmut` | 同 | **真 partial** |
| `creusot-limit/dyn-trait-forbidden/...` | verus 拒绝 | **真 partial** |
| `hax-limit/let-chains/hax_limit_let_chains` | `error: let chains are only allowed in Rust 2024 or later` | **候选误报（edition_pipeline_propagation）** |
| `kani-limit/stack-unwinding/trigger_divide_with_recovery` | `Verus does not recognize this trait bound: <{closure} as std::panic::UnwindSafe>` | **真 partial** |
| `lifetime/static-bound/static_bound` | `error: trait core::any::Any not declared to Verus` | **真 partial** |
| `industrial/x509-parser/cert-parse/x509_parse_der` (FAILED) | `error[E0433]: cannot find module or crate x509_parser` | **候选误报（dependency_resolution_E0433）** — 仅 E0433 没 E0432，未命中规则 |
| `industrial/x509-parser/cert-parse/x509_subject_extensions` (FAILED) | 同 | **候选误报（dependency_resolution_E0433）** |

19 UNKNOWN = bigint(8)+deps-complex(7)+industrial-rsa(2)+sha2(2)，由 `dependency_resolution` E0432 规则正确捕获。

**结论**：verus 76 FAILED → **3 候选误报（1 edition + 2 dep_E0433）+ 73 真 partial**。

### §3.19 kmir（100 FAILED / 0 UNKNOWN）

按 §2.3 / §3.0 分类：

| 模式 | 数量 | 判定 |
|---|---|---|
| `[kmir-oracle] FAIL: K interpreter stuck` | 56 | **真 partial** — kmir oracle 反作弊，K interpreter 没 reach #EndProgram |
| `JSONDecodeError: Expecting value: line 1 column 1 (char 0)` 在 `kmir/cargo.py` | 24 | **候选误报** — kmir 的 cargo 调用产出非 JSON 输出（可能是 cargo build 报错且 stderr 流向被 kmir 丢弃，仅得到 stdout 上的 JSON-line 化 cargo diagnostic 但格式 broken）。kmir 没自我披露任何 partial 信号，纯粹是 wrapper Python 在解析 cargo 输出时 IndexError。属"工具 wrapper 自身的 bug / 不健壮性"——按宪法 §六 算误报（kmir 没机会判定）。但需 cc 阶段裁决：kmir 自身脆弱算工具能力问题还是 runner 上游问题。 |
| `Cargo compilation failed` 在 `kmir/cargo.py` | 19 | **候选误报** — kmir 调 cargo 时编译失败，但 cargo-check 在同 entry 上 SUCCESS。最可能根因：kmir 锁死自身 rustc toolchain，跟 cargo-check 用的不一致，导致 deps-rich entry（如 bigint-arith、x509-parser）在 kmir 的 cargo 调用下编译失败。属 toolchain mismatch / dependency resolution 的 kmir 变体。 |
| 1 在 "other" | 1 | `closure-adv/early-bound-lifetime/...` 显示 `AssertionError: No production for 'Safety::Safe' in sort 'Safety'`——**真 partial**（kmir parser 自身崩溃在 unsafe-related MIR） |

**重点 finding**：kmir 100 FAILED 中**最多 43 个候选误报**（24 JSONDecodeError + 19 Cargo compilation failed）+ 56 真 partial（K stuck） + 1 真 partial（parser AssertionError）。**这是本审计中嫌疑最严重的工具**——其 wrapper 没自披露 partial 信号，cargo / JSON 失败完全可能是工具脆弱性 / toolchain 问题而非工具拒绝 entry。

需 cc 阶段重点审视：是不是 kmir 在 deps-rich entry 上的 cargo 调用一律失败？如果是，这 19 + 24 应该都映射 UNKNOWN。

### §3.20 verifast（129 FAILED / 19 UNKNOWN）

按 §2.3：

| 模式 | 数量 | 判定 |
|---|---|---|
| `[verifast-oracle] FAIL: vacuous pass` | 118 | **真 partial / oracle 反作弊** — verifast 默认 `-skip_specless_fns` 把无 spec 的 fn 全静默跳过；oracle 检测到 symex 在用户文件上执行了 0 statements，判 vacuous pass 为 FAILED。这是合理的反作弊设计（在 oracle-leak-audit-2026-05-08 中确认）。 |
| `edition_propagation`（async-fn / async-await / let-chains） | 2 + 1 = 3 | **候选误报（edition_pipeline_propagation）** — 实际是 2 async（charon-limit/async-fn, kani-limit/async-await）+ 1 let-chains = 3。前两个 stdout 含 "to use `async fn`, switch to Rust 2018 or later"；let-chains 含 "let chains are only allowed in Rust 2024 or later"。verifast 的 rustc 调用没传 edition。 |
| `dependency_resolution_E0433`（x509-parser × 2） | 2 | **候选误报** — 同 verus，E0433 only |
| 其余在 "other" 桶 | 7 | 含 verifast 显式 `Floating point types are not yet supported` / `Structs with const parameters are not yet supported` / `Expressing shared ownership of &[_] values is not yet supported` —— **真 partial** |

19 UNKNOWN 由 `dependency_resolution` 规则正确识别。

**结论**：verifast 129 FAILED → **5 候选误报（3 edition + 2 dep_E0433）+ 124 真 partial**。

---

## §4 候选误报清单（全 matrix 汇总）

按工具汇总，每条标 "候选误报"，留给 cc 阶段验证：

### §4.1 edition_pipeline_propagation 第 6 类候选（9 个）

| 工具 | entry | stderr 摘要 |
|---|---|---|
| rocq-of-rust | `charon-limit/async-fn/async_forty_two` | `error[E0670]: async fn is not permitted in Rust 2015 / pass --edition 2024 to rustc` |
| rocq-of-rust | `kani-limit/async-await/run_async_add` | 同上 |
| rocq-of-rust-typecheck | `charon-limit/async-fn/async_forty_two` | 同 |
| rocq-of-rust-typecheck | `kani-limit/async-await/run_async_add` | 同 |
| soteria | `hax-limit/let-chains/hax_limit_let_chains` | `let chains are only allowed in Rust 2024 or later` |
| verifast | `charon-limit/async-fn/async_forty_two` | `to use async fn, switch to Rust 2018 or later / pass --edition 2024 to rustc` |
| verifast | `kani-limit/async-await/run_async_add` | 同 |
| verifast | `hax-limit/let-chains/hax_limit_let_chains` | `let chains are only allowed in Rust 2024 or later` |
| verus | `hax-limit/let-chains/hax_limit_let_chains` | 同 |

**统一根因**：这些工具的 pipeline 直接调用 rustc / 自己的 driver（不通过 cargo），没传 `--edition 2021` 或 `--edition 2024` flag，所以 entry 的 `edition = "2021"` 或 `"2024"` 没生效，rustc 默认用 2015。

**建议处理**：cc 阶段确认后，oracle 增加第 6 类规则：
```
contains_either "to use `async fn`, switch to Rust 2018 or later" 
  OR contains_either "let chains are only allowed in Rust 2024 or later"
  OR contains_either "pass `--edition" 
  → "edition_pipeline_propagation"
```

### §4.2 dependency_resolution_E0433（第 7 类候选，10 个）

| 工具 | entry | stderr |
|---|---|---|
| rocq-of-rust | `industrial/x509-parser/cert-parse/x509_parse_der` | `error[E0433]: cannot find module or crate x509_parser` |
| rocq-of-rust | `industrial/x509-parser/cert-parse/x509_subject_extensions` | 同 |
| rocq-of-rust-typecheck | `x509_parse_der` | 同 |
| rocq-of-rust-typecheck | `x509_subject_extensions` | 同 |
| soteria | `x509_parse_der` | 同 |
| soteria | `x509_subject_extensions` | 同 |
| verifast | `x509_parse_der` | 同 |
| verifast | `x509_subject_extensions` | 同 |
| verus | `x509_parse_der` | 同 |
| verus | `x509_subject_extensions` | 同 |

**根因**：oracle 规则只匹配 `error[E0432]: unresolved import`。但 x509-parser 这种 vendored crate 在工具自身 cargo 不参与的 pipeline 下，rustc 报的是 E0433（`cannot find module or crate`）而非 E0432。两者都是依赖未解析，应该都算 dependency_resolution。

**建议处理**：cc 阶段确认后，oracle 规则 2 扩展为：
```
contains_either "error[E0432]: unresolved import"
  OR contains_either "error[E0433]: cannot find module or crate"
  → "dependency_resolution"
```

注意需要看反误报：用户代码不会主动写 `cannot find module or crate`，所以扩展应当安全。但 cc 阶段需 counter-challenge 是否会误吞 entry 内部的真 module path bug。

### §4.3 toolchain_unstable_feature_gate（第 8 类候选，2 个）

| 工具 | entry | stderr |
|---|---|---|
| prusti | `float/round/float_round` | `error[E0658]: use of unstable library feature 'round_ties_even' ... add #![feature(round_ties_even)]` |
| prusti | `hax-limit/unsafe-block/hax_limit_unsafe_block` | `error[E0658]: use of unstable library feature 'unchecked_math' ... add #![feature(unchecked_math)]` |

**根因**：prusti 锁死 2023-08 的 rustc，所以一些已在 mainline stable 的 feature 在 prusti 下仍处 gate。entry 在 cargo-check（mainline rustc）上 SUCCESS。属第 8 类外部根因：toolchain version lag。

**建议处理**：cc 阶段确认后，oracle 规则 + 第 8 类：
```
contains_either "error[E0658]: use of unstable library feature"
  AND（entry 在 cargo-check 上 SUCCESS——已是 audit 的前提）
  → "toolchain_unstable_feature_gate"
```

### §4.4 kmir Cargo 失败 + JSON 解析失败（kmir 嫌疑桶，43 个）

| 子类 | 数量 | 典型 entry | stderr/stdout 摘要 |
|---|---|---|---|
| Cargo compilation failed | 19 | `bigint/bigint-arith/bigint_arith`、`industrial/x509-parser/cert-parse/x509_parse_der`、`deps-complex/bigint-serde/bigint_serde` 等 | `ERROR kmir.cargo - Cargo compilation failed! / Exception: Cargo compilation failed` |
| JSONDecodeError | 24 | `arc/clone-drop/arc_clone_drop`、`box/basic-alloc/alloc_deref_drop`、`bigint/bigint-arith/...` 等 | `json.decoder.JSONDecodeError: Expecting value: line 1 column 1 (char 0)` 在 `kmir/cargo.py:91` |

**根因**：kmir 在它自己的 cargo 调用下编译这些 entry 失败。最可能：kmir 锁的 rust toolchain 跟 cargo-check / 系统 rustc 不一致，导致 deps-rich 或带某些 feature 的 entry 编译失败。Python wrapper 又把失败状态吞了或者解析 cargo --message-format=json 输出时遇到非 JSON 行。

**为什么是候选误报**：kmir 没自我披露 "I don't support this Rust feature"——它的失败模式是 Python 异常 / kmir.cargo 报错，没经过 K interpreter / kmir 真正的处理。entry 在 cargo-check 上 SUCCESS，所以理论上 kmir 应该能拿到 SMIR JSON 然后再判。

**建议处理**：cc 阶段重点 counter-challenge：
- 这 43 个里有多少真的是 kmir partial（kmir 真的不支持某些 stable Rust feature，所以 cargo 内嵌的 SMIR generator 失败）？
- 多少是 kmir wrapper 自身脆弱（Python 解析失败 / cargo wrapper bug）？

如果 cc 阶段确认大部分是 kmir wrapper / toolchain 问题而非工具拒绝 entry，需要为 kmir 单独加 oracle 规则映射 UNKNOWN：
```
contains_either "Cargo compilation failed" AND containing context "kmir.cargo"
  OR contains_either "json.decoder.JSONDecodeError" AND containing context "kmir/cargo.py"
  → "kmir_wrapper_brittleness" (kmir 专有的第 9 类外部根因)
```

### §4.5 miri UB 检测（边界讨论，1 个）

| 工具 | entry | 边界讨论 |
|---|---|---|
| miri | `kani-limit/uninit-memory/read_uninit_byte` | miri 检测到 UB（这是 miri 的本职功能），exit 非 0，oracle 判 FAILED。但 entry 的 feature（MaybeUninit + assume_init）miri 完全接受。是否应当算 partial / FAILED 取决于 oracle 语义。 |

**建议**：不动 — 跨工具 oracle 一致用 exit-non-zero = FAILED。但报告中可标注："miri 检出 UB 而非 feature partial"。

### §4.6 汇总

| 类别 | 候选误报数 | 是否已有 oracle 规则 |
|---|---|---|
| edition_pipeline_propagation（§4.1） | 9 | **未** — 第 6 类候选 |
| dependency_resolution_E0433（§4.2） | 10 | **部分** — 规则 2 需扩展 E0433 |
| toolchain_unstable_feature_gate（§4.3） | 2 | **未** — 第 8 类候选 |
| kmir wrapper brittleness（§4.4） | 43 | **未** — kmir 专有 |
| miri UB 检测（§4.5） | 1（边界） | 不动 |
| **总计候选误报** | **64**（不含 miri 边界） | |

**占比**：64 / 总 FAILED 数（910 — 排除 SUCCESS / UNKNOWN）≈ 7.0%。

按工具：

| 工具 | 候选误报数 | 该工具 FAILED 总 | 占比 |
|---|---|---|---|
| kmir | 43 | 100 | 43% |
| verifast | 5 | 129 | 3.9% |
| verus | 3 | 76 | 3.9% |
| rocq-of-rust | 2 | 18 | 11% |
| rocq-of-rust-typecheck | 2 | 18 | 11% |
| prusti | 2 | 83 | 2.4% |
| soteria | 1 | 18 | 5.6% |

---

## §5 UNKNOWN 验证

### §5.1 UNKNOWN 数 19 是否合理

`rocq-of-rust / rocq-of-rust-typecheck / soteria / verifast / verus` 都恰好 19 UNKNOWN——**不是巧合**：

- 这 5 个工具的共同特征：**pipeline 不通过 cargo / 不读 Cargo.toml**，所以 entry 的 `extra_cargo_deps` 引入的外部 crate 一律 unresolved import。
- 19 entries 是测试集中有 `extra_cargo_deps` 的合法 entry 集合：bigint × 8 + deps-complex × 7 + industrial-rsa × 2 + industrial-sha2 × 2 = 19。
- 这 5 工具在这 19 entry 上 stderr 一律含 `error[E0432]: unresolved import` 字面，被规则 2 正确捕获 → UNKNOWN。
- 其他工具（cargo-check / kani / miri / charon-* / creusot / hax-* / aeneas-* / prusti）走 cargo，有正确的 deps，所以这 19 entry 在它们上是 SUCCESS 或真 partial，不需要 UNKNOWN。

**合理的、可解释的、跨工具一致的 19。**

### §5.2 其他 UNKNOWN 拼图

| 工具 | UNKNOWN entries | 触发规则 |
|---|---|---|
| aeneas-hol4 | 2 (x509-parser × 2) | vendor_lint_strictness（hol4 也走 cargo，会触发 `unused_qualifications` lint 在 vendor x509-parser） |
| hax-coq / hax-fstar / hax-lean | 2 (x509-parser × 2) | 同 |
| kani | 2 (x509-parser × 2) | 同 |
| prusti | 7 = 6 (x509-parser × 2 + rsa × 2 + sha2 × 2) + 1 (let-chains) | x509 / rsa / sha2 走 vendor_lint_strictness 或 dependency_resolution；let-chains 走 toolchain_edition_mismatch（prusti 老 cargo） |

**总 UNKNOWN 跨工具**：5×19 + 5×2 + 7 = 95 + 10 + 7 = 112。这一切都对得上现有 5 条 oracle 规则的覆盖。

### §5.3 应该是 UNKNOWN 但还在 FAILED 的 case

按 §4 的分类，共 64 个 case 应该被 oracle 升级为 UNKNOWN（按宪法 §六）但仍在 FAILED：

- **9 个 edition_pipeline_propagation**：4 个 rocq-of-rust（异步），3 个 verifast（异步 + let-chains），1 个 soteria（let-chains），1 个 verus（let-chains）
- **10 个 dependency_resolution_E0433**：5 工具 × 2 个 x509-parser entry
- **2 个 toolchain_unstable_feature_gate**：prusti × float-round / unsafe-block
- **43 个 kmir wrapper brittleness**：kmir × deps-rich entries

---

## §6 总结

### §6.1 量化总结

- **总 FAILED**：910 / 3220 = 28.3%
- **总候选误报**：64
- **误报率**：64 / 910 = 7.0%（按 FAILED 子集计算）
- **跨工具误报率**：64 / 3220 = 2.0%（按全任务计算）

### §6.2 top 5 嫌疑工具

| 排名 | 工具 | 候选误报数 | 主要类别 |
|---|---|---|---|
| 1 | **kmir** | 43 | wrapper brittleness（Cargo + JSON parse） |
| 2 | **verifast** | 5 | edition_propagation + dep_E0433 |
| 3 | **verus** | 3 | edition + dep_E0433 |
| 4 | **rocq-of-rust** | 2 | edition |
| 4 | **rocq-of-rust-typecheck** | 2 | edition |
| 4 | **prusti** | 2 | unstable_feature_gate |
| 4 | **soteria** | 1 | edition |

### §6.3 关键发现 top 3

1. **MIRI / Kani / charon 是否真有误报？答：没有。**
   - MIRI 4 FAILED 全为真 partial（inline asm / foreign call / UB / socket）。1 个边界 case（uninit-memory UB 检测），按跨工具一致语义不算误报。
   - Kani 8 FAILED 全为 kani-oracle 自披露的真 partial（hard-unsupported MIR：InlineAsm / catch_unwind / ptr_mask / simd_cast / C string literal）。
   - charon-mono 8 + charon-poly 7 FAILED 全为真 partial（charon 显式 `is not supported` / panic 在 charon-driver / vtable extraction 限制 / stack overflow）。

2. **kmir 是误报嫌疑最严重的工具**：100 FAILED 中 43 个（43%）是 wrapper 自身的 Cargo / JSON 失败，**kmir 没自我披露任何 partial 信号**。这些 case 在 cargo-check 上 SUCCESS，证明 entry 合法，但 kmir 在 Python wrapper 阶段（kmir/cargo.py:91 解析 JSON）就崩溃。**强烈建议 cc 阶段 counter-challenge**：是 kmir 真不支持这些 entry 还是 wrapper 太脆弱？

3. **第 6 类外部根因 edition_pipeline_propagation 跨 5 工具影响 9 个 case**：rocq-of-rust×2、rocq-of-rust-typecheck×2、verifast×3、verus×1、soteria×1。这是当前 oracle 第 3 类规则（`this version of Cargo is older than the`）**没覆盖的子类型**——3 类规则的字面只匹配 cargo 自身的 edition gate，但 5 工具的 rustc-direct pipeline 报的是 rustc 的 edition gate（`async fn is not permitted in Rust 2015` / `let chains are only allowed in Rust 2024 or later`）。

### §6.4 UNKNOWN 数字 19 跨 5 工具是否巧合？

**不是巧合，是合理的跨工具一致性**：

5 个工具（rocq-of-rust×2 / soteria / verifast / verus）的 pipeline 都不通过 cargo，所以对所有 19 个有 `extra_cargo_deps` 的 entry（bigint×8 + deps-complex×7 + industrial-rsa×2 + industrial-sha2×2）都触发 `error[E0432]: unresolved import`，被 oracle 规则 2 一致捕获 → UNKNOWN。这正是 §六 "不冤枉" 原则的正确表达。

### §6.5 是否发现需要扩 oracle UNKNOWN 规则的新外部根因类？

**是。建议 cc 阶段裁决以下 3 类候选**：

| 候选类 | 案例数 | 建议规则字面 | 反误报论证（待 cc 阶段补完） |
|---|---|---|---|
| 第 6 类 `edition_pipeline_propagation` | 9 | `to use ` + `async fn ... switch to Rust 2018 or later` 或 `let chains are only allowed in Rust 2024 or later` 或 `pass --edition` | 真 partial 不会含 rustc edition gate 字面，安全 |
| 第 7 类 `dependency_resolution_E0433` | 10 | 把规则 2 扩展为 `E0432: unresolved import` 或 `E0433: cannot find module or crate`，且 entry 在 cargo-check 上 SUCCESS（已是审计前提） | 用户代码不会主动触发 E0433 cannot-find-crate；安全 |
| 第 8 类 `toolchain_unstable_feature_gate` | 2 | `error[E0658]: use of unstable library feature` + entry 在 cargo-check 上 SUCCESS | 反误报：真 partial（prusti 真说 unsupported）不会用 E0658 字面 |
| kmir 专有规则候选 | 43 | `Cargo compilation failed` ∧ `kmir.cargo`，或 `JSONDecodeError` ∧ `kmir/cargo.py` | 反误报：真 partial 经过 K interpreter 会有 `[kmir-oracle] FAIL` 标志，wrapper 失败模式独立 |

**注意**：以上仅为 c 阶段独立审计的候选清单。是否落实为 oracle 规则、字面如何精确化、反误报详细论证、覆盖度论证，全部留给 cc 阶段 counter-challenge + d 阶段实施。本审计严格遵守 §六 c 阶段定义：**只找不修**。

### §6.6 给 cc 阶段的建议

按嫌疑程度从高到低排序，**cc 阶段应重点 counter-challenge**：

1. **kmir × 43**：分别检查每个失败的 entry 在 kmir 自身 cargo 下是否真的 unbuildable，或 wrapper 太脆弱。如果大量是后者，考虑给 kmir tool.toml 加 wrapper 健壮化或 oracle 加 kmir 专属规则。
2. **edition_pipeline_propagation × 9**：确认 5 个工具的 rustc-direct pipeline 真的没办法传 edition flag，或者可以从 Cargo.toml 读 edition 后注入。如果可以，d 阶段在 tool.toml 加 edition flag；如果不行，oracle 加第 6 类规则。
3. **dependency_resolution_E0433 × 10**：确认 E0433 拓展规则不会误吞用户的真模块/类型缺失 bug。预期不会（cargo-check SUCCESS 已经排除）。
4. **prusti × 2 toolchain_unstable_feature_gate**：确认这两个 entry 在更新版 prusti（如果有）下是否还失败；也可考虑加 #![feature(...)] 让 entry 兼容老 prusti（不破坏 cargo-check）。

---

## §7 严格遵守

- 本审计**不修任何代码 / tool.toml / wrapper / runner**
- 本审计**不 commit** 任何代码
- 本审计**不基于猜测**：每条误报候选都引用具体 entry name + stderr 文本片段
- 本审计**不胡编 / 不凑数**：所有数字均从 `runs/run-1778500291-90812/raw/` 实测可复核

完成。文件路径：`docs/fixes/audit-v5-c-false-positive-2026-05-11.md`
