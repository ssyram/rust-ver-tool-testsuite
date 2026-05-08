# 项目级 Claude 指令

> 本文是项目对 Claude 的硬性约束。Claude 在本项目中所有工作都必须遵守。

---

## 一、文档中心：`docs/design/` 是一切的根本

**所有工作以 [`docs/design/`](docs/design/) 为中心**：

```
docs/design/
├── principles.md       ← 【绝对宪法】未经允许不可篡改；所有讨论与争议以此为准
├── architecture.md     ← 在宪法约束下的核心架构；不违反宪法时为设计核心
└── detailed-design.md  ← 函数级细化
```

### 1.1 `principles.md` 是绝对宪法

- **未经用户显式允许，禁止修改 `principles.md`**
- Claude 提出修改 `principles.md` 的建议时，必须明确告知用户这是宪法级修订，等用户确认后才能改
- 任何技术决策、代码改动、配置修改都必须能溯源到 `principles.md` 的某一条
- 当下游设计或代码与 `principles.md` 冲突时：**改下游不改宪法**（除非用户显式同意修订宪法）

### 1.2 `architecture.md` 是核心架构（次于宪法）

- 在不违反 `principles.md` 的前提下，`architecture.md` 是核心架构设计
- 若 `architecture.md` 与 `principles.md` 冲突：**改 architecture 不改 principles**
- 若实现代码与 `architecture.md` 冲突：**改实现不改 architecture**（除非显式讨论修订）

### 1.3 文档优先原则

**任何更新都优先维护文档，再做代码 / 配置 / 实测的修改**：

1. 发现需求或问题 → 先看 `principles.md` 是否覆盖；不覆盖则提议修订宪法
2. 宪法允许 → 看 `architecture.md` 是否覆盖；不覆盖则修订架构
3. 架构允许 → 看 `detailed-design.md` 或 `tools/<name>/README.md`；不覆盖则修订细化
4. 设计完整后 → 才做代码 / 配置 / 实测修改

**禁止跳过文档直接改代码**——除非是显而易见的笔误或局部 fix。

---

## 二、工作流规范

遵循 [`/workflow`](https://) skill 的三阶段：

1. **调研阶段** — `docs/research/testsuite-research.md`
2. **架构阶段** — `docs/design/architecture.md` + `principles.md`
3. **细化阶段** — `docs/design/detailed-design.md` + `tools/<name>/README.md`

每个阶段产出文档后必须等用户确认才能进入下一阶段。

详见 [`/workflow`](https://) skill 与 `~/.claude/projects/-Users-ssyram-workspace-rust-ver-rust-ver-tool-testsuite/memory/feedback_workflow_adherence.md`。

---

## 三、模块优先级

按 `principles.md` §三：

| 模块 | 优先级 | 关注度 |
|---|---|---|
| **核心模块 1**：runner（测试运行与结果分析框架）| first-class | 长期承诺 |
| **核心模块 2**：examples（样例库）| first-class | 长期承诺 |
| **次要模块 3**：tools 集成 + 测试报告 | non-first-class | 应用展示，时效性强 |

工具集成（`tools/<name>/`）的深度调优、Oracle 精确性研究等工作属次要模块——**不应抢核心模块的优先级**。

---

## 四、行为禁令

- 禁止修改 `principles.md` 未经用户显式允许
- 禁止跳过文档直接改代码（除非显然笔误）
- 禁止把次要模块的工作（工具集成调优）当作核心承诺
- 禁止在测试报告里给工具能力下绝对结论（必须明确锚定时间 + 工具版本）
- 禁止把"测试结果"等同于"工具能力评判"（前者是当下快照，后者超出本项目范围）

---

## 五、新增工具的硬指标

详见 `principles.md` §六。Claude 帮用户新增工具时必须：

1. 该工具的 README 明确 pipeline 阶段图、前端/后端边界、`tool.toml` 切割 flag
2. 形式指标的 SUCCESS 信号 + partial 暴露机制 + 形式严格性（0 误报 / 0 漏报状态）+ 漏报盲点
3. 配置满足"不允许 partial"+"反作弊"+"形式指标为最终解释"

---

## 六、与项目长期记忆的关系

项目内存路径：`~/.claude/projects/-Users-ssyram-workspace-rust-ver-rust-ver-tool-testsuite/memory/`

memory 中的 feedback / project / user / reference 类文档是 Claude 跨会话的工作辅助，**不替代 `docs/design/`**。如 memory 与 `principles.md` 冲突，以 `principles.md` 为准；同时建议更新 memory 反映最新原则。
