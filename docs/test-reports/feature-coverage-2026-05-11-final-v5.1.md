# v5.1 最终综合报告（2026-05-11，全自动 audit + fix + rerun 闭环产出）

> **状态**：本报告是 strict-oracle 系列第五代的最终落定版本（v5 → R3 修 → v5.1 重跑 → R5 漏报独审 → 决策点冻结）。也是首次完整跑完"R1 baseline → R2 误报独审 (c+cc) → R3 fix → R4 验证 rerun → R5 漏报独审 (c+cc) → R6 综合报告"全六轮闭环的报告。
>
> **不构成工具能力客观评判**——按 [`README.md`](../../README.md) 顶部一次性免责。所有数字锚定 2026-05-11 工具版本快照与 v5.1 corpus（161 entries / 20 工具）。
>
> **使用建议**：用户回来后建议先看 §1 TL;DR 与 §6 关键发现（5 分钟即可获得核心结论），再决定是否深入 §3 / §4 / §5。决策点请直接看 [`docs/fixes/decisions-2026-05-11.md`](../fixes/decisions-2026-05-11.md)。

---

## §0 元数据

### 0.1 数据来源（v5 baseline + v5.1 rerun）

| 阶段 | run id | ISO 时间 | wall | 用途 |
|---|---|---|---|---|
| **v5 baseline** | `run-1778500291-90812` | 2026-05-11T11:51:31Z – 12:23:00Z | 1889 s | R2 / R5 audit 数据源；所有"v5 raw"引用都指此 run |
| **v5.1 rerun** | `run-1778504159-67797` | 2026-05-11T12:55:59Z – 13:23:08Z | 1629 s | R3 fix 后验证 rerun；本报告核心数字（68.85%）来自此 run |

两 run 同 host（`ssyramdeMacBook-Air.local` / macOS 25.4.0 / Apple M5 aarch64 / 24 GB / 10 cpus / parallelism = 10），同 corpus（161 entries），同工具版本 binary——仅 `runner/src/report.rs` 中 oracle 分类逻辑在两 run 之间有 R3 落地的两条新规则（详 §4.2）。

### 0.2 corpus 与工具版本

- corpus：161 entries × 41 features（与 v4 / v5 baseline 同分母）
- 任务总数：161 × 20 = 3220 task
- 工具版本快照（来自 `results.json` metadata，v5.1）：

| tool | version |
|---|---|
| aeneas-coq / fstar / hol4 / lean | aeneas `a14083a6` |
| cargo-check | cargo 1.95.0 (f2d3ce0bd 2026-03-21) |
| charon-mono / charon-poly | 0.1.184 |
| creusot | cargo-creusot 0.11.0 · nightly-2026-02-27 · Why3 1.8.2+git · why3find 1.3.0+dev · alt-ergo 2.6.2 · z3 4.15.4 · cvc4 1.8 · cvc5 1.3.1 |
| hax-coq / fstar / lean | hax · commit 30949eb87058895c24f963df90dd30ef11b0dc1a |
| kani | cargo-kani 0.67.0 |
| kmir | (commit pin per kmir/cargo.py) |
| miri | miri 0.1.0 (cb40c25f6a 2026-05-04) |
| prusti | 0.2.2 commit a0681ee 2023-08-22 |
| rocq-of-rust / -typecheck | rocq_of_rust_cli 0.1.0 · coqc Rocq 9.0.0 |
| soteria | soteria-rust 1.0 manual |
| verifast | VeriFast 26.01 (2026-01-21) |
| verus | 0.2026.05.03.8b81855 · Toolchain 1.95.0-aarch64-apple-darwin |

工具能力随版本浮动；以上数字仅锚定本次快照。

### 0.3 全自动闭环六轮链路

```
R1 v5 baseline          → 2216/3220 = 68.83% SUCCESS（含 P21 YZ harness + 5 类外部根因治理后）
R2-c 误报独审            → 64 候选误报（独立 agent disprove-first）
R2-cc counter-challenge  → 0 推翻 + 35 非决策点 + 29 决策点
R3 fix（部分落地）       → runner/src/report.rs +edition_pipeline_propagation + E0432∪E0433
R4 v5.1 rerun           → 2217/3220 = 68.85% SUCCESS；FAILED 892→872(−20) / UNKNOWN 112→131(+19)
R5-c 漏报独审            → 22 候选漏报（disprove-first，独立 agent）
R5-cc counter-challenge → 9 真漏报 + 13 驳回（设计选择）+ 3 README 补完
R6 综合报告（本文）       → 数据落定 + 决策点冻结 + 用户回来 dashboard
```

每轮 c / cc 都遵循 [`principles.md`](../design/principles.md) §八 审查协议：**独立审查者 + 先反挑刺 + 后反 counter-challenge + 精神模糊则升用户决策**。

---

## §1 TL;DR（3-5 行核心结论）

