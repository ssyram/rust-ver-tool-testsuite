# ISSTA 2026 Reviewer 审查 — c 轴 E：Conclusion Validity + Threats to Validity 完备性

> 角色：ISSTA 2026 reviewer，按 Wohlin Ch.8.5 四类 ToV（construct / internal / external / conclusion）。
> 范围：v6 final（commit `ebe6858`）。
> 立场：disprove-first。R5/R6/R8/R11 反状态过滤。

---

## 0. TL;DR（reviewer 视角）

| # | 检查项 | 结论 | 严重度 |
|---|---|---|---|
| 1 | 主报告缺独立 "Threats to Validity" 章节 | **成立**（散落 §6/§9 不够标准）| **major**（论文 blocking）|
| 2 | 时空锚定层级 | 总体锚定到位；个别数字未附 run id 上下文 | minor |
| 3 | 20 工具漏报盲点 cross-tool 一致性 | **三档措辞参差实质合理**（按工具机制不同），但**缺一张矩阵图**说明分档 | minor → major |
| 4 | 过强声明（cargo-check "形式可证"）| 已加注脚，**站得住但措辞可加 "engineering proof" 标签** | minor |
| 5 | 运行次数 / 重复性 | **major**：单次 run，仅 ror N=7；其他工具未做 multi-run 噪声估计 | **major** |
| 6 | 样本量 161 / 41 features | **缺代表性论证**（covering criterion / feature taxonomy 引用）| **major** |
| 7 | 统计学（无 CI / 跨工具差异 significance）| **major**：所有 % 无 CI，跨工具排序无显著性检验 | **major**（ISSTA 必问）|

**总体判定**：本报告**论文化前需补独立 ToV 章节 + 重复性论证 + 统计学最小补充**（Wilson CI / McNemar）。当前形态是**工程报告**，不是 ISSTA 论文形态。

---

## 1. 检查项 1：缺独立 "Threats to Validity" 章节

**事实**：`feature-coverage-2026-05-12-v6.md` 全文 grep `threat|validity` 命中 0。相关声明散落在：
- §0.2 "corpus 与工具版本"（construct/external 锚定）
- §3.2 "v6 UNKNOWN 10 个全列"（construct，归因 vendored lint）
- §6.1 "修宪后语义变化"（construct，语义飘移说明）
- §9 "不再适用的旧报告"（external/historical 锚定）

**reviewer 立场**：ISSTA 评审会 ctrl-F "Threats to Validity"——不到独立章节会扣分。R8 流程建议级别：**non-blocking for 工程 doc，blocking for 论文化**。

**反挑刺自查**：是否本就不打算论文化？— 但项目 README + audit/ 目录里多处暗示对学术报告标准上靠（"宪法 §六 时空锚定"语言、c+cc disprove-first 协议直接对应 Popper / Wohlin）。判定：**应补**。

**最小补充建议**（non-决策，只是 ISSTA reviewer 提议）：
```
## §X Threats to Validity
### X.1 Construct（数字是否度量了 "工具能力"）
- v6 删 3 条 oracle UNKNOWN 规则后，FAILED 定义变了——和 v5.1 通过率不可直接比；§3.1 ΔS/ΔF/ΔU 守恒分解已部分对冲，但 reviewer 可能仍质疑 "通过率 68.39%" 与 "工具能力" 的 construct 距离
- 前端测量切割（§六-3）排除求解层——soteria/verus/kani SUCCESS 不蕴含 verified；这是构造定义而非缺陷，但需声明
### X.2 Internal（因果 / 混淆）
- wrapper bug 与工具能力混淆 → c 路 0 误报 audit 已 grep `unbound variable` / `TS_*: not set` 等封堵
- 环境损坏（JVM / /tmp）与工具能力混淆 → UNKNOWN (a)/(b) 严格语义 + 治源迁移
### X.3 External（推广性）
- 161 entries / 41 features 不覆盖完整 Rust feature space；async / proc-macro / build-script 缺
- 工具版本快照锚定（每个工具一行 version）→ 升级后衰减不可避免
### X.4 Conclusion（统计 / 归纳）
- 单次 run（ror 除外 N=7）；其他工具未估计 run-to-run 方差
- 通过率无 Wilson / Clopper-Pearson CI；跨工具差异无 McNemar / paired bootstrap
- 41 features × ~4 entries 不构成 feature-level 显著性比较
```

---

## 2. 检查项 2：时空锚定（**reviewer 满意度 7/10**）

**已到位**：
- §0.1 三行 run id + ISO 时间（包括 verus rerun + R7 aeneas/ror-tc rerun）
- §0.2 corpus 锚定 161 entries × 41 features
- principles.md §六 "时空锚定" 明文：(工具版本, 时间) 二元组
- 各 cc-report 的 "时长分布 / 时效声明" 段（如 ror.md L11）

