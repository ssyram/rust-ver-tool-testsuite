# v6 全面法律审查综合报告（2026-05-12）

> 走 `/workflow-audit` disprove-first 多向并行 + 反质疑流程；spec gate 走 `/hoare-audit` Step 0（宪法 + 架构即 spec）；汇报姿态走 `/principle-derivation-v2`。
>
> 输入：v6 全闭环落定后的完整法律体系（宪法 → 架构 / 集成 → 细化 → runner 实施 → 20 工具 / 161 examples）。
>
> 输出：本报告 + 修订 commit + 11 份 axis-级 audit 中间产物（`docs/audit/v6-law-{c,cc}-axis-{A,B,C,D,E}-*.md`）。

---

## 一、最先看到的现象

5 路独立 challenger 派出去——A 审宪法自身、B 审宪法→一级法律、C 审设计→runner、D 审工具 README→wrapper、E 审 examples 中立性——回来一共抛出 **35 候选 + 1 axis 0 候选**。然后派 4 路 counter（E 跳过）回头逐条挑刺这些 challenge，最后**站得住的只有 12 个 + 8 部分**。

数字本身不重要。重要的是**怎么分布**：

| Axis | c 抛 | cc 成立 | cc 部分 | cc 推翻 / 排除 |
| --- | ---:| ---:| ---:| ---: |
| A 宪法自身 | 13 | 1 | 3 | 9 |
| B 宪法→一级法律 | 9 | 6 | 1 | 2 |
| C 设计→runner | 10 | 5 | 3 | 2 |
| D 工具 README→wrapper | 4 | 0 | 2 | 2 |
| E examples 中立性 | 0 | — | — | — |

两个极端：**A 13 抛 1 立**，**E 0 抛 0 立**。中间 B / C 是审查产出率最高的。

这分布本身就在说话——但说什么，下面慢慢看。

---

## 二、E 为什么是 0

independent challenger 在 examples/ 下做了全量 grep + 抽样 25/161：

- `examples/**/*.rs` 找任何 verifier attribute / macro（`kani::proof` / `prusti::` / `creusot_contracts` / `verus!` / `#[ensures/requires/invariant]`）：**0 命中**
- `examples/**/Cargo.toml` 找 verifier 依赖：**0 命中**
- 根 `Cargo.toml` 的 `workspace.exclude = ["examples", ...]` 兜底——`hirusttest.toml` 不被任何 Cargo.toml 引用 → cargo 字节级中立成立
- 每 entry 一个独立 `src/lib.rs`，无跨 entry helper

宪法 §四 原则 A "双方都不可侵入" + §五 examples "纯净性 + 多样性" 在 corpus 层**完全成立**。这条线不用动。

为什么这条线特别干净？因为它早在 P11 就被设定为绝对规则（"双轨 schema 信号文件加入前后 cargo 字节级一致" 是 P11 锁的），并且后续所有补 entry 的工作都在这条规则下进行。**早期严格 + 持续遵守 = 后来不需要审查**。

---

## 三、宪法自己的 13 抛 1 立

A 路 challenger 拿 charter-craft skill（防腐蚀 + 奥卡姆 + 反 bloat + 反流程指令 + 反分析框架入宪）当尺子，对着 145 行的 v8 宪法逐条挑：

- §一 三原则栈（本地性 / 社区惯例 / 最大善意）被指"分析框架入宪"
- §二 Q5 4 条不可妥协价值被指"与 §四 / §五 / §六 重复"
- §四 A 双轨 schema 段被指"工程现实绑死"
- §六 末段四类 FAILED 列举被指"enumerate"
- §八 5 步审查流程被指"流程指令入宪"

cc 路 counter 后只剩 §四 A 双轨 schema 这一条真站得住——因为"什么算简单 / 什么算复杂"确实是工程判断，不该锁在宪法。剃掉，下沉 detailed-design。其余四条都不成立：

- 三原则栈是用户**明示的冲突消解优先级**（charter-craft §3.5 把冲突消解作为宪法必含）
- Q5 是问题层（不可妥协价值），§四 是答案层（核心原则），是问题 → 答案映射不是重复
- §六 末段 enumerate 表达"对立面"是 contrastive 表达，加"包括但不限于"即可保持开放
- §八 审查协议是 charter-craft §4.8 的**项目级化身**，"独立 + 先反后反反" 是核心精神不是任意流程

