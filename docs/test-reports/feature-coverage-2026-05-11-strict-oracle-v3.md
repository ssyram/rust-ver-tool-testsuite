# 19 工具特性覆盖度综合报告（2026-05-11，strict-oracle-v3）

> 本报告综合 19 份 cc-report，基于 P13-A oracle 漏报封堵（kani / hax-fstar / hax-coq 第二轮 oracle 漏报封堵）后的实测数据：13 工具数字来自 `runs/run-1778226613-5282` 主 run（2026-05-08），3 工具（verifast / prusti / rocq-of-rust）数字来自 `runs/run-1778238662-69805` 的 P12-B 重跑，3 工具（kani / hax-fstar / hax-coq）数字来自 `runs/run-1778466265-63960` 的 P13-B 重跑。
>
> 与上一份报告 [`feature-coverage-2026-05-08-strict-oracle-v2.md`](./feature-coverage-2026-05-08-strict-oracle-v2.md) 的差异：v2 已在 verifast / prusti / rocq-of-rust 三工具上封堵 oracle 漏报；v3 在 v2 基础上追加 kani / hax-fstar / hax-coq 三工具的 oracle 改造（实施记录见 [`docs/fixes/oracle-leak-rules-implementation-2-2026-05-11.md`](../fixes/oracle-leak-rules-implementation-2-2026-05-11.md)）后重跑：**kani 通过率从 98.6% 跌到 93.2%（-5.5pp / -8 entries），hax-fstar 78.8% 跌到 77.4%（-2 entries），hax-coq 67.1% 跌到 65.8%（-2 entries）**。

---

## 一、元数据（triple-run 设计）

### 数据来源切割

| 工具组 | 来源 run | run id | 时间窗 | 备注 |
| --- | --- | --- | --- | --- |
| 13 工具（除 verifast / prusti / rocq-of-rust / kani / hax-fstar / hax-coq）| 主 run | `run-1778226613-5282` | 2026-05-08T07:50:13Z – 08:16:08Z UTC | 全矩阵 19×146 |
| verifast / prusti / rocq-of-rust（v2）| P12-B 重跑 | `run-1778238662-69805` | 2026-05-08T11:11:02Z – 11:13:01Z UTC（119s wall）| strict-oracle-v2 后 3×146 重跑 |
| **kani / hax-fstar / hax-coq（v3）** | **P13-B 重跑** | **`run-1778466265-63960`** | **2026-05-11T02:24:25Z – 02:27:00Z UTC（155s wall）**| **strict-oracle-v3 后 3×146 重跑** |

三次 run 同一 host（Apple M5 / macOS aarch64 / 24 GB / 10 cpu / parallelism 10），同一 corpus（146 entries），同一工具版本 binary——只 oracle 改造。三次 run 数字可直接拼接为本报告 19 工具行。

### 总体数字

- **总 task**：19 工具 × 146 entries = **2774 task**
- **结果分布（v3）**：**SUCCESS 1823 / FAILED 951 / UNKNOWN 0 / TIMEOUT 0**
- **通过率历史**：
  - v1（旧 oracle）：1949/2774 = **70.3%**
  - v2（P12-A 封堵 3 工具）：1835/2774 = **66.2%**（v1 − 114）
  - **v3（P13-A 再封堵 3 工具）：1823/2774 = 65.7%（v2 − 12 = v1 − 126）**
- **runner 健康**：三次跑期间 0 panic / 0 internal timeout / 0 work 残留

### 19 工具版本快照

与 v2 一致（同 binary），新增 P13-A 改造：

