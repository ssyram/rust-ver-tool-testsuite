# Axis B: 宪法 → 一级法律 (architecture / tool-integration) 一致性 challenge

> 审查者：独立 agent（与制宪非同上下文）
> 基准：principles.md v8（145 行，2026-05-12 P27 修订）
> 受审：architecture.md (299 行) + tool-integration.md (179 行)
> 协议：workflow-audit R1-R12 反状态清单 + §八 disprove-first

---

## 1. 总览

- 候选总数：**9**（强 4 / 弱 5 / 自身排除若干已剔除）
- 涉及文档：architecture.md + tool-integration.md
- 主要 issue 类型：v8 新条款下游零吸收（"本地性"/"当前性"/"最大善意"/"我们 wrapper vs 官方 wrapper"/UNKNOWN 严格语义 / 工具自陈必须尊重 / Q5 四条新值）+ 章节编号引用旧版宪法

---

## 2. 候选清单

### 候选 1：tool-integration.md:3,5,19,32,40,57,65,87,125,141,147,151 ── 章节编号引用旧版宪法（多处）

**当前文本**（多处典型）：
- L3 "本文是 principles.md §六-本节自我性声明 + §三-3-原则 3 的展开"
- L5 "仅需遵守 principles.md §四 三大派生原则"
- L19 "按宪法 §五"工具非静态原则""
- L32 "按宪法 §三-3-2.a"
- L40 "按宪法 §三-3-2.b 上限保证"
- L65 "按宪法 §三-3-2.c 下限诚实"
- L141 "按 principles.md §三-3-1 时效性"
- L147 "本节展开 principles.md §三-3-原则 3 的"实测报告责任边界""

**违反宪法**：principles.md v8 结构是 §一根本问题 / §二范围价值 / §三模块定位 / §四核心原则 / §五核心模块精神 / §六次要模块精神 / §七外围 / §八审查。**新版无 §三-3-原则 3、无 §三-3-2.a/b/c、无 §五"工具非静态原则"、无 §六-本节自我性声明**。引用全部错位：
- "工具非静态原则" → 新版位于 §五 runner 段"工具版本 + ISO 时间戳"
- "0 误报 / 0 漏报硬指标" → 新版 §六 "Oracle 责任·不冤枉 / 不藏"
- "时效性" → 新版 §六 "时空锚定"
- "自我性声明" → 新版 §二 "生效约束"段（外部复用只需遵守 §四 + §三）

**反状态排除**：
- R1：错引宪法章节直接破坏 §八 "审查 / 自修改 ... 区分决策点"——读者无法溯源 ✅ 构成
- R2：v8 是 P27 主动修订；旧引用是被淘汰的 contract，不是 deliberate 设计
- R5：v8 是当前生效宪法，非历史
- R11：是文档同步问题但影响理解链路 → 仍要修，定为强候选

**建议处理**：批量替换为 v8 章节号。最小集：
- §三-3-原则 3 → §六（次要模块）
- §三-3-2.a/b/c → §六 Oracle 责任三子条
- §三-3-1 → §六 时空锚定
- §五 工具非静态原则 → §五 runner 结果记录
- §六-本节自我性声明 → §二 生效约束
- §四 三大派生原则 → §四 A/B/C 核心原则

---

### 候选 2：architecture.md:64,168,221,281 ── UNKNOWN 旧宽松语义 vs v8 §六严格语义

**当前文本**：
- L64 "runner-internal 错误归 UNKNOWN，subprocess 跑完且非零归 FAILED"
- L168 "任一步 IO 错误时返回 Err(anyhow)，由 main 转为 UNKNOWN"
- L281 决策表 "UNKNOWN 仅记 runner-internal 错误"

**违反宪法**：principles.md §六（L111）UNKNOWN 严格语义仅两类：(a) 全局工具链崩溃可重装修，(b) 我们这边可识别问题且暂未修。架构当前定义"runner-internal 错误"是较旧的口径——
- 含义偏窄：未涵盖宪法 §六 (a) 的"全局工具链崩溃"
- 含义偏宽：把所有 anyhow Err 转 UNKNOWN，可能把"我们 corpus 引入的 vendored crate lint 错误"等 (b) 类问题与"运行时 IO 失败"混在一起，没强制要求附"明确归因 + 会修计划"

**反状态排除**：
- R1：宪法 L111 明文 "每类必附明确归因 + 会修计划" → 架构未投影此约束 ✅
- R2：旧"runner-internal"是 P21 时代 contract（5 类外部根因被剃后保留的语义）；P27 后被进一步收紧
- R7：宪法措辞 "严格语义"是强化，下游不可弱化
- R11：但 runner 实现可能仍按旧口径走 → 文档与实现都该跟进；本审查只看文档

