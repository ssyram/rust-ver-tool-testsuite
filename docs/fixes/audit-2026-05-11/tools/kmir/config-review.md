# kmir config Review

## §1 问题意识

kmir 是矩阵中两类解释执行工具之一（另一类是 miri），但需通过 K Framework 跑 MIR 操作语义。oracle 设计：exit 0 + stdout 含 `#EndProgram ~> .K` —— K-stuck 检测，封堵 K cell 残留但 CLI exit 0 的 silent path（实测 102 个原始 SUCCESS 中 52 个是 K-stuck 假阳性）。

恶意角度考察：
1. K-stuck grep 是否会被 K interpreter 输出格式变更破坏？
2. tool.toml inline `sh -c` 的 PATH 注入是否正确？
3. timeout 180s 与 KMIR 增量 kompile 开销（10-30s/entry）是否匹配？

## §2 审查方法

参照源：

- [`docs/design/principles.md`](../../../design/principles.md) §六-2 / §六-4；
- [`docs/design/tool-integration.md`](../../../design/tool-integration.md) §三 / §四；
- [`tools/kmir/tool.toml`](../../../../tools/kmir/tool.toml)；
- [`deep-reports/cc-reports/kmir.md`](../../../../deep-reports/cc-reports/kmir.md)；
- kmir / stable-mir-json / K Framework 三层版本锁定。

## §3 审查现象

#1 (严重度: 高) — K-stuck grep 是 kmir 反作弊关键 — `#EndProgram ~> .K` 终结模式

**现象**：[`tools/kmir/tool.toml`](../../../../tools/kmir/tool.toml) L14-17：

```
command = [
  "sh", "-c",
  "PATH=\"${TS_KMIR_JAVA_BIN_DIR}:$PATH\" kmir run --bin __ts_harness > /tmp/kmir-out-$$ 2>&1; rc=$?; cat /tmp/kmir-out-$$; if [ $rc -eq 0 ]; then if ! grep -q '#EndProgram[[:space:]]*~>[[:space:]]*\\.K' /tmp/kmir-out-$$; then echo '[kmir-oracle] FAIL: K interpreter stuck (no #EndProgram ~> .K terminator)' >&2; rc=2; fi; fi; rm -f /tmp/kmir-out-$$; exit $rc"
]
```

**违反**：未违反——按 [`tool-integration.md`](../../../design/tool-integration.md) §四-4.2 防漏报机制。注释 L10-12 说明 K 跑完的稳定 signature 是 `#EndProgram ~> .K`。

**推理链**：实测证据：旧 oracle 102 SUCCESS 中 52 个是 K-stuck 假阳性（README L26 详尽列出），新 oracle 翻转。这是矩阵中规模最大的 oracle 漏报修复案例。

**决策性**：非决策点——是 kmir 反 silent path 的核心机制。

**建议**：无须改。

#2 (严重度: 中) — `/tmp/kmir-out-$$` 临时文件 PID-based 命名——理论上有竞争风险（虽极低）

**现象**：[`tools/kmir/tool.toml`](../../../../tools/kmir/tool.toml) L16 用 `/tmp/kmir-out-$$` —— `$$` 是 shell PID，每个 wrapper 实例独立。

**违反**：未违反——shell PID 在 spawn 时唯一。runner 用 process group 隔离，每次 kmir 实例 PID 不同。

**推理链**：极低风险——理论上若两个 kmir 实例 PID 相同（不可能，PID 唯一），可能竞争。实际不存在。

**决策性**：非决策点。

**建议**：可选——用 `mktemp` 而非 `/tmp/...-$$` 更鲁棒。当前实施 work。

#3 (严重度: 中) — inline sh -c 复杂——可抽 wrapper（与 hax-* 同模式）

**现象**：[`tools/kmir/tool.toml`](../../../../tools/kmir/tool.toml) L14-17 inline sh -c 长达 1 行。

**违反**：[`tool-integration.md`](../../../design/tool-integration.md) §五——README 与 tool.toml 维护性。同 hax-* config #1。

**决策性**：决策点。

**建议**：可选——抽 `kmir-wrapper.sh` 与其他工具体例一致。

#4 (严重度: 低) — `PATH=${TS_KMIR_JAVA_BIN_DIR}:$PATH` 注入合规

**现象**：[`tools/kmir/tool.toml`](../../../../tools/kmir/tool.toml) L16 PATH 注入 + 注释 L4-6 说明"K Framework 在运行时需要 Java"。

**违反**：未违反——按 [`tool-integration.md`](../../../design/tool-integration.md) §一 安装方式说明。

**推理链**：合规。但 `cargo` 由 runner 继承 PATH 找到（注释 L5）——隐式依赖系统 PATH。

**决策性**：非决策点。

**建议**：无须改。

#5 (严重度: 中) — timeout_secs = 180 与 kompile 10-30s 开销不匹配深递归 entry

**现象**：[`tools/kmir/tool.toml`](../../../../tools/kmir/tool.toml) L18：`timeout_secs = 180`。

**违反**：未违反——按 [`principles.md`](../../../design/principles.md) §七 性能不算问题。但 README L25 说每次 kompile 10-30s——若 entry 含深递归 / 大循环，K interpreter 跑慢，可能 180s 超时。

**推理链**：实测 102 SUCCESS 显示 180s 多数 entry 够用。但 industrial 类目（如 rsa-pkcs8 / sha256-digest）可能超时——是否影响 oracle 上限保证（不冤枉能力）？

按 [`principles.md`](../../../design/principles.md) §三-3-2.b 上限保证，timeout 把能跑完的 entry 误判 FAILED 违反原则。但 §七 说"性能不算问题除非升级为功能问题"——这是边界情况。

**决策性**：决策点——industrial entry 可能受影响。

**建议**：可选——上调到 300s（与 miri 一致）。或在 README 显式列出受影响 entry。

#6 (严重度: 低) — version_command 与 command 同模式

**现象**：[`tools/kmir/tool.toml`](../../../../tools/kmir/tool.toml) L19-22：

```toml
version_command = [
  "sh", "-c",
  "PATH=\"${TS_KMIR_JAVA_BIN_DIR}:$PATH\" kmir --version"
]
```

**违反**：未违反——PATH 注入一致。

**决策性**：非决策点。

**建议**：无须改。

#7 (严重度: 低) — harness 朴素 main + entry 调用 + 注释说明 kmir 入口

**现象**：[`tools/kmir/harness.rs.tera`](../../../../tools/kmir/harness.rs.tera)：

```rust
fn main() {
    {{ target_crate_name }}::{{ entry_fn }}();
}
```

注释 L7-10：

> Plain `fn main()` is required: kmir run looks for a binary target and enters at `main` by default (--start-symbol overrides this).

**违反**：未违反。

**决策性**：非决策点。

**建议**：无须改。

## §4 决策点 vs 非决策点

- 决策点：2（抽 wrapper / timeout 300s）
- 非决策点：5

## §5 审查结论

kmir 配置最关键的是 K-stuck grep 反 silent path——实测翻转 52/102 假阳性，是矩阵中最有 impact 的 oracle 修复案例之一。

最值得补强：

1. **抽 wrapper**：inline sh -c 与 hax-* 一并重构；
2. **timeout 300s**：与 miri 一致，避免 industrial entry 超时。

整体属于"经实测验证收紧的优秀 oracle 设计"。
