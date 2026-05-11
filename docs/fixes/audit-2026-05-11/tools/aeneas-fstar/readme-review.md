# aeneas-fstar readme Review

## §1 问题意识

aeneas-fstar README 是 aeneas-lean README 的"复用版"——结构同形，差 F\* backend specifics（`.fst` 产物、F\* / Karamel 下游）。

## §2 审查方法

参照源：

- [`docs/fixes/audit-2026-05-11/tools/aeneas-lean/readme-review.md`](../aeneas-lean/readme-review.md) shared issues；
- [`tools/aeneas-fstar/README.md`](../../../../tools/aeneas-fstar/README.md)；
- [`deep-reports/cc-reports/aeneas-fstar.md`](../../../../deep-reports/cc-reports/aeneas-fstar.md)。

## §3 审查现象

#1 (严重度: 低) — README 8 章节齐全，体例与 aeneas-lean 一致

**现象**：[`tools/aeneas-fstar/README.md`](../../../../tools/aeneas-fstar/README.md) 整体结构齐全。

**违反**：未违反。

**决策性**：非决策点。

**建议**：无须改。

#2 (严重度: 低) — README 没有 aeneas-lean 中"成功 entry 产物零 sorry / Admitted" 的对应陈述

**现象**：[`tools/aeneas-fstar/README.md`](../../../../tools/aeneas-fstar/README.md) 缺与 aeneas-lean L34 对应的"`.fst` 产物零 admit" 说明。

**违反**：未违反——但 aeneas-lean 有这条声明体现 backend 特点，aeneas-fstar / aeneas-coq / aeneas-hol4 也应有同等清晰度。aeneas-coq 有 L26 "Primitives.v Axiom 与翻译质量无关"。

**推理链**：4 个 backend 中只有 aeneas-lean / aeneas-coq 显式说明这点，aeneas-fstar / aeneas-hol4 缺。F\* 的 admit primitive 是 `admit` keyword 或 `assume val`——README 未说明 F\* 产物如何处理类似情况。

**决策性**：非决策点——aeneas 设计不引入证明义务，所有 backend 同质。

**建议**：可选——加一句"成功 entry 的 `.fst` 中无 `admit` / `assume val`（aeneas backward function 模型纯函数化）"——与 aeneas-lean / aeneas-coq 对齐。

#3 (严重度: 低) — README L20 "F\* backend 在 hax 里有 6 个 reject phase" —— 这是 hax 的描述，不是 aeneas

**现象**：等等——这是 aeneas-fstar README，不是 hax-fstar。我需检验是否有错引内容。

读 [`tools/aeneas-fstar/README.md`](../../../../tools/aeneas-fstar/README.md) L19-20——实际上 aeneas-fstar README 没有提"hax 6 个 reject phase"。该陈述在 hax-fstar README L20。aeneas-fstar README L19 是"用户自己拿 `.fst` 给 F* / Karamel 做下游验证"。Confirmed：aeneas-fstar README 不混淆。

**违反**：未违反——审查发现无错引。

**决策性**：非决策点。

**建议**：无须改。

#4 (严重度: 低) — README L52-54 已知限制 / 坑 简明

**现象**：[`tools/aeneas-fstar/README.md`](../../../../tools/aeneas-fstar/README.md) L56-60：

> - macOS 上安装时需用 `gmake`（见 aeneas-lean 安装步骤）
> - F\* backend 在 Aeneas 中文档覆盖相对 Lean 少，但核心 extraction 功能同样稳定
> - charon-pin 路径独立于框架原 charon，切勿混用

**违反**：未违反。

**决策性**：非决策点。

**建议**：无须改。

#5 (严重度: 低) — README 形式严格性同 aeneas-lean

**现象**：L30-36 与 aeneas-lean / aeneas-coq 形式严格性论证一致：

> **形式严格性 — 0 误报**：✅ 形式可证。aeneas exit 0 ⇔ `Errors.error_list` 空
> **形式严格性 — 0 漏报**：✅ 形式可证。

**违反**：未违反。

**决策性**：非决策点。

**建议**：无须改。

## §4 决策点 vs 非决策点

- 决策点：0
- 非决策点：5

## §5 审查结论

aeneas-fstar README 是 aeneas-lean 的标准复用，结构与论证质量同等。无 critical 问题。可选改进：补充与 aeneas-lean L34 / aeneas-coq L26 对应的 ".fst 产物 admit 状态" 说明。
