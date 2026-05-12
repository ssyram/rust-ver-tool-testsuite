# aeneas-lean — 特性支持评估报告（v6 baseline）

## 元数据

- **数据源**：`runs/run-1778560393-59119/`（2026-05-12 v6 final，合并 verus rerun + R7 5-tool rerun）
- **工具配置**：`tools/aeneas-lean/`
- **工具版本**：`aeneas a14083a6` + 自家 charon `0.1.184`（commit `ed22146b`，由 `charon-pin` 锁定）
- **本工具实测**：n=161 / SUCCESS=98 / FAILED=63 / UNKNOWN=0，通过率 **60.9%**
- **时长分布**：avg 3284ms / median 1267ms / p90 7730ms / max 43353ms（`timeout_secs=600`，未触发）
- **宪法 baseline**：`principles.md` v8（P27 修宪后 / P31 法律传导后）
- **时效声明**：本快照锚定上述 run id + 工具版本 + corpus，不构成长期承诺。

## pipeline + 前端边界

aeneas-lean 是 **Rust → Lean 4 的两段式纯翻译流水线**，由项目维护的 `aeneas-lean-wrapper.sh` 把两段命令打包成单一 tool 入口：

```
stage 1: charon cargo --preset=aeneas      →  <crate>.llbc
stage 2: aeneas -backend lean -dest lean-out  <crate>.llbc  →  lean-out/<Mod>.lean
```

stage 1 做完整 cargo build + 把 MIR 序列化为 LLBC；stage 2 以 LLBC 为输入做 symbolic interp + SymbolicToPure（把 mut borrow 重写成 backward-function 形态），然后由 `Extract.ml` Lean printer 落 `.lean`。**4 backend（fstar / coq / lean / hol4）共享同一 charon binary 与 aeneas OCaml engine（mid-end），差异仅在 `Extract.ml` printer 分支选择**。

**前端边界**（本测试范围）：完整跑 charon LLBC 序列化 + aeneas OCaml engine 翻译 + Lean printer 写文件落盘。**后端**（本测试不覆盖）：用户自己拿 `.lean` 给 `lean` / `lake` 做下游 type-check 与证明。

`entry_mode = "lib"`：runner 只把 lib target 喂给 charon，不渲染 bin harness（aeneas 操作整 lib，没有"per-fn"调用形式）。

错误分流到不同 stream：charon 阶段错误（rustc stack overflow / cargo build failure / charon-driver SIGABRT）落 stderr；aeneas 阶段错误以 `[Error]` 彩色行写到 **stdout**；wrapper `[aeneas-lean-wrapper] ...` 提示也在 stdout。分类需 stdout + stderr 一起读。

**wrapper 归属**：`aeneas-lean-wrapper.sh` 是项目维护，但其内部 gate 仅做"工具自陈 partial 信号"检测——把工具已经在 stderr/stdout 写出的 unsupported / warn 行翻译成 wrapper exit ≠ 0。按 `tool-integration.md` §四.5 最近责任主体原则，wrapper 自身从不报错（它没有 IO / 解析 / 网络逻辑），exit 非零必由工具层信号触发 → 失败仍归工具。

## SUCCESS 信号 + 形式严格性

按宪法 §六 双通路 partial 暴露：

- **主信号通路**（工具自身）：
  - aeneas exit 0 ⇔ `Errors.error_list` 空（`Main.ml:773` 末尾 `if has_errors then exit 1`）
  - charon exit 0（无 `--abort-on-error`，但 aeneas preset 下 charon stderr 不再 silent）
