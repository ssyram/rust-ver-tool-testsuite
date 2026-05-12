# Axis B counter-challenge: 宪法 → 一级法律 disprove-first 复审

> counter agent：独立于 c 路 audit agent
> 基准：principles.md v8 (145 行，P27 修订)
> 对象：c 路 9 候选（强 4 / 弱 4 / 自身排除 1）
> 协议：workflow-audit §八 disprove-first + R1-R12 反状态清单

---

## 候选 1：tool-integration 9 处章节号错引

**c 路挑刺**：L3/L5/L19/L32/L40/L65/L141/L147 等引"§三-3-原则 3""§三-3-2.a/b/c""§五 工具非静态原则""§六-本节自我性声明"，v8 结构里全无对应章节。

**逐位对照**：
- L3 "§六-本节自我性声明 + §三-3-原则 3" → v8 §二 "生效约束" 段 + §六 (次要模块精神)。**错引证实** ✓
- L5 "§四 三大派生原则" → v8 §四 改为 A/B/C 三条核心原则，"派生"措辞已删。**部分错引** ✓
- L19 "§五 工具非静态原则" → v8 §五 runner 段措辞改为"结果记录严谨全面"，无"工具非静态原则"。**错引证实** ✓
- L32/L40/L65 "§三-3-2.a/b/c 上限/下限" → v8 §六 "Oracle 责任"三子条（不冤枉/不藏/UNKNOWN 严格语义），不再有 2.a/b/c。**错引证实** ✓
- L141 "§三-3-1 时效性" → v8 §六 "时空锚定"。**错引证实** ✓
- L147 "§三-3-原则 3 实测报告责任边界" → v8 无对应小节；最相近落点是 §二 生效约束 + §六 时空锚定。**错引证实** ✓

**counter 角度**：
- R6（局部 stale ≠ 全局缺）：tool-integration 是次要模块文档，runner / examples / report 实现层未受影响——挑刺仅触及"溯源链路"层面 → 部分命中
- R11（文案 ≠ 实现）：错引是 wording 滞后，runner / wrapper 行为本身未失锚——影响降级
- 作者反驳点：宪法 P27 是主动改宪法结构号，下游同步本就是预期跟进项；批量 sed 解决，不属"违反 contract"

**反驳能否消解**：消解不了。R6 与 R11 把"严重性"降级，但不消除"读者无法溯源"这一精神性后果——`principles.md` §八 第 2 条 "审查者 ... 给具体理由 + 文件位置 + 引用"要求引用准确性，错位引用直接破坏溯源。挑刺仍成立。

**判定**：**成立**（强候选，严重性降一级为"文档同步类强候选"）

**fix 操作**：批量替换：
- §六-本节自我性声明 → §二 生效约束段
- §三-3-原则 3 → §六（次要模块精神）
- §四 三大派生原则 → §四 A/B/C 核心原则
- §五 工具非静态原则 → §五 runner 结果记录严谨全面
- §三-3-2.a → §六 Oracle 责任·不冤枉
- §三-3-2.b → §六 Oracle 责任·不冤枉（上限）
- §三-3-2.c → §六 Oracle 责任·不藏（下限）
- §三-3-1 时效性 → §六 时空锚定

---

## 候选 2：architecture L64/168/281 UNKNOWN 旧宽松语义

**c 路挑刺**：L64 / L168 / L281 仍是"runner-internal 错误"口径，与 v8 §六 L111 严格两类语义（全局工具链崩溃 / 我方可识别问题）+ "明确归因 + 会修计划"未对齐。

**对照 v8 §六 L111 原文**：
> UNKNOWN 严格语义：UNKNOWN 只在两类场景使用——(a) 全局工具链崩溃（重装可修），(b) 我们这边可识别的问题且暂未修 ... 每类必附明确归因 + 会修计划。**官方 wrapper 失败 / 工具自选 toolchain 不支持新特性 / 工具单文件 pipeline 不读 Cargo.toml / 官方 wrapper 不传 --edition** 一律 FAILED

**对照 architecture**：
- L64 "runner-internal 错误归 UNKNOWN" → 偏宽：可能把 vendored crate lint / corpus 引入的 IO 错误统统装进；未要求归因
- L168 "任一步 IO 错误时返回 Err(anyhow)，由 main 转为 UNKNOWN" → **直接违反**：v8 明确 IO 错误若非 (a)/(b) 应是实现 bug 不该进 UNKNOWN
- L281 "ResultClass 三分 ... UNKNOWN 仅记 runner-internal 错误" → 与 v8 严格两类不一致

**counter 角度**：
- R11（文案 ≠ 实现）：作者会说 architecture 是文档层，runner 实际行为已对齐 P27——但 c 路 audit 已声明本 Axis 不查实现层，仅查法律层。文档与法律不一致即问题
- R5（历史/废弃）：旧"runner-internal"措辞是 P21 时代 contract，被 P27 收紧。不是 deliberate 当前设计
- 作者反驳点："L64/168/281 应该如何精确改？是改写定义 vs 加 P27 update note"——这是 fix 形式问题不是挑刺是否成立的问题

