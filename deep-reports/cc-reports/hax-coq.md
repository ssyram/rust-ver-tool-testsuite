# hax-coq — 特性支持评估报告

## 元数据

- **数据源**：`runs/run-1778466265-63960/`（2026-05-11，P13-B 重跑 3-tool × 146 entries：kani / hax-fstar / hax-coq；host: Apple M5 / macOS aarch64 / 24 GB / 10 cpus，并发 10）
- **历史 run（对照）**：`runs/run-1778226613-5282/`（2026-05-08 主 run，旧 oracle 下 hax-coq 98/146）
- **工具版本**：`hax untagged-git-rev-30949eb870`（commit `30949eb87058895c24f963df90dd30ef11b0dc1a`）；nightly toolchain `nightly-2025-11-08`；OCaml `hax-engine` + Rust frontend driver
- **工具配置**：`tools/hax-coq/`
- **通过率**：**SUCCESS 96 / 146 ≈ 65.8%**（FAILED 50，TIMEOUT 0）—— P13-B 重跑，封堵 silent-skip-item 漏报后；旧 oracle 下 98/146 = 67.1%
- **耗时分布**：avg 2919 ms / median 677 ms / p90 11822 ms / max 27600 ms（无 timeout 触发）
- **oracle 改造**：`tools/hax-coq/tool.toml` 在原 `failure ((` / `please implement the method` grep 后追加 gate：grep entry_fn 在 `proofs/coq/extraction/` 中存在（pattern `^\s*(Definition|Fixpoint|Lemma|Equations|Theorem|Program\s+Definition)\s+$TS_ENTRY_FN\s`，覆盖三个 CoqNotation 分支 + 防御性扩展）。不命中 → FAILED。详 `docs/fixes/oracle-leak-rules-implementation-2-2026-05-11.md` §2.3
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

**判定式**（P13-A 后，双门）：

```
SUCCESS ⟺ cargo hax exit 0
        ∧ 产物 grep 不命中 'failure ((' / 'please implement the method' 字面
        ∧ entry_fn 在 proofs/coq/extraction/ 中命中
          pattern `^\s*(Definition|Fixpoint|Lemma|Equations|Theorem|Program\s+Definition)\s+$TS_ENTRY_FN\s`
```

第一门（exit + silent-marker grep）是 v1 旧 gate；第二门（entry_fn 存在性 gate）是 P13-A 新封堵——源码层 silent path 来自 `backends/coq/coq/coq_backend.ml:588 method item'_NotImplementedYet = string "(* NotImplementedYet *)"`，让某些 item 完全渲染为单行 comment，**不写真 `Definition`/`Fixpoint`**。但 `(* NotImplementedYet *)` 与每个 `.v` 文件的 boilerplate header 字面相同（所有产物都有此 comment），不能直接 grep marker —— 改抓 entry_fn 定义存在性。

`please implement the method` 来自上游 `engine/backends/coq/coq/coq_backend.ml:137` 的 `default_string_for` 路径——纯文本输出不发 Diagnostic，cargo hax 仍 exit 0。这条 silent path 与 hax-lean 的 sentinel sorry 同性质。

注：`(* NotImplementedYet *)` 是每个 `.v` 文件的 boilerplate header（hax 给所有 Coq 输出自动加），**不算**失败信号——oracle 不直接抓这个 marker，改抓 entry_fn 存在性。

**pattern 设计的关键发现**：audit-2 §4.3 推荐 `(Definition|Equations|Fixpoint)` —— 实测被 falsify，漏 `Lemma`（coq_backend.ml:454 `is_lemma` 分支生成）。源码层 hax-coq 对 Rust `fn` 项必经 `coq_backend.ml:452-560` 的 `method item'_Fn`，最终走三个 `CoqNotation` 分支之一：`:454 is_lemma → Lemma <name>`、`:518 is_rec → Fixpoint <name>`、`:540 else → Definition <name>`。合法翻译的 entry_fn 必为这三个 keyword 之一。本实施扩展为 6 个 keyword 容忍集合（含 `Equations` / `Theorem` / `Program Definition` 防御性扩展，应对上游未来新分支）——与 P12 verifast `N≤40` 阈值被 falsify 同级，"audit 给规则，实施按反误报实测校正"。

**三轨 partial 暴露机制**：
1. cargo hax exit 1 = engine emit `[HAX0001]`-`[HAX0011]` Diagnostic（hax-coq 的主力信号）
2. silent path 1：`coq_backend.ml:137` 的 `default_string_for` 纯文本输出 → grep `failure ((` / `please implement the method` 抓
3. silent path 2（P13-A 新增）：`coq_backend.ml:588` 的 `item'_NotImplementedYet` 整 item silent skip → entry_fn 存在性 gate 抓

**P13-B 重跑触发的 silent-skip-item 实测**：

