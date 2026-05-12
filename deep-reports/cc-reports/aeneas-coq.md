# aeneas-coq — 特性支持评估报告（v6 baseline）

## 元数据

- **数据源**：`runs/run-1778560393-59119/`（2026-05-12 v6 final，合并 verus rerun + R7 5-tool rerun）
- **工具配置**：`tools/aeneas-coq/`
- **工具版本**：`aeneas a14083a6`（按 `results.json` `tools[].version`）+ 自家 charon `0.1.184` commit `ed22146b`
- **本工具实测**：n=161 / SUCCESS=98 / FAILED=63 / UNKNOWN=0，通过率 **60.87%**
- **时长分布**：avg 3335ms / median 1498ms / p90 6505ms / max 46016ms（`timeout_secs=600`，未触发）
- **宪法 baseline**：`principles.md` v8（P27 修宪后 / P31 法律传导后）
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

**前端边界**（本测试范围）：完整跑 charon LLBC 序列化 + aeneas OCaml engine 翻译 + Coq printer 写 `.v`。**后端**（本测试不覆盖）：用户自己拿 `.v` 给 `coqc` 做下游 type-check + 手工补充 Coq proof——本测试集筛选前端特性覆盖广度，下游 prover type-check 不在本范围。

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

主通路：aeneas 用 `craise` 单一信号通路把所有 unsupported 推入 `Errors.error_list`；`Main.ml` 末尾 `if has_errors then exit 1`——exit 0 ⇔ error_list 空。这是 [`tool-integration.md`](../../docs/design/tool-integration.md) §四.1 列举的 aeneas 单一通路形式可证案例。

wrapper 补抓通路：四类 Warn 自陈 + charon "is not supported" / "^error:" 是 aeneas / charon 上游明示的"模型会有问题"字面文本（不是工程启发式或代码字面字串），双向实测无误报（合法成功 entry 不含这些字面）。

**形式严格性 — 0 漏报（不高估能力）**

按 `tool-integration.md` §四.1 与 §四.3：

- 主通路 craise → exit ≠ 0 是 aeneas 内部唯一 unsupported 入口，形式可证 0 漏报
- Warn 通道 + charon stage silent 这两条非 craise 通路，由 wrapper grep 补抓——**这是实测有效性，不构成形式 0 漏报**（按 §四.3：grep marker 只覆盖已知 silent path）

**漏报盲点**（诚实声明，按 §四.4）：

- 已 wrapper gate 封堵：aeneas Warn 通道四类（mutually-recursive trait / associated type / builtin model 缺字段 / core trait method silent drop）+ charon 两类（"is not supported" 字面 + "^error:" type-error）
- 仍存在的盲点：
  - aeneas 上游若新增 Warn 通道 partial 自陈字面 → 需扩展 wrapper grep pattern list（上游 commit 滞后）
  - charon 若引入新的 silent fallback 路径不带已知 markers → 当前 gate 无法覆盖
  - 完全 skip item 类（item 既不写 sorry 也不发 Diagnostic / Warn，不出现在产物里）—— aeneas/charon 是否存在此类路径需源码层穷尽验证，当前未做

## 失败分桶（按 P31 §四.5 归因分类）

63 个 FAILED 按"主信号位置 + 归因"分 7 桶。**全部 63 条归属"工具不支持 / 工具自身 bug"——0 条归属"我们导致"**。详细分桶如下：

### 桶 1：charon stage silent partial（wrapper gate 拦截，4 case）

代表 entry：`box/shallow-init/shallow_init_box`、`charon-limit/box-branch-init/vec_with_early_return`、`charon-limit/inline-asm/nop_via_asm`、`closure-adv/boxed-dyn-fn/boxed_dyn_fn`

stderr 特征：

```
warning: Could not reconstruct `Box` initialization; branching during `Box` initialization is not supported.
warning: Inline assembly is not supported
[aeneas-coq-oracle] FAIL: charon exited 0 but emitted partial-signal stderr ('is not supported' or '^error:'); silent degradation prevented
```

