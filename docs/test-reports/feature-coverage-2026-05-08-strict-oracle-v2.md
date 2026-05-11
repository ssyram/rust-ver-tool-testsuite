# 19 工具特性覆盖度综合报告（2026-05-08，strict-oracle-v2）

> 本报告综合 19 份 cc-report，基于 P12-A oracle 漏报封堵后的实测数据：3 工具（verifast / prusti / rocq-of-rust）数字来自 `runs/run-1778238662-69805` 的 P12-B 重跑，其余 16 工具数字来自 `runs/run-1778226613-5282` 的全矩阵 run。
>
> 与上一份报告 [`feature-coverage-2026-05-08-19tools-strict-oracle.md`](./feature-coverage-2026-05-08-19tools-strict-oracle.md)（"strict-oracle-v1"）的差异：v1 在 verifast / prusti / rocq-of-rust 三个工具上的 oracle 对部分 silent path 仍欠缺封堵，v2 把这些封堵规则落地为代码（实施记录见 [`docs/fixes/oracle-leak-rules-implementation-2026-05-08.md`](../fixes/oracle-leak-rules-implementation-2026-05-08.md)）后重跑，verifast 通过率从 79.5% 跌到 **8.2%**（-71pp），rocq-of-rust 从 82.9% 跌到 **76.0%**（-7pp），prusti 56/146 数字不变（防御性改造，理论窗口封堵）。

---

## 一、元数据（dual-run）

### 数据来源切割

| 工具组 | 来源 run | run id | 时间窗 | 备注 |
| --- | --- | --- | --- | --- |
| 16 工具（除 verifast / prusti / rocq-of-rust） | 主 run | `run-1778226613-5282` | 2026-05-08T07:50:13Z – 08:16:08Z UTC | 全矩阵 19×146 |
| **verifast / prusti / rocq-of-rust** | **P12-B 重跑** | **`run-1778238662-69805`** | **2026-05-08T11:11:02Z – 11:13:01Z UTC**（119s wall） | **strict-oracle-v2 后 3×146 重跑** |

两次 run 同一 host（Apple M5 / macOS aarch64 / 24 GB / 10 cpu / parallelism 10），同一 corpus（146 entries），同一工具版本 binary——只 oracle 改造（参见 [`docs/fixes/oracle-leak-rules-implementation-2026-05-08.md`](../fixes/oracle-leak-rules-implementation-2026-05-08.md) §1）。两次 run 数字可直接拼接为本报告 19 工具行。

### 总体数字

- **总 task**：19 工具 × 146 entries = **2774 task**
- **结果分布（v2）**：SUCCESS 1835 / FAILED 939 / **UNKNOWN 0** / **TIMEOUT 0**（v1 SUCCESS 1949 / FAILED 825——v2 多翻 114 个 FAILED：verifast +104（116→12），rocq-of-rust +10（121→111），prusti 0；合计 -114 SUCCESS / +114 FAILED，符合 P12-A 改造预期）
- **runner 健康**：两次跑期间 0 panic / 0 internal timeout / 0 work 残留

### 19 工具版本快照

