# hax-fstar readme Review

## §1 问题意识

hax-fstar README 重点：(1) F\* backend 6 个 reject phase (介于 lean 4 / coq 9 之间)；(2) F\* backend "几乎不走 silent path" 的工具自陈；(3) 第二轮 audit 增加的 entry_fn 存在性 grep 封堵 fstar_backend.ml:1771 silent skip-item path。

## §2 审查方法

参照源：

- [`docs/fixes/audit-2026-05-11/tools/hax-coq/readme-review.md`](../hax-coq/readme-review.md) shared issues；
- [`tools/hax-fstar/README.md`](../../../../tools/hax-fstar/README.md)；
- [`deep-reports/cc-reports/hax-fstar.md`](../../../../deep-reports/cc-reports/hax-fstar.md)。

## §3 审查现象

#1 (严重度: 中) — README L20 跨 backend 通过率比较——同 hax-coq L20

**现象**：[`tools/hax-fstar/README.md`](../../../../tools/hax-fstar/README.md) L20：

> F* backend 在 hax 里有 6 个 reject phase（介于 lean 4 与 coq 9 之间）：`RawOrMutPointer` / `Reject_impl_type_method` / `Arbitrary_lhs` / `Question_mark` / `As_pattern` / `Trait_item_default`。printer 自身也有多处 `Error.unimplemented` 抛异常路径，叠加后通过率 ~79%（lean 89% / coq 67% 中间）。

**违反**：[`tool-integration.md`](../../../design/tool-integration.md) §六禁忌——临近"跨工具能力排序"。

**推理链**：与 hax-coq readme-review.md #1 同——锚定 commit + corpus + 时点的实测数字是合规的事实陈述。但"通过率 ~79%（lean 89% / coq 67% 中间）" 表述格式相同。

**决策性**：决策点——同 hax-coq #1。

**建议**：补"本通过率数字锚定 hax commit `30949eb` + 本 corpus 实测"。

#2 (严重度: 低) — README L22 "几乎不走 silent path"——工具自陈

**现象**：[`tools/hax-fstar/README.md`](../../../../tools/hax-fstar/README.md) L22：

> 判定与 hax-lean 同源但 hax-fstar 后端较成熟，**几乎不走 silent path**——unsupported 都让 cargo hax exit 1。

**违反**：[`tool-integration.md`](../../../design/tool-integration.md) §六禁忌——"较成熟"是工具相对评判。但措辞是中性陈述工具自身状态，不算绝对结论。

**推理链**：未来 hax 上游引入新 silent path 可能让该陈述失效——但锚定 commit 30949eb 实测，时效性约定。

**决策性**：非决策点。

**建议**：可选——把"较成熟" 改为"silent path 比 lean/coq 少"或"实测 silent path 0 现象"。

#3 (严重度: 低) — README L31-37 silent path B 论据 + 双向实测 4 类 case 充分

**现象**：[`tools/hax-fstar/README.md`](../../../../tools/hax-fstar/README.md) L31-37：

> **NEW (2026-05-11)**：entry_fn `let` / `let rec` / `and` 字面在 .fst 产物中存在性 check。封堵 `backends/fstar/fstar_backend.ml:1771 | Use _ | NotImplementedYet -> []` 的 silent-skip-item 路径...
> **形式严格性 — 0 误报**：✅ 实测 + 设计层论证 0 误报。规则 (1)(2) 同前；规则 (3) 的反误报：hax-fstar 对 Rust `fn` 项统一用 `TopLevelLet (NoLetQualifier, ...)` 渲染...双向实测验证四种 case（plain / let rec / 'and' mutual rec / missing）

**违反**：未违反——双向实测验证 4 类 case 扎实。

**决策性**：非决策点。

**建议**：无须改。

#4 (严重度: 低) — README L36 line 506-512 catch 论据扎实

**现象**：[`tools/hax-fstar/README.md`](../../../../tools/hax-fstar/README.md) L36：

> 规则 (3) 抓的 silent-skip-item 是 fstar_backend.ml:1771 的唯一 silent path（line 506-512 的 `pexpr` SpanFreeError catch 不构成 silent path —— `SpanFreeError.raise` 会先 `report` Diagnostic 触发 exit ≠ 0，详见 audit-2 §3.2）

**违反**：未违反——按 [`tool-integration.md`](../../../design/tool-integration.md) §三-源码层穷尽，论证 SpanFreeError 不是 silent path（因为先 report Diagnostic）。

**推理链**：扎实的源码层论证。

**决策性**：非决策点。

**建议**：无须改。

#5 (严重度: 低) — README L38-40 漏报盲点诚实声明

**现象**：[`tools/hax-fstar/README.md`](../../../../tools/hax-fstar/README.md) L38-40：

> **漏报盲点**：
> - hax engine 上游引入新 silent path（如 backend item kind 新增 `-> []` 分支）—— 当前 fstar_backend.ml:1771 是唯一已知 silent path
> - F* fn 渲染未来引入新 keyword（如 `unfold let` / `inline_for_extraction let` 等修饰）—— 实测当前 hax-fstar 未使用，未来上游可能引入

**违反**：未违反——按 §四-4.4 诚实声明。

**决策性**：非决策点。

**建议**：无须改。

#6 (严重度: 低) — 关联 sub-tests 与 hax-lean / hax-coq 共享

**现象**：[`tools/hax-fstar/README.md`](../../../../tools/hax-fstar/README.md) L70-72 与 hax-coq / hax-lean 一致。

**违反**：未违反。

**决策性**：非决策点。

**建议**：无须改。

## §4 决策点 vs 非决策点

- 决策点：1（L20 通过率锚定）
- 非决策点：5

## §5 审查结论

hax-fstar README 是 hax-coq 的"轻量版"——结构同形，论证质量同等。

主要补强：

1. L20 通过率比较应明确锚定 commit + corpus + 时点；
2. L22 "较成熟"措辞可中性化。

整体属于"高诚实度、高论证质量"的范例，与 hax-coq / hax-lean 同级。