- **wrapper 补抓通路**（项目维护，已 R7 / P30 加固）：
  1. **charon silent partial gate**：charon exit 0 但 stderr 含 `is not supported` 或 `^error:` → wrapper exit 1（D3.2 / D3.3 决策；charon 在 boxed-init / inline-asm 等场景 warn 后 exit 0；charon 内部 type checker `Type error after transformations: Found incorrect clause var: Bound(N, M)` 同样 exit 0）
  2. **aeneas Warn-channel partial gate**：aeneas exit 0 但 stdout 含 4 类 Warn 自陈 → wrapper exit 1：
     - `model will not type-check` → mutually-recursive trait / impl partial
     - `generated code will likely be incorrect` → associated type partial
     - `seems to be missing the corresponding field` → Lean builtin model 缺字段
     - `could not find the information for item` → core trait method silent drop

形式严格性 0 误报 / 0 漏报状态：

- **0 误报（不冤枉能力）**：✅ 形式可证。aeneas exit 0 ⇔ `Errors.error_list` 空 ⇔ `craise` 主通路无 push ⇔ 翻译完整。叠加上述 wrapper gate 后，SUCCESS 同时满足"charon 无 silent partial / aeneas 无 Warn-channel partial"。
- **0 漏报（不高估能力）**：**实测 + wrapper 双通路封堵，非源码层证明**。aeneas 主信号通路 `craise` 已封死；charon 的 silent partial（exit 0 + stderr warn）+ aeneas 的 Warn 通道两路绕开点都已加 wrapper grep gate。但本断言仍依赖 grep 模式列表的完备性——上游若新增新的 partial 自陈措辞（例如不在 4 类 Warn / 2 类 charon 信号内的措辞），需扩 grep。
- **漏报盲点**（2026-05-12 v6 修订后）：
  - aeneas / charon 上游新增 partial 自陈措辞 → 需扩 wrapper grep pattern list
  - 依赖 aeneas 上游 `craise` 实现正确性（主信号通路）
  - charon 内部若新增不走 stderr 的 silent skip 路径，本 gate 不可见

## 失败分桶（按 P31 §四.5 归因分类）

63 个 FAILED 全部归"工具能力边界 / 工具内部 bug" — 无任何"我们 wrapper bug / corpus bug / 环境损坏"类。

### 桶 A：aeneas mid-end / backend "Generated the partial file" 路径（38 case，exit 1）

aeneas 主 oracle 通路：mid-end 或 Lean printer 遇 unsupported / internal-error / assertion，`craise` push `Errors.error_list`，`Main.ml` 末尾 `if has_errors then exit 1`，同时把已能处理部分写成 `<Mod>.lean` 部分产物。

按 stdout `[Error]` 主模板细分（38 条按主信号归桶；单 entry 可命中多模板，下表按出现顺序选第一主模板）：

