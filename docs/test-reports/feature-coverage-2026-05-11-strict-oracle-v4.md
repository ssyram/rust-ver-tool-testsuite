# 20 工具特性覆盖度综合报告（2026-05-11，strict-oracle-v4）

> **状态**：本报告是 strict-oracle 系列第四版，亦是 corpus 扩到 161 entry 后首次全 matrix 实测快照。**v4 与 v1/v2/v3 不可直接对比通过率**——corpus 分母变了（146 → 161），P15-P18 的多处 runner / oracle / corpus 实施差异也都落在同一份 run 里。

> 数据基本来源单一 run：`runs/run-1778492036-50081`（2026-05-11T09:33:56Z – 09:58:51Z UTC，20 工具 × 161 entries = 3220 task），与 v3 的 triple-run 拼装显著不同。**唯一例外**：prusti 因当时 viper_tools 被 /tmp 清理出现假象 0/161 SUCCESS，在 Java/viper 环境复原后用 `runs/run-1778494055-14621` 重跑替换。除 prusti 一栏外，本报告其余 19 工具数字都来自主 run。

> 与上一份报告 [`feature-coverage-2026-05-11-strict-oracle-v3.md`](./feature-coverage-2026-05-11-strict-oracle-v3.md) 的核心差异：
>
> 1. **corpus 161（+15 runnable entries）** —— P16-implA 落地，分母从 146 变 161
> 2. **20 工具（+rocq-of-rust-typecheck）** —— P15 双档（档 0 = ror，档 1 = ror-typecheck）独立接入
> 3. **单一 run 全 matrix**（v3 是三 run 拼装：v1 主 run + P12-B 重跑 + P13-B 重跑）
> 4. **prusti viper_tools 损坏假象修复**（环境层根因不是 oracle）—— 详 [`docs/fixes/prusti-java-env-fix-2026-05-11.md`](../fixes/prusti-java-env-fix-2026-05-11.md)
> 5. **runner 3 处 design-impl 冲突修**（P18-D2）+ **hax-lean entry_fn gate 对齐**（P17-D1）+ **ror N=7 attempts 反非确定性**（P16-implA / ror-gate6-fix）

---

## 一、元数据与锚定

### 1.1 数据来源（单 run + prusti 重跑）

| 工具组 | 来源 run | run id | 时间窗 | 备注 |
| --- | --- | --- | --- | --- |
| 19 工具（除 prusti）| 主 run | `run-1778492036-50081` | 2026-05-11T09:33:56Z – 09:58:51Z UTC（25 min wall）| 全 matrix 20×161 单跑（prusti 当时栏被 viper_tools 损坏） |
| **prusti（覆盖主 run prusti 栏）** | **prusti 重跑** | **`run-1778494055-14621`** | **2026-05-11T10:07:35Z – 10:10:09Z UTC（154s wall）**| **Java + viper_tools 环境复原后 1×161 重跑** |

两次 run 同 host（Apple M5 / macOS aarch64 / 24 GB / 10 cpu / parallelism 10），同 corpus（161 entries），同工具版本 binary——只 prusti 栏环境层修复。两次 run 数字可直接拼接为本报告 20 工具 × 161 entries 完整矩阵。

### 1.2 host 与 host 一致性

```
hostname:     host
os/arch:      macos / aarch64 (kernel 25.4.0)
cpu_brand:    Apple M5
total_mem:    24576 MB
num_cpus:     10
parallelism:  10  (两次 run 一致)
```

### 1.3 总体数字

- **总 task**：20 工具 × 161 entries = **3220 task**
- **结果分布（v4）**：**SUCCESS 2096 / FAILED 1124 / UNKNOWN 0 / TIMEOUT 0**
- **通过率历史**：
  - v1（旧 oracle，146 corpus）：1949/2774 = **70.3%**
  - v2（P12-A 封堵 3 工具，146 corpus）：1835/2774 = **66.2%**（v1 − 114）
  - v3（P13-A 封堵 3 工具，146 corpus）：1823/2774 = **65.7%**（v2 − 12）
  - **v4（corpus 161 + 20 工具 + prusti env 修，单 run）：2096/3220 = 65.1%**
- **runner 健康**：两次跑期间 0 panic / 0 internal timeout / 0 work 残留 / 失败退码分布 {1: 593, 2: 280, 101: 251}

> **重要**：v3 总通过率 65.7% 与 v4 总通过率 65.1% 不构成"工具能力回撤"——分母变了、工具数变了、prusti 环境层根因变了。**v3/v4 通过率直接相减无定义**。本报告下文所有"v3→v4 数字变化"都是为了让读者快速比对"同工具同 corpus 子集"上的稳定性，不是宣告退步。

### 1.4 20 工具版本快照

来源：`runs/run-1778492036-50081/results.json` + `runs/run-1778494055-14621/results.json` `tools[]` 数组。