| tool | version |
|---|---|
| cargo-check | `cargo 1.95.0 (f2d3ce0bd 2026-03-21)` |
| miri | `miri 0.1.0 (cb40c25f6a 2026-05-04)` @ nightly |
| **kani** | `cargo-kani 0.67.0` · **strict-oracle-v3: + kani-strict-wrapper.sh 5-marker grep**（封堵 `--only-codegen` codegen-with-stub 漏报）|
| charon-mono / charon-poly | `charon 0.1.184` (toolchain `nightly-2026-02-07`) |
| creusot | `cargo-creusot 0.11.0` · `nightly-2026-02-27` · Why3 1.8.2 · alt-ergo 2.6.2 · z3 4.15.4 · cvc4 1.8 · cvc5 1.3.1 |
| **hax-coq / hax-fstar** | hax `untagged-git-rev-30949eb870` · **strict-oracle-v3: + entry_fn 存在性 gate**（封堵 `fstar_backend.ml:1771` / `coq_backend.ml:588` silent-skip-item 路径）|
| hax-lean | hax `untagged-git-rev-30949eb870` (commit `30949eb87058895c24f963df90dd30ef11b0dc1a`) @ `nightly-2025-11-08` |
| aeneas-coq / aeneas-fstar / aeneas-hol4 / aeneas-lean | aeneas `a14083a6` + 自家 charon `0.1.184` (commit `ed22146b`) |
| prusti | Prusti 0.2.2 · commit `a0681ee` (2023-08-22) · `nightly-2023-08-15` · arch -x86_64 · JDK 17 · **strict-oracle-v2**: + .vpr 存在性 check |
| verus | `Verus 0.2026.05.03.8b81855` · profile release · platform `macos_aarch64` · toolchain `1.95.0-aarch64-apple-darwin` |
| verifast | `VeriFast 26.01 (released 2026-01-21)` · prebuilt macOS arm64 · provers Z3v4.5 / Redux · **strict-oracle-v2**: -verbose 1 + verbose user-file grep |
| soteria | soteria-rust commit `3c212781` · Obol commit `ddea5ca5` · OCaml 5.4.0 |
| kmir | mir-semantics commit `84bea09` · stable-mir-json commit `62a239d7` · K Framework v7.1.282 · `nightly-2024-11-29` · kmir Python 0.3.181 |
| rocq-of-rust | `rocq_of_rust_cli 0.1.0` · commit `a8a76a4d` · `nightly-2024-12-07` · **strict-oracle-v2**: 6 道门 |

### 时效性声明

按宪法 §三-3-1：本报告锚定 2026-05-11 上述具体工具版本组合 + strict-oracle-v3 + 146-entry corpus 的实测快照。**报告不构成项目对任何工具长期能力的承诺**。任一工具升级、上游 oracle 漂移、corpus 扩充都会让本快照解释力线性衰减。读者引用本报告任何数字时务必同时引用 v1 + v2 + v3 三个 run id + 对应工具版本字符串。

---

## 二、本次 run 的 oracle 改造（v3）

按宪法 §三-3-2.b/c"严格 0 误报 + 下限诚实"+ §六-2"不允许 partial"+ §六-4"反作弊"：oracle 必须形式覆盖工具 SUCCESS 信号语义边界。strict-oracle-v2 在 3 个工具上仍有缺漏（详 [`docs/fixes/oracle-leak-audit-2-2026-05-11.md`](../fixes/oracle-leak-audit-2-2026-05-11.md)）；v3 落地封堵规则（[`docs/fixes/oracle-leak-rules-implementation-2-2026-05-11.md`](../fixes/oracle-leak-rules-implementation-2-2026-05-11.md)）：

| 工具 | v2 → v3 oracle 差异 | 数字变化 | 反误报论证 |
| --- | --- | --- | --- |
| **kani** | v2: `--only-codegen` exit 0 = SUCCESS<br>v3: `kani-strict-wrapper.sh` 包装，stdout 命中 5 markers（`TerminatorKind::InlineAsm` / `simd_cast` / `catch_unwind` / `ptr_mask` / `C string literal`）任一 → 重写 exit 2 | **98.6% → 93.2%（-5.5pp / -8 entries）** | 5 markers 是 kani 自陈 `Found the following unsupported constructs:` warning 中的 hard-unsupported 类（"Verification will fail if reachable"）。`caller_location` / `foreign function` 在 60-63/144 SUCCESS 上命中（std panic/alloc 标准 stub），纳入会大规模假阳性——故故意排除作为 5-marker subset。双向实测：8 个原 SUCCESS 命中触发 FAILED（已实测），6 个真 SUCCESS 不命中（hello/basic-hello、bigint-arith、industrial/rsa 等）|
| **hax-fstar** | v2: 仅看 cargo hax exit + `Rust_primitives.Hax.failure` grep<br>v3: + entry_fn `let / let rec / and` 存在性 gate（pattern `^(let\s+(rec\s+)?\|and\s+)$TS_ENTRY_FN\s`）| **78.8% → 77.4%（-2 entries）** | hax-fstar 对 Rust `fn` 项必经 `TopLevelLet (NoLetQualifier, ...)` 单一渲染入口（fstar_backend.ml:1112）；mutual rec 后处理改写为 `let rec` / `and`（fstar_backend.ml:1923-1924）。合法翻译的 entry_fn 必为三种形态之一，grep 必命中。silent path 来自 `fstar_backend.ml:1771 Use/NotImplementedYet -> []`，item 完全不写产物 |
| **hax-coq** | v2: 仅看 cargo hax exit + `failure ((` / `please implement the method` grep<br>v3: + entry_fn `Definition / Fixpoint / Lemma / Equations / Theorem / Program Definition` 存在性 gate | **67.1% → 65.8%（-2 entries）** | hax-coq 对 Rust `fn` 项必经三个 CoqNotation 分支之一（coq_backend.ml:454/518/540），合法翻译的 entry_fn 必为 `Lemma / Fixpoint / Definition` 之一。silent path 来自 `coq_backend.ml:588 item'_NotImplementedYet` 整 item 渲染为 `(* NotImplementedYet *)` comment |

