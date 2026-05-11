# aeneas-hol4 readme Review

## §1 问题意识

aeneas-hol4 README 与 aeneas-lean / aeneas-coq / aeneas-fstar 结构同形，但**新增"HOL4 backend 的硬天花板"段**（L20-37）——诚实声明 aeneas-hol4 ~37% 通过率显著低于其他 3 个 backend 的单一根因。审查重点：硬天花板段是否合规（陈述工具事实 vs 越界给能力下结论）？

## §2 审查方法

参照源：

- [`docs/fixes/audit-2026-05-11/tools/aeneas-lean/readme-review.md`](../aeneas-lean/readme-review.md) shared issues；
- [`tools/aeneas-hol4/README.md`](../../../../tools/aeneas-hol4/README.md)；
- [`docs/design/tool-integration.md`](../../../design/tool-integration.md) §六 禁忌（绝对结论 / 跨工具排序）；
- [`deep-reports/cc-reports/aeneas-hol4.md`](../../../../deep-reports/cc-reports/aeneas-hol4.md)。

## §3 审查现象

#1 (严重度: 高) — README L26-30 "硬天花板"段含具体源码行号 + 通过率数字——临近"对工具能力下绝对结论"边界

**现象**：[`tools/aeneas-hol4/README.md`](../../../../tools/aeneas-hol4/README.md) L20-30：

> ### HOL4 backend 的硬天花板
> 矩阵上 aeneas-hol4 ~37% 显著低于其它 3 个 backend (~60%)，**单一根因**在 `Extract.ml + ExtractBase.ml` 的 backend 分支：
> - `ExtractBase.ml:1412 type_decl_kind_to_qualif` 在 HOL4 上 trait decl 始终返回 `None`
> - `Extract.ml:3166 extract_trait_decl` 调 `Option.get` 不防御
> - **LLBC 里只要含 ≥1 个 trait declaration（`FnOnce` / `From` / `Iterator`...）就抛 `Invalid_argument "option is None"` 截断产物**
> - 与 entry 是否真"用"该 trait 无关，与下游 prover 无关
> 矩阵 60/60/60/37 的差距 ≈ "矩阵上含 trait declaration 的 entry 比例"。这是 aeneas-hol4 upstream 的硬天花板，作为工具事实陈述。

**违反**：[`tool-integration.md`](../../../design/tool-integration.md) §六禁忌——"对工具能力下绝对结论"、"跨工具能力排序"。

具体审视：
- "aeneas-hol4 ~37% 显著低于其它 3 个 backend (~60%)" —— 是跨 backend 比较，**接近 §六禁忌的"跨工具能力排序"**。但 README 注释"作为工具事实陈述"——它是描述上游 aeneas 自身 backend 实现状态，不是评比；
- 引用上游源码具体行号（`ExtractBase.ml:1412` / `Extract.ml:3166`），这是 [`tool-integration.md`](../../../design/tool-integration.md) §三-源码层穷尽允许的论证形式。

**推理链**：这段属于"诚实陈述实测结果 + 工具自身已知 bug"的边界——合规但临近禁忌线。措辞"硬天花板" / "单一根因" / "矩阵 60/60/60/37 的差距 ≈ ..." 比起按 §六禁忌的中性描述（"在本测试方法学下 aeneas-hol4 在含 trait declaration 的 entry 上 FAILED"）更具断定语气。

但 README 也明确说"是 aeneas-hol4 upstream 的硬天花板"——把责任归因到 upstream，不是测试方法学问题。这符合 [`principles.md`](../../../design/principles.md) §三-3-1 时效性。

**决策性**：决策点——是否调整措辞使更符合"非评判性"原则。

**建议**：可保留 60/60/60/37 数字（实测事实）+ 源码行号（upstream 实施），但建议调整措辞：

