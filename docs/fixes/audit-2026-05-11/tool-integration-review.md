# 工具集成原则审查 — `docs/design/tool-integration.md`

> 综合审计 `audit-2026-05-11` / 第 1 组（宪法 + 设计原则文档层）
> 审查日期：2026-05-11
> 审查范围：`docs/design/tool-integration.md`

---

## §1 问题意识

### 这份文档要审查什么？

`docs/design/tool-integration.md`（179 行）是 "我方维护 tools 集成与实测报告"的具体实现原则。按其自身 line 3-5 定位：「本文是 principles.md §六-本节自我性声明 + §三-3-原则 3 的展开——是**我方维护 tools 集成与实测报告**的具体实现原则」。

**核心承担**：
- §一 工具版本锁定
- §二 SUCCESS 信号的形式指标声明
- §三 0 误报的论证（必须，硬指标）
- §四 0 漏报的论证与防御（最好但不强求，软指标 + 漏报盲点声明）
- §五 README 必含章节清单
- §六 禁忌（自我性的体现）
- §七 实测报告原则

### 为什么要审查？

1. **"自我性"边界**：本文是项目方法学选择，自我性声明（line 5）"外来 tools 无须遵守"。要审查这条边界**是否被本文自己正确反映**，以及与 principles.md §六 自我性声明是否对齐。
2. **0 误报论证 vs 0 漏报论证**的论证套路是否周延：§3 / §4 都是"硬约束论证"，但其**论证标准**（什么算"足够证据"）是否对未来集成有充分指导？
3. **反误报双向实测**（§4.2）是否覆盖了 P12-P16 实施中发现的所有调整？特别是 P14（kani 5-marker）、P15（rocq-of-rust gate 6）的"实测 falsify 推荐"是否被吸纳。
4. **§7.1 约束对象明示**：principles.md / 项目长期记忆都反复强调"克制工具评判"——但本文 §7.1 又说"对实测数据本身的诚实汇总不约束"。这两者边界是否清晰？
5. **§六 / §八 禁忌**是否覆盖 P15 ror-typecheck 等新工具入口属性？

### 用什么方法 / 角度审查？

- **范围一致性**：本文范围声明（自我性）vs 内容覆盖（是否漏覆盖范围内的事）
- **论证标准周延性**：§3 / §4 给的"论证形式"是否对未来集成有充分指导
- **与实施现状对齐**：grep 实际工具的 oracle 形态（wrapper.sh）是否反映到本文
- **与 principles 的交叉对照**：是否有引用 principles 错误章节号、是否复述了 principles 但未明示

---

## §2 审查方法

### 参照源

- `docs/design/tool-integration.md` 全文
- `docs/design/principles.md` §三-3 + §六 + §八
- 实际工具 README：`tools/{kani,rocq-of-rust,rocq-of-rust-typecheck,prusti,hax-lean,creusot}/README.md`
- 实际工具 wrapper：`tools/{kani,rocq-of-rust,rocq-of-rust-typecheck,prusti,verifast,aeneas-*}/<name>-wrapper.sh`
- `docs/fixes/oracle-leak-audit-2026-05-08.md` + `oracle-leak-audit-2-2026-05-11.md`（实测 falsify 历史）
- `docs/fixes/oracle-leak-rules-implementation-{1,2}-2026-05-{08,11}.md`（实施记录）

### 恶意角度具体实施

- 假设 §3 0 误报论证不周延 —— 找具体工具不能套用任何一种论证形式的反例
- 假设 §4 0 漏报防御不周延 —— 找实际 wrapper 实施已采用但本文没说的防御机制
- 假设 §五 README 必含章节缺反作弊（§六-4）声明
- 假设 §七 实测报告"约束对象"边界被自我矛盾

### 诚实底线

每条问题给出文件路径 + 行号 + 引文 + 推理链。

---

## §3 审查现象

### #1（严重度: 高）—— "rocq-of-rust 5 道门"过时 + gate 6 描述缺漏

**现象**：

