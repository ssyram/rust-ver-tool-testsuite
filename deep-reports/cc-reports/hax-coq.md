# hax-coq — 特性支持评估报告

## 元数据

- **数据源**：`runs/run-1778226613-5282/`（2026-05-08，146 entries × 19 工具矩阵；host: Apple M5 / macOS aarch64 / 24 GB / 10 cpus，并发 10）
- **工具版本**：`hax untagged-git-rev-30949eb870`（commit `30949eb87058895c24f963df90dd30ef11b0dc1a`）；nightly toolchain `nightly-2025-11-08`；OCaml `hax-engine` + Rust frontend driver
- **工具配置**：`tools/hax-coq/`
- **通过率**：SUCCESS 98 / 146 ≈ **67.1%**（FAILED 48，TIMEOUT 0）
- **耗时分布**：avg 3736 ms / median 1790 ms / p90 8004 ms / p95 16943 ms / max 26428 ms（无 timeout 触发）
- **时效声明**：本快照锚定上述 run id + hax commit + nightly 工具链 + corpus，不构成长期承诺。Coq backend 在 hax upstream 标记为 partial（`(* NotImplementedYet *)` boilerplate header 是设计内的、不计入失败信号），上游对 Coq Printer 的 reject phase 序列改动会让本快照失效。

## 工具内部 pipeline + 前端边界

```
rustc + driver-hax-frontend-exporter
  → THIR JSON
  → hax-engine（OCaml）
  → phase pipeline（reject phases + 改写 passes）
  → Coq Printer（OCaml→Coq/Rocq 文本生成）
  → 写出 <work>/proofs/coq/extraction/<Crate>.v + _CoqProject
```

本测试关心 `cargo +nightly-2025-11-08 hax -C --lib ; into coq` 的"前端通过"。**前端 / 后端切割**：纯翻译工具，pipeline 终点是 `.v` 文件 + `_CoqProject`——下游 `coqc` / hax Coq 支持库 / 用户证明完全不在测试范围。`_CoqProject` 中库名为 `TODO`（`-R ./ TODO`），未配合 hax Coq 支持库时不能直接 `coqc` 编译——这是 upstream 的预期行为。

Coq backend 在三 hax backend 中**reject phase 数量最多**（实测 stderr 观察到 9 个：`reject_Unsafe`、`reject_RawOrMutPointer`、`reject_Arbitrary_lhs`、`reject_Dyn`、`reject_TraitItemDefault`、`AndMutDefsite`、`DirectAndMut`、`LocalMutation`、`CfIntoMonads/FunctionalizeLoops`），所以矩阵通过率 67% 显著低于 lean 75% / fstar 79%。

## SUCCESS 信号 + 形式严格性

**判定式**：

```
SUCCESS ⟺ cargo hax exit 0 ∧ 产物 grep 不命中
          'failure ((' 或 'please implement the method' 字面
```

`please implement the method` 来自上游 `engine/backends/coq/coq/coq_backend.ml:137` 的 `default_string_for s = "TODO: please implement the method '..."` 路径——这是**纯文本输出，不发 Diagnostic**——cargo hax 仍 exit 0 但 `.v` 文件里散布 `"TODO: please implement..."` 字面字符串。这条 silent path 与 hax-lean 的 sentinel sorry 同性质：工具自陈"我没全干完"，按宪法 §六-2 必须 → FAILED。

注：`(* NotImplementedYet *)` 是每个 `.v` 文件的 boilerplate header（hax 给所有 Coq 输出自动加），**不算**失败信号——oracle 不抓这个 marker。

**双轨 partial 暴露机制**：
1. cargo hax exit 1 = engine emit `[HAX0001]`-`[HAX0011]` Diagnostic（hax-coq 的主力信号）
2. silent path：`coq_backend.ml:137` 的 `default_string_for` 纯文本输出 → grep `failure ((` / `please implement the method` 抓

本矩阵下 oracle 对 hax-coq **0 次触发 silent-path 改判**——所有 48 条 FAILED 都通过 cargo hax exit 1 信号识别，silent partial 在本 corpus 上未观察到。Coq Printer 在 reject phase 上很激进，多数潜在 silent path 已被前置的 phase reject 拦截。

**形式严格性**：
- **0 误报**：⚠️ 实测验证，不可形式证明。实测：hax-coq **不翻译 Rust doc comment**（与 hax-lean 不同），所以注释里的 `please implement` 不会出现在 `.v` 产物；用户合法代码极难写 `failure ((` 双括号字面
- **0 漏报**：⚠️ 实测验证，不可形式证明。grep 抓 hax-coq 已知 silent path
- **漏报盲点**：hax engine 完全 skip item（实测 0 现象）；上游引入新 silent path 的可能

