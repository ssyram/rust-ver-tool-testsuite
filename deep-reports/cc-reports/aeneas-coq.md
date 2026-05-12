# aeneas-coq — 特性支持评估报告（v6 final post-P35 baseline）

## 元数据

- **数据源**：`runs/run-1778560393-59119/`（2026-05-12 v6 final，已合并 P29 verus rerun + R7 + P33 x509 + P35 bug-detect 派生）
- **工具配置**：`tools/aeneas-coq/`
- **工具版本**：`aeneas a14083a6` + 自家 charon `0.1.184` commit `ed22146b`（按 `results.json` `tools[].version`）
- **本工具实测**：n=161 / SUCCESS=98 / FAILED=63 / UNKNOWN=0，通过率 **60.87%**
- **时长分布**：avg 3335ms / median 1498ms / p90 6505ms / max 46016ms（`timeout_secs=600`，未触发）
- **宪法 baseline**：`principles.md` v8 + P27-P35 累积派生（双根本问题 + UNKNOWN 严格语义两类 + 前端测量深度切割 + 当前 crate 焦点宽度切割 + bug detect 归 SUCCESS + 双通路 Oracle）
- **时效声明**：本快照锚定上述 run id + 工具版本 + corpus，不构成长期承诺。

## pipeline + 前端边界

aeneas-coq 是 **Rust → Coq 的两段式纯翻译流水线**，由 `aeneas-coq-wrapper.sh` 把两段命令打包成单一 tool 入口（`set -euo pipefail`，stage 1 charon 非零直接退出，不进 stage 2）：

```
stage 1: charon cargo --preset=aeneas      →  <crate>.llbc
stage 2: aeneas -backend coq -dest coq-out  <crate>.llbc  →  coq-out/<Mod>.v + Primitives.v
```

- **stage 1（charon，工具侧）**：完整 cargo build + 把 MIR 序列化为 LLBC；charon 是 aeneas 上游官方前端，由 AeneasVerif 团队维护
- **stage 2（aeneas，工具侧）**：以 LLBC 为输入做 borrow forward / backward translation（mut borrow 重写成 functional update），然后由 `Extract.ml` 的 Coq printer 分支落 `.v` 文件
- **我方 wrapper（`aeneas-coq-wrapper.sh`，项目侧）**：把两段命令串起来；额外加 R7 引入的两道 partial 自陈 grep gate（见下"SUCCESS 信号"段）

4 个 aeneas backend（fstar / coq / lean / hol4）共享同一 charon binary 与 aeneas OCaml engine（mid-end），差异仅发生在最后的 `Extract.ml` printer 分支选择。

**前端测量（深度切割，§六）**：完整跑 charon LLBC 序列化 + aeneas OCaml engine 翻译 + Coq printer 写 `.v`。**后端**（本测试不覆盖）：用户自己拿 `.v` 给 `coqc` 做下游 type-check + 手工补充 Coq proof——本测试集筛选前端特性覆盖广度，下游 prover type-check 不在本范围。

**当前 crate 焦点（宽度切割，§六）**：测量对象是 entry crate（runner 注入的 `TS_TARGET_CRATE` / `TS_ENTRY_FN`）。外部依赖（std / core / 第三方 cargo registry crate / 我方 vendored 但被 entry 当外部依赖引入的 crate）的 opaque / skip / partial 不算 partial。该宽度切割原则在当前 wrapper grep gate 中**尚未实施**（grep 不区分 partial site 是 entry 还是 external）—— 见下"修订建议"段。

`Primitives.v` 约 100 条 `Axiom` 是运行时库抽象（与翻译质量无关）。

`entry_mode = "lib"`：runner 只把 lib target 喂给 charon，不渲染 bin harness。

错误分流：charon 阶段错误（rustc stack overflow / cargo build / charon-driver SIGABRT）落 **stderr**；aeneas 阶段错误以彩色 `[Error] <msg>` 行落到 **stdout**；wrapper 自身 `[aeneas-coq-wrapper] ...` 提示也在 stdout。下游分类需 stdout + stderr 一起读。

