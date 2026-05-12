# hax-coq — 特性支持评估报告（v6 baseline）

## 元数据

- **数据源**：`runs/run-1778560393-59119/`（v6 final，2026-05-12T04:33:13Z 起跑；合并 verus rerun + R7 5-tool rerun 的统一 161-entry corpus）
- **历史对照 run**：`runs/run-1778466265-63960/`（v5.1，2026-05-11，P13-B 重跑，146 entry，hax-coq 96/146）
- **工具版本**：`hax untagged-git-rev-30949eb870`（commit `30949eb87058895c24f963df90dd30ef11b0dc1a`），nightly toolchain `nightly-2025-11-08`，OCaml `hax-engine` + Rust frontend driver
- **工具配置**：`tools/hax-coq/`（`tool.toml` + `README.md`）
- **本工具实测**：n=161 / **SUCCESS=111** / FAILED=48 / UNKNOWN=2，**通过率 68.9%**（111/161）
- **时长分布**：avg 2850 ms / median 1427 ms / p90 5642 ms / max 21244 ms（无 timeout 触发）
- **宪法 baseline**：`principles.md` v8（双根本问题 / 本地性 / UNKNOWN 严格语义两类）+ `tool-integration.md` §四.5 "我们 wrapper vs 官方 wrapper" 归因判据
- **运行环境**：Apple M5 / macOS aarch64 / 24 GB / 10 cpus，并发 10
- **时效声明**：本快照锚定上述 run id + hax commit + nightly 工具链 + corpus，不构成长期承诺。Coq backend 在 hax upstream 标记为 partial；上游对 Coq Printer 的 reject phase 序列或 silent-skip 路径改动会让本快照失效。

## 工具内部 pipeline + 前端边界

```
rustc + driver-hax-frontend-exporter (Rust)
  → THIR JSON
  → hax-engine (OCaml)
  → phase pipeline (reject phases + 改写 passes)
  → Coq Printer (OCaml → Coq/Rocq 文本生成)
  → 写出 <work>/proofs/coq/extraction/<Crate>.v + _CoqProject
```

本测试关心 `cargo +nightly-2025-11-08 hax -C --lib ';' into coq` 的前端通过。

**前端 / 后端切割**：纯翻译工具，pipeline 终点是 `.v` 文件 + `_CoqProject` 落盘。下游 `coqc` type-check / 用户证明 / 与 hax Coq 支持库整合**全部不在测试范围**。`_CoqProject` 中库名为字面 `TODO`（`-R ./ TODO`）是 upstream 预期行为。

**项目维护的 wrapper 范围**：`tools/hax-coq/tool.toml` 直接用 `sh -c` 内联包了 `cargo hax` 命令 + oracle 双门（grep silent marker / entry_fn 存在性 gate）。**没有独立 wrapper 脚本**——oracle 逻辑全在 `tool.toml` 单行 shell 里。`tool.toml` 内联 shell 失败 = 我们这边责任（按 §四.5 "最近责任主体"切）。

Coq backend 在三 hax backend 中 **reject phase 数量最多**——本快照实测观察到 9 类 phase 触发：`reject_Unsafe`、`reject_RawOrMutPointer`、`reject_Arbitrary_lhs`、`reject_Dyn`、`reject_TraitItemDefault`、`AndMutDefsite`、`DirectAndMut`、`LocalMutation`、`CfIntoMonads/FunctionalizeLoops`。这是 hax-coq 通过率（68.9%）低于 hax-fstar / hax-lean 的结构性原因。

## SUCCESS 信号 + 形式严格性

**判定式**（P13-A 双门 oracle）：

```
SUCCESS ⟺ cargo hax exit 0
        ∧ proofs/coq/extraction/ 不命中 'failure ((' / 'please implement the method'
        ∧ entry_fn 在 .v 产物中命中
          pattern `^\s*(Definition|Fixpoint|Lemma|Equations|Theorem|Program\s+Definition)\s+$TS_ENTRY_FN\s`
```