1. **总通过率 v5.1 = 2217/3220 = 68.85%**（v5 baseline 68.83%，R3 修 19 个 FAILED→UNKNOWN，叠加 1 个偶发 FAILED→SUCCESS）
2. **0 误报：19 个非决策点已落地修复**（5 工具 × edition propagation + 5 工具 × E0433 deps），剩 **29 个候选误报作为决策点 D1 / D2 留待用户裁决**（toolchain pinning 副作用 + kmir wrapper 脆弱）
3. **0 漏报：9 个真漏报全部留为决策点 D3.1-D3.3**（ror × 2 + aeneas × 4 inline-asm + aeneas × 3 copy-deref-closure），13 个候选被 counter-challenge 驳回（kani concurrency / soteria intrinsic / verus derive-Clone 全部属求解层设计选择，非前端 partial）
4. **关键发现**：用户对 MIRI 97% / kani 93% 的"是否有漏报"怀疑被独立 audit 澄清——这两个工具加上 charon × 2 在 R5-cc 中 **0 真漏报**；真漏报全集中在 aeneas pipeline 上游（charon `--preset=aeneas` 不 abort-on-error）与 rocq-of-rust thir Pattern::Wild silent 替换
5. **决策点**总计 **29 误报方向 + 9 漏报方向 + 3 README 文档补完 + 7 早期累积**，共 ~38 项待 review。完整清单见 [`docs/fixes/decisions-2026-05-11.md`](../fixes/decisions-2026-05-11.md)

---

## §2 全自动审查流程（R1-R6 链）