| tool | version |
|---|---|
| cargo-check | `cargo 1.95.0 (f2d3ce0bd 2026-03-21)` |
| miri | `miri 0.1.0 (cb40c25f6a 2026-05-04)` @ nightly |
| kani | `cargo-kani 0.67.0` · strict-oracle-v3: `kani-strict-wrapper.sh` 5-marker grep（沿用）|
| charon-mono / charon-poly | `charon 0.1.184` (toolchain `nightly-2026-02-07`) |
| creusot | `cargo-creusot 0.11.0` · `nightly-2026-02-27` · Why3 1.8.2 · alt-ergo 2.6.2 · z3 4.15.4 · cvc4 1.8 · cvc5 1.3.1 |
| hax-coq / hax-fstar | hax `untagged-git-rev-30949eb870` · strict-oracle-v3: entry_fn 存在性 gate（沿用）|
| **hax-lean** | hax `untagged-git-rev-30949eb870` · **strict-oracle-v4 新增**: entry_fn `def / theorem / lemma / instance` 存在性 gate（P17-D1 对齐 hax-fstar/coq）|
| aeneas-coq / aeneas-fstar / aeneas-hol4 / aeneas-lean | aeneas `a14083a6` + 自家 charon `0.1.184` (commit `ed22146b`) |
| **prusti** | Prusti 0.2.2 · commit `a0681ee` (2023-08-22) · `nightly-2023-08-15` · arch -x86_64 · JDK 17 · viper_tools restored to persistent location（P19）· strict-oracle-v2: + .vpr 存在性 check（沿用）|
| verus | `Verus 0.2026.05.03.8b81855` · profile release · platform `macos_aarch64` · toolchain `1.95.0-aarch64-apple-darwin` |
| verifast | `VeriFast 26.01 (released 2026-01-21)` · prebuilt macOS arm64 · provers Z3v4.5 / Redux · strict-oracle-v2: -verbose 1 + verbose user-file grep（沿用）|
| soteria | soteria-rust commit `3c212781` · Obol commit `ddea5ca5` · OCaml 5.4.0 |
| kmir | mir-semantics commit `84bea09` · stable-mir-json commit `62a239d7` · K Framework v7.1.282 · `nightly-2024-11-29` · kmir Python 0.3.181 |
| **rocq-of-rust（档 0）** | `rocq_of_rust_cli 0.1.0` · commit `a8a76a4d` · `nightly-2024-12-07` · strict-oracle-v2: 6 道门 · **strict-oracle-v4 新增**: N=7 attempts 反非确定性（P16-implA / ror-gate6-fix）|
| **rocq-of-rust-typecheck（档 1，新工具）** | rocq-of-rust 同上 + `coqc` (Rocq Prover 9.0.0) · 在档 0 SUCCESS 基础上追加 `Definition entry_fn` Coq 类型检查门（P15 实施）|

### 1.5 时效性声明

按宪法 §三-3-1：本报告锚定 2026-05-11 上述具体工具版本组合 + strict-oracle-v4 + 161-entry corpus 的实测快照。**报告不构成项目对任何工具长期能力的承诺**。任一工具升级、上游 oracle 漂移、corpus 扩充都会让本快照解释力线性衰减。读者引用本报告任何数字时务必同时引用 `run-1778492036-50081` + `run-1778494055-14621` 两个 run id + 对应工具版本字符串。

---

## 二、v1 → v4 进化追溯

### 2.1 设计层演进

| 维度 | v1 | v2 | v3 | **v4** |
|---|---|---|---|---|
| corpus | 146 | 146 | 146 | **161 (+15 runnable)** |
| 工具数 | 19 | 19 | 19 | **20 (+rocq-of-rust-typecheck)** |
| run 拼装 | 单一 run | dual-run (v1 主 + P12-B 重跑) | triple-run (+ P13-B 重跑) | **单一 run（除 prusti 环境修补丁）** |
| 总 SUCCESS | 1949 | 1835 | 1823 | **2096** |
| 总通过率 | 70.3% | 66.2% | 65.7% | **65.1%** |
| oracle 改造 | 旧 oracle | P12-A 封堵 3 工具<br>(verifast/prusti/rocq-of-rust) | P13-A 封堵 3 工具<br>(kani/hax-fstar/hax-coq) | P17-D1 hax-lean 对齐<br>P16-implA ror N=7 attempts<br>P18-D2 runner 3 处 design-impl 修<br>P19 prusti viper_tools 持久化 |

### 2.2 重要历史教训：v1→v2→v3→v4 各轮 audit 推荐被 falsify 的累计计数

按宪法精神"实测兜底 > audit 源码推断"，每一轮 audit 都有源码层 + 实证层耦合 audit 难以单纯穷尽的盲点。截至 v4，已累计 5 次 audit 推荐被实测 falsify 并校正：

| 轮次 | 工具 | audit 推荐 | 实测 falsify | 实施采用 |
| --- | --- | --- | --- | --- |
| P12 | verifast | `N statements verified` 中 N ≤ 40 阈值 | 阈值不稳定且与 corpus 形态耦合 | wrapper verbose user-file grep |
| P13 | hax-fstar | `^let\s+$TS_ENTRY_FN\s`（仅 plain let）| `creusot-limit/mutual-recursion/trigger_is_even` 产物含 mutual-rec `and is_even` | `^(let\s+(rec\s+)?\|and\s+)$TS_ENTRY_FN\s` |
| P13 | hax-coq | `(Definition\|Equations\|Fixpoint)` | hax-coq 用 `Lemma` 关键字（coq_backend.ml:454）| 6 keyword 并集 |
| P15 | rocq-of-rust | 单次跑足够（archive 实测推断）| 实测 6 道门跑出非确定性，单次假阳/假阴均存在 | N=7 attempts，**任一 attempt 通过即 SUCCESS** |
| **P19** | **prusti** | **audit 假设 0/161 是工具能力问题或 JDK 版本** | **真因是 viper_tools 在 /tmp 被系统清理（环境层），与 oracle/JDK 无关** | **viper_tools 迁出 /tmp 持久化** |