**主信号通路**：`cargo hax` exit code（exit 0 → engine 未 emit Diagnostic）。

**wrapper 补抓通路**（`tool.toml` 内联）：
1. silent path A：`coq_backend.ml:137` `default_string_for "TODO: please implement..."` → grep 字面抓
2. silent path B：`coq_backend.ml:588` `item'_NotImplementedYet = string "(* NotImplementedYet *)"` 让某些 item 渲染为单行 comment，不写 `Definition` —— 但 `(* NotImplementedYet *)` 与每 `.v` boilerplate header 字面相同，不能直接 grep marker。改抓 entry_fn 定义存在性（pattern 容忍 6 个 keyword：`Definition`/`Fixpoint`/`Lemma`/`Equations`/`Theorem`/`Program Definition`，覆盖 `coq_backend.ml:454/518/540` 三个 CoqNotation 分支 + 防御性扩展）

**形式严格性（按 P30 后语义）**：

- **0 误报**：实测 + 设计层论证。entry_fn 存在性 pattern 覆盖 hax-coq 源码层 `coq_backend.ml:452-560 method item'_Fn` 的全部三个 CoqNotation 分支封闭集合 + 容忍 3 个上游未来可能引入的 keyword。`failure ((` 双括号字面在合法 Rust→Coq 翻译中难产生；hax-coq 不翻译 Rust doc comment（与 hax-lean 不同），所以注释里 `please implement` 不会出现在 `.v` 产物。v5.1 时代双向实测 5 类 case（Definition / Fixpoint / Lemma / 嵌套 module / missing），见 `docs/fixes/oracle-leak-rules-implementation-2-2026-05-11.md` §2.3
- **0 漏报**：实测 + 源码层封堵。silent path A/B 都有 oracle gate；v5.1→v6 持续 stable，本次 v6 重跑 P13-A gate 仍命中 2 条 silent-skip-item entry（`closure-adv/fn-once`、`impl-trait/return-iter`），与 v5.1 测出的同 2 条一致——证明 audit-2 §3.3 标记的源码层 silent path 非空
- **漏报盲点（诚实声明）**：
  - 上游 Coq backend 未来引入新 silent path 不通过 `(* NotImplementedYet *)` / `please implement the method` / 不通过 silent-skip-item（理论窗口；本基线 commit 30949eb 已穷举两条）
  - hax-engine OCaml 端 internal compiler error 类（如 `repr/union`）：当前 oracle 通过 exit ≠ 0 抓，正常；但若上游某次回归让 ICE 走 silent fallback 渲染为部分 `.v` 落盘 + exit 0，会绕过 oracle —— 当前无证据该路径存在，但属未来观察盲点

## 失败分桶（按 P31 §四.5 归因分类）

下表所有 48 个 FAILED entry 按"主导 reject phase / 错误码"归桶（多 phase 叠加按主信号归一桶；可能少量 entry 在多桶都有触发器，本表按 stderr 首条 reject context 归类）。

### 桶 1：`reject_Unsafe`（8 case）— 工具不支持

代表 entry：`hax-limit/unsafe-block`、`unsafe-adv/{transmute, maybe-uninit, ptr-write}`、`unsafe-ptr/raw-read`、`miri-limit/{ffi-unshimmed-extern, soundness-not-guaranteed}`、`kani-limit/{extern-ffi, uninit-memory}`、`lifetime/thread-local`（`thread_local!` 宏展开内含 unsafe）

stderr 特征：

```
error: [HAX0008] Explicit rejection by a phase in the Hax engine:
Note: the error was labeled with context `reject_Unsafe`.
```

**归因**：工具不支持。hax-coq pipeline 设计明示拒绝 `unsafe` block。
**处理**：不修。本地性原则下 FAILED 站得住。

### 桶 2：`reject_Dyn`（6 case）— 工具不支持