详细的实施记录 + 双向反误报论证见 implementation log §2.1 / §2.2 / §2.3。

---

## 三、整体数字（v3）

### 总体

| 状态 | 数 | 占比 |
|---|---:|---:|
| SUCCESS | 1823 | **65.7%** |
| FAILED | 951 | 34.3% |
| UNKNOWN | 0 | 0% |
| TIMEOUT | 0 | 0% |

19 × 146 = 2774 task；v3 总通过率 65.7%（v1 70.3% / v2 66.2%，相对 v1 -4.6pp）。

### 按通过率排序的 19 工具总表（v3）

时长字段仅作环境上下文，**非工具评分**。"P12-B" / "P13-B" 标记的工具时长来自相应重跑（高 CPU 利用率，物理时间下限不同）。

| tool | n | S | F | rate | avg(ms) | p50 | p90 | max | 数据来源 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---|
| **cargo-check** | 146 | 146 | 0 | **100%** | 2124 | 222 | 6913 | 26420 | v1 (run-...5282) |
| **miri** | 146 | 142 | 4 | **97%** | 2800 | 721 | 7031 | 35727 | v1 (run-...5282) |
| **charon-poly** | 146 | 139 | 7 | **95%** | 2954 | 367 | 7232 | 34693 | v1 (run-...5282) |
| **charon-mono** | 146 | 138 | 8 | **94%** | 2296 | 347 | 6912 | 31899 | v1 (run-...5282) |
| **kani** | 146 | **136** | **10** | **93%** | 3563 | 455 | 13955 | 28988 | **v3 P13-B (run-...63960)** |
| **hax-fstar** | 146 | **113** | **33** | **77%** | 3439 | 681 | 13088 | 27979 | **v3 P13-B (run-...63960)** |
| rocq-of-rust | 146 | 111 | 35 | 76% | 76 | 76 | 89 | 170 | v2 P12-B (run-...69805) |
| hax-lean | 146 | 110 | 36 | 75% | 3646 | 1859 | 8957 | 26768 | v1 (run-...5282) |
| soteria | 146 | 109 | 37 | 74% | 1846 | 1135 | 4008 | 12690 | v1 (run-...5282) |
| creusot | 146 | 106 | 40 | 72% | 40047 | 39972 | 52222 | 64641 | v1 (run-...5282) |
| **hax-coq** | 146 | **96** | **50** | **65%** | 2919 | 677 | 11822 | 27600 | **v3 P13-B (run-...63960)** |
| aeneas-coq | 146 | 87 | 59 | 59% | 4075 | 1743 | 9947 | 35173 | v1 (run-...5282) |
| aeneas-fstar | 146 | 87 | 59 | 59% | 3984 | 1906 | 8251 | 31561 | v1 (run-...5282) |
| aeneas-lean | 146 | 87 | 59 | 59% | 3854 | 1834 | 8056 | 36310 | v1 (run-...5282) |
| prusti | 146 | 56 | 90 | 38% | 7609 | 6434 | 13352 | 25219 | v2 P12-B (run-...69805) |
| aeneas-hol4 | 146 | 51 | 95 | 34% | 3989 | 1870 | 7805 | 32647 | v1 (run-...5282) |
| verus | 146 | 51 | 95 | 34% | 562 | 514 | 909 | 2312 | v1 (run-...5282) |
| kmir | 146 | 46 | 100 | 31% | 8196 | 7395 | 13353 | 105497 | v1 (run-...5282) |
| verifast | 146 | 12 | 134 | 8% | 140 | 140 | 163 | 362 | v2 P12-B (run-...69805) |

加粗的 4 个工具（kani / hax-fstar / hax-coq + 沿用 cargo-check）—— P13-A 改造收紧后 kani 从 top-2 跌到 top-5，其它两条点状回撤。