## SUCCESS 信号 + 形式严格性

**主信号通路**：wrapper 最终 exit code = stage 2 aeneas 的 exit code（stage 1 失败时直接 propagate）。

- **exit 0** = aeneas 全程跑通 ⇔ `Errors.error_list` 空 ⇔ 翻译完整，产物 `coq-out/<Mod>.v` 写出 → SUCCESS
- **exit 1** = aeneas 检出 `craise` 类 unsupported（`Main.ml:773` `if has_errors then exit 1`）→ FAILED
- **exit 2 / exit 101** = OCaml 未捕获异常或 charon-driver SIGABRT → FAILED

**wrapper 补抓通路**（R7 2026-05-12 引入的两道 partial 自陈 grep gate）：

1. **charon stage silent partial gate**：charon 可能 exit 0 但 stderr 含 `is not supported` 或 `^error:`（charon 把 unknown 构造 opaque 化 + 把 type error 不传播）。wrapper grep 命中即 FAILED。
2. **aeneas stage Warn-channel partial gate**：aeneas exit 0 但 stdout/stderr 含四类 Warn 自陈——`model will not type-check` / `generated code will likely be incorrect` / `seems to be missing the corresponding field` / `could not find the information for item`——wrapper grep 命中即 FAILED。

**形式严格性 — 0 误报（不冤枉能力）**

主通路：aeneas 用 `craise` 单一信号通路把所有 unsupported 推入 `Errors.error_list`；`Main.ml` 末尾 `if has_errors then exit 1`——exit 0 ⇔ error_list 空。这是 `tool-integration.md` §四.1 列举的 aeneas 单一通路形式可证案例。

wrapper 补抓通路：四类 Warn 自陈 + charon "is not supported" / "^error:" 是 aeneas / charon 上游明示"模型会有问题"的字面文本，从字面方向（grep marker → 工具自陈）实测无误报（合法成功 entry 不含这些字面）。**但宽度向的 0 误报有缺陷**：grep 命中位置在 entry crate 还是 external dep 未区分——按 §六 当前 crate 焦点，external dep 上的 partial 不该被算成 entry 的 FAILED。见下"修订建议"。

**形式严格性 — 0 漏报（不高估能力）**

按 `tool-integration.md` §四.1 与 §四.3：

- 主通路 craise → exit ≠ 0 是 aeneas 内部唯一 unsupported 入口，实测 + 源码层论证 0 漏报
- Warn 通道 + charon stage silent 这两条非 craise 通路，由 wrapper grep 补抓——**这是实测有效性，不构成形式 0 漏报**（按 §四.3：grep marker 只覆盖已知 silent path）

## 失败分桶（按 P31 §四.5 + P35 §六 当前 crate 焦点归因分类）

63 个 FAILED 按"主信号位置 + 归因 + 当前 crate 焦点"分 7 桶。**52 条"工具不支持 / 工具自身 bug"** + **11 条"我们 wrapper 宽度切割未实施"导致的 over-flag**。

### 桶 1：charon stage silent partial — 真 entry 命中（6 case）

代表 entry：`box/shallow-init/shallow_init_box`、`charon-limit/box-branch-init/vec_with_early_return`、`charon-limit/copy-deref-closure/deref_copy_in_closure`、`charon-limit/inline-asm/nop_via_asm`、`closure-adv/boxed-dyn-fn/boxed_dyn_fn`、`deps-complex/itertools-multi/itertools_multi`

stderr 特征（partial 位置 → `src/__ts_inner.rs`）：

```
warning: Could not reconstruct `Box` initialization; branching during `Box` initialization is not supported.
 --> src/__ts_inner.rs:5:5
[aeneas-coq-oracle] FAIL: charon exited 0 but emitted partial-signal stderr ('is not supported' or '^error:')
```

`deps-complex/itertools-multi` 的 charon error 同时命中 entry `src/__ts_inner.rs:15` 与外部 `itertools-0.13.0` + `core::iter`——按"entry crate 有 partial 即算 partial"判定 FAILED 合理。

