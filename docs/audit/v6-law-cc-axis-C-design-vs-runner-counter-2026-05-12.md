# Axis C cc-counter: 设计文档 → runner 实现一致性反挑刺

> Reviewer: counter-challenger（cc round, disprove-first）
> 对象：v6-law-c-axis-C-design-vs-runner-2026-05-12.md（10 候选 C1-C10）
> 协议：对每条 c 路结论做反挑刺；R1/R2/R5/R6/R11/R12 视角检视；落"成立 / 不成立 / 部分"。

---

## 1. 候选 C1：Tera 模板变量 = 2 vs impl 注入 `entry_args`

**c 路指证**：detailed-design.md:134-141 + architecture.md:274 "锁定两个"，exec.rs:108-113 注入第三个。

**counter 视角**：

- **R1（现象 ≠ 缺陷）反查**：detailed-design.md:141 原文"变量集**固定**——不允许 tool config 扩展（破坏原则 C 的'标准词汇'约束）"。措辞"不允许 tool config 扩展"——`entry_args` 不是 tool config 扩展，是 example schema 扩展（来自 `[runnable.<entry>]` 表，per-entry 给定），且对所有工具的 harness 模板统一可见，等价于"标准词汇"新增第三个标准词，不破坏 C 精神。
- **R6（局部 stale ≠ 全局缺）反查**：grep `entry_args` 在 docs/design/ — 仅 hax-lean-consistency-design 例子有，detailed-design / architecture 主章均无（确认无全局补述）。但 exec.rs:101-107 注释明确锚 "discover.rs::Example::entry_args + detailed-design.md §一 ('Schema 向后兼容性') + false-positive-audit-2026-05-11.md §4.1"，detailed-design §一 经核查无"Schema 向后兼容性"小节——impl 注释引用了一个不存在的章节。**这反而加重 c 路指证**。
- **R11（文案 ≠ 实现）反查**：架构 §七决策表"锁定两个"是 axis-A 已审过的结构性陈述。决策表 = contract（不是说明文）；contract 与 impl 发散 = 实质冲突。
- **R12（作者反驳）**：作者会说"P-runnable / runnable corpus 后补丁；C 是 detailed-design 滞后于 impl 的同步问题"——可接受，但"滞后"恰是 c 路诉求的 "改下游不改宪法 → 改 design 同步"。

**判定**：**成立**。c 路完全站得住，无法 disprove。`entry_args` 是已落地的第三个标准模板变量，detailed-design / architecture 必须同步——且 exec.rs:104 注释里的 "detailed-design.md §一（'Schema 向后兼容性'）" 引用本身指向不存在章节，属次生 dangling reference，建议合并修。

**fix 操作**：
1. detailed-design.md:134-141 Tera 变量表加 `entry_args` 行 + 说明来源 (`Example::entry_args` / `[runnable.<entry>]`)；删/改"恰好两个 / 固定"措辞为"标准词汇集（runner 端管理，tool config 不可扩展）"。
2. detailed-design.md §一 同时补"Schema 向后兼容性"小节（exec.rs:104 已锚此处）。
3. architecture.md:274 决策表"标准模板变量"行改"三个" + 注脚 P-runnable / corpus 端展开。

---

## 2. 候选 C2：`classify_external_fault` 在 detailed-design §七 缺席

**c 路指证**：detailed-design.md:570-579 错误处理表无 FAILED→UNKNOWN 重分类一行。

**counter 视角**：

- **R5（历史 / 废弃）反查**：principles.md:108-111 §六 是 2026-05-12 v8 内容；detailed-design §七 表对应 P-pre-21 旧契约。c 路指证成立，**不是 stale 章节**——detailed-design 不带 "P21 前" 标注。
- **R6 反查**：核查 detailed-design 全文 grep 无 `external_fault` / `narrowed` / `重分类`，确认 §七 之外亦无补述。架构 §一末段 line 64 + §七决策表 line 281 同样停在旧表述。
- **R12（作者反驳）**：作者会说"P27 是 oracle 实现的同步性 fix，design 文档已计划批量重写，不算 contract violation 而是 sync gap"——**这恰是 R6 不成立的承认**。"sync gap"在 contract 文档里就是 contract 失同步，不是免责。principles.md §六 是宪法层（最高），detailed-design §七 是 contract spec 层，宪法立法已落而 contract spec 未传导，等于"立法 → 行政细则"漏一跳。
- **R11（文案 ≠ 实现）反查**：§七 错误处理表是穷举式（5 行覆盖 5 路），不是说明文——缺一行是穷举缺，不是文案。