**归因**：charon 上游自陈"is not supported"——把 Box-branch-init / inline asm / boxed dyn fn 这类构造 opaque 化后 silent 退出 0。按"本地性 + 不允许 partial"由 wrapper R7 gate 升 FAILED。
**处理**：不修。工具自陈的能力边界，FAILED 站得住。

### 桶 2：charon stage type error silent（wrapper gate 拦截，4 case）

代表 entry：`charon-limit/copy-deref-closure/deref_copy_in_closure`、`deps-complex/itertools-multi/itertools_multi`、`industrial/sha2/sha256-digest/{sha256_digest_one_shot, sha256_digest_incremental}`

stderr 特征：

```
error: Type error after transformations:
       Mismatched type generics:
       target: IntoIterator
       expected: [Self, Self_Item, Self_IntoIter]
            got: [Self::ArrayType]
[aeneas-coq-oracle] FAIL: charon exited 0 but emitted partial-signal stderr ('is not supported' or '^error:'); silent degradation prevented
```

**归因**：charon 内部"transformations 后 type error"但 exit 仍 0（charon 上游未把内部 type error 传播为非零 exit）。涉及 hybrid-array crate 的 trait 关联类型 binding 元信息缺失 / closure-deref 的 region 元信息错配。由 wrapper R7 gate 升 FAILED。
**处理**：不修。属 charon 上游 bug / 能力边界；wrapper 正确暴露，FAILED 站得住。

### 桶 3：aeneas Warn-channel partial（wrapper gate 拦截，2 case）

代表 entry：`aeneas-limit/mutually-recursive-traits/trigger_mutually_recursive_traits`、`impl-trait/return-iter/impl_trait_iter`

stdout 特征：

```
[Warn ] Mutually recursive trait declarations are not supported; the following group of mutually recursive
       traits is going to be extracted but their model will not type-check:
[Warn ] Found an associated type in a trait declaration; trait associated types are usually lifted to
       become parameters of the trait definition, but this can fail with mutually-recursive traits as
       well as GATs. Aeneas cannot handle such types today, and the generated code will likely be incorrect.
```

**归因**：aeneas 上游自陈"model will not type-check" / "generated code will likely be incorrect"——属 `tool-integration.md` §四.4 列举的"上游 Warn 通道 partial 自陈"路径，wrapper grep 4-pattern OR 拦截。
**处理**：不修。aeneas 上游自陈的能力边界（mutually-recursive trait / associated type 翻译不完整）。

### 桶 4：aeneas 显式 craise（exit 1 + `[Error]` 主通路，38 case）

代表 entry 与典型 `[Error]` 信号：

- `[Error] Improperly typed constant value`（14 条）：整 `float/*` 10 条、`int-width/cast-float-int`、`bigint/num-complex-ops`、`enum/data-variants`、`kani-limit/float-overapprox`、`aeneas-limit/float-types`、`float/total-order`（同时含 `Invalid input for unop: wrap.- on f64`）
- `[Error] Nested borrows are not supported yet` / `[Error] Found a case of unsupported nested borrows`（2 条）：`aeneas-limit/nested-borrow-array`、`collections/btreemap`
- `[Error] Arrow types are not supported yet`（2 条）：`closure-adv/early-bound-lifetime`、`creusot-limit/fn-ptr-reify`
- `[Error] Function pointers are not supported yet`（1 条）：`trait-obj/dyn-dispatch`
- `[Error] Dynamic trait types are not supported yet`（1 条）：`creusot-limit/dyn-trait-forbidden`
- `[Error] Invalid inputs for binop` / `Invalid input for unop`（2 条）：`aeneas-limit/bool-bitwise-op`（bool 位运算）+ 上述 float/total-order
- `[Error] Unsupported operation: &raw mut/const` / `Charon failed to compile constant: ConstantExprKind::Cast` / `Invalid inputs for unsized cast` / `Breaks to outer loops are not supported yet` / `unions are not supported` / `Can not extract trait associated types with parameters` / `Detected groups of mixed mutually recursive definitions` / `Unimplemented`：约 10 条覆盖 `unsafe-ptr/raw-{read, ptr-const}`、`unsafe-adv/ptr-write`、`charon-limit/arc-slice-unsize`、`hax-limit/labelled-break`、`repr/union`、`error/result-question`、`gat/lending-iter`、`kani-limit/async-await`、`aeneas-limit/closure-if-capture`、`charon-limit/async-fn`
- `[Error] Inconsistent projection:` / `The input arguments don't have the proper type`（4 条）：`deps-complex/{bigint-serde, chrono-serde, collections-serde, trait-serde-generic}`——serde 派生宏展开后的 LLBC 在 aeneas projection / argument type 检查处拒
- `[Error] Unhandled BuiltinOrAuto for trait N`（1 条）：`deps-complex/error-chain`