代表 entry：`trait-obj/dyn-dispatch`、`charon-limit/generic-to-dyn-unsize`、`creusot-limit/dyn-trait-forbidden`、`kani-limit/stack-unwinding`、`lifetime/static-bound`、`miri-limit/thread-interleaving-partial`

stderr 特征：`Note: the error was labeled with context `reject_Dyn``，issue https://github.com/hacspec/hax/issues/15

**归因**：工具不支持。Coq backend 显式拒绝 `dyn Trait`，这是上游设计选择。
**处理**：不修。

### 桶 3：`reject_RawOrMutPointer`（1 case，partial 与桶 1 重叠）— 工具不支持

唯一独立 entry：`unsafe-adv/ptr-write`（其它 raw-ptr entry 主信号是 `reject_Unsafe`）

**归因**：工具不支持原始指针。
**处理**：不修。

### 桶 4：mut-ref 系（HAX0003 / HAX0006 / HAX0010 / HAX0011 + DirectAndMut / AndMutDefsite / LocalMutation 等）（13 case）— 工具不支持

包含错误码 `[HAX0003]`（8 次）/ `[HAX0010]`（3 次）/ `[HAX0011]`（1 次）/ `[HAX0006]`（计入散落） + reject context `DirectAndMut` / `AndMutDefsite` / `LocalMutation`。

代表 entry：`closure/fn-fnmut/{closure_fn, closure_fnmut}`、`hax-limit/{closure-mutates-outer, ret-mut-ref, mut-arg-pattern, mut-ref-alias, mut-in-assoc-type}`、`aeneas-limit/{fnmut-closure-unit-return, trait-impl-mut-param-mismatch}`、`miri-limit/simd-bitmask-large-vector`、`refcell/borrow/refcell_borrow_mut`、`concurrency/thread-mutex/thread_mutex_join`

**归因**：工具不支持。hax pipeline 对 mut 引用、`&mut` 返回、可变闭包捕获等有系统性 reject。
**处理**：不修。

### 桶 5：`[HAX0001]` 控制流 / AST import 类（8 case）— 工具不支持

代表 entry：`gat/lending-iter`、`assoc-type/iter-style`、`aeneas-limit/return-inside-nested-loop`、`hax-limit/{labelled-break, let-chains}`、`prusti-limit/loan-crosses-loop-boundary`、`charon-limit/async-fn`、`creusot-limit/thread-local-ref`、`lifetime/thread-local`

stderr 特征：`[HAX0001] something is not implemented yet.` + 具体说明（如 `Let-chains ... are not supported, issue #2018`、`CfIntoMonads` / `FunctionalizeLoops`）

**归因**：工具不支持。
**处理**：不修。

### 桶 6：`[HAX0002]` Coq printer "Unreachable" — serde derive 形态（4 case）— 工具不支持（Coq backend 特定）

代表 entry：`deps-complex/{bigint-serde, chrono-serde, collections-serde}`、`unsafe-ptr/raw-ptr-const/raw_ptr_const_match`、`closure-adv/boxed-dyn-fn`、`charon-limit/inline-asm`、`kani-limit/async-await`

stderr 特征：

```
error: [HAX0002] Fatal error: something we considered as impossible occurred!
Details: Unreachable
Note: the error was labeled with context `Coq printer generic printer`.
```

错误位置精确指向 `#[derive(Debug, Serialize, Deserialize, PartialEq)]` 的 `Serialize` token（serde 类）或其它 Coq printer 走不通的 AST 形态。这是 **Coq backend 特定瓶颈**：AST 导入成功，Coq 输出生成时遇到 printer 视为 "unreachable" 的形态。同 corpus 的非 serde-derive entry（`bigint/*` 8/8、`deps-complex/{trait-serde-generic, error-chain, itertools-multi, num-rational-arith}` 4 条 SUCCESS）都通过——说明瓶颈是 `serde::Serialize` derive 展开形态 + 少量其它 Coq printer fragility，不是公共前端能力问题。

