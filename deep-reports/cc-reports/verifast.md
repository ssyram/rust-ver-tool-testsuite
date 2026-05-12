# verifast — 特性支持评估报告（v6 baseline）

## 元数据

- **数据源**：`runs/run-1778560393-59119/`（2026-05-12 v6 final，合并 verus rerun + R7 5-tool rerun）
- **工具配置**：`tools/verifast/`（`tool.toml` + `verifast-strict-wrapper.sh`，wrapper-strict-oracle-v2 = exit 0 ∧ `-verbose 1` 输出含 ≥1 行 `src/lib.rs(` 锚）
- **工具版本**：`VeriFast 26.01 (released 2026-01-21) for C and Java`，prover `Z3v4.5 / Redux`，macOS arm64 prebuilt（`.tmp/agents-staging/tool-verifast/install/verifast-26.01/bin/verifast`）
- **本工具实测**：n=161 / SUCCESS=13 / FAILED=148 / UNKNOWN=0，通过率 **8.1%**
- **退出码分布**：exit 0 = 13（wrapper anti-cheat gate 合法透传 SUCCESS）/ exit 1 = 30（verifast 自身 reject）/ exit 2 = 118（同一 gate 合法拦截 vacuous pass → FAILED）
- **关键语义**：corpus 0 entry 含 `//@ req/ens/inv/pred` 注解 → verifast 不真做用户 spec verification。13 SUCCESS 与 118 vacuous-pass FAILED 是**同一 wrapper anti-cheat gate 的两侧**——前者因 entry 声明用户类型触发自动生成结构谓词（`is_Send_<T>_def` 等）使 symex 命中 `src/lib.rs(`；后者因 entry 无用户类型 / 无 spec / `-skip_specless_fns` 跳净所有 user fn → 0 命中 → 合法拦截。
- **时长分布**：avg 298ms / median 251ms / p90 520ms / max 818ms
- **宪法 baseline**：`principles.md` v8（P27 修宪后 UNKNOWN 严格语义 + 双根本问题 + §六 不允许 partial + 反作弊）
- **时效声明**：本快照锚定 VeriFast 26.01 / 2026-01-21 prebuilt + macOS arm64 + wrapper-strict-oracle-v2 + 上述 6 件 verifast flag 组合 + 本 run id + corpus，不构成长期承诺。

## pipeline + 前端边界

VeriFast 不走 cargo / rustc，直接读 `src/lib.rs` 跑自家 pipeline：

```
verifast binary → 内嵌 rustc-style Rust parser
              → MIR → VeriFast IR（含 separation-logic share predicate）
              → 符号执行（按 //@ req/ens/inv/pred 注解驱动）
              → SMT prover (Z3v4.5 / Redux)
```

本测试 `tool.toml` 的 `command` 走 `tools/verifast/verifast-strict-wrapper.sh`（**项目维护的脚本**，不是 verifast 官方 wrapper）。wrapper 内部以下 argv 调 verifast：

```
verifast -verbose 1 -target macOS -shared -skip_specless_fns
         -ignore_unwind_paths -disable_overflow_check
         -read_options_from_source_file src/lib.rs
```

`-verbose 1` 是 P12-A 加的，让 verifast 把每一步 symex 动作打成 `<secs>: <source-path>(LINE,COL-LINE,COL): <action>` 行；wrapper 按 `src/lib.rs(` grep 用户文件命中数判 vacuous pass。`entry_mode` 默认 `bin`：runner 写 harness 到 `src/bin/__ts_harness.rs`，但 verifast 命令只读 `src/lib.rs` —— harness 被忽略。

**前端测量边界**：verifast 自带 rustc parser + MIR / IR 构造 + symex（按 §六 "前端测量"）。SMT prover 层不计入测量结论，仅作为 oracle 信号源。

## SUCCESS 信号 + 形式严格性

按 §六 不允许 partial 双通路 partial 暴露：

- **主信号通路**：verifast 自身 exit code。
  - exit 0 = 验证完成无 error。
  - exit 1 = 验证 error / parser reject / IR 构造拒（async / closure / float-in-composite / const-generic 等 → 走"工具不支持"语义）。
- **wrapper 补抓通路**（项目自维护）：exit 0 后再 grep `-verbose 1` 输出中含 `src/lib.rs(` 行的数量。
  - ≥ 1 行 → 透传 SUCCESS（symex 真在用户代码上跑了 statement）
  - 0 行 → wrapper 重写为 exit 2 + stderr 诊断 → FAILED（vacuous pass：`-skip_specless_fns` 跳掉所有 user fn，仅 prelude 被 verify）

**形式严格性 — 0 误报（不冤枉能力）**：✅ 形式可证 + 双向实测。新规则的 reject 条件（exit 0 ∧ 0 user-file mention）对真 SUCCESS **构造性不可达**——spec-bearing fn 必有 `//@ req`，一旦未被 `-skip_specless_fns` 跳掉，verifast 必走 prototype-implementation-check + per-statement symex，每条都打 `<source-path>(LINE,COL):` 标签。`oracle-validation/spec_bearing_add_one.rs` 实测：spec-bearing 最小例 exit 0 / 10 行 user-file 命中 → wrapper 透传 ✓。