**判定**：**成立**。无可 disprove。

**fix 操作**：detailed-design §七 错误处理表新增一行 "子进程退出非零且 stderr/stdout 命中 narrowed external-fault catalogue → UNKNOWN + `error: external_fault: <tag>`"，附 3 条 catalogue 子表（runnable_harness_arg_mismatch / vendor_lint_strictness / environment_corruption）。

---

## 3. 候选 C3：`TS_*` env 隔离 + `${VAR}` 展开未文档化

**c 路指证**：architecture / detailed-design 主文档完全无 `TS_*` / `${VAR}` / `env_remove` / `expand_env` 提及，仅 hax-lean-consistency-design sample 出现。

**counter 视角**：

- **R6 反查**：grep `TS_\|\${VAR\|expand_env` in docs/design/ → 全部命中在 hax-lean-consistency-design-2026-05-11.md（专题设计文档），architecture.md / detailed-design.md / principles.md / tool-integration.md 主三轴**完全未提**。确认无全局补述。
- **R11（文案 ≠ 实现）反查**：这是 runtime contract 而非措辞：
  - `${VAR}` 展开发生在 discover.rs:430-471（schema 加载期），影响 `tool.toml` 字段语义
  - `env_remove(TS_*)` + `env(TS_ENTRY_FN/TARGET_CRATE)` 发生在 exec.rs:178-192（spawn 前），影响子进程 env 表面
  - 两者都是**集成者必须知道**的接口契约（写 `tool.toml` 要用 `${VAR}`、写 wrapper 要读 `TS_*`），不是内部实现细节。
- **R12（作者反驳）**：作者会说"TS_* 隔离是 mechanical contract，每个工具 wrapper 都已自陈；design 不重复说是 R6 + R11"——**反驳无力**：
  - "每个工具 wrapper 自陈"：core contract 不该靠 N 份 sample 文档拼出，而应在 detailed-design §一 (tool.toml schema) 与 §四 (单次执行单元) 集中说明。
  - "mechanical"不构成"不需要文档化"——架构层 spec 的意义恰是把 mechanical contract 集中。
- **R5 反查**：`TS_PROJECT_ROOT` 起源在 main.rs:66-70，是 host.rs / .env.example 引入的当前机制，非废弃段。

**判定**：**成立**。c 路诉求完全站得住。

**fix 操作**：
1. detailed-design §一 `tool.toml` 字段表后加 "`${VAR}` 展开规则" 小节：变量名 `[A-Z_][A-Z0-9_]*`、`source .env` 提供、missing → 空串、apply 范围 `command` / `version_command`。
2. detailed-design §四 单次执行单元在步骤 6 前插入 "6_pre. spawn 前 env 处理：strip `TS_*` 前缀全部 env、注入 `TS_ENTRY_FN` = entry / `TS_TARGET_CRATE` = crate_ident"。
3. architecture §五（runner ↔ external_subprocess 接口）补 env 协议条目，简短一段。

---

## 4. 候选 C4：detailed-design §五 kani timeout 120 vs §六 results.json 例子 600 自相矛盾

**counter 视角**：

- **R11 部分适用**：是 design 自身文本不一致，不涉及 impl 行为差异——属内部文档自洽问题。c 路自己也归类"次要决策点"。
- **R12（作者反驳）**：作者会说"§六 results.json 是 schema 示例，timeout_secs 字段示意性占位，不绑定 §五 配置；读者按 §五 实际值理解即可"——**部分成立**：JSON schema 例子用 600 不一定表示 kani 真值，可视为 placeholder。但 c 路指证示例里直接用 `"name": "kani"` 同时填 600，作为 kani 的 schema 输出确有歧义。

**判定**：**部分成立**——design 内部小不一致，optional fix。

**fix 操作**：§六 例子 timeout_secs 改 120 同步 §五，或加注 "示例值，实际以 tool.toml 为准"。

---

## 5. 候选 C5：report.md 章节布局 design 欠描述（impl 是 superset）

**counter 视角**：

