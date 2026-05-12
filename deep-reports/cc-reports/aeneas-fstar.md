# aeneas-fstar — 特性支持评估报告（v6 final post-P35 baseline）

## 元数据

- **数据源**：`runs/run-1778560393-59119/`（2026-05-12 v6 final post-P35）
- **工具配置**：`tools/aeneas-fstar/`
- **工具版本**：`aeneas a14083a6`（OCaml engine + Extract.ml F* printer）+ 自家 `charon` 前端 binary（`/tmp/ts-tools-install/charon/bin/charon`，runner env injected）
- **本工具实测**：n=161 / SUCCESS=98 / FAILED=63 / UNKNOWN=0，通过率 **60.9%**
- **时长分布**：avg 3497ms / median 1340ms / p90 6369ms / max 44445ms（`timeout_secs=600`，未触发）
- **宪法 baseline**：`principles.md` v8（P27 修宪 / P31 wrapper 归因 / §六 当前 crate 焦点 / UNKNOWN 严格语义）
- **时效声明**：本快照锚定上述 run id + 工具版本 + corpus；工具升级后解释力衰减是必然，不构成长期承诺

## pipeline + 前端边界

aeneas-fstar 是 **Rust → F* 的两段式纯翻译流水线**，由 `aeneas-fstar-wrapper.sh` 把两段命令打包成单一 tool 入口（`set -euo pipefail`，stage 1 charon 非零直接退出，不进 stage 2）：

```
stage 1: charon cargo --preset=aeneas          →  <crate>.llbc
stage 2: aeneas -backend fstar -dest fstar-out  <crate>.llbc  →  fstar-out/<Mod>.fst
```

- stage 1 做完整 cargo build + 把当前 entry crate 的 MIR 序列化为 LLBC（按 §六 当前 crate 焦点：外部 std / core / 第三方依赖被 opaque-fy 不算 silent partial，但若 charon 自陈 partial 仍触发 wrapper gate）
- stage 2 以 LLBC 为输入做 borrow forward/backward translation，由 `Extract.ml` 的 F* printer 分支落 `.fst` 文件
- **4 个 backend（fstar/coq/lean/hol4）共享同一份 charon binary 与 aeneas OCaml engine（mid-end）**，差异仅在 `Extract.ml` printer 分支

**前端边界**（本测试范围）：完整跑 charon LLBC 序列化 + aeneas OCaml engine 翻译 + F* printer 写 `.fst`。**后端**（本测试不覆盖）：用户自己拿 `.fst` 给 `fstar.exe` 做 type-check + Z3 提交。

**项目维护层 vs 官方工具层**（按 `tool-integration.md` §四.5）：

- **官方层**：charon binary（`charon-driver` + Rust → LLBC）+ aeneas OCaml engine（mid-end + Extract.ml）
- **项目维护层**：`aeneas-fstar-wrapper.sh`（两段拼接 + 两个 silent-partial 自陈 gate）
- 失败归因切割按"最近责任主体"：wrapper grep 命中 → wrapper 抓住官方层的真 partial 自陈（FAILED 站得住，wrapper 不背锅）；官方层直接 exit ≠ 0 → 工具自身能力边界 FAILED；wrapper 本身解析 / shell / IO 错 → UNKNOWN (b) 类

`entry_mode = "lib"`：runner 只把 lib target 喂给 charon，不渲染 bin harness（无 main entry，符合 entry-fn 用 lib 暴露的约定）。

## SUCCESS 信号 + 形式严格性

按宪法 §六 双通路 partial 暴露：

- **主信号通路**：wrapper 最终 exit code = stage 2 aeneas exit code = aeneas `Errors.error_list` 空 ⇔ `Main.ml` `if has_errors then exit 1` 不触发
- **wrapper 补抓通路**（v6 cc-route audit 2026-05-12 已落地）：
  1. **charon 阶段 stderr gate**（wrapper 第 45-49 行）：stage 1 charon exit 0 但 stderr 含 `is not supported` 或 `^error:` → FAILED（防 charon silent partial — charon 把 unsupported 项 opaque-fy 后继续走，但 stderr 留下自陈痕迹）
  2. **aeneas 阶段 Warn gate**（wrapper 第 80-84 行）：stage 2 aeneas exit 0 但 stdout/stderr 含 4 类 Warn 自陈（`model will not type-check` / `generated code will likely be incorrect` / `seems to be missing the corresponding field` / `could not find the information for item`）→ FAILED（防 aeneas Warn-channel silent partial — 不走 `craise` 但工具自陈 partial）

形式严格性 0 误报 / 0 漏报状态（按 `tool-integration.md` §三 / §四）：

