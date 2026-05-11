# hax-fstar config Review

## §1 问题意识

hax-fstar 与 hax-lean / hax-coq 同体例——内联 sh -c oracle，差产物路径与 F\* fn 渲染分支。第二轮 audit 增加：entry_fn 在 .fst 中以 `let` / `let rec` / `and` 关键字存在性 grep——封堵 `fstar_backend.ml:1771 | Use _ | NotImplementedYet -> []` silent skip-item path。

恶意角度考察：
1. F\* fn 渲染分支是否真只用 `let` / `let rec` / `and`（fstar_backend.ml:1112, 1923-1924）？是否漏 `unfold let` 等修饰？
2. `$TS_ENTRY_FN` trap 同 hax-coq；
3. timeout_secs = 300 < hax-lean/hax-coq 的 600，依据 README "F\* 提取比 Lean/Coq 快"。

## §2 审查方法

参照源：

- [`docs/fixes/audit-2026-05-11/tools/hax-coq/config-review.md`](../hax-coq/config-review.md) shared issues；
- [`tools/hax-fstar/tool.toml`](../../../../tools/hax-fstar/tool.toml)；
- [`docs/fixes/oracle-leak-audit-2-2026-05-11.md`](../../oracle-leak-audit-2-2026-05-11.md) §3.2；
- [`docs/fixes/oracle-leak-rules-implementation-2-2026-05-11.md`](../../oracle-leak-rules-implementation-2-2026-05-11.md) §2.2；
- [`deep-reports/cc-reports/hax-fstar.md`](../../../../deep-reports/cc-reports/hax-fstar.md)。

## §3 审查现象

#1 (严重度: 中) — grep keyword `(let[[:space:]]+(rec[[:space:]]+)?|and[[:space:]]+)` 覆盖 F\* fn 渲染——但未涵盖 `unfold let` / `inline_for_extraction let` 等修饰

**现象**：[`tools/hax-fstar/tool.toml`](../../../../tools/hax-fstar/tool.toml) L22：

```
... ! grep -rqE \"^(let[[:space:]]+(rec[[:space:]]+)?|and[[:space:]]+)$TS_ENTRY_FN[[:space:]]\" ...
```

注释 L7-11：

> hax-fstar 对 Fn 项统一用 NoLetQualifier，即 `let`；mutual recursion 第一个改写为 `let rec`，后续改写为 `and` —— 详见 fstar_backend.ml:1112 / 1923-1924。

**违反**：未违反——按 [`tool-integration.md`](../../../design/tool-integration.md) §三-源码层穷尽，覆盖 fstar_backend.ml:1112 / 1923 / 1924 三个分支。

**推理链**：README L39-40 已声明漏报盲点："F\* fn 渲染未来引入新 keyword（如 `unfold let` / `inline_for_extraction let` 等修饰）—— 实测当前 hax-fstar 未使用，未来上游可能引入"。诚实声明。

**决策性**：非决策点。

**建议**：无须改——盲点已声明。

#2 (严重度: 高) — `$TS_ENTRY_FN` 用法（无大括号）与 runner expand_env 假设——与 hax-coq 同 trap

**现象**：[`tools/hax-fstar/tool.toml`](../../../../tools/hax-fstar/tool.toml) L13-16：

> runner expand_env 陷阱：tool.toml 内必须用 `$TS_ENTRY_FN`（无 `{}`）而非 `${TS_ENTRY_FN}`。runner 的 expand_env 把 `${VAR}` 形式在 TOML parse 时展开为运行时 env var；但 `TS_ENTRY_FN` 在 runner 启动时不存在（仅在 spawn child 前才注入），所以 `${TS_ENTRY_FN}` 会被展开为空串。

**违反**：与 hax-coq 同。注释扎实，trap 说明清晰。

**推理链**：同 hax-coq config #1。

**决策性**：决策点——加更醒目警告。

**建议**：与 hax-coq 同步。

#3 (严重度: 低) — timeout_secs = 300 与 hax-lean / hax-coq 600 不同

**现象**：[`tools/hax-fstar/tool.toml`](../../../../tools/hax-fstar/tool.toml) L24：`timeout_secs = 300`。

**违反**：未违反——按 [`tools/hax-fstar/README.md`](../../../../tools/hax-fstar/README.md) L57："F\* 提取比 Lean/Coq 快，timeout 较短"。

**决策性**：非决策点。

**建议**：无须改。

#4 (严重度: 低) — 内联 sh -c 与 hax-lean / hax-coq 同形

**现象**：[`tools/hax-fstar/tool.toml`](../../../../tools/hax-fstar/tool.toml) L20-23 内联 sh -c。

**违反**：同 hax-lean / hax-coq——可抽 wrapper。

**决策性**：决策点。

**建议**：与 hax-lean / hax-coq 同步重构。

#5 (严重度: 低) — version_command 与 hax-lean / hax-coq 同模式

**现象**：[`tools/hax-fstar/tool.toml`](../../../../tools/hax-fstar/tool.toml) L25-29 与 hax-coq / hax-lean 一致。

**违反**：未违反。

**决策性**：非决策点。

**建议**：无须改。

## §4 决策点 vs 非决策点

- 决策点：2（DO NOT TOUCH 警告 + 内联 sh -c 抽 wrapper）
- 非决策点：3

## §5 审查结论

hax-fstar 配置是 hax-coq 的"轻量版"——少 1 个 silent literal grep（hax-fstar 只 grep `Rust_primitives.Hax.failure` / `failure ((`），多 entry_fn 存在性 grep。

最值得补强：

1. 与 hax-coq / hax-lean 同步抽 wrapper；
2. 更醒目 DO NOT TOUCH 警告（$TS_ENTRY_FN trap）。

整体属于"2 轮 audit 收紧后的稳态配置"。无 backend-specific critical 问题。
