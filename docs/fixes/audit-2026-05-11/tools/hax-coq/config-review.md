# hax-coq config Review

## §1 问题意识

hax-coq 是 hax 3 backend 中 oracle 设计最复杂的——3 轨 partial 暴露机制：(1) cargo hax exit 1；(2) 产物 grep `failure ((` / `please implement the method` 抓 silent literal；(3) 2026-05-11 第二轮 audit 新加：entry_fn 在 .v 产物中以 `Definition` / `Fixpoint` / `Lemma` / `Equations` / `Theorem` / `Program Definition` 关键字存在性 grep。

关键 trap：`$TS_ENTRY_FN`（无大括号）vs `${TS_ENTRY_FN}` —— 后者会被 runner expand_env 提前展开为空（TS_ENTRY_FN 是 spawn child 前才注入的 runtime env）。

恶意角度考察：
1. 内联 sh -c 是否正确处理 `$TS_ENTRY_FN`？
2. grep keyword list 是否覆盖 Coq backend 所有 fn 渲染分支（coq_backend.ml:454, 518, 540）？
3. README 与 wrapper 一致？

## §2 审查方法

参照源：

- [`docs/design/principles.md`](../../../design/principles.md) §六-2；
- [`docs/design/tool-integration.md`](../../../design/tool-integration.md) §三 / §四；
- [`tools/hax-coq/tool.toml`](../../../../tools/hax-coq/tool.toml)；
- [`docs/fixes/oracle-leak-audit-2-2026-05-11.md`](../../oracle-leak-audit-2-2026-05-11.md) §3.3；
- [`docs/fixes/oracle-leak-rules-implementation-2-2026-05-11.md`](../../oracle-leak-rules-implementation-2-2026-05-11.md) §2.3；
- [`runner/src/exec.rs:178`](../../../../runner/src/exec.rs)（TS_ENTRY_FN 注入路径）。

## §3 审查现象

#1 (严重度: 高) — `$TS_ENTRY_FN` 用法（无大括号）必须与 runner expand_env 假设精确对齐

**现象**：[`tools/hax-coq/tool.toml`](../../../../tools/hax-coq/tool.toml) L21-22 注释 + L27 实施：

```
# runner expand_env 陷阱：与 hax-fstar / rocq-of-rust 同 —— `$TS_ENTRY_FN` 而非
# `${TS_ENTRY_FN}`，避开 runner expand_env 提前展开。
...
... elif [ -n \"$TS_ENTRY_FN\" ] && ! grep -rqE \"^[[:space:]]*(Definition|Fixpoint|Lemma|Equations|Theorem|Program[[:space:]]+Definition)[[:space:]]+$TS_ENTRY_FN[[:space:]]\" proofs/coq/extraction/ ...
```

**违反**：未违反——这是 P12 audit 暴露的 trap，hax-coq tool.toml 注释明确说明。`runner expand_env` 只匹配 `${VAR}`，不展开 `$VAR`——所以 `$TS_ENTRY_FN` 留给 sh runtime 展开，TS_ENTRY_FN 在 spawn child 前才注入（[`runner/src/exec.rs:178`](../../../../runner/src/exec.rs)）。

**推理链**：理论上合规。但若有维护者随手改 `$TS_ENTRY_FN` 为 `${TS_ENTRY_FN}` 想 "更规范"，会让 grep pattern 变 `^...+\s` 永远 0 命中 → silent SUCCESS for all entries → silent oracle leak。这是 P12-P16 audit 反复警告的陷阱。

**决策性**：决策点——是否在 tool.toml 加更显眼的"DO NOT TOUCH" 警告。

**建议**：当前注释 L21-22 已明示陷阱。可考虑加更醒目的"### DANGER: DO NOT CHANGE $TS_ENTRY_FN to ${TS_ENTRY_FN}"。

#2 (严重度: 中) — grep keyword list 覆盖 Coq backend fn 渲染分支——但容忍未来扩展

**现象**：[`tools/hax-coq/tool.toml`](../../../../tools/hax-coq/tool.toml) L27：

```
(Definition|Fixpoint|Lemma|Equations|Theorem|Program[[:space:]]+Definition)
```

注释 L13-18：

> 合法翻译的 fn 必有 `Definition <fn>` / `Fixpoint <fn>` / `Lemma <fn>` 之一（coq_backend.ml:454,518,540 三个 CoqNotation 分支）。
> Coq backend 不使用 `Equations`（仅 hax-coq fork 可能扩展，本基线 commit 30949eb 无），grep pattern 容忍 `Equations` 与 `Theorem`/`Program Definition` 以防上游引入。

**违反**：未违反——按 [`tool-integration.md`](../../../design/tool-integration.md) §三-源码层穷尽，3 个 CoqNotation 分支已覆盖。Equations / Theorem / Program Definition 是 future-proof 防御。

**推理链**：合规。3 个核心分支（coq_backend.ml:454 lemma / :518 fixpoint / :540 definition）+ 3 个防御分支。覆盖完整。

**决策性**：非决策点。

**建议**：无须改。

#3 (严重度: 中) — README L31 "NEW (2026-05-11)" 引用 audit 文档充分

**现象**：[`tools/hax-coq/README.md`](../../../../tools/hax-coq/README.md) L31：

> 3. **NEW (2026-05-11)** silent path B：`engine/backends/coq/coq/coq_backend.ml:588 method item'_NotImplementedYet = string "(* NotImplementedYet *)"` ...