| tool | version |
|---|---|
| cargo-check | `cargo 1.95.0 (f2d3ce0bd 2026-03-21)` |
| miri | `miri 0.1.0 (cb40c25f6a 2026-05-04)` @ nightly |
| kani | `cargo-kani 0.67.0` |
| charon-mono / charon-poly | `charon 0.1.184` (toolchain `nightly-2026-02-07`) |
| creusot | `cargo-creusot 0.11.0` · `nightly-2026-02-27` · Why3 1.8.2 · alt-ergo 2.6.2 · z3 4.15.4 · cvc4 1.8 · cvc5 1.3.1 |
| hax-coq / hax-fstar / hax-lean | hax `untagged-git-rev-30949eb870` (commit `30949eb87058895c24f963df90dd30ef11b0dc1a`) @ `nightly-2025-11-08` |
| aeneas-coq / aeneas-fstar / aeneas-hol4 / aeneas-lean | aeneas `a14083a6` + 自家 charon `0.1.184` (commit `ed22146b`) |
| prusti | Prusti 0.2.2 · commit `a0681ee` (2023-08-22) · `nightly-2023-08-15` · arch -x86_64 · JDK 17 · **strict-oracle-v2: + .vpr 存在性 check（防御层）** |
| verus | `Verus 0.2026.05.03.8b81855` · profile release · platform `macos_aarch64` · toolchain `1.95.0-aarch64-apple-darwin` |
| verifast | `VeriFast 26.01 (released 2026-01-21)` · prebuilt macOS arm64 · provers Z3v4.5 / Redux · **strict-oracle-v2: -verbose 1 + verbose user-file grep（vacuous-pass 封堵）** |
| soteria | soteria-rust commit `3c212781` · Obol commit `ddea5ca5` · OCaml 5.4.0 |
| kmir | mir-semantics commit `84bea09` · stable-mir-json commit `62a239d7` · K Framework v7.1.282 · `nightly-2024-11-29` · kmir Python 0.3.181 |
| rocq-of-rust | `rocq_of_rust_cli 0.1.0` · commit `a8a76a4d` · `nightly-2024-12-07` · **strict-oracle-v2: 6 道门（追加 entry_fn `Definition` 存在性）** |

### 时效性声明

按宪法 §三-3-1：本报告锚定 2026-05-08 上述具体工具版本组合 + strict-oracle-v2 + 146-entry corpus 的实测快照。**报告不构成项目对任何工具长期能力的承诺**。任一工具升级、上游 oracle 漂移、corpus 扩充都会让本快照解释力线性衰减。读者引用本报告任何数字时务必同时引用主 run id (`run-1778226613-5282`) + P12-B run id (`run-1778238662-69805`) + 对应工具版本字符串。

### 与 strict-oracle-v1 的关系

| 报告 | run | oracle 状态 | 主要差异 |
|---|---|---|---|
| `feature-coverage-2026-05-08-19tools-strict-oracle.md` | `run-1778226613-5282` | strict-oracle-v1（旧） | verifast 79.5% / rocq-of-rust 82.9%，对 verifast vacuous-pass 与 rocq-of-rust silent skip-item 仅在 README 文字层声明，未在 oracle 落地 |
| **本报告（v2）** | **`run-1778226613-5282` + `run-1778238662-69805`** | **strict-oracle-v2（新）** | **verifast 8.2% / rocq-of-rust 76.0%；prusti 56/146 防御层兜底，数字不变** |

v1 与 v2 在 16 工具上数字一致；3 工具数字差由 oracle 改造造成（不是工具能力变化，是 oracle 对工具语义降级 / silent path 的更严格抓获）。

---

## 二、本次 run 的 oracle 改造

按宪法 §三-3-2.b/c"严格 0 误报 + 下限诚实"+ §六-2"不允许 partial"+ §六-4"反作弊"：oracle 必须形式覆盖工具 SUCCESS 信号语义边界。strict-oracle-v1 在 3 个工具上有缺漏（详 [`docs/fixes/oracle-leak-audit-2026-05-08.md`](../fixes/oracle-leak-audit-2026-05-08.md) §3）；v2 落地封堵规则（[`docs/fixes/oracle-leak-rules-implementation-2026-05-08.md`](../fixes/oracle-leak-rules-implementation-2026-05-08.md) §1）：

