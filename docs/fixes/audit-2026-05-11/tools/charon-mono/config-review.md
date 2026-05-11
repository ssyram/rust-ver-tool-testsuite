# charon-mono config Review

## §1 问题意识

charon-mono 是矩阵中 2 个 charon 配置之一——单态化模式（`--monomorphize`）。oracle 依赖 charon 的 `--abort-on-error` flag 把 silent skip 暴露为 exit ≠ 0（charon 默认遇内部 panic 仍 exit 0，是 charon 设计 quirk）。还需 `-- --lib --target aarch64-apple-darwin` 绕 macOS arm64 rlib 路径假设（P8 commit fix）。

恶意角度考察：
1. `--abort-on-error` 是否真把所有 silent path 抓住？
2. macOS arm64 假设是否仅限于该平台——在 Linux 上会失效吗？
3. `--print-llbc` 输出到 stdout 是否影响 runner buffering？
4. `${TS_CHARON_BIN}` 与 charon-poly 共享同一 binary——是否真的同 commit？

## §2 审查方法

参照源：

- [`docs/design/principles.md`](../../../design/principles.md) §六-1 / §六-2；
- [`docs/design/tool-integration.md`](../../../design/tool-integration.md) §三-反向证明 / §四-4.1 形式证明；
- [`tools/charon-mono/tool.toml`](../../../../tools/charon-mono/tool.toml)；
- [`deep-reports/cc-reports/charon-mono.md`](../../../../deep-reports/cc-reports/charon-mono.md)；
- charon 上游 commit `ed22146b`。

## §3 审查现象

#1 (严重度: 中) — tool.toml command 含 `--target aarch64-apple-darwin` 硬编码 macOS arm64

**现象**：[`tools/charon-mono/tool.toml`](../../../../tools/charon-mono/tool.toml) L1：

```
command = ["${TS_CHARON_BIN}", "cargo", "--monomorphize", "--abort-on-error", "--print-llbc", "--", "--lib", "--target", "aarch64-apple-darwin"]
```

**违反**：[`tool-integration.md`](../../../design/tool-integration.md) §七-7.3 报告头部锚定——`--target` 硬编码只在 macOS arm64 work。Linux x86_64 用户跑 runner 会因 target 不匹配失败。

**推理链**：README L57 提到"macOS arm64 同样必须加 `--lib --target aarch64-apple-darwin`"——是 macOS-specific workaround（charon 的 rlib 路径 bug，P8 commit fix）。但 tool.toml 硬编码——非可移植。

**决策性**：决策点——是否参数化 target / 自动检测 host triple。

**建议**：可选——把 `aarch64-apple-darwin` 改为 `${TS_HOST_TRIPLE}`，让用户在 `.env` 配置自己的 host triple。当前实施在 macOS arm64 上 work，但 Linux 用户需手动改 tool.toml。

#2 (严重度: 高) — `--abort-on-error` 论据扎实，但 wrapper 不存在——风险全压在 charon flag 单点

**现象**：[`tools/charon-mono/tool.toml`](../../../../tools/charon-mono/tool.toml) 全文 3 行，无 wrapper：

```
command      = ["${TS_CHARON_BIN}", "cargo", "--monomorphize", "--abort-on-error", "--print-llbc", "--", "--lib", "--target", "aarch64-apple-darwin"]
timeout_secs = 600
version_command = ["${TS_CHARON_BIN}", "version"]
```

**违反**：未违反——按 [`tool-integration.md`](../../../design/tool-integration.md) §三-反向证明，`--abort-on-error` + charon-driver/driver.rs:143 `register_error!` 单一通路是形式可证 0 误报 / 0 漏报的论据。

**推理链**：与 aeneas（Errors.error_list 单一通路）同模式——若 charon `--abort-on-error` 真按 README L26 描述工作（"`register_error!` 在第一次错误就 panic"），oracle 形式可证。当前论据扎实，**但前提是 charon 上游不引入新 silent path**。README L29-30 已声明无盲点。

**决策性**：非决策点——`--abort-on-error` 是 charon 上游提供的反 silent path flag，论据完整。

**建议**：可选——按 hax-coq / rocq-of-rust 体例，加 entry_fn 存在性 grep 作为额外封堵（防 charon 上游引入新 silent skip-item 路径）。当前未做这个 belt-and-braces，但 README L29-30 自信"0 漏报形式可证"。

