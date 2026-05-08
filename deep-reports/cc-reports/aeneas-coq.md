# aeneas-coq — 特性支持评估报告

## 元数据

- **数据源**：`runs/run-1778226613-5282/`（2026-05-08T07:50:13Z–08:16:08Z UTC，146 entries × 19 工具矩阵）
- **工具配置**：`tools/aeneas-coq/`
- **工具版本**：`aeneas a14083a6` + 自家 charon `0.1.184`（commit `ed22146b`）
- **本工具实测**：n=146 / SUCCESS=87 / FAILED=59 / TIMEOUT=0，通过率 **59%**
- **时长分布**：avg 4075ms / median 1743ms / p90 9947ms / max 35173ms（`timeout_secs=600`，未触发）
- **时效声明**：本快照锚定上述 run id + 工具版本 + corpus，不构成长期承诺。

## pipeline + 前端边界

aeneas-coq 是 **Rust → Coq 的两段式纯翻译流水线**，由 `aeneas-coq-wrapper.sh` 把两段命令打包成单一 tool 入口（`set -euo pipefail`，stage 1 charon 非零直接退出，不进 stage 2）：

```
stage 1: charon cargo --preset=aeneas      →  <crate>.llbc
stage 2: aeneas -backend coq -dest coq-out  <crate>.llbc  →  coq-out/<Mod>.v + Primitives.v
```

stage 1 做完整 cargo build + 把 MIR 序列化为 LLBC；stage 2 以 LLBC 为输入做 borrow forward / backward translation（mut borrow 重写成 functional update），然后由 `Extract.ml` 的 Coq printer 分支落 `.v` 文件，并把 aeneas 自带的 Coq 运行时抽象 `Primitives.v`（约 100 条 `Axiom`，与翻译质量无关）一并 copy 到 `coq-out/`。**4 个 backend 共享同一份 charon binary 与 aeneas OCaml engine（mid-end），差异仅发生在最后的 `Extract.ml` printer 分支选择**。

**前端边界**（本测试范围）：完整跑 charon LLBC 序列化 + aeneas OCaml engine 翻译 + Coq printer 写 `.v`。**后端**（本测试不覆盖）：用户自己拿 `.v` 给 `coqc` 做下游 type-check + 手工补充 Coq proof——本测试集筛选前端特性覆盖广度，下游 prover type-check 不在本范围。成功 entry 的 `<Mod>.v` 里零 `Admitted` / `Axiom`（仅 `Primitives.v` 含 axiom）。

`entry_mode = "lib"`：runner 只把 lib target 喂给 charon，不渲染 bin harness。

错误分流：charon 阶段错误（rustc stack overflow / cargo build / charon-driver SIGABRT）落 stderr；aeneas 阶段错误以彩色 `[Error] <msg>` 行落到 **stdout**；wrapper 自身 `[aeneas-coq-wrapper] ...` 提示也在 stdout。下游分类需 stdout + stderr 一起读。

## SUCCESS 信号 + 形式严格性

**单一信号**：wrapper 最终 exit code = stage 2 aeneas 的 exit code。

判定语义：

- **exit 0** = aeneas 全程跑通 ⇔ `Errors.error_list` 空 ⇔ 翻译完整，产物 `coq-out/<Mod>.v` 写出且无 `[Error]` → SUCCESS
- **exit 1 + `Generated the partial file (because of N errors)`** = aeneas 在 mid-end 或 backend 遇 unsupported 项，仍写出**部分** `.v` 后非零退出。按宪法 §六-2"不允许 partial：工具自陈'我没全干完'必须被尊重" → FAILED
- **exit 2 / exit 101** = OCaml 未捕获异常或 charon-driver SIGABRT → FAILED

**形式严格性 — 0 误报**：✅ 形式可证。aeneas 用 `craise` 单一信号通路把所有 unsupported 推入 `Errors.error_list`；`Main.ml` 末尾 `if has_errors then exit 1`——exit 0 ⇔ error_list 空。

**形式严格性 — 0 漏报**：✅ 形式可证。aeneas 无 silent emit-stub 路径——所有 unsupported 走 `craise`；OCaml uncaught exception 同样 exit ≠ 0。

