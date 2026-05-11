# verifast 深度报告

## 元数据

- **run**: `run-1778238662-69805`（2026-05-08T11:11:02Z → 11:13:01Z UTC，**P12-B 重跑**：3 工具 × 146 entries，host：Apple M5 / macOS aarch64 / 24 GB / 10 cpu / parallelism 10）
- **封堵前对照 run**: `run-1778226613-5282`（2026-05-08T07:50:13Z → 08:16:08Z UTC，旧 oracle，下文称"旧 run"）
- **工具版本**：`VeriFast 26.01 (released 2026-01-21) for C and Java`，prover 列表含 `Z3v4.5` / `Redux`，原生 macOS arm64 prebuilt（与旧 run 同 binary）
- **通过率**：12/146 = **8.2%**（旧 run 116/146 = 79.5%，**delta = -71pp**；FAILED 134 / 134，TIMEOUT 0，UNKNOWN 0）
- **退出码分布（新 oracle）**：exit 0 = 12（透传 SUCCESS）/ exit 1 = 30（verifast 自身 reject，与旧 run 一致）/ exit 2 = 104（wrapper 重写：exit 0 但 verbose 输出 0 命中用户文件 → 翻 FAILED）
- **时长（ms）**：avg 140 / median 140 / p90 163 / max 362（vs 旧 run avg 282 / max 1479——变快是因为 134/146 任务直接走 wrapper reject path 不再走完 IR 构造；剩 12 个 SUCCESS 时长 134–172 ms 与旧 run 同 entry 时长一致）
- **时效声明**：本快照锚定 VeriFast 26.01 + 上述两个 run id + corpus，不构成长期承诺。

## 工具内部 pipeline + 前端边界

VeriFast 不经过 cargo / rustc，直接读 `src/lib.rs` 跑自家 pipeline：

```
verifast binary → 内嵌 rustc-style Rust parser
              → MIR → VeriFast IR（含 separation-logic share predicate 等结构）
              → 符号执行（按 //@ req/ens/inv/pred 注解驱动）
              → SMT prover (Z3v4.5 / Redux)
```

本测试 `tool.toml` 的 `command` 指向 `tools/verifast/verifast-strict-wrapper.sh`，wrapper 内部用以下 argv 调 verifast：

```
verifast -verbose 1 -target macOS -shared -skip_specless_fns
         -ignore_unwind_paths -disable_overflow_check
         -read_options_from_source_file src/lib.rs
```

`-skip_specless_fns` 仍是**前端 dry-run 等价 flag**：无 `//@ req/ens` 注解的函数全跳过符号执行。`-verbose 1` 是 P12-A 新加的：让 verifast 把每一步 symex 动作打成 `<secs>: <source-path>(LINE,COL-LINE,COL): <action>` 行，wrapper 按 `src/lib.rs(` grep 用户文件命中数判 vacuous pass。`entry_mode` 默认 `bin`：runner 写 harness 到 `src/bin/__ts_harness.rs`，但 verifast 命令只读 `src/lib.rs` —— harness 被忽略。

## SUCCESS 信号 + 形式严格性（P12-A 改造后）

P12-A 把判据从"裸 exit 0"升级为 wrapper post-check（实施细节见 `docs/fixes/oracle-leak-rules-implementation-2026-05-08.md` §2.1）：

```
SUCCESS ⟺ verifast exit 0  ∧  verbose 输出含 ≥1 行 'src/lib.rs(' 的 source-path tag
```

- exit ≠ 0：透传（真 reject，FAILED）
- exit 0 + 0 用户文件 verbose 行：wrapper 重写为 exit 2 + stderr 诊断 → FAILED（vacuous pass）
- exit 0 + ≥ 1 用户文件 verbose 行：透传 → SUCCESS（user code 在 IR 里被 symex 触及）

**形式严格性**：

- **0 误报**：✅ 形式可证。新规则的 reject 条件（exit 0 ∧ 0 user-file mention）对真 SUCCESS **构造性不可达**——理由（详 implementation log §2.1 反误报论证 + `tools/verifast/verifast-strict-wrapper.sh` 头部注释）：spec-bearing fn 必有至少一个 `//@ req`，一旦未跳过，verifast 必走 prototype-implementation-check + symex 路径，每条都打 `<source-path>(LINE,COL):` 标签到 verbose 输出。"有 spec → 必有 user-file mention"由 verifast 设计强制保证。`oracle-validation/spec_bearing_add_one.rs` 实测：spec-bearing 最小例 → exit 0，10 行 user-file 命中 → wrapper 透传 ✓。
- **0 漏报**：✅ 实测验证升级。审计推荐的"N ≤ 40 statements"阈值在 spec-bearing 最小例上 N=39 落入 spec-less 区间会误报；改用 verbose user-file grep 双向均通过 micro-test。`oracle-validation/spec_less_baseline.rs` 实测：spec-less → exit 0，0 行 user-file 命中 → wrapper 重写 exit 2 ✓。
- **漏报盲点**：spec-bearing entry 中 verifast 内部某条 path 上 silent skip user fn 而仍 exit 0 + 写 verbose user-file 行 —— 理论窗口；当前 corpus 0 spec entry 故未现身。规则在 spec corpus 上行为待实测。