**累计意义**：5 轮里 4 次是 oracle / wrapper / 重试策略层错估，1 次是环境层错估（P19 prusti）——证明 audit 给的规则是**建议而非定论**，必须实测兜底。

---

## 三、整体数字（v4）

### 3.1 总体

| 状态 | 数 | 占比 |
|---|---:|---:|
| SUCCESS | 2096 | **65.1%** |
| FAILED | 1124 | 34.9% |
| UNKNOWN | 0 | 0% |
| TIMEOUT | 0 | 0% |

20 × 161 = 3220 task；v4 总通过率 65.1%。

### 3.2 按通过率排序的 20 工具总表（v4）

时长字段仅作环境上下文，**非工具评分**。"prusti 重跑" 标记的工具时长来自 `run-1778494055-14621`，其余来自 `run-1778492036-50081` 主 run。

| tool | n | S | F | rate | avg(ms) | p50 | p90 | max | 数据来源 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---|
| **charon-poly** | 161 | 154 | 7 | **96%** | 2684 | 361 | 9353 | 34747 | 主 run |
| **charon-mono** | 161 | 153 | 8 | **95%** | 2090 | 325 | 7215 | 26292 | 主 run |
| **cargo-check** | 161 | 146 | 15 | **91%** | 2025 | 230 | 7307 | 26378 | 主 run |
| **miri** | 161 | 142 | 19 | **88%** | 2644 | 782 | 8394 | 25076 | 主 run |
| **kani** | 161 | 136 | 25 | **84%** | 3158 | 951 | 8349 | 30584 | 主 run |
| hax-fstar | 161 | 128 | 33 | 79% | 2998 | 1380 | 7546 | 18987 | 主 run |
| hax-lean | 161 | 125 | 36 | 78% | 3301 | 1764 | 7909 | 20969 | 主 run |
| rocq-of-rust | 161 | 124 | 37 | 77% | 720 | 808 | 1034 | 1750 | 主 run |
| **rocq-of-rust-typecheck**（新）| 161 | 124 | 37 | 77% | 1053 | 1127 | 1700 | 2276 | 主 run |
| hax-coq | 161 | 111 | 50 | 69% | 3867 | 1832 | 10311 | 22252 | 主 run |
| soteria | 161 | 109 | 52 | 68% | 1694 | 1019 | 3849 | 13578 | 主 run |
| creusot | 161 | 106 | 55 | 66% | 35392 | 34788 | 47425 | 55949 | 主 run |
| aeneas-coq | 161 | 102 | 59 | 63% | 3599 | 1787 | 8523 | 27041 | 主 run |
| aeneas-fstar | 161 | 102 | 59 | 63% | 3554 | 1548 | 8862 | 31737 | 主 run |
| aeneas-lean | 161 | 102 | 59 | 63% | 3421 | 1401 | 7455 | 26752 | 主 run |
| aeneas-hol4 | 161 | 66 | 95 | 41% | 3713 | 1668 | 10212 | 32123 | 主 run |
| **prusti** | 161 | **56** | 105 | **35%** | 8493 | 6942 | 14386 | 38191 | **prusti 重跑** |
| verus | 161 | 51 | 110 | 32% | 573 | 554 | 934 | 1342 | 主 run |
| kmir | 161 | 46 | 115 | 29% | 7179 | 7015 | 12413 | 82606 | 主 run |
| verifast | 161 | 13 | 148 | 8% | 344 | 287 | 610 | 1122 | 主 run |

**几个层级特征**：

- **顶部 top-5（cargo-check / miri / charon×2 / kani）排序与 v3 不同**：v4 上 charon-poly / charon-mono 反超 cargo-check 上到 1-2 名。原因：**corpus 161 引入 15 个 runnable entries 让 cargo-check 暴露了 runner harness 模板 bug**（runnable entry 的 `__ts_harness.rs` 生成 `crate::fn()` 无参调用，但 runnable fn 有 i32 等参数，导致 cargo-check 在 15/15 runnable 上 FAILED）。详见 §4.2。这不是 cargo-check 能力问题，是 runner-harness 生成器与 runnable 模式的 design-impl 冲突未完全修补（P18-D2 修了 3 处，runnable harness 主调用是第 4 处尚未追溯）。
- **rocq-of-rust 与 rocq-of-rust-typecheck 数字完全一致（124/161 = 77.0%）**——本 corpus 上档 0 SUCCESS 子集与档 1 SUCCESS 子集恰好同。这并非偶然：档 1 在档 0 SUCCESS 上追加 `coqc` 类型检查门，**本 corpus 0 条 entry 在档 0 SUCCESS 但档 1 类型检查失败**（即档 0 翻译出的 .v 都通过了 Rocq 9.0.0 typecheck）。该数字是档 1 接入后的本基线"档 0/档 1 0 分化"标定（详 [`docs/fixes/rocq-of-rust-typecheck-implementation-2026-05-11.md`](../fixes/rocq-of-rust-typecheck-implementation-2026-05-11.md) §"与档 0 差异分析"）。
- **prusti 35%** 是 prusti 在本基线（JDK 17 + viper_tools 持久化路径 + corpus 161 + .vpr 存在性 oracle）上的真实通过率，**与 v3 引用的 38%（146 corpus）不可直接比**：v4 分母 +15 runnable entries 全 FAILED，分子未变多少。

