# aeneas-coq readme Review

## §1 问题意识

aeneas-coq README 是 aeneas-lean README 的"复用版"——结构同形，差 backend specifics (Coq backend / `.v` 产物 / `Primitives.v` Axiom)。审查重点同 aeneas-lean，外加 Coq-specific 准确性。

## §2 审查方法

参照源：

- [`docs/fixes/audit-2026-05-11/tools/aeneas-lean/readme-review.md`](../aeneas-lean/readme-review.md) shared issues；
- [`tools/aeneas-coq/README.md`](../../../../tools/aeneas-coq/README.md)；
- [`deep-reports/cc-reports/aeneas-coq.md`](../../../../deep-reports/cc-reports/aeneas-coq.md)。

## §3 审查现象

#1 (严重度: 低) — README 结构与 aeneas-lean 一致

**现象**：[`tools/aeneas-coq/README.md`](../../../../tools/aeneas-coq/README.md) 8 章节按 [`tool-integration.md`](../../../design/tool-integration.md) §五齐全。

**违反**：未违反。

**决策性**：非决策点。

**建议**：无须改。

#2 (严重度: 低) — README L26 "`Primitives.v` 约 100 条 `Axiom` 是运行时库抽象（与翻译质量无关）"

**现象**：[`tools/aeneas-coq/README.md`](../../../../tools/aeneas-coq/README.md) L26：

> `Primitives.v` 约 100 条 `Axiom` 是运行时库抽象（与翻译质量无关）。成功 entry 的 `<Mod>.v` 里零 `Admitted` / `Axiom`。

**违反**：未违反——准确陈述 aeneas 设计。

**推理链**：与 aeneas-lean L34 "成功 entry 产物里零 sorry / Admitted" 体例一致。Coq backend 用 `Admitted` 而非 `sorry`（Lean 4 习语）。

**决策性**：非决策点。

**建议**：可选——同 aeneas-lean 建议，澄清"这是 aeneas 设计保证，非 oracle 实施"。

#3 (严重度: 低) — README L32-38 形式严格性

**现象**：[`tools/aeneas-coq/README.md`](../../../../tools/aeneas-coq/README.md) L32-38：

> **形式严格性 — 0 误报**：✅ 形式可证。aeneas exit 0 ⇔ `Errors.error_list` 空
> **形式严格性 — 0 漏报**：✅ 形式可证。所有 unsupported 都通过 `craise` push error_list

**违反**：未违反——形式证明同 aeneas-lean。

**决策性**：非决策点。

**建议**：无须改。

#4 (严重度: 低) — README L62 "Coq backend 文档覆盖相对 Lean 少；extraction 功能本身稳定，但 Coq proof 侧需要手工补充"

**现象**：[`tools/aeneas-coq/README.md`](../../../../tools/aeneas-coq/README.md) L62：

> Coq backend 文档覆盖相对 Lean 少；extraction 功能本身稳定，但 Coq proof 侧需要手工补充

**违反**：未违反——描述 aeneas Coq backend 文档相对成熟度，是工具自身陈述，非"工具能力评判"。"Coq proof 侧需要手工补充"是事实陈述（aeneas 不自动写 proof），不是测试结论。

**决策性**：非决策点。

**建议**：无须改。

#5 (严重度: 低) — README L64 "仅测 extraction（`aeneas` 退出码）；`coqc` 类型检查不在本 testsuite 范围内"

**现象**：[`tools/aeneas-coq/README.md`](../../../../tools/aeneas-coq/README.md) L64：

> 仅测 extraction（`aeneas` 退出码）；`coqc` 类型检查不在本 testsuite 范围内

**违反**：未违反——明确边界声明，与 rocq-of-rust-typecheck 的档 1 形成对比（后者真做 coqc）。aeneas-coq oracle 不做 coqc，是 [`principles.md`](../../../design/principles.md) §六-1 切线选择。

**推理链**：清晰边界声明是好的诚实做法。

**决策性**：非决策点。

**建议**：无须改。

#6 (严重度: 低) — 关联 sub-tests 段与 aeneas-lean 一致

**现象**：[`tools/aeneas-coq/README.md`](../../../../tools/aeneas-coq/README.md) L66-68 与 aeneas-lean L74-76 文字完全相同。

**违反**：未违反——4 个 aeneas backend 共享 limit 类目，措辞一致是预期。

**决策性**：非决策点。

**建议**：无须改。

## §4 决策点 vs 非决策点

- 决策点：0
- 非决策点：6

## §5 审查结论

aeneas-coq README 是 aeneas-lean 的标准复用，结构与论证质量同等。**无 critical 问题**。所有可选改进与 aeneas-lean review 共享。

Coq-specific 内容（Primitives.v Axiom / Coq backend 文档状态）准确。