## 关键发现：P12-B 实测证实先前 79.5% 是空过

旧报告（`run-1778226613-5282`，旧 oracle 仅看 exit code）声称 verifast 通过率 79.5%，但同时**强声明** SUCCESS 实际语义降级为"rustc-verifast 接受 IR + 无 spec 可证伪"（vacuous pass）—— corpus 全集 grep `//@\s*(req|ens|inv|pred)` 0 命中。彼时只能基于"104/116 SUCCESS 报同一 baseline `37 statements verified`"间接论证空过性质，**无法把那部分 entry 直接翻为 FAILED**——因为 N 阈值在 spec-bearing 最小例上会误报（详 implementation log §2.1）。

P12-A 用 verbose user-file grep 替换 N 阈值，P12-B 重跑结果**直接落锤**：

- 旧 116 SUCCESS 中 **104 条**被 wrapper 翻为 exit 2（vacuous pass）
- 剩 12 条仍 SUCCESS（exit 0 + 用户 verbose 命中，22–81 行）

之前的"79% 是空过率"假说从间接论证升级为**实测结论**：原 116 SUCCESS 中 71pp 是空过，封堵后跌到 8.2%。

## 实测结果

### 12 个残留 SUCCESS — 仍非真 spec 验证

12 条 entry（实读 stdout 抓 `'src/lib.rs(' 计数 / 末尾 N statements`）：

| entry | user-file 命中 | N statements | 时长 ms |
| --- | ---: | ---: | ---: |
| `aeneas-limit/mutually-recursive-traits/trigger_mutually_recursive_traits` | 44 | 39 | 170 |
| `aeneas-limit/nested-borrow-array/trigger_nested_borrow_array` | 37 | 38 | 144 |
| `aeneas-limit/trait-impl-mut-param-mismatch/trigger_trait_impl_mut_param_mismatch` | 22 | 38 | 144 |
| `assoc-type/iter-style/assoc_type_iter` | 22 | 38 | 157 |
| `drop/custom-drop/custom_drop_order` | 29 | 38 | 140 |
| `gat/lending-iter/gat_lending` | 22 | 38 | 155 |
| `generic/pair-struct/generic_pair` | 37 | 38 | 147 |
| `prusti-limit/shallow-borrow-match-guard/trigger_shallow_borrow_match_guard` | 22 | 38 | 139 |
| `repr/c-struct/repr_c_struct` | 22 | 38 | 134 |
| `trait/cyclic-bound/cyclic_bound_use` | 81 | 40 | 147 |
| `trait-obj/conditional-method/conditional_method` | 59 | 39 | 137 |
| `trait-obj/dyn-dispatch/dyn_dispatch` | 44 | 39 | 172 |

**这 12 条 entry 也都不写用户 spec**（`grep -cE '//@\s*(req|ens|inv|pred)' src/lib.rs` = 0 全部）。它们与 vacuous-pass 那 104 条的差别在于：**用户声明了 struct / trait / enum 类型**——verifast 自动为每个用户类型生成 `is_Send_<T>_def` / `is_Sync_<T>_def` / `init_ref_padding_<T>` / `close_ref_initialized_<T>` / `open_ref_init_perm_<T>` 等结构性谓词，每条谓词都打 `src/lib.rs(LINE,COL)` 源文件 tag —— wrapper grep 命中。

实例（`repr/c-struct`）：用户声明 `struct Point { x: i32, y: i32, z: i32 }`，verifast 自动生成并 verify 7 条结构谓词：

```
0.052546s: src/lib.rs(4,5-44): Verifying function 'repr_c_struct::is_Send_Point_def'
0.052569s: src/lib.rs(4,5-44): Executing statement
0.052609s: src/lib.rs(4,5-44): Verifying function 'repr_c_struct::is_Sync_Point_def'
0.052655s: src/lib.rs(4,5-44): Verifying function 'repr_c_struct::open_ref_init_perm_Point'
...
0 errors found (38 statements verified)
```

所以这 12 条 SUCCESS **仍非"用户 spec 验证完成"**——是"verifast 对用户声明的类型**自动生成**的结构谓词验证完成"。判据上达到 wrapper 设定的边界（symex 触及用户文件），但 corpus 0 spec 这一根本约束没变——本工具在本矩阵上 SUCCESS 的语义仍降级。

要测 verifast 真正能力，需要给 entry 加 `//@ req/ens` 注解（违反 entry 自包含原则）或不加 `-skip_specless_fns`（plain Rust 会因隐式溢出 / panic 路径触发误判）。**8.2% 是去除最严重空过类后的接受面，不是验证通过率。**

### 134 个 FAILED 分桶

按 wrapper / verifast 输出形态分两层：

**层 1：wrapper 重写 exit 2（104 个，vacuous pass）**——verifast 自身 exit 0 但 verbose 输出 0 行命中 `src/lib.rs(`。stderr 诊断模板：