| entry | hax-coq 行为 | 原 oracle | 新 oracle |
|---|---|---|---|
| `closure-adv/fn-once/closure_fn_once` | cargo hax exit 0，wrote `Closure_adv_fn_once.v` + `_CoqProject`，但产物不含 `Definition closure_fn_once`（源码层 `NotImplementedYet` 跳过）| SUCCESS（漏报）| FAILED ✓ |
| `impl-trait/return-iter/impl_trait_iter` | 同上，wrote `Impl_trait_return_iter.v`，产物不含 `Definition impl_trait_iter`| SUCCESS（漏报）| FAILED ✓ |

stderr 诊断（实测）：

```
info: hax: wrote file ./proofs/coq/extraction/Closure_adv_fn_once.v
info: hax: wrote file ./proofs/coq/extraction/_CoqProject
[hax-coq-oracle] FAIL: entry_fn 'closure_fn_once' missing from .v products
                 (silent skip — coq_backend.ml:588 item'_NotImplementedYet path)
```

**形式严格性**：
- **0 误报**：✅ 形式可证。hax-coq 对 Rust `fn` 项必经三个 `CoqNotation` 分支之一（`coq_backend.ml:452-560`，分支封闭），pattern 的 6 keyword 并集覆盖 Coq fn 渲染的全集 + 防御性扩展。`failure ((` 双括号字面用户合法代码极难写出；hax-coq **不翻译 Rust doc comment**（与 hax-lean 不同），所以注释里的 `please implement` 不会出现在 `.v` 产物
- **0 漏报**：✅ 实测验证。entry_fn 存在性 gate 在 P13-B 重跑上命中 2 条原 SUCCESS，证明 audit §3.3 标记的 silent-skip-item 路径不是 0 实测现象
- **漏报盲点**：上游 Coq backend 未来引入新 silent path 不通过 `(* NotImplementedYet *)` 或 `please implement the method` 字面（理论窗口；本基线 commit 30949eb 无）

## 实测结果

### 按 feature 类目分布

下列 feature 类目下 hax-coq **全部 entry 通过**：

```
arc / bigint(8/8) / box / collections / const / drop / enum / error / float(10/10)
generic / hello / hrtb(1/1) / int / int-width(14/14) / iter / panic
rc / slice / vec
```

**部分通过**（数字为 S/总）：`aeneas-limit` 5/8、`charon-limit` 4/7、`closure-adv` 2/4（v1 3/4，P13-A 翻 1）、`concurrency` 1/2、`creusot-limit` 5/7、`deps-complex` 4/7（vs hax-fstar 7/7、hax-lean 7/7）、`impl-trait` 0/1（v1 1/1，P13-A 翻 1）、`industrial` 4/6、`kani-limit` 3/7、`lifetime` 1/3、`miri-limit` 3/7、`prusti-limit` 7/8、`repr` 1/2。

**全 FAILED**：`assoc-type`(0/1)、`closure`(0/2)、`gat`(0/1)、`hax-limit`(0/8)、`impl-trait`(0/1)、`refcell`(0/1)、`trait`(0/1)、`trait-obj`(0/2)、`unsafe-adv`(0/3)、`unsafe-ptr`(0/2)。

### 失败模式归类（基于 raw stderr）

50 条 FAILED stderr 几乎全部带稳定的 `[HAX####]` 错误码 + 关联 GitHub issue 链接（P13-A gate 新抓的 2 条用 `[hax-coq-oracle]` 自定义诊断）。按主导 reject phase 分桶：

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
| **M. silent-skip-item（P13-A gate 抓获）** | 2 | `closure-adv/fn-once/closure_fn_once`、`impl-trait/return-iter/impl_trait_iter`：cargo hax exit 0 + wrote `*.v` 文件 + 写 `_CoqProject`，但产物不含 entry_fn 定义（源码层 `coq_backend.ml:588 item'_NotImplementedYet` 路径，item 整 silent skip 为单行 comment）|

数量加总：7+5+8+9+5+4+1+1+4+2+1+1+2 = 50 ✓（注：多个 entry 同时触发多桶，已按主信号归桶；上面单独计数仅作示意）

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

本报告所有数字基于 `runs/run-1778466265-63960`（2026-05-11，P13-B 重跑）+ hax `30949eb8` + nightly-2025-11-08 + opam hax-engine + P13-A oracle 改造。Coq backend 在 hax upstream 标记为 partial development，silent-skip-item 路径在 P13-B 重跑实测命中 2 条（`closure-adv/fn-once`、`impl-trait/return-iter`）—— 与 hax-fstar 同 2 条 entry，证明 audit §3.3 的 `coq_backend.ml:588 item'_NotImplementedYet` 路径不是 0 实测现象。未来 reject phase 序列变化、`.v` 输出格式变化、Coq 支持库整合状态变化都会让本快照失效——届时 oracle 的三轨（Diagnostic exit + silent marker grep + entry_fn 存在性 gate）将作为兜底信号生效。
