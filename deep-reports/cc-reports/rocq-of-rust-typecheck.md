# rocq-of-rust-typecheck 深度报告

## 元数据

- **run**: `run-1778473345-64581`（2026-05-11T04:22:25Z → 04:22:34Z UTC，host: Apple M5 / macOS aarch64 25.4.0 / 24 GB / 10 cpu，parallelism 10，~9 s wall）
- **配对档 0 run**: `run-1778473390-70662`（同 corpus，相邻时刻跑 `rocq-of-rust`，用于档 0 vs 档 1 对照）
- **工具版本**：
  - `rocq_of_rust_cli 0.1.0` @ commit `a8a76a4d`（与 `tools/rocq-of-rust` 同 binary）
  - `coqc The Coq Proof Assistant, version 9.0.0` via opam switch `ror-test`（Rocq 9.0.0 + rocq-stdlib 9.0.0 + rocq-smpl + coq-hammer 1.3.2+9.0 + coq-coqutil 0.0.7）
- **通过率**：109/146 = **74.7%**（档 0 同 corpus 110/146 = 75.3%，**delta = -0.7pp / -1 entry**；FAILED 37 个；TIMEOUT 0）
- **时长（ms）**：SUCCESS avg 680 / min 435 / max 1074；FAILED avg 136 / min 84 / max 249；全量 avg 542
- **时效声明**：本快照锚定上述 run id + 工具版本 + corpus + 9 道 gate oracle，不构成长期承诺。ror 翻译质量、Rocq 9 typecheck 行为都可能随上游版本浮动。

## 工具内部 pipeline + 前端边界

档 1 是 `rocq-of-rust`（档 0：纯翻译落盘）的严格上层包裹：

```
src/lib.rs
    │
    ▼  Stage 1: rocq-of-rust translate
    │  (DYLD_LIBRARY_PATH = nightly sysroot lib, PATH 前置 nightly bin)
    │  (rustc_interface → Rocq monadic embedding → 写 .v)
    │
rocq_translation/<absolute-source-path>/lib.v
    │
    ▼  Gate 1-6 (sourced from tools/rocq-of-rust): exit 0, .v exists,
    │  > 200 B, no failure marker, Definition <entry_fn> present
    │
    ▼  Stage 2: coqc -R <runtime> RocqOfRust -impredicative-set <product>.v
    │  (opam switch = ror-test; Rocq 9.0.0)
    │  (ror 9 个核心 runtime .v 已预编：M / RocqOfRust / RecordUpdate /
    │   lib/lib / lib/simulate/lib / links/M / links/RocqOfRust /
    │   simulate/M / simulate/RocqOfRust —— wrapper 启动时 stat + 缺则补)
    │
<product>.vo + stderr 检查
    │
    ▼  Gate 7-9 (new at tier-1): coqc exit 0, .vo present, no "Error" line in stderr
    │
SUCCESS / FAILED
```

档 1 与档 0 的"前端边界"不同：
- 档 0 前端 = rocq-of-rust 翻译落盘（停在 `.v` 文件写盘）
- 档 1 前端 = 翻译落盘 **+ Rocq coqc 接受**（停在 `.vo` 文件写盘）

两者都满足硬指标 §六-1（前端支持性观察原则），是不同档位的合法切割。

## SUCCESS 信号 + 形式严格性（9 道门）

**SUCCESS 必须满足 9 道门**：

档 0 已有的 6 道（在 `wrapper.sh` 中独立实现，与 `tools/rocq-of-rust` 等价）：

1. rocq-of-rust translate exit code = 0
2. 至少一个 `.v` 产物存在
3. 无 0-byte `.v`
4. 至少一个 `.v` > 200 字节
5. 产物不含显式 failure marker：`grep -rqE '\(\* (Error |Unexpected |Please report!|thir failed to compile|Unimplemented )'`
6. entry_fn 的 `Definition <fn>` 出现在某个 `.v` 产物中：`grep -rqE "^[[:space:]]*Definition[[:space:]]+$TS_ENTRY_FN[[:space:]]"`

档 1 新增的 3 道：