**归因**：aeneas extract 阶段 `craise` 主通路抛出的显式 unsupported 信号——`Errors.error_list` 非空 → `Main.ml:773` exit 1。这是 aeneas 自身明示"我没全干完"的标准信号，按宪法 §六"不允许 partial：工具自陈必须被尊重" → FAILED。
**处理**：不修。每条都是 aeneas 自陈的能力边界。

### 桶 5：aeneas 自标 internal error（exit 1 + `[Error]` 内部 bug 类，9 case）

代表 entry 与 `[Error]` 信号：

- `[Error] Region ids should not be visited directly`（3 条）：`industrial/rsa/rsa-pkcs8/{rsa_pkcs1v15_encrypt, rsa_pubkey_from_pkcs8}`、`iter/chain-collect/iter_chain_collect`
- `[Error] Assertion failed: new value doesn't have the same type as its destination`（3 条）：`charon-limit/generic-to-dyn-unsize/boxed_display_from_u32`、`kani-limit/stack-unwinding/trigger_divide_with_recovery`、`prusti-limit/shallow-borrow-match-guard/trigger_shallow_borrow_match_guard`
- `[Error] Internal error, please file an issue`（3 条）：`bigint/num-traits-abstract`、`hrtb/for-all-lifetime`、`prusti-limit/spec-entailment-unsupported`

**归因**：aeneas 自标"please file an issue"——内部不变量违反 / interpreter assertion 触发，属 aeneas 工具自身 bug（不是显式 unsupported），但仍走 craise → exit 1。工具明示是 internal error，FAILED 站得住。
**处理**：不修。属 aeneas 上游 bug，等其修复。

### 桶 6：aeneas OCaml uncaught exception（exit 2，5 case）

代表 entry：`concurrency/thread-mutex/thread_mutex_join`、`creusot-limit/thread-local-ref/read_thread_local`、`lifetime/{static-bound, thread-local}`、`miri-limit/thread-interleaving-partial/unsynchronised_counter_race`

stdout 末段 + stderr 栈帧：

```
[Error] Unreachable | Assertion failed: new value doesn't have the same type as its destination
  (Failure
Raised at Charon__NameMatcher.ty_to_pattern_aux in file "charon/charon-ml/src/NameMatcher.ml", lines 1064-1066
Called from Charon__NameMatcher.impl_elem_to_pattern (NameMatcher.ml:986)
```

或 `[Error] We don't support arrow types with locally quantified regions` 后续触发 OCaml uncaught。

**归因**：aeneas 在处理 `dyn Any + Send` 模式匹配 / arrow type with quantified region / static-bound 时触发未捕获的 OCaml `Failure` 异常（`Charon__NameMatcher.ty_to_pattern_aux` 未覆盖的 pattern branch）。属 aeneas + 自家 charon-ml 上游 bug。
**处理**：不修。aeneas OCaml main 未捕获 exception → exit 2，属工具自身实现 bug，FAILED 站得住。

### 桶 7：charon stage rustc stack overflow（exit 101，1 case）

唯一 entry：`trait/cyclic-bound/cyclic_bound_use`

stderr：