**违反**：未违反——按 [`tool-integration.md`](../../../design/tool-integration.md) §五 必含章节、§六诚实声明。docs/fixes/oracle-leak-audit-2-2026-05-11.md §3.3 / oracle-leak-rules-implementation-2-2026-05-11.md §2.3 引用充分。

**推理链**：第二轮 audit 暴露的 silent skip-item path 已通过 entry_fn 存在性 grep 封堵。

**决策性**：非决策点。

**建议**：无须改。

#4 (严重度: 中) — README L33-35 形式严格性 ✅ 实测 + 设计层论证 vs ⚠️ 实测验证 —— 措辞精确度

**现象**：[`tools/hax-coq/README.md`](../../../../tools/hax-coq/README.md) L33-35：

> **形式严格性 — 0 误报（不冤枉能力）**：✅ 实测 + 设计层论证 0 误报。规则 (1)(2) 同前；规则 (3) 的反误报：hax-coq 对 Rust `fn` 项必经过 coq_backend.ml:454-540 三个 CoqNotation 分支之一...

**违反**：[`tool-integration.md`](../../../design/tool-integration.md) §四-4.3 措辞——hax-coq 用 ✅ 而非 ⚠️。论证形式是"实测 + 设计层论证"——比 hax-lean 的 ⚠️ "实测验证" 更强（多了"设计层论证"）。

**推理链**：考量 ✅ 是否合理：

- 规则 (1) cargo hax exit 1 是 hax 自身设计的 Diagnostic 信号——形式可证 hax engine 任何 emit Diagnostic 都 exit ≠ 0；
- 规则 (2) failure / please implement 是 coq_backend.ml:137 唯一的 `default_string_for` literal——源码层穷尽；
- 规则 (3) entry_fn 必经过 coq_backend.ml:454-540 三个 CoqNotation 分支之一——源码层穷尽。

所以"设计层论证"是真的——3 条规则都有 hax 上游源码层证据。但**仍依赖 grep**（无法形式证明所有未来变体），从这个角度看 ⚠️ 更准确。但 README 也写双向实测验证 5 类 case——证据扎实。

**决策性**：决策点——✅ 是否过强？

**建议**：可保留 ✅，因为论证含设计层论证（coq_backend.ml 源码穷尽）+ 双向实测——比纯 grep heuristics 强。但应明确说"这不是形式证明，是源码层穷尽 + 实测的强经验论证"。

#5 (严重度: 低) — 内联 sh -c 与 hax-lean 同形——可统一抽外部 wrapper

**现象**：[`tools/hax-coq/tool.toml`](../../../../tools/hax-coq/tool.toml) L25-28 内联 sh -c——与 hax-lean / hax-fstar 同模式。

**违反**：同 hax-lean review #1——可抽出 wrapper。

**决策性**：决策点。

**建议**：与 hax-lean / hax-fstar 同步重构。

#6 (严重度: 低) — harness 用 `let _ = ...` 而 hax-lean 用 `...();` —— 体例不一致但无害

**现象**：

- [`tools/hax-coq/harness.rs.tera`](../../../../tools/hax-coq/harness.rs.tera) L6：`let _ = {{ target_crate_name }}::{{ entry_fn }}();`
- [`tools/hax-lean/harness.rs.tera`](../../../../tools/hax-lean/harness.rs.tera) L5：`{{ target_crate_name }}::{{ entry_fn }}();`

**违反**：未违反——但体例不一致。`let _ = ` 显式 drop 返回值，防 `#[must_use]` warning；`...();` 默认 drop。两种都合规。

**推理链**：hax `-C --lib` 跳过 bin target，harness 不参与 hax 翻译——`let _ =` vs `...();` 仅影响 cargo build 的 warning。

**决策性**：非决策点。

**建议**：可选——3 个 hax backend harness 体例对齐（都用 `let _ =` 或都用 `...();`）。

#7 (严重度: 低) — version_command 与 hax-lean / hax-fstar 同模式

**现象**：[`tools/hax-coq/tool.toml`](../../../../tools/hax-coq/tool.toml) L30-35 与 hax-lean / hax-fstar 体例一致。

**违反**：未违反。

**决策性**：非决策点。

**建议**：无须改。

## §4 决策点 vs 非决策点

- 决策点：3（DO NOT TOUCH 警告 / ✅ vs ⚠️ 措辞 / 内联 sh -c 抽 wrapper）
- 非决策点：4

## §5 审查结论

hax-coq 配置是矩阵中 oracle 设计最复杂的——3 轨 partial 暴露 + 第二轮 audit 增加的 entry_fn 存在性 grep。`$TS_ENTRY_FN`（无大括号）trap 明确标注。

最值得补强：

1. **更醒目 DO NOT TOUCH 警告**：防维护者改 `$TS_ENTRY_FN` 为 `${TS_ENTRY_FN}` 导致 silent oracle leak；
2. **内联 sh -c 抽 wrapper**：与 hax-lean / hax-fstar 一并重构；
3. **✅ vs ⚠️ 措辞**：README L33-35 用 ✅ "实测 + 设计层论证"——比 hax-lean ⚠️ 更强，可保留但需明示这是源码层穷尽 + 实测，非形式证明。

整体属于"两轮 audit 收紧后的稳态配置"，论证质量高。