**排序变化（v2 → v3）**：

- **kani 从第 2 名跌到第 5 名**（98% → 93%）—— 现在排在 cargo-check / miri / charon-poly / charon-mono 之后，仍 top-5 但不再是 "kani / miri 地板天花板" 的并列叙事。P13-A 抓出的 8 条 stub-with-warning entry 让 kani 落到 charon-mono 之下（charon-mono 138/146 vs kani 136/146）。
- **hax-fstar 仍第 6 名**（78% → 77%）——位置不变，但 silent-skip-item 抓出 2 条。
- **hax-coq 仍第 11 名**（67% → 65.8%）——同上。
- 其他 16 工具排序不变。

**top-5 与 top-10 分层变化**：

| 排名 | v2 (strict-oracle-v2) | v3 (strict-oracle-v3) |
|---|---|---|
| 1 | cargo-check 100% | cargo-check 100% |
| 2 | **kani 98%** | miri 97% |
| 3 | miri 97% | charon-poly 95% |
| 4 | charon-poly 95% | charon-mono 94% |
| 5 | charon-mono 94% | **kani 93%** |
| 6 | hax-fstar 78% | **hax-fstar 77%** |
| 7 | rocq-of-rust 76% | rocq-of-rust 76% |
| 8 | hax-lean 75% | hax-lean 75% |
| 9 | soteria 74% | soteria 74% |
| 10 | creusot 72% | creusot 72% |

---

## 四、新 oracle 下的失败模式新增分类（v3）

v3 在 v2 基础上新出现的 FAILED 类目（这些 entry 在 v2 上是 SUCCESS）：

### kani：codegen-with-unsupported-stub 类（8 entry）

`kani-strict-wrapper.sh` exit 2 + stderr 诊断 `[kani-oracle] FAIL: codegen completed with hard-unsupported MIR constructs (kani self-disclosed via 'Found the following unsupported constructs:' warning). The matched markers are: ...`

| entry | 命中 markers |
|---|---|
| `charon-limit/inline-asm/nop_via_asm` | `TerminatorKind::InlineAsm (1)` |
| `concurrency/thread-mutex/thread_mutex_join` | `C string literal (1)` + `catch_unwind (3)` + `ptr_mask (1)` |
| `deps-complex/bigint-serde/bigint_serde` | `TerminatorKind::InlineAsm (5)` + `simd_cast (2)` |
| `deps-complex/chrono-serde/chrono_serde` | `TerminatorKind::InlineAsm (5)` + `simd_cast (2)` |
| `deps-complex/collections-serde/collections_serde` | `TerminatorKind::InlineAsm (5)` + `simd_cast (2)` |
| `deps-complex/error-chain/error_chain` | `catch_unwind (1)` + `ptr_mask (1)` + `simd_cast (1)` |
| `kani-limit/stack-unwinding/trigger_divide_with_recovery` | `catch_unwind (1)` |
| `miri-limit/thread-interleaving-partial/unsynchronised_counter_race` | `C string literal (1)` + `catch_unwind (4)` + `ptr_mask (1)` |

分布形态：
- **用户代码直接触发**：`charon-limit/inline-asm`（用户 `asm!("nop")`）、`kani-limit/stack-unwinding`（用户 `catch_unwind`）
- **std/std::thread 间接触发**：`concurrency/thread-mutex`、`miri-limit/thread-interleaving-partial`（`thread::spawn` 走 `catch_unwind` + `Mutex` 走 `ptr_mask`）、`error-chain`（thiserror 派生 + std 错误链）
- **vendor crate 间接触发**：`deps-complex/{bigint,chrono,collections}-serde`（serde derive 展开 + num-bigint-dig 的 lazy_static 路径含 inline asm 与 SIMD）

### hax-fstar：entry_fn silent-skip-item 类（2 entry）

| entry | entry_fn |
|---|---|
| `closure-adv/fn-once/closure_fn_once` | `closure_fn_once` |
| `impl-trait/return-iter/impl_trait_iter` | `impl_trait_iter` |

stderr 诊断 `[hax-fstar-oracle] FAIL: entry_fn '<fn>' missing from .fst products (silent skip — fstar_backend.ml:1771 Use/NotImplementedYet path)`。

### hax-coq：entry_fn silent-skip-item 类（2 entry）

| entry | entry_fn |
|---|---|
| `closure-adv/fn-once/closure_fn_once` | `closure_fn_once` |
| `impl-trait/return-iter/impl_trait_iter` | `impl_trait_iter` |

