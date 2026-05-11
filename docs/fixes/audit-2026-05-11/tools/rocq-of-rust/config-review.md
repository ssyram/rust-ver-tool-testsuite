# rocq-of-rust config Review

## §1 问题意识

rocq-of-rust 是矩阵中 oracle 最复杂的工具——6 道门 + N-attempt（默认 7）AND-reduce。封堵两类 silent path：(a) audit-1 §3.2 "完全 skip item" 漏报（top_level.rs:349-390 走 vec![]）；(b) 2026-05-11 P15-impl 反向暴露的非确定性翻译路径（`thread_local!` 宏 entry 上 P(drop entry_fn) ≈ 40%）。

恶意角度考察：
1. N=7 是否真把 P(漏报) 压到 99.84% catch？
2. 6 道门覆盖完整？
3. wrapper 与 README 一致？
4. `TS_ENTRY_FN` 注入路径（runner/src/exec.rs:178）？

## §2 审查方法

参照源：

- [`docs/design/principles.md`](../../../design/principles.md) §六-2 / §六-4；
- [`docs/design/tool-integration.md`](../../../design/tool-integration.md) §三 / §四；
- [`tools/rocq-of-rust/tool.toml`](../../../../tools/rocq-of-rust/tool.toml)；
- [`tools/rocq-of-rust/rocq-of-rust-wrapper.sh`](../../../../tools/rocq-of-rust/rocq-of-rust-wrapper.sh)；
- [`docs/fixes/oracle-leak-audit-2026-05-08.md`](../../oracle-leak-audit-2026-05-08.md) §3.2；
- [`docs/fixes/ror-gate6-fix-2026-05-11.md`](../../ror-gate6-fix-2026-05-11.md)；
- [`runner/src/exec.rs:178`](../../../../runner/src/exec.rs)（TS_ENTRY_FN 注入）。

## §3 审查现象

#1 (严重度: 高) — 6 道门 + N-attempt 设计——矩阵 oracle 最复杂

**现象**：[`tools/rocq-of-rust/rocq-of-rust-wrapper.sh`](../../../../tools/rocq-of-rust/rocq-of-rust-wrapper.sh) L94-119 实施 6 道门 + L121-138 N-attempt 循环。

**违反**：未违反——按 [`tool-integration.md`](../../../design/tool-integration.md) §四-4.2 防漏报机制。**矩阵中 oracle 设计最复杂的工具**——6 道门覆盖：

1. exit code = 0；
2. ≥ 1 .v 产物；
3. 无 0-byte .v；
4. ≥ 1 .v > 200B；
5. 无 5 类显式 failure marker；
6. entry_fn 在 .v 中以 `Definition <fn>` 出现。

**推理链**：与 verifast `-verbose 1` 反作弊设计同性质——通过工具自身产物 signal 抓 silent path。rocq-of-rust 反向更甚——工具设计上 exit 0 对所有 unsupported case，所以 oracle 完全靠产物 grep + 产物 shape + N-attempt。

**决策性**：非决策点——是 rocq-of-rust 反 silent path 的必要机制。

**建议**：无须改。

#2 (严重度: 高) — N-attempt 默认 7 实证校准（P(drop)=0.4 → catch rate 99.84%）

**现象**：[`tools/rocq-of-rust/rocq-of-rust-wrapper.sh`](../../../../tools/rocq-of-rust/rocq-of-rust-wrapper.sh) L77-91：

```bash
# N=7 gives 1 - 0.4^7 ≈ 99.84% catch rate at ≈ 35 ms / entry overhead...
N_ATTEMPTS=${ROCQ_OF_RUST_N_ATTEMPTS:-7}
```

**违反**：未违反——按 [`tool-integration.md`](../../../design/tool-integration.md) §四-4.2 实证 calibration。N=7 经过 30-run 手测，P(drop)=0.4 → catch rate 99.84%。

**推理链**：实证 calibration 论证扎实。N=3 仅 93.6% catch（每 16 runs 翻一次），太 flaky；N=7 99.84% 是工程合理选择。

**决策性**：非决策点——实证 calibration。

**建议**：无须改。

#3 (严重度: 中) — `TS_ENTRY_FN` 注入路径——runner/src/exec.rs:178

**现象**：[`tools/rocq-of-rust/tool.toml`](../../../../tools/rocq-of-rust/tool.toml) L51-54：

> Env contract: ROCQ_OF_RUST_TOOLCHAIN_SYSROOT is re-exported from TS_* (runner strips TS_* before spawn — runner/src/exec.rs:165) so the wrapper script sees it as plain. TS_ENTRY_FN is re-injected by the runner after the strip (runner/src/exec.rs:178), so the wrapper accesses it directly.

[`runner/src/exec.rs`](../../../../runner/src/exec.rs) L168-179 实施：

```rust
let ts_vars: Vec<String> = std::env::vars()
    .map(|(k, _)| k)
    .filter(|k| k.starts_with("TS_"))
    .collect();
for k in &ts_vars {
    command_builder.env_remove(k);
}
// These are set AFTER env_remove so they survive into the child env.
command_builder.env("TS_ENTRY_FN", entry);
command_builder.env("TS_TARGET_CRATE", &crate_ident);
```

**违反**：未违反——按 [`architecture.md`](../../../design/architecture.md) §五 协议约定，runner 在 strip TS_* 后注入 TS_ENTRY_FN / TS_TARGET_CRATE。wrapper L114 用 `${TS_ENTRY_FN:-}` 读取——一致。

**推理链**：与 hax-coq / hax-fstar 的 `$TS_ENTRY_FN`（无大括号）trap 不同——rocq-of-rust wrapper 用 `${TS_ENTRY_FN:-}` 形式（bash 兼容大括号 + 默认值）。在 bash 内执行（wrapper.sh 是 bash），所以大括号是合规的。