`tool-integration.md` line 28-30：

```
- **exit code 单一信号**（如 cargo-check / kani / verus / aeneas / charon）
- **exit code + 产物字面 grep**（如 hax × 3 / rocq-of-rust）
- **exit code + stderr 模式 grep**（如 kmir 的 `#EndProgram ~> .K`）
- **多门组合**（如 rocq-of-rust 的 5 道门：exit 0 + 至少一个 .v + 无 0-byte + > 200B + 无 silent marker）
```

但实际 `tools/rocq-of-rust/rocq-of-rust-wrapper.sh` line 55-61：

```
# Oracle (still 6 gates — wrapper is gate-equivalent to the old tool.toml,
#         just samples N times and AND-reduces):
#   1. rocq-of-rust exit code = 0 (every run)
#   2. ≥ 1 .v product per run
#   3. no zero-byte .v product
#   4. ≥ 1 .v product > 200 bytes
#   5. no failure marker in any .v
#   6. entry_fn appears as `Definition <fn>` in some .v product — must
#      hold on EVERY of the N attempts.
```

且 `principles.md` line 95 已对齐 6 道门。

**违反 / 嫌疑**：

- tool-integration.md line 30 落后于 principles 与实施 —— gate 6（entry_fn 出现校验）由 P15 引入（`docs/fixes/oracle-leak-rules-implementation-2-2026-05-11.md` §3.2），本文未追
- 这条直接错误，必须改

**推理链**：

- gate 6 是 P15 加入的反 silent-skip 闸门，属"防漏报机制"的实测加固
- 本文 §四 4.2 防漏报机制描述需要覆盖这一情况
- 但 §二 形式指标举例 line 30 还是 "5 道门"——内容上 stale

**决策性**：☐ 非决策点（直接改）

**建议**：line 30 改：

```
- **多门组合**（如 rocq-of-rust 的 6 道门：exit 0 + 至少一个 .v + 无 0-byte + > 200B + 无 silent marker + entry_fn 必须以 `Definition <fn>` 形式出现于 .v 产物——后者用于防 silent skip-item 漏报，由 P15 引入）
```

---

### #2（严重度: 高）—— §五 README 必含章节清单缺 "反作弊（§六-4）声明"

**现象**：

`tool-integration.md` line 120-130 § 五 README 必含章节清单：

```
1. **简介** + GitHub 上游 URL + 工具的 elevator pitch
2. **本测试集中的"前端接受"定义** + pipeline 阶段图 + 我方切割点
3. **SUCCESS 信号** —— 按 §二 形式指标
4. **形式严格性** —— 按 §三 / §四 声明 0 误报 / 0 漏报状态 + 防漏报机制 + 漏报盲点
5. **安装** —— 锁定版本，按 §一
6. **本框架配置** —— `tool.toml` 关键参数 + `entry_mode` / `extra_cargo_deps` / `version_command` 等
7. **已知限制 / 坑** —— 平台限制、依赖限制、工具自身已知 bug
8. **关联 sub-tests** —— 是否有 `examples/<tool>-limit/` 类目
```

但 `principles.md` line 261-265 § 六-4 反作弊推论：

```
### 硬指标 4：反作弊推论

dry-run flag 必须使工具**真的拿样例代码喂自己的前端**，不能把代码绕路给 stock rustc 走 cargo-check 等价路径。否则 SUCCESS 信号退化为"rustc parses it"——所有工具会齐刷刷高分，丢失工具间的特性子集分化信号。

