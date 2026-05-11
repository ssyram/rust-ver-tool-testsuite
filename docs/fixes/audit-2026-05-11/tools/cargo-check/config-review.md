# cargo-check config Review

## §1 问题意识

cargo-check 在矩阵中是 baseline——若它在某 entry FAILED，则该 entry 自身 Rust 不合法，与任何验证器能力无关。所以它的配置必须最简洁，oracle 必须严格等于 `cargo check` 自带 exit code，且 harness 形态绝不能引入"工具适配"的污染。

恶意角度的考察重点：
1. tool.toml 字段是否完备（缺 timeout_secs 会落到 default 300s——这是问题吗？）；
2. command 是否走最快路径——是否会被 cargo 默认的 bin auto-discovery 假象耍到（`--bin __ts_harness`）；
3. harness 是否真的"零工具适配"，仅是普通 main 调用 lib 中 entry。

## §2 审查方法

- 参照源：[`docs/design/principles.md`](../../../design/principles.md) §三-1（baseline 模块定位）、§六-3（不区分翻译深浅，cargo-check 是 baseline 不参与 partial 抓取）；
- [`docs/design/tool-integration.md`](../../../design/tool-integration.md) §五（必含章节）——但 cargo-check 不在 §六-硬指标 覆盖范围（baseline 不需要 0 误报 / 0 漏报论证，因为它自身定义就是 rustc 的 exit code）；
- [`runner/src/discover.rs`](../../../../runner/src/discover.rs) L271-283 `ToolToml` struct 期望字段；
- [`runner/src/exec.rs`](../../../../runner/src/exec.rs) L98-99 `crate_ident = example.crate_name.replace('-', "_")` 模板变量绑定。

## §3 审查现象

#1 (严重度: 低) — tool.toml 不显式声明 timeout_secs

**现象**：[`tools/cargo-check/tool.toml`](../../../../tools/cargo-check/tool.toml) 全文 2 行，仅有 `command` 与 `version_command`，无 `timeout_secs`。

**违反**：未违反——`ToolToml::timeout_secs` 有 `#[serde(default = "default_timeout")]` 默认 300s（[`discover.rs:285-287`](../../../../runner/src/discover.rs)）。

**推理链**：cargo-check 在 baseline 用途上一般秒级返回，300s 默认绰绰有余。但每个矩阵 run 上若有数百 entry，整体时间敏感时不显式声明 timeout 是潜在风险——例：某 entry 上 cargo registry flock 阻塞导致 fetch 慢，但 cargo-check 本身无内在原因卡住 5 分钟。

**决策性**：非决策点——默认值合适，显式声明只是文档清晰度问题，不强求修改。

**建议**：可选——在 tool.toml 注释里写一句"timeout 用默认 300s（baseline 通常 < 10s）"。

#2 (严重度: 低) — command 使用 `--bin __ts_harness` 强依赖 harness 渲染目标命名

**现象**：[`tools/cargo-check/tool.toml`](../../../../tools/cargo-check/tool.toml) L1：

```
command = ["cargo", "check", "--bin", "__ts_harness"]
```

**违反**：未违反，但耦合：`__ts_harness` 是 runner 在 [`exec.rs:112`](../../../../runner/src/exec.rs) 写入的 bin 文件名，硬编码在 runner 代码里。tool.toml 与 runner 内部命名约定强耦合。

**推理链**：若 runner 未来修改 bin 文件名（如 `_ts_harness` 或 `__hirusttest_harness`），cargo-check 的 tool.toml 必须同步——这是隐藏依赖。但按"通过配置而非代码实现工具兼容"原则，bin 名作为框架公约存在于 runner 内是合理的（C 原则：异质性归配置，但同质命名约定可在框架内部）。

**决策性**：非决策点——bin 名约定是框架内部细节。

**建议**：无须修改；记录 `__ts_harness` 是框架内部公约即可。

#3 (严重度: 低) — harness `fn main() { <crate>::<entry>(); }` 不显式处理返回值

**现象**：[`tools/cargo-check/harness.rs.tera`](../../../../tools/cargo-check/harness.rs.tera)：

```rust
fn main() {
    {{ target_crate_name }}::{{ entry_fn }}();
}
```

**违反**：未违反。但与 hax-coq / hax-fstar harness 不同——它们用 `let _ = ...`。若某 entry fn 签名 `pub fn foo() -> u32 { 42 }`（如 rocq-of-rust 的 entry 形态），cargo check 会触发 `unused_result` lint（如 entry 返回 `#[must_use]` 类型如 `Result`）—— 但只是 warning，不影响 exit code。

**推理链**：cargo-check entry 由 corpus 决定形态。corpus 自包含——entry 是 `pub fn xxx()` 零参，返回任意。`Result` 等 `#[must_use]` 类型可能产生 warning 但不阻 exit 0。所以 oracle 不变。

**决策性**：非决策点——warning 不影响 SUCCESS 判定。

**建议**：可选改为 `let _ = {{ target_crate_name }}::{{ entry_fn }}();`，与 hax-coq / soteria 一致。但若 entry 返回 `()`，会触发 `let_underscore_drop` lint——同样 warning。维持现状即可。

#4 (严重度: 低) — version_command 只跑 `cargo --version`，不区分 toolchain

**现象**：[`tools/cargo-check/tool.toml`](../../../../tools/cargo-check/tool.toml) L2：

```
version_command = ["cargo", "--version"]
```

**违反**：[`tool-integration.md`](../../../design/tool-integration.md) §一"工具版本锁定"——`version_command` 是 runner 每次 run 捕获工具版本字符串的方式。`cargo --version` 输出 stable cargo 版本，但不包含 rustc 版本——而 cargo-check 的真实判定者是 rustc。

**推理链**：若某 entry FAILED 是因为 rustc nightly 改了某 lint 默认值，单看 `cargo 1.83.0` 不知道发生在哪个 rustc。

**决策性**：非决策点——cargo-check baseline 通常稳定，且 cargo 版本基本与 rustc 一对一绑定。

**建议**：可补强 `["sh", "-c", "cargo --version; rustc --version"]`。但当前用法在 baseline 语境下足够。

#5 (严重度: 低) — README 自陈"形式可证 0 误报 / 0 漏报"，但实际 cargo-check 不在 §六 硬指标覆盖范围

**现象**：[`tools/cargo-check/README.md`](../../../../tools/cargo-check/README.md) L24-26：

> **形式严格性 — 0 误报（不冤枉能力）**：✅ 形式可证。rustc 单一信号，exit 0 ⇔ type / borrow check 全部通过

**违反**：未严格违反——但需要注意 cargo-check 不属于"市面工具样例"（次要模块 3），它是矩阵 baseline。按 [`principles.md`](../../../design/principles.md) §六-本节自我性声明，硬指标针对验证类工具。

**推理链**：cargo-check 写"0 误报 / 0 漏报"是合理但越界——它本身定义就是 rustc exit code，等价于自指。这条声明无害但语义上略空。

**决策性**：非决策点。

**建议**：保留——为统一 README 体例。

## §4 决策点 vs 非决策点

- 决策点：0
- 非决策点：5

cargo-check 配置非常简单且作用明确，无显著问题。

## §5 审查结论

cargo-check 的 config + wrapper + harness 三件无显著问题。它作为 baseline，配置应该简到极致——目前正是。所有 5 条审查现象都是非决策点（默认值、命名约定、warning 不影响 oracle），无需修复。

唯一可考虑（非必须）的小补强：`version_command` 同时捕 cargo + rustc 版本，让 baseline 出错时溯源更精确。