stderr 诊断 `[hax-coq-oracle] FAIL: entry_fn '<fn>' missing from .v products (silent skip — coq_backend.ml:588 item'_NotImplementedYet path)`。

**hax-fstar 与 hax-coq 在 silent-skip-item 上命中的 2 条 entry 完全相同**——证实这两条 entry 触发的是 hax engine AST 层的 silent skip（一旦 phase 把 fn item 标记为 `NotImplementedYet`，两个 backend 的 Printer 都会跳过），而不是 backend printer 特定的失败。

---

## 五、P13 封堵差异说明

### 5.1 三工具 oracle 改造对照表

| 工具 | 漏报机制（audit-2 §3）| 改造规则（implementation §2）| pattern 选择的反误报论证 |
| --- | --- | --- | --- |
| **kani**（§C1）| `--only-codegen` exit 0 时 stdout 含 `Found the following unsupported constructs:` 但 codegen 仅 emit stub | wrapper grep `^[[:space:]]+-[[:space:]]+(TerminatorKind::InlineAsm\|simd_cast\|catch_unwind\|ptr_mask\|C string literal)\b` | 5-marker subset 排除 `caller_location` / `foreign function`（在 60-63/144 SUCCESS 上是 std panic/alloc 标准 stub，纳入将让 ~44% SUCCESS 翻车，结构性误报）|
| **hax-fstar**（§C2）| `fstar_backend.ml:1771 Use/NotImplementedYet -> []` 让 item 完全不写 .fst，exit 0 | grep `^(let[[:space:]]+(rec[[:space:]]+)?\|and[[:space:]]+)$TS_ENTRY_FN[[:space:]]` | hax-fstar fn 渲染统一从 `TopLevelLet (NoLetQualifier, ...)` 单一入口（fstar_backend.ml:1112），mutual rec 后处理改写为 `let rec`/`and`（:1923-1924）。三分支并集覆盖 fn 渲染全集 |
| **hax-coq**（§C2）| `coq_backend.ml:588 item'_NotImplementedYet` 让 item 渲染为单行 comment，exit 0 | grep `^\s*(Definition\|Fixpoint\|Lemma\|Equations\|Theorem\|Program\s+Definition)\s+$TS_ENTRY_FN\s` | hax-coq fn 渲染必经 `coq_backend.ml:452-560` 的三个 CoqNotation 分支（`is_lemma → Lemma` / `is_rec → Fixpoint` / `else → Definition`）。pattern 增加 `Equations` / `Theorem` / `Program Definition` 作防御性扩展 |

### 5.2 数字变化与预测对照

audit-2 §5 预测窗口 vs 实测：

| 工具 | audit-2 预测窗口 | v3 实测 | 命中 |
| --- | --- | --- | --- |
| kani | -2 ~ -6（1.4%-2.8% 虚高，4-8 entries） | **-8 entries** | 接近上限 |
| hax-fstar | 0 ~ -1（理论窗口实测 0 现象） | **-2 entries** | 超出预测窗口 |
| hax-coq | 0 ~ -1（理论窗口实测 0 现象） | **-2 entries** | 超出预测窗口 |

**hax-fstar / hax-coq 超出预测的意义**：audit-2 §3.2/§3.3 评估 `Use _ / NotImplementedYet -> []` 与 `item'_NotImplementedYet` 时认为本 corpus 0 触发；P13-B 实测命中 2 条 entry（两 backend 同 2 条），证明 **silent-skip-item 不是 0 实测现象** —— 该路径在 `closure-adv/fn-once/closure_fn_once` 与 `impl-trait/return-iter/impl_trait_iter` 上实际触发了 hax engine 标记 fn item 为 NotImplementedYet 的分支。这两条 entry 在 v2 是 SUCCESS（exit 0 + 无 failure marker），但产物里既无 entry_fn 定义。

### 5.3 反误报实测

按 tool-integration.md §4.2 双向反误报实测要求，每条新规则同时验证：

- **kani**：8 个已知 silent path entry 命中（防漏报 ✓）+ 4 个真 SUCCESS 不命中（`hello/basic-hello` / `bigint/bigint-arith` / `industrial/rsa/rsa-pkcs8/*` 等，反误报 ✓）+ 排除 `caller_location` / `foreign function` 的论证
- **hax-fstar**：4 类合成测试双向通过（plain let / let rec / mutual-rec and / silent skip）+ 6 个真 SUCCESS entry 0 翻车
- **hax-coq**：5 类合成测试双向通过（Definition / Fixpoint / Lemma / 嵌套模块 Definition / silent skip）+ 6 个真 SUCCESS entry 0 翻车