- **0 误报**：✅ 主通路形式可证。aeneas exit 0 ⇔ `Errors.error_list` 空（`craise` 单一入口 + `Main.ml` exit 决定 — `tool-integration.md` §三 直接列出的工具之一）。wrapper 补抓通路按 §四.2 双向实测—— 4 个 Warn 字符串与 charon `^error:` / `is not supported` 都是 aeneas/charon 源码内部识别为 partial 时才发的字符串，不会误命中合法代码
- **0 漏报**：⚠️ 主通路形式可证（`craise` 单一）；wrapper 补抓通路属 §四.3 "实践有效性"——抓**已知** silent path（v6 cc-route audit 时枚举的 4+2 个 Warn/Error 模式），上游引入新 silent pattern 时机制会滞后

**漏报盲点（诚实声明）**：

- ✅ 已封堵：aeneas Warn 通道 4 类（mutually-recursive trait / associated type 警告 / builtin model 缺字段 / core trait method silent drop）；charon 通道 2 类（`is not supported` warning + `^error:` type-error-after-transformations 但 charon 仍 exit 0）
- ⚠️ 仍可能盲点：上游若新增不带这 6 个标记字符串的 silent partial 路径 → 需扩 wrapper grep
- ⚠️ aeneas / charon 完全 skip item（既不写 sorry / 不发 Warn / Error，item 不出现在产物里）—— 与同族 aeneas-lean / aeneas-coq / aeneas-hol4 共享此盲点；按 §六 当前 crate 焦点降低概率但不消除：本测试集 entry 函数命名固定（`__ts_inner::<entry>`）+ entry-mode lib 强制 reach，若 silent skip 命中 entry crate 自有 item 才算真漏报
- 主信号通路本身不形式证明 SUCCESS ⟹ 真 SUCCESS——避免按 P30 已推翻的"形式可证 0 漏报"过强自陈

## 失败分桶（按 `tool-integration.md` §四.5 归因分类）

63 个 FAILED 按 wrapper exit code + stdout/stderr 主信号归一类（每桶单一归因）：

### 桶 A：charon stage rustc 栈溢出（1 case）— 工具不支持

代表 entry：`trait/cyclic-bound/cyclic_bound_use`

stderr 特征：

```
thread 'rustc' (...) has overflowed its stack
fatal runtime error: stack overflow, aborting
error: could not compile `trait_cyclic_bound` (lib)
... charon-driver ... (signal: 6, SIGABRT: process abort signal)
[aeneas-fstar-wrapper] charon failed: exit 101
```

**归因**：cyclic trait bound 让 charon-driver 包装的 rustc 递归 → stack overflow → SIGABRT。错误发生在 charon 官方驱动调 rustc 上，属官方层工具能力边界。
**处理**：不修。本地性原则下 FAILED 站得住。

### 桶 B：charon stage silent partial（wrapper-gated）（8 case）— 工具不支持

charon exit 0 但 stderr 含 `is not supported` 或 `^error:`，wrapper grep 拦截升 FAILED。

完整 entry 列表（按 charon 自陈）：

- `box/shallow-init/shallow_init_box` — `Could not reconstruct \`Box\` initialization; branching during \`Box\` initialization is not supported.`
- `charon-limit/box-branch-init/vec_with_early_return` — 同上
- `charon-limit/copy-deref-closure/deref_copy_in_closure` — `error: Type error after transformations:`
- `charon-limit/inline-asm/nop_via_asm` — `Inline assembly is not supported`
- `closure-adv/boxed-dyn-fn/boxed_dyn_fn` — `error: Type error after transformations:`（dyn 通道）
- `deps-complex/itertools-multi/itertools_multi` — itertools 内部 type error
- `industrial/sha2/sha256-digest/sha256_digest_one_shot` — hybrid-array crate 触发 `error: Type error after transformations` + `Could not compute the value of Self::Size`
- `industrial/sha2/sha256-digest/sha256_digest_incremental` — 同上

**归因**：charon 官方层把 unsupported 项 opaque-fy 后继续 exit 0；这是 charon 设计选择（继续把能翻译的部分跑完），但对本测试集"不允许 partial"精神（宪法 §六 不冤枉）而言是工具能力边界。wrapper grep 把这条 silent path 显式化为 FAILED。
**处理**：不修。FAILED 站得住——charon stderr 自陈 partial，wrapper 只是把它从 exit 0 显式化。

### 桶 C：aeneas stage Warn-channel partial（wrapper-gated）（11 case）— 工具不支持

aeneas exit 0 但 stdout 含 4 类 Warn 自陈，wrapper grep 拦截升 FAILED。

完整 entry 列表：