> 在本时点（aeneas commit `a14083a6`）下，aeneas-hol4 在含 trait declaration 的 entry 上 OCaml panic 截断产物。矩阵 60/60/60/37（lean / coq / fstar / hol4）的实测差距由 corpus 中含 trait decl 的 entry 比例驱动。这是 aeneas-hol4 在该 commit 上的实测状态，不构成长期承诺。

这避免"硬天花板" / "单一根因"等绝对化措辞。

#2 (严重度: 中) — README L37 "aeneas-hol4 的 FAILED 集合中 `trait_decl` panic 占比最高，是该 backend 通过率显著低于其它 3 个的单一原因"

**现象**：[`tools/aeneas-hol4/README.md`](../../../../tools/aeneas-hol4/README.md) L37：

> aeneas-hol4 的 FAILED 集合中 `trait_decl` panic 占比最高，是该 backend 通过率显著低于其它 3 个的单一原因。

**违反**：同 #1——这是跨 backend 比较 + 单一原因断言。

**推理链**：同 #1。"单一原因" 是强断言，按 §六禁忌应避免。但有上游源码层证据（ExtractBase.ml:1412），实测有数据支持，按 §三-源码层穷尽合规。

**决策性**：决策点。

**建议**：可改"是该 backend 通过率较其它 3 个偏低的主要观察因子"——保留实测结论，避免"单一原因"断言。

#3 (严重度: 低) — README L40-48 形式严格性段同 aeneas-lean

**现象**：[`tools/aeneas-hol4/README.md`](../../../../tools/aeneas-hol4/README.md) L40-48 形式严格性论证与 aeneas-lean 一致。L43 显式补"hol4 backend 还有 `trait_decl_kind_to_qualif` 触发的 OCaml panic（exit 2）—— 共同指向 hol4 backend 的硬天花板"——与 #1 同样涉及"硬天花板"措辞。

**违反**：同 #1。

**推理链**：形式严格性段本身合规——0 误报 / 0 漏报论证扎实（aeneas exit 0 ⇔ Errors.error_list 空 + 无 OCaml panic）。但"硬天花板"措辞重复。

**决策性**：决策点。

**建议**：与 #1 同步调整。

#4 (严重度: 低) — README L62 "Aeneas 中标记为成熟" —— 工具自陈

**现象**：[`tools/aeneas-hol4/README.md`](../../../../tools/aeneas-hol4/README.md) L72：

> HOL4 backend 在 Aeneas 中标记为成熟，但 `.sml` 加载到 HOL4 需要额外 `primitivesLib`

**违反**：未违反——"在 Aeneas 中标记为成熟"引用上游声明，不是测试结论。

**决策性**：非决策点。

**建议**：无须改。

#5 (严重度: 低) — 关联 sub-tests 段与其他 aeneas backend 一致

**现象**：[`tools/aeneas-hol4/README.md`](../../../../tools/aeneas-hol4/README.md) L78 与 aeneas-lean L76 文字完全相同。

**违反**：未违反。

**决策性**：非决策点。

**建议**：无须改。

## §4 决策点 vs 非决策点

- 决策点：3（"硬天花板"措辞 / "单一原因"断言 / 形式严格性段重复"硬天花板"）
- 非决策点：2

## §5 审查结论

aeneas-hol4 README 内容结构齐全，技术细节准确（源码行号有效）。但**"硬天花板"段措辞临近 [`tool-integration.md`](../../../design/tool-integration.md) §六禁忌的"绝对结论"边界**——"单一根因" / "硬天花板" / "矩阵 60/60/60/37 的差距 ≈ ..." 等措辞虽以上游源码为论据，但**断定语气过强**。

**关键建议**：

1. 把"硬天花板" / "单一根因"措辞调整为更中性的"在本时点实测状态" / "主要观察因子"；
2. 保留 60/60/60/37 数字 + 源码行号（实测证据 + upstream 实施事实）；
3. 明确措辞"本结论锚定 aeneas commit `a14083a6`，不构成长期承诺"。

这是 4 个 aeneas backend 中**唯一显著触及 §六禁忌边界**的 README。其他 3 个 backend README 措辞中性。
