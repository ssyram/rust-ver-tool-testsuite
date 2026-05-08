# aeneas-hol4 — 特性支持评估报告

## 元数据

- **数据源**：`runs/run-1778226613-5282/`（2026-05-08T07:50:13Z–08:16:08Z UTC，146 entries × 19 工具矩阵）
- **工具配置**：`tools/aeneas-hol4/`
- **工具版本**：`aeneas a14083a6` + 自家 charon `0.1.184`（commit `ed22146b`）
- **本工具实测**：n=146 / SUCCESS=51 / FAILED=95 / TIMEOUT=0，通过率 **34%**
- **时长分布**：avg 3989ms / median 1870ms / p90 7805ms / max 32647ms（`timeout_secs=600`，未触发）
- **时效声明**：本快照锚定上述 run id + 工具版本 + corpus，不构成长期承诺。

## pipeline + 前端边界

aeneas-hol4 是 **Rust → HOL4 的两段式纯翻译流水线**，由 `aeneas-hol4-wrapper.sh` 把两段命令打包成单一 tool 入口（`set -euo pipefail`，stage 1 charon 非零直接退出，不进 stage 2）：

```
stage 1: charon cargo --preset=aeneas        →  <crate>.llbc
stage 2: aeneas -backend hol4 -dest hol4-out  <crate>.llbc  →  hol4-out/<CamelCaseMod>Script.sml
```

stage 1 做完整 cargo build + 把 MIR 序列化为 LLBC；stage 2 以 LLBC 为输入做 borrow forward / backward translation（mut borrow 重写成 functional update），然后由 `Extract.ml` 的 HOL4 printer 分支落 Standard ML 写法的 HOL4 Theory Script（`<CamelCaseCrateName>Script.sml`）。**4 个 backend 共享同一份 charon binary 与 aeneas OCaml engine（mid-end），差异仅发生在最后的 `Extract.ml` printer 分支选择**——本 backend 的差异化失败几乎全部源自此分支。

**前端边界**（本测试范围）：完整跑 charon LLBC 序列化 + aeneas OCaml engine 翻译 + HOL4 printer 写 `.sml`。**后端**（本测试不覆盖）：用户自己拿 `.sml` 给 polyml / Holmake 加载 + 手工补充 HOL4 证明——加载到 HOL4 还需要额外 `primitivesLib`，不属本测试范围。

`entry_mode = "lib"`：runner 只把 lib target 喂给 charon，不渲染 bin harness。

错误分流：charon 阶段错误（rustc stack overflow）落 stderr；aeneas 阶段 `craise` 错误以彩色 `[Error] <msg>` 落到 **stdout**；HOL4 backend 特有的 `Invalid_argument "option is None"` panic 走 OCaml uncaught exception 通路落到 **stderr**。

## SUCCESS 信号 + 形式严格性

**单一信号**：wrapper 最终 exit code = stage 2 aeneas 的 exit code。

判定语义：

- **exit 0** = aeneas 全程跑通 ⇔ `Errors.error_list` 空 ⇔ 翻译完整，产物 `hol4-out/<Camel>Script.sml` 写出 → SUCCESS
- **exit 1 + `Generated the partial file (because of N errors)`** = aeneas 在 mid-end 或 HOL4 backend 遇 unsupported 项，仍写出**部分** `.sml` 后非零退出。按宪法 §六-2 → FAILED
- **exit 2** = OCaml 未捕获异常（HOL4 backend 高频触发 `Invalid_argument "option is None"` panic，**无产物**）→ FAILED
- **exit 101** = charon-driver SIGABRT → FAILED

**形式严格性 — 0 误报**：✅ 形式可证。aeneas exit 0 ⇔ `Errors.error_list` 空 + 无 OCaml panic。

**形式严格性 — 0 漏报**：✅ 形式可证。所有 unsupported 经 `craise` 推 error_list；HOL4 backend 的 `Option.get None` 触发 `Invalid_argument` panic 仍 exit ≠ 0；charon 阶段失败由 wrapper 的 `set -euo pipefail` 直接传出。

**漏报盲点**：无。

### HOL4 backend 的硬天花板

矩阵 59% / 59% / 59% / 34% 的差距集中在 HOL4 backend：

- `ExtractBase.ml:1412 type_decl_kind_to_qualif` 在 HOL4 上 trait decl 始终返回 `None`
- `Extract.ml:3166 extract_trait_decl` 调 `Option.get` 不防御
- **LLBC 含 ≥1 个 trait declaration（含 `FnOnce` / `From` / `Iterator` / 用户自定义 trait...）即抛 `Invalid_argument "option is None"` 截断产物**
- 与 entry 是否真"用"该 trait 无关；与下游 prover 无关——这是 aeneas-hol4 upstream 的硬天花板，作为工具事实陈述