- `aeneas-limit/mutually-recursive-traits/trigger_mutually_recursive_traits` — mutually-recursive trait
- `deps-complex/bigint-serde/bigint_serde`
- `deps-complex/chrono-serde/chrono_serde`
- `deps-complex/collections-serde/collections_serde`
- `deps-complex/error-chain/error_chain`
- `deps-complex/trait-serde-generic/trait_serde_generic`
- `error/result-question/result_question`
- `gat/lending-iter/gat_lending` — GAT lending iterator
- `impl-trait/return-iter/impl_trait_iter`
- `iter/chain-collect/iter_chain_collect`
- `kani-limit/async-await/run_async_add`

主要 Warn 模式：`model will not type-check`（mutually-recursive trait / parameterized associated type）+ `generated code will likely be incorrect`。

**归因**：aeneas mid-end 把 unsupported trait 形态生成"型式形似但可能不可证"的代码 + 自陈 Warn，仍 exit 0。这是 aeneas 设计选择（best-effort partial 出货），按本测试集"不允许 partial"精神 → FAILED。
**处理**：不修。FAILED 站得住——aeneas 自陈 partial，wrapper 把 exit 0 显式化。

### 桶 D：aeneas mid-end `craise` 路径 `Generated the partial file`（38 case）— 工具不支持

aeneas mid-end 或 backend 在翻译时遇 unsupported 构造，走 `craise` 累加到 `Errors.error_list`，`Main.ml` exit 1，stdout 末尾印 `[Info] Generated the partial file (because of N errors, ...)` + 部分 `.fst`。按宪法 §六 "不允许 partial" → FAILED。

按 `[Error]` 首消息聚合（一个 entry 取首错；本快照 38 个 entry 全部含 "Generated the partial file" 字串）：

| `[Error]` 模板 | N | 代表 entry |
|---|---:|---|
| `Improperly typed constant value` | 14 | 整 `float/*` 9 条（basic/bits/cast-int/cast-widening/cmp/nan-prop/round/special-vals/transcendental）、`int-width/cast-float-int`、`enum/data-variants/enum_match_data`、`aeneas-limit/float-types`、`bigint/num-complex-ops`、`kani-limit/float-overapprox/trigger_check_sin_cos_identity` |
| `Internal error, please file an issue` | 3 | `bigint/num-traits-abstract`、`hrtb/for-all-lifetime/hrtb_apply`、`prusti-limit/spec-entailment-unsupported` |
| `Assertion failed: new value doesn't have the same type as its destination` | 3 | `charon-limit/generic-to-dyn-unsize/boxed_display_from_u32`、`kani-limit/stack-unwinding/trigger_divide_with_recovery`、`prusti-limit/shallow-borrow-match-guard` |
| `Arrow types are not supported yet` | 2 | `closure-adv/early-bound-lifetime`、`creusot-limit/fn-ptr-reify/get_fn_ptr` |
| `Region ids should not be visited directly` | 2 | `industrial/rsa/rsa-pkcs8/{rsa_pubkey_from_pkcs8, rsa_pkcs1v15_encrypt}` |
| `Invalid inputs for binop` | 1 | `aeneas-limit/bool-bitwise-op` |
| `Invalid input for unop: wrap.- on f64` | 1 | `float/total-order` |
| `Unimplemented` / `Mixed declaration groups` | 1 | `aeneas-limit/closure-if-capture` |
| `Found a case of unsupported nested borrows` | 1 | `collections/btreemap/btreemap_basic` |
| `Nested borrows are not supported yet` | 1 | `aeneas-limit/nested-borrow-array` |
| `Invalid inputs for unsized cast` | 1 | `charon-limit/arc-slice-unsize` |
| `Found type error in the output of charon` | 1 | `charon-limit/async-fn/async_forty_two`（aeneas 拒 charon 给出的 LLBC） |
| `Dynamic trait types are not supported yet` | 1 | `creusot-limit/dyn-trait-forbidden` |
| `Function pointers are not supported yet` | 1 | `trait-obj/dyn-dispatch` |
| `Breaks to outer loops are not supported yet` | 1 | `hax-limit/labelled-break` |
| `unions are not supported` | 1 | `repr/union/repr_union` |
| `Unsupported operation: &raw mut` | 1 | `unsafe-adv/ptr-write` |
| `Unsupported operation: &raw const` | 1 | `unsafe-ptr/raw-read` |
| `Charon failed to compile constant: ConstantExprKind::Cast` | 1 | `unsafe-ptr/raw-ptr-const/raw_ptr_const_match` |

（合计 38；每行 N 由 `[Error]` 首消息聚合，与 wrapper exit=1 非桶 B/C 行总数一致）

**归因**：均为 aeneas mid-end / Extract.ml backend 显式 `craise`，对应工具自陈"我不支持这个构造"。`Improperly typed constant value`（float / 大整数常量超 mid-end 类型模型）/ `Invalid inputs for binop` / `Arrow types` / `Function pointers` / `Nested borrows` / `Region ids should not be visited directly` 等都是 aeneas 源码中标注的 unsupported 路径或 `assert false` 触发。
**处理**：不修。本地性原则下 FAILED 站得住，工具能力边界。

