# hax-fstar — 特性支持评估报告（v6 final post-P35）

## 元数据

- **数据源**：`runs/run-1778560393-59119/`（2026-05-12 v6 final post-P33 [env] 治源 + post-P35 bug-detect；run started 2026-05-12T04:33:13Z / finished 04:55:27Z）
- **工具配置**：`tools/hax-fstar/`
- **工具版本**：`hax untagged-git-rev-30949eb870`（commit `30949eb87058895c24f963df90dd30ef11b0dc1a`）；nightly toolchain `nightly-2025-11-08`；OCaml `hax-engine` + Rust frontend driver `driver-hax-frontend-exporter`
- **本工具实测**：n=161 / SUCCESS=130 / FAILED=31 / UNKNOWN=0，通过率 **80.7%**（130/161）
- **时长分布**：avg 2966 ms / median 1138 ms / p90 6726 ms / p95 9462 ms / max 47196 ms（max 来自 x509 vendored crate 完整 cargo build；无 timeout 触发，上限 300 s 远未触达）
- **宪法 baseline**：`principles.md` v8（P27 修宪后 / P31 法律传导后 / P33 [env] schema 治源后 / P35 bug-detect 后）。UNKNOWN 严格语义两类：(a) 全局工具链崩溃；(b) 我们这边可识别且暂未修问题。工具自身能力边界一律 FAILED。
- **时效声明**：本快照锚定上述 run id + 工具 commit + nightly 工具链 + corpus，不构成长期承诺。三个 hax backend 共享同一 `hax-engine` OCaml binary，但 F\* Printer 与 Lean / Coq Printer 是各自独立 OCaml 模块，未来上游对各 printer 拒绝边界改动会让本快照失效。

## pipeline + 前端边界

```
rustc + driver-hax-frontend-exporter
  → THIR JSON
  → hax-engine（OCaml，phase pipeline：reject phases + 改写 passes）
  → F* Printer（OCaml→F* 文本生成）
  → 写出 <work>/proofs/fstar/extraction/<Crate>.fst
```

测试切割点：`cargo +nightly-2025-11-08 hax -C --lib ; into fstar` exit 0 + 产物双门 grep gate。**前端 / 后端切割**：hax 是纯翻译工具，pipeline 终点是 `.fst` 落盘——下游 F\* type-check / Karamel / 用户证明完全不在测试范围。

**工具本身 vs 项目维护成分**：
- 工具自身：`cargo hax` 二进制 + `driver-hax-frontend-exporter` + `hax-engine`（含 F\* Printer，`backends/fstar/fstar_backend.ml`）—— 全部来自上游 `hacspec/hax` repo
- 项目维护：仅 `tools/hax-fstar/tool.toml` 内的一段 shell oracle 包装（grep 产物 silent-marker + entry_fn 存在性 gate）。无独立 wrapper 脚本。oracle 失败按"最近责任主体"切（`tool-integration.md` §四.5）：grep gate 报错 = 工具 silent path 命中 → FAILED（工具锅）；shell 包装本身语法 / IO 错才属"我们 wrapper bug"

差异于 hax-lean：F\* backend 在 hax 中较成熟，phase pipeline 上走的 reject phases 更多（含 `reject_RawOrMutPointer` / `reject_ArbitraryLhs` / `reject_TraitItemDefault` 等显式拒收节点），**几乎不走 silent path**——unsupported 都让 cargo hax 在 phase 阶段 emit `[HAX####]` Diagnostic 并 exit 1。

## SUCCESS 信号 + 形式严格性

**判定式**（双门）：

```
SUCCESS ⟺ cargo hax exit 0
        ∧ 产物 grep 不命中 'Rust_primitives.Hax.failure' / 'failure ((' 字面
        ∧ entry_fn 在 proofs/fstar/extraction/ 中命中
          pattern `^(let\s+(rec\s+)?|and\s+)$TS_ENTRY_FN\s`
```

**主信号通路**：cargo hax exit 0 / 1。F\* backend 在 unsupported entry 上稳定 exit 1，是主要信号。