**未到位**：
- §3 表里每个工具版本号未列在主报告（cc-reports 里有，但读者要跳）
- §1 TL;DR 68.39% 无脚注链回 §0.1 run id
- §6.0 "v6 最终数据" 列 R7 后数字但未明示 R7 commit hash（§8 commit 链有 P30 但用 `<P30>` 占位符——R11 文案 vs 实现 stale 嫌疑）

**反挑刺**：R6 "局部 stale vs 全局缺"——这是局部 stale（commit hash 占位）而非全局缺，落 minor 类。

---

## 3. 检查项 3：20 工具漏报盲点 cross-tool 一致性

**实测分档**（grep "漏报盲点" 全部 README）：

| 档 | 措辞 | 工具 | 数量 |
|---|---|---|---|
| A "无 / 形式可证" | cargo-check / charon-mono / charon-poly / miri / creusot / kmir | 6 |
| B "无（理论窗口 + 实测 0 现象）" | prusti | 1 |
| C 具体列盲点（诚实声明）| aeneas×4 / hax×3 / verus / soteria / ror / ror-tc / verifast / kani | 12 |
| D "无已知 / 仅实测" | charon×2（重叠 A）| — |

**reviewer 视角**：三档措辞是否一致？— **实质合理但表面参差**：
- cargo-check / miri 档 A 站得住（rustc / MIRI 设计上无 silent skip 路径，单一 exit 通路）
- charon×2 档 A 偏弱（cc-report L37/L40 写 "形式可证 单一通路"，但 R5 audit 历史曾削弱过相关声明——需 cross-check 是否所有地方都同步）
- kmir 档 A 但 cc-report L49 自承 "**软指标**"——README "盲点：无" 与 cc-report "软指标" **存在不一致**（R5 已废弃文本同步性问题）

**reviewer blocking**：cc-report 与 README 之间档差是论文 reviewer 必抓的点。建议加一张表 in 主报告：

```
| 工具 | oracle 严格度档 | 依据 |
|---|---|---|
| cargo-check | engineering-proof (rustc 单一信号) | cc-report §形式严格性 |
| miri / charon×2 | implementation-grounded (实测 + 设计审视) | … |
| aeneas / hax / ror / verus / soteria / kani / kmir / verifast / prusti / creusot | empirical + wrapper-gated | … |
```

这能一次性消除 cross-tool 参差感。

---

## 4. 检查项 4：cargo-check "0 误报 / 0 漏报均形式可证"

**事实**：cc-reports/cargo-check.md L47-51 + tools/cargo-check/README.md L26 写 "形式可证"。
**L51 注脚**："此处'形式可证'指可溯源到 rustc 源码层的单一信号通路 + 无 silent skip 设计；非项目自陈，rustc 的 error 模型是 Rust 官方的设计约束。"

**reviewer 判**：这是 **engineering proof / design-level guarantee**，不是 source-level mechanized proof。注脚已**降级了措辞强度**，站得住。但在 ISSTA 论文里建议改用 "by-design no-silent-skip"（避免与定理证明的 "formally proved" 混淆）。

**R11 文案 vs 实现一致性**：cargo-check 实测 161/161 SUCCESS，符合 "无 silent skip" 设计推断——文案没超过实现。**non-blocking**。

---

## 5. 检查项 5：运行次数 / 重复性（**ISSTA 必抓 major**）

**事实**：
- v6 主跑 1 次 + verus 单独 rerun 1 次 + R7 5 工具 rerun 1 次。每个 (tool, entry) 测量基于**单次观察**
- 唯一多 run 工具：ror（wrapper 内 N=7-attempt AND-reduce）—— 但这是 oracle 内部对**单次 task** 的反非确定性手段，**不是** run-to-run 重复性估计
- 其他工具未做 multi-run 噪声 / variance 估计

**reviewer 立场**：
- ISSTA 标准做法：每 (tool, entry) 跑 3-5 次，报告 majority vote 或 unanimous-pass
- 单次 run 风险：(a) 工具内部非确定性翻译 (ror) 已被 N=7 cover，但 (b) 求解器 timeout 边界 (kani / verus / prusti / soteria) 未 cover——一次跑到 timeout 边界的 entry 下次可能过
- 当前 timeout 120s 远高于 max 1657ms（ror）——但其他工具的 wall-time 分布未在主报告呈现

**最小补充建议**：
1. 抽 ~10% 高风险 entries（涉及 SMT / BMC 工具的 SUCCESS）做 5× rerun，估 flip rate
2. 主报告 §X.4 conclusion validity 声明 "单次 run / N=7 仅限 ror / 其他工具假定确定性"

**反挑刺**：是否本项目 v1 范围声明排除？— principles.md §七 "本项目 v1 只解决功能层面问题。性能、资源用量、缓存效率不在 v1 范围"——但**重复性不是性能问题，是 conclusion validity 问题**，宪法未排除。**blocking 类**。

---

## 6. 检查项 6：样本量 / feature 分布