| 子桶 | 数 | 代表 entry | aeneas `[Error]` 主模板 |
|---|---:|---|---|
| A1. `Improperly typed constant value` | 11 | `float/basic/float_basic` 等 9 条 `float/*` + `bigint/num-complex-ops` + `int-width/cast-float-int` | aeneas LLBC 常量类型检查阶段对 f32/f64 常量、含 f64 字段的 enum 等显式拒 |
| A2. `Improperly typed constant value` + `Invalid inputs for binop` | 3 | `aeneas-limit/float-types`、`enum/data-variants`、`kani-limit/float-overapprox` | 同 A1 + binop 类型不匹配（如 f64 wrap.-） |
| A3. `Invalid inputs for binop` / `Invalid input for unop` 单出 | 2 | `aeneas-limit/bool-bitwise-op`、`float/total-order` | bool 位运算、f64 取负越界 |
| A4. `Region ids should not be visited directly` + `Internal error` + `please file an issue` | 3 | `bigint/num-traits-abstract`、`hrtb/for-all-lifetime`、`prusti-limit/spec-entailment-unsupported` | aeneas 自标"请上报 issue" |
| A5. `Region ids should not be visited directly` 单出 | 2 | `industrial/rsa/rsa-pkcs8/{rsa_pubkey_from_pkcs8, rsa_pkcs1v15_encrypt}` | aeneas 翻译 vendored rsa crate 实战时触发 |
| A6. `Assertion failed` | 3 | `charon-limit/generic-to-dyn-unsize`、`kani-limit/stack-unwinding`、`prusti-limit/shallow-borrow-match-guard` | aeneas 内部 assertion |
| A7. `Unimplemented` | 1 | `aeneas-limit/closure-if-capture` | aeneas 自陈 todo |
| A8. `Found a case of unsupported nested borrows` | 1 | `aeneas-limit/nested-borrow-array` | aeneas 自陈不支持 |
| A9. `Nested borrows`（collections 路径） | 1 | `collections/btreemap/btreemap_basic` | 同上，BTreeMap 实例化时触发 |
| A10. `Dynamic trait types are not supported yet` | 1 | `creusot-limit/dyn-trait-forbidden/trigger_call_dyn_display` | aeneas 自陈 dyn trait 类型 todo |
| A11. `Function pointers are not supported yet` | 1 | `trait-obj/dyn-dispatch/dyn_dispatch` | aeneas 自陈 fn pointer todo |
| A12. `Arrow types are not supported yet` (+ Invalid unop) | 1 | `creusot-limit/fn-ptr-reify/get_fn_ptr` | aeneas 自陈 arrow types todo |
| A13. `Arrow types are not supported yet` + `Internal error` + `please file an issue` | 1 | `closure-adv/early-bound-lifetime/early_bound_closure_arg` | 同上 + internal |
| A14. `Breaks to outer loops` | 1 | `hax-limit/labelled-break` | aeneas 自陈 labelled break todo |
| A15. `unions are not supported` + `Internal error` + `please file an issue` | 1 | `repr/union/repr_union` | aeneas 自陈 union 不支持 |
| A16. `&raw mut` / `&raw const` Unsupported operation | 2 | `unsafe-adv/ptr-write`、`unsafe-ptr/raw-read` | aeneas 自陈 raw ptr 操作 todo |
| A17. `Invalid inputs for unsized cast` | 1 | `charon-limit/arc-slice-unsize` | aeneas 拒 array → slice unsize cast |
| A18. `Found type error in the output of charon` | 1 | `charon-limit/async-fn/async_forty_two` | aeneas 在 LLBC 上做二次类型检查时拒 |
| A19. `Charon failed to compile constant: ConstantExprKind::Cast` + `Raw ptr casts are only supported between pointers to literal types` | 1 | `unsafe-ptr/raw-ptr-const/raw_ptr_const_match` | 实质走 aeneas extract，但 charon 已 warn 类似的 partial 信号（被本桶 partial-file 路径捕获）|

**归因**：全部为 aeneas 主 oracle 通路的工具自陈"我没全干完"（`craise` push error_list）。本地性原则下 FAILED 站得住，工具开发者不能驳回。

**处理**：不修。

### 桶 B：aeneas Warn-channel partial（11 case，wrapper gate 触发）

aeneas exit 0、产物落盘，但 stdout `[Warn]` 行自陈"产物不可用"。wrapper grep gate（4 类 pattern）→ exit 1。

| entry | Warn 通道命中 pattern |
|---|---|
| `aeneas-limit/mutually-recursive-traits/trigger_mutually_recursive_traits` | `model will not type-check` + `generated code will likely be incorrect` |
| `deps-complex/bigint-serde/bigint_serde` | `model will not type-check` |
| `deps-complex/chrono-serde/chrono_serde` | `model will not type-check` |
| `deps-complex/collections-serde/collections_serde` | `model will not type-check` |
| `deps-complex/trait-serde-generic/trait_serde_generic` | `model will not type-check` |
| `deps-complex/error-chain/error_chain` | `generated code will likely be incorrect` + `seems to be missing the corresponding field` + `could not find the information for item` |
| `error/result-question/result_question` | `generated code will likely be incorrect` |
| `gat/lending-iter/gat_lending` | `generated code will likely be incorrect` |
| `impl-trait/return-iter/impl_trait_iter` | `seems to be missing the corresponding field` + `could not find the information for item` |
| `iter/chain-collect/iter_chain_collect` | `seems to be missing the corresponding field` + `could not find the information for item` |
| `kani-limit/async-await/run_async_add` | `generated code will likely be incorrect` |