| 工具 | v1 → v2 oracle 差异 | 数字变化 | 反误报论证 |
| --- | --- | --- | --- |
| **verifast** | v1: 仅看 exit code（接受 vacuous pass）<br>v2: `verifast-strict-wrapper.sh` 用 `-verbose 1` + grep `src/lib.rs(` 命中数 ≥ 1（symex 触及用户文件） | **79.5% → 8.2%（-71pp）** | spec-bearing fn 必经 prototype-impl-check + symex，每条都打 source-path tag 到 verbose；reject 条件（exit 0 ∧ 0 user-file mention）对真 SUCCESS 构造性不可达。`oracle-validation/spec_bearing_add_one.rs` 实测 ✓ |
| **prusti** | v1: 仅看 cargo-prusti exit code<br>v2: `prusti-strict-wrapper.sh` 追加 ≥ 1 个 `target/verify/log/viper_program/*.vpr` 存在性 check | **38.4% → 38.4%（0pp）** | `PRUSTI_DUMP_VIPER_PROGRAM=true` 是 commit `a0681ee` 的 unconditional dump，encoder 完成后必写 .vpr 再短路。在当前 commit + config 下 reject 条件不可达，规则是 commit-drift 防御层 |
| **rocq-of-rust** | v1: 5 道门（exit 0 + .v 存在 + 非 0-byte + > 200 字节 + 5-marker grep）<br>v2: + 第 6 道门 `grep -rqE '^[[:space:]]*Definition[[:space:]]+$TS_ENTRY_FN[[:space:]]'` | **82.9% → 76.0%（-7pp）** | 合法 entry 必为 `entries = [...]` 列出的 fn 名，rocq-of-rust 对每个 fn item 生成 `Definition`；reject 条件（fn 名找不到 `Definition`）只在 fn item 被 silently skip 时成立，与"合法翻译完成"互斥 |

详细的实施记录 + 双向反误报论证（防漏报实测 + 反误报实测）见 implementation log §2.1 / §2.2 / §2.3；端到端 runner 验证记录见 implementation log §2.1（verifast hello entry 3861ms reject 验证）+ §2.3（rocq-of-rust 2-entry 验证）。

---

## 三、整体数字（v2）

### 总体

| 状态 | 数 | 占比 |
|---|---:|---:|
| SUCCESS | 1835 | **66.2%** |
| FAILED | 939 | 33.8% |
| UNKNOWN | 0 | 0% |
| TIMEOUT | 0 | 0% |

19 × 146 = 2774 task；v2 总通过率 66.2%（v1 70.3%，-4.1pp，整体差异由 verifast 大幅下降主导）。

### 按通过率排序的 19 工具总表（v2）

时长字段仅作环境上下文，**非工具评分**。"P12-B"标记的工具时长来自 P12-B 重跑（3 工具高 CPU 利用率，物理时间下限不同）。

| tool | n | S | F | rate | avg(ms) | p50 | p90 | max | 数据来源 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---|
| **cargo-check** | 146 | 146 | 0 | **100%** | 2124 | 222 | 6913 | 26420 | run-...5282 |
| **kani** | 146 | 144 | 2 | **98%** | 3871 | 1108 | 9952 | 40023 | run-...5282 |
| **miri** | 146 | 142 | 4 | **97%** | 2800 | 721 | 7031 | 35727 | run-...5282 |
| charon-poly | 146 | 139 | 7 | 95% | 2954 | 367 | 7232 | 34693 | run-...5282 |
| charon-mono | 146 | 138 | 8 | 94% | 2296 | 347 | 6912 | 31899 | run-...5282 |
| hax-fstar | 146 | 115 | 31 | 78% | 3315 | 1333 | 7785 | 33132 | run-...5282 |
| **rocq-of-rust** | 146 | **111** | **35** | **76%** | 76 | 76 | 89 | 170 | **P12-B (run-...69805)** |
| hax-lean | 146 | 110 | 36 | 75% | 3646 | 1859 | 8957 | 26768 | run-...5282 |
| soteria | 146 | 109 | 37 | 74% | 1846 | 1135 | 4008 | 12690 | run-...5282 |
| creusot | 146 | 106 | 40 | 72% | 40047 | 39972 | 52222 | 64641 | run-...5282 |
| hax-coq | 146 | 98 | 48 | 67% | 3735 | 1790 | 8004 | 26428 | run-...5282 |
| aeneas-coq | 146 | 87 | 59 | 59% | 4075 | 1743 | 9947 | 35173 | run-...5282 |
| aeneas-fstar | 146 | 87 | 59 | 59% | 3984 | 1906 | 8251 | 31561 | run-...5282 |
| aeneas-lean | 146 | 87 | 59 | 59% | 3854 | 1834 | 8056 | 36310 | run-...5282 |
| **prusti** | 146 | 56 | 90 | **38%** | 7609 | 6434 | 13352 | 25219 | **P12-B (run-...69805)** |
| aeneas-hol4 | 146 | 51 | 95 | 34% | 3989 | 1870 | 7805 | 32647 | run-...5282 |
| **verus** | 146 | 51 | 95 | **34%** | 562 | 514 | 909 | 2312 | run-...5282 |
| **kmir** | 146 | 46 | 100 | **31%** | 8196 | 7395 | 13353 | 105497 | run-...5282 |
| **verifast** | 146 | **12** | **134** | **8%** | 140 | 140 | 163 | 362 | **P12-B (run-...69805)** |