## 实测结果

### 按 feature 类目分布

矩阵里这些 feature 类别下 aeneas-hol4 **全部 entry 通过**：

```
arc / const / drop / hello / int / panic / rc / refcell / vec
```

部分通过：`int-width` 13/14、`hax-limit` 4/8、`miri-limit` 3/7、`creusot-limit` 3/7、`bigint` 3/8、`kani-limit` 3/7、`unsafe-adv` 2/3、`generic` 2/4、`box` 1/2、`enum` 1/2、`lifetime` 1/3、`repr` 1/2、`charon-limit` 1/7、`concurrency` 1/2、`aeneas-limit` 1/8。

整类零通过：`assoc-type` 0/1、`closure` 0/2、`closure-adv` 0/4、`collections` 0/2、`deps-complex` 0/7、`error` 0/1、`float` 0/10、`gat` 0/1、`hrtb` 0/1、`impl-trait` 0/1、`industrial` 0/6、`iter` 0/1、`prusti-limit` 0/8、`slice` 0/1、`trait` 0/1、`trait-obj` 0/2、`unsafe-ptr` 0/2。注：`closure / closure-adv / impl-trait / assoc-type / industrial` 在 aeneas-coq / aeneas-fstar / aeneas-lean 上多数通过——HOL4 整类落差与"含 trait declaration"高度相关。

### 失败模式归类

95 个 FAILED 按 wrapper exit code + stdout/stderr 信号分桶（数据基于 `runs/run-1778226613-5282/raw/aeneas-hol4/`）：

| 桶 | 数量 | exit | 信号位置 | 与其他 backend 关系 |
|---|---:|---|---|---|
| A. charon 阶段崩溃 | 1 | 101 | stderr `signal: 6, SIGABRT` | 共享 |
| B. aeneas mid-end / charon→IR mismatch "partial file" | 36 | 1 | stdout `[Error] ...` + `Generated the partial file` | 多数共享 |
| C1. HOL4 backend `Option.get None` panic | 52 | 2 | stderr `Aeneas__Extract.extract_trait_decl ... Invalid_argument "option is None"` | **HOL4-only 主因** |
| C2. 其他 OCaml uncaught exception | 6 | 2 | stderr `Not_found` / `Failure "Can't convert type to pattern: dyn ..."` | 共享（与 fstar/coq/lean 同 entry 同栈） |

#### A 桶（共享）

`trait/cyclic-bound/cyclic_bound_use`：charon-driver 在 cyclic trait bound 上递归触发 rustc stack overflow，未进 stage 2。

#### B 桶（36 条，多数共享）

stdout `[Error]` 模板分布：

- **`Improperly typed constant value`**（14 条）：整 `float/*` 10 条 + 含浮点 const 的 entry。aeneas mid-end 拒，与 backend 无关，与 fstar/coq/lean 同。
- **`Constant generics are not supported yet when generating code for HOL4`**（4 条 HOL4-only）：`generic/array-len`、`prusti-limit/const-generics`、`charon-limit/precise-drops-const-generic`、`hax-limit/closure-mutates-outer`。aeneas 自己针对 HOL4 写死的 guard——同 4 条在 fstar/coq/lean SUCCESS。
- **`Internal error` 走 partial file**：含 `industrial/x509-parser_cert-parse/{*}`、`miri-limit/soundness-not-guaranteed`、`hax-limit/ret-mut-ref`、`slice/index-iter` 等——多数为 HOL4 backend 能力边界（同 entry 在 fstar/coq/lean SUCCESS）。
- **其他模板共享**：`Found type error in the output of charon`（async-fn、async-await）、`Function pointers are not supported yet`、`unions are not supported`、`Unsupported operation: &raw mut/const` 等——绝大多数与 fstar/coq/lean 同信号。

#### C1 桶（52 条，HOL4-only 主因）

stderr 完整栈帧固定：

```
Uncaught exception:
  (Invalid_argument "option is None")
Raised at Stdlib.invalid_arg in file "stdlib.ml", line 30
Called from Stdlib__Option.get in file "option.ml" (inlined), line 21
Called from Aeneas__Extract.extract_trait_decl in file "extract/Extract.ml", lines 3166-3167
Called from Aeneas__Translate.export_trait_decl in file "Translate.ml", line 1019
Called from Aeneas__Translate.extract_definitions ... 
Called from Aeneas__Translate.extract_translated_crate in file "Translate.ml", line 2032
```