**归因**：工具不支持（Coq backend printer 的 Unreachable bug，属上游设计/实现）。
**处理**：不修。

### 桶 7：`reject_Arbitrary_lhs` / `reject_TraitItemDefault`（2 case）— 工具不支持

`refcell/borrow/refcell_borrow_mut`（`*c.borrow_mut() = 42;` LHS 形态）已归桶 4；`trait-obj/conditional-method/conditional_method` `reject_TraitItemDefault`。

**归因**：工具不支持。
**处理**：不修。

### 桶 8：hax-engine OCaml ICE（1 case）— 工具不支持

唯一 entry：`repr/union/repr_union`

stderr 特征：

```
Called from Hax_engine__Phase_utils.BindPhase.ditems in file "lib/phase_utils.ml", line 299
...
error: hax: hax-engine exited with non-zero code 2
```

OCaml 端 phase pipeline 在 `repr(union)` 类型上 ICE，是工具自身实现 bug。

**归因**：工具不支持（hax-engine ICE = 工具锅）。
**处理**：不修。本地性原则下 FAILED 站得住。

### 桶 9：rustc 栈溢出（1 case）— 工具不支持

唯一 entry：`trait/cyclic-bound/cyclic_bound_use`

stderr：`thread 'rustc' has overflowed its stack` + `process didn't exit successfully: ... driver-hax-frontend-exporter ... (signal: 6, SIGABRT)`，exit 101。

**归因**：本地性原则下，nightly-2025-11-08 + hax frontend driver 在此 cyclic trait bound 上 rustc 栈溢出 = 工具链组合栈溢出 = 工具不支持。
**处理**：不修。

### 桶 10：silent-skip-item（oracle gate 抓获）（2 case）— 工具不支持（产物层 partial）

entry：`closure-adv/fn-once/closure_fn_once`、`impl-trait/return-iter/impl_trait_iter`

stderr 特征：cargo hax exit 0 + `info: hax: wrote file ./proofs/coq/extraction/X.v` + `info: hax: wrote file ./proofs/coq/extraction/_CoqProject` + oracle 自定义诊断：

```
[hax-coq-oracle] FAIL: entry_fn 'closure_fn_once' missing from .v products
(silent skip — coq_backend.ml:588 item'_NotImplementedYet path)
```

**归因**：工具不支持。hax-engine 走 `coq_backend.ml:588 item'_NotImplementedYet` 路径让该 item silent 渲染为单行 comment，不写 `Definition`/`Fixpoint`/`Lemma`。oracle 双门第二门正确抓住——这是 oracle 设计点，不是 FAILED 异常。
**处理**：不修。

### 桶 11：vendored 第三方 crate 在 nightly-2025-11-08 lint→error（2 case，UNKNOWN）— 我们导致

entry：`industrial/x509-parser/cert-parse/{x509_parse_der, x509_subject_extensions}`

stderr 特征：

```
warning: `x509-parser` (lib) generated 121 warnings
error: could not compile `x509-parser` (lib) due to 7 previous errors; 121 warnings emitted
warning: hax: running `cargo build` was not successful, continuing anyway.
```

exit 101。我们 vendored 进 `vendor/x509-parser` 的 crate 在 nightly-2025-11-08 上触发新启用为 error 的 lint（"hiding a lifetime that's elided elsewhere is confusing"），让 cargo build 阶段失败，hax frontend 走不到。

**归因**：我们 corpus / vendored crate 与所选 nightly 不兼容（§六 (b) 类）。属"我们这边可识别 + 暂未修"。v6 baseline 已升 UNKNOWN（v5.1 时为 FAILED，本次重分类）。
**处理**：**修**。见修订建议清单。

## v5.1 → v6 ΔS 解释