7. **coqc exit code = 0**
8. **coqc 产出 `.vo`**（exit 0 + 无产物的 belt-and-braces）
9. **coqc stderr 无 `^Error` 行**（exit 0 + 含 Error 的 belt-and-braces）

任一门未满足 → FAILED。

### 形式严格性

- **0 误报**：✅ **基本可形式证明**。gate 7（coqc exit 0）即 Rocq 9 typecheck 完整通过——Rocq typecheck 是确定性算法，无随机性、无 silent partial 路径。gate 1-6 的形式严格性继承档 0（详 `deep-reports/cc-reports/rocq-of-rust.md`）。gate 8/9 是 coqc exit 0 的复核，不会冤枉合法 SUCCESS。
- **0 漏报**：✅ **基本可形式证明**。Rocq typecheck 对任何失败必 exit ≠ 0；gate 7 直接捕获。
- **partial 暴露机制**：
  - rocq-of-rust 端 silent partial（同档 0）：translate exit 0 + 产物含 marker / 缺 entry_fn → gate 5/6 抓
  - coqc 端 partial：typecheck 失败 → gate 7 抓（Rocq 9 标准 exit code）
- **漏报盲点**：
  - **`Admitted` 占位通过**：ror 翻译生成的 `Global Instance Instance_IsFunction_<fn> : ... Admitted.` 是 ror 的设计选择（让 `Instance` lookup 通过），typecheck 通过 = 包含 `Admitted` 路径——这**不算漏报**，是档 1 的诚实边界：本档**只**测产物可编译性、**不**测语义正确性（档 2/3 范畴，且 ror 不适合自动化）
  - 本档边界**不**等同档 2/3（evaluate / 一致性）；不要把档 1 的"SUCCESS"误读为"产物语义可信"

## 实测结果

### 通过率分布（按 feature 类目）

| feature | SUCCESS | 总 entry |
|---|---|---|
| aeneas-limit | 6 | 8 |
| arc | 1 | 1 |
| assoc-type | 1 | 1 |
| bigint | 0 | 8 |
| box | 2 | 2 |
| charon-limit | 6 | 7 |
| closure | 2 | 2 |
| closure-adv | 4 | 4 |
| collections | 2 | 2 |
| concurrency | 2 | 2 |
| const | 1 | 1 |
| creusot-limit | 6 | 7 |
| deps-complex | 0 | 7 |
| drop | 1 | 1 |
| enum | 2 | 2 |
| error | 1 | 1 |
| float | 10 | 10 |
| gat | 1 | 1 |
| generic | 4 | 4 |
| hax-limit | 7 | 8 |
| hello | 1 | 1 |
| hrtb | 1 | 1 |
| impl-trait | 1 | 1 |
| industrial | 0 | 6 |
| int | 2 | 2 |
| int-width | 14 | 14 |
| iter | 1 | 1 |
| kani-limit | 5 | 7 |
| lifetime | 2 | 3 |
| miri-limit | 4 | 7 |
| panic | 2 | 2 |
| prusti-limit | 4 | 8 |
| rc | 1 | 1 |
| refcell | 1 | 1 |
| repr | 1 | 2 |
| slice | 1 | 1 |
| trait | 1 | 1 |
| trait-obj | 2 | 2 |
| unsafe-adv | 3 | 3 |
| unsafe-ptr | 2 | 2 |
| vec | 1 | 1 |

全 SUCCESS 类目（含部分小类目全过）：`arc / assoc-type / box / closure / closure-adv / collections / concurrency / const / drop / enum / error / float (10/10) / gat / generic / hello / hrtb / impl-trait / int / int-width (14/14) / iter / panic / rc / refcell / slice / trait / trait-obj / unsafe-adv / unsafe-ptr / vec`。

### 失败模式归类（37 个 FAILED）

37 个失败全部在 Stage 1（rocq-of-rust 自身或 oracle gate 1-6），**Stage 2（coqc）阶段无 FAILED**。