最典型例子：Verus harness 的 `mod __ts_inner` **必须**写在 `verus! { }` 块内；写在块外，Verus 把 mod 内容直接当 stock Rust 透传 rustc，等于把它降级成 cargo-check（详见 `tools/verus/README.md`）。
```

**违反 / 嫌疑**：

- 宪法 §六-4 反作弊是硬指标——但 tool-integration §五 README 必含章节清单**未要求集成者声明"已防止 cargo-check 退化"**
- 这意味着新工具集成者按本文 §五 写 README 时，可能完全忽略反作弊证据
- §六-4 反作弊场景在 hax / aeneas / verifast / verus / creusot 等工具上都可能出现（凡是依赖 cargo 子命令、可能被工具透传给 stock rustc 的工具）

**推理链**：

- principles.md §六-4 是宪法硬指标
- tool-integration.md 是"我方维护 tools 集成"的方法学
- 方法学必须把宪法硬指标全覆盖到 README 必含章节中
- 当前 §五 章节 2（"前端接受"定义 + pipeline 阶段图 + 我方切割点）只覆盖了 §六-1 前端支持性观察（切割落点）+ §六-2 不允许 partial（"我方切割点"暗含 SUCCESS 在前端层完整完成）
- 但 §六-4 反作弊（dry-run flag 必须真喂前端）没有专门 README 章节承载

**决策性**：☑ **决策点** —— 需要用户审议：

- 是否在 §五 章节 2 中扩展为"前端接受定义 + pipeline 阶段图 + 我方切割点 + **反作弊证据（确证 dry-run 真喂前端，不退化为 cargo-check 等价）**"？
- 或加新章节 9 "反作弊证据"？

**建议**：§五 章节 2 改为：

```
2. **本测试集中的"前端接受"定义** + pipeline 阶段图 + 我方切割点 + **反作弊证据**
   - 前端接受定义：工具自身的前端 / 后端边界
   - pipeline 阶段图：工具内部 pipeline 关键阶段
   - 我方切割点：本框架在哪个阶段停（按 §六-1 落点）
   - 反作弊证据：dry-run flag 确证真喂工具前端，不退化为 cargo-check（按 §六-4），典型例 verus mod __ts_inner 写在 verus! {} 块内
```

---

### #3（严重度: 高）—— §三 0 误报论证标准对未来新工具有缝隙

**现象**：

`tool-integration.md` line 36-58 §三 0 误报论证：

```
**0 误报的定义**：oracle 判 SUCCESS 时一定是真 SUCCESS（不冤枉工具能力）。

**这是必须论证的硬指标**（按宪法 §三-3-2.b 上限保证）。论证形式不限定——**任何能提供足够证据的方式都可**：

- **反向证明**（最常见、最简单）：论证 oracle FAILED 时一定说明工具内部有问题
- **源码层穷尽**：grep 工具源码所有 oracle FAILED 触发点，每点对应真 partial
- **双向蕴含**：oracle SUCCESS ⇔ 工具内部某条断言成立（更强但代价大）
- **实测验证**：在足够多样的合法 SUCCESS 样例上验证 oracle 不误报（不构成形式证明，但提供足够经验证据时可接受）
```

**违反 / 嫌疑**：

- "实测验证"被列为合法论证形式之一——但**多少样本算"足够多样"？多少个反例才算"经验证据足够"？**本文未明示
- 这条缝隙在 P14（kani 5-marker）发现时被实测打击过：原假设 7 markers，实测 caller_location + foreign function 在常见 std-using entry 上"齐刷刷误报"（按 `docs/fixes/oracle-leak-audit-2-2026-05-11.md` §3.1）
- 即：「实测验证」标准下，"足够多样"由集成者主观判断——但实测 corpus 的规模和分布若不充分，会留漏报或误报盲点

**推理链**：

- §三 line 45 列出"实测验证"标准缺操作性
- P14 / P15 实测都发生了"原以为已论证 0 误报，实测打击"的 falsification 案例（kani 7-marker 削到 5-marker、ror gate 6 加入）
- 本文应吸纳这一历史并把"实测验证"的最低要求显式化

**决策性**：☑ **决策点** —— 需要用户审议：

- 是否在 §三 加最低标准：「实测验证 corpus 覆盖至少 N 个 std-using entry + 至少 M 个 self-contained entry，且包含 K 个已知反例」？
- 或者保留模糊性，让集成者按工具特性自行裁定？

**建议**：line 45 改为：

```
- **实测验证**：在足够多样的合法 SUCCESS 样例上验证 oracle 不误报（不构成形式证明，但提供足够经验证据时可接受）。
  **实测验证的最低要求**（按 P14 / P15 实测 falsify 经验）：
  - corpus 至少包含工具的典型应用场景样例（≥ 20）
  - 覆盖 std 依赖 / 无 std / async / unsafe 等典型 std-using 分类
  - 在 oracle 调整后必须重跑全 corpus 验证"调整前 SUCCESS 现 FAILED"与"调整前 FAILED 现 SUCCESS"两个方向都未引入新偏差
  - 任何调整都进 docs/fixes/<topic>-{audit,implementation}-<date>.md 留底