**归因**：charon 上游自陈"is not supported" / "Type error after transformations"——把 Box-branch-init / inline asm / boxed dyn fn / 跨外部 trait 的 closure-deref 这类构造 opaque 化后 silent 退出 0。按"本地性 + 不允许 partial"由 wrapper R7 gate 升 FAILED。
**处理**：不修。工具自陈的能力边界，FAILED 站得住。

### 桶 2：charon stage silent partial — 纯外部依赖命中（2 case，**我们导致**）

代表 entry：`industrial/sha2/sha256-digest/sha256_digest_one_shot`、`industrial/sha2/sha256-digest/sha256_digest_incremental`

stderr 特征（partial 位置**全部**在 `/cargo/registry/.../hybrid-array-0.4.12/src/`，与 entry crate `sha256_digest` 无关）：

```
warning: Could not compute the value of Self::Size ... hybrid_array::traits::AssocArraySize<Self::ArrayType>
 --> /cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hybrid-array-0.4.12/src/traits.rs:22:1
error: Type error after transformations:
 --> /cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hybrid-array-0.4.12/src/lib.rs:165:1
```

**归因**：charon 在外部依赖 `hybrid-array` 的 `AssocArraySize` trait 抽象上失败；entry crate `sha256_digest` 自身的 fn / type / trait 没有出问题。按 P35 §六 当前 crate 焦点（"外部依赖 opaque / skip / partial 不算 partial"），这两条不应被 wrapper 算 FAILED。但当前 wrapper grep 不区分 partial site → over-flag。
**处理**：**修**。我们 wrapper 缺 §六 width-cut filter。属"我们这边可识别问题暂未修" → 应升 UNKNOWN (b) 类，附会修计划（grep gate 增加 source location filter）。见下"修订建议清单 R1"。

### 桶 3：aeneas Warn-channel partial — 真 entry 命中（1 case）

代表 entry：`aeneas-limit/mutually-recursive-traits/trigger_mutually_recursive_traits`

stdout 特征（mutually recursive trait 在 entry `src/__ts_inner.rs`）：

```
[Warn ] Mutually recursive trait declarations are not supported; the following group of mutually recursive
       traits is going to be extracted but their model will not type-check:
'mutually_recursive_traits::__ts_inner::Trait1', source: 'src/__ts_inner.rs', lines 18:0-20:1
'mutually_recursive_traits::__ts_inner::Trait2', source: 'src/__ts_inner.rs', lines 22:0-22:27
```

**归因**：aeneas 上游自陈 entry crate 自定义 `Trait1 / Trait2 / T1 / T2` 的 mutually recursive 翻译"model will not type-check"——属 `tool-integration.md` §四.4 列举的"上游 Warn 通道 partial 自陈"路径，wrapper grep 拦截。
**处理**：不修。aeneas 上游自陈的能力边界。

### 桶 4：aeneas Warn-channel partial — 纯外部依赖命中（9 case，**我们导致**）

代表 entry：`deps-complex/{bigint-serde, chrono-serde, collections-serde, error-chain, trait-serde-generic}`、`error/result-question/result_question`、`impl-trait/return-iter/impl_trait_iter`、`iter/chain-collect/iter_chain_collect`、`kani-limit/async-await/run_async_add`

stdout 特征（Warn 列出的 trait 全部在 external dep）：

```
[Warn ] Mutually recursive trait declarations are not supported ... model will not type-check:
'serde_core::ser::Serialize', source: '/cargo/registry/.../serde_core-1.0.228/src/ser/mod.rs'
'core::iter::traits::iterator::Iterator', source: '/rustc/library/core/src/iter/traits/iterator.rs'
'core::iter::traits::collect::FromIterator', source: '/rustc/library/core/src/iter/traits/collect.rs'
'core::ptr::metadata::Pointee', source: '/rustc/library/core/src/ptr/metadata.rs'
'core::str::pattern::Pattern', source: '/rustc/library/core/src/str/pattern.rs'
```