- **R1（现象 ≠ 缺陷）反查**：design 例子展示矩阵 + Total，确实没标"省略其他章节"，但 detailed-design.md:546 "按 feature 分组的工具 × entry 矩阵"是**核心契约**，metadata header / per-tool summary 是辅助信息——属"实现可丰富 design 主结构不变"范畴。
- **R2 反查**：架构 §四 后置条件 line 181 "report.md 含按 feature 分组的工具 × entry 矩阵"——只规约**含**而未规约**仅含**，open contract 性质，impl 加章节不违约。
- **R12（作者反驳）**：作者会说"report.md 是 human-readable，加 metadata / 聚合段是渐进 UX 增强，schema 在 results.json，report.md 不是 contract spec"——**站得住**。design 没把 report.md layout 当 closed contract。

**判定**：**部分成立**（明示性弱化）。non-contract 层差距，不属"design 与 impl 实质冲突"。

**fix 操作**：detailed-design §六 加一句 "report.md 在矩阵之外还会渲染 Run metadata / Tool versions / Per-tool summary / Per-feature summary 等聚合段（详见 report.rs::write_report_md）"。低优先级。

---

## 6. 候选 C6：`schema_kind` / `hirusttest_dir` impl 标 `#[allow(dead_code)]`

**counter 视角**：

- **R2 反查**：detailed-design.md:284-288 §运行时机制 3c 自标"v1 仅识别字段，stub 注入由后续版本支持"——design **已自带 forward 缓冲**。
- **R12（作者反驳）**：作者会说"design 明示 v1 缓冲、impl `#[allow(dead_code)]` 是 v1 的诚实标注，两者方向一致"——**完全站得住**。
- **R6 反查**：design 与 impl 方向一致（暴露但 v1 不消费），契约明示性弱但不冲突。

**判定**：**不成立**（c 路自评 "弱差距"准确）。design / impl 一致，仅明示性弱。

**fix 操作**：可选——detailed-design §三 `find_examples` 后置条件补 "v1 实现仅暴露字段，exec 端 stub 注入留 v2"。极低优先级。

---

## 7. 候选 C7：`entries = []` "必填"措辞 vs impl 拒绝空

**counter 视角**：

- **R11 部分适用**：design "必填"在工程语境通常即"非空必填"（参见 toml 常见解释），措辞略松但意图一致。
- **R12（作者反驳）**：作者会说"配置者写 `entries = []` 触发 hard error 是合理 fail-fast，与 §七 'schema bug discover 阶段 panic' 兜底原则一致"——**站得住**。
- **R1 反查**：行为差（拒绝 vs 接受）的"接受"分支在 design 中未承诺，是读者推断；impl 拒绝有具体 anyhow 信息，非沉默差。

**判定**：**部分成立**——文档措辞精度问题，非实质契约冲突。

**fix 操作**：detailed-design.md:21 注释改 "必填，非空 pub fn 名列表"。低优先级。

---

## 8. 候选 C8：`extra_cargo_deps` discover 阶段 TOML 预校验未文档化

**counter 视角**：

- **R2 反查**：c 路自承"行为方向与 §七 兜底原则一致" + "不矛盾"。属补强而非冲突。
- **R12（作者反驳）**：作者会说"discover 阶段 toml_edit 预校验是 §七 'schema 解析失败 → discover panic' 的具体落地，不构成新契约"——**站得住**。

**判定**：**不成立**（c 路自评"低优先级"准确）。design / impl 兼容。

**fix 操作**：optional，detailed-design §一 `extra_cargo_deps` 字段加一句"discover 阶段做 toml_edit 预校验"。极低优先级。

---

## 9. 候选 C9：principles §六 UNKNOWN 严格语义 → architecture / detailed-design 未传导

**counter 视角**：

- **R5 反查**：principles.md:108-111 是 2026-05-12 v8 现行表述（DP-4 已落），不是历史段。architecture.md:64 / 281 仍停在 P-pre-21 旧描述，**确为传导漏跳**。
- **R6 反查**：grep `narrowed\|external_fault` 在 architecture.md 完全无；line 281 决策表 "UNKNOWN 仅记 runner-internal 错误" 是与 principles §六 直接冲突的旧 contract。
- **R12（作者反驳）**：作者会说"宪法 → architecture → detailed-design 传导是逐步推进的，DP-4 立场刚落（principles v8）就要求架构同步偏激进"——**反驳无力**：宪法立法已落，下游 contract 文档与宪法发生直接冲突（line 281 决策表"仅记"vs 宪法"两类合法 + 工具边界一律 FAILED"），是必须立刻同步的传导跳跃。
- **R11 反查**：决策表"锁定"行 = 结构性 contract，不是文案。

