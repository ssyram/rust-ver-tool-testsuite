# rocq-of-rust-typecheck — 特性支持评估报告（v6 baseline）

## 元数据

- **数据源**：`runs/run-1778560393-59119/`（2026-05-12 v6 final，合并 verus rerun + R7 5-tool rerun）
- **工具配置**：`tools/rocq-of-rust-typecheck/`（`tool.toml` + `rocq-of-rust-typecheck-wrapper.sh`）
- **工具版本**：
  - `rocq-of-rust: rocq_of_rust_cli 0.1.0`（commit `a8a76a4d`，与 `tools/rocq-of-rust` 同 binary）
  - `coqc: The Rocq Prover, version 9.0.0` via opam switch `ror-test`
- **本工具实测**：n=161 / SUCCESS=124 / FAILED=37 / UNKNOWN=0，通过率 **77.0 %**
- **时长分布**：全量 avg 895 ms / p50 919 ms / p90 1561 ms / max 2304 ms；SUCCESS avg 1126 ms（min 335 / max 2304）；FAILED avg 122 ms（早退）
- **宪法 baseline**：`principles.md` v8（P27 修宪后 / P30 D3.1 silent partial gate / P31 法律传导后）
- **时效声明**：本快照锚定上述 run id + 工具版本 + corpus + 10 道 gate oracle，不构成长期承诺。ror 翻译质量、Rocq 9 typecheck 行为都可能随上游版本浮动。

## pipeline + 前端边界

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
    ▼  Gate 1-7 (sourced from tools/rocq-of-rust): exit 0, .v exists,
    │  > 200 B, no failure marker, Definition <entry_fn> present,
    │  stderr 无 "is not yet supported"（P30 D3.1 同步加 gate）
    │
    ▼  Stage 2: coqc -R <runtime> RocqOfRust -impredicative-set <product>.v
    │  (opam switch = ror-test; Rocq 9.0.0)
    │  (9 个核心 runtime .v 已预编：M / RocqOfRust / RecordUpdate /
    │   lib/lib / lib/simulate/lib / links/M / links/RocqOfRust /
    │   simulate/M / simulate/RocqOfRust —— wrapper 启动时 stat + 缺则补)
    │
<product>.vo + stderr 检查
    │
    ▼  Gate 8-10 (new at tier-1): coqc exit 0, .vo present, no "Error" line in stderr
    │