```
thread 'rustc' (23603013) has overflowed its stack
fatal runtime error: stack overflow, aborting
process didn't exit successfully: `charon-driver rustc --crate-name trait_cyclic_bound ...` (signal: 6, SIGABRT)
[aeneas-coq-wrapper] charon failed: exit 101
```

**归因**：charon-driver 在 cyclic trait bound 上递归调用 rustc，触发 rustc stack overflow + SIGABRT。属 charon × rustc 边界的 bug（同 entry 在 aeneas-fstar / aeneas-lean / aeneas-hol4 上均 exit 101）。
**处理**：不修。属 charon 上游 + rustc 交互 bug，FAILED 站得住。

## 漏报盲点（诚实声明）

- **已通过 wrapper gate 封堵**：
  - aeneas Warn 通道 4 类 partial 自陈：`model will not type-check` / `generated code will likely be incorrect` / `seems to be missing the corresponding field` / `could not find the information for item`
  - charon stage 2 类 silent partial：`is not supported`（warning 字面）/ `^error:`（type error after transformations）
- **仍存在的盲点**：
  - aeneas / charon 上游若引入新的 silent partial 字面（不在当前 grep marker 集合内）→ 当前 gate 无法覆盖；fix backlog：每次工具升级后跑 cc-route audit 重新校准 marker 列表
  - 完全 skip item 类盲点（aeneas 是否在某些路径上完全 drop item 而不发任何 Warn 与 Error）→ 需源码层穷尽验证，当前 README 未做此论证
  - charon stage 上 `silent partial gate` 用的两个 marker（`is not supported` / `^error:`）在 v6 wrapper 实测无误报，但未做反向证明；新加 grep marker 必须按 [`tool-integration.md`](../../docs/design/tool-integration.md) §四.2 双向实测才能上线

## v5.1 → v6 ΔS 解释

v5.1（`run-1778504159-67797`）：SUCCESS=102 / FAILED=59
v6（`run-1778560393-59119`）：SUCCESS=98 / FAILED=63
**ΔS = -4**

4 条由 SUCCESS → FAILED 的来源：v6 wrapper R7 加入两道 partial 自陈 grep gate（charon stage `is not supported` / `^error:` + aeneas Warn 通道 4 类），如实把先前 silent partial 暴露为 FAILED：

| Entry | v5.1 | v6 | 原因 |
|---|---|---|---|
| `aeneas-limit/mutually-recursive-traits/trigger_mutually_recursive_traits` | SUCCESS | FAILED | aeneas Warn 通道 `Mutually recursive trait declarations are not supported ... model will not type-check` |
| `charon-limit/copy-deref-closure/deref_copy_in_closure` | SUCCESS | FAILED | charon stage `error: Type error after transformations` exit 0 silent |
| `charon-limit/inline-asm/nop_via_asm` | SUCCESS | FAILED | charon stage `warning: Inline assembly is not supported` exit 0 silent |
| `impl-trait/return-iter/impl_trait_iter` | SUCCESS | FAILED | aeneas Warn 通道 mutually recursive trait + `generated code will likely be incorrect` |

ΔS 是 oracle 严格化的结果（v5.1 的 4 条 SUCCESS 都是 silent partial 误判），不是工具能力衰退。

## 修订建议清单（仅"我们导致"失败）

**无需修订**。本次 v6 baseline 63 条 FAILED **全部归属"工具不支持 / 工具自身 bug"**（charon 上游 silent partial / charon × rustc 边界 / aeneas 显式 craise / aeneas internal error / aeneas OCaml uncaught），无任何条目可归"我们 wrapper bug" / "我们 corpus bug" / "环境损坏"。

按 [`principles.md`](../../docs/design/principles.md) §六 UNKNOWN 严格语义：所有 63 条都是工具自身能力边界（含 aeneas / 自家 charon 上游 bug），按本地性原则 FAILED 站得住，工具开发者不能驳回。

我方 wrapper R7 的两道 partial 自陈 grep gate 工作正常——在 8 个本应被工具静默吞掉的 silent partial 上正确升级到 FAILED，这是 oracle 责任"不冤枉、不藏"的实测体现，不属于"我们导致"。