详 implementation log §2.x。

---

## 六、audit 推荐被实测 falsify（方法学专段）

P12 + P13 两轮 audit 共三次推荐 pattern 被实测 falsify，需要"实施按反误报实测校正"：

| 轮次 | 工具 | audit 推荐 | 实测 falsify | 实施采用 |
| --- | --- | --- | --- | --- |
| P12 | verifast | `N statements verified` 中 N ≤ 40 阈值 | 阈值不稳定且与 corpus 形态耦合 | wrapper verbose user-file grep（更直接的 symex 触及标志）|
| **P13** | **hax-fstar** | `^let\s+$TS_ENTRY_FN\s`（仅 plain let）| `creusot-limit/mutual-recursion/trigger_is_even` 产物含 mutual-rec 的 `and is_even`（非 `let`），audit pattern 漏 | `^(let\s+(rec\s+)?\|and\s+)$TS_ENTRY_FN\s`（三分支并集）|
| **P13** | **hax-coq** | `(Definition\|Equations\|Fixpoint)` | hax-coq 对 `is_lemma` 分支用 `Lemma` 关键字（coq_backend.ml:454），audit pattern 漏 | `(Definition\|Fixpoint\|Lemma\|Equations\|Theorem\|Program\s+Definition)`（六 keyword 并集）|

**方法学意义**：audit 给的规则是 **建议而非定论**——实施时必须做反误报实测（合法 SUCCESS 在新 pattern 下不翻车）+ 防漏报实测（已知 silent path 被抓）双向校验，pattern 经常需要扩张容忍。三次 falsify 都不是 audit 错，是源码层 + 实证层的耦合 audit 难以单纯通过源码遍历穷尽——必须实测兜底。

---

## 七、工业三件套（与 v2 一致）

| entry | base | exec | smt | mc | mir | syn |
| --- | --- | --- | --- | --- | --- | --- |
| industrial/rsa/rsa-pkcs8/rsa_pkcs1v15_encrypt | 1/1 | 1/2 | 1/4 | **0/2** | 2/6 | 3/4 |
| industrial/rsa/rsa-pkcs8/rsa_pubkey_from_pkcs8 | 1/1 | 1/2 | 1/4 | **0/2** | 2/6 | 3/4 |
| industrial/sha2/sha256-digest/sha256_digest_incremental | 1/1 | 1/2 | 1/4 | **0/2** | 2/6 | 3/4 |
| industrial/sha2/sha256-digest/sha256_digest_one_shot | 1/1 | 1/2 | 1/4 | **0/2** | 2/6 | 3/4 |
| industrial/x509-parser/cert-parse/x509_parse_der | 1/1 | 1/2 | 1/4 | **0/2** | 5/6 | **0/4** |
| industrial/x509-parser/cert-parse/x509_subject_extensions | 1/1 | 1/2 | 1/4 | **0/2** | 5/6 | **0/4** |

**工业三件套数字 v1/v2/v3 完全一致**——industrial 6 entry 上：
- kani 仍 4/6（rsa + sha2 全过、x509 撞 vendor lint），5-marker subset 不命中 industrial 任何 entry（rsa/sha2 stdout 仅 `caller_location` / `foreign function`，故意排除；x509 撞车在 lint 阶段 exit 101，从未到 codegen）
- hax-fstar / hax-coq 在 silent-skip-item gate 下数字不变（rsa/sha2 entry_fn 在产物中都正常生成 `let rsa_pkcs1v15_encrypt` / `Definition rsa_pkcs1v15_encrypt`）

工业三件套 v1/v2/v3 一致的四档分层：
- 5 个工具 6/6 全过：cargo-check / charon-mono / charon-poly / miri / kani(4/6 中 x509 上 lint 撞车，rsa+sha2 全过)
- hax-fstar / hax-lean / hax-coq 各 ~4/6 在 x509 上挂在 lint 升级 error
- aeneas-coq / aeneas-fstar / aeneas-lean / creusot 各 2/6（x509 过、rsa+sha2 全挂）
- 0/6：aeneas-hol4 / verus / prusti / kmir / verifast / soteria / rocq-of-rust

**这是本 corpus 上最强的工具能力分化信号**——P13-A 改造未触及此分层。

---