**反驳能否消解**：消解不了。法律层文档承接宪法是其本职——R11 不能为法律层文档脱困。挑刺成立。

**判定**：**成立**（强候选）

**fix 操作**：
- L64 改写为："按宪法 §六，UNKNOWN 严格语义两类：(a) 全局工具链崩溃，(b) 我们这边可识别但暂未修的问题（均附明确归因 + 会修计划）；subprocess 跑完且非零归 FAILED，包括官方 wrapper 失败 / 工具自选 toolchain 不支持新特性等"
- L168 改写为："任一步 IO 错误时返回 Err(anyhow)，由 main 按 §六 UNKNOWN 严格语义分流——能归 (a)/(b) 即记 UNKNOWN + 归因 + 会修计划，否则视为实现 bug（fail-fast 或上报）"
- L281 表述同步更新；推自栏可加 "+ §六 UNKNOWN 严格语义"

---

## 候选 3：tool-integration 零提"我们 wrapper vs 官方 wrapper"区分

**c 路挑刺**：tool-integration 全文无 "wrapper" 字样，未投影 v8 §六 L111 "官方 wrapper 失败一律 FAILED" 这一新加判定规则。

**对照**：
- principles.md L111 明文区分两类失败链路：我方 harness/wrapper bug → UNKNOWN；官方 wrapper 失败 → FAILED
- tool-integration §三/§四 论 0 误报 / 0 漏报，但未涉及"失败链路归因"

**counter 角度**：
- 作者最强反驳点：宪法 §六 已直接 articulate wrapper 区分；tool-integration 默认 inherits 宪法精神，是否需要在 tier-1 重复声明？
- R11：是否所有宪法新条都必须在 tier-1 显式投影？
- R6：宪法 §六 已在，tool-integration 不显式提，可能不构成"全局缺失"

**反驳能否消解**：部分消解。"重复显式声明" vs "默认继承" 是文档冗余度的选择。**但 tool-integration §三/§四 的具体话题（0 误报 / 0 漏报论证）与 wrapper 区分话题强耦合**——新增工具集成时判 SUCCESS / FAILED / UNKNOWN 的归因落点是此文档的本职任务，未投影即真盲点。

**判定**：**成立**（强候选，但优先级低于候选 1/2）

**fix 操作**：tool-integration §四 加 §四.5（或在 §三 末尾加 1 段）"失败链路归因区分"：
- 我方 harness / wrapper / 模板 bug → UNKNOWN + 归因 + 会修计划
- 官方 wrapper 失败 / 工具自选 toolchain 不支持新特性 / 工具单文件 pipeline 不读 Cargo.toml / 官方 wrapper 不传 --edition → FAILED（本地性原则站得住）

---

## 候选 4：tool-integration §四.1 单一通路 vs Warn channel 自陈

**c 路挑刺**：aeneas craise / charon register_error! 单一通路论证仅覆盖 exit 通路；Warn channel 类工具自陈未涵盖。v8 §六 L109 "工具自陈"我没全干完"必须被尊重"。

**counter 角度**：
- 作者反驳点：单一通路论证就是论"exit 通路上 0 漏报"。Warn channel 不走 exit ⟹ 工具自身未把 Warn 视为失败信号 ⟹ "尊重工具自陈"反而是 oracle 不抓 Warn（工具自陈 Warn 不是 failure）。所以不构成漏报盲点
- 但"工具自陈"语义模糊：是 exit 通路自陈 vs Warning 自陈 vs 产物自陈？
- §四.4 漏报盲点段已隐含覆盖

**反驳能否消解**：能消解。"工具自陈必须被尊重"的精神是不绕过工具自己说"失败"的信号——若工具自身没让 Warn channel 触发 exit ≠ 0，则工具自陈层面 Warn 不是失败。**c 路挑刺过度强读"工具自陈" → "凡 Warn 都是工具自陈失败"**——这是过度解读，宪法精神未明示这一强读。

**判定**：**不成立**（counter 站得住）

**fix 操作**：无须改文档。如果用户希望显式声明可在 §四.1 加 1 行注："单一通路论证仅覆盖 exit 通路。Warn channel 不被纳入 oracle 失败信号——这与"尊重工具自陈"一致（Warn 在工具自身配置中本就不是 failure 信号）"。可选，弱建议。

---

## 候选 5：architecture L287 决策表 "§三-3 诚实测试范围"

**c 路挑刺**：L287 引 "principles §三-3 诚实测试范围"，v8 §三 仅为"模块定位"，无 §三-3。

**对照**：
- v8 §三 = 模块定位（核心 vs 次要二分）
- "0 误报第一"精神现在 v8 §六 "Oracle 责任·不冤枉"

**counter 角度**：与候选 1 同源——章节号错引；R11 文案同步类。

**判定**：**成立**（弱候选，机械同步）

**fix 操作**：L287 推自栏改为 "诚实性宗旨（principles §六 Oracle 责任·不冤枉）"

---

## 候选 6：architecture L288 决策表 "§六-2 不允许 partial"

**c 路挑刺**：L288 引 "§六-2"，v8 §六 无 -2 子条编号。