```

---

### #4（严重度: 中）—— §4.2 反误报双向实测的"两边都通过"概念可能过强

**现象**：

`tool-integration.md` line 84-96 § 4.2 防漏报机制 + 反误报检查：

```
**关键约束（与 §三 0 误报硬指标共享）**：防漏报机制**绝不能反向引入误报**。每个 grep marker 选取必须经过**双向实测**：

- 已知 silent path → grep 命中（防漏报有效）
- 合法代码 / 合法注释 / prelude / 用户合法字面 → grep 不命中（不引入误报）

**两边都通过**才能上线。例：hax-lean 的 grep `(:=|pure|mk|,)\s*sorry\b|\bsorry\s*[,)\]]` 经实测：
- `(pure sorry)` / `mk sorry` 命中 ✓
- 用户合法 `let sorry : i32 := 5;` + doc comment 含 sorry 字面 不命中 ✓

**不能两边都通过的 marker 不准上线**——宁可保留漏报盲点（按 §4.4 诚实声明），不可引入误报。
```

**违反 / 嫌疑**：

- "两边都通过"是 0 误报 absolute 标准
- 但实际 P14 kani 5-marker 实施时，对 caller_location / foreign function 两类 marker 走的是"承认会误报，所以删 marker"——不是改 marker
- 即：实际工程上 "两边都通过" 等于 "或者改进 marker、或者降级删 marker"。本文 §4.2 line 96 暗示前者，没明示后者

**推理链**：

- §4.2 line 96「**不能两边都通过的 marker 不准上线**——宁可保留漏报盲点（按 §4.4 诚实声明），不可引入误报」—— 这是"删 marker"路径的合理表达
- 但读者可能误读为"必须先改 marker 到两边都通过才能上线"——实际工程上完全允许"降级删 marker + 显式声明漏报盲点"
- 这条措辞稍显歧义

**决策性**：☐ 非决策点（措辞小修）

**建议**：line 96 末尾加一句「具体取舍：(a) 调整 marker 模式直到两边都通过 / (b) 删该 marker + 在 §4.4 漏报盲点中显式声明该类 silent path 未抓——两者任选其一」

---

### #5（严重度: 中）—— §四 4.1 形式证明 0 漏报例子不完整 + 与实施不符

**现象**：

`tool-integration.md` line 79-82 §4.1 形式证明 0 漏报例子：

```
- **aeneas**：唯一 unsupported 入口是 `craise`，唯一 exit 决定是 `Errors.error_list` 非空 → exit 1，单一通路 ✅
- **charon `--abort-on-error`**：`register_error!` 是唯一 unsupported 入口，加 abort 后 panic 是唯一 exit 决定 ✅
- **creusot**：所有 unsupported 用 `crash_and_error / span_err / span_fatal` 升 rustc error，单一通路 ✅
```

**违反 / 嫌疑**：

- aeneas 实际有 4 个 wrapper（aeneas-coq / aeneas-fstar / aeneas-hol4 / aeneas-lean）—— 文中说 aeneas 形式证明 0 漏报，但 4 个 backend 实际行为是否一致？
- 实际 aeneas-lean-wrapper.sh / aeneas-coq-wrapper.sh 是否仅依赖 exit code？还是还加了产物校验？未在 §4.1 反映

**推理链**：

- §4.1 是"形式证明 0 漏报"理论分析，应给出"单一通路"的精准论证依据
- aeneas 4 backend 实际 wrapper 是否仍走 exit code 单一通路需要核查；如有 wrapper 加产物校验，则 aeneas 已升级为"§4.2 防漏报机制"而非 §4.1 形式证明
- 当前文字 line 79 把 aeneas 列入 §4.1 暗示 4 backend 都是 exit code 单一通路；如不符则错误

**决策性**：☐ 非决策点（核查后修）

**建议**：核实 aeneas 4 backend wrapper 实际行为，把它们正确归类到 §4.1 或 §4.2。

---

### #6（严重度: 中）—— §7.1 约束对象明示与 principles 重复 + 措辞冗长

**现象**：

`tool-integration.md` line 149-163 § 7.1 约束对象明示：

```
`principles.md` / `docs/design/` 的"克制工具评判"精神，**约束对象**是：