**wrapper 补抓通路**（tool.toml 内 shell gate）：
1. 产物 grep `Rust_primitives.Hax.failure` / `failure ((`——抓 `hax_failure_expr` 渲染为 fstar literal 的潜在 silent path
2. 产物 grep entry_fn `^(let\s+(rec\s+)?|and\s+)<entry_fn>\s`——封堵源码层 `backends/fstar/fstar_backend.ml:1771 | Use _ | NotImplementedYet -> []` 路径（让某些 item 完全不写产物且不发 Diagnostic）。三分支并集覆盖 NoLetQualifier / mutual-rec `let rec` / mutual-rec `and` 三种 F\* fn 渲染形态

**形式严格性 — 0 误报**（不冤枉能力）：实测 + 设计层论证。规则 1 / 2 是 exit 信号 + 已知 silent literal 字面 grep；规则 3 的反误报由 fstar_backend.ml:1112 / 1923-1924 单一渲染入口保证：合法翻译的 entry_fn 必为 `let` / `let rec` / `and` 之一。本 v6 run 在 130 个 SUCCESS 上未观测 gate 误判。

**形式严格性 — 0 漏报**：实测 + 源码层封堵到当前已知 silent path；不构成形式可证。规则 3 抓的 silent-skip-item 是 fstar_backend.ml:1771 的唯一已知 silent path。本 v6 run 在 `closure-adv/fn-once` 与 `impl-trait/return-iter` 上命中 entry_fn gate（cargo hax exit 0 + 产物文件存在但缺 entry_fn 定义），把两条原 SUCCESS 转 FAILED。

**漏报盲点（诚实声明）**：
- 上游引入新 silent path（backend item kind 新增 `-> []` 分支）—— 当前 fstar_backend.ml:1771 是已知唯一，未来上游变动需重新审 oracle
- F\* fn 渲染未来引入新 keyword（如 `unfold let` / `inline_for_extraction let`）—— 当前 hax 30949eb 不使用
- 非 entry_fn 项（type / trait / impl）的 silent-skip：当前 oracle 只 gate entry_fn 自身，伴随 type / trait 被 silent skip 仍可能 SUCCESS。这是测试设计层选择（entry_mode=bin，关注 entry_fn 自身是否被翻译），不是 oracle bug。

## 失败分桶（按 P31 §四.5 归因分类）

下表 31 条 FAILED 按 stderr 主导信号分桶，每桶单一归因。v6 final 已无 UNKNOWN（P33 [env] cap-lints 治源后 x509 两条 entry 由 UNKNOWN → SUCCESS）。

### 桶 A：`[HAX0008] reject_RawOrMutPointer`（纯触发器，3 case，工具不支持）

代表 entry：`unsafe-ptr/raw-read/raw_ptr_read`、`unsafe-ptr/raw-ptr-const/raw_ptr_const_match`、`unsafe-adv/ptr-write/unsafe_ptr_write`

stderr 特征：

```
error: [HAX0008] ...
Note: the error was labeled with context `reject_RawOrMutPointer`.
```

**归因**：工具不支持。hax-engine 显式 reject 阶段对 raw / mut pointer 拒收，是工具明文 phase 设计。
**处理**：不修。FAILED 站得住。

### 桶 B：`[HAX0003] / [HAX0006] / [HAX0010] DirectAndMut / LocalMutation / 复合`（10 case，工具不支持）

代表 entry：`closure/fn-fnmut/closure_fn`、`closure/fn-fnmut/closure_fnmut`、`aeneas-limit/fnmut-closure-unit-return`、`hax-limit/closure-mutates-outer`、`aeneas-limit/trait-impl-mut-param-mismatch`、`hax-limit/mut-in-assoc-type`、`hax-limit/mut-ref-alias`、`hax-limit/ret-mut-ref`、`miri-limit/simd-bitmask-large-vector`、`unsafe-adv/maybe-uninit`

stderr 特征：

```
error: [HAX0003] The mutation of this &mut is not allowed here.
Note: the error was labeled with context `DirectAndMut`.
```