SUCCESS / FAILED
```

- **工具自身**：Stage 1 是 ror 官方 binary（`rocq-of-rust translate`），Stage 2 是官方 `coqc`
- **项目维护 wrapper**（`rocq-of-rust-typecheck-wrapper.sh`）：activate opam switch + runtime bootstrap + 7 道 grep gate + Stage 2 coqc 调用 + gate 8-10
- **前端测量边界**：rocq-of-rust 翻译落盘 **+ Rocq coqc 接受**（停在 `.vo` 写盘）。档 2/3（evaluate / 一致性）架构上不可达——详 `docs/research/ror-runnable-deep-dive-2026-05-11.md`

## SUCCESS 信号 + 形式严格性

按宪法 §六 双通路 partial 暴露：

**SUCCESS 必须满足 10 道门**：

档 0 已有的 7 道门（gate 1-7，wrapper 中独立实现，与 `tools/rocq-of-rust` 等价）：

1. rocq-of-rust translate exit code = 0
2. 至少一个 `.v` 产物存在
3. 无 0-byte `.v`
4. 至少一个 `.v` > 200 字节
5. 产物不含显式 failure marker：`\(\* (Error |Unexpected |Please report!|thir failed to compile|Unimplemented )`
6. entry_fn 的 `Definition <fn>` 出现在某个 `.v` 产物中
7. stderr 不含 `is not yet supported`（**P30 同步加的 D3.1 silent partial gate**，恢复 tier-1 ⊆ tier-0 不变式——v6 cc-route 漏报审查发现 tier-1 漏抄此 gate 已破声明，补此 gate）

档 1 新增的 3 道门（gate 8-10）：

8. **coqc exit code = 0**
9. **coqc 产出 `.vo`**（exit 0 + 无产物的 belt-and-braces）
10. **coqc stderr 无 `^Error` 行**（exit 0 + 含 Error 的 belt-and-braces）

任一门未满足 → FAILED。

**主信号通路 / wrapper 补抓通路**：

- 主信号：rocq-of-rust translate exit code + coqc exit code（工具自带）
- wrapper 补抓：gate 2-6（落盘形态校验 + entry_fn skip-item 抓）+ gate 7（D3.1 silent partial）+ gate 8-10（coqc exit 0 belt-and-braces）

### 形式严格性

- **0 误报**：✅ **基本可形式证明**。gate 8（coqc exit 0）即 Rocq 9 typecheck 完整通过——Rocq typecheck 是确定性算法，无随机性、无 silent partial 路径。gate 1-7 的严格性继承档 0（详 `deep-reports/cc-reports/rocq-of-rust.md`）。gate 9/10 是 coqc exit 0 的复核，不会冤枉合法 SUCCESS。
- **0 漏报**：✅ **基本可形式证明**（在 typecheck 范畴内）。Rocq typecheck 对任何失败必 exit ≠ 0；gate 8 直接捕获。gate 7（D3.1）补封 ror 翻译端 silent partial（"is not yet supported" warning + exit 0 + feature 从 .v 删除）。
- **漏报盲点（诚实声明）**：
  - **`Admitted` 占位通过**：ror 翻译生成的 `Global Instance Instance_IsFunction_<fn> : ... Admitted.` 是 ror 设计选择（让 `Instance` lookup 通过），typecheck 通过 = 包含 `Admitted` 路径——这**不算漏报**，是档 1 的诚实边界：本档**只**测产物可编译性、**不**测语义正确性
  - **ror 翻译走 axiom**：若 ror 上游引入 `Axiom <name> : ...`（理论上 typecheck 仍过），同上不算漏报，是档位边界
  - **本档边界不等同档 2/3**（evaluate / 一致性）；不要把档 1 的 SUCCESS 误读为"产物语义可信"

## 失败分桶（按 P31 §四.5 归因分类）

37 个 FAILED 全部命中 Stage 1（rocq-of-rust translate 阶段或 wrapper gate 1-7），**Stage 2（coqc）阶段无 FAILED**。

### 桶 A：rustc 解析阶段 unresolved import / undeclared crate（21 case）

代表 entry：`bigint/bigint-arith/bigint_arith`、`deps-complex/chrono-serde/chrono_serde`、`industrial/rsa/rsa-pkcs8/rsa_pubkey_from_pkcs8`

stderr 特征：

```
error[E0432]: unresolved import `num_bigint`
error[E0433]: failed to resolve: use of undeclared crate or module `serde_json`
error[E0599]: no method named `chunks` found ...
[ror-typecheck-oracle] FAIL: rocq-of-rust translate exit 101
```

涵盖：
- `bigint/*`（8 个）— num_bigint / num_traits / num_integer / num_rational / num_complex
- `deps-complex/*`（7 个）— chrono / serde_json / itertools / thiserror
- `industrial/*`（6 个）— rsa / sha2 / x509_parser

**归因**：工具不支持。`rocq-of-rust translate --path src/lib.rs` 是单文件 pipeline，**不读 Cargo.toml 的 dependency**，rustc 在 import 阶段就拒了，ror 根本没机会做翻译。这是 ror 官方 driver 的能力边界。

**处理**：不修。本地性原则下 FAILED 站得住，工具开发者不能驳回。

### 桶 B：nightly default edition / unstable feature（3 case）

涉及 entry：
- `charon-limit/async-fn/async_forty_two`（E0670 `async fn` not permitted in Rust 2015）
- `kani-limit/async-await/run_async_add`（同 E0670）
- `hax-limit/let-chains/hax_limit_let_chains`（E0658 `let` chains unstable）

stderr 特征：

```
error[E0670]: `async fn` is not permitted in Rust 2015
  = help: pass `--edition 2024` to `rustc`
error[E0658]: `let` expressions in this position are unstable
  = help: add `#![feature(let_chains)]` to the crate attributes
[ror-typecheck-oracle] FAIL: rocq-of-rust translate exit 101
```

**归因**：工具不支持。`rocq-of-rust translate` 用 nightly-2024-12-07 调 rustc 但**未透传 cargo manifest 的 `edition`** 或开启 unstable feature——属 ror 官方 driver 设计选择。

**处理**：不修。FAILED 站得住。

### 桶 C：翻译阶段产物含 explicit failure marker（1 case）

entry：`repr/union/repr_union`

stderr 特征：gate 5 命中 `(* Error Variant *)`（产物中 ror 自陈无法翻译 `Variant::Union`）。

**归因**：工具不支持。ror 翻译器对 union 显式 emit `TopLevelItem::Error(Variant::Union)`——是工具明示自陈 reject。

**处理**：不修。

### 桶 D：gate 6 entry_fn silent skip-item（11 case）

涉及 entry：

| entry |
|---|
| `aeneas-limit/mutually-recursive-traits/trigger_mutually_recursive_traits` |
| `aeneas-limit/trait-impl-mut-param-mismatch/trigger_trait_impl_mut_param_mismatch` |
| `creusot-limit/thread-local-ref/read_thread_local` |
| `kani-limit/float-overapprox/trigger_check_sin_cos_identity` |
| `miri-limit/simd-bitmask-large-vector/trigger_bitmask_over_64_elements` |
| `miri-limit/soundness-not-guaranteed/trigger_safe_wrapper_hides_ub` |
| `miri-limit/weak-memory-incomplete/relaxed_load_may_not_observe_all_stores` |
| `prusti-limit/loan-crosses-loop-boundary/trigger_loan_crosses_loop_boundary` |
| `prusti-limit/ref-typed-struct-field/trigger_ref_typed_struct_field` |
| `prusti-limit/shallow-borrow-match-guard/trigger_shallow_borrow_match_guard` |
| `prusti-limit/spec-entailment-unsupported/trigger_spec_entailment_unsupported` |

stderr 特征：

```
[ror-typecheck-oracle] FAIL: entry_fn '<name>' missing from .v products
```

ror translate exit 0 + 产物落盘，但产物中**不含 `Definition <entry_fn>`**——ror 内部 silently 跳过该 entry（如对 `thread_local!` macro 展开后 ror 翻译出 `Definition value_*` 等支撑符号但跳过用户 fn 本体；对 mutually-recursive traits / mut param mismatch 等同理）。

**归因**：工具不支持。是 ror 翻译器对该类 MIR 构造的能力边界——silent partial 已由 gate 6 抓出。

**处理**：不修。FAILED 站得住，按宪法 §六 oracle "不冤枉"原则严格执行（partial 翻译不算 SUCCESS）。

### 桶 E：D3.1 silent partial（"is not yet supported" warning，1 case）

entry：`unsafe-ptr/raw-ptr-const/raw_ptr_const_match`

stderr 特征：

```
warning: This kind of constant in patterns is not yet supported.
[ror-typecheck-oracle] FAIL: rocq-of-rust emitted 'is not yet supported' warning (silent partial translation; exit 0 but feature dropped from .v product)
```

ror translate exit 0 + 产物落盘 + 产物中 `Definition raw_ptr_const_match` 存在（gate 6 通过），但 ror stderr 自陈"该形态的常量 pattern 还不支持"——属 ror 明示自陈 partial。

**归因**：工具不支持（自陈 partial）。

**处理**：不修。**此 entry 是 v6 新捕获 case**：v5.1 时 wrapper 漏抄此 gate 让该 entry 误判 SUCCESS，P30 D3.1 同步加 gate 7 后正确归 FAILED（恢复 tier-1 ⊆ tier-0 不变式）。

### 桶 F：coqc Stage 2 失败（0 case）

本 corpus 上**0 个**entry 在档 0 SUCCESS 但 coqc typecheck 失败——说明 ror 在当前 corpus 上 translate ⇒ coqc 接受是稳定的。理想情况下档 1 应该暴露"档 0 落盘但 coqc 编不过"的 entry（archive §6.1 提到的 typecheck silent gap），本 baseline 上无此现象。

## 漏报盲点（诚实声明）

- **已通过 wrapper gate 封堵**：
  - 产物落盘形态退化（0 字节 / 小于 200 字节 / 无产物）→ gate 2/3/4
  - 显式 failure marker → gate 5
  - entry_fn silent skip-item → gate 6
  - "is not yet supported" warning（D3.1 silent partial）→ gate 7（P30 同步加）
  - coqc exit 非 0 → gate 8
  - coqc exit 0 但无 .vo 或 stderr 含 Error → gate 9/10
- **仍存在的盲点**：
  - **`Admitted.` 占位 typecheck 过**：ror 设计选择，档 1 边界明确不评判（不算漏报）
  - **ror 上游引入 `Axiom`**：理论上 typecheck 仍过，同档位边界
  - **档 2/3**（evaluate / 一致性）架构上不可达，不在本工具范围（参 `tools/rocq-of-rust-typecheck/README.md` §"与 hax-lean 的可运行性对比"）

## v5.1 → v6 ΔS 解释

- v5.1: 109/146 = 74.7 %
- v6: 124/161 = 77.0 %
- ΔS = **+15 SUCCESS**

来源：
- **+15 runnable/* SUCCESS**（v6 新增 15 个 runnable feature entry，全部翻译并 typecheck 通过）
- **+1 lifetime/thread-local/thread_local_read FAILED→SUCCESS**（v5.1 时被 gate 6 抓 entry_fn missing；v6 同一 entry 现 SUCCESS——upstream / corpus 变化使 ror 翻译该 entry 现在产出 `Definition thread_local_read`）
- **−1 unsafe-ptr/raw-ptr-const/raw_ptr_const_match SUCCESS→FAILED**（P30 D3.1 gate 7 同步加，原 v5.1 误判 SUCCESS，v6 正确归 FAILED——这是 oracle 严格性提升而非工具能力退步）

净 ΔS = +15 +1 −1 = +15。

corpus 同时从 146 增到 161（+15 runnable entries），其他 entry 集合保持。

## 修订建议清单（仅"我们导致"失败）

**无需修订**——所有 37 个 FAILED 均为工具能力边界（21 桶 A + 3 桶 B + 1 桶 C + 11 桶 D + 1 桶 E），按宪法 §六 UNKNOWN 严格语义 + §一 本地性原则 + tool-integration §四.5"我们 wrapper vs 官方 wrapper 归因"判据，全部归"工具不支持"——FAILED 站得住，工具开发者不能驳回。

无 "我们 wrapper bug" / 无 "我们 corpus bug" / 无 "环境损坏" / 无 "漏报候选"。

P30 D3.1 同步加 gate 7 已治"tier-1 漏抄 D3.1 gate 破 tier-1 ⊆ tier-0 不变式"——属 oracle 严格性补丁，**非**新增漏报修复（v5.1 该 case 是 oracle 漏抓导致 SUCCESS 误报，v6 修正后正确归 FAILED）。

## 该工具的 testsuite 内位置

- **档位**：档 1（产物 typecheck）
- **配对档 0 工具**：`tools/rocq-of-rust`
- **配对档 1 工具**（其他翻译类）：暂无；hax × 3 / aeneas × 4 都目前停留在档 0
- **独立性**：本工具独立测试"档 1 typecheck 通过"，与档 0 通过率配对呈现可暴露 ror 翻译"档 0 落盘 ≠ 档 1 typecheck 接受"现象（本 corpus 上 0 现象，意味 ror 在 typecheck 层稳定）

## 翻译产物可运行性

本工具实现 ror 档 1（产物 typecheck）。档 2/3（evaluate / 与 Rust 一致）**架构上不可达**——ror 上游设计选择（deep embedding monadic `M`，无 native compute；语义留作用户 proof obligation），不是 bug。详见 [`docs/research/ror-runnable-deep-dive-2026-05-11.md`](../../docs/research/ror-runnable-deep-dive-2026-05-11.md)。

**项目决策**：

- **投入档 1 自动化**：本工具上线，10 道 gate（档 0 的 7 道 + coqc exit/产物/stderr 3 道）
- **不投入档 2/3 自动化**：per-entry 5–50+ 行手工 Coq tactic vs corpus ~160 entries 规模严重不匹配；ror 上游设计哲学就是"语义留作用户 proof obligation"
- 严格说，本工具测的是"翻译产物在 Coq 里是有效的 Coq 项"——**结构正确，语义不验证**。`Admitted.` 占位通过 typecheck 是档 1 诚实边界，与档 2/3 范畴严格区分。