- 工具集成的**设计认知** —— 避免把实测当作"工具能力客观判定"灌进设计层
- **后续基于本框架开发**的人 / AI —— 避免把数字硬塞进工具改进 / 认知模型
- 对工具开发者的**外溢免责** —— 挂在 [`README.md`](../../README.md) 顶部一次性陈述（避免无意得罪）

**不约束**：

- 报告书写者**对实测数据本身的诚实汇总与认知陈述**——基于数据归纳"X 在 corpus Y 下支持率 Z%，partial 模式集中在 W 类型"是诚实汇总，不是越界
- 报告书写者基于数据给出**自身认知与判断** —— 锚定 corpus / 时间 / 版本即可
- 报告**正文**无须反复自贬 / 加"不构成评判"声明 —— README.md 一次性免责已足够，正文写自然些

**不要**把"克制评判"误传到报告书写本身上，把报告写成无认知的快照陈列——那不是谨慎，是放弃汇总责任。
```

**违反 / 嫌疑**：

- 这段文字在"自我性"层做了关键区分：约束对象 = 设计认知 + 后续开发者 + 工具开发者免责；不约束对象 = 报告作者对数据的诚实汇总
- 但本段引用 `principles.md / docs/design/` 时，**只在精神层叙述**，没具体引到 principles.md §三-3-原则 3 line 122-130
- 风险：未来读者可能怀疑"7.1 是否得到 principles 明确授权"——其实 principles §三-3-原则 3 line 128 已说"报告内部分析（特别是失败模式归类、跨工具发现等）**不构成对工具的实质评判**——**不是工具本身的责任，工具开发者无须为内容负责**"，与 §7.1 一致

**推理链**：

- 这是 design 之间的"半引用半重述"——清晰度损失
- 建议直接引用 principles §三-3-原则 3，再加 §7.1 的"不约束"细化

**决策性**：☐ 非决策点（措辞小修）

**建议**：§7.1 开头改为「principles.md §三-3-原则 3（line 122-130）的'非评判性'+'真实性诚实责任'约束的对象是...」，明示引用关系。

---

### #7（严重度: 中）—— §六 禁忌未覆盖 P15 wrapper 模式的新陷阱

**现象**：

`tool-integration.md` line 134-141 § 六 禁忌：

```
工具 README **禁止**：

- 对工具能力下绝对结论（如"工具 X 不支持特性 Y"）—— 改为"我方测试方法学下，X 在本 corpus 的特性 Y 上 FAILED"
- 暗示客观真理（如"工具 X 是错的"）
- 跨工具能力排序（如"X 比 Y 更好"）—— 框架不评比，只测 entry 级二值信号

