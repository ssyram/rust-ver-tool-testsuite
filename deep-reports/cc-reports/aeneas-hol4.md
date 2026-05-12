# aeneas-hol4 — 特性支持评估报告（v6 baseline）

## 元数据

- **数据源**：`runs/run-1778560393-59119/`（2026-05-12T04:33:13Z–04:55:27Z UTC，161 entries × 工具矩阵；v6 final）
- **工具配置**：`tools/aeneas-hol4/`
- **工具版本**：`aeneas a14083a6` + 自家 charon `0.1.184`（commit `ed22146b`，路径 `/tmp/aeneas-src/charon/bin/charon`）
- **本工具实测**：n=161 / SUCCESS=65 / FAILED=94 / UNKNOWN=2，通过率 **40.4%**
- **时长分布**：avg 3526ms / median 1329ms / p90 7880ms / max 43794ms（`timeout_secs=600`，未触发）
- **宪法 baseline**：`principles.md` v8（P27 修宪后 / P31 法律传导后）
- **时效声明**：本快照锚定上述 run id + 工具版本 + corpus，不构成长期承诺。

## pipeline + 前端边界

aeneas-hol4 是 **Rust → HOL4 的两段式纯翻译流水线**，由项目维护的 `aeneas-hol4-wrapper.sh` 把两段命令打包成单一 tool 入口（`set -euo pipefail`，stage 1 charon 非零直接退出，不进 stage 2）：

```
stage 1: charon cargo --preset=aeneas          →  <crate>.llbc
stage 2: aeneas -backend hol4 -dest hol4-out   →  hol4-out/<CamelCaseMod>Script.sml
```

stage 1 做完整 cargo build + 把 MIR 序列化为 LLBC；stage 2 以 LLBC 为输入做 borrow forward / backward translation（mut borrow 重写成 functional update），然后由 `Extract.ml` 的 HOL4 printer 分支落 Standard ML 写法的 HOL4 Theory Script。4 个 aeneas backend 共享同一份 charon binary 与 aeneas OCaml engine（mid-end），差异仅发生在最后的 `Extract.ml` printer 分支。

**前端边界**（本测试范围）：完整跑 charon LLBC 序列化 + aeneas OCaml engine 翻译 + HOL4 printer 写 `.sml`。**后端**（本测试不覆盖）：用户自己拿 `.sml` 给 polyml / Holmake 加载 + 手工补充 HOL4 证明 + `primitivesLib` 支持库。

**项目维护组件 vs 工具组件**：

- 项目维护：`aeneas-hol4-wrapper.sh`（两段调度 + 双 oracle gate）、`tool.toml`、entry mode 配置
- 工具组件：`charon` binary、`aeneas` OCaml engine（含 HOL4 backend）

`entry_mode = "lib"`：runner 只把 lib target 喂给 charon，不渲染 bin harness。

## SUCCESS 信号 + 形式严格性

按宪法 §六 双通路 partial 暴露：

**主信号通路**：wrapper 最终 exit code = stage 2 aeneas 的 exit code（stage 1 非零直接 propagate）。

判定语义：

- **exit 0** = aeneas 全程跑通 ⇔ `Errors.error_list` 空 + 无 OCaml panic + 产物 `hol4-out/<Camel>Script.sml` 写出 → SUCCESS
- **exit 1 + `Generated the partial file ...`** = aeneas 遇 unsupported 项写出 partial `.sml` → FAILED（宪法 §六-2 不允许 partial）
- **exit 2** = OCaml 未捕获异常（HOL4 backend 高频 `Invalid_argument "option is None"` panic，无产物）→ FAILED
- **exit 101** = charon-driver SIGABRT → FAILED

**wrapper 补抓通路**（v6 cc-route audit 后加入）：

1. **charon-stage silent-partial gate**：charon exit 0 但 stderr 含 `is not supported` 或 `^error:` → wrapper 强制 exit 1（封堵 charon opaque-fy 后静默降级）
2. **aeneas Warn-channel partial gate**：aeneas exit 0 但 stdout/stderr 含 4 类 Warn 自陈（`model will not type-check` / `generated code will likely be incorrect` / `seems to be missing the corresponding field` / `could not find the information for item`）→ wrapper 强制 exit 1