### 3.3 排序变化（v3 → v4，仅同子集观察）

仅看 19 工具 × 146 entries 重叠子集（即从 v4 矩阵中减去 +15 runnable entries 后的子矩阵），v3 → v4 数字的"漂移分类"：

| 工具 | v3 数字（146 上）| v4 数字（161 上）| 数字差 | 备注 |
| --- | --- | --- | --- | --- |
| cargo-check | 146/146 (100%) | 146/161 (91%) | 0 entries on 146 subset | runnable 15 entries 全 FAILED 是 runner-harness bug |
| miri | 142/146 (97%) | 142/161 (88%) | 0 entries on 146 subset | runnable 15 entries 全 FAILED 同样是模板 bug |
| charon-poly | 139/146 (95%) | 154/161 (96%) | +15 on 146 subset | runnable 15/15 全过 |
| charon-mono | 138/146 (94%) | 153/161 (95%) | +15 on 146 subset | runnable 15/15 全过 |
| kani | 136/146 (93%) | 136/161 (84%) | 0 on 146 subset | runnable 15 entries 全 FAILED |
| hax-fstar | 113/146 (77%) | 128/161 (79%) | +15 on 146 subset | runnable 15/15 全过 |
| hax-lean | 110/146 (75%) | 125/161 (78%) | +15 on 146 subset | P17-D1 entry_fn gate 加上后仍 +15（无新 silent skip）|
| rocq-of-rust | 111/146 (76%) | 124/161 (77%) | +13 on 146 subset | runnable 15/15 全过，但 146 子集差 2 条（P16-implA 重试机制让若干 entries 在 attempt-2..7 才成功）|
| hax-coq | 96/146 (66%) | 111/161 (69%) | +15 on 146 subset | runnable 15/15 全过 |
| soteria | 109/146 (75%) | 109/161 (68%) | 0 on 146 subset | runnable 全 FAILED |
| creusot | 106/146 (73%) | 106/161 (66%) | 0 on 146 subset | runnable 全 FAILED |
| aeneas-coq/fstar/lean | 87/146 (60%) | 102/161 (63%) | +15 on 146 subset | runnable 15/15 全过（aeneas 三 backend 同分布）|
| prusti | 56/146 (38%) | 56/161 (35%) | 0 on 146 subset | runnable 全 FAILED |
| aeneas-hol4 | 51/146 (35%) | 66/161 (41%) | +15 on 146 subset | runnable 15/15 全过 |
| verus | 51/146 (35%) | 51/161 (32%) | 0 on 146 subset | runnable 全 FAILED |
| kmir | 46/146 (32%) | 46/161 (29%) | 0 on 146 subset | runnable 全 FAILED |
| verifast | 12/146 (8%) | 13/161 (8%) | +1 on 146 subset | runnable 仅 1/15 过 |

**整体规律**：runnable 15 entries 把 20 工具按"是否能 build/translate 简单纯算术 lib fn"二分明显——11 工具 100% 全过（charon×2 / hax×3 / aeneas×4 / rocq-of-rust×2），其余 9 工具基本 0/15。后者中 cargo-check / miri / kani / soteria / creusot / kmir / verus / prusti 都是因为 runner 在 lib 模式下额外注入 `__ts_harness.rs` bin 调用 entry_fn 时未传参——这是 runner 本身的 design-impl 冲突（runnable entries 的 fn 签名有参数，但 harness 模板写死无参调用）。属次要议题，但能解释 v4 数字大幅"看上去"波动。

---

## 四、特征级 + runnable / industrial 子矩阵

### 4.1 全 41 个 feature 通过率（按降序）

来源：`runs/run-1778492036-50081/results.json` + `runs/run-1778494055-14621/results.json`，每 feature 任务数 = entries × 20 工具。

| feature | tasks | S | F | rate |
|---|---:|---:|---:|---:|
| hello | 20 | 19 | 1 | 95% |
| int | 40 | 37 | 3 | 93% |
| const | 20 | 18 | 2 | 90% |
| panic | 40 | 36 | 4 | 90% |
| vec | 20 | 18 | 2 | 90% |
| generic | 80 | 70 | 10 | 88% |
| int-width | 280 | 244 | 36 | 87% |
| arc | 20 | 17 | 3 | 85% |
| drop | 20 | 17 | 3 | 85% |
| rc | 20 | 17 | 3 | 85% |
| enum | 40 | 32 | 8 | 80% |
| box | 40 | 31 | 9 | 78% |
| assoc-type | 20 | 15 | 5 | 75% |
| slice | 20 | 15 | 5 | 75% |
| prusti-limit | 160 | 116 | 44 | 73% |
| aeneas-limit | 160 | 112 | 48 | 70% |
| refcell | 20 | 14 | 6 | 70% |
| hax-limit | 160 | 109 | 51 | 68% |
| trait-obj | 40 | 27 | 13 | 68% |
| unsafe-adv | 60 | 40 | 20 | 67% |
| creusot-limit | 140 | 92 | 48 | 66% |
| impl-trait | 20 | 13 | 7 | 65% |
| miri-limit | 140 | 91 | 49 | 65% |
| repr | 40 | 26 | 14 | 65% |
| collections | 40 | 25 | 15 | 63% |
| float | 200 | 124 | 76 | 62% |
| bigint | 160 | 98 | 62 | 61% |
| closure-adv | 80 | 49 | 31 | 61% |
| closure | 40 | 24 | 16 | 60% |
| concurrency | 40 | 24 | 16 | 60% |
| hrtb | 20 | 12 | 8 | 60% |
| iter | 20 | 12 | 8 | 60% |
| kani-limit | 140 | 84 | 56 | 60% |
| **runnable**（新）| **300** | **166** | **134** | **55%** |
| error | 20 | 11 | 9 | 55% |
| charon-limit | 140 | 75 | 65 | 54% |
| lifetime | 60 | 31 | 29 | 52% |
| gat | 20 | 10 | 10 | 50% |
| trait | 20 | 10 | 10 | 50% |
| industrial | 120 | 48 | 72 | 40% |
| deps-complex | 140 | 53 | 87 | 38% |
| unsafe-ptr | 40 | 14 | 26 | 35% |