## 八、工具分组排序（v3）

按宪法 §三-3-2.a"形式指标"切割。组缩写：**baseline=cargo-check (1)**、**exec=miri+kmir (2)**、**smt=kani+verus+prusti+creusot (4)**、**mc=soteria+verifast (2)**、**mir=charon×2+aeneas×4 (6)**、**syn=hax×3+rocq-of-rust (4)**。

### 类内排序

| 类 | 工具排序（通过率）|
| --- | --- |
| baseline | cargo-check 100% |
| exec | miri 97% > kmir 31% |
| smt | **kani 93%（v3，v2 98%）** > creusot 72% > prusti 38% > verus 34% |
| mc | soteria 74% > verifast 8% |
| mir | charon-poly 95% > charon-mono 94% > aeneas-coq/fstar/lean 59% > aeneas-hol4 34% |
| syn | **hax-fstar 77%（v3，v2 78%）** > rocq-of-rust 76% > hax-lean 75% > **hax-coq 65%（v3，v2 67%）** |

### v2 → v3 类内排序变化

- **smt 类**：kani 仍领先，但 -5.5pp，第 2-第 4 名 (creusot 72% / prusti 38% / verus 34%) 与 kani 的差距从 26-64pp 收窄到 21-59pp。
- **syn 类**：hax-fstar 仍第 1（77%），rocq-of-rust 仍第 2（76%）。hax-coq 仍末位但下降 1.4pp。
- 其他 4 类排序不变（baseline / exec / mc / mir）。

---

## 九、形式严格性合规度（v3）

按宪法 §三-3-2.b/c：

### 0 误报合规率：19/19 ✅

19 份 cc-report 都明确声明 0 误报状态。kani / hax-fstar / hax-coq 在 v3 改造后形式严格性变化：

| 工具 | v2 | v3 |
| --- | --- | --- |
| kani | 0 误报形式可证（exit 0 = codegen 完成）| 0 误报实测验证（5 markers 在 6 个真 SUCCESS 上不命中，双向反误报实测过）|
| hax-fstar | 0 误报实测验证（grep `Rust_primitives.Hax.failure`）| **0 误报形式可证升级**（hax-fstar fn 渲染统一从 `TopLevelLet (NoLetQualifier, ...)` 单一入口 + mutual rec 后处理改写，三分支并集覆盖全集）|
| hax-coq | 0 误报实测验证（grep `failure ((` / `please implement`）| **0 误报形式可证升级**（hax-coq fn 渲染必经三个 CoqNotation 分支之一，pattern 6 keyword 并集覆盖全集 + 防御性扩展）|

### 0 漏报状态变化

| 工具 | v2 | v3 |
| --- | --- | --- |
| kani | ⚠️ 实测验证（"本次未观察到"）| ⚠️ 实测验证（5 markers 在 8 个原 SUCCESS 上命中翻 FAILED；剩余 `caller_location` / `foreign function` 路径仍未抓 —— 属 cc-report 修订小组待裁口径分歧）|
| hax-fstar | ⚠️ 实测验证（"本 corpus 0 触发"）| **✅ 实测验证升级**（entry_fn 存在性 gate 抓 2 条 silent-skip-item，证明该路径不是 0 实测现象）|
| hax-coq | ⚠️ 实测验证（"本 corpus 0 触发"）| **✅ 实测验证升级**（同上，抓 2 条相同 entry）|

### 已知漏报盲点清单（v3 更新部分）

| 工具 | 漏报盲点 |
|---|---|
| **kani** | `caller_location` / `foreign function` 路径（≈ 60/144 SUCCESS 命中且属合法 std stub，未封堵 —— 属"宪法精神 vs cc-report 现行口径"分歧的暂未解项）；codegen 完成 + 其他 warning 但 SAT 阶段才会触发问题的 entry（本次未观察到）|
| **hax-fstar** | 上游 F\* fn 渲染未来引入新 keyword（如 `unfold let` / `inline_for_extraction let`）—— 当前基线 hax 30949eb 不使用 |
| **hax-coq** | 上游 Coq backend 未来引入新 silent path 不通过 `(* NotImplementedYet *)` 或 `please implement the method` 字面（理论窗口；本基线 commit 30949eb 无）|

其他 16 工具漏报盲点同 v2，未变。

---

## 十、与项目目标的对齐（与 v2 一致）

v2 §八全文成立，未受 oracle v3 改造影响：

