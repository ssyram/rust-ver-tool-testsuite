# hax-coq readme Review

## §1 问题意识

hax-coq README 重点：(1) Coq backend 9 个 reject phase 列表（比 fstar 6 个 / lean 4 个多）；(2) 第二轮 audit 增加的 silent path B（`(* NotImplementedYet *)` boilerplate marker）+ entry_fn 存在性 grep 封堵；(3) 形式严格性 ✅ 论证。

恶意角度考察：是否对 backend 通过率（~67%）做绝对评判？9 个 reject phase 列表是否准确？

## §2 审查方法

参照源：

- [`docs/design/tool-integration.md`](../../../design/tool-integration.md) §五 / §六；
- [`tools/hax-coq/tool.toml`](../../../../tools/hax-coq/tool.toml) 对照实施；
- [`docs/fixes/oracle-leak-audit-2-2026-05-11.md`](../../oracle-leak-audit-2-2026-05-11.md) §3.3；
- [`deep-reports/cc-reports/hax-coq.md`](../../../../deep-reports/cc-reports/hax-coq.md)。

## §3 审查现象

#1 (严重度: 中) — README L20 "Coq backend 是 hax 中 reject phase 最多的（9 个）"——临近"跨 backend 比较"

**现象**：[`tools/hax-coq/README.md`](../../../../tools/hax-coq/README.md) L20：

> Coq backend 是 hax 中 reject phase 最多的（9 个：`Reject.Unsafe`、`RawOrMutPointer`、`Arbitrary_lhs`、`Continue ×2`、`EarlyExit`、`As_pattern`、`Dyn`、`Trait_item_default`），所以矩阵通过率 ~67%（低于 lean 89% / fstar 79%）。

**违反**：[`tool-integration.md`](../../../design/tool-integration.md) §六禁忌"跨工具能力排序"。这条措辞描述 hax 内部不同 backend 之间的 reject phase 数量对比 + 通过率比较——临近跨工具排序。

**推理链**：但 README 也实测锚定具体数字（67% / 79% / 89%）——这是某 corpus / 时点的实测快照，不是绝对评比。措辞"通过率 ~67%（低于 lean 89% / fstar 79%）"是事实描述。

与 aeneas-hol4 README "硬天花板" 比较：aeneas-hol4 说"60/60/60/37 单一根因" 更强；hax-coq 这里说"9 个 reject phase / ~67%"——相对客观。

按 §三-3-1 时效性，需要锚定 corpus + 时点——README 没明确"该数字锚定 commit 30949eb + 本 corpus"。

**决策性**：决策点——是否补"该数字锚定 commit 30949eb + 本 corpus"以避免长期承诺误解。

**建议**：补一句"本通过率数字锚定 hax commit `30949eb` + 本 corpus 实测，随上游版本演进与 corpus 变化浮动"。

#2 (严重度: 中) — README L20 "9 个 reject phase" 列表准确性

**现象**：[`tools/hax-coq/README.md`](../../../../tools/hax-coq/README.md) L20：

> 9 个：`Reject.Unsafe`、`RawOrMutPointer`、`Arbitrary_lhs`、`Continue ×2`、`EarlyExit`、`As_pattern`、`Dyn`、`Trait_item_default`

**违反**：未违反——按 [`tool-integration.md`](../../../design/tool-integration.md) §三-源码层穷尽，列出 hax-coq 上游 reject phase 列表是诚实陈述。

**推理链**：这是 hax upstream commit `30949eb` 中 Coq backend 的 reject phase 模块清单。锚定 commit。

**决策性**：非决策点——列表是事实陈述。

**建议**：无须改。

#3 (严重度: 低) — README L22 "Coq printer 的 silent fallback 路径" 引用源码行号

**现象**：[`tools/hax-coq/README.md`](../../../../tools/hax-coq/README.md) L22-23：

> > **Coq printer 的 silent fallback 路径**：上游源码 `engine/backends/coq/coq/coq_backend.ml:137` 的 `default_document_for s = "TODO: please implement the method `..."` 是纯文本输出，**不发 Diagnostic**——cargo hax 仍 exit 0 但 .v 文件里散布 `"TODO: please implement..."` 字面字符串。当前 oracle 不抓这种情况。