**归因**：aeneas 自身的 partial 自陈（mutually-recursive trait / associated type / Lean builtin 模型缺字段 / core trait method silent drop）。wrapper 仅做信号翻译，错误源头是工具。

**处理**：不修。FAILED 站得住。本桶是 v6 cc-route audit (P30) 修宪后新引入的盲点封堵——v5.1 时这些 case 因 aeneas exit 0 而被误算 SUCCESS。

### 桶 C：charon silent partial（8 case，wrapper gate 触发）

charon exit 0，但 stderr 含 `is not supported` warn 或 `^error: Type error after transformations` 行。wrapper gate（R7 / D3.2-3.3）→ exit 1。

| entry | charon stderr 主信号 |
|---|---|
| `box/shallow-init/shallow_init_box` | `Could not reconstruct Box initialization; branching during Box initialization is not supported` |
| `charon-limit/box-branch-init/vec_with_early_return` | 同上 |
| `closure-adv/boxed-dyn-fn/boxed_dyn_fn` | 同上 |
| `charon-limit/inline-asm/nop_via_asm` | `Inline assembly is not supported` |
| `charon-limit/copy-deref-closure/deref_copy_in_closure` | `Type error after transformations: Found incorrect clause var: Bound(1, 0)` |
| `deps-complex/itertools-multi/itertools_multi` | `Type error after transformations` |
| `industrial/sha2/sha256-digest/sha256_digest_one_shot` | `Type error after transformations: Found incorrect clause var: Bound(0, 0)` |
| `industrial/sha2/sha256-digest/sha256_digest_incremental` | 同上 |

**归因**：charon 在 boxed-init / inline-asm 等场景 warn 后 exit 0（charon 设计 quirk），或 charon 内部 type checker 报错但不传播 exit code。两类都是工具层自陈。

**处理**：不修。FAILED 站得住。

### 桶 D：aeneas OCaml uncaught exception（5 case，exit 2）

aeneas 完成 SymbolicToPure 后在 `Extract.ml` 名字计算阶段抛 OCaml 未捕获异常，无完整产物。

| entry | 异常 |
|---|---|
| `concurrency/thread-mutex/thread_mutex_join` | `Failure "Can't convert type to pattern: dyn ([TraitClause@0]: core::any::Any<T@1_0> + ...)"` |
| `miri-limit/thread-interleaving-partial/unsynchronised_counter_race` | 同上 |
| `creusot-limit/thread-local-ref/read_thread_local` | `Failure "Can't convert type to pattern: !"` |
| `lifetime/thread-local/thread_local_read` | 同上 |
| `lifetime/static-bound/static_bound` | 同 `ty_to_pattern_aux` 系列 internal failure |
| `deps-complex/itertools-multi/itertools_multi` | （已计入桶 C：charon stage 先 silent partial 拦截，没进 aeneas）|

栈帧统一指向 `Charon__NameMatcher.ty_to_pattern_aux` (charon-ml) 被 `Aeneas__ExtractBase.ctx_compute_trait_impl_name_raw` 调到——aeneas 名字解析对 `dyn` trait 组合类型与 never type `!` 抛 `Failure`。

**归因**：aeneas / charon-ml 内部 bug（应 `craise` 但走了 `Failure raise`）。属"工具内部"。

**处理**：不修。FAILED 站得住。可向上游报 issue，但不在本项目修复范围。

### 桶 E：charon stack overflow（1 case，exit 101）

`trait/cyclic-bound/cyclic_bound_use`：

