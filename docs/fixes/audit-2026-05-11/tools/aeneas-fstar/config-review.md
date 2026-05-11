# aeneas-fstar config Review

## §1 问题意识

aeneas-fstar 是 4 个 aeneas backend 之一，与 aeneas-lean / aeneas-coq 结构同形——差 backend flag (`-backend fstar`) + 产物后缀 (`.fst`)。

## §2 审查方法

参照源：

- [`docs/fixes/audit-2026-05-11/tools/aeneas-lean/config-review.md`](../aeneas-lean/config-review.md) shared issues；
- [`tools/aeneas-fstar/aeneas-fstar-wrapper.sh`](../../../../tools/aeneas-fstar/aeneas-fstar-wrapper.sh)；
- [`tools/aeneas-fstar/tool.toml`](../../../../tools/aeneas-fstar/tool.toml)；
- [`deep-reports/cc-reports/aeneas-fstar.md`](../../../../deep-reports/cc-reports/aeneas-fstar.md)。

## §3 审查现象

#1 (严重度: 中) — wrapper 与 aeneas-lean / aeneas-coq wrapper 完全同形

**现象**：[`tools/aeneas-fstar/aeneas-fstar-wrapper.sh`](../../../../tools/aeneas-fstar/aeneas-fstar-wrapper.sh) L13、L46-60 与 aeneas-lean / aeneas-coq wrapper 同：

```bash
set -euo pipefail
...
"$AENEAS_BIN" -backend fstar -dest "$FSTAR_OUT" "$LLBC_FILE"
AENEAS_EXIT=$?
echo "[aeneas-fstar-wrapper] aeneas exit: $AENEAS_EXIT"
```

**违反**：与 aeneas-lean 同——`set -e` 与 $? 抓取冲突。

**推理链**：4 个 wrapper 完全同形——共有缺陷。

**决策性**：决策点（已在 aeneas-lean review 标注）。

**建议**：与 aeneas-lean 同步修。

#2 (严重度: 中) — opam env 假设——与 aeneas-lean 同

**现象**：L22-24 与 aeneas-lean 同。

**违反**：同 aeneas-lean。

**决策性**：决策点。

**建议**：与 aeneas-lean 同步修。

#3 (严重度: 低) — tool.toml entry_mode / harness / version_command 与 aeneas-lean 完全同形

**现象**：[`tools/aeneas-fstar/tool.toml`](../../../../tools/aeneas-fstar/tool.toml) 与 [`tools/aeneas-lean/tool.toml`](../../../../tools/aeneas-lean/tool.toml) 仅差 backend flag。

**违反**：未违反——shared infra 应同形。

**决策性**：非决策点。

**建议**：无须改。

#4 (严重度: 低) — README L44 "F\* backend 自身不需要额外组件——`aeneas -backend fstar` 只生成 `.fst` 文件，不调用 `fstar.exe`"

**现象**：[`tools/aeneas-fstar/README.md`](../../../../tools/aeneas-fstar/README.md) L44：

> F\* backend 自身不需要额外组件——`aeneas -backend fstar` 只生成 `.fst` 文件，不调用 `fstar.exe`。

**违反**：未违反——明确 oracle 不查 fstar.exe，与 §六-1 切线一致。

**决策性**：非决策点。

**建议**：无须改。

## §4 决策点 vs 非决策点

- 决策点：2（与 aeneas-lean 共享）
- 非决策点：2

## §5 审查结论

aeneas-fstar 配置是 aeneas-lean 的标准复用。无 backend-specific 额外问题。所有问题与 aeneas-lean 共享。