### 桶 E：OCaml uncaught exception（5 case）— 工具不支持

aeneas exit 2，stdout 末尾 `Uncaught exception:` + OCaml 栈帧。

完整 entry 列表（exit 2）：

- `concurrency/thread-mutex/thread_mutex_join` — `Failure ...` at `Charon__NameMatcher.ty_to_pattern_aux`
- `creusot-limit/thread-local-ref/read_thread_local` — `Failure "Can't convert type to pattern: !"`（NameMatcher 未覆盖 `!` never type）
- `lifetime/static-bound/static_bound` — `Failure ...` at NameMatcher
- `lifetime/thread-local/thread_local_read` — `Failure "Can't convert type to pattern: !"`
- `miri-limit/thread-interleaving-partial/unsynchronised_counter_race` — `Failure ...` at `Aeneas__Translate.trait_impl_is_builtin`

**归因**：aeneas mid-end 内部异常未被 `craise` 包装——`trait_impl_is_builtin` 查表 `Not_found`、`ty_to_pattern_aux` 类型 → pattern 转换缺 case（含 never type `!`），都是 aeneas 源码层未覆盖 case 或上游 OCaml exception。aeneas 自身的工程边界。
**处理**：不修。本地性原则下 FAILED 站得住（exit ≠ 0 是 aeneas 出的，按宪法 §六 "工具自身能力边界"）。

## 与 aeneas-coq / aeneas-lean / aeneas-hol4 的关系

aeneas 4 backend 共享同一份 charon + OCaml engine（mid-end），差异仅在 `Extract.ml` printer 分支。**本矩阵 161 entry 上 4 backend 的 status 分布预期高度一致**——mid-end `craise`（桶 D）、Warn 通道（桶 C）、charon-stderr gate（桶 B）、OCaml exn（桶 E）、栈溢出（桶 A）都在 printer 选择之前已确定，仅在 printer 阶段对个别 entry（如名字冲突 / 关键字冲突 / fstar/coq/lean/hol4 各自语法忌讳）可能出现 backend 间分化。

读者比较 4 backend 数字时需注意：差异主要在 printer 阶段，不构成"backend X 能力强 / 弱"的判断——共享 mid-end 决定了大部分 entry 上的 status 一致性。

## v5.1 → v6 ΔS 解释

- v5.1 baseline：`runs/run-1778226613-5282/`（2026-05-08），146 entries / SUCCESS=87 / FAILED=59，通过率 59.6%
- v6 final post-P35（本次）：161 entries / SUCCESS=98 / FAILED=63 / UNKNOWN=0，通过率 60.9%

ΔS = +11 / ΔF = +4 来源：

- **corpus 扩张**：+15 entries（新增 industrial / closure-adv / unsafe-ptr / runnable 等类目）
- **wrapper 补抓 gate（v6 cc-route audit 2026-05-12）**：v5.1 时 wrapper 已有主信号 + craise，v6 加入 charon-stderr gate + Warn-channel gate 把若干原 v5.1 silent SUCCESS 升 FAILED（桶 B 8 + 桶 C 11 = 19 个 entry 的归属由此通道决定）
- 通过率小幅升（59.6% → 60.9%）：扩张项 SUCCESS 多 + gate 抓回若干虚高 SUCCESS，净效应通过率几乎稳定，符合"加 gate 不显著降通过率"的诚实分布

## 修订建议清单（仅"我们导致"失败）

**无需修订**——所有 63 个 FAILED 均归一"工具不支持"或"工具自身能力边界"类（A / B / C / D / E 五桶全部）。

具体核查（按 `tool-integration.md` §四.5 "最近责任主体"判据）：

- 桶 A（rustc stack overflow）：错误发生在 charon-driver 包的 rustc 中，属官方层
- 桶 B（charon silent partial）：wrapper grep 把 charon stderr 自陈的 silent partial 升 FAILED——wrapper 工作正确，charon 官方层 partial 是工具能力边界
- 桶 C（aeneas Warn-channel partial）：wrapper grep 把 aeneas 自陈的 Warn-channel partial 升 FAILED——wrapper 工作正确，aeneas 官方层 partial 是工具能力边界
- 桶 D（aeneas `craise` 路径 + Generated partial file）：aeneas 主信号通路 exit 1 自陈，工具自身能力边界
- 桶 E（OCaml uncaught exit 2）：aeneas 内部异常 exit 2，工具自身能力边界

无项目维护 wrapper bug（无 IO / shell / 解析错） / 无 corpus 引入的非法 Rust / 无环境损坏 / 无 UNKNOWN 升 FAILED 误判。