加粗的 4 个工具是 oracle 收紧后从虚高位置下移的——这些下降是宪法 §六-2 / §六-4 对 partial 与反作弊精神的正向落地，不是工具能力倒退。

**排序变化（v1 → v2）**：

- verifast 从第 7 名（79%，紧跟 charon×2）**跌到末位**（8%，低于 kmir 31%）—— v1 79.5% 是 vacuous pass 的语义降级（116 SUCCESS 中 104 个报 baseline `37 statements verified` 来自 verifast prelude 自身）。v2 把 vacuous pass 翻为 FAILED 后真实落到 corpus 0 spec 注解 + `-skip_specless_fns` 下 verifast 接受面的实际下限。
- rocq-of-rust 从第 6 名（83%，仅次于 charon×2 / cargo-check / kani / miri）**降到第 7 名**（76%，仍在 hax-fstar 之后但仍是 syntactic 派最高）——下降主要由 gate 6 抓获的 10 个 entry_fn silent skip-item 类（详 cc-report）贡献。
- prusti 排序不变（仍第 15 名，38%）。

---

## 四、新 oracle 下的失败模式新增分类

v2 在 v1 基础上新出现的 FAILED 类目（这些 entry 在 v1 上是 SUCCESS）：

### verifast：vacuous pass 类（104 entry）

wrapper exit 2 + stderr 诊断 `[verifast-oracle] FAIL: vacuous pass — symex executed 0 statements in src/lib.rs`。stdout 末行 `0 errors found (37 statements verified)` 来自 verifast 自家 prelude `lem_aux.rsspec`，verbose 输出 0 行命中 `src/lib.rs(`。

覆盖：整个 `float/* (10/10)` / `int-width/* (14/14)` / `int/* (2/2)` / `closure/* (2/2)` / `closure-adv (4/4)` / `arc / rc / refcell / vec / hello / box / unsafe-ptr / unsafe-adv / panic / generic（除 pair-struct）/ collections / impl-trait / hrtb / lifetime（除 cyclic-bound）/ slice / iter / error / box / 大量 *-limit 类目`等。

### rocq-of-rust：entry_fn silent skip-item 类（10 entry）

| entry | entry_fn |
| --- | --- |
| `aeneas-limit/mutually-recursive-traits/trigger_mutually_recursive_traits` | `trigger_mutually_recursive_traits` |
| `aeneas-limit/trait-impl-mut-param-mismatch/trigger_trait_impl_mut_param_mismatch` | `trigger_trait_impl_mut_param_mismatch` |
| `kani-limit/float-overapprox/trigger_check_sin_cos_identity` | `trigger_check_sin_cos_identity` |
| `miri-limit/simd-bitmask-large-vector/trigger_bitmask_over_64_elements` | `trigger_bitmask_over_64_elements` |
| `miri-limit/soundness-not-guaranteed/trigger_safe_wrapper_hides_ub` | `trigger_safe_wrapper_hides_ub` |
| `miri-limit/weak-memory-incomplete/relaxed_load_may_not_observe_all_stores` | `relaxed_load_may_not_observe_all_stores` |
| `prusti-limit/loan-crosses-loop-boundary/trigger_loan_crosses_loop_boundary` | `trigger_loan_crosses_loop_boundary` |
| `prusti-limit/ref-typed-struct-field/trigger_ref_typed_struct_field` | `trigger_ref_typed_struct_field` |
| `prusti-limit/shallow-borrow-match-guard/trigger_shallow_borrow_match_guard` | `trigger_shallow_borrow_match_guard` |
| `prusti-limit/spec-entailment-unsupported/trigger_spec_entailment_unsupported` | `trigger_spec_entailment_unsupported` |