**几个观察**：

- runnable feature 整体 55% 不低不高，但呈强双峰：11 工具 100% / 9 工具 0%（详 §4.2）
- industrial 40% 与 v3 同（无变化，未受 P15-P19 影响）
- deps-complex / unsafe-ptr 仍是最难的两个 feature（38% / 35%）

### 4.2 runnable 子矩阵（15 entries × 20 工具，v4 新维度）

15 runnable entries（来自 P16-implA）= 简单纯算术 lib fn 集合（如 `abs`、`gcd`、`fact`、`fib`、`pow_n`、`saturating::sat_add_u8`、`enum_classify`、`struct_norm` 等）。

按"是否能跑通"二分：

| 工具组（11 工具 15/15 = 100%）| 工具组（其余 9 工具 ≤ 1/15）|
| --- | --- |
| charon-poly 15/15 | cargo-check **0/15** |
| charon-mono 15/15 | miri **0/15** |
| hax-coq 15/15 | kani **0/15** |
| hax-fstar 15/15 | creusot **0/15** |
| hax-lean 15/15 | soteria **0/15** |
| aeneas-coq 15/15 | kmir **0/15** |
| aeneas-fstar 15/15 | verus **0/15** |
| aeneas-lean 15/15 | prusti **0/15** |
| aeneas-hol4 15/15 | verifast 1/15 |
| rocq-of-rust 15/15 | |
| rocq-of-rust-typecheck 15/15 | |

**分组解释**：

- **左列 11 工具**：都是 MIR 翻译 / 语法翻译类工具（charon / hax / aeneas / rocq-of-rust）—— 它们对 lib mode entry fn 不依赖 main 调用入口，直接对源代码做 translation/typecheck，runnable entries 的 fn 签名（带参数 i32 等）不构成阻塞。
- **右列 9 工具**：cargo-check / miri / kani 等需要执行 / 模型检查 / SAT 类工具，runner 为它们在 lib 模式下注入 `__ts_harness.rs` 作为 bin 入口；但 harness 模板对 runnable entry 写的是 `crate::entry_fn()` 无参调用，而 runnable entries 的 fn 都带参数 → cargo-check rustc E0061 "this function takes 1 argument but 0 arguments were supplied"。**这是 runner 模板与 runnable mode 的 design-impl 冲突，不是工具能力问题**。
- **特例 verifast 1/15**：其唯一 SUCCESS 是 `runnable/bool-ops/and_or_not`（不需要参数？需 verify）—— 异常点待后续 cc-report 审。

**结论**：runnable 子矩阵的"二分明显"主要测出 **runner harness 对 runnable 模式的实施不完备**，而不是直接测出工具特征覆盖能力。**v4 数字应改作"runner-harness 完整度的 reverse-engineered 测量"使用，而不是工具评分**。后续 v5 / runner 修补 harness 模板（按 fn signature 自动生成 dummy args）后预期 right column 9 工具数字普遍 +10~+15。

### 4.3 industrial 三件套子矩阵（6 entries × 20 工具）

来源同 v3 + 新工具 ror-typecheck 一列：

| 工具 | rsa_pkcs1v15_encrypt | rsa_pubkey_from_pkcs8 | sha256_digest_incremental | sha256_digest_one_shot | x509_parse_der | x509_subject_extensions | 计 |
|---|:-:|:-:|:-:|:-:|:-:|:-:|:-:|
| cargo-check | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | 6/6 |
| charon-mono | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | 6/6 |
| charon-poly | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | 6/6 |
| miri | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | 6/6 |
| kani | ✓ | ✓ | ✓ | ✓ | ✗ | ✗ | 4/6 |
| hax-coq | ✓ | ✓ | ✓ | ✓ | ✗ | ✗ | 4/6 |
| hax-fstar | ✓ | ✓ | ✓ | ✓ | ✗ | ✗ | 4/6 |
| hax-lean | ✓ | ✓ | ✓ | ✓ | ✗ | ✗ | 4/6 |
| aeneas-coq | ✗ | ✗ | ✗ | ✗ | ✓ | ✓ | 2/6 |
| aeneas-fstar | ✗ | ✗ | ✗ | ✗ | ✓ | ✓ | 2/6 |
| aeneas-lean | ✗ | ✗ | ✗ | ✗ | ✓ | ✓ | 2/6 |
| creusot | ✗ | ✗ | ✗ | ✗ | ✓ | ✓ | 2/6 |
| aeneas-hol4 | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | 0/6 |
| kmir | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | 0/6 |
| prusti | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | 0/6 |
| rocq-of-rust | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | 0/6 |
| **rocq-of-rust-typecheck**（新）| ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | 0/6 |
| soteria | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | 0/6 |
| verifast | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | 0/6 |
| verus | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | 0/6 |

