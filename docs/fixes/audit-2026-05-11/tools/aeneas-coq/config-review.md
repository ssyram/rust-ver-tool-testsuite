# aeneas-coq config Review

## §1 问题意识

aeneas-coq 是 4 个 aeneas backend 之一，与 aeneas-lean 同 charon → llbc → aeneas 两阶段 pipeline，差 backend flag (`-backend coq`)。共享 craise + Main.ml:773 形式证明级 oracle。

恶意角度考察：
1. 配置是否与 aeneas-lean 体例一致（shared infra 不能漂移）？
2. wrapper 是否同样有 `set -e` + aeneas $? 抓取问题？
3. tool.toml env 注入正确？

## §2 审查方法

参照源：

- [`docs/fixes/audit-2026-05-11/tools/aeneas-lean/config-review.md`](../aeneas-lean/config-review.md) shared issues；
- [`tools/aeneas-coq/aeneas-coq-wrapper.sh`](../../../../tools/aeneas-coq/aeneas-coq-wrapper.sh)；
- [`tools/aeneas-coq/tool.toml`](../../../../tools/aeneas-coq/tool.toml)；
- [`deep-reports/cc-reports/aeneas-coq.md`](../../../../deep-reports/cc-reports/aeneas-coq.md)。

## §3 审查现象

#1 (严重度: 中) — wrapper 与 aeneas-lean wrapper 几乎完全相同，包括同样的 `set -e` + aeneas $? 抓取冲突

**现象**：[`tools/aeneas-coq/aeneas-coq-wrapper.sh`](../../../../tools/aeneas-coq/aeneas-coq-wrapper.sh) L13、L46-60 与 [`tools/aeneas-lean/aeneas-lean-wrapper.sh`](../../../../tools/aeneas-lean/aeneas-lean-wrapper.sh) 同形：

```bash
set -euo pipefail
...
"$AENEAS_BIN" -backend coq -dest "$COQ_OUT" "$LLBC_FILE"
AENEAS_EXIT=$?
echo "[aeneas-coq-wrapper] aeneas exit: $AENEAS_EXIT"
```

**违反**：与 aeneas-lean 同——`set -e` 下 aeneas 非 0 退出会让脚本直接 exit，`AENEAS_EXIT=$?` 那行不执行。诊断信号缺失。

**推理链**：参见 aeneas-lean/config-review.md #3。

**决策性**：决策点（已在 aeneas-lean review 标注）。

**建议**：与 aeneas-lean 同步修。

#2 (严重度: 中) — opam env --set-switch=default 假设——4 个 aeneas backend 共享

**现象**：[`tools/aeneas-coq/aeneas-coq-wrapper.sh`](../../../../tools/aeneas-coq/aeneas-coq-wrapper.sh) L22-24 与 aeneas-lean / aeneas-fstar / aeneas-hol4 wrapper 同形。

**违反**：与 aeneas-lean 同。

**推理链**：参见 aeneas-lean/config-review.md #5。

**决策性**：决策点。

**建议**：与 aeneas-lean 同步修；4 个 wrapper 应共享 opam 激活逻辑（DRY）。

#3 (严重度: 低) — README L46 安装段提"Coq backend 自身不需要额外组件——`aeneas -backend coq` 只生成 `.v` 文件，不调用 `coqc`"

**现象**：[`tools/aeneas-coq/README.md`](../../../../tools/aeneas-coq/README.md) L46：

> Coq backend 自身不需要额外组件——`aeneas -backend coq` 只生成 `.v` 文件，不调用 `coqc`。

**违反**：未违反——明确划定 oracle 不查 coqc，与 rocq-of-rust-typecheck 形成对比。

**推理链**：aeneas-coq oracle 停在 aeneas 产物落盘，不进 coqc——与 [`principles.md`](../../../design/principles.md) §六-1 切线一致。

**决策性**：非决策点。

**建议**：无须改。

#4 (严重度: 低) — tool.toml entry_mode = "lib" + harness 与 aeneas-lean 相同

**现象**：[`tools/aeneas-coq/tool.toml`](../../../../tools/aeneas-coq/tool.toml) L21、[`tools/aeneas-coq/harness.rs.tera`](../../../../tools/aeneas-coq/harness.rs.tera) 与 aeneas-lean 同。

**违反**：未违反。

**决策性**：非决策点。

**建议**：无须改。

#5 (严重度: 低) — version_command 形式与 aeneas-lean 一致

**现象**：[`tools/aeneas-coq/tool.toml`](../../../../tools/aeneas-coq/tool.toml) L16 与 aeneas-lean L16 字面相同。

**违反**：未违反。

**决策性**：非决策点。

**建议**：无须改。

## §4 决策点 vs 非决策点

- 决策点：2（与 aeneas-lean 共享）
- 非决策点：3

## §5 审查结论

aeneas-coq 配置是 aeneas-lean 的标准复用——共享 wrapper / harness / tool.toml 体例。所有问题与 aeneas-lean review 共享，无 backend-specific 额外问题。

**关键建议**：4 个 aeneas backend wrapper 应抽出共享脚本（如 `aeneas-common-wrapper.sh`），避免维护时 4 处需同步——但这是非决策点的工程优化。