经常 `[HAX0003]` 与 `[HAX0006] LocalMutation` 共出现（闭包对外层局部变量赋值），或与 `[HAX0010]` 共出现（`*x.borrow_mut() = ...` 类 deref-place 赋值）。

**归因**：工具不支持。mut-ref + 闭包捕获 mut 是 hax 项目明文 issue #420 拒收范围。
**处理**：不修。FAILED 站得住。

### 桶 C：`[HAX0001] FunctionalizeLoops`（3 case，工具不支持）

代表 entry：`gat/lending-iter/gat_lending`、`assoc-type/iter-style/assoc_type_iter`、`aeneas-limit/return-inside-nested-loop/outer_break_label`

stderr 特征：`context FunctionalizeLoops` — 循环 monadic lowering 未实现。

**归因**：工具不支持。**处理**：不修。

### 桶 D：`[HAX0001 / 0002 / 0008]` 复合 reject — thread-local / async（3 case，工具不支持）

代表 entry：`creusot-limit/thread-local-ref/read_thread_local`、`kani-limit/async-await/run_async_add`、`lifetime/thread-local/thread_local_read`

stderr 同时含 `reject_RawOrMutPointer` + `DirectAndMut` + `AST import` 多触发器（thread-local / async 用到的内部 mut-ref + raw ptr）。

**归因**：工具不支持。**处理**：不修。

### 桶 E：`[HAX0001 / 0002] AST import` 其他（4 case，工具不支持）

代表 entry：`hax-limit/let-chains`（issue #2018）、`charon-limit/async-fn/async_forty_two`（issue #924 coroutines）、`charon-limit/inline-asm/nop_via_asm`（`[HAX0002]` `expression Todo InlineAsm`）、`closure-adv/boxed-dyn-fn/boxed_dyn_fn`（type Dyn with non-trait predicate）

**归因**：工具不支持（AST import 阶段拒收）。**处理**：不修。

### 桶 F：`reject_ArbitraryLhs` / `reject_TraitItemDefault` / `AndMutDefsite` / `[HAX0010]`（4 case，工具不支持）

代表 entry：`refcell/borrow/refcell_borrow_mut`（`*c.borrow_mut() = 42;` 非 place expr，`[HAX0008]` + `reject_ArbitraryLhs`）、`concurrency/thread-mutex/thread_mutex_join`（`[HAX0008]` + `[HAX0010]` + `DirectAndMut` + `reject_ArbitraryLhs`）、`trait-obj/conditional-method/conditional_method`（`[HAX0008]` + `reject_TraitItemDefault`）、`hax-limit/mut-arg-pattern`（issue #1405 `[HAX0011] AndMutDefsite`）

**归因**：工具不支持（独立的显式 reject phase）。**处理**：不修。

### 桶 G：F\* backend OCaml 异常（1 case，工具不支持）

entry：`repr/union/repr_union`

stderr 特征（实测）：

```
Called from Fstar_backend.string_of_items in file "backends/fstar/fstar_backend.ml", line 1955
Called from Fstar_backend.translate_as_fstar.(fun) in file "backends/fstar/fstar_backend.ml", line 2003
error: hax: hax-engine exited with non-zero code 2
```

**归因**：工具不支持（F\* backend 自身在 union 上抛 OCaml 异常，是工具内部 panic-on-unsupported，与 `[HAX####]` 显式拒同性质）。
**处理**：不修。FAILED 站得住——本地性原则下，工具自带 OCaml 栈即工具能力边界。

### 桶 H：rustc / hax frontend 栈溢出（1 case，工具不支持）

entry：`trait/cyclic-bound/cyclic_bound_use`

stderr 特征：`thread 'rustc' has overflowed its stack` / `signal: 6, SIGABRT` —— `driver-hax-frontend-exporter` 驱动 rustc 时栈溢出。

**归因**：工具不支持。hax 自带的 `driver-hax-frontend-exporter` 是工具组件，crash 即工具锅。
**处理**：不修。FAILED 站得住。

### 桶 I：silent-skip-item（fstar_backend.ml:1771 gate 抓获，2 case，工具不支持）

entry：`closure-adv/fn-once/closure_fn_once`、`impl-trait/return-iter/impl_trait_iter`