**建议处理**：L64 改写为"按宪法 §六，UNKNOWN 严格语义两类：(a) 全局工具链崩溃 (b) 我们这边可识别但暂未修的问题"。L168 改为 "由 main 按 §六 严格语义分流：归 (a)/(b) 之一即 UNKNOWN 并记归因 + 会修计划，否则视为实现 bug 不应进 UNKNOWN"。L281 决策同步更新表述。

---

### 候选 3：tool-integration.md ── 未提"我们 wrapper vs 官方 wrapper"区分

**当前文本**：tool-integration.md 全文无 "wrapper"、无"我们的 wrapper"、无"官方 wrapper"区分。

**违反宪法**：principles.md §六 L111 "官方 wrapper 失败 / 工具自选 toolchain 不支持新特性 / 工具单文件 pipeline 不读 Cargo.toml / 官方 wrapper 不传 --edition 一律 FAILED"。这是 v8 新加的判定规则——下游必须落地为工具集成时 oracle 设计的指引：判 SUCCESS / FAILED / UNKNOWN 时应区分失败链路是来自我方 harness/wrapper（→ UNKNOWN）还是官方 wrapper（→ FAILED）。

**反状态排除**：
- R1：宪法明文要求，下游未投影 ✅
- R2：是 P27 新加约束，无 deprecate
- R11：文档同步类，但影响新增工具时的判定决策——属 tool-integration 范围

**建议处理**：tool-integration.md §四（漏报论证）或新增 §四.5 "wrapper 链路归因" 简短一节：区分两类失败链路 + 对应 ResultClass 落点。

---

### 候选 4：tool-integration.md:79-82 ── 单一通路论证 vs §六"工具自陈必须被尊重"

**当前文本**（L79）：
> aeneas：唯一 unsupported 入口是 craise，唯一 exit 决定是 Errors.error_list 非空 → exit 1，单一通路 ✅
> charon --abort-on-error：register_error! 是唯一 unsupported 入口，加 abort 后 panic 是唯一 exit 决定 ✅

**违反宪法**：principles.md §六 L109 "工具自陈"我没全干完"必须被尊重"。aeneas / charon 等工具实际不止 craise / register_error! 一条通路——还有 Warn channel（warning level）也可能是工具的自陈。当前仅论"exit 通路"通过 craise，构成"必要条件"但不构成"工具自陈完整尊重"——若工具走 Warn channel 自陈"我没全干完"，oracle 还可 SUCCESS（exit 0 不抓 warning）。

**反状态排除**：
- R1：宪法 L109 明文 "必须被尊重"，单一 exit 通路论证未覆盖 Warn 通路 ✅
- R2：tool-integration §四 形式证明小节 ✅ 标记是 "单一通路"——是 deliberate 设计，但 deliberate 的范围仅限"exit 通路 0 漏报"——不蕴含"工具自陈尊重"
- R11：理论文档问题但有实际语义后果——工具更新加新 Warn channel 时 oracle 不感知

**建议处理**：§四.1 加注：单一通路论证仅覆盖 exit 通路，Warn channel 类自陈需 §四.2 防漏报机制兜底；或显式声明 "Warn channel 不被纳入 oracle 失败信号"作为漏报盲点（按 §4.4 诚实声明）。**弱候选**：可视为已在 §四.4 漏报盲点段隐含——但宪法新加"必须被尊重"是强语义，需显式投影。

---

### 候选 5：architecture.md:287 ── 决策表引用 "principles §三-3 诚实测试范围"

**当前文本**：L287 "Oracle 0 误报第一，0 漏报次要（grep-based 工具不可形式证明） | 诚实性宗旨（principles §三-3 诚实测试范围） | 锁定"

**违反宪法**：v8 §三 仅为"模块定位"（核心 vs 次要），无 §三-3 子条。0 误报 / 0 漏报硬指标位于 v8 §六 "Oracle 责任·不冤枉 / 不藏"。

**反状态排除**：同候选 1，错引被淘汰章节号 ✅

**建议处理**：改为 "诚实性宗旨（principles §六 Oracle 责任·不冤枉）"。

---

### 候选 6：architecture.md:288 ── 决策表引用 "principles §六-2 不允许 partial"

**当前文本**：L288 "SUCCESS = 工具完整完成；不允许 partial（即便产物落盘） | 不允许 partial（principles §六-2） | 锁定"

**违反宪法**：v8 §六 无 "-2" 子条编号。当前 §六 段名是 "Oracle 责任"，内含三短子（不冤枉 / 不藏 / UNKNOWN 严格语义）。"不允许 partial" 落在 "不冤枉" 子条 (L109)。

**反状态排除**：同候选 1。

**建议处理**：改为 "（principles §六 Oracle 责任·不冤枉）"。

---

### 候选 7：architecture / tool-integration ── §二 Q5 四条新值零吸收

**当前文本**：下游零提及 v8 §二 "不可妥协价值"四条：
1. 样例与工具两端不可被中介层侵入
2. SUCCESS 信号必须诚实
3. 测试结果必须可重现
4. 测量姿势对社区惯例 + 本地性对齐

