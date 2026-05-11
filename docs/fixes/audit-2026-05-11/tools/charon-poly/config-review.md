# charon-poly config Review

## §1 问题意识

charon-poly 与 charon-mono 同源（同 binary），差仅在 `--monomorphize` flag——poly 不展开泛型。审查重点同 charon-mono。

## §2 审查方法

参照源：

- [`docs/fixes/audit-2026-05-11/tools/charon-mono/config-review.md`](../charon-mono/config-review.md) shared issues；
- [`tools/charon-poly/tool.toml`](../../../../tools/charon-poly/tool.toml)；
- [`deep-reports/cc-reports/charon-poly.md`](../../../../deep-reports/cc-reports/charon-poly.md)。

## §3 审查现象

#1 (严重度: 中) — tool.toml 同 charon-mono 硬编码 `--target aarch64-apple-darwin`

**现象**：[`tools/charon-poly/tool.toml`](../../../../tools/charon-poly/tool.toml) L1：

```
command = ["${TS_CHARON_BIN}", "cargo", "--abort-on-error", "--print-llbc", "--", "--lib", "--target", "aarch64-apple-darwin"]
```

**违反**：同 charon-mono——`--target` 硬编码 macOS arm64。Linux 不可用。

**推理链**：与 charon-mono review #1 同。

**决策性**：决策点。

**建议**：同 charon-mono 同步修——参数化 host triple。

#2 (严重度: 低) — tool.toml 与 charon-mono 差仅在 `--monomorphize` flag

**现象**：对比：

- charon-mono：`["${TS_CHARON_BIN}", "cargo", "--monomorphize", "--abort-on-error", "--print-llbc", "--", "--lib", "--target", "aarch64-apple-darwin"]`
- charon-poly：`["${TS_CHARON_BIN}", "cargo", "--abort-on-error", "--print-llbc", "--", "--lib", "--target", "aarch64-apple-darwin"]`

**违反**：未违反——两个 backend 共享 binary 与配置体例，差仅在 monomorphize flag。

**决策性**：非决策点。

**建议**：无须改。

#3 (严重度: 低) — harness.rs.tera 与 charon-mono 完全相同

**现象**：[`tools/charon-poly/harness.rs.tera`](../../../../tools/charon-poly/harness.rs.tera) 与 [`tools/charon-mono/harness.rs.tera`](../../../../tools/charon-mono/harness.rs.tera) 字面相同。

**违反**：未违反——shared harness。

**决策性**：非决策点。

**建议**：无须改。

#4 (严重度: 低) — version_command 与 charon-mono 相同

**现象**：L3 与 charon-mono L3 相同。

**违反**：未违反。

**决策性**：非决策点。

**建议**：无须改。

## §4 决策点 vs 非决策点

- 决策点：1（与 charon-mono 共享）
- 非决策点：3

## §5 审查结论

charon-poly 配置与 charon-mono 同形，仅差 `--monomorphize`。无 backend-specific 问题。所有问题与 charon-mono 共享。
