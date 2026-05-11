# miri config Review

## §1 问题意识

miri 是矩阵中两类解释执行工具之一（另一类是 kmir）。它的 oracle 设计是单一 exit code——`exit 0` ⇔ 解释执行完整跑完且无 UB / unsupported / panic。

恶意角度考察：
1. `cargo +nightly miri run --bin __ts_harness` 是否能被 silent path 绕过？是否存在 miri exit 0 但实际没解释执行的情况？
2. timeout 300s 是否合理（miri 比原生慢 10-100x）？
3. harness 是最朴素的 main → entry 调用——是否会触发某种"miri silent skip" 路径？
4. miri 是否会在 isolation default 配置下做"读 RTC / 网络"等被静默 reject 的事情？

## §2 审查方法

参照源：

- [`docs/design/principles.md`](../../../design/principles.md) §六-1（前端 / 后端切分——miri 属"解释执行类，秒级开销可接受"，无求解器后端切分需求）；
- [`docs/design/principles.md`](../../../design/principles.md) §六-2 不允许 partial；
- [`docs/design/tool-integration.md`](../../../design/tool-integration.md) §三 / §四 形式严格性论证；
- [`runner/src/discover.rs`](../../../../runner/src/discover.rs) `ToolToml` struct；
- [`deep-reports/cc-reports/miri.md`](../../../../deep-reports/cc-reports/miri.md) cc-report；
- miri 上游：<https://github.com/rust-lang/miri>。

## §3 审查现象

#1 (严重度: 中) — tool.toml 未启用 `-Zmiri-disable-isolation` 但 README 与 oracle 都未提及 isolation 隐式 reject

**现象**：[`tools/miri/tool.toml`](../../../../tools/miri/tool.toml) L1：

```
command = ["cargo", "+nightly", "miri", "run", "--bin", "__ts_harness"]
```

无 `MIRIFLAGS=-Zmiri-disable-isolation` 之类。

**违反**：[`principles.md`](../../../design/principles.md) §六-2 "不允许 partial"——miri 默认 isolation 开启时，任何 entry 试图触碰 OS 接口（getrandom / 网络 / 文件 / 时间）会被 miri 报"unsupported isolated operation" 然后 exit 非 0。这是 miri 自陈的"我做不了" → FAILED。**这是正确的，符合 §六-2**。

**推理链**：isolation 开启时 reject 走 exit ≠ 0，oracle 正确捕获。所以无 `-Zmiri-disable-isolation` 是合规的——这是 miri 的 default、最严格设置。**反向**确认：若开启 `-Zmiri-disable-isolation`，会让某些原本 FAILED 的 entry SUCCESS——那是更宽松，会引入"miri 用 host RTC 代替仿真"等不诚实场景，反而违反 §六-4 反作弊推论。

**决策性**：决策点已经做了——但 README 未显式阐述这条选择。

**建议**：在 README "本框架配置" 段补充："不启用 `-Zmiri-disable-isolation`——保持 miri default isolation，让 OS 调用走 exit ≠ 0 → FAILED 路径，与 §六-2 不允许 partial 对齐。" 当前 tool.toml 配置合规，但缺关键论证。

#2 (严重度: 中) — 与 kmir 同样是解释器，但 oracle 没有 `#EndProgram` 之类的"K-stuck"防御

**现象**：[`tools/miri/tool.toml`](../../../../tools/miri/tool.toml) 全文 3 行，无 wrapper、无 silent-path 后处理：

```
command      = ["cargo", "+nightly", "miri", "run", "--bin", "__ts_harness"]
timeout_secs = 300
version_command = ["cargo", "+nightly", "miri", "--version"]
```

**违反**：未违反——miri 的 unsupported 路径都 exit ≠ 0（详见 README 与 cc-report 论证）；与 kmir 的 K-stuck silent path（K cell 残留但 CLI exit 0）不同，miri 设计上不存在 silent skip。

**推理链**：根据 [`deep-reports/cc-reports/miri.md`](../../../../deep-reports/cc-reports/miri.md) 与 miri 上游源码，miri 的 reject 路径统一通过 `throw_unsupported_format!` 等 macro 升级为 InterpError，最终通过 `process::exit(1)` 暴露。无 silent emit-stub 路径。

**决策性**：非决策点——miri 设计上已封死 silent path。

**建议**：无须修改。但 README 对此可加一句"miri 设计上 reject 都走 InterpError → exit ≠ 0，无 silent emit-stub 路径"。

#3 (严重度: 中) — harness 不捕获 entry 返回值，对返回 `Result` / `#[must_use]` 类型的 entry 可能 silent drop

**现象**：[`tools/miri/harness.rs.tera`](../../../../tools/miri/harness.rs.tera)：

```rust
fn main() {
    {{ target_crate_name }}::{{ entry_fn }}();
}
```

