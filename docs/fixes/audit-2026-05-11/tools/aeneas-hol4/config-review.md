# aeneas-hol4 config Review

## §1 问题意识

aeneas-hol4 是 4 个 aeneas backend 之一，与 aeneas-lean / aeneas-coq / aeneas-fstar 结构同形。HOL4 specific 边界：README L26-30 明示 backend 有 `trait_decl_kind_to_qualif None` panic 硬天花板——所有含 trait declaration 的 entry 在该 backend exit 2。

## §2 审查方法

参照源：

- [`docs/fixes/audit-2026-05-11/tools/aeneas-lean/config-review.md`](../aeneas-lean/config-review.md) shared issues；
- [`tools/aeneas-hol4/aeneas-hol4-wrapper.sh`](../../../../tools/aeneas-hol4/aeneas-hol4-wrapper.sh)；
- [`tools/aeneas-hol4/tool.toml`](../../../../tools/aeneas-hol4/tool.toml)；
- [`deep-reports/cc-reports/aeneas-hol4.md`](../../../../deep-reports/cc-reports/aeneas-hol4.md)。

## §3 审查现象

#1 (严重度: 中) — wrapper 与其他 3 个 aeneas wrapper 完全同形

**现象**：[`tools/aeneas-hol4/aeneas-hol4-wrapper.sh`](../../../../tools/aeneas-hol4/aeneas-hol4-wrapper.sh) L13、L46-60 同其他 aeneas backend wrapper：

```bash
set -euo pipefail
...
"$AENEAS_BIN" -backend hol4 -dest "$HOL4_OUT" "$LLBC_FILE"
AENEAS_EXIT=$?
echo "[aeneas-hol4-wrapper] aeneas exit: $AENEAS_EXIT"
```

**违反**：与 aeneas-lean 同——`set -e` 与 $? 抓取冲突 + 硬编码 opam switch=default。

**推理链**：4 个 wrapper 同样问题。

**决策性**：决策点（已在 aeneas-lean review 标注）。

**建议**：与 aeneas-lean 同步修。

#2 (严重度: 中) — wrapper L57 `find "$HOL4_OUT" -name "*.sml" -o -name "*.thy"` 用 `-o` 短路 — 与 shell 优先级有 trap

**现象**：[`tools/aeneas-hol4/aeneas-hol4-wrapper.sh`](../../../../tools/aeneas-hol4/aeneas-hol4-wrapper.sh) L57：

```bash
find "$HOL4_OUT" -name "*.sml" -o -name "*.thy" | sort
```

**违反**：`find` 的 `-o` 优先级与 `-name` 默认 `-a` 在某些场景下需括号显式分组。这里 `-name "*.sml" -o -name "*.thy"` 解析为 `(-name "*.sml") -o (-name "*.thy")`——find 等价于 OR。当前正确。

**推理链**：仅做诊断输出，不影响 oracle。

**决策性**：非决策点。

**建议**：无须改。

#3 (严重度: 中) — HOL4 backend exit 2 panic 路径是否被 oracle 正确处理

**现象**：[`tools/aeneas-hol4/aeneas-hol4-wrapper.sh`](../../../../tools/aeneas-hol4/aeneas-hol4-wrapper.sh) L60 `exit $AENEAS_EXIT` 直接传出。README L26-30 提到 HOL4 backend 的 `trait_decl_kind_to_qualif None` 触发 `Invalid_argument "option is None"` panic，exit 2。

**违反**：未违反——runner 二值化（非 0 = FAILED），exit 2 与 exit 1 都 FAILED，oracle 正确。

**推理链**：HOL4 backend 的 panic 是 aeneas 上游硬 bug——但按宪法精神，oracle 不评判工具 bug，只观察 exit code。

**决策性**：非决策点。

**建议**：无须改。

#4 (严重度: 低) — tool.toml / entry_mode / harness 与其他 aeneas backend 同形

**现象**：[`tools/aeneas-hol4/tool.toml`](../../../../tools/aeneas-hol4/tool.toml) 与 aeneas-lean / aeneas-coq / aeneas-fstar tool.toml 字面相同，仅差 wrapper 路径。

**违反**：未违反。

**决策性**：非决策点。

**建议**：无须改。

## §4 决策点 vs 非决策点

- 决策点：2（与 aeneas-lean 共享）
- 非决策点：2

## §5 审查结论

aeneas-hol4 配置与其他 3 个 aeneas backend 同形。最显著特殊点：README L26-30 诚实声明 backend 的硬天花板（`trait_decl_kind_to_qualif None` panic）——这是 aeneas-hol4 实测通过率显著低于其他 3 个 backend 的根因。诚实声明合规。

无 backend-specific 配置问题；所有问题与 aeneas-lean 共享。