**A. 单文件输入 + 不读 Cargo.toml（22）**——`error[E0432]: unresolved import`：
- `bigint/*` 8 个（num_bigint / num_traits / num_integer / num_rational / num_complex）
- `deps-complex/*` 7 个（chrono / serde / itertools / anyhow / thiserror）
- `industrial/*` 6 个（rsa / sha2 / x509_parser）
- `kani-limit/async-await/run_async_add` 1 个

不是 ror 翻译能力边界——rustc 在 import 阶段就拒了，ror 没机会做翻译。

**B. nightly toolchain 默认 edition / unstable feature（2）**——
- `charon-limit/async-fn/async_forty_two`（`E0670: 'async fn' is not permitted in Rust 2015`）
- `hax-limit/let-chains/hax_limit_let_chains`（`E0658: 'let' expressions in this position are unstable`）

rocq-of-rust 用 nightly-2024-12-07 调 rustc 但未透传 cargo manifest 的 `edition`。

**C. 翻译阶段产物含 explicit failure marker（1）**——`repr/union/repr_union`（gate 5 命中 `(* Error Variant *)`，来自 `TopLevelItem::Error(Variant::Union)`）

**D. gate 6 entry_fn silent skip-item（12）**——entry_fn 被 silently 丢弃，产物中无对应 `Definition`：

| entry | 备注 |
|---|---|
| `aeneas-limit/mutually-recursive-traits/trigger_mutually_recursive_traits` | 同档 0 |
| `aeneas-limit/trait-impl-mut-param-mismatch/trigger_trait_impl_mut_param_mismatch` | 同档 0 |
| `creusot-limit/thread-local-ref/read_thread_local` | **档 0 SUCCESS 但档 1 FAILED**（详 §"档 0 vs 档 1 差异"）|
| `kani-limit/float-overapprox/trigger_check_sin_cos_identity` | 同档 0 |
| `lifetime/thread-local/thread_local_read` | 同档 0 |
| `miri-limit/simd-bitmask-large-vector/trigger_bitmask_over_64_elements` | 同档 0 |
| `miri-limit/soundness-not-guaranteed/trigger_safe_wrapper_hides_ub` | 同档 0 |
| `miri-limit/weak-memory-incomplete/relaxed_load_may_not_observe_all_stores` | 同档 0 |
| `prusti-limit/loan-crosses-loop-boundary/trigger_loan_crosses_loop_boundary` | 同档 0 |
| `prusti-limit/ref-typed-struct-field/trigger_ref_typed_struct_field` | 同档 0 |
| `prusti-limit/shallow-borrow-match-guard/trigger_shallow_borrow_match_guard` | 同档 0 |
| `prusti-limit/spec-entailment-unsupported/trigger_spec_entailment_unsupported` | 同档 0 |

**E. Stage 2 coqc 失败（0）**——本 corpus 上无 entry 在档 0 SUCCESS 但 coqc typecheck 失败。

### 档 0 vs 档 1 差异

档 0 = 110 SUCCESS，档 1 = 109 SUCCESS，**差 1 个 entry**：`creusot-limit/thread-local-ref/read_thread_local`。

- entry fn `read_thread_local` 调 `thread_local!` macro + `with(|c| ...)`
- ror 翻译产物含 `Definition value_COUNTER` + `Definition __init`，**不**含 `Definition read_thread_local`（silent skip）
- 档 0 oracle 中 gate 6 是同一逻辑（grep `Definition <entry_fn>`）但**未**抓到，本档独立实现的 gate 6 抓到了

**事实**：这不是 ror 翻译落盘后 coqc 编不过的"档 1-specific 失败"，而是 `tools/rocq-of-rust` 的一个潜在 oracle 漏报（在该 entry 上 gate 6 应抓但没抓）。本档 wrapper 用独立 bash 实现 gate 6，逻辑一致但执行路径不同，意外把档 0 的这个漏报抓出来了。

**对档 1 价值的反推**：理想情况下档 1 应该暴露"档 0 落盘但 coqc 编不过"的 entry（这就是 archive §6.1 提到的 typecheck silent gap）。本 corpus 上**0 个**这类 entry——说明 ror 在当前 corpus 上 typecheck 层稳定，translate 落盘 ⇒ coqc 接受。

## 时长分布