**违反**：未违反。但 L22 写"当前 oracle 不抓这种情况"——实际上 tool.toml L27 grep `please implement the method` 抓——README 与实施有矛盾？

读 tool.toml L27 grep：`failure \\(\\(|please implement the method`——是抓的。

读 README L26："SUCCESS = cargo hax exit 0 **且** 产物 grep 不命中 `failure ((` / `please implement the method` **且** entry_fn 在 .v 中有定义"——也是抓的。

L22 "当前 oracle 不抓这种情况" 与 L26 / tool.toml 矛盾——内部矛盾。

**违反**：内部矛盾——README L22 表述错误。

**推理链**：可能 L22 是旧版（早期 oracle 不抓），后来 oracle 加了 grep 但 L22 没同步更新。

**决策性**：决策点——L22 需修正。

**建议**：把 L22 "当前 oracle 不抓这种情况"删除或改为"当前 oracle 通过 §SUCCESS 信号段的 grep 抓"。这是 README 内部矛盾，应修正。

#4 (严重度: 中) — README L33-35 形式严格性用 ✅ 实测 + 设计层论证

**现象**：[`tools/hax-coq/README.md`](../../../../tools/hax-coq/README.md) L33-35：

> **形式严格性 — 0 误报**：✅ 实测 + 设计层论证 0 误报...
> **形式严格性 — 0 漏报**：✅ 实测 + 源码层封堵...

**违反**：与 hax-coq config-review.md #4 关联——✅ vs ⚠️ 措辞。当前 ✅ 论证是"源码层穷尽 + 双向实测"，比纯 grep heuristics 强但仍依赖 grep。

**推理链**：合规——hax-coq 的 entry_fn 存在性 grep 经过双向实测验证 5 类 case（Definition / Fixpoint / Lemma / 嵌套 module / missing），覆盖完整。✅ 标注可接受。

**决策性**：非决策点（与 hax-coq config #4 重叠）。

**建议**：与 config-review.md #4 同步——保留 ✅ 但补说明"这是源码层穷尽 + 实测的强经验论证，非形式证明"。

#5 (严重度: 低) — README L65-66 已知限制 / 坑 描述准确

**现象**：[`tools/hax-coq/README.md`](../../../../tools/hax-coq/README.md) L65-66：

> - Coq backend 在 hax upstream 标记为 **partial**：生成的 `.v` 文件中可能含 `(* NotImplementedYet *)` 注释占位...
> - 生成的 `_CoqProject` 中库名为 `TODO`（`-R ./ TODO`），未配合 hax Coq 支持库时不能直接 `coqc` 编译——这是 upstream 的预期行为

**违反**：未违反——工具自身限制陈述。

**推理链**：诚实声明 "Coq backend 在 hax upstream 标记为 partial"——锚定 upstream 标记，不是测试结论。

**决策性**：非决策点。

**建议**：无须改。

#6 (严重度: 低) — 关联 sub-tests 与 hax-lean / hax-fstar 共享

**现象**：[`tools/hax-coq/README.md`](../../../../tools/hax-coq/README.md) L72 与 hax-lean / hax-fstar 一致。

**违反**：未违反。

**决策性**：非决策点。

**建议**：无须改。

## §4 决策点 vs 非决策点

- 决策点：3（L20 通过率锚定 + L22 内部矛盾修正 + L33-35 措辞）
- 非决策点：3

## §5 审查结论

hax-coq README 内容详尽，2 轮 audit 论证完整。

**关键问题**：

1. **L22 内部矛盾**：写"当前 oracle 不抓这种情况"，但 L26 + tool.toml L27 实际是抓的——需要修正；
2. **L20 通过率措辞**：跨 backend 通过率比较（67% / 79% / 89%）应明确锚定 commit + corpus + 时点。

整体 oracle 设计经 2 轮 audit 收紧，论证扎实——但 README L22 是 critical typo 需修。