**漏报盲点**：无。

## 实测结果

### 按 feature 类目分布

矩阵里这些 feature 类别下 aeneas-coq **全部 entry 通过**（与 aeneas-fstar / aeneas-lean 完全一致）：

```
arc / assoc-type / closure / const / drop / generic / hello
impl-trait / int / panic / rc / refcell / slice / vec
```

部分通过：`int-width` 13/14（仅 `cast-float-int` 拒）、`hax-limit` 7/8、`miri-limit` 6/7、`prusti-limit` 6/8、`bigint` 6/8、`creusot-limit` 4/7、`kani-limit` 4/7、`aeneas-limit` 4/8、`closure-adv` 2/4、`charon-limit` 3/7、`industrial` 2/6、`unsafe-adv` 2/3、`box` 1/2、`enum` 1/2、`lifetime` 1/3、`repr` 1/2、`trait-obj` 1/2、`collections` 1/2、`concurrency` 1/2、`deps-complex` 1/7。

整类零通过：`error` 0/1、`float` 0/10、`gat` 0/1、`hrtb` 0/1、`iter` 0/1、`trait` 0/1、`unsafe-ptr` 0/2。

### 失败模式归类

59 个 FAILED 按 wrapper exit code + stdout/stderr 信号分桶（数据基于 `runs/run-1778226613-5282/raw/aeneas-coq/`）：

| 桶 | 数量 | exit | 信号位置 |
|---|---:|---|---|
| A. charon 阶段崩溃 | 1 | 101 | stderr `thread 'rustc' has overflowed its stack` + `signal: 6, SIGABRT` |
| B. aeneas mid-end / backend "partial file" 路径 | 49 | 1 | stdout `[Error] ...` + `Generated the partial file (because of N errors)` |
| C. OCaml uncaught exception | 9 | 2 | stderr `Uncaught exception:` 栈帧 |

A 桶唯一 entry：`trait/cyclic-bound/cyclic_bound_use`——charon-driver 在 cyclic trait bound 上递归触发 rustc stack overflow，未进 stage 2。同 entry 在 aeneas-fstar / aeneas-lean / aeneas-hol4 上均 exit 101。

B 桶 49 条主要 stdout `[Error]` 模板（按 entry 主信号归类，单 entry 可叠加多模板）：

- **`Improperly typed constant value`**（14 条）：整 `float/*` 10 条、`int-width/cast-float-int`、`enum/data-variants/enum_match_data`、`aeneas-limit/float-types`、`bigint/num-complex-ops`、`kani-limit/float-overapprox`。aeneas 在 LLBC 常量类型检查阶段拒；浮点字面常量不进 mid-end。
- **`Invalid inputs for binop` / `Invalid input for unop`**（约 8 条）：bool 位运算、f64 取负、bool/float 跨类型 binop。
- **`Unsupported / Not yet supported` 显式 todo**（约 13 条覆盖）：`Nested borrows are not supported yet`（`aeneas-limit/nested-borrow-array`、`collections/btreemap`）、`Unsupported operation: shallow-init-box`（`box/shallow-init`、`charon-limit/box-branch-init`、`closure-adv/boxed-dyn-fn`）、`Dynamic trait types are not supported yet`（`creusot-limit/dyn-trait-forbidden`）、`Function pointers are not supported yet`（`trait-obj/dyn-dispatch`、`creusot-limit/fn-ptr-reify`）、`Arrow types are not supported yet` 与 `arrow types with locally quantified regions`（`closure-adv/early-bound-lifetime`、`kani-limit/async-await`、`creusot-limit/thread-local-ref`、`lifetime/thread-local`）、`Breaks to outer loops`（`hax-limit/labelled-break`）、`unions are not supported`（`repr/union`）、`Unsupported operation: &raw mut/const`（`unsafe-adv/ptr-write`、`unsafe-ptr/raw-read`）、`Mixed declaration groups`（`kani-limit/async-await`）、`Invalid inputs for unsized cast`（`charon-limit/arc-slice-unsize`）、`Unimplemented`（`aeneas-limit/closure-if-capture`、`deps-complex/bigint-serde`）。这一桶能直接读到 aeneas 自己的"todo"信号——非异常，是阶段性能力边界。
- **`Internal error` / `Region ids should not be visited directly` / `Assertion failed`**（约 14 条）：aeneas 自标 `please file an issue`，含 backend 内部 bug 与边界情形混合。`industrial/rsa_rsa-pkcs8/{rsa_pkcs1v15_encrypt, rsa_pubkey_from_pkcs8}` 主信号是 `Region ids should not be visited directly`（含 `der::asn1` `AnyRef` / `BitStringRef` 这类带 region 参数的 unknown 类型在 `interp/Interp.ml:550` 触发）；`industrial/sha2_sha256-digest/{*}` 在 charon→aeneas JSON 边界即抛 `Charon__Generated_OfJson.translated_crate_of_json] failed on:`（hybrid-array crate 的 trait 关联类型 binding 元信息缺失）。
- **charon → LLBC IR 不完整**（4 条）：`charon-limit/async-fn`、`kani-limit/async-await` 的 `Found type error in the output of charon`；`deps-complex/{bigint-serde, chrono-serde, collections-serde, trait-serde-generic}` 的 `Inconsistent projection` / `The input arguments don't have the proper type`——serde 派生宏展开后的 LLBC 在 aeneas 投影 / argument type 检查处拒。