**形式严格性 — 0 误报**：实测 + 源码两通路审视。aeneas exit 0 ⇔ `Errors.error_list` 空 + 无 OCaml panic；wrapper 双 gate 捕获两类"exit 0 但实际 partial"。

**形式严格性 — 0 漏报**：实测 + wrapper 双通路封堵。主通路 `craise` / `Invalid_argument` panic → exit ≠ 0；v6 cc-route audit (2026-05-12) 暴露的 Warn 通道 partial 已由 wrapper grep gate 拦截（mutually-recursive trait / associated type / builtin model 缺字段 / core trait method silent drop 四类）。

**漏报盲点**（诚实声明）：

- 上游若新增 Warn 通道 partial 自陈措辞（4-pattern 外的新句式）→ 需扩展 wrapper grep pattern list
- charon silent-partial gate 当前 grep `is not supported` 与 `^error:`；上游若改用新措辞同样需扩展
- 已 wrapper 封堵的 4 类 + charon 静默降级 2 类不再是漏报，但属"实测 + wrapper 封堵"而非"形式可证"

## HOL4 backend 的硬天花板

矩阵 60%+ / 60%+ / 60%+ / 40% 的差距集中在 HOL4 backend：

- `ExtractBase.ml:1412 type_decl_kind_to_qualif` 在 HOL4 上 trait decl 始终返回 `None`
- `Extract.ml:3166 extract_trait_decl` 调 `Option.get` 不防御
- **LLBC 含 ≥1 个 trait declaration（含 `FnOnce` / `From` / `Iterator` / 用户自定义 trait...）即抛 `Invalid_argument "option is None"` 截断产物**
- 与 entry 是否真"用"该 trait 无关——这是 aeneas-hol4 upstream 的硬天花板

## 失败分桶（按 P31 §四.5 归因分类）

94 个 FAILED 按 exit code + stderr/stdout 信号分桶。

### 桶 C1：HOL4 backend `extract_trait_decl Option.get None` panic（47 case，HOL4-only 主因）

代表 entry：`aeneas-limit/closure-if-capture/...`、`assoc-type/iter-style/...`、`bigint/bigint-arith/...`、`closure/*`、`closure-adv/*`、`collections/*`、`deps-complex/*`（7 条）、`error/result-question`、`hrtb/for-all-lifetime`、`industrial/rsa_*`、`iter/chain-collect`、`prusti-limit/*`（多条）、`trait-obj/*`，等。

stderr / stdout 特征：

```
Uncaught exception:
  (Invalid_argument "option is None")
Raised at Stdlib.invalid_arg in file "stdlib.ml", line 30
Called from Stdlib__Option.get in file "option.ml" (inlined), line 21
Called from Aeneas__Extract.extract_trait_decl in file "extract/Extract.ml", lines 3166-3167
```

**归因**：工具不支持（aeneas-hol4 backend `Extract.ml + ExtractBase.ml` 实现硬限制）。
**处理**：不修。本地性原则下 FAILED 站得住，工具开发者不能驳回。占 94 FAILED 的 47/94 ≈ 50%，是 aeneas-hol4 通过率显著低于其他 3 个 backend 的**单一最大根因**。

### 桶 B：aeneas mid-end / HOL4 backend 写 `Generated the partial file`（34 case）

代表 entry：`float/*`（10 条全数）、`enum/data-variants`、`charon-limit/async-fn`、`creusot-limit/fn-ptr-reify`、`unsafe-ptr/raw-ptr-*`、`industrial/x509-parser_*`、`generic/array-len`、`prusti-limit/const-generics`、等。

stdout 错误模板分布：

