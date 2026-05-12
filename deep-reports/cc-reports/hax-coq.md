# hax-coq — 特性支持评估报告（v6 final post-P35 baseline）

## 元数据

- **数据源**：`runs/run-1778560393-59119/`（v6 final post-P35，2026-05-12T04:33:13Z 起跑 / 04:55:27Z 结束；合并 verus rerun + R7 5-tool rerun + P33 [env] schema 治源后 x509 重跑 + P35 bug-detect 派生 wrapper 切换的统一 161-entry corpus）
- **历史对照 run**：`runs/run-1778466265-63960/`（v5.1，2026-05-11，P13-B 重跑，146 entry，hax-coq 96/146）
- **工具版本**：`hax untagged-git-rev-30949eb870`（commit `30949eb87058895c24f963df90dd30ef11b0dc1a`），nightly toolchain `nightly-2025-11-08`，OCaml `hax-engine` + Rust frontend driver
- **工具配置**：`tools/hax-coq/`（`tool.toml` + `README.md`）
- **本工具实测**：n=161 / **SUCCESS=113** / FAILED=48 / UNKNOWN=0，**通过率 70.2%**（113/161）
- **时长分布**：avg 3159 ms / median 1427 ms / p90 5642 ms / max 45773 ms（无 timeout 触发；max 由 P33 治源后真跑通的 x509 entry 贡献）
- **宪法 baseline**：`principles.md` v8（P27 双根本问题 + P31 法律传导 + P33 [env] schema + P34 §六 当前 crate 焦点 / 前端测量双切割 + P35 bug-detect=SUCCESS 派生原则）+ `tool-integration.md` §四.5 "我们 wrapper vs 官方 wrapper" 归因判据
- **运行环境**：Apple M5 / macOS aarch64 / 24 GB / 10 cpus，并发 10
- **时效声明**：本快照锚定上述 run id + hax commit + nightly 工具链 + corpus，不构成长期承诺。Coq backend 在 hax upstream 标记为 partial development；上游对 Coq Printer 的 reject phase 序列或 silent-skip 路径改动会让本快照失效。

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

**前端 / 后端切割**（按 P34 §六 深度切割精神）：纯翻译工具，pipeline 终点是 `.v` 文件 + `_CoqProject` 落盘。下游 `coqc` type-check / 用户证明 / 与 hax Coq 支持库整合**全部不在测试范围**。`_CoqProject` 中库名为字面 `TODO`（`-R ./ TODO`）是 upstream 预期行为。

**当前 crate 焦点**（按 P34 §六 宽度切割精神）：测量对象是 example 的 entry crate，runner 注入的 `TS_TARGET_CRATE` / `TS_ENTRY_FN` 锚点指向的部分。外部 deps（serde / num-bigint / chrono / x509-parser 自身）若被工具 silently opaque / stub，**不**触发 partial 判定；只有 entry crate 自身 fn / type / trait 被 silent-skip 才算 partial。

**项目维护的 wrapper 范围**：`tools/hax-coq/tool.toml` 直接用 `sh -c` 内联包了 `cargo hax` 命令 + oracle 双门（grep silent marker / entry_fn 存在性 gate）。**没有独立 wrapper 脚本**——oracle 逻辑全在 `tool.toml` 单行 shell 里。`tool.toml` 内联 shell 失败 = 我们这边责任（按 §四.5 "最近责任主体"切）。

Coq backend 在三 hax backend 中 **reject phase 数量最多**——本快照实测观察到 9 类 phase 触发：`reject_Unsafe`、`reject_RawOrMutPointer`、`reject_Arbitrary_lhs`、`reject_Dyn`、`reject_TraitItemDefault`、`AndMutDefsite`、`DirectAndMut`、`LocalMutation`、`CfIntoMonads/FunctionalizeLoops`。这是 hax-coq 通过率（70.2%）低于 hax-fstar / hax-lean 的结构性原因。

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
  - **P35 bug-detect = SUCCESS 派生不适用于本工具**：hax-coq 是纯翻译工具，pipeline 终点是 `.v` 文件落盘，不进求解层，没有"工具自陈发现 bug"的语义渠道——P35 派生（MIRI / soteria 类把 UB 检出翻 SUCCESS）在 hax-coq 不触发，本盲点声明不变更

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

### 桶 6：`[HAX0002]` Coq printer "Unreachable" — serde derive 形态（6 case）— 工具不支持（Coq backend 特定）

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

## 漏报盲点（诚实声明）

- 已通过 wrapper gate 封堵：
  - silent path A（`coq_backend.ml:137` `please implement the method` 字面文本）→ 产物 grep
  - silent path B（`coq_backend.ml:588` `item'_NotImplementedYet` item-level skip）→ entry_fn 定义存在性 gate（覆盖 6 个 Coq keyword）
- 仍存在的盲点：
  - 上游 hax-engine 未来若引入第三类 silent path（如新增 `string "..."` 分支但 marker 字面不属 A / B 两个）→ 当前 oracle 看不见。监控点：`engine/backends/coq/coq/coq_backend.ml` upstream diff
  - hax-engine OCaml ICE 若回归出现 silent fallback（exit 0 + 部分 .v 落盘）→ 当前 oracle 主信号路径会漏。当前实测无此现象，列为理论盲点
  - P35 bug-detect 派生不适用：hax-coq 无求解层，不存在 UB 检出 → SUCCESS 渠道