**形式严格性 — 0 漏报（不高估能力）**：✅ 实测 + 设计论证。`-skip_specless_fns` 让 verifast 在 user fn 上 0 verbose 行；`oracle-validation/spec_less_baseline.rs` 实测：spec-less → exit 0 / 0 行 → wrapper 重写 exit 2 ✓。

**已知漏报盲点**（按宪法 §六 "不藏"诚实声明）：

- 现 13 SUCCESS 全部**仅触发自动生成的结构谓词**（`is_Send_<T>_def` / `is_Sync_<T>_def` / `open_ref_init_perm_<T>` 等），corpus 0 个 entry 含 `//@ req/ens/inv/pred` 注解。"symex 触及 user file"是设计上能给出的形式 SUCCESS 边界，但**不等价于"用户 spec 验证完成"** —— 这是本 corpus 设计选择决定的语义降级，不是 wrapper bug。
- 理论窗口：spec-bearing entry 中 verifast 内部某条 path 上 silent skip user fn 而仍 exit 0 + 写 verbose user-file 行。当前 corpus 0 spec entry 故未现身；规则在 spec corpus 上行为待实测。

## 失败分桶（按 P31 §四.5 归因分类）

148 FAILED 分两层 7 桶。**所有桶归因均为"工具不支持"或"工具能力边界"（含 wrapper anti-cheat gate 合法拦截 vacuous pass 这个设计意图）**——本工具在本 corpus 上无"我们 wrapper bug"或"我们 corpus bug"导致的失败。注意：13 SUCCESS 与桶 1 的 118 FAILED 共享同一形式严格性机制，前者是 gate 的合法穿透（auto-generated struct predicates 触及 user file），后者是 gate 的合法拦截（0 user-file 命中）。

### 桶 1：wrapper 拒 vacuous pass（118 case）

代表 entry：`aeneas-limit/bool-bitwise-op/trigger_bool_bitwise_op` / 整套 `float/*` (10) / `int-width/*` (14) / `int/*` (2) / `closure/*` / `closure-adv/*` / `arc / rc / refcell / vec / hello / box / unsafe-ptr / unsafe-adv / panic` 等普通无类型声明的 entry。

stdout 尾部特征：

```
0 errors found (37 statements verified) (target: arm64-apple-macosx (LP64))
```

stderr 特征：

```
[verifast-oracle] FAIL: vacuous pass — symex executed 0 statements in
[verifast-oracle]       src/lib.rs. `-skip_specless_fns` is on and this
[verifast-oracle]       entry has no `//@ req/ens/inv/pred` annotation, so
[verifast-oracle]       verifast verified only its prelude (...)
```

**归因**：**工具能力边界 + wrapper 设计意图**。verifast 自身 exit 0（因 `-skip_specless_fns` 跳掉所有 user fn），wrapper 按宪法 §六 "不允许 partial / silent skip / 半翻译" 把 vacuous pass 重写为 FAILED。这**不是** "我们 wrapper bug"——是 wrapper 按宪法精神**正确**封堵 verifast 在本 corpus 上的语义降级。
**处理**：**不修**。本地性 + 不允许 partial 原则下 FAILED 站得住。要让此类 entry 转 SUCCESS 必须给 entry 加 `//@ req/ens` 注解（违反 entry 自包含 / §四 原则 A 双方不可侵入），或换工具评估 verifier-deep 部分（如 hax / kani）。

### 桶 2：单文件 pipeline 不读 Cargo.toml deps（21 case）

代表 entry：`bigint/*` (8) / `deps-complex/*` (7) / `industrial/rsa-pkcs8` (2) / `industrial/sha2` (2) / `industrial/x509-parser` (2)。

stdout 特征：

```
error[E0432]: unresolved import `rsa` / `sha2` / `num_bigint` / `chrono` ...
```

**归因**：**工具能力边界**。verifast 单文件输入模式直接读 `src/lib.rs`，不经 cargo 链接、不解析 `Cargo.toml`——任何 `use <external-crate>` 都报 unresolved import。这是 verifast 自陈的 pipeline 设计（README §"无跨 crate 调用"）。
**处理**：**不修**。FAILED 站得住。

### 桶 3：Floating point in composite types not supported（3 case）

代表 entry：`aeneas-limit/float-types/make_measurement` / `enum/data-variants/enum_match_data` / `repr/union/repr_union`。

stdout 特征：

```
Floating point types are not yet supported
```

**归因**：**工具能力边界**。verifast 26.01 Rust frontend 拒在 struct / enum / union 字段中的 `f32` / `f64`（顶层 fn 签名中可接受）。
**处理**：**不修**。

### 桶 4：Structs with const parameters not supported（2 case）