precision 1/13 ≈ 8%——单看会觉得 c 路效率低；但 1/13 中那 1 真的剃掉了一段工程绑死，这一处剃了就一直在了。剩下 12 个挑刺被 counter 推翻，意味着宪法 v8 在 P27 修订后**精神面已相当稳**——再加内容大概率是 bloat。

**这条线之后的工作建议是减法为主，不是再加新章。**

---

## 四、宪法和下游的同步漏

B 路和 C 路并起来看最有意思。这两路加起来 19 c 抛，cc 后 11 站得住 + 4 部分——precision 接近 60%。命中率比 A 高一个数量级。

为什么？因为它们审的不是"精神层"，是"传导链"——P27 改了宪法 §一 §六（双根本问题 + UNKNOWN 严格语义 + wrapper 区分），但是：

- `architecture.md` 还停在"UNKNOWN = runner-internal 错误"的旧口径
- `tool-integration.md` 还在引"§三-3-原则 3"、"§五 工具非静态原则"这种 v8 重排后已不存在的章节号
- `detailed-design.md` 还在锁定"Tera 模板变量恰好两个"（P21 已加 `entry_args` 第三个）
- `results.json` schema 没把 `error` 字段两种 UNKNOWN 子来源描述出来（runner-internal vs `external_fault: <subtype>`）
- TS_* env 隔离 + `${VAR}` 展开 + `TS_PROJECT_ROOT` / `TS_ENTRY_FN` 注入这套 wrapper 工具的核心 contract，design docs 完全没记

这些都不是**实施有 bug**——runner / oracle / wrappers 实际行为全部对齐了 P27 修订；错的是**法律文档没追上实施**。属于"立法已落、行政细则未传导"。

修订动作就两类：

- **机械同步**：9 处章节号引用 / 旧 enumerate / 旧"5 类外部根因"——sed-friendly batch
- **传导段落**：architecture L64 改写 UNKNOWN 段（加 §一 双侧投影），detailed-design §一 / §四 / §六 / §七 加 `entry_args` / TS_* / external_fault subtype 一致描述

两类都已落地。

为什么这条线 c 抛多 cc 又多成立？因为审查的目标是**机械可检的同步差**——只要 c 找到 spec 与 impl / 上游与下游不一致的点，cc 没什么角度可推翻。这是 audit 投入产出比最高的区域。

**审查的边际效用集中在"近期改动的传导链"上**——这是这次 audit 给后续的最具体启示。

---

## 五、工具 README 4 抛 0 真不一致

D 路找了 4 个 c 候选：

- ror README "6 道门" / wrapper 实际 7 道（P28 加 gate 漏改 README 数字）
- ror-typecheck README "9 道门" / wrapper 实际 10 道（同源 P30 漏改）
- prusti README L62 "wrapper 联合实施 marker 条件" 措辞误导（marker 必伴 exit ≠ 0，wrapper 实际只检 exit + .vpr 数）
- verifast tool.toml L7-13 旧 "N ≤ 40 threshold" 注释段（wrapper 早改为 `src/lib.rs(` 锚 grep）

cc 路把 4 个全降为 R11（文案 ≠ 实现问题）——**oracle 实际行为全部对齐宪法 §六**，问题面只在文档描述滞后。

D-2 ror-typecheck 的 wrapper 注释甚至自己已经写道"v6 cc-route 漏报审查发现 tier-1 漏抄此 gate，破声明"——作者自陈已 acknowledge，只是 README 门数表没改。

四处文案都已修。这条线的产出比例和 A 路是反的：c 路 4/4 命中（都是真的文案差），但 cc 路把严重性全部降到非阻塞——所以"成立但不重要"。这种类别**应该收 + 应该按低优先级收**，不该一发现就当重要 finding。

---

## 六、实施层的零问题

C 路同时也审了 `runner/src/*.rs` 五个文件，**没找出任何实施违反 design 的地方**。10 个 c 候选全部是 design 文档过时——impl 没有问题。

加上 D 路的 4 个 R11 文案——**实施层从 runner 到 20 个工具 wrapper 是完全对齐宪法 v8 + Oracle 严格语义的**。

这件事本身比"我们找出 14 个 fix 点"更重要：经过 P27 / P28 / P29 / P30 的高密度修订后，实施层已经稳定。后续的工作压力不在 fix bug，在维持文档同步。