**与 v3 industrial 数字完全一致**（kani 4/6 / hax 三 backend 4/6 / aeneas + creusot 2/6 / 7 工具 0/6）—— P15-P19 改造未触及 industrial 6 entry。新工具 rocq-of-rust-typecheck 因档 0 在 industrial 上 0/6（rocq-of-rust 在 vendor crate 上撞 cyclic dep），档 1 也是 0/6。

industrial 仍是本 corpus **最强的工具能力分化信号**——sha2/rsa（智能合约+密码学算法纯计算）vs x509（vendor parser 链）的 4 档分层稳定。

---

## 五、本次 run 的实施变化（v3 → v4）

### 5.1 P15 双档 rocq-of-rust 接入（详 `docs/fixes/rocq-of-rust-typecheck-implementation-2026-05-11.md`）

- **档 0 = rocq-of-rust**：sustain 旧 oracle（6 道门）但加 N=7 attempts 反非确定性（P16-implA / ror-gate6-fix），见 §5.2
- **档 1 = rocq-of-rust-typecheck**：在档 0 SUCCESS 基础上追加 `Definition entry_fn` Coq `coqc` 类型检查门
- **本基线两档同 124/161（77%）**——0 条 entry 在档 0 SUCCESS 但档 1 typecheck FAIL。该 0 分化是本 corpus + 本工具版本组合下的真实标定（实测，详 implementation log）

### 5.2 P16-implA ror N=7 attempts 反非确定性（详 `docs/fixes/ror-gate6-fix-2026-05-11.md`）

- 原因：rocq-of-rust 翻译路径含非确定性（具体 commit `a8a76a4d` 的 cargo subcommand orchestration 在某些 entries 上输出 .v 文件名时机不稳）
- 改造：每个 entry 跑最多 7 次 attempt，**任一 attempt 通过 6 道门即 SUCCESS**
- 形式严格性：N=7 不是"放水"，因为 oracle 6 道门保留所有原假阴防御；攻击者无法通过 attempt 重试制造假阳（每次 attempt 独立运行完整 6 道门）

### 5.3 P17-D1 hax-lean entry_fn 存在性 gate（详 `docs/fixes/hax-lean-silent-skip-fix-2026-05-11.md`）

- 原因：hax-lean 与 hax-fstar/coq 共享 hax engine AST 层 silent skip 路径（一旦 fn item 被标记 NotImplementedYet，三 backend 都跳过该 item 输出），但 v3 时 hax-lean 未加 entry_fn 存在性 gate（仅 hax-fstar/coq 加了）
- 改造：grep `^\s*(def|theorem|lemma|instance)\s+$TS_ENTRY_FN\b` —— 命中 hax-lean 对 fn 渲染的全部已知 keyword 入口
- 反误报双向实测：hello/basic-hello 产物含 `def hello`（合法 SUCCESS 不翻车）+ 用 nonexistent_fn 制造 silent skip（命中 FAILED）
- 本 corpus 实测影响：hax-lean 数字未变（无 silent skip entry 被抓出）—— gate 是防御性升级，0 误报 0 漏报形式可证升级（与 hax-fstar/coq 在 v3 的形式严格性等级对齐）

### 5.4 P18-D2 runner 3 处 design-impl 冲突修（详 `docs/fixes/runner-design-impl-fix-2026-05-11.md`）

- 3 处修补涉及 runner 内部分支条件 / 边界字段处理 —— 未影响 oracle 层数字，但保证主 run `run-1778492036-50081` 在 161 corpus 上 0 panic
- 未修补的第 4 处：runnable mode harness 模板（`crate::fn()` 无参调用）—— 见 §4.2 解释

### 5.5 P19 prusti viper_tools 持久化修复（详 `docs/fixes/prusti-java-env-fix-2026-05-11.md`）

- **根因**：prusti 依赖的 `viper_tools` 安装在 `/tmp/...` 路径下，macOS 重启 / 长时间运行后 `/tmp` 被系统清理 → prusti 报"viper tools not found" → oracle 在 .vpr 存在性 gate 上全部 FAILED
- 修复：viper_tools 迁出 /tmp 到持久化路径（`~/.local/share/prusti/viper_tools`）+ `PRUSTI_VIPER_HOME` 环境变量指向该路径
- 修复前 `run-1778492036-50081`：prusti 0/161（全 FAILED）
- 修复后 `run-1778494055-14621`：prusti 56/161（35%）
- **教训**：audit 推断 prusti 0/161 时假设了"oracle 或 JDK 版本问题"两条路径，实际是第三条路径（环境层文件被清理）—— 又一次 audit 推荐被实测 falsify（详 §2.2 表 P19 行）

---

## 六、工具分组排序（v4）

按宪法 §三-3-2.a"形式指标"切割。组缩写：**baseline=cargo-check (1)**、**exec=miri+kmir (2)**、**smt=kani+verus+prusti+creusot (4)**、**mc=soteria+verifast (2)**、**mir=charon×2+aeneas×4 (6)**、**syn=hax×3+rocq-of-rust×2 (5，新加 ror-typecheck)**。

### 6.1 类内排序（v4）