C 桶 9 条触发 OCaml uncaught exception，stderr 完整栈帧（与 aeneas-fstar / aeneas-lean 完全相同）：

- **`Not_found`** 栈指向 `Aeneas__Translate.trait_impl_is_builtin (Translate.ml:994-995)` → `export_trait_impl (Translate.ml:1033)`：`error/result-question`、`gat/lending-iter`、`deps-complex/error-chain`、`creusot-limit/thread-local-ref`、`lifetime/thread-local`。前置 `[Error] Can not extract trait associated types with parameters` + `[Error] Could not find: trait_decl_id: N`——含参数 trait 关联类型 + builtin trait 查表抛 `Map.Make.find` `Not_found`。
- **`Failure "Can't convert type to pattern: dyn ..."`** 栈指向 `Charon__NameMatcher.ty_to_pattern_aux`：`concurrency/thread-mutex/thread_mutex_join`、`miri-limit/thread-interleaving-partial`——`dyn Any + Send` 模式匹配未覆盖。
- 其他 internal：`lifetime/static-bound`、`deps-complex/itertools-multi`。

## 与本次测试边界的关系

- **不测下游 Coq type-check**：本测试停在 `coq-out/<Mod>.v` 落盘 + aeneas exit 0；产物是否在 Coq 中 type-check 通过、是否需手工补 proof、是否兼容 aeneas Coq 支持库——均不在本测试范围。
- **partial file 仍写出**（B 桶 49 条，stdout 末尾常见 `[Info] Generated the partial file (because of N errors)`，落盘 `coq-out/<Mod>.v` 与 `Primitives.v`），但本测试以 wrapper exit code 为单一信号，partial file 不影响 FAILED 判定（见硬指标 §六-2）。
- **wrapper 的两段 pipeline 让根因层位可读**：`[aeneas-coq-wrapper] charon exit: 0/N` 与 `[aeneas-coq-wrapper] stage 2: aeneas` 标记使每条 FAILED 都能干净判定到底落在 stage 1 还是 stage 2。本矩阵 59 条 FAILED 中 1 条在 stage 1，58 条在 stage 2。

## 历史快照声明

本数字锚定 `runs/run-1778226613-5282/`（2026-05-08）+ aeneas commit `a14083a6` + charon `ed22146b`。aeneas-coq 与 aeneas-fstar / aeneas-lean 在本矩阵 byte-identical 通过率（87/146 各）的观察对**本矩阵 entry 集合 + 当前 aeneas 版本**成立——aeneas 上游若改动 mid-end 或 Coq printer，本快照随之失效。读者引用本数字时务必同时引用 run id 与 aeneas commit hash。