**违反宪法**：第 4 条 "测量姿势对社区惯例 + 本地性对齐" 是 v8 P27 新加（前三条原已有对应原则映射，第 4 条无下游落点）。

**反状态排除**：
- R1：宪法 §二 L50 明文，下游 0 投影 ✅
- R2：第 4 条是 P27 新增，非历史
- R5：v8 立刻生效
- R11：文档同步——但第 4 条直接影响 oracle 失败归因决策

**建议处理**：弱候选——可在 architecture §一推导 / tool-integration §三 0 误报论证开头加一段："SUCCESS / FAILED 之外还要保证测量姿势对社区惯例 + 本地性对齐——按 §一三原则栈装好工具要求的 toolchain + 按工具文档姿势用即可；尽到善意后 FAILED 站得住"。

---

### 候选 8：architecture.md:11 ── 设计推导段仅引"三大原则" 未引"两根本问题"

**当前文本**：L11 "principles.md 的三大原则（A 双方不可侵入 / B 测必要条件非语义对错 / C 异质性归配置）落到架构层面后产生三件具体设计选择"

**违反宪法**：v8 §一 双根本问题（不公平 + 不公信）是宪法最高位阶——三大原则 A/B/C 实质是"不公平"的对策（principles §一 L15 "把不公平沉淀到第三方中介层"）。架构推导只承接"不公平"侧，未承接"不公信"侧（本地性 / 社区惯例 / 最大善意三原则栈）——后者在架构 oracle 分类、UNKNOWN 严格语义、wrapper 区分上都有具体投影责任。

**反状态排除**：
- R1：宪法 §一是 v8 P27 新立的最高位阶，架构 §一未承接 ✅
- R2：v8 是当前
- R11：直接影响架构推导链路完整性——属强候选

**建议处理**：架构 §一 推导段补一小节"§一.0 双根本问题的双侧承接"，明示三大原则 A/B/C 是"不公平"侧投影，而 §三 oracle 分类（含 UNKNOWN 严格语义）+ tool-integration §三/§四（0 误报 / 0 漏报硬指标）是"不公信"侧投影。

---

### 候选 9：tool-integration.md ── 未含 README 漏报盲点诚实声明的"D3.4-3.6 范式"

**当前文本**：tool-integration §四.4 L107-115 "漏报盲点的诚实声明（兜底）"已含基本范式（例 hax-lean / rocq-of-rust 漏报盲点）。

**违反宪法**：用户输入提及 "D3.4-3.6 README 漏报盲点诚实声明范式" 但宪法 principles.md 全文无 D3.4-3.6 编号；架构 / tool-integration 也无。**这一项需查 detailed-design.md 是否定义该范式**——如未定义则属用户输入笔误或在尚未审查到的下游文档中。在 tool-integration 范围内 §四.4 已实质覆盖。

**反状态排除**：
- R1：找不到 D3.4-3.6 对应宪法条款，挑刺前置不成立 ✗
- R2：现有 §四.4 已是 deliberate contract
- 自身排除：候选不成立

**建议处理**：**移出清单**——此候选自身排除。如用户确指某具体范式，建议另起 Axis 在 detailed-design / tool README 层审。

---

## 3. 总结

**强候选（必修，影响理解或宪法投影完整性）**：4 条
- 候选 1：tool-integration 9 处章节号错引旧版宪法 → 批量同步
- 候选 2：architecture UNKNOWN 旧宽松语义 vs v8 严格两类 → 改写定义
- 候选 3：tool-integration 未提"我们 wrapper vs 官方 wrapper"区分 → 新增段
- 候选 8：architecture §一推导只承接"不公平"未承接"不公信" → 补 §一.0

**弱候选（文档同步类）**：5 条
- 候选 4：单一通路论证 vs Warn channel 工具自陈尊重 → 加注
- 候选 5：architecture 决策表 L287 §三-3 引用错位 → 改为 §六
- 候选 6：architecture 决策表 L288 §六-2 引用错位 → 改为 §六 不冤枉
- 候选 7：Q5 第 4 条 "社区惯例 + 本地性对齐" 零下游投影 → 加段

**自身排除（counter-challenge 站得住）**：1 条
- 候选 9：D3.4-3.6 范式宪法层不存在 → 移出

**总结观察**：下游一级法律（architecture / tool-integration）整体仍停在 P21-P26 时代——v8 P27 主要新加内容（双根本问题 / 本地性原则栈 / wrapper 区分 / UNKNOWN 严格语义 / Q5 第 4 条）几乎零吸收；同时存在大量章节号错引旧版结构。这是一次较系统的下游同步缺口，不是单点违反——建议批量修订一轮，而非逐条。文档级 issue 占主体，**runner 实现层是否同步对齐 P27 不在本 Axis 范围**（需 Axis C 实现 vs 法律审查）。