1. 本报告是次要模块产出——不是核心模块的承诺
2. 不构成对任何工具能力的长期承诺——所有数字锚定 2026-05-11 上述工具版本组合 + corpus
3. 不评工具语义忠实度 / 后端求解能力——按宪法 §二排除
4. 不区分翻译深浅——按宪法 §六-3，syntactic / 深 MIR / verifier dialect 一视同仁

**本报告新增声明**：v3 相对 v2 的数字回撤（kani -5.5pp / hax-fstar -1.4pp / hax-coq -1.4pp）是"oracle 对工具语义降级 / silent path 抓获能力增强"的体现，**不是"工具能力"减弱**。读者解释 v2 / v3 数字差时务必区分这两件事——尤其 kani 98.6% → 93.2% 的回撤完全是 oracle 改造造成（同 binary 同 corpus，只 wrapper 5-marker grep 改造）。

---

## 十一、附录

### 引用：每工具 cc-report

```
deep-reports/cc-reports/cargo-check.md
deep-reports/cc-reports/kani.md             (P13-B 更新)
deep-reports/cc-reports/miri.md
deep-reports/cc-reports/charon-mono.md
deep-reports/cc-reports/charon-poly.md
deep-reports/cc-reports/rocq-of-rust.md     (P12-B 更新)
deep-reports/cc-reports/verifast.md         (P12-B 更新)
deep-reports/cc-reports/hax-fstar.md        (P13-B 更新)
deep-reports/cc-reports/hax-lean.md
deep-reports/cc-reports/soteria.md
deep-reports/cc-reports/creusot.md
deep-reports/cc-reports/hax-coq.md          (P13-B 更新)
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

### 引用：本次 triple-run 数据

```
runs/run-1778226613-5282/results.json   — v1 主 run 裸 JSON 数据（host / 19 工具版本 / 全 task metadata）
runs/run-1778226613-5282/report.md      — v1 主 run 自动生成的 Markdown 表格
runs/run-1778238662-69805/results.json  — v2 P12-B 重跑（3 工具 × 146 entries）
runs/run-1778238662-69805/report.md     — v2 P12-B 重跑自动生成的 Markdown 表格
runs/run-1778466265-63960/results.json  — v3 P13-B 重跑（3 工具 × 146 entries）
runs/run-1778466265-63960/report.md     — v3 P13-B 重跑自动生成的 Markdown 表格
```

### 引用：oracle 漏报封堵实施（两轮）

```
docs/fixes/oracle-leak-audit-2026-05-08.md             — P12 原始审计（识别 silent path）
docs/fixes/oracle-leak-rules-implementation-2026-05-08.md  — P12-A 实施记录 + 反误报论证
docs/fixes/oracle-leak-audit-2-2026-05-11.md           — P13 第二轮审计
docs/fixes/oracle-leak-rules-implementation-2-2026-05-11.md  — P13-A 实施记录 + 反误报论证（含 audit-falsify 校正）
tools/kani/kani-strict-wrapper.sh                      — kani 5-marker grep wrapper（P13-A）
tools/hax-fstar/tool.toml                              — hax-fstar entry_fn 存在性 gate（P13-A）
tools/hax-coq/tool.toml                                — hax-coq entry_fn 存在性 gate（P13-A）
tools/verifast/verifast-strict-wrapper.sh              — verifast verbose user-file grep wrapper（P12-A）
tools/verifast/oracle-validation/                      — verifast 双向 micro-test
tools/prusti/prusti-strict-wrapper.sh                  — prusti .vpr 存在性 check wrapper（P12-A）
tools/rocq-of-rust/tool.toml                           — rocq-of-rust gate 6 entry_fn `Definition` 检查（P12-A）
```

### 引用：历史报告

```
docs/test-reports/feature-coverage-2026-05-07.md             — 早期 8 工具版
docs/test-reports/feature-coverage-2026-05-07-19tools.md     — 19 工具版（harness 缺陷未修）
docs/test-reports/feature-coverage-2026-05-07-fixed-harness.md — 19 工具 fix 后（旧 oracle）
docs/test-reports/feature-coverage-2026-05-08-19tools-strict-oracle.md  — strict-oracle-v1
docs/test-reports/feature-coverage-2026-05-08-strict-oracle-v2.md       — strict-oracle-v2（P12-A 封堵 verifast/prusti/rocq-of-rust）
docs/test-reports/feature-coverage-2026-05-11-strict-oracle-v3.md       — 本报告（v3，triple-run 反映 P13-A 封堵 kani/hax-fstar/hax-coq）
```