```
[verifast-oracle] FAIL: vacuous pass — symex executed 0 statements in
[verifast-oracle]       src/lib.rs. `-skip_specless_fns` is on and this
[verifast-oracle]       entry has no `//@ req/ens/inv/pred` annotation, so
[verifast-oracle]       verifast verified only its prelude (in
[verifast-oracle]       <install>/bin/rust/rust_belt/*.rsspec) and exited.
```

stdout 末行总是 `0 errors found (37 statements verified) (target: arm64-apple-macosx (LP64))`——37 是 verifast 自家 prelude (`lem_aux.rsspec`) 的 baseline。这 104 条覆盖整个 `float/* (10/10 翻 FAILED)` / `int-width/* (14/14)` / `int/* (2/2)` / `closure/* (2/2)` / `closure-adv (4/4)` / `arc / rc / refcell / vec / hello / box / unsafe-ptr / unsafe-adv / panic / generic` 等普通无类型声明的 entry——这些就是旧 run 116 SUCCESS 里的"104 个报 baseline 37"那部分。

**层 2：verifast 自身 exit 1（30 个，与旧 run 同集合）**——按 verifast 输出形态归 5 子类（与旧报告一致，不重述细节）：

- A 不解析 cargo 依赖（21）：`bigint/*` 8 + `deps-complex/*` 7 + `industrial/*` 6（`unresolved import`）
- B 自家 parser 拒绝 float 在复合类型中（3）：`Floating point types are not yet supported`（aeneas-limit/float-types、enum/data-variants、repr/union）
- C const generics 结构定义不支持（2）：`Structs with const parameters are not yet supported`（charon-limit/precise-drops-const-generic、prusti-limit/const-generics）
- D shared ownership of `&[_]` 字段不支持（1）：`prusti-limit/ref-typed-struct-field`
- E edition 边界（3）：async fn / let-chains 在 Rust 2015 拒（charon-limit/async-fn、kani-limit/async-await、hax-limit/let-chains）

合计 104（vacuous）+ 21 + 3 + 2 + 1 + 3 = 134，与 results.json 一致。

### 工业三件套

| entry | 旧 run（exit code） | 新 run（exit code） | 备注 |
| --- | --- | --- | --- |
| `industrial/rsa/rsa-pkcs8/rsa_pkcs1v15_encrypt` | FAILED (1) | FAILED (1) | unresolved import 'rsa'（A 桶） |
| `industrial/rsa/rsa-pkcs8/rsa_pubkey_from_pkcs8` | FAILED (1) | FAILED (1) | 同上 |
| `industrial/sha2/sha256-digest/sha256_digest_incremental` | FAILED (1) | FAILED (1) | unresolved import 'sha2' |
| `industrial/sha2/sha256-digest/sha256_digest_one_shot` | FAILED (1) | FAILED (1) | 同上 |
| `industrial/x509-parser/cert-parse/x509_parse_der` | FAILED (1) | FAILED (1) | unresolved import 'x509_parser' |
| `industrial/x509-parser/cert-parse/x509_subject_extensions` | FAILED (1) | FAILED (1) | 同上 |

verifast 在工业三件套上**两次 run 均 0/6**，全部死在 verifast 内嵌 rustc parser 的 import resolution 阶段——单文件输入模式不读 `Cargo.toml`，与封堵无关（这 6 条本来就 verifast exit 1，不进入 vacuous-pass 判据路径）。

## 与本次测试边界的关系

**vacuous pass 的实测落锤**：本 corpus 0 spec 注解的事实未变；旧 oracle 把"verifast 接受 IR + 无 spec 可证伪"判 SUCCESS，新 oracle 把"verbose 输出 0 命中用户文件"那部分翻 FAILED。新 8.2% 是"用户至少声明了 struct / trait / enum 让 verifast 自动生成结构谓词"的接受面。仍**不**等于"用户 spec 验证完成"——后者在本 corpus 上 0 个 entry 触发。

**新 oracle 与宪法 §六-2 / §六-4 对齐**：旧"79.5%"符合"工具自报 SUCCESS"但违反"反作弊 + 不允许 partial"——symex 在用户代码上跑 0 statement 的 SUCCESS 是工具语义降级。新 oracle 把这个降级翻进 oracle 形式语义，是 P11 → P12 oracle 漏报封堵规划的最终落地。详细论证见 `docs/fixes/oracle-leak-rules-implementation-2026-05-08.md` §2.1。

**子毫秒到亚秒级响应**（new avg 140ms，134/146 任务在 wrapper grep 阶段就 reject 不再做完整 IR 构造）反映：本 corpus 上 verifast 的 prover 几乎不被使用——这也是空过性质的间接物理证据。

## 历史快照声明

本报告所有数字与归类锚定 VeriFast 26.01 / 2026-01-21 prebuilt + macOS arm64 + wrapper-strict-oracle-v2（基于 `verifast-strict-wrapper.sh` 的 verbose user-file grep）+ 上述 6 件 verifast flag 组合。版本升级、edition 默认值变化、share predicate 自动生成规则修订、verbose 输出格式变更都可能改写本归类。