**违反**：未违反——`#[must_use]` 触发 warning，不影响 miri 行为；解释执行返回值被 drop 也合规。

**推理链**：若 entry 返回 `Result::Err(...)`，main 函数 drop 它（除非该类型在 drop 时 panic）——但 panic 走 exit ≠ 0，oracle 正确捕获。若 entry 返回 `Box<dyn Trait>` 等含 vtable，drop glue 在 miri 下也会被解释——这符合"完整跑完"。

**决策性**：非决策点。

**建议**：可选——`let _ = ...();` 显式 drop 与其他工具体例一致（cargo-check / kmir 同样不显式 drop，所以维持现状也可接受）。

#4 (严重度: 中) — timeout_secs = 300 对深递归 / 大循环 entry 可能不够

**现象**：[`tools/miri/tool.toml`](../../../../tools/miri/tool.toml) L2：`timeout_secs = 300`。README L48 也提示"解释执行比原生慢 10-100 倍"。

**违反**：未违反——300s 是 runner 默认值的两倍。

**推理链**：若某 entry 在原生跑 1s（深嵌套循环），miri 上跑 100s——仍在 300s 内。但 industrial/rsa-pkcs8 这种 entry 含 modpow（含 ~100 次幂运算 + bigint），miri 跑可能数小时。300s 不够，会 timeout=true → status=Failed → 框架记 FAILED。

按 [`principles.md`](../../../design/principles.md) §七"性能不算问题，除非升级为功能问题"——timeout 让原本能 SUCCESS 的 entry 错判 FAILED 是功能问题。

**决策性**：决策点——是否区分"前端 reject"与"解释执行 timeout"？

按 [`principles.md`](../../../design/principles.md) §三-3-2.b "上限保证（不冤枉能力）"——miri timeout 实际上把"能跑完的 entry"误判 FAILED，违反上限保证。但 [`principles.md`](../../../design/principles.md) §六-1 又把 miri 归为"秒级开销可接受"——隐含"timeout 不应频繁触发"，矛盾不显著。

**建议**：可选——industrial 类目下的 entry 可能需要更长 timeout。但若上调到 1800s，整个矩阵 run 时长激增。当前 300s 是工程上的合理折衷，宜在 README "已知限制 / 坑" 段显式提示。当前 README L48 已经提到"循环展开量大的样例需注意超时风险"——这点已覆盖。

#5 (严重度: 低) — `cargo +nightly miri` 不显式 pin 具体 nightly 日期

**现象**：[`tools/miri/tool.toml`](../../../../tools/miri/tool.toml) L1：`"cargo", "+nightly", "miri"`——使用 rustup 通用 nightly channel。

**违反**：[`tool-integration.md`](../../../design/tool-integration.md) §一 "锁定 commit / 版本号 / nightly toolchain pin"——但 miri component 跟随 nightly toolchain 走，README L39 显式说"跟随当前 nightly toolchain 中 miri component 的可用版本"，这是诚实声明。

**推理链**：miri 是 rustup component，其版本与 nightly toolchain 绑定。pin 到具体日期需在 README 里写 `nightly-2025-XX-XX`。当前用 `+nightly` 容许 nightly drift。

**决策性**：非决策点——按 README 自陈，时效性已由 `results.json` 的 version_command 捕获。

**建议**：可选——若 deep-reports / cc-reports 需要严格可重现，pin 到具体日期会更稳。但 baseline 容忍 drift 在 baseline 模块（次要模块 3）允许。

#6 (严重度: 低) — version_command 用 `cargo +nightly miri --version` 而非 `miri --version`

**现象**：[`tools/miri/tool.toml`](../../../../tools/miri/tool.toml) L3：`version_command = ["cargo", "+nightly", "miri", "--version"]`

**违反**：未违反。

**推理链**：`cargo +nightly miri --version` 输出形如 `miri 0.1.0 (commit-hash 2026-...)` ——同时含 commit-hash 与日期，比单纯 `miri --version` 更详细。合规。

**决策性**：非决策点。

**建议**：无须改。

## §4 决策点 vs 非决策点

- 决策点：1（timeout_secs = 300 vs 1800 的权衡——但当前合理，无须改）
- 非决策点：5

## §5 审查结论

miri 配置整体合规且简洁——它依赖 miri 自身设计的 reject 通路（InterpError → exit ≠ 0），无需 wrapper 后处理。harness 朴素。

需补强的地方：

1. README 应显式说明"不启用 `-Zmiri-disable-isolation`，让 OS 调用走 exit ≠ 0"——当前用户读 README 不会注意到 isolation default 选择背后的方法学考量。
2. README "已知限制 / 坑" 已提到 timeout 风险，但 industrial 类目 entry 可能需更长 timeout——可在 README 或 cc-report 显式列出受影响 entry。

无 critical 问题；整体属"低维护成本工具集成"——这与 miri 是 Rust 工具链自带 component 的稳定性一致。
