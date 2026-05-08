# aeneas-lean — 特性支持评估报告

## 元数据

- **数据源**：`runs/run-1778226613-5282/`（2026-05-08T07:50:13Z–08:16:08Z UTC，146 entries × 19 工具矩阵）
- **工具配置**：`tools/aeneas-lean/`
- **工具版本**：`aeneas a14083a6` + 自家 charon `0.1.184`（commit `ed22146b`）
- **本工具实测**：n=146 / SUCCESS=87 / FAILED=59 / TIMEOUT=0，通过率 **59%**
- **时长分布**：avg 3854ms / median 1834ms / p90 8056ms / max 36310ms（`timeout_secs=600`，未触发）
- **时效声明**：本快照锚定上述 run id + 工具版本 + corpus，不构成长期承诺。

## pipeline + 前端边界

aeneas-lean 是 **Rust → Lean 4 的两段式纯翻译流水线**，由 `aeneas-lean-wrapper.sh` 把两段命令打包成单一 tool 入口（`set -euo pipefail`，stage 1 charon 非零直接退出，不进 stage 2）：

```
stage 1: charon cargo --preset=aeneas      →  <crate>.llbc
stage 2: aeneas -backend lean -dest lean-out  <crate>.llbc  →  lean-out/<Mod>.lean
```

stage 1 做完整 cargo build + 把 MIR 序列化为 LLBC；stage 2 以 LLBC 为输入做 borrow forward / backward translation（把 mut borrow 重写成 functional update），然后由 `Extract.ml` 的 Lean printer 分支落 `.lean` 文件。**4 个 backend 共享同一份 charon binary 与 aeneas OCaml engine（mid-end），差异仅发生在最后的 `Extract.ml` printer 分支选择（`-backend lean / coq / fstar / hol4`）**。

**前端边界**（本测试范围）：完整跑 charon LLBC 序列化 + aeneas OCaml engine 翻译 + Lean printer 写文件落盘。**后端**（本测试不覆盖）：用户自己拿 `.lean` 给 `lean` / `lake` 做下游 type-check 与证明——本测试集筛选前端特性覆盖广度，下游 prover type-check 不在本范围。

`entry_mode = "lib"`：runner 只把 lib target 喂给 charon，不渲染 bin harness（aeneas 操作整 lib，没有"per-fn"调用形式）。

错误分流到不同 stream：charon 阶段错误（rustc stack overflow / cargo build failure / charon-driver SIGABRT）落 stderr；aeneas 阶段错误以彩色 `[Error] <msg>` 行落到 **stdout**（不是 stderr）；wrapper 自身 `[aeneas-lean-wrapper] ...` 提示也在 stdout。下游分类需 stdout + stderr 一起读。

## SUCCESS 信号 + 形式严格性

**单一信号**：wrapper 的最终 exit code = stage 2 aeneas 的 exit code（除非 stage 1 提前失败 wrapper 自己 exit 1）。

判定语义：

- **exit 0** = aeneas 全程跑通 ⇔ `Errors.error_list` 空 ⇔ 翻译完整，产物 `lean-out/<Mod>.lean` 写出且无 `[Error]` → SUCCESS
- **exit 1 + `Generated the partial file (because of N errors)` 路径** = aeneas 在 mid-end 或 backend 遇 unsupported 项，仍写出**部分** `.lean` 后非零退出。按宪法 §六-2"不允许 partial：工具自陈'我没全干完'必须被尊重" → FAILED
- **exit 2 / exit 101** = OCaml 未捕获异常或 charon-driver SIGABRT，无完整产物 → FAILED

**形式严格性 — 0 误报（上限保证 §三-3-2.b）**：✅ 形式可证。aeneas 用 `craise` 单一信号通路把所有 unsupported 推入 `Errors.error_list`，`Main.ml` 末尾 `if has_errors then exit 1`——exit 0 ⇔ error_list 空。SUCCESS 等价于"aeneas 自陈完整翻译"。

**形式严格性 — 0 漏报（下限诚实 §三-3-2.c）**：✅ 形式可证。aeneas 无 silent emit-stub 路径——所有 unsupported 走 `craise` push error_list；OCaml uncaught exception 同样 exit ≠ 0。

**漏报盲点**：无。

## 实测结果

### 按 feature 类目分布

矩阵里这些 feature 类别下 aeneas-lean **全部 entry 通过**：

```
arc / assoc-type / closure / const / drop / generic / hello
impl-trait / int / panic / rc / refcell / slice / vec
```

部分通过：`int-width` 13/14、`hax-limit` 7/8、`miri-limit` 6/7、`prusti-limit` 6/8、`bigint` 6/8、`kani-limit` 4/7、`creusot-limit` 4/7、`aeneas-limit` 4/8、`closure-adv` 2/4、`charon-limit` 3/7、`industrial` 2/6、`unsafe-adv` 2/3、`box` 1/2、`enum` 1/2、`lifetime` 1/3、`repr` 1/2、`trait-obj` 1/2、`collections` 1/2、`concurrency` 1/2、`deps-complex` 1/7。

整类零通过：`error` 0/1、`float` 0/10、`gat` 0/1、`hrtb` 0/1、`iter` 0/1、`trait` 0/1、`unsafe-ptr` 0/2。`float/*` 10/10 全失败是单 feature 最显著的整类拒。

### 失败模式归类

59 个 FAILED 按 wrapper exit code + stdout/stderr 信号分桶（数据基于 `runs/run-1778226613-5282/raw/aeneas-lean/`）：