entry crate 自身的 fn / type 没有触发 Warn，partial 完全发生在 serde_core / core::iter / core::ptr / core::str 等外部 trait 的翻译。

**归因**：aeneas 在外部依赖的 mutually recursive trait / associated type 上翻译"model will not type-check"。按 P35 §六 当前 crate 焦点，外部依赖 partial 不算 entry partial——wrapper grep 不区分 partial site → over-flag。
**处理**：**修**。同桶 2 → 应升 UNKNOWN (b) 类。见"修订建议清单 R1"。

### 桶 5：aeneas 显式 craise（exit 1 + `[Error]` 主通路，29 case）

代表 entry 与典型 `[Error]` 信号（partial site 在 entry `src/__ts_inner.rs`）：

- `[Error] Improperly typed constant value`（14 条）：整 `float/*` 10 条、`int-width/cast-float-int`、`bigint/num-complex-ops`、`enum/data-variants`、`kani-limit/float-overapprox`、`aeneas-limit/float-types`、`float/total-order`（同时含 `Invalid input for unop: wrap.- on f64`）
- `[Error] Nested borrows are not supported yet` / `[Error] Found a case of unsupported nested borrows`（2 条）：`aeneas-limit/nested-borrow-array`、`collections/btreemap`
- `[Error] Arrow types are not supported yet`（2 条）：`closure-adv/early-bound-lifetime`、`creusot-limit/fn-ptr-reify`
- `[Error] Function pointers are not supported yet`（1 条）：`trait-obj/dyn-dispatch`
- `[Error] Dynamic trait types are not supported yet`（1 条）：`creusot-limit/dyn-trait-forbidden`
- `[Error] Invalid inputs for binop` / `Invalid input for unop`（2 条）：`aeneas-limit/bool-bitwise-op`（bool 位运算）+ 上述 float/total-order
- `[Error] Can not extract trait associated types with parameters`（1 条）：`gat/lending-iter`（entry 自定义 `LendIter` 触发，同时也命中 Warn 通道但 craise 才是根因）
- `[Error] Unsupported operation: &raw mut/const` / `Charon failed to compile constant: ConstantExprKind::Cast` / `Invalid inputs for unsized cast` / `Breaks to outer loops are not supported yet` / `unions are not supported` / `Detected groups of mixed mutually recursive definitions` / `Unimplemented`（约 8 条）：`unsafe-ptr/raw-{read, ptr-const}`、`unsafe-adv/ptr-write`、`charon-limit/arc-slice-unsize`、`hax-limit/labelled-break`、`repr/union`、`aeneas-limit/closure-if-capture`、`charon-limit/async-fn`

**归因**：aeneas extract 阶段 `craise` 主通路抛出的显式 unsupported 信号——`Errors.error_list` 非空 → `Main.ml:773` exit 1。是 aeneas 自身明示"我没全干完"的标准信号。
**处理**：不修。aeneas 自陈的能力边界。

### 桶 6：aeneas 自标 internal error（exit 1 + `[Error]` 内部 bug 类，9 case）

代表 entry 与 `[Error]` 信号：

- `[Error] Region ids should not be visited directly`（3 条）：`industrial/rsa/rsa-pkcs8/{rsa_pkcs1v15_encrypt, rsa_pubkey_from_pkcs8}`、`iter/chain-collect`（注：iter/chain-collect 在桶 4 已归 over-flag — 这里指其他场景；实际 v6 数据 iter/chain-collect 走 Warn-channel gate 触发，不进 B6）
- `[Error] Assertion failed: new value doesn't have the same type as its destination`（3 条）：`charon-limit/generic-to-dyn-unsize/boxed_display_from_u32`、`kani-limit/stack-unwinding/trigger_divide_with_recovery`、`prusti-limit/shallow-borrow-match-guard/trigger_shallow_borrow_match_guard`
- `[Error] Internal error, please file an issue`（3 条）：`bigint/num-traits-abstract`、`hrtb/for-all-lifetime`、`prusti-limit/spec-entailment-unsupported`
- 还含 `closure-adv/early-bound-lifetime`（assertion 类）、`repr/union`（Unimplemented 类）— 部分归类边界与桶 5 接壤