## 实测结果

### 按 feature 类目分布

下列 feature 类目下 hax-coq **全部 entry 通过**：

```
arc / bigint(8/8) / box / collections / const / drop / enum / error / float(10/10)
generic / hello / hrtb(1/1) / impl-trait / int / int-width(14/14) / iter / panic
rc / slice / vec
```

**部分通过**（数字为 S/总）：`aeneas-limit` 5/8、`charon-limit` 4/7、`closure-adv` 3/4、`concurrency` 1/2、`creusot-limit` 5/7、`deps-complex` 4/7（vs hax-fstar 7/7、hax-lean 7/7）、`industrial` 4/6、`kani-limit` 3/7、`lifetime` 1/3、`miri-limit` 3/7、`prusti-limit` 7/8、`repr` 1/2。

**全 FAILED**：`assoc-type`(0/1)、`closure`(0/2)、`gat`(0/1)、`hax-limit`(0/8)、`refcell`(0/1)、`trait`(0/1)、`trait-obj`(0/2)、`unsafe-adv`(0/3)、`unsafe-ptr`(0/2)。

### 失败模式归类（基于 raw stderr）

48 条 FAILED stderr 几乎全部带稳定的 `[HAX####]` 错误码 + 关联 GitHub issue 链接。按主导 reject phase 分桶：

| 桶 | 数量 | 主导 phase / 错误码 |
|---|---:|---|
| **A. `[HAX0008] reject_Dyn`** | 7 | `trait-obj/dyn-dispatch`、`charon-limit/generic-to-dyn-unsize`、`creusot-limit/dyn-trait-forbidden`、`kani-limit/stack-unwinding`、`lifetime/static-bound`、`miri-limit/thread-interleaving-partial`、`concurrency/thread-mutex` |
| **B. `[HAX0008] reject_RawOrMutPointer`** | 5 | `unsafe-ptr/raw-read`、`unsafe-ptr/raw-ptr-const`、`unsafe-adv/ptr-write`、`creusot-limit/thread-local-ref`、`kani-limit/async-await` |
| **C. `[HAX0008] reject_Unsafe`** | 8 | `hax-limit/unsafe-block`、`unsafe-adv/{transmute, maybe-uninit, ptr-write}`（部分与 B 重叠）、`miri-limit/{ffi-unshimmed-extern, soundness-not-guaranteed}`、`kani-limit/{extern-ffi, uninit-memory}`、`lifetime/thread-local`（thread_local! 宏展开内含 unsafe） |
| **D. `[HAX0003]/[HAX0006]/[HAX0010]/[HAX0011]` mut-ref 系** | 9 | `closure/fn-fnmut/{closure_fn,closure_fnmut}`、`hax-limit/{closure-mutates-outer,ret-mut-ref,mut-arg-pattern,mut-ref-alias,mut-in-assoc-type}`、`aeneas-limit/{fnmut-closure-unit-return, trait-impl-mut-param-mismatch}`、`miri-limit/simd-bitmask-large-vector` |
| **E. `[HAX0001] CfIntoMonads / FunctionalizeLoops`** | 5 | `gat/lending-iter`、`assoc-type/iter-style`、`aeneas-limit/return-inside-nested-loop`、`hax-limit/labelled-break`、`prusti-limit/loan-crosses-loop-boundary` |
| **F. `[HAX0001]/[HAX0002]` AST import 其他** | 4 | `hax-limit/let-chains`（issue #2018）、`charon-limit/{async-fn, inline-asm}`、`closure-adv/boxed-dyn-fn`（type Dyn with non-trait predicate） |
| **G. `[HAX0008] reject_ArbitraryLhs`** | 1 | `refcell/borrow/refcell_borrow_mut`（`*c.borrow_mut() = 42;`） |
| **H. `[HAX0008] reject_TraitItemDefault`** | 1 | `trait-obj/conditional-method` |
| **I. `[HAX0002] Coq printer generic printer Unreachable`** | 4 | `deps-complex/{bigint-serde, chrono-serde, collections-serde}`、`creusot-limit/dyn-trait-forbidden`：错误位置精确指向 `#[derive(..., Serialize, Deserialize, ...)]` 列表里的 `Serialize` token。**Coq backend 特定**——AST 导入成功，生成 Coq 输出时遇到引擎认为"unreachable"形态 |
| **J. industrial 工业代码 lint→error** | 2 | `industrial/x509-parser/cert-parse/{x509_parse_der, x509_subject_extensions}` |
| **K. 底层 rustc / hax frontend 栈溢出** | 1 | `trait/cyclic-bound/cyclic_bound_use`：exit 101，cargo build 阶段栈溢出 |
| **L. 其他混合（多 phase 叠加触发，按主信号归桶后剩余）** | 1 | （`unsafe-adv/transmute` 等已计入 C；本桶接受复合归桶后的剩余 entry） |