stderr 特征：

```
info: hax: wrote file ./proofs/fstar/extraction/Closure_adv_fn_once.fst
[hax-fstar-oracle] FAIL: entry_fn 'closure_fn_once' missing from .fst products
                   (silent skip — fstar_backend.ml:1771 Use/NotImplementedYet path)
```

cargo hax exit 0，wrote `.fst` 文件，但产物不含 entry_fn 定义。

**归因**：工具不支持。silent-skip-item 路径来自工具源码 `backends/fstar/fstar_backend.ml:1771 | Use _ | NotImplementedYet -> []`，是上游工具的 silent partial 翻译路径；oracle gate 命中后转 FAILED 反映"工具未真正翻译 entry_fn"——属工具能力边界，不是我们 wrapper bug。
**处理**：不修。FAILED 站得住——若 wrapper 漏抓才会 SUCCESS（漏报），这正是 oracle 设计的封堵点。

## v5.1 → v6 final ΔS 解释

| 维度 | v5.1 (`run-1778466265-63960`) | v6 pre-P33 | v6 final (post-P33, `run-1778560393-59119`) |
|---|---|---|---|
| n | 146 | 161 | 161 |
| SUCCESS | 113 | 128 | **130** |
| FAILED | 33 | 31 | **31** |
| UNKNOWN | 0 | 2 | **0** |
| 通过率 | 77.4% | 79.5% | **80.7%** |

ΔS（v5.1 → v6 final）= +17（130 vs 113）：

- **+15 from corpus expansion**：v6 新增 15 entry，hax-fstar 在新 entry 上大多 SUCCESS
- **+2 from P33 [env] schema 治源**：x509 两条 entry（`x509_parse_der` / `x509_subject_extensions`）在 v5.1 因 vendored crate `#![deny(unused_qualifications, unstable_features)]` 触发 cargo build error；v6 pre-P33 已被 `classify_external_fault` 归 UNKNOWN（vendor_lint_strictness）；v6 final post-P33 通过 `hirusttest.toml [env] RUSTFLAGS = "--cap-lints=warn"` 在 entry 级 inject 环境变量，**hax-engine 真跑通 x509 cert-parse 真实代码**，转 SUCCESS。详 `docs/audit/v6-cc-report-rewrite-2026-05-12.md` §八
- **silent-skip-item 命中**：v6 与 v5.1 一致命中 2 条（`closure-adv/fn-once`、`impl-trait/return-iter`），oracle 行为稳定
- **FAILED 桶组成（A-I 桶）**：相对 v5.1 几乎同构，无新增工具能力 regression

整体通过率 v5.1 → v6 final +3.3 pp：corpus 扩张 +15 + P33 治源 +2 共同贡献。UNKNOWN 由 2 归 0，符合 P33 "UNKNOWN 应尽量治源不留 backlog" 的精神延伸。

## 修订建议清单（仅"我们导致"失败）

**无需修订**：

v6 final post-P33 后，全部 31 FAILED 均为工具能力边界（桶 A-I）—— hax-engine phase pipeline 显式 reject（A / B / C / D / E / F）/ F\* backend OCaml 异常（G）/ driver-hax-frontend-exporter 栈溢出（H）/ silent-skip-item 已 gate 抓回（I）。按本地性原则全部 FAILED 站得住，不修。

原 v6 pre-P33 唯一"我们导致"项（J 桶 vendored crate lint→error，2 case）已通过 P33 [env] schema 治源消除：`examples/industrial/x509-parser/cert-parse/hirusttest.toml` 加 `[env] RUSTFLAGS = "--cap-lints=warn"`，runner 在 spawn 子进程时 inject 该 env var，cargo build 不再因 vendor lint deny 失败，hax-engine 真测出 x509 cert-parse 真实代码——两条 entry 在 v6 final 均 SUCCESS，**无残留"我们导致"FAILED**。

P35 bug-detect 在 hax-fstar 上无新增 oracle 误判 / 漏报，oracle 双门（cargo hax exit + 产物 grep × 2）实测稳定。