**事实**：161 entries × 41 features ≈ avg 4 entries/feature。examples/ 目录列 41 feature 子目录覆盖：trait / lifetime / unsafe / closure / iter / float / generic / hrtb / gat / impl-trait / drop / panic / concurrency / collections / industrial / *-limit 等。

**reviewer 立场**：
- **覆盖了什么**：Rust 1.x 主流 feature 大类齐全，包括 GAT / HRTB / impl-trait / async-adjacent 部分
- **缺什么（reviewer 会问）**：
  - async/await（grep 无 `examples/async/` 目录）
  - proc-macro / build-script
  - cargo workspace / feature flag 组合
  - FFI / extern "C"（unsafe 下可能有但未独列）
- **代表性论证缺失**：项目无 "我们的 corpus 凭什么代表 Rust feature space" 的方法学论证。没有引用 Rust reference / Rustonomicon / RFC 列表作为 covering criterion

**最小补充建议**：在 §X.3 external validity 引用一份 feature taxonomy（如 Rust Reference TOC）+ 给一张矩阵说明 covered / not-covered。

---

## 7. 检查项 7：统计学（**ISSTA 必抓 major**）

**事实**：所有通过率以裸百分比给出，无 CI / 无显著性检验：
- 总体 68.39% — 无 CI
- 跨工具 hax-fstar 79% vs hax-lean 77% — 无 paired test
- v5.1 → v6 ΔS = -15 — 守恒论证但无 "这个变化是否系统性 vs 噪声" 表述

**reviewer 立场**：
- 161 个 binary outcome 的 Wilson CI 是 30 秒计算
- 同 corpus 同 entries 的跨工具比较应用 **McNemar's test**（paired binary）
- 即使作者立场是 "不做工具能力评判 / 不做排序"（README 顶部免责 + memory/feedback_no_depth_distinction.md），**% 数字一旦写出来就构成 implicit ranking**——reviewer 会问 "你不做排序为什么报 %?"

**反挑刺**：可不可以说 "我们是工程报告不是论文，不需要统计学"？— 不行。报告自陈"对外可见的工具能力筛查"（principles.md §一）——一旦对外，% 数字的不确定性必须量化。**blocking**。

**最小补充**：在 §X.4 加：
- 每工具通过率附 Wilson 95% CI
- 同族工具（如 aeneas-coq/fstar/lean/hol4 / hax-coq/fstar/lean / charon-mono/poly）跨 backend 差异做 McNemar
- 一句话声明 "工具间排序不是本报告目标；% 仅供 single-snapshot 描述用，跨工具 % 差异不蕴含能力优劣"

---

## 8. 反状态盘查

| 反状态 | 命中 |
|---|---|
| R5 已废弃文本（"形式可证"被 P30 cc-route 部分削弱后是否同步）| **部分命中**：kmir README "盲点：无" vs cc-report "软指标" 未同步；其他 19 工具基本同步 |
| R6 局部 stale vs 全局缺 | **命中**：主报告 §8 commit 链 `<P30>` 占位符；不影响实质 |
| R8 流程建议 ≠ blocking | 本审查 7 项中：1/5/6/7 = blocking（论文化前必补）；2/3/4 = non-blocking |
| R11 文案 vs 实现 | cargo-check / aeneas R7 修订 / ror N=7 — 实测站得住，文案没超 |

---

## 9. ISSTA reviewer 综合判

**accept 类**：宪法 §六 "时空锚定 / 不冤枉 / 不藏"、c+cc disprove-first audit 协议、UNKNOWN 严格语义、wrapper gate 双通路封堵——**这些是 SE 实证研究的良好实践，应在论文中作为 contribution 之一陈述**。

**reject-without-revision 类**：无。

**major revision 类**（论文化前必补）：
1. 独立 §Threats to Validity 章节（Wohlin 4 类）
2. 重复性论证（至少 10% sample 多 run）
3. 通过率 Wilson CI + 同族跨 backend McNemar

**minor revision 类**：
- §8 commit 链占位符替换
- 主报告加一张 oracle 严格度档表（消除 cross-tool 措辞参差）
- 加 covering-criterion 论证（async / proc-macro 缺口声明）

---

## 10. 决策点 vs 非决策点

按 principles.md §八-决策点判据：

| 项 | 决策点？ | 理由 |
|---|---|---|
| 是否加独立 ToV 章节 | **决策点**（涉及报告对外形态——工程报告 vs 论文化）| 用户裁 |
| 是否做 multi-run 重复性 | **决策点**（成本 vs conclusion validity） | 用户裁 |
| 是否加 Wilson CI / McNemar | 非决策点（30 秒计算，无成本）| 直接补 |
| kmir README "盲点：无" 与 cc-report "软指标" 不一致 | 非决策点（文案 stale 同步）| 直接修 |
| commit hash 占位符 `<P30>` | 非决策点（笔误）| 直接补 |

---

end.