stderr 诊断 `[rocq-oracle] FAIL: entry_fn '<fn>' missing from .v products (silent skip — top-level kind likely emitted vec![] or item dropped)`。原假说 "`call_unshimmed_foreign_fn` 96ms 是 ForeignMod silent skip" 被证伪：实测产物含 `Parameter getpid` + `Definition call_unshimmed_foreign_fn`，那个 entry 是真翻译完成（详 implementation log §2.3 末段）。

### prusti：无新增 FAILED

P12-B 重跑 56/146 数字与 v1 完全一致，无 entry 触发 `prusti-strict-wrapper.sh` 的新 `.vpr ≥ 1` reject 路径——证实 NEW config（`PRUSTI_NO_VERIFY=false + PRUSTI_DUMP_VIPER_PROGRAM=true + PRUSTI_PRINT_HASH=true`）在当前 commit `a0681ee` 上已抓住所有 silent path，新 wrapper 是反 commit-drift 的冗余防御层（实施动机 + 论证见 implementation log §2.2）。

---

## 五、工业三件套（更新）

| entry | base | exec | smt | mc | mir | syn |
| --- | --- | --- | --- | --- | --- | --- |
| industrial/rsa/rsa-pkcs8/rsa_pkcs1v15_encrypt | 1/1 | 1/2 | 1/4 | **0/2** | 2/6 | 3/4 |
| industrial/rsa/rsa-pkcs8/rsa_pubkey_from_pkcs8 | 1/1 | 1/2 | 1/4 | **0/2** | 2/6 | 3/4 |
| industrial/sha2/sha256-digest/sha256_digest_incremental | 1/1 | 1/2 | 1/4 | **0/2** | 2/6 | 3/4 |
| industrial/sha2/sha256-digest/sha256_digest_one_shot | 1/1 | 1/2 | 1/4 | **0/2** | 2/6 | 3/4 |
| industrial/x509-parser/cert-parse/x509_parse_der | 1/1 | 1/2 | 1/4 | **0/2** | 5/6 | **0/4** |
| industrial/x509-parser/cert-parse/x509_subject_extensions | 1/1 | 1/2 | 1/4 | **0/2** | 5/6 | **0/4** |

工业三件套数字与 v1 完全一致——industrial 6 entry 在 verifast 上**两个 oracle 版本都是 0/6**，verifast 的 6 个 industrial FAILED 在新旧 oracle 上同样是 exit 1（A 桶 unresolved import），与 strict-oracle-v2 的 vacuous-pass 封堵无关；rocq-of-rust 的 industrial 6/6 全 FAILED 也来自单文件不读 Cargo.toml（与 gate 6 无关）。

工业三件套 v1/v2 一致的四档分层：
- 5 个工具 6/6 全过：cargo-check / charon-mono / charon-poly / miri / kani(4/6 中 x509 上 lint 撞车，rsa+sha2 全过)
- hax-fstar / hax-lean / hax-coq 各 ~4/6 在 x509 上挂在 lint 升级 error
- aeneas-coq / aeneas-fstar / aeneas-lean / creusot 各 2/6（x509 过、rsa+sha2 全挂）
- 0/6：aeneas-hol4 / verus / prusti / kmir / verifast / soteria / rocq-of-rust

**这是本 corpus 上最强的工具能力分化信号**——P12-A 改造未触及此分层。

---

## 六、工具分组排序（v2）

按宪法 §三-3-2.a"形式指标"切割。组缩写：**baseline=cargo-check (1)**、**exec=miri+kmir (2)**、**smt=kani+verus+prusti+creusot (4)**、**mc=soteria+verifast (2)**、**mir=charon×2+aeneas×4 (6)**、**syn=hax×3+rocq-of-rust (4)**。