**决策性**：非决策点。

**建议**：无须改。

#4 (严重度: 中) — wrapper L122-123 `rm -rf "$OUTDIR"` 在每次 attempt 前清理——防 N 次产物互污染

**现象**：[`tools/rocq-of-rust/rocq-of-rust-wrapper.sh`](../../../../tools/rocq-of-rust/rocq-of-rust-wrapper.sh) L121-128：

```bash
for ((i = 1; i <= N_ATTEMPTS; i++)); do
    OUTDIR="rocq_translation_${i}"
    rm -rf "$OUTDIR"
    mkdir -p "$OUTDIR"
    "$ROCQ_OF_RUST_BIN" translate --path src/lib.rs --output-path "$OUTDIR" 2>rocq_stderr.log
    ...
```

**违反**：未违反——每 attempt 独立输出目录，AND-reduce 跨 attempt。

**决策性**：非决策点。

**建议**：无须改。

#5 (严重度: 中) — wrapper L74-76 PATH + DYLD_LIBRARY_PATH 注入——macOS 特定

**现象**：[`tools/rocq-of-rust/rocq-of-rust-wrapper.sh`](../../../../tools/rocq-of-rust/rocq-of-rust-wrapper.sh) L74-76：

```bash
SYSROOT="$ROCQ_OF_RUST_TOOLCHAIN_SYSROOT"
export DYLD_LIBRARY_PATH="$SYSROOT/lib"
export PATH="$SYSROOT/bin:$PATH"
```

**违反**：[`tool-integration.md`](../../../design/tool-integration.md) §一——`DYLD_LIBRARY_PATH` 是 macOS 特定（Linux 是 `LD_LIBRARY_PATH`）。

**推理链**：rocq-of-rust binary 由 cargo +nightly-2024-12-07 install 出，依赖 rustc_private API + librustc_driver-*.dylib——macOS 用 `DYLD_LIBRARY_PATH`，Linux 应用 `LD_LIBRARY_PATH`。

**决策性**：决策点——macOS-only 实施。

**建议**：可选——`if [[ "$(uname)" == "Darwin" ]]; then export DYLD_LIBRARY_PATH=...; else export LD_LIBRARY_PATH=...; fi`。当前 macOS 实测 work，Linux 用户需手动调整。

#6 (严重度: 中) — wrapper L140-146 symlink `rocq_translation/` → `rocq_translation_1/`

**现象**：[`tools/rocq-of-rust/rocq-of-rust-wrapper.sh`](../../../../tools/rocq-of-rust/rocq-of-rust-wrapper.sh) L140-146：

```bash
# Symlink rocq_translation/ -> rocq_translation_1/ so downstream consumers
# (e.g. the tier-1 typecheck wrapper, future deep-report scripts, manual
# inspection) keep the historical product path. The choice of attempt 1
# is arbitrary; the contract is "any one of the N translations succeeded
# all gates", so any attempt suffices.
rm -rf rocq_translation
ln -s rocq_translation_1 rocq_translation
```

**违反**：未违反——为下游消费者（rocq-of-rust-typecheck wrapper / cc-report 脚本）保持历史路径。

**推理链**：选 attempt 1 任意——任何 attempt 都满足所有 gate（AND-reduce 通过）。

**决策性**：非决策点。

**建议**：无须改。

#7 (严重度: 低) — timeout_secs = 120 + N=7 × ~5ms ≈ 35ms overhead

**现象**：[`tools/rocq-of-rust/tool.toml`](../../../../tools/rocq-of-rust/tool.toml) L62：`timeout_secs = 120`。

**违反**：未违反——按 wrapper 注释 L83-85 N=7 overhead < 35%。120s 足够。

**决策性**：非决策点。

**建议**：无须改。

#8 (严重度: 低) — harness pub fn 返回 u32 而非 fn main——bin target 但无 main

**现象**：[`tools/rocq-of-rust/harness.rs.tera`](../../../../tools/rocq-of-rust/harness.rs.tera):

```rust
pub fn {{ entry_fn }}() -> u32 {
    42
}
```

**违反**：[`runner/src/exec.rs`](../../../../runner/src/exec.rs) L107-115 entry_mode = "bin" 时 renders to src/bin/__ts_harness.rs。bin target 需 fn main——rocq-of-rust harness 无 main。

**推理链**：但 rocq-of-rust 不通过 cargo 编译 bin target——它直接读 src/lib.rs。所以 harness 不被编译，无 missing-main 问题。tera 模板 L8-15 注释说"This file is NOT passed to rocq-of-rust"。

**决策性**：非决策点——harness 仅作框架契约 placeholder，不被 cargo 编译。

**建议**：可选——把 harness 改为 `fn main() {}` 与其他工具体例一致，更不易引起新维护者疑问。

## §4 决策点 vs 非决策点

- 决策点：2（macOS-only DYLD 注入 / harness fn main 体例对齐）
- 非决策点：6

## §5 审查结论

rocq-of-rust 配置是**矩阵中 oracle 设计最复杂的工具**：

- 6 道门 + N-attempt（默认 7）AND-reduce；
- 实证 calibration（N=7 → 99.84% catch rate）扎实；
- 封堵两类 silent path（audit-1 §3.2 + P15-impl 反向暴露的非确定性）；
- 与 rocq-of-rust-typecheck wrapper 复用（symlink rocq_translation/ → rocq_translation_1/）。

最值得补强：

1. **DYLD_LIBRARY_PATH macOS-only** —— 可加 Linux 兼容；
2. **harness fn main 体例** —— 可与其他工具对齐。

整体属于"两轮 oracle audit + 反向暴露后的稳态配置"，论证质量极高。