#3 (严重度: 中) — `--print-llbc` 输出到 stdout——runner 用 reader threads 异步读，不存在 buffer 阻塞，但 stdout 体量大时影响性能

**现象**：[`tools/charon-mono/tool.toml`](../../../../tools/charon-mono/tool.toml) L1 `--print-llbc`。

[`runner/src/exec.rs`](../../../../runner/src/exec.rs) L193-201 reader threads 异步读 stdout / stderr 到内存 buffer。

**违反**：未违反——按 [`principles.md`](../../../design/principles.md) §七 性能不算问题，除非升级为功能问题。

**推理链**：`--print-llbc` 输出大型 LLBC 可能是 MB 级别，runner 把它一次性读到内存——理论上 OOM 风险，但 corpus 中 entry 都是小 lib，不存在。

**决策性**：非决策点。

**建议**：无须改。考虑用 `--dest-dir` 让 LLBC 落盘，runner 仅看 exit code——更省内存。但当前实施足够。

#4 (严重度: 低) — `${TS_CHARON_BIN}` 展开正确——与 aeneas-* 同模式

**现象**：[`tools/charon-mono/tool.toml`](../../../../tools/charon-mono/tool.toml) L1 用 `${TS_CHARON_BIN}` —— runner expand_env 在 read tool.toml 时展开（process env 已有 TS_CHARON_BIN）。

**违反**：未违反——与 aeneas-lean / aeneas-coq / aeneas-fstar / aeneas-hol4 同模式。

**决策性**：非决策点。

**建议**：无须改。

#5 (严重度: 低) — README L26 引用 `charon-driver/driver.rs:143` 论据合规

**现象**：[`tools/charon-mono/README.md`](../../../../tools/charon-mono/README.md) L26：

> **partial 暴露机制**：`--abort-on-error` 让 charon 内部任何 unsupported 项触发 panic + exit 1。`charon-driver/driver.rs:143` 设 `error_ctx.continue_on_failure = false`，`register_error!` 在第一次错误就 panic

**违反**：未违反——[`tool-integration.md`](../../../design/tool-integration.md) §三-源码层穷尽。行号锚定于 charon commit `ed22146b`。

**决策性**：非决策点。

**建议**：无须改。

#6 (严重度: 低) — entry_mode 默认 "bin" + `--lib` flag 让 charon 跳过 harness

**现象**：[`tools/charon-mono/tool.toml`](../../../../tools/charon-mono/tool.toml) 未显式 entry_mode；[`tools/charon-mono/harness.rs.tera`](../../../../tools/charon-mono/harness.rs.tera) 是 `fn main() { ... }` bin 形态。但 tool.toml 用 `-- --lib`——cargo 只翻译 lib target，harness（bin target）被跳过。

**违反**：未违反——README L51 明确说明"harness 写到 `src/bin/__ts_harness.rs`，但实际被 `--lib` 跳过编译"。

**推理链**：合规但浪费——harness 文件存在但不参与翻译。可考虑改 entry_mode = "lib"，但那需要 charon-mono 翻译 lib，与现实"通过 cargo 驱动"无差，所以维持现状无害。

**决策性**：非决策点。

**建议**：无须改。

#7 (严重度: 低) — version_command 调用 `${TS_CHARON_BIN} version` —— charon 自身有 `version` 子命令

**现象**：[`tools/charon-mono/tool.toml`](../../../../tools/charon-mono/tool.toml) L3：`version_command = ["${TS_CHARON_BIN}", "version"]`。

**违反**：未违反——charon 0.1.184 支持 `version` 子命令。

**决策性**：非决策点。

**建议**：无须改。

## §4 决策点 vs 非决策点

- 决策点：1（macOS arm64 硬编码 vs 参数化 target）
- 非决策点：6

## §5 审查结论

charon-mono 配置整体合规：

- `--abort-on-error` 形式可证 0 误报 / 0 漏报论据扎实，引用 charon-driver/driver.rs:143 上游源码；
- 与 charon-poly 共享 binary（同 commit），差仅在 `--monomorphize` flag；
- macOS arm64 workaround 诚实记录在 README。

最值得调整：

1. **target 硬编码**：把 `aarch64-apple-darwin` 改为 `${TS_HOST_TRIPLE}` ——让 Linux 用户也能直接跑。当前限定 macOS arm64 单平台，与 baseline 测试通用性目标不一致。

整体属于"形式严格的优秀集成"，与 aeneas family 同级。