**判定**：**成立**。c 路结论与 axis-A 候选 2（§六 enumerate）同源指向，无 disprove 空间。

**fix 操作**：
1. architecture.md:64 §一末段 UNKNOWN 描述：把"runner-internal 错误"改为 "principles §六 narrowed catalogue（(a) 全局工具链崩溃 (b) 我方可识别问题，工具自身能力边界一律 FAILED）"。
2. architecture.md:281 决策表 "UNKNOWN 仅记 runner-internal 错误" 改为 "UNKNOWN 限 narrowed catalogue（详见 principles §六）"。
3. detailed-design §七 错误处理表与 C2 fix 合并修。

---

## 10. 候选 C10：UNKNOWN 子类型 schema 未在 design 区分

**counter 视角**：

- **R6 反查**：detailed-design.md:529-542 schema 例子仅展示 runner-internal Err 形态，无 `external_fault:` 前缀样本；§六 文字也未声明 `error` 字段子类型。
- **R12（作者反驳）**：作者会说"`error` 字段是 free-form 字符串，下游消费者按 catalogue 字面匹配即可，schema 不必声明"——**反驳无力**：results.json 是 contract（principles.md §二 测量姿势可追溯），下游消费者期望 schema 明示，自由文本 + 隐式前缀语义不是 schema 应有形态。
- **R11 反查**：c 路指证是 schema 完整性缺失，不是措辞。

**判定**：**成立**（与 C2 / C9 同源，P27 改动的 schema 端传导缺口）。

**fix 操作**：detailed-design §六 results.json schema 注明 `error` 字段的两种合法形态：
- `external_fault: <tag>`（FAILED→UNKNOWN 重分类，tag ∈ narrowed catalogue）
- 自由文本（runner-internal Err 的 `format!("{:#}", e)`）

---

## 总结

| 候选 | 类别 | 判定 | fix 操作 |
|---|---|---|---|
| C1 | 强 | **成立** | detailed-design §一 + architecture §七 决策表 + 补 "Schema 向后兼容性" 小节 |
| C2 | 强 | **成立** | detailed-design §七 错误处理表加一行 + catalogue 子表 |
| C3 | 强 | **成立** | detailed-design §一 + §四 + architecture §五 补 env 协议 |
| C4 | 弱 | 部分成立 | §六 kani timeout 例子改 120 或加注 |
| C5 | 中 | 部分成立 | detailed-design §六 一句话提 report.md superset 章节 |
| C6 | 弱 | **不成立** | 可选小注，design / impl 已方向一致 |
| C7 | 中 | 部分成立 | detailed-design.md:21 措辞 "必填" → "非空必填" |
| C8 | 弱 | **不成立** | 可选小注，属补强 |
| C9 | 强 | **成立** | architecture.md:64 + 281 + detailed-design §七 同步 principles §六 |
| C10 | 强 | **成立** | detailed-design §六 results.json schema 注明 error 字段两种形态 |

**主体判断**：

- **5 条成立**（C1 / C2 / C3 / C9 / C10）—— c 路指证强候选全部站得住，无法 disprove。C2 / C9 / C10 是同源 P27 改动的 architecture / detailed-design / schema 三端传导缺口；C1 是 P-runnable 改动的 design 同步缺；C3 是 wrapper 类工具核心 contract 从未文档化。
- **3 条部分成立**（C4 / C5 / C7）—— 文档明示性 / 内部自洽问题，optional fix。
- **2 条不成立**（C6 / C8）—— design / impl 方向一致，仅明示性弱。

**根本判断**：宪法 → 架构 → 细化的传导链上，principles.md ✅ 已落 v8；architecture / detailed-design ✗ 停在 P-pre-21 旧表述；impl ✅ 已落。**修宪以下的层级文档（architecture + detailed-design）是当前最严重的传导漏跳点**，建议下一轮文档维护以"传导同步批量修订"为主，不再加新宪法层内容。同时 C1 / C3 暴露出 P-runnable / wrapper env contract 两条独立现行机制的文档化缺位，与 P27 传导不同源，需要单独补充。