- **v5.1**：96/146 = 65.8%（旧 corpus 146 entry）
- **v6**：111/161 = 68.9%（新 corpus 161 entry）
- **ΔS = +15 SUCCESS**：全部来自新增的 `runnable/*` 子集（15 个微型可运行算术/逻辑 entry，hax-coq 全 SUCCESS）
- **2 个 industrial entry 重分类**：`industrial/x509-parser/cert-parse/*` 从 FAILED 升 UNKNOWN（因为是我们 vendored crate + nightly lint 触发，属 §六 (b) 类），FAILED 计数 50→48，SUCCESS 不变
- 通过率轻微上升 65.8% → 68.9%，主要由 runnable 子集 SUCCESS 拉升；非 runnable 部分 FAILED 阵列基本稳定

## 修订建议清单（仅"我们导致"失败）

| # | 桶 | 涉及 case | 修复方案 | 优先级 |
|---|---|---|---|---|
| 1 | 桶 11 vendored x509-parser lint→error | `industrial/x509-parser/cert-parse/x509_parse_der`、`industrial/x509-parser/cert-parse/x509_subject_extensions`（2 case，当前 UNKNOWN）| 二选一：(a) bump `vendor/x509-parser` 到一个修好 lifetime elision 的上游 commit；(b) 在 `tool.toml` / 该 entry 的 `Cargo.toml` 注入 `RUSTFLAGS=-A elided_named_lifetimes` 或 `[lints.rust]` allow 该 lint；(c) 接受现状 + 在 corpus 该 sub-test 标注 "vendored crate × nightly 兼容性悬挂"。**注**：这与 industrial 子集的"通过工业代码看前端"目的相关，宪法不强制修——但若想看 hax-coq 在 x509-parser 上的真正前端表现，需先修通 cargo build 阶段 | 中 |

**"我们导致" fix 项总数：1 类（涉及 2 个 entry）**。

所有其它 FAILED（46 case，桶 1-10）均为 hax-coq 工具能力边界，按本地性原则 FAILED 站得住，不修。

## 与本次测试边界的关系

- 测试切割点：`.v` 文件落盘 + cargo hax exit 0 + 产物 grep 不命中 silent marker + entry_fn 存在性 gate → SUCCESS。**未触达**：`.v` 是否 `coqc` 可编译、是否需要 hax Coq 支持库（`proof-libs/coq/`）整合——超出测试范围
- 三 hax backend 的 reject 阵列差异：hax-coq 9 类 reject phase 最完整最严格；同 corpus 下 hax-coq < hax-fstar < hax-lean 通过率。但这不构成"能力排序"——只是 backend 设计阶段的接受范围不同（按宪法 §四 原则 B 不区分翻译深浅 / 不做工具间排序，本节仅作 backend 对比观察）
- `hax-limit/*` 8/8 全 FAILED：8 条都按 hax 项目自己的 issue tracker 期望 fail，错误码（`LocalMutation` / `CfIntoMonads` / `AST import` / `AndMutDefsite` / `DirectAndMut` / `reject_Unsafe`）与设计意图一致

## 历史快照声明

本报告所有数字基于 `runs/run-1778560393-59119`（v6 final，2026-05-12T04:33:13Z）+ hax `30949eb8` + nightly-2025-11-08 + opam `hax-engine` + P13-A oracle 双门 + P31 法律传导（UNKNOWN 严格语义两类）。

Coq backend 在 hax upstream 标记为 partial development，silent-skip-item 路径在 v6 重跑实测命中 2 条（`closure-adv/fn-once`、`impl-trait/return-iter`），与 v5.1 一致——证明 audit-2 §3.3 的 `coq_backend.ml:588 item'_NotImplementedYet` 路径不是 0 实测现象。

未来 reject phase 序列变化、`.v` 输出格式变化、Coq printer Unreachable bug 修复、Coq 支持库整合状态变化都会让本快照失效——届时 oracle 的双门（exit + silent marker grep + entry_fn 存在性 gate）将作为兜底信号生效。

锚定免责：详见仓库 `README.md` 顶部一次性声明。