**对照**：
- v8 §六 "Oracle 责任·不冤枉" L109 明文 "不允许任何 partial / silent skip / 半翻译"
- "-2" 是 v6 / v7 时代的子条编号，已淘汰

**counter 角度**：同候选 5。

**判定**：**成立**（弱候选）

**fix 操作**：L288 推自栏改为 "（principles §六 Oracle 责任·不冤枉）"

---

## 候选 7：§二 Q5 第 4 条 "社区惯例 + 本地性对齐" 零下游投影

**c 路挑刺**：v8 §二 L50 第 4 条 "测量姿势对社区惯例 + 本地性对齐"是 P27 新加，architecture / tool-integration 零吸收。

**counter 角度**：
- 作者反驳点：第 4 条本质是 §一 "不公信"问题的简短复述——已在 §一 三原则栈（本地性/社区惯例/最大善意）展开。下游若已承接 §一 即等价已承接 Q5 第 4 条
- R6：宪法已有，下游是否必须重复
- 但 architecture §一只承接"不公平"侧（A/B/C 三大原则），未承接"不公信"侧 → **与候选 8 重叠**

**反驳能否消解**：部分消解。Q5 第 4 条是 §一 的 Q5 重述，候选 7 的实质内容已被候选 8 涵盖——单独立项冗余。

**判定**：**部分成立**（建议并入候选 8，单独立项移除）

**fix 操作**：合入候选 8 fix。无需独立操作。

---

## 候选 8：architecture §一 推导只承接"不公平"未承接"不公信"

**c 路挑刺**：L11 "三大原则 A/B/C 落到架构层面"——但 v8 §一 双根本问题（不公平 + 不公信）是最高位阶。A/B/C 是"不公平"侧投影；"不公信"侧（本地性 / 社区惯例 / 最大善意）三原则栈在架构 oracle 分类 / UNKNOWN 严格语义 / wrapper 区分上都有具体投影责任。

**对照**：
- v8 §一 L7-33：双根本问题是宪法最高位阶
- §一 L21-25：本地性 / 社区惯例 / 最大善意三原则栈
- architecture §一 L11：仅承接 A/B/C → 三具体设计选择

**counter 角度**：
- 作者反驳点：架构 §一推导是"原则到结构"映射；"不公信"侧主要在 oracle 行为（§六 Oracle 责任），不在 architecture 模块切分。架构 §一 不承接"不公信"是恰当的范围切分？
- 但 architecture §一 §二.B 扩展（L62-64 ResultClass 三分）+ UNKNOWN 语义（L168/281）确实是"不公信"侧投影——只是没明示宪法溯源

**反驳能否消解**：部分消解。"不公信"侧确实在架构层有投影点（ResultClass + UNKNOWN），但未明示从 §一"不公信"链路下来——这是溯源链路缺，而非投影缺。挑刺成立但严重性降级。

**判定**：**成立**（强候选，但描述应精确为"溯源链路未明示"而非"零投影"）

**fix 操作**：architecture §一 加 1 段（不必新立 §一.0）："principles §一 双根本问题分两路投影到本架构：（1）'不公平'侧 → A/B/C 三大原则 → 上述三件设计选择；（2）'不公信'侧 → §六 Oracle 责任 + UNKNOWN 严格语义 → 本节末 ResultClass 三分 + §七决策表 UNKNOWN / partial 行"

---

## 候选 9：D3.4-3.6 范式（自身排除）

**c 路自身排除**：宪法层无 D3.4-3.6 编号，挑刺前置不成立。

**counter 角度**：confirm。已查 principles.md 全文无 D3.4-3.6 编号。tool-integration §四.4 已实质覆盖漏报盲点诚实声明范式。

**判定**：**不成立**（自身排除站得住）

**fix 操作**：无。

---

## 总结

**成立（必修）4 强候选 + 2 弱候选**：

| 候选 | 强度 | fix 操作 |
|---|---|---|
| 1 | 强 | 批量 sed 9 处章节号 |
| 2 | 强 | architecture L64/168/281 改写 UNKNOWN 定义 |
| 3 | 强 | tool-integration 新加 §四.5 wrapper 链路归因 |
| 8 | 强 | architecture §一 加 1 段双侧承接 |
| 5 | 弱 | architecture L287 改 §六 |
| 6 | 弱 | architecture L288 改 §六 |

**不成立 / 并入**：

- 候选 4：counter 站得住，宪法精神未明示"凡 Warn 都视为工具自陈失败"
- 候选 7：实质并入候选 8
- 候选 9：c 路已自身排除

**操作汇总（批量一轮）**：

1. tool-integration.md：batch sed 9 处章节号引用 + 新加 §四.5 wrapper 归因段
2. architecture.md：改写 L64/168/281 UNKNOWN 定义 + §一加双侧承接段 + L287/288 决策表章节号
3. 不动 principles.md（按 v8 P27 已生效）

**整体观察**：c 路 9 候选中 6 条成立（4 强 2 弱），1 条不成立（候选 4），2 条合并 / 自身排除（候选 7 / 9）。下游一级法律的 P27 同步缺口确认存在，建议批量修订一轮。