按 [`principles.md`](principles.md) §三-3-1 时效性，所有工具陈述都锚定具体时间 + 工具版本组合，不构成长期承诺。
```

**违反 / 嫌疑**：

- §六 禁忌是 README 书写规范——但当前覆盖只针对"言论性禁忌"
- P15 wrapper 模式（rocq-of-rust 用 N 次重试 + 6 道门）引入新陷阱：**wrapper 自己可能引入非确定性**（如 N=3 vs N=7 给出不同 catch rate）
- 这条陷阱在 README 应被明示："wrapper 的非确定性参数（N、阈值等）需在 README 注明、并实测论证"
- 但本文 §六 / §五 都没说

**推理链**：

- wrapper.sh 是"集成者描述工具自身行为"的延伸（principles.md A-4 / line 188）
- wrapper 内部参数（N、阈值）属于"我方测试方法学"的可调参数——必须在 README 公开
- 否则未来 wrapper 内部参数变化时，外部读者无法溯源 "为什么 N=7 不是 N=5"

**决策性**：☑ **决策点** —— 需要用户审议：

- 是否在 §五 章节 4 "形式严格性"中扩展为"... + wrapper 非确定性参数（如重试次数、阈值）+ 选 N=7 的实测论证"？
- 或加新章节 9 "wrapper.sh 设计细节"？

**建议**：§五 章节 4 后扩展：

```
4. **形式严格性** —— 按 §三 / §四 声明 0 误报 / 0 漏报状态 + 防漏报机制 + 漏报盲点
4.b （如使用 wrapper.sh） **wrapper 设计细节** —— wrapper 内部参数（重试次数 N、阈值、产物校验路径等）的实测论证；wrapper 与裸 tool.toml 调用的等价性证据
```

---

### #8（严重度: 中）—— §一 "tool.toml 必须含 version_command" 与 detailed-design "可选" 表面冲突

**现象**：

- `tool-integration.md` line 19：「`tool.toml` 必须含 `version_command`——runner 在每次 run 自动捕获工具版本字符串到 `results.json` metadata 段，按宪法 §五'工具非静态原则'裸数据自描述。」
- `detailed-design.md` line 121-130：「`version_command  # 可选，默认 []`」

**违反 / 嫌疑**：

- tool-integration: 必须含
- detailed-design: 可选
- 表面矛盾，实际是位阶不同（前者是方法学约束 / 后者是 schema 约束）但措辞未澄清

**推理链**：

- 见 principles-review.md #X6（同源）

**决策性**：☐ 非决策点（措辞澄清）

**建议**：line 19 改：「按宪法 §五'工具非静态原则'+ 自我性方法学约束，**本项目所有自集成的 tool.toml 必须**含 `version_command`（runner schema 接受空数组以让第三方复用框架灵活配置，但本项目自集成的工具不接受空）」

---

### #9（严重度: 低）—— §二 形式指标列表未明示"多门组合"的合法上限

**现象**：

`tool-integration.md` line 25-33 § 二 SUCCESS 信号的形式指标声明：