### 类内排序

| 类 | 工具排序（通过率） |
| --- | --- |
| baseline | cargo-check 100% |
| exec | miri 97% > kmir 31% |
| smt | kani 98% > creusot 72% > prusti 38% > verus 34% |
| mc | soteria 74% > **verifast 8%（v2；v1 79%）** |
| mir | charon-poly 95% > charon-mono 94% > aeneas-coq/fstar/lean 59% > aeneas-hol4 34% |
| syn | hax-fstar 78% > **rocq-of-rust 76%（v2；v1 83%）** > hax-lean 75% > hax-coq 67% |

### v1 → v2 类内排序变化

- **mc 类**：v1 verifast 79% > soteria 74%（数字接近）→ v2 soteria 74% **远超** verifast 8%（同类内 66pp 落差）—— 暴露的真相是：v1 下表面相近的两个数字背后 verifast 是 vacuous pass、soteria 是真符号执行。
- **syn 类**：v1 rocq-of-rust 83% 排第 1 → v2 rocq-of-rust 76% 跌至第 2（hax-fstar 78% 反超）。10 个 entry_fn silent skip 让 syntactic 派最高位易主。
- 其他 4 类排序不变（baseline / exec / smt / mir）。

### 工具按"容易错的大类"分组评（v2 校正版）

| 类 | 工具 | 通过率（v2） | 备注 |
| --- | --- | --- | --- |
| 编译型 baseline | cargo-check | 100% | 健全性基准；其他工具 FAILED 都来自工具自身能力差异 |
| 机械语义执行 | miri | 97% | 4 个 FAILED 全部预期：inline-asm / FFI / TCP socket / `MaybeUninit::assume_init()` UB |
| 机械语义执行 | kmir | 31% | stable-mir-json schema 漂移 + K rule 缺失（K-stuck 56 / SMIR JSON 35 / cargo build 9） |
| SMT 验证 | kani | 98% | `--only-codegen` 切割，几乎无 reject 路径；2 FAILED 在 x509 lint 撞车 |
| SMT 验证 | verus | 34% | `--no-verify` 切割；vstd 边界（A 桶 29）+ 语言子集（B 桶 19）+ verus-driver panic（D 桶 12）|
| SMT 验证 | prusti | **38%（P12-B）** | `NO_VERIFY=false ∧ DUMP_VIPER ∧ PRINT_HASH` + .vpr 存在性 check（v2 防御层）|
| SMT 验证 | creusot | 72% | binary 无 subcommand，Coma 翻译完成 |
| 模型检查 / 符号执行 | verifast | **8%（P12-B）** | wrapper-strict-oracle-v2 verbose user-file grep。corpus 0 spec → 残留 12 SUCCESS 是 verifast 对用户类型自动生成结构谓词验证完成（仍非用户 spec 验证完成） |
| 模型检查 / 符号执行 | soteria | 74% | 真符号执行，单文件输入模式 21 entry unresolved import |
| MIR 翻译 | charon-poly | 95% | 多态 LLBC，bigint/deps-complex/industrial 全过 |
| MIR 翻译 | charon-mono | 94% | 单态化 LLBC；mono / poly 5 entry 对称差，FAILED 集合互不蕴含 |
| MIR 翻译 | aeneas-coq / fstar / lean | 各 59% | 三 backend byte-identical，共享 charon stage 1 + aeneas mid-end |
| MIR 翻译 | aeneas-hol4 | 34% | hol4 backend `extract_trait_decl Option.get None` 单点 35pp 落差 |
| syntactic 翻译 | hax-fstar | 78% | F\* printer reject phase 多，几乎不走 silent path |
| syntactic 翻译 | rocq-of-rust | **76%（P12-B）** | 6 道门（v2 新增 entry_fn `Definition` 存在性）；10 entry 由 gate 6 抓获 silent skip-item |
| syntactic 翻译 | hax-lean | 75% | lean Printer 部分 silent sorry path 被 oracle 抓回 20 条 |
| syntactic 翻译 | hax-coq | 67% | coq Printer reject phase 阵列最完整、最严格 |