数量加总：7+5+8+9+5+4+1+1+4+2+1+1 = 48 ✓（注：多个 entry 同时触发多桶，已按主信号归桶；上面单独计数仅作示意）

stderr 形态稳定，举一例（`trait-obj/dyn-dispatch`）：

```
error: [HAX0008] explicit rejection by a phase.
This is discussed in issue https://github.com/hacspec/hax/issues/15.
Note: the error was labeled with context `reject_Dyn`.
```

`reject_Dyn` 是 hax-coq 在本矩阵下的最高频 reject signal（26 次出现，含同 entry 多触发器叠加）——hax-lean 的 Lean Printer 在 `dyn Trait` 上以 `[HAX0001] Lean Printer` 拒（hax-lean B 桶），hax-fstar 在 `dyn Trait` 上很多反而通过，三 backend 在 `dyn Trait` 处理上对称分化。

**I 桶（serde derive 在 Coq printer 阶段）**举一例：

```
[HAX0002] Fatal error: something we considered as impossible occurred!
Details: Unreachable
Note: the error was labeled with context `Coq printer generic printer`.
```

错误位置都精确指向 `#[derive(Debug, Serialize, Deserialize, PartialEq)]` 的 `Serialize` token。这是 Coq backend 特定的失败：hax engine 把 serde 的 derive 展开后的 trait impl 引到 IR，在生成 Coq 输出时遇到引擎认为"unreachable"的形态——bigint / num-rational 类外部 crate 的 trait 调用本身处理良好（`bigint/*` 8/8 全 SUCCESS），瓶颈在 `serde::Serialize` 的 derive 展开形态。

**`hax-limit/*` 8/8 全 FAILED**：8 条都按 hax 项目自己的 issue tracker 期望 fail，错误码与设计意图一致：

| entry | reject context | issue |
|---|---|---|
| `closure-mutates-outer` | `LocalMutation` | #1060 |
| `labelled-break` | `CfIntoMonads` | — |
| `let-chains` | `AST import` | #2018 |
| `mut-arg-pattern` | `AndMutDefsite` | #1405 |
| `mut-in-assoc-type` | `DirectAndMut` 系 | — |
| `mut-ref-alias` | `DirectAndMut` 系 | #420 |
| `ret-mut-ref` | `DirectAndMut` | #420 |
| `unsafe-block` | `reject_Unsafe` | — |

## 与本次测试边界的关系

- 测试切割点：`.v` 文件落盘 + cargo hax exit 0 + 产物 grep 不命中 silent marker → SUCCESS。**未触达**：`.v` 是否 `coqc` 可编译、是否需要 hax Coq 支持库（`proof-libs/coq/`）——超出测试范围
- hax-coq 在 `deps-complex/*` 上 4/7（vs hax-fstar 与 hax-lean 都 7/7）的差异**完全由 Coq printer 在 serde derive 形态上的 `Unreachable` 失败导致**。这是 Coq printer 特有的瓶颈，与 hax 公共前端能力无关——同 corpus 的非 serde-derive entry（`bigint/*` 8/8、`deps-complex/{trait-serde-generic, error-chain, itertools-multi, num-rational-arith}` 4 条）hax-coq 都接受
- 三 hax backend 的对比：在 `hax-limit/*` 上 hax-coq 0/8（全部按设计 fail）、hax-fstar 2/8、hax-lean 4/8——hax-coq 的 reject phase 阵列最完整、最严格，其他两 backend 在部分 hax-limit entry 上反而走 sentinel body（hax-lean）或部分 phase 不触发（hax-fstar）。这不是排序，只是接受范围在矩阵上不同

## 历史快照声明

本报告所有数字基于 `runs/run-1778226613-5282`（2026-05-08）+ hax `30949eb8` + nightly-2025-11-08 + opam hax-engine。Coq backend 在 hax upstream 标记为 partial development，未来 reject phase 序列变化、`.v` 输出格式变化、Coq 支持库整合状态变化都会让本快照失效。