**归因**：aeneas 自标"please file an issue"——内部不变量违反 / interpreter assertion 触发，属 aeneas 工具自身 bug。错误位置主要在 entry crate（rsa 两条在 entry 的 `rsa-pkcs8/__ts_inner.rs` 上下游），属"工具吃不下 entry 代码"的工具 bug。
**处理**：不修。属 aeneas 上游 bug，等其修复。FAILED 站得住。

### 桶 7：aeneas OCaml uncaught exception（exit 2，5 case）

代表 entry：`concurrency/thread-mutex/thread_mutex_join`、`creusot-limit/thread-local-ref/read_thread_local`、`lifetime/{static-bound, thread-local}`、`miri-limit/thread-interleaving-partial/unsynchronised_counter_race`

stderr 栈帧：

```
(Failure
Raised at Charon__NameMatcher.ty_to_pattern_aux in file "charon/charon-ml/src/NameMatcher.ml", lines 1064-1066
Called from Charon__NameMatcher.impl_elem_to_pattern (NameMatcher.ml:986)
```

或 `[Error] We don't support arrow types with locally quantified regions` 后续触发 OCaml uncaught。

**归因**：aeneas 在处理 `dyn Any + Send` 模式匹配 / arrow type with quantified region / static-bound / `thread_local!` macro 时触发未捕获的 OCaml `Failure` 异常（`Charon__NameMatcher.ty_to_pattern_aux` 未覆盖的 pattern branch）。属 aeneas + 自家 charon-ml 上游 bug，**与 partial-site 是否外部依赖无关**——工具未在所有输入上完成进程。
**处理**：不修。属工具自身实现 bug，FAILED 站得住。

### 桶 8：charon stage rustc stack overflow（exit 101，1 case）

唯一 entry：`trait/cyclic-bound/cyclic_bound_use`

stderr：

```
thread 'rustc' (23603013) has overflowed its stack
fatal runtime error: stack overflow, aborting
process didn't exit successfully: `charon-driver rustc ...` (signal: 6, SIGABRT)
[aeneas-coq-wrapper] charon failed: exit 101
```

**归因**：charon-driver 在 cyclic trait bound 上递归调用 rustc，触发 rustc stack overflow + SIGABRT。属 charon × rustc 边界的 bug（同 entry 在 aeneas-fstar / aeneas-lean / aeneas-hol4 上均 exit 101）。
**处理**：不修。属 charon 上游 + rustc 交互 bug，FAILED 站得住。

## P35 bug-detect 派生是否适用

不适用。架构 §一 P35 bug-detect 规则适用于"完整跑前端 + 求解，自陈在 entry 代码里找到 bug / UB / counterexample"的工具（MIRI / soteria / verifast）。aeneas-coq 是纯翻译器，不做语义验证，**不存在 bug detect 类输出**。故 P35 不翻转任何 aeneas-coq entry。

## 漏报盲点（诚实声明）

- **已通过 wrapper gate 封堵**：
  - aeneas Warn 通道 4 类 partial 自陈：`model will not type-check` / `generated code will likely be incorrect` / `seems to be missing the corresponding field` / `could not find the information for item`
  - charon stage 2 类 silent partial：`is not supported`（warning 字面）/ `^error:`（type error after transformations）
- **当前 gate 已知缺陷**：
  - **§六 当前 crate 焦点未实施**：grep gate 不区分 partial site 是 entry crate 还是 external dep → 桶 2 / 桶 4 共 11 条 over-flag。修订计划见 R1。
- **仍存在的盲点**（修 R1 后仍有）：
  - aeneas / charon 上游若引入新的 silent partial 字面（不在当前 grep marker 集合内）→ 当前 gate 无法覆盖；fix backlog：每次工具升级后跑 cc-route audit 重新校准 marker 列表
  - 完全 skip item 类盲点（aeneas 是否在某些路径上完全 drop entry item 而不发任何 Warn 与 Error）→ 需源码层穷尽验证，当前 README 未做此论证
  - charon stage 上 `silent partial gate` 用的两个 marker（`is not supported` / `^error:`）在 v6 wrapper 实测无误报，但未做反向证明；新加 grep marker 必须按 `tool-integration.md` §四.2 双向实测才能上线