| 阶段 | SUCCESS avg | SUCCESS p50 | SUCCESS max | FAILED avg |
|---|---|---|---|---|
| 全量 | 680 ms | ~660 ms | 1074 ms | 136 ms |

SUCCESS 时长 ≈ rocq-of-rust translate (~50-100 ms) + coqc -R RocqOfRust 编译产物 (~400-600 ms) + opam env 切换 (~50 ms) + bootstrap stat 9 .vo (~ms 级)。FAILED 都在 Stage 1，~ 50-150 ms 早退。

## 该工具的 testsuite 内位置

- **档位**：档 1（产物 typecheck）
- **配对档 0 工具**：`tools/rocq-of-rust`
- **配对档 1 工具**（其他翻译类）：暂无；hax × 3 / aeneas × 4 都目前停留在档 0
- **该工具的独立性**：本工具独立测试一项可形式 attribute"档 1 typecheck 通过"，与档 0 通过率配对呈现可暴露 ror 翻译的"档 0 落盘 ≠ 档 1 typecheck 接受"现象（本 corpus 上 0 现象，意味 ror 在 typecheck 层 OK）。

## 翻译产物可运行性

本工具实现 ror 档 1（产物 typecheck）。档 2/3（evaluate / 与 Rust 一致）**架构上不可达**——这是 ror **设计选择**，不是 bug。详见 [`../../docs/research/ror-runnable-deep-dive-2026-05-11.md`](../../docs/research/ror-runnable-deep-dive-2026-05-11.md)。

**深嵌入 vs 浅嵌入**：

- **ror 产物 = deep embedding**：`Definition fn (a b : Value.t) : M.t LowM.t (Value.t + Exception.t) := ...`，`Value.t` 是 inductive 包装类型，`M` 是 effect monad，所有 op（`alloc` / `read` / `call_closure` / `call_primitive`）是 inductive constructor，**无 Compute 语义**。`vm_compute` / `native_compute` 在 axiom-laden `Run.t` proof tree 上 SIGSEGV。
- **hax-lean 产物 = shallow embedding**（对照）：`def fn (a b : Int32) : RustM Int32 := RustM.ok (a + b)`，Lean `#eval fn 3 4` 一行直接出值。

**ror 上游"官方运行模式"**：

- API：`SimulateM.eval` / `SimulateM.eval_f`（`simulate/M.v:343 / 445`）
- 性质：**propositional 解释器**，不是 native compute
- 输入：`LinkM.t R Output` + 需要 `Run.Trait` 实例（用户用 `run_symbolic` tactic 推导）
- 输出：`SimulateM.t` inductive，**不是** native `Z`
- 与"值"关系：propositional（`🌲` = `Run.t`），用 `repeat (eapply Run.Call || apply Run.Pure)` 证明
- 性能：per-entry **5–50+ 行手工 Coq tactic**；递归 fn 需手工 well-founded induction

**档可达性总结表**：

| 档 | ror | hax-lean |
| --- | --- | --- |
| 档 0 前端接受 | ✅ `tools/rocq-of-rust` | ✅ `tools/hax-lean` |
| 档 1 typecheck | ✅ **本工具（rocq-of-rust-typecheck）** | ✅ feasibility 实测 |
| 档 2 auto evaluate | ❌ **架构上不可达** | ✅ `#eval` 实测 |
| 档 2 半人工 lemma | ⚠️ per-entry 5–50 行手工证明 | — |
| 档 3 与 Rust 一致 | ❌ 除非档 2 解决 | ✅ byte-identical 实测 |

**项目决策**：

- 投入档 1 自动化（**本工具**，9 道 gate = 档 0 的 6 道 + coqc exit / 产物 / stderr 3 道）
- 不投入档 2/3 自动化：per-entry 手工证 vs corpus ~150 entries 规模严重不匹配；ror 上游设计哲学就是"语义留作用户 proof obligation"（产物头部 `Admitted.` 已声明该立场）
- 严格说，本工具测的是"翻译产物在 Coq 里是有效的 Coq 项"——**结构正确，语义不验证**。`Admitted.` 占位通过 typecheck 是档 1 诚实边界，与档 2/3 范畴严格区分。