stdout 末尾仅以 `[Info] Imported: <crate>.llbc` 一行结束（mid-end 进度条多数显示"Translated trait declarations / impls"已走完，进入 `extract_translated_crate` 的 trait decl 提取环节才崩）。**根因**：`ExtractBase.ml:1412 type_decl_kind_to_qualif` 在 HOL4 backend 分支对 trait decl 类型始终返回 `None`，`Extract.ml:3166 extract_trait_decl` 直接 `Option.get` 触发 `Invalid_argument`——只要 LLBC 含任意 trait declaration（用户定义 trait、或来自 `core` 的 `FnOnce` / `From` / `Iterator` / `Fn` / `FnMut` / `Index` / `IntoIterator` / `Display` 等闭包 / iterator / 序列化 trait）即抛此 panic。

C1 桶覆盖几乎所有"含 trait 操作"的样例（与 entry 是否真"用"该 trait 无关）：闭包族（`closure/*`、`closure-adv/*`）、iterator / collection（`collections/*`、`assoc-type/iter-style`、`impl-trait/return-iter`、`iter/chain-collect`）、bignum / 序列化（`bigint/*` 多条、`deps-complex/*` 7 条、`industrial/rsa_rsa-pkcs8/*`）、dyn / fn-ptr / async（`trait-obj/*`、`creusot-limit/{dyn-trait-forbidden, generic-for-loop}`、`kani-limit/{async-await, loop-unwinding, stack-unwinding}`）、自定义 trait（`generic/sum-bound`、`hrtb/for-all-lifetime`、`error/result-question`、`hax-limit/*` 3 条、`prusti-limit/*` 3 条、`miri-limit/*` 2 条、`aeneas-limit/*` 5 条）等。完整列表见 raw 目录。

C1 桶占 95 个 FAILED 的 **52/95 ≈ 55%**——是 aeneas-hol4 通过率显著低于其他 3 个 backend 的**单一最大根因**。

#### C2 桶（6 条共享）

OCaml uncaught exception 但**栈帧不在** `Extract.ml:3166`，与 fstar/coq/lean 在同 entry 抛同栈：

- **`Not_found`** 栈指向 `Aeneas__Translate.trait_impl_is_builtin (Translate.ml:994-995)`：`error/result-question`（注：此条同时也在 C1 命中）、`gat/lending-iter`、`creusot-limit/thread-local-ref`、`lifetime/thread-local`。
- **`Failure "Can't convert type to pattern: dyn ..."`**：`concurrency/thread-mutex/thread_mutex_join`、`miri-limit/thread-interleaving-partial`、`lifetime/static-bound`。

### 与其他 backend 的逐 entry 对比

aeneas-coq / aeneas-fstar / aeneas-lean 在本矩阵 146 entry 上**逐 entry exit code byte-identical**（diff = 0）；aeneas-hol4 与三者相比**多出 36 条 HOL4-only 失败**（fstar/coq/lean SUCCESS 但 hol4 FAILED），分布为：C1 panic 27 / `Constant generics` guard 4 / HOL4 backend `Internal error` 走 partial 5。这 36 条全部发生在 stage 2 的 HOL4-specific extract / pretty-print 阶段——不发生在 charon 或 aeneas mid-end，证据是同一组 entry 在 fstar / coq / lean 三 backend 上都是 SUCCESS。

## 与本次测试边界的关系

- **不测下游 HOL4 加载**：本测试停在 `.sml` 落盘 + aeneas exit 0；产物能否被 polyml / Holmake 加载、是否需补 `primitivesLib` 与 HOL4 支持库——均不在本测试范围。
- **HOL4 backend 的硬天花板是 upstream 实现限制**：`type_decl_kind_to_qualif` / `extract_trait_decl` 的 `Option.get None` 在 LLBC 含 trait declaration 时确定性触发。本测试集如实暴露此天花板，不为之做 oracle 特殊化（按硬指标 §六-2 partial / panic 一律 FAILED）。
- **partial file 仍写出**（B 桶 36 条 stdout 常见 `Generated the partial file`），但本测试以 wrapper exit code 为单一信号，partial file 不影响 FAILED 判定。

## 历史快照声明

本数字锚定 `runs/run-1778226613-5282/`（2026-05-08）+ aeneas commit `a14083a6` + charon `ed22146b`。aeneas-hol4 通过率 34% vs 其他 3 个 backend 59%（35pp 落差）这一观察对**本矩阵 entry 集合 + 当前 aeneas commit `a14083a6`** 成立——aeneas 上游若修补 `extract_trait_decl Option.get` 路径，本快照的 35pp 差距将随之消解；HOL4 backend 添加 const-generic 支持等亦同。读者引用本数字时务必同时引用 run id 与 aeneas commit hash。