---

## 七、形式严格性合规度（v2）

按宪法 §三-3-2.b/c：

### 0 误报合规率：19/19 ✅

19 份 cc-report 都明确声明 0 误报状态：

| 形式可证 0 误报（13 工具） | 实测验证 0 误报（6 工具）|
|---|---|
| cargo-check / kani / miri / charon-poly / charon-mono / creusot / aeneas-coq / aeneas-fstar / aeneas-lean / aeneas-hol4 / prusti / verus / soteria / verifast | rocq-of-rust / hax-fstar / hax-lean / hax-coq / kmir |

**verifast 0 误报合规升级**：v1 是"⚠️ 实测验证"（基于 N statements 的 marker grep 不可形式证 0 误报），v2 切换到 verbose user-file grep——由 verifast 设计强制保证（spec-bearing fn 必走 prototype-impl-check + symex，必打 source-path tag），reject 条件构造性不可达 → 形式可证 ✅。

**rocq-of-rust 仍是实测**：gate 6 的 0 误报由"合法 entry 必为 fn item，rocq-of-rust 必为 fn item 生成 `Definition`"实测保证；当前 corpus 上 5 个用例均通过（fn / use / macro / extern crate / nested mod，详 implementation log §2.3）。

### 0 漏报状态

| 形式可证 0 漏报（14 工具） | 实测验证 0 漏报（5 工具）|
|---|---|
| cargo-check / miri / charon-poly / charon-mono / creusot / aeneas-coq / aeneas-fstar / aeneas-lean / aeneas-hol4 / prusti / verus / soteria / verifast / kmir | **kani / hax-fstar / hax-lean / hax-coq / rocq-of-rust** |

**verifast 0 漏报升级**：v1"⚠️ 实测验证"（依靠 README 文字层声明 vacuous pass）→ v2"✅ 形式可证"（verbose user-file grep 是 verifast 自身设计的 symex 标签，规则覆盖整个 spec-bearing 接受面）。

**rocq-of-rust 0 漏报升级**：v1 的 5 道门留 audit §3.2 标记的 silent skip-item 漏洞 → v2 gate 6 抓 entry_fn `Definition` 存在性，封堵该类。剩余盲点：上游引入新 silent fallback 路径不带已知 markers 且 entry_fn 仍被生成（理论窗口；本 corpus 0 现象）。

### 已知漏报盲点清单

按 v2 cc-report 自陈：

| 工具 | 漏报盲点 |
|---|---|
| cargo-check / miri / charon-poly / charon-mono / creusot / aeneas × 4 / prusti / verus / soteria / kmir | 无 |
| verifast | spec-bearing entry 中 verifast 内部某条 path 上 silent skip user fn 而仍 exit 0 + 写 verbose user-file 行（理论窗口；当前 corpus 0 spec entry 故未现身） |
| kani | codegen 完成 + unsupported warning 但 SAT 阶段才会触发的 entry（本次未观察到）|
| hax-fstar | hax engine 完全 skip item（实测 0 现象）；上游引入新 silent path 的可能 |
| hax-lean | hax engine 完全 skip item；上游可能引入新 silent path 而 grep 滞后 |
| hax-coq | hax engine 完全 skip item（实测 0 现象）；上游引入新 silent path 的可能 |
| rocq-of-rust | 上游引入新 silent fallback 路径不带已知 markers 且 entry_fn 仍被生成（理论窗口；本 corpus 0 现象） |

---

## 八、与项目目标的对齐（与 v1 一致）

v1 §八全文成立，未受 oracle v2 改造影响：

1. 本报告是次要模块产出——不是核心模块的承诺
2. 不构成对任何工具能力的长期承诺——所有数字锚定 2026-05-08 上述工具版本组合 + corpus
3. 不评工具语义忠实度 / 后端求解能力——按宪法 §二排除
4. 不区分翻译深浅——按宪法 §六-3，syntactic / 深 MIR / verifier dialect 一视同仁