| 桶 | 数量 | exit | 信号位置 |
|---|---:|---|---|
| A. charon 阶段崩溃 | 1 | 101 | stderr `thread 'rustc' has overflowed its stack` + `signal: 6, SIGABRT` |
| B. aeneas mid-end / backend "partial file" 路径 | 49 | 1 | stdout `[Error] ...` + `Generated the partial file (because of N errors)` |
| C. OCaml uncaught exception | 9 | 2 | stderr `Uncaught exception:` 栈帧 |

A 桶唯一 entry：`trait/cyclic-bound/cyclic_bound_use`——charon-driver 递归处理 cyclic trait bound 触发 rustc stack overflow，不进 stage 2。同 entry 在其他三个 backend 上同样 101。

B 桶 49 条覆盖以下 stdout `[Error]` 模板（按出现 entry 数排序，单 entry 可命中多模板，下表按 entry 主信号归桶）：

- **`Improperly typed constant value`**（14 条）：整 `float/*` 10 条、`int-width/cast-float-int`、`enum/data-variants/enum_match_data`、`aeneas-limit/float-types`、`bigint/num-complex-ops`、`kani-limit/float-overapprox`。aeneas 在 LLBC 常量类型检查阶段拒。
- **`Invalid inputs for binop` / `Invalid input for unop`**（8 条覆盖）：`aeneas-limit/bool-bitwise-op`、`float/total-order`（`wrap.- on f64`）等——aeneas 算术节点对 bool 位运算、f64 取负、bool/float 跨类型 binop 拒。
- **`Unsupported / Not yet supported` 显式 todo**（约 12 条，子模板含 `Nested borrows`、`Function pointers are not supported yet`、`Arrow types are not supported yet`、`Unsupported operation: shallow-init-box`、`Dynamic trait types are not supported yet`、`Breaks to outer loops`、`unions are not supported`、`Unsupported operation: &raw mut/const`、`Mixed declaration groups`、`Invalid inputs for unsized cast`、`Unimplemented`）：aeneas 自带的能力边界 todo——非异常，是阶段性能力声明。
- **`Internal error` / `Region ids should not be visited directly` / `Assertion failed`**（约 14 条）：aeneas 自标 `please file an issue`，含 backend 内部 bug 与边界情形混合，本报告不进一步分辨。
- **charon → LLBC IR 不完整后 aeneas 二次拒**（4 条，含 `charon-limit/async-fn`、`kani-limit/async-await` 的 `Found type error in the output of charon`；`deps-complex/{bigint-serde,chrono-serde,collections-serde,trait-serde-generic}` 的 `Inconsistent projection`）。
- **industrial 实战**：`industrial/rsa_rsa-pkcs8/{rsa_pkcs1v15_encrypt, rsa_pubkey_from_pkcs8}`、`industrial/sha2_sha256-digest/{*}` 4 条以 partial-file 路径退出，主信号是 `Region ids should not be visited directly`（rsa）与 `Generated_OfJson.translated_crate_of_json] failed on:` JSON 反序列化异常（sha2 — charon→aeneas IR 兼容性问题）。

C 桶 9 条触发 OCaml uncaught exception，stderr 完整栈帧：

- **`Not_found`** stack 指向 `Aeneas__Translate.trait_impl_is_builtin (Translate.ml:994-995)` → `export_trait_impl (Translate.ml:1033)`：`error/result-question`、`gat/lending-iter`、`deps-complex/error-chain`、`creusot-limit/thread-local-ref`、`lifetime/thread-local`。前置都伴随 `[Error] Can not extract trait associated types with parameters` / `[Error] Could not find: trait_decl_id: N`——含参数 trait 关联类型 + builtin trait 查表触发同一 `Map.Make.find` 抛 `Not_found`。
- **`Failure "Can't convert type to pattern: dyn ..."`** 栈指向 `Charon__NameMatcher.ty_to_pattern_aux`：`concurrency/thread-mutex/thread_mutex_join`、`miri-limit/thread-interleaving-partial`。
- 其他 internal 形态：`lifetime/static-bound`、`deps-complex/itertools-multi`。

## 与本次测试边界的关系

- **不测下游 prover type-check**：本测试停在 `lean-out/<Mod>.lean` 落盘 + aeneas exit 0；产物是否在 Lean 4 中 type-check 通过、是否需要补充 lemma、aeneas 的 Lean 支持库 (`Base/`) 是否就位——均不在本测试范围。
- **stage 1 charon 失败**与 stage 2 aeneas 失败混在同一 exit code 通道，但 wrapper 在 stdout 打 `[aeneas-lean-wrapper] charon exit: 0/N` 与 `[aeneas-lean-wrapper] stage 2: ...` 两段标记，使根因层位事后可读。本矩阵 59 条 FAILED 中 1 条在 stage 1（A 桶），58 条都在 stage 2——意味着对本矩阵覆盖的 entry 集合，aeneas-lean 的"接受 / 不接受"信号几乎完全由 stage 2 决定。
- **partial file 仍写出**（B 桶 49 条 stdout 末尾常见 `[Info] Generated the partial file (because of N errors)`）。本测试以 wrapper 最终 exit code 为单一信号，partial file 的存在不影响 FAILED 判定（见硬指标 §六-2 与 README 对应说明）。

## 历史快照声明

本数字锚定 `runs/run-1778226613-5282/`（2026-05-08）+ aeneas commit `a14083a6` + charon `ed22146b`。4 backend 通过率几乎一致（aeneas-coq/fstar/lean 各 87/146）这一观察对**本矩阵 entry 集合 + 当前 aeneas 版本**成立——aeneas 上游若改动 mid-end 或 Lean printer，本快照随之失效。读者引用本数字时务必同时引用 run id 与 aeneas commit hash。