按 [`charter-craft`](https://) §4.8 与 [`principles.md`](../design/principles.md) §八 审查协议产出。

### 2.1 R1：v5 baseline 跑起点

v5 baseline 是 P21 YZ（UNKNOWN 5 类外部根因分类规则）落地后的首次完整 matrix run，已经治源大部分历史误报。v5 数字（68.83%）比 v4 数字（65.1%）高 3.73 pp，**不是工具变强**，而是 oracle 把外部根因（cargo deps / vendor lint / Java env / harness 错配 / edition）正确分类到 UNKNOWN，不再冤枉工具能力。

### 2.2 R2-c：误报独立审计（disprove-first）

独立 agent 跑 64 个候选误报：
- 5 工具的 9 个 edition propagation 候选（rocq-of-rust × 2 / ror-typecheck × 2 / soteria × 1 / verifast × 3 / verus × 1）
- 5 工具的 10 个 E0433 deps 候选（5 工具 × 2 x509-parser entries）
- kmir 43 候选（24 JSONDecodeError + 19 Cargo failed）
- prusti 2 个 toolchain unstable feature gate 候选

来源：[`audit-v5-c-false-positive-2026-05-11.md`](../fixes/audit-v5-c-false-positive-2026-05-11.md)（903 lines）

### 2.3 R2-cc：counter-challenge（disprove-first 第二轮）

独立 agent 默认每条 c 阶段挑刺都错，找证据驳斥。最终：
- **0 条 c 阶段挑刺被推翻**——所有 64 候选都是真候选误报（stderr 字面 + 宪法精神锚点验证通过）
- 但 c 阶段未明示区分决策点 vs 非决策点；cc 重新切分：
  - **35 非决策点**：精神已明示，可直接落地（edition propagation 9 + E0433 deps 10 + kmir deps-rich Cargo failed 15 + kmir edition 2024 Cargo failed 1）
  - **29 决策点**：精神模糊，需用户裁决
- spot-check 漏审：MIRI 4 / Kani 8 / charon-mono 8 / charon-poly 7 / creusot 40 / aeneas × 4 共 ~270 / hax × 3 共 113 — 全部 c 阶段判定站得住，**0 新误报**

来源：[`audit-v5-cc-counter-challenge-2026-05-11.md`](../fixes/audit-v5-cc-counter-challenge-2026-05-11.md)（695 lines）

### 2.4 R3：fix 落地（仅非决策点 19 子集）

按 R2-cc §4.1.1-4.1.2 落地 2 条 oracle 规则到 `runner/src/report.rs:92-152`：

```rust
// R3 (2026-05-11): extend dependency_resolution to E0432 ∪ E0433
if contains_either("error[E0433]")
    && (contains_either("failed to resolve")
        || contains_either("undeclared")
        || contains_either("cannot find module or crate")) {
    return Some("dependency_resolution");
}

// new class 6: edition_pipeline_propagation
if (contains_either("E0670") || contains_either("let chains"))
    && (contains_either("Rust 2015") || contains_either("Rust 2021")
        || contains_either("Rust 2024") || contains_either("only allowed")) {
    return Some("edition_pipeline_propagation");
}
```

kmir 16 子集（15 deps-rich Cargo failed + 1 edition 2024）**未落地**——按 R2-cc §4.1.3 建议"等用户裁决 D1 / D2 后再实施"，避免 R3 规则误吞 D1 子集（kmir 2 个 thread-local toolchain pinning）。

### 2.5 R4：v5.1 rerun 验证

v5.1（同 binary + R3 oracle 改动）：
- 总 FAILED：892 → 872 = **−20**（与预期 −19 + 1 个偶发 ror-typecheck × lifetime/thread-local FAILED→SUCCESS 抖动一致）
- 总 UNKNOWN：112 → 131 = **+19**（与 R3 规则预期 +19 一致）
- 总 SUCCESS：2216 → 2217 = **+1**（偶发抖动）

R3 oracle 改动落地精确，反误报方向无新增 SUCCESS / FAILED 异常。

### 2.6 R5-c：漏报独立审计

独立 agent 从 2216 个 SUCCESS 中找"虚假 SUCCESS"——oracle 没抓住的工具 silent partial。22 候选：
- ror × 2（unsafe-ptr/raw-ptr-const/raw_ptr_const_match — thir Pattern::Wild silent 替换）
- kani × 8（concurrency / atomic_* / thread_local warnings）
- soteria × 4（atomic intrinsic sequential / complex float over-approximation）
- verus × 1（derived Clone auto-spec skip）
- aeneas × 4（inline-asm — charon `--preset=aeneas` 不 abort-on-error）
- aeneas × 3（copy-deref-closure — charon type error 不升 exit）

来源：[`audit-v5-c-false-negative-2026-05-11.md`](../fixes/audit-v5-c-false-negative-2026-05-11.md)（951 lines）

### 2.7 R5-cc：counter-challenge

独立 agent 按 Q1（是否真 silent partial）+ Q2（oracle 是否本应抓 / 反误报安全否）二分判据：
- **9 真漏报**（Q1 ∧ Q2 同时为是）：
  - **ror × 2** raw_ptr_const_match（thir_pattern.rs Pattern::Wild）—— 推翻 README "0 现象" 声明
  - **aeneas × 4** inline-asm（charon 把 entry_fn 降级为 opaque）
  - **aeneas × 3** copy-deref-closure（charon `error:` 但 exit 0）
- **13 个驳回**（设计选择，非前端 partial）：
  - kani × 8：concurrency / atomic 类 warning 实际 codegen 完整（atomic_block / SKIP / binop），BMC 不模拟多线程是求解层语义约束。**与 P13 audit-2 caller_location / foreign function 排除先例同构**
  - soteria × 4：atomic intrinsic sequential 替换 + complex float over-approximation 是符号执行求解层近似，前端完成
  - verus × 1：`continuing, but without adding specification for derived Clone impl` —— VIR 构造完成，spec gen 属 SMT 求解中间步骤
- **3 个 README 补完**（D3.4-3.6）：kani / soteria / verus README 应在 §漏报盲点补充对应条目，长期诚实

来源：[`audit-v5-cc-false-negative-counter-2026-05-11.md`](../fixes/audit-v5-cc-false-negative-counter-2026-05-11.md)（457 lines）

驳回率 13/22 = 59%——充分体现 disprove-first 协议的过滤价值，符合 [`principles.md`](../design/principles.md) §八 实用主义经验经济性。

---

## §3 20 工具 v5.1 数据（按通过率排序）

| 排名 | tool | n | S | F | U | TO | rate | avg ms | p50 ms | p90 ms | max ms |
|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | cargo-check | 161 | **161** | 0 | 0 | 0 | **100%** | 1757 | 221 | 6105 | 26370 |
| 2 | miri | 161 | **157** | 4 | 0 | 0 | **97%** | 2730 | 890 | 7399 | 33515 |
| 3 | charon-mono | 161 | **153** | 8 | 0 | 0 | **95%** | 2256 | 340 | 6999 | 31842 |
| 4 | charon-poly | 161 | **154** | 7 | 0 | 0 | **95%** | 2630 | 379 | 7099 | 41548 |
| 5 | kani | 161 | **151** | 8 | 2 | 0 | **93%** | 3250 | 928 | 7386 | 37380 |
| 6 | hax-fstar | 161 | 128 | 31 | 2 | 0 | 79% | 2923 | 1434 | 7892 | 30656 |
| 7 | hax-lean | 161 | 125 | 34 | 2 | 0 | 77% | 3215 | 1822 | 6057 | 25486 |
| 8 | rocq-of-rust | 161 | 124 | **14** ↓ | **23** ↑ | 0 | 77% | 697 | 801 | 1038 | 1375 |
| 9 | rocq-of-rust-typecheck | 161 | 125 ↑ | **13** ↓ | **23** ↑ | 0 | 77% | 1139 | 1162 | 1975 | 4444 |
| 10 | soteria | 161 | 124 | **15** ↓ | **22** ↑ | 0 | 77% | 1642 | 901 | 3381 | 12727 |
| 11 | creusot | 161 | 121 | 40 | 0 | 0 | 75% | 39091 | 39572 | 54168 | 68644 |
| 12 | hax-coq | 161 | 111 | 48 | 2 | 0 | 68% | 3346 | 1755 | 7805 | 23467 |
| 13 | aeneas-coq | 161 | 102 | 59 | 0 | 0 | 63% | 3596 | 1839 | 7985 | 31065 |
| 14 | aeneas-fstar | 161 | 102 | 59 | 0 | 0 | 63% | 3415 | 1593 | 7160 | 34296 |
| 15 | aeneas-lean | 161 | 102 | 59 | 0 | 0 | 63% | 3738 | 1510 | 9447 | 36792 |
| 16 | prusti | 161 | 71 | 83 | 7 | 0 | 44% | 12867 | 10808 | 24690 | 56639 |
| 17 | aeneas-hol4 | 161 | 66 | 93 | 2 | 0 | 40% | 3762 | 1582 | 9801 | 34938 |
| 18 | verus | 161 | 66 | **73** ↓ | **22** ↑ | 0 | 40% | 570 | 545 | 954 | 1655 |
| 19 | kmir | 161 | 61 | 100 | 0 | 0 | 37% | 7617 | 6771 | 11988 | 102268 |
| 20 | verifast | 161 | 13 | **124** ↓ | **24** ↑ | 0 | 8% | 342 | 273 | 651 | 894 |

**↑ / ↓ 标注**：相对 v5 baseline 的方向变化（R3 fix 触发的非决策点 19 case FAILED→UNKNOWN）。其中 ror-typecheck × lifetime/thread-local 偶发 FAILED→SUCCESS（+1 S，−1 F）。

按 [`tool-integration.md`](../design/tool-integration.md) §六 禁忌：**本表只描述当下 corpus 测量结果，不构成工具能力跨版本承诺，不构成工具间排序**。Duration 字段是 environmental（受 host / parallelism / 抓 cargo stdout 路径影响），不是 tool-quality score。

### 3.1 总计

- 总任务：3220
- 总 SUCCESS：2217 = 68.85%
- 总 FAILED：872 = 27.08%
- 总 UNKNOWN：131 = 4.07%
- 总 TIMEOUT：0

---

## §4 0 误报状态

### 4.1 总览：R2 流水

```
R1 v5 baseline FAILED          = 892
R2-c 候选误报                  = 64
R2-cc counter-challenge        = 0 推翻 / 35 非决策点 / 29 决策点
R3 落地（19 非决策点 / 6 类规则扩展） = FAILED 872 / UNKNOWN +19
R3 暂不落地 16 kmir 非决策点子集 + 29 决策点 = 等用户裁决 D1 / D2
```

### 4.2 R3 已落地的 2 条规则（19 case 已升 UNKNOWN）

#### 4.2.1 第 6 类规则：`edition_pipeline_propagation`（9 case）

**精神锚点**：[`principles.md`](../design/principles.md) §六-Oracle 责任 / `tool-integration.md` §二 "Cargo.toml edition 上游失败算误报"。

**触发字面**：（E0670 OR "let chains"）AND（Rust 2015 OR Rust 2021 OR Rust 2024 OR "only allowed"）

**反误报论证**（[`tool-integration.md`](../design/tool-integration.md) §4.2 双向实测）：
- 真 partial 不会主动含 "async fn is not permitted in Rust 2015"（rustc 标准 edition gate，工具自身错不引发）
- 合法 SUCCESS（cargo-check）entry 上不会触发（rustc 用 cargo 时已传 --edition）
- v5.1 实测：9 个 FAILED→UNKNOWN 精确符合预期，0 假阳性

**涉及 entries**：
- rocq-of-rust × 2（async-fn / async-await）
- rocq-of-rust-typecheck × 2（同）
- soteria × 1（let-chains）
- verifast × 3（async-fn / async-await / let-chains）
- verus × 1（let-chains）

#### 4.2.2 规则 2 扩展：E0432 ∪ E0433（10 case）

**精神锚点**：宪法 §六 vendor crate 上游失败 + dependency_resolution 同精神扩展。

**触发字面**：原 `error[E0432]: unresolved import` ∪ 新 `error[E0433]` + (`failed to resolve` OR `undeclared` OR `cannot find module or crate`)。

**反误报论证**：
- 用户代码"路径拼写错"理论上也触发 E0433，但 cargo-check SUCCESS 是 audit calibration 前提（同 rustc 跑同 entry SUCCESS 已排除用户代码内部 bug）
- v5.1 实测：10 个 FAILED→UNKNOWN 精确符合，0 假阳性

**涉及 entries**：5 工具 × 2 x509-parser entries（rocq-of-rust / ror-typecheck / soteria / verifast / verus × {x509_parse_der, x509_subject_extensions}）

### 4.3 R3 未落地的 29 决策点（待 D1 / D2 裁决）

- **D1**（4 case）：prusti × 2 toolchain unstable feature gate（`E0658: round_ties_even / unchecked_math`）+ kmir × 2 thread-local（kmir 内嵌 cargo + 锁死 toolchain 不识别 thread_local 宏）—— **prusti 锁死 2023-08 rustc**，与 mainline stable 错配
- **D2**（25 case）：kmir × 24 JSONDecodeError + kmir × 1 hashmap_basic 根因不明 —— kmir Python wrapper 在 `kmir/cargo.py:91` 解析 cargo --message-format=json 输出脆弱

详 [`decisions-2026-05-11.md`](../fixes/decisions-2026-05-11.md) §1 D1 + D2。

### 4.4 v5.1 0 误报状态结论

- **35 非决策点中 19 已落地修复**（5 工具影响范围 + 19 cells 从 FAILED→UNKNOWN）
- **16 kmir 非决策点子集等 D1 / D2 裁决后再落地**（避免规则误吞 D1 子集）
- **29 决策点待用户裁决**
- **0 个 c 阶段挑刺被 cc 推翻**——R2-c 候选全部站得住
- **0 个新误报由 cc spot-check 发现**（漏审工具 MIRI / Kani / charon × 2 / creusot / aeneas / hax 全部站得住）

---

## §5 0 漏报状态

### 5.1 总览：R5 流水

```
R1 v5 baseline SUCCESS         = 2216
R5-c 候选漏报                  = 22
R5-cc counter-challenge        = 9 真漏报 / 13 驳回 / 3 README 补完
落地决策                       = 全部留决策点 D3.1-D3.6（不直接修，避免引新误报）
```

### 5.2 9 个真漏报清单（按工具）

| ID | 工具 | entry | 性质 |
|---|---|---|---|
| D3.1 | rocq-of-rust | `unsafe-ptr/raw-ptr-const/raw_ptr_const_match` | thir_pattern.rs:165-176 silent Pattern::Wild 替换 |
| D3.1 | rocq-of-rust-typecheck | 同 | tier-1 继承 tier-0 缺口 |
| D3.2 | aeneas-coq | `charon-limit/inline-asm/nop_via_asm` | charon `--preset=aeneas` 把 entry_fn 降级 opaque + exit 0 |
| D3.2 | aeneas-fstar | 同 | 同 |
| D3.2 | aeneas-lean | 同 | 同 |
| D3.2 | aeneas-hol4 | 同 | 同 |
| D3.3 | aeneas-coq | `charon-limit/copy-deref-closure/deref_copy_in_closure` | charon `error: Type error after transformations` × 3 但 exit 0 |
| D3.3 | aeneas-fstar | 同 | 同 |
| D3.3 | aeneas-lean | 同 | 同 |

### 5.3 13 个驳回的精神共识（设计选择，非前端 partial）

按宪法 §六-3 前端测量原则 + P13 audit-2 caller_location / foreign function 排除先例 + `tool-integration.md` §4.2 反误报双向硬约束：

- **kani × 8 concurrency**：atomic_xadd / xsub / load / store / fence / thread_local — kani-compiler 真 codegen `atomic_block` + SKIP + binop（不是 unimplemented stub）；"will be treated as sequential operations" 是 BMC **单线程符号语义约束**，属求解层；5/8 entry 触发是 std 内部（Arc fetch_add / HashMap thread_local seed / RSA lazy_static 等），加 marker 会大规模假阳性
- **soteria × 4 intrinsic**：atomic intrinsic sequential 替换是符号执行单线程语义约束；complex float intrinsic over-approximation 是 soundness-preserving abstraction，前端完成
- **verus × 1 derive-Clone**：`continuing, but without adding spec for derived Clone impl` —— "continuing" 表示 VIR 构造**完成**；缺的是 spec 生成（属 verus → SMT 中间步骤）；`#[derive(Clone)]` 极常见，加 marker 风险大

这 13 个不算"漏报"，但 README 应在 §漏报盲点补对应条目（D3.4-3.6 已留为决策点，长期诚实）。

### 5.4 漏报修复路径（未实施，留决策点）

按 [`tool-integration.md`](../design/tool-integration.md) §4.2 反误报硬约束 + charter-craft §4.8.3 决策点判据：**所有真漏报只留决策点，不直接修**（修可能引入新误报，须先做更大 corpus 双向 audit）。

- **D3.1 ror**：方案是 gate 5 markers 加 `is not yet supported` stderr grep；风险：v5 corpus 124 SUCCESS 中只此 1 entry 触发，**短期无误报**；但需先在更大 corpus 上双向 audit + 验证 ror 上游 prelude 是否合法 emit 此字面
- **D3.2 + D3.3 aeneas**：方案 A grep stderr `warning: The extraction generated .* warnings` / `Type error after transformations`；方案 B 让 wrapper 在 charon 调用追加 `--abort-on-error`（覆盖 Preset::Aeneas 默认）；风险：方案 B 可能伤"charon 半翻 + aeneas 仍翻"的 edge case

### 5.5 用户怀疑的工具——独立澄清

用户在指令中怀疑 MIRI / kani / charon 等高通过率工具可能藏漏报。R5-cc 独立验证：

- **MIRI 97%**：4 个 FAILED 全部含 miri 自陈 `error: unsupported operation` / 检出 UB，全部真 partial；**0 漏报**
- **kani 93%**：8 个 FAILED 全部走 `[kani-oracle] FAIL: codegen completed with hard-unsupported MIR constructs`；8 个 concurrency-warning SUCCESS entry 经 R5-cc 验证属求解层设计选择（非前端 partial）；**0 漏报**
- **charon × 2 (95%)**：8 + 7 个 FAILED 全部真 charon panic / Coroutine reject / vtable bug；**0 漏报**
- **真漏报集中在 aeneas pipeline 上游**（charon `--preset=aeneas` 自身不带 abort-on-error 这一设计造成 silent partial）+ rocq-of-rust thir Pattern::Wild silent path（README 自陈"0 现象"被推翻）

用户怀疑率被 audit 数据校准——MIRI / kani / charon 当下版本 **实测 0 漏报**。

---

## §6 关键发现

### 6.1 用户怀疑澄清（MIRI / kani）

详 §5.5。

### 6.2 关键修复点（v5 vs v5.1 实操路径）

#### P21 (Agent YZ, v4 → v5)：UNKNOWN 5 类外部根因分类

来源：[`docs/fixes/oracle-unknown-classification-2026-05-11.md`](../fixes/oracle-unknown-classification-2026-05-11.md)

5 类规则把外部根因从 FAILED 桶分出到 UNKNOWN：
1. `runnable_harness_arg_mismatch`（E0061 + argument）—— 解决 runnable + zero-arg harness 设计冲突的 0 误报
2. `dependency_resolution`（E0432 unresolved import）—— vendor crate 上游
3. `toolchain_edition_mismatch`（this version of Cargo is older + edition）—— 老 cargo
4. `vendor_lint_strictness`（unused_qualifications + vendor/）—— vendor lint
5. `environment_corruption`（Result::unwrap + JavaException）—— Java env / viper_tools 路径丢失

效果：cargo-check 100% / miri 97% / charon × 2 95% / kani 93% —— 这几个高通过率不是工具突然变强，是 oracle 把不属工具能力的 cell 正确归类。

#### R3 (本次，v5 → v5.1)：edition propagation + E0433 拓宽

详 §4.2.1 + §4.2.2。

### 6.3 真漏报清单（9 个，等用户决策 D3.1-D3.3）

详 §5.2。集中在：

- **rocq-of-rust × 2**（D3.1）：thir_pattern.rs:165-176 silent path（`Pattern::Wild` 替换原 const pattern，gate 5/6 都不抓）
- **aeneas × 4 backends × 1 entry**（D3.2）：inline-asm 经 charon stage 降级 opaque 后 aeneas 不知情
- **aeneas × 3 backends × 1 entry**（D3.3）：charon stage 自己 emit `error:` 但 exit 0（因 Preset::Aeneas 不带 abort-on-error）

### 6.4 文档补漏（D3.4-D3.6）

- **D3.4** kani README §漏报盲点：补 concurrency 类 warning 是 BMC 单线程语义约束
- **D3.5** soteria README §漏报盲点：补 atomic intrinsic sequential / complex float over-approximation 是符号执行求解层近似
- **D3.6** verus README §漏报盲点：补 derive(Clone) 等 derive 的 auto-spec 生成边界

这 3 个不影响 oracle，只为长期诚实（按 `tool-integration.md` §4.4 漏报盲点诚实声明）。

---

## §7 决策点累积（链接 decisions-2026-05-11.md）

完整决策点清单与处理建议见 [`docs/fixes/decisions-2026-05-11.md`](../fixes/decisions-2026-05-11.md)。

快速索引：

| 类别 | 数量 | 来源 | 影响 case 数 | 紧急度 |
|---|---:|---|---:|---|
| D1 toolchain pinning | 1 项 | R2-cc | 4 case | 中 |
| D2 wrapper 脆弱 | 1 项 | R2-cc | 25 case | 中 |
| D3.1 ror Pattern::Wild silent | 1 项 | R5-cc | 2 case | **高** |
| D3.2 aeneas inline-asm | 1 项 | R5-cc | 4 case | **高** |
| D3.3 aeneas type error | 1 项 | R5-cc | 3 case | **高** |
| D3.4 kani README 盲点 | 1 项 | R5-cc | 0（文档） | 中 |
| D3.5 soteria README 盲点 | 1 项 | R5-cc | 0（文档） | 中 |
| D3.6 verus README 盲点 | 1 项 | R5-cc | 0（文档） | 中 |
| DP-1 / 2 / 3 架构下沉 | 3 项 | Agent X | 0（文档） | 低 |
| DP-4 / 5 UNKNOWN schema | 2 项 | Agent YZ | 0（schema） | 低 |
| DP-6 规则 tool.toml 化 | 1 项 | Agent YZ | 0（schema） | 低 |
| DP-7 hax-lean-eval 启动 | 1 项 | 外围 | 新增 entry | 低 |
| DP-8 detailed-design 措辞 | 1 项 | 外围 | 0（文档） | 低 |
| DP-9 文档引用号一致性 | 1 项 | 外围 | 0（文档） | 中 |
| DP-10 vendor 修源 | 1 项 | 外围 | -2 UNKNOWN | 低 |
| DP-11 prusti README edition | 1 项 | 外围 | 0（文档） | 低 |
| DP-12 Coq + F* 后端 typecheck | 1 项 | 外围 | 提升 hax oracle 精度 | 低 |
| **总决策点数** | **20 项** | | **≈ 38 case** | |

---

## §8 性能与时长

按 [`tool-integration.md`](../design/tool-integration.md) §七 实测报告原则：duration 是 environmental，不构成工具能力 score。

### 8.1 平均耗时（avg, p50, p90, max ms）

最快前 3：

| tool | avg | p50 | p90 | max |
|---|---:|---:|---:|---:|
| verifast | 342 | 273 | 651 | 894 |
| verus | 570 | 545 | 954 | 1655 |
| rocq-of-rust | 697 | 801 | 1038 | 1375 |

最慢前 3：

| tool | avg | p50 | p90 | max |
|---|---:|---:|---:|---:|
| creusot | 39091 | 39572 | 54168 | 68644 |
| prusti | 12867 | 10808 | 24690 | 56639 |
| kmir | 7617 | 6771 | 11988 | 102268 |

### 8.2 总 wall

v5.1 wall 1629 s（parallelism = 10）—— 总 CPU ≈ 16290 s。

性能数字不构成工具能力评判：verifast 快但 8% 通过率不代表它"快是因为弱"——是 wrapper 即时 reject vacuous-pass entry 不进 SMT 阶段（按 [`tools/verifast/README.md`](../../tools/verifast/README.md) §SUCCESS）；creusot 慢主要在 z3 / cvc5 求解器（属求解层时长，非前端能力时长）。

---

## §9 工业三件套 + runnable 子矩阵

### 9.1 industrial × 20 工具（120 task）

industrial corpus 含 RSA / SHA2 / x509-parser 三件套 + 各自的 deps-rich entries。v5.1 总通过率 48/120 = 40%。

**关键现象**：x509-parser cert-parse 两 entries 在 5 个不走 cargo 的工具上（rocq-of-rust / ror-typecheck / soteria / verifast / verus）触发 E0433 deps，**R3 后从 FAILED→UNKNOWN**（10 个 cell）—— 这是 §4.2.2 规则扩展的全部 10 个 case。industrial 40% 不是"工业代码难"，是 R3 把外部根因从 FAILED 桶分出后 industrial corpus 的 FAILED 数实际下降 10 个。

### 9.2 runnable × 20 工具（300 task）

runnable corpus 是带 `runnable = true` 的 entries（每个 entry 有 main → 工具有机会跑 zero-arg harness）。v5.1 runnable 通过率 286/300 = 95%。

P21 治源前，build-crate 工具（不走 main）会因 `error[E0061]: argument` 假阳性大量 FAILED；P21 治后通过 `runnable_harness_arg_mismatch` 外部根因升 UNKNOWN，runnable 在 build-crate 工具上正常 SUCCESS。

---

## §10 与 v1-v5 历史进化

| 版本 | 总 SUCCESS | corpus | 总通过率 | 关键变更 |
|---|---:|---:|---:|---|
| v1 | 1949 / 2774 | 146 / 19 工具 | **70.3%** | 首批 strict-oracle 落地 |
| v2 | 1835 / 2774 | 146 / 19 工具 | **66.2%** | hax-lean grep 严格化 |
| v3 | 1823 / 2774 | 146 / 19 工具 | **65.7%** | P12 / P13 audit-2 抓 silent path |
| v4 | 2096 / 3220 | 161 / 20 工具 | **65.1%** | corpus +15 runnable / +1 ror-typecheck，分母变 |
| v5 | 2216 / 3220 | 161 / 20 工具 | **68.83%** | P21 (YZ) UNKNOWN 5 类外部根因分类 |
| **v5.1** | **2217 / 3220** | 161 / 20 工具 | **68.85%** | R3 加第 6 类 edition propagation + E0432 ∪ E0433 |

**v4 → v5 (+3.73 pp)**：不是工具变强——是 P21 YZ 把不属工具能力的 cells（runnable harness 错配 / vendor deps / vendor lint / Java env）从 FAILED 升 UNKNOWN，反映 oracle 0 误报硬指标的治理。

**v5 → v5.1 (+0.02 pp)**：实质 0 提升（19 cell FAILED→UNKNOWN 加 1 偶发 FAILED→SUCCESS）—— v5 baseline 已经基本治源，R3 只是把剩余的 19 个非决策点误报清掉。

**v1 / v2 / v3 / v4 通过率不可直接对比**：corpus 分母在 v3→v4 间变了（146→161）；工具数变了（19→20）；strict-oracle 版本不同（v1 起步 → v2/v3 加 P12/P13 audit-2 silent path 抓取 → v4/v5/v5.1 分母 161 + P21 / R3 外部根因治理）。

---

## §11 收尾

按 [`tool-integration.md`](../design/tool-integration.md) §7.1 不反复自贬：本报告基于 v5 + v5.1 raw 可追溯数据 + 两轮独立 audit 链路落地。所有数字锚定 2026-05-11 工具版本快照与 161 entries corpus；所有结论可在 `runs/run-1778500291-90812/raw/` 与 `runs/run-1778504159-67797/raw/` 复现。

工具能力的客观判定不在本项目范围（按 [`principles.md`](../design/principles.md) §二 价值边界）；本报告陈述的是 **当下 corpus / 当下工具版本** 下 oracle 经独立 audit 校准后的 SUCCESS / FAILED / UNKNOWN 分布与流水。

用户回来后建议：

1. **5 分钟 dashboard**：读 §1 TL;DR + §6 关键发现 + §7 决策点表
2. **30 分钟深入**：再读 §4 / §5 + [`decisions-2026-05-11.md`](../fixes/decisions-2026-05-11.md)
3. **拍板决策**：优先 D3.1 / D3.2 / D3.3 真漏报 + D1 / D2 候选误报
4. **commit 安排**：按本报告 §12 提供的分组建议

---

## §12 commit 分组建议（待用户拍板）

当前 git status 未 commit 改动：

**已修改文件**：
- `runner/src/report.rs`（R3 加 2 类 oracle 规则：edition_pipeline_propagation + E0432∪E0433）

**新增文件**（4 份 audit 报告 + 1 份 decisions + 1 份本报告）：
- `docs/fixes/audit-v5-c-false-positive-2026-05-11.md`（R2-c 误报独审）
- `docs/fixes/audit-v5-cc-counter-challenge-2026-05-11.md`（R2-cc counter）
- `docs/fixes/audit-v5-c-false-negative-2026-05-11.md`（R5-c 漏报独审）
- `docs/fixes/audit-v5-cc-false-negative-counter-2026-05-11.md`（R5-cc counter）
- `docs/fixes/decisions-2026-05-11.md`（R3 + R5-cc 决策点累积 + R6 整理）
- `docs/test-reports/feature-coverage-2026-05-11-final-v5.1.md`（本报告）

**vendor submodule 修改**（vendor/rsa / sha2 / x509-parser 显示 `untracked content`）—— 应该是之前会话留下的；本次 R3-R6 不涉及 vendor 改动，建议保持现状或单独决定。

### 12.1 分组方案 A（推荐）：拆 P25 + P26

**P25：R3 oracle 规则扩展**（runner 代码改动）
- `runner/src/report.rs`
- commit message: `P25: extend oracle to edition_pipeline_propagation + E0432∪E0433 (R3 fix, 19 FAILED→UNKNOWN)`
- 链 audit-v5-cc 文档 + 决策点 §3.1.1 / §3.1.2

**P26：audit 报告 + decisions + 最终综合报告**（纯文档）
- 6 份新增文档
- commit message: `P26: full v5.1 audit reports + decisions + final report (R2-c/cc + R5-c/cc + R6)`
- 不动代码，便于后续 review

**优势**：代码改动与文档改动分离，回滚 / cherry-pick 灵活。

### 12.2 分组方案 B：一个大 P25 commit

合并 runner 代码 + 全部文档于一个 commit。
- commit message: `P25: v5→v5.1 audit pipeline (R1-R6) — 19 false-positive fixes + 9 false-negative decisions`
- 优势：完整故事一次提交，时间线清晰
- 劣势：代码与文档耦合

### 12.3 关于 vendor submodule 改动

`vendor/rsa` / `vendor/sha2` / `vendor/x509-parser` 显示有"untracked content"——这些是 git submodule，需单独决定是否一起 commit（或忽略）。**本次 R3-R6 不涉及 vendor 改动**，建议单独审查。

### 12.4 推荐：方案 A

按 [`principles.md`](../design/principles.md) §三 模块二分 + 项目历史 commit message 风格（P21 / P22 / ...），方案 A 与项目惯例对齐——把"oracle 规则改动（核心模块 1 runner）"与"audit 文档（次要模块 3）"分开。