```
thread 'rustc' (...) has overflowed its stack
fatal runtime error: stack overflow, aborting
... (signal: 6, SIGABRT: process abort signal)
[aeneas-lean-wrapper] charon failed: exit 101
```

**归因**：charon-driver 在 cyclic trait bound 上无限递归。同 entry 在 charon-poly / charon-mono / 其他 3 个 aeneas backend 上同样 101。

**处理**：不修。

## 漏报盲点（诚实声明）

- **已通过 wrapper gate 封堵**：
  - aeneas Warn-channel partial 4 类 pattern（桶 B 覆盖）
  - charon silent partial 2 类信号（桶 C 覆盖）
- **仍存在的盲点**：
  - aeneas / charon 上游新增不在已知 pattern 列表内的 partial 自陈措辞 → 需扩 grep
  - aeneas / charon 上游若新增 silent skip 路径（既不 `craise` 也不 emit Warn / `is not supported` 字样、exit 0 静默 drop）→ 本 gate 不可见。修复 backlog：上游 release 时人工 cc-route 抽样审 + 必要时扩 grep
  - aeneas 主 oracle 通路依赖 `craise` 实现正确性，上游若把 `craise` 改为 `Log.warn` + exit 0 ⇒ 系统性漏报（本断言锚定 commit `a14083a6`）

## v5.1 → v6 ΔS 解释

v5.1：87/146 = 59.6%。v6：98/161 = 60.9%。

ΔS 来源：
- **+15 entries**（corpus 扩增 146 → 161，主要为 `runnable/*` 等新增类目）
- **+11 SUCCESS** 来自新增 entries 中 aeneas 能翻译的部分
- **桶 B aeneas Warn-channel partial（11 case）从 v5.1 SUCCESS 调整为 v6 FAILED**：v5.1 期 wrapper 无 Warn 通道 gate，这 11 case aeneas exit 0 → 误算 SUCCESS。v6 cc audit (P30) 修宪后 wrapper 加 grep gate，调整为 FAILED。此项是 ΔS 中"实质漏报封堵"，非通过率波动
- **桶 C charon silent partial 部分 case**（如 box / inline-asm 类）在 v5.1 期 wrapper 已加 charon stderr gate（D3.2-3.3），与 v6 一致；不构成 ΔS

通过率 60.9% vs 59.6% 表面上看略升，实际是"corpus 扩增带来 +新 SUCCESS"与"Warn 通道封堵带来 -11 SUCCESS"两相抵消的净值。读者引用本数字时应同时引用桶 B 的存在——本工具对 trait associated type / mutually-recursive trait / Lean builtin 模型缺字段类构造仍属"形式不可用"。

## 修订建议清单（仅"我们导致"失败）

**无需修订**。63 个 FAILED 全部为工具层（aeneas mid-end / aeneas Warn 通道 / charon silent partial / aeneas OCaml uncaught / charon stack overflow），无任何"我们 wrapper bug / 我们 corpus 引入的 lint / 环境损坏"类失败：

- wrapper 自身无 IO / 解析 / 网络逻辑，gate 仅做信号翻译——wrapper exit 非零必由工具层 stderr/stdout 信号驱动
- 桶 B / C 是 wrapper gate 触发，但归因到工具自陈（aeneas Warn / charon `is not supported`），非 wrapper bug
- 桶 D 的 OCaml uncaught 是 aeneas / charon-ml 上游 bug
- 桶 E 是 charon-driver stack overflow，charon 自身锅

| # | 桶 | 涉及 case | 修复方案 | 优先级 |
|---|---|---|---|---|
| — | — | 0 | 无 | — |

**前瞻 backlog（非 fix，是维护）**：上游 aeneas / charon release 时人工 cc-route 抽样审 partial 自陈措辞表，必要时扩 wrapper grep pattern——这是漏报封堵的持续性维护，不是本批 v6 baseline 的修订项。