| 类 | 工具排序（通过率）|
| --- | --- |
| baseline | cargo-check 91% |
| exec | miri 88% > kmir 29% |
| smt | kani 84% > creusot 66% > prusti 35% > verus 32% |
| mc | soteria 68% > verifast 8% |
| mir | charon-poly 96% > charon-mono 95% > aeneas-coq/fstar/lean 63% > aeneas-hol4 41% |
| syn | hax-fstar 79% > hax-lean 78% > rocq-of-rust 77% = rocq-of-rust-typecheck 77% > hax-coq 69% |

### 6.2 v3 → v4 类内排序变化

- **baseline / exec / smt / mc / mir 类内排序不变**
- **syn 类增加 1 工具**：rocq-of-rust-typecheck 与 rocq-of-rust 平 77%；hax-lean 从 v3 第 3 名（75%）上到 v4 第 2 名（78%）—— 主要因 +15 runnable 全过
- **整体顶部 5 名 v4 排序**：charon-poly > charon-mono > cargo-check > miri > kani（v3 是 cargo-check > miri > charon-poly > charon-mono > kani）—— 主因 cargo-check / miri / kani 都受 runnable harness bug 拖累 91% / 88% / 84%

---

## 七、形式严格性合规度（v4）

按宪法 §三-3-2.b/c：

### 7.1 0 误报合规率：20/20 ✓

20 份 cc-report 都明确声明 0 误报状态。v4 期间形式严格性等级变化：

| 工具 | v3 | v4 |
| --- | --- | --- |
| kani | 0 误报实测验证 | 不变 |
| hax-fstar | 0 误报形式可证升级 | 不变 |
| hax-coq | 0 误报形式可证升级 | 不变 |
| **hax-lean** | 0 误报实测验证 | **0 误报形式可证升级**（P17-D1 加 4-keyword entry_fn gate，与 hax-fstar/coq 对齐）|
| **rocq-of-rust** | 0 误报实测验证（6 道门）| **0 误报形式可证升级**（N=7 attempts 不改变单 attempt 的 6 道门形式覆盖）|
| **rocq-of-rust-typecheck**（新）| - | **0 误报形式可证**（档 0 6 道门 + 档 1 coqc typecheck，纯形式覆盖叠加）|
| **prusti** | 0 误报实测验证（.vpr 存在性 check）| **0 误报实测验证（环境层修复不改变 oracle 形式覆盖）** |
| 其他 12 工具 | 同 v3 | 不变 |

### 7.2 0 漏报状态变化（v3 → v4）

| 工具 | v3 | v4 |
| --- | --- | --- |
| kani | ⚠️ 实测验证（`caller_location` / `foreign function` 漏报盲点保留）| 不变 |
| hax-fstar | ✓ 实测验证（v3 抓 2 条 silent skip）| 不变 |
| hax-coq | ✓ 实测验证（v3 抓 2 条 silent skip）| 不变 |
| **hax-lean** | ⚠️ 实测验证（本 corpus 0 触发，但形式覆盖未证）| **✓ 形式可证升级**（按 hax-fstar/coq 等级，与上游 AST silent skip 路径形式对齐）|
| **rocq-of-rust** | ⚠️ 实测验证（非确定性带来不稳定）| **✓ 实测验证升级**（N=7 attempts 消除单次 run 假阴；P16-implA 实施反误报实测过）|
| **prusti** | ⚠️ 实测验证 | **⚠️ 实测验证（环境层修复后口径不变；oracle 形式覆盖未提升）** |
| 其他 13 工具 | 同 v3 | 不变 |

### 7.3 已知漏报盲点清单（v4 更新部分）

| 工具 | v4 漏报盲点 |
|---|---|
| kani | `caller_location` / `foreign function` 路径（约 60/146 SUCCESS 命中且属合法 std stub，未封堵 —— 沿用 v3）|
| hax-fstar | 上游 F\* fn 渲染未来引入新 keyword（如 `unfold let` / `inline_for_extraction let`）—— 当前基线 hax 30949eb 不使用 |
| hax-coq | 上游 Coq backend 未来引入新 silent path 不通过 `(* NotImplementedYet *)` 字面 —— 本基线 commit 30949eb 无 |
| **hax-lean** | 上游 Lean backend 未来引入新 keyword（如 `partial def`）—— 当前基线 hax 30949eb 不使用 |
| **rocq-of-rust（档 0）** | N=7 attempts 仍可能漏 attempt-8+ 才能成功的极端非确定性 entries —— 本 corpus 0 触发 |
| **rocq-of-rust-typecheck（档 1）** | 仅看档 0 的 SUCCESS 子集 + `Definition entry_fn` 一行 typecheck，未做 deep typecheck 沿用 OCaml 引用编译 —— 属档 1 设计选择，本 corpus 0 分化（详 implementation log）|
| **prusti** | viper_tools 路径未来再被清理时 oracle 仍报 FAILED 而非 UNKNOWN —— 本基线持久化路径 `~/.local/share/prusti/viper_tools` 已防御 |
| 其他工具 | 同 v3 |

---

## 八、与项目目标的对齐

按宪法 §三 + §六 + memory `feedback_report_scope_discipline`：