## v5.1 → v6 final post-P35 ΔS 解释

- **v5.1**：96/146 = 65.8%（旧 corpus 146 entry）
- **v6 final post-P35**：113/161 = 70.2%（新 corpus 161 entry）
- **ΔS = +17 SUCCESS**：
  - +15 来自新增的 `runnable/*` 子集（15 个微型可运行算术/逻辑 entry，hax-coq 全 SUCCESS）
  - **+2 来自 P33 [env] schema 治源**：`industrial/x509-parser/cert-parse/{x509_parse_der, x509_subject_extensions}` 通过 `hirusttest.toml [env] RUSTFLAGS=--cap-lints=warn` 让 vendor crate 的 nightly lint 降级为 warning，cargo build 成功，hax frontend 真正跑通整个 x509-parser 翻译并对 entry crate 产出合规 `.v`（duration 45 s 左右，最慢两条），**从 v5.1 的 FAILED → v6 的 SUCCESS**（v6 中间态曾短暂 UNKNOWN，P33 后稳定 SUCCESS）
- **P35 bug-detect 派生对 hax-coq 无作用**：hax-coq 是纯翻译工具，pipeline 不进求解层，没有"工具自陈检出 bug"语义渠道——P35 影响 MIRI / soteria，不影响本工具
- 通过率上升 65.8% → 70.2%，主要由 runnable 子集 + P33 治源后两条 x509 真翻共同贡献；非 runnable 非 industrial 部分 FAILED 阵列基本稳定

## 修订建议清单（仅"我们导致"失败）

**无需修订**。

P33 [env] schema 已通过 `RUSTFLAGS=--cap-lints=warn` 治源解决"vendored x509-parser 在 nightly-2025-11-08 lint→error 阻断 cargo build"的我们导致问题——v5.1 时代的桶 11（vendored crate × nightly 兼容性悬挂）在 v6 final post-P35 baseline 已消失，相应 2 条 x509 entry 真翻 SUCCESS。

剩余 48 条 FAILED（桶 1-10）全部为 hax-coq 工具能力边界（unsafe / dyn / mut-ref / 控制流 / Coq printer Unreachable / 上游 ICE / rustc 栈溢出 / silent-skip-item），按宪法 §一 本地性原则 FAILED 站得住，工具开发者不能驳回，**不修**。

## 与本次测试边界的关系

- 测试切割点（按 P34 §六 深度切割）：`.v` 文件落盘 + cargo hax exit 0 + 产物 grep 不命中 silent marker + entry_fn 存在性 gate → SUCCESS。**未触达**：`.v` 是否 `coqc` 可编译、是否需要 hax Coq 支持库（`proof-libs/coq/`）整合——超出测试范围
- 当前 crate 焦点（按 P34 §六 宽度切割）：x509 case 中 hax-coq 把 `x509-parser` 上千行翻译为 `.v` 之事——entry crate `cert-parse` 自身 `x509_parse_der` / `x509_subject_extensions` 两条 fn 在产物中 entry_fn gate 命中 → SUCCESS；外部 deps（`nom` / `der-parser` / `oid-registry` 等）即便被部分 stub / opaque 亦不影响判定
- 三 hax backend 的 reject 阵列差异：hax-coq 9 类 reject phase 最完整最严格；同 corpus 下 hax-coq < hax-fstar < hax-lean 通过率。但这不构成"能力排序"——只是 backend 设计阶段的接受范围不同（按宪法 §四 原则 B 不区分翻译深浅 / 不做工具间排序，本节仅作 backend 对比观察）
- `hax-limit/*` 8/8 全 FAILED：8 条都按 hax 项目自己的 issue tracker 期望 fail，错误码（`LocalMutation` / `CfIntoMonads` / `AST import` / `AndMutDefsite` / `DirectAndMut` / `reject_Unsafe`）与设计意图一致

## 历史快照声明

本报告所有数字基于 `runs/run-1778560393-59119`（v6 final post-P35，2026-05-12T04:33:13Z 起跑）+ hax `30949eb8` + nightly-2025-11-08 + opam `hax-engine` + P13-A oracle 双门 + P27/P31/P33/P34/P35 累积派生原则。

Coq backend 在 hax upstream 标记为 partial development，silent-skip-item 路径在 v6 重跑实测命中 2 条（`closure-adv/fn-once`、`impl-trait/return-iter`），与 v5.1 一致——证明 audit-2 §3.3 的 `coq_backend.ml:588 item'_NotImplementedYet` 路径不是 0 实测现象。

P33 [env] schema 治源后 x509 entry 真翻 SUCCESS 是 v6 final post-P35 与 v5.1 / v6 中间态最显著的能力快照差异，须配套时空锚定理解：(a) RUSTFLAGS=--cap-lints=warn 是 corpus-side 信号文件声明，不改 example 字节；(b) 未来 nightly 进一步把 cap-lints 不可控制的 lint 升级到 error 时，本治源失效。

未来 reject phase 序列变化、`.v` 输出格式变化、Coq printer Unreachable bug 修复、Coq 支持库整合状态变化、`coq_backend.ml` silent path 新增、nightly toolchain lint policy 变化都会让本快照失效——届时 oracle 的双门（exit + silent marker grep + entry_fn 存在性 gate）将作为兜底信号生效。

锚定免责：详见仓库 `README.md` 顶部一次性声明。