```
每个 `tools/<name>/README.md` 必须明示"形式指标"——即 oracle 用什么具体可机检条件判定 SUCCESS。形式指标可以是：

- **exit code 单一信号**（如 cargo-check / kani / verus / aeneas / charon）
- **exit code + 产物字面 grep**（如 hax × 3 / rocq-of-rust）
- **exit code + stderr 模式 grep**（如 kmir 的 `#EndProgram ~> .K`）
- **多门组合**（如 rocq-of-rust 的 5 道门：...）
```

**违反 / 嫌疑**：

- "形式指标"分类列出 4 种但未明示是否还有第 5 类（如 stdout grep、duration 阈值等）
- 实际 wrapper 实现中可能用更复杂判定（如 prusti-strict-wrapper.sh 的 .vpr 文件数判定——属"产物存在性 + count 阈值"，与 line 28 "产物字面 grep"略有不同）
- 文档应明示"形式指标"是开放分类

**决策性**：☐ 非决策点（措辞小修）

**建议**：line 32 后加「**本列表是举例，非穷举**——任何具体可机检的条件组合都属形式指标范畴。若使用本列表外的新形式指标，需在工具 README 中明示其判定逻辑」

---

### #10（严重度: 低）—— §六 禁忌未明示"反作弊推论"的负面表述

**现象**：

`tool-integration.md` line 134-141 § 六 禁忌仅约束"言论"，不约束"oracle 设计本身"。

但实际：oracle 设计层面的最大禁忌是"`dry-run flag 退化为 cargo-check`"（principles.md §六-4 反作弊）—— 当 oracle 设计出错时，整工具 SUCCESS 数字全部退化为 cargo-check 数字，工具间分化信号完全丢失。

**违反 / 嫌疑**：

- §六 禁忌应同时含"言论禁忌"+"oracle 设计禁忌"
- 当前仅前者

**推理链**：

- §六-4 是宪法硬指标，本文 §五 必含章节也应反映（见本文 #2）
- §六 禁忌同步加 oracle 设计禁忌

**决策性**：☐ 非决策点（与 #2 同源）

**建议**：§六 加一条禁忌「**oracle 不允许退化为 cargo-check 等价**——dry-run flag / harness 形态必须使工具真喂前端而非透传 stock rustc（按 §六-4 反作弊）」

---

## §4 决策点 vs 非决策点汇总

### 决策点（需用户审 / 拍板）

| # | 摘要 |
|---|---|
| #2 | §五 README 必含章节加反作弊证据要求 |
| #3 | §三 0 误报论证"实测验证"最低标准是否明示 |
| #7 | §五 加 wrapper 设计细节章节 |

### 非决策点（局部 fix）

| # | 摘要 |
|---|---|
| #1 | rocq-of-rust 5 道门 → 6 道门 + gate 6 描述 |
| #4 | §4.2 "两边都通过"补充"删 marker"路径 |
| #5 | aeneas 4 backend 在 §4.1 归类核查 |
| #6 | §7.1 引用 principles 明示化 |
| #8 | §一 version_command 必/可选位阶澄清 |
| #9 | §二 形式指标列表标"非穷举" |
| #10 | §六 禁忌加"oracle 设计层面"禁忌 |

---

## §5 审查结论

### 总体判断

`tool-integration.md` 主体方法学正确、论证套路严谨——0 误报 / 0 漏报二元论证、双向实测约束、漏报盲点诚实声明都对齐 principles.md 的诚实测试范围精神。但存在三类问题：

1. **覆盖缺失**（高严重度）：§五 README 必含章节不覆盖 §六-4 反作弊声明；§三 0 误报论证标准对"实测验证"标准模糊，缺 P14/P15 实测 falsify 历史的最低要求；§六 禁忌仅"言论禁忌"未覆盖"oracle 设计禁忌"。
2. **实施漂移**（高严重度）：rocq-of-rust 道门数 5 → 6 未追；wrapper.sh 模式（特别是 N 次重试参数）作为 P15 引入物未在 §五 章节清单反映。
3. **小漂移**（中低严重度）：与 principles 引用关系未明示 / 与 detailed-design 位阶冲突未澄清 / 形式指标列表未标非穷举 / aeneas 4 backend 在 §4.1 归类待核。

### 严重度分布

- 高严重度：3 条（#1 + #2 + #3）
- 中严重度：4 条（#4 + #5 + #6 + #7 + #8）
- 低严重度：2 条（#9 + #10）

### 关键风险

最严重风险是 **#2**：§五 README 必含章节不要求反作弊证据——这让宪法 §六-4 反作弊推论在新工具集成时**可能被遗漏**。未来集成者按 §五 章节顺序写 README，可能完全没"反作弊"一节，导致 oracle 退化为 cargo-check 等价的风险存在但不被捕捉。

次严重风险是 **#3**：0 误报论证"实测验证"标准模糊。P14（kani 7→5 markers）/ P15（rocq-of-rust 加 gate 6）都是"原以为已论证 0 误报，实测打击后修正"的案例——这条历史经验应吸纳进本文，告诉未来集成者"实测验证"的最低 corpus 规模和反例覆盖标准。

### 与 principles 的派生关系

整体派生顺畅。但 §二 / §三 / §四 中对 principles.md 的引用大多停留在精神层，未给出具体章节锚点（line 引用）。这条不是宪法违反，但写作风格可改进。