```
12 [Error] Improperly typed constant value             (float / 含 float const)
 6 [Error] Internal error, please file an issue        (HOL4 backend 能力边界)
 2 [Error] Constant generics ... not supported for HOL4 (HOL4-only guard)
 2 [Error] Constant generics and type definitions with trait clauses ... HOL4
 2 [Error] Invalid inputs for binop
 1 [Error] Found type error in the output of charon    (async-fn)
 1 [Error] Dynamic trait types are not supported yet
 1 [Error] Arrow types are not supported yet
 1 [Error] Invalid input for unop: wrap.- on f64
 1 [Error] unions are not supported
 1 [Error] Unsupported operation: &raw mut (...)
 1 [Error] Unsupported operation: &raw const (...)
 1 [Error] Invalid inputs for unsized cast
 1 [Error] Assertion failed: new value doesn't have the same type ...
 1 [Error] Charon failed to compile constant: `ConstantExprKind::Cast {{..}}`
 ... (剩余分布散尾)
```

**归因**：工具不支持。两类细分：
1. aeneas mid-end 共享拒绝（float / unsized cast / raw-ptr / union / async-fn 等）——同 entry 在 fstar/coq/lean 多数同样 FAILED；
2. HOL4 backend 特定 guard（`Constant generics ... HOL4` 共 4 条 + `Internal error` HOL4-only 数条）——同 entry 在 fstar/coq/lean 多 SUCCESS。

**处理**：不修。partial file 已被 wrapper 按宪法 §六-2 翻译为 FAILED；本地性原则下 FAILED 站得住。

### 桶 D-charon：charon silent-partial gate 触发（8 case）

代表 entry：`box/shallow-init/shallow_init_box`、`charon-limit/box-branch-init/*`、`charon-limit/copy-deref-closure/*`、`charon-limit/inline-asm/*`、`closure-adv/boxed-dyn-fn/*`、`deps-complex/itertools-multi/*`、`industrial/sha2/sha256-digest/*`（2 条）。

stderr 特征：

```
warning: <X> is not supported
 --> ...
[aeneas-hol4-oracle] FAIL: charon exited 0 but emitted partial-signal stderr
```

典型不支持项：`Box` branching init、`&raw` 在 closure copy、inline asm、boxed dyn fn、associated-type 实现细节、`hybrid-array` 的 GAT 解算。

**归因**：工具不支持（charon 自陈 "is not supported"，本工具 wrapper 把"charon exit 0 + warn 自陈"翻译为 FAILED 避免静默降级——这是 wrapper 在执行宪法 §六"不冤枉、不藏"，不是 wrapper bug）。
**处理**：不修。

### 桶 C2：其他 OCaml panic（5 case）

包括：

- **`Failure "Can't convert type to pattern: dyn ..."`**（3 条）：`concurrency/thread-mutex/thread_mutex_join`、`lifetime/static-bound/static_bound`、`miri-limit/thread-interleaving-partial/*`
- **`Failure "Can't convert type to pattern: !"`**（2 条）：`creusot-limit/thread-local-ref/read_thread_local`、`lifetime/thread-local/thread_local_read`（栈在 `ctx_compute_trait_impl_name_aux`，源头是 `!` never type）

**归因**：工具不支持（aeneas type→pattern 转换器不覆盖 `dyn` 与 `!`）。
**处理**：不修。

### 桶 D-warn：aeneas Warn-channel partial gate 触发（1 case）

`gat/lending-iter/gat_lending` — aeneas exit 0 + Warn 自陈"model will not type-check"等。

**归因**：工具不支持（GAT lending iterator 在 aeneas 模型中未完整支持，aeneas 自陈生成代码"可能不正确"——wrapper 把它翻译为 FAILED 是执行宪法 §六-2 不允许 partial）。
**处理**：不修。

### 桶 A：charon-driver SIGABRT（1 case）

`trait/cyclic-bound/cyclic_bound_use`：charon-driver 在 cyclic trait bound 上递归触发 rustc stack overflow。

**归因**：工具不支持（charon / rustc 处理 cyclic bound 时崩溃，是工具能力边界）。
**处理**：不修。