---

## 七、disprove-first 协议的实证有效性

回头看四路 cc 的 precision 分布：

- cc-A 1/13 ≈ 8%
- cc-D 0/4 ≈ 0%（4 全降为 R11）
- cc-B 6/9 ≈ 67%
- cc-C 8/10 = 80%

差异最大近 10 倍。

charter-craft §4.8.1 的实用主义经验"反两次过滤低效挑刺"——本次正好实证。如果只走 c 路把 35 个候选全当 finding 落地，会做 23 次过度修订（A 12 + D 4 + B 3 + C 2 + 部分 1 = 22 ≈ 65% noise）。

c+cc 双层比单 c 贵一倍，但把 noise 砍掉 65%，是工程实用 ROI 最佳点。

这次也补强了一个观察——**审查 axis 的选择决定 ROI 而不是 audit 严格程度**：

- 审"已稳定的精神层"（A、D）→ 高 noise
- 审"传导链 / 机械可检差异"（B、C）→ 高 signal

下次再做这种全局法律 audit，axis A / D 投入可减半甚至跳过，把精力集中在 B / C 类轴。

---

## 八、修订落地清单

| 文件 | 修订 |
| --- | --- |
| `docs/design/principles.md` | 剃 §四 A 双轨 schema 段（4 行）+ §六 enumerate 加"包括但不限于"+ §六 前端测量加"等"|
| `docs/design/architecture.md` | §一 §B 在结果分类上的扩展段重写（加 §一 双根本投影 + UNKNOWN 严格语义两类详述 + P27 删 3 规则脚注）+ exec 模块 UNKNOWN 描述精化 + §七决策表 ResultClass 行扩 |
| `docs/design/tool-integration.md` | 9 处旧章节号 sed 同步（§三-3 / §五 / §六-本节 → v8 章节号）+ §四.5 新增"我们 wrapper vs 官方 wrapper" 失败归因段 + rocq-of-rust 6 道门 → 7 道门 + cargo-prusti 措辞 |
| `docs/design/detailed-design.md` | Tera 模板变量 2 → 3 (加 entry_args) + §四 新增"子进程 env 契约（TS_*）"段 + §六 results.json schema 加 external_fault subtype 例子 + §七 错误处理表加 §六 投影 + §七.1 UNKNOWN 严格语义投影段 |
| `tools/rocq-of-rust/README.md` | 6 道门 → 7 道门（全文 sed）+ gate 7 描述 |
| `tools/rocq-of-rust-typecheck/README.md` | 9 道门 → 10 道门 + gate 7 / 8-10 重编号 + Stage 1 7 道 grep 描述 + 上线档案 |
| `tools/prusti/README.md` | L62 改写"联合实施" → marker 必伴 exit ≠ 0 等价 |
| `tools/verifast/tool.toml` | L1-13 / L36-40 旧 "N ≤ 40 threshold" 注释段重写为当前 oracle 描述 |

---

## 九、commit 链（本次 audit 周期）

| commit | 内容 |
| --- | --- |
| (本 commit) | P31: v6 全面法律 audit (5 路 c + 4 路 cc) + 法律传导修订（宪法剃 1 处 + architecture / tool-integration / detailed-design 同步 + 4 处工具文案）+ 综合报告 |

---

## 十、剩余开放性

本次审查不构成"宪法 → 实施全栈 0 不一致"的最终断言。审查锁定的是：

- 在 5 路 axis 提名的 36 候选中，cc 路过滤后 20 个站得住或部分站得住——全部已落地修订
- 实施层（runner / wrappers / examples）经过 P27-P30 的高密度修订已对齐宪法 v8
- 法律层（architecture / tool-integration / detailed-design）经本次修订也对齐了 P27 改动

未审查的剩余：

- `docs/research/` 下三份研究文档（testsuite-research / ror-runnable-deep-dive / translation-correctness-feasibility）——属调研产物，不属法律体系
- `docs/fixes/` 下大量过往 audit 文档——历史快照，按 R5（历史 ≠ 当前错误）不予追溯
- `docs/design/hax-lean-consistency-design-2026-05-11.md`（729 行）——单工具特性设计文档，本次未独立审

后续若有新一轮宪法修订（如 v9）或大型 design 改动，建议再走一次 c+cc 双层，但 axis 可裁剪为 B + C（同步链审查）为主。