## v5.1 → v6 ΔS 解释

v5.1（`run-1778504159-67797`）：SUCCESS=102 / FAILED=59
v6（`run-1778560393-59119`）：SUCCESS=98 / FAILED=63
**ΔS = -4**

ΔS 来源：v6 wrapper R7 加入两道 partial 自陈 grep gate（charon stage `is not supported` / `^error:` + aeneas Warn 通道 4 类），把先前 silent partial 暴露。其中：

| Entry | v5.1 | v6 | 原因 |
|---|---|---|---|
| `aeneas-limit/mutually-recursive-traits/trigger_mutually_recursive_traits` | SUCCESS | FAILED | aeneas Warn 通道 entry crate 内 mutually recursive trait（**legit FAILED**）|
| `charon-limit/copy-deref-closure/deref_copy_in_closure` | SUCCESS | FAILED | charon stage `error: Type error after transformations` entry 命中（**legit FAILED**）|
| `charon-limit/inline-asm/nop_via_asm` | SUCCESS | FAILED | charon stage `warning: Inline assembly is not supported` entry 命中（**legit FAILED**）|
| `impl-trait/return-iter/impl_trait_iter` | SUCCESS | FAILED | aeneas Warn 通道 partial 但站点在外部 `core::iter`（**over-flag，应修**）|

ΔS = -4 中：3 条是 oracle 严格化的诚实暴露，1 条是 §六 宽度切割未实施导致的误升 FAILED。R1 修订后 1 条会回退到 SUCCESS。

## 修订建议清单（仅"我们导致"失败）

**R1：wrapper grep gate 加 entry-crate 焦点 filter（§六 width-cut 实施）**

| 信息 | 值 |
|---|---|
| 覆盖 case 数 | **11**（桶 2 共 2 + 桶 4 共 9）|
| 覆盖 entry | `industrial/sha2/sha256-digest/{sha256_digest_one_shot, sha256_digest_incremental}`、`deps-complex/{bigint-serde, chrono-serde, collections-serde, error-chain, trait-serde-generic}`、`error/result-question`、`impl-trait/return-iter`、`iter/chain-collect`、`kani-limit/async-await` |
| 归因 | 我方 wrapper 未实施 §六 当前 crate 焦点：grep gate 不区分 partial site 在 entry 还是 external dep |
| 当前误判走向 | 应 SUCCESS（仅 external dep partial）→ 实测 FAILED |
| 修复方案 | 修改 `aeneas-coq-wrapper.sh` 两道 gate，对每个 marker 命中行用其上下文（`source: '...'` 或 `--> ...:` 的紧邻 ±5 行）检查路径前缀：仅当至少一处 partial source 命中 entry crate（`src/__ts_inner.rs` / `src/lib.rs` 或 `$TS_TARGET_CRATE` 名空间内的 source）时升 FAILED；纯外部依赖 partial 放过。具体形式参考 `tool-integration.md` §四.2 双向实测要求（合法 SUCCESS 不命中 + 真 entry partial 命中） |
| UNKNOWN 桥接 | 在修好之前，11 条按 P27 修宪后 §六 UNKNOWN (b) 类"我们这边可识别问题暂未修"应理解为 **status=FAILED-but-not-工具的锅**；R1 落地后会回 SUCCESS |
| 优先级 | 高（影响 aeneas-coq 通过率约 +6.83pp：98/161 → 109/161 ≈ 67.70%；同样影响 aeneas-fstar/lean/hol4，需同步修） |

无其他"我们导致"失败。其余 52 条 FAILED（桶 1 + 桶 3 + 桶 5 + 桶 6 + 桶 7 + 桶 8）均为 aeneas / charon 上游能力边界或上游 bug，按本地性原则 FAILED 站得住。