代表 entry：`charon-limit/precise-drops-const-generic/consume_portable_hash` / `prusti-limit/const-generics/entry_const_generic_buffer`。

stdout 特征：

```
Structs with const parameters are not yet supported
```

**归因**：**工具能力边界**。verifast 26.01 不支持 const generics 结构定义。
**处理**：**不修**。

### 桶 5：Shared ownership of `&[_]` field not supported（1 case）

代表 entry：`prusti-limit/ref-typed-struct-field/trigger_ref_typed_struct_field`。

stdout 特征：

```
Error while building default .share predicate conjunct for field data:
Expressing shared ownership of &[_] values is not yet supported
```

**归因**：**工具能力边界**。verifast separation-logic share-predicate 生成器目前不支持包含 `&[_]` 字段的 struct。
**处理**：**不修**。

### 桶 6：Edition 边界（async fn / let-chains）（3 case）

代表 entry：`charon-limit/async-fn/async_forty_two` / `kani-limit/async-await/run_async_add` / `hax-limit/let-chains/hax_limit_let_chains`。

stdout 特征：verifast 内嵌 rustc parser 在默认 edition（Rust 2015）下拒 async fn / let chains。

**归因**：**工具能力边界**。verifast 不读 `Cargo.toml` 故拿不到 `edition = "2021"` 设置；当前 wrapper 也未传 `--edition`（verifast 不提供该 flag）。
**处理**：**不修**。FAILED 站得住——按本地性原则，verifast 自身 pipeline 设计就是单文件 + 不读 cargo metadata。

### 桶 7：X509 parser deps 死 + framework alloc shim 引发 rustc 类型推断错（嵌在桶 2 内）

`industrial/x509-parser_cert-parse/*` 2 case stdout 同时包含 `cannot infer type` + `function cannot return without recursing`（针对 verifast 自陈的 `VeriFast_alloc<T>() -> *mut T { VeriFast_alloc() }` 类型推断 shim）。但这两 case 已被桶 2 unresolved-import 主因覆盖（top-level 是 `unresolved import 'x509_parser'`），归 verifast 单文件 pipeline 不读 deps 同因。

## 漏报盲点（诚实声明）

- **已通过 wrapper gate 封堵**：vacuous pass（`-skip_specless_fns` silent-skip 所有 user fn 仍 exit 0）—— 由 `verifast-strict-wrapper.sh` 用 `-verbose 1` + `src/lib.rs(` 命中计数封堵，2026-05-08 落地，本 run 实测拦 118/161。
- **仍存在的盲点**：
  - 当前 corpus 0 spec entry，故 13 SUCCESS 全部仅触发自动生成结构谓词（`is_Send_<T>_def` 类），不等于"用户 spec 验证完成"。这是 corpus 设计选择，**不计入工具能力高估**——前端测量边界已守住（symex 真触及 user code）。
  - 理论窗口：spec-bearing entry 内部某条 path 上 silent skip user fn 而仍 exit 0 + 写 verbose user-file 行。当前 corpus 无 spec 故未现身。修复 backlog：未来若 corpus 引入 `//@ req/ens` 注解，需在 wrapper 加路径级 silent-skip detection（grep verbose 输出中的 "Skipping" 标签）。

## v5.1 → v6 ΔS 解释

v5.1（`run-1778238662-69805`，146 entries）：12 SUCCESS / 8.2%
v6（`run-1778560393-59119`，161 entries）：13 SUCCESS / 8.1%

**ΔS = +1**：新增 entry `runnable/struct-norm/manhattan_of` 进 SUCCESS（user-file 命中 22 行）。该 entry 声明 `pub struct Point`，与既有 12 SUCCESS 同模式——verifast 自动为用户类型生成结构谓词时打 source-path tag → wrapper 透传。语义与 v5.1 一致：仍非"用户 spec 验证"，是"用户至少声明了 struct / trait / enum 触发自动谓词"的接受面。新增 14 个 entry 全部进 FAILED（vacuous pass / unresolved import / 工具能力边界），分布同既有桶，未引入新失败模式。

通过率从 8.2% 降到 8.1% 仅因分母从 146 涨到 161（绝对 SUCCESS 数 +1，分母 +15）。

## 修订建议清单（仅"我们导致"失败）

**无需修订**。本工具 148 FAILED 全部归"工具能力边界" + 1 桶"wrapper 按宪法设计意图拒 vacuous pass"。**没有任何"我们 wrapper bug" / "我们 corpus bug" / "环境损坏"类失败需要修。**

特别说明：桶 1（118 vacuous pass）虽然由 `verifast-strict-wrapper.sh`（项目自维护脚本）翻为 FAILED，但 wrapper 行为是宪法 §六 "不允许 partial / silent skip / 半翻译" + §六 反作弊原则的**正确**实现，不是 wrapper bug。要让此类 entry 转 SUCCESS 必须让 entry 写 `//@ req/ens` 注解——违反 §四 原则 A（"样例源码不为工具改动"）+ entry 自包含原则。**保留 FAILED 是宪法精神。**