## UNKNOWN 分类（按宪法 §六 严格语义）

2 个 UNKNOWN，均归 §六-(b)"我们这边可识别的问题且暂未修"：

| entry | UNKNOWN reason | 归因 |
|---|---|---|
| `industrial/x509-parser/cert-parse/x509_parse_der` | `external_fault: vendor_lint_strictness` | 我们 corpus 引入的 vendored crate (`vendor/x509-parser`) 含 `#![deny(unused_qualifications)]`，新版 rustc 触发该 lint 失败 → 我们的锅 |
| `industrial/x509-parser/cert-parse/x509_subject_extensions` | 同上 | 同上 |

runner `report.rs:92` 的 oracle 已识别此模式 (`unused_qualifications` + `vendor/` → "vendor_lint_strictness") → UNKNOWN 而非 FAILED。

## 失败 / 通过的 feature 类目分布（concise）

**整类通过**：`arc` / `const` / `drop` / `hello` / `int` / `panic` / `rc` / `refcell` / `vec` / `runnable`（15/15 v6 新加） — 共 10 类。

**部分通过**：`int-width` 13/14、`hax-limit` 4/8、`miri-limit` 3/7、`kani-limit` 3/7、`creusot-limit` 3/7、`bigint` 3/8、`unsafe-adv` 2/3、`generic` 2/4、`enum` 1/2、`box` 1/2、`concurrency` 1/2、`repr` 1/2、`lifetime` 1/3、`aeneas-limit` 1/8。

**整类零通过**：`assoc-type` / `closure` / `closure-adv` / `collections` / `deps-complex` / `error` / `float` / `gat` / `hrtb` / `impl-trait` / `industrial` / `iter` / `prusti-limit` / `slice` / `trait` / `trait-obj` / `unsafe-ptr` / `charon-limit`。注：上述零通过类目中含 trait declaration / closure / iterator / GAT / dyn 的 entry 多数被 C1 panic 截断；同 entry 在 fstar/coq/lean 上多有 SUCCESS。

## v5.1 → v6 ΔS 解释

- v5.1：n=146 / SUCCESS=51 / 通过率 34.9%
- v6：n=161 / SUCCESS=65 / 通过率 **40.4%**
- ΔS = +14；ΔN = +15

主要来源：v6 新增 `runnable/` 类目 15 entries 全部 SUCCESS（runnable 是用户提供的简单 lib 样例，未触 trait declaration / closure / GAT 路径）。其余 4 个 backend 共享的 mid-end 改动 + wrapper 双 gate 在本工具上不引入新 SUCCESS，因此通过率上升基本完全归因于 corpus 扩展。

## 修订建议清单（仅"我们导致"失败）

无"我们导致"的 FAILED。

2 条 UNKNOWN（`industrial/x509-parser_cert-parse/*` 因 `vendor_lint_strictness`）属"我们这边可识别问题"，但 runner oracle 已正确将其归为 UNKNOWN（§六-(b)），未冤枉工具能力边界。

| # | 桶 | 涉及 case | 修复方案 | 优先级 |
|---|---|---|---|---|
| 1 | UNKNOWN: vendor_lint_strictness | 2 | 修补 `vendor/x509-parser` 移除 `#![deny(unused_qualifications)]`，或在 corpus 层为该 vendored crate 提供 `RUSTFLAGS=-A unused_qualifications` 覆写；当前已 UNKNOWN 不冤枉工具，可作为 corpus backlog 处理 | 中（不影响工具能力评估，影响 corpus 完整性）|

**所有 FAILED 均为工具能力边界**——本地性原则下站得住，工具开发者不能驳回。aeneas-hol4 的 40.4% 通过率（vs aeneas-fstar/coq/lean 的 ~60%+）是 aeneas commit `a14083a6` 时点 + HOL4 backend `Extract.ml` 当前实现的真实事实陈述。aeneas 上游若修补 `extract_trait_decl Option.get` 路径，本快照差距将随之消解；读者引用本数字时务必同时引用 run id 与 aeneas commit hash。