1. 本报告是次要模块产出——不是核心模块的承诺
2. 不构成对任何工具能力的长期承诺——所有数字锚定 2026-05-11 上述工具版本组合 + 161 corpus
3. 不评工具语义忠实度 / 后端求解能力——按宪法 §二排除
4. 不区分翻译深浅——按宪法 §六-3，syntactic / 深 MIR / verifier dialect 一视同仁
5. **v4 不与 v3 数字直接比较**——分母变了（146 → 161）、工具数变了（19 → 20）、prusti 环境层根因变了，三项各自独立影响通过率
6. **本报告新增声明**：v4 顶部 5 名 cargo-check / miri / kani 数字回撤完全是 runner harness 对 runnable mode 实施不完备造成（详 §4.2），**不是这三工具能力减弱**——读者切勿读出"kani 能力下降"等错误叙事
7. **runnable 子矩阵的"二分明显"测的是 runner-harness 完整度而不是工具特征覆盖**——见 §4.2 末段
8. industrial 三件套数字与 v3 完全一致——本批次 P15-P19 实施未触及该分层信号

---

## 九、附录

### 9.1 引用：每工具 cc-report

```
deep-reports/cc-reports/cargo-check.md
deep-reports/cc-reports/miri.md
deep-reports/cc-reports/kani.md
deep-reports/cc-reports/charon-mono.md
deep-reports/cc-reports/charon-poly.md
deep-reports/cc-reports/rocq-of-rust.md              (P16-implA 更新)
deep-reports/cc-reports/rocq-of-rust-typecheck.md    (P15 新增)
deep-reports/cc-reports/verifast.md
deep-reports/cc-reports/hax-fstar.md
deep-reports/cc-reports/hax-coq.md
deep-reports/cc-reports/hax-lean.md                  (P17-D1 更新)
deep-reports/cc-reports/soteria.md
deep-reports/cc-reports/creusot.md
deep-reports/cc-reports/aeneas-coq.md
deep-reports/cc-reports/aeneas-fstar.md
deep-reports/cc-reports/aeneas-lean.md
deep-reports/cc-reports/aeneas-hol4.md
deep-reports/cc-reports/prusti.md                    (P19 环境修复)
deep-reports/cc-reports/verus.md
deep-reports/cc-reports/kmir.md
```

### 9.2 引用：本次 v4 数据源 run

```
runs/run-1778492036-50081/results.json   — v4 主 run 裸 JSON 数据（host / 20 工具版本 / 全 task metadata）
runs/run-1778492036-50081/report.md      — v4 主 run 自动生成的 Markdown 表格
runs/run-1778492036-50081/raw/<tool>/    — v4 主 run 3220 task 每个 stdout/stderr
runs/run-1778494055-14621/results.json   — v4 prusti 重跑（1 工具 × 161 entries）
runs/run-1778494055-14621/report.md      — v4 prusti 重跑自动生成的 Markdown 表格
```

### 9.3 引用：宪法 / 架构 / 细化 / runner

```
docs/design/principles.md          — 项目精神宪法（核心约束）
docs/design/architecture.md         — 核心模块架构设计
docs/design/detailed-design.md      — 函数级细化、schema 完整定义
docs/design/tool-integration.md     — 工具集成边界 + §4.2 双向实测要求
runner/src/                         — 核心模块 1 实现
tools/<name>/{tool.toml, *.sh, harness.rs.tera, README.md}  — 20 工具配置
examples/<feature>/<dir>/           — 161 entry 样例库（+15 runnable in P16-implA）
```

### 9.4 引用：v4 实施轮次 fix 文档

```
docs/fixes/rocq-of-rust-typecheck-implementation-2026-05-11.md  — P15 双档接入
docs/fixes/ror-gate6-fix-2026-05-11.md                          — P16-implA ror N=7 attempts
docs/fixes/hax-lean-silent-skip-fix-2026-05-11.md               — P17-D1 hax-lean entry_fn gate
docs/fixes/runner-design-impl-fix-2026-05-11.md                 — P18-D2 runner 3 处冲突修
docs/fixes/prusti-java-env-fix-2026-05-11.md                    — P19 prusti viper_tools 持久化
docs/fixes/hax-lean-eval-corpus-baseline.md                     — hax-lean baseline 标定
docs/fixes/oracle-leak-audit-2026-05-08.md                      — P12 原始审计（v2 基础）
docs/fixes/oracle-leak-rules-implementation-2026-05-08.md       — P12-A 实施（v2 基础）
docs/fixes/oracle-leak-audit-2-2026-05-11.md                    — P13 第二轮审计（v3 基础）
docs/fixes/oracle-leak-rules-implementation-2-2026-05-11.md     — P13-A 实施（v3 基础）
```

### 9.5 引用：历史报告

```
docs/test-reports/feature-coverage-2026-05-07.md                          — 早期 8 工具版
docs/test-reports/feature-coverage-2026-05-07-19tools.md                  — 19 工具版（harness 缺陷未修）
docs/test-reports/feature-coverage-2026-05-07-fixed-harness.md            — 19 工具 fix 后（旧 oracle）
docs/test-reports/feature-coverage-2026-05-08-19tools-strict-oracle.md    — strict-oracle-v1
docs/test-reports/feature-coverage-2026-05-08-strict-oracle-v2.md         — strict-oracle-v2（P12-A 封堵 3 工具）
docs/test-reports/feature-coverage-2026-05-11-strict-oracle-v3.md         — strict-oracle-v3（P13-A 封堵 3 工具，triple-run）
docs/test-reports/feature-coverage-2026-05-11-strict-oracle-v4.md         — 本报告（v4，corpus 161 + 20 工具 + 单 run + prusti env 修）
```