**本报告新增声明**：v2 相对 v1 的数字回撤是"oracle 对工具语义降级 / silent path 抓获能力增强"的体现，**不是"工具能力"减弱**。读者解释 v1 / v2 数字差时务必区分这两件事——尤其 verifast 79.5% → 8.2% 的回撤完全是 oracle 改造造成（同 binary 同 corpus）。

---

## 九、附录

### 引用：每工具 cc-report

19 份深度报告路径同 v1，但 verifast / prusti / rocq-of-rust 三份已按 P12-B 数据更新：

```
deep-reports/cc-reports/cargo-check.md
deep-reports/cc-reports/kani.md
deep-reports/cc-reports/miri.md
deep-reports/cc-reports/charon-mono.md
deep-reports/cc-reports/charon-poly.md
deep-reports/cc-reports/rocq-of-rust.md     (P12-B 更新)
deep-reports/cc-reports/verifast.md         (P12-B 更新)
deep-reports/cc-reports/hax-fstar.md
deep-reports/cc-reports/hax-lean.md
deep-reports/cc-reports/soteria.md
deep-reports/cc-reports/creusot.md
deep-reports/cc-reports/hax-coq.md
deep-reports/cc-reports/aeneas-coq.md
deep-reports/cc-reports/aeneas-fstar.md
deep-reports/cc-reports/aeneas-lean.md
deep-reports/cc-reports/prusti.md           (P12-B 更新)
deep-reports/cc-reports/aeneas-hol4.md
deep-reports/cc-reports/verus.md
deep-reports/cc-reports/kmir.md
```

### 引用：宪法 / 架构 / 细化 / runner

```
docs/design/principles.md          — 项目精神宪法（核心约束）
docs/design/architecture.md         — 核心模块架构设计
docs/design/detailed-design.md      — 函数级细化、schema 完整定义
docs/design/tool-integration.md     — 工具集成边界 + §4.2 双向实测要求
runner/src/                         — 核心模块 1 实现
tools/<name>/{tool.toml, *.sh, harness.rs.tera, README.md}  — 19 工具配置
examples/<feature>/<dir>/           — 146 entry 样例库
```

### 引用：本次 dual run 数据

```
runs/run-1778226613-5282/results.json   — 主 run 裸 JSON 数据（host / 19 工具版本 / 全 task metadata）
runs/run-1778226613-5282/report.md      — 主 run 自动生成的 Markdown 表格
runs/run-1778238662-69805/results.json  — P12-B 重跑（3 工具 × 146 entries）
runs/run-1778238662-69805/report.md     — P12-B 重跑自动生成的 Markdown 表格
```

### 引用：oracle 漏报封堵实施

```
docs/fixes/oracle-leak-audit-2026-05-08.md          — 原始审计（识别 silent path）
docs/fixes/oracle-leak-rules-implementation-2026-05-08.md  — P12-A 实施记录 + 反误报论证
tools/verifast/verifast-strict-wrapper.sh           — verifast verbose user-file grep wrapper
tools/verifast/oracle-validation/                   — verifast 双向 micro-test
tools/prusti/prusti-strict-wrapper.sh               — prusti .vpr 存在性 check wrapper
tools/rocq-of-rust/tool.toml                        — gate 6 entry_fn `Definition` 检查
```

### 引用：历史报告

```
docs/test-reports/feature-coverage-2026-05-07.md             — 早期 8 工具版
docs/test-reports/feature-coverage-2026-05-07-19tools.md     — 19 工具版（harness 缺陷未修）
docs/test-reports/feature-coverage-2026-05-07-fixed-harness.md — 19 工具 fix 后（旧 oracle）
docs/test-reports/feature-coverage-2026-05-08-19tools-strict-oracle.md  — strict-oracle-v1（vacuous pass / silent skip-item 仍未抓）
docs/test-reports/feature-coverage-2026-05-08-strict-oracle-v2.md       — 本报告（v2，dual run 反映 P12-A 封堵）
```
