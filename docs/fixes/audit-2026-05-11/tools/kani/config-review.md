# kani config Review

## §1 问题意识

kani 是矩阵中的 SMT-based 验证工具——前端是 MIR → GotoC codegen，后端是 CBMC SAT/SMT 求解。按宪法 §六-1 前端 / 后端切分原则，应用 `--only-codegen` 切到 codegen 之前停下。oracle 设计上需防"codegen-with-unsupported-stub 漏报"——kani 用 stub 替代 unsupported 后**仍 exit 0** + 印 warning。wrapper 抓 5 marker（TerminatorKind::InlineAsm / simd_cast / catch_unwind / ptr_mask / C string literal）翻转为 FAILED。

恶意角度考察：
1. 5 marker 是否漏抓上游新加的 stub 路径？
2. `caller_location` / `foreign function` 排除是否真不引入误报？
3. wrapper 的 exit code 语义（exit 2）是否被 runner 正确视为 FAILED？
4. `--only-codegen` 在 kani 最新版本是否仍是同一行为？
5. P12-P16 audit 调整是否完整落地到 wrapper？

## §2 审查方法

参照源：

- [`docs/design/principles.md`](../../../design/principles.md) §六-1 前端 / 后端切分（kani 用 `--only-codegen`）；
- [`docs/design/principles.md`](../../../design/principles.md) §六-2 不允许 partial；
- [`docs/design/principles.md`](../../../design/principles.md) §六-4 反作弊推论；
- [`docs/design/tool-integration.md`](../../../design/tool-integration.md) §三 / §四 形式严格性论证；
- [`docs/fixes/oracle-leak-audit-2-2026-05-11.md`](../../oracle-leak-audit-2-2026-05-11.md) §3.1（kani §C1 codegen-with-stub 漏报）；
- [`docs/fixes/oracle-leak-rules-implementation-2-2026-05-11.md`](../../oracle-leak-rules-implementation-2-2026-05-11.md) §2.1；
- [`runner/src/exec.rs`](../../../../runner/src/exec.rs) L256-260 status 判定（!timed_out && exit_status.success()）；
- [`deep-reports/cc-reports/kani.md`](../../../../deep-reports/cc-reports/kani.md)。

## §3 审查现象

#1 (严重度: 中) — wrapper exit code 2 与 runner exit code 判定的"非零即 Failed" 语义重叠，但 runner 不区分 1 / 2 / 101

**现象**：[`tools/kani/kani-strict-wrapper.sh`](../../../../tools/kani/kani-strict-wrapper.sh) L120-127：

```bash
if [[ -n "$hit" ]]; then
    cat >&2 <<EOF
[kani-oracle] FAIL: codegen completed with hard-unsupported MIR constructs
...
EOF
    exit 2
fi
```

[`runner/src/exec.rs`](../../../../runner/src/exec.rs) L256-260：

```rust
let status = if !timed_out && exit_status.success() {
    Status::Success
} else {
    Status::Failed
};
```

**违反**：未违反——`exit_status.success()` 是 `exit_code == 0`，非零均判 Failed。wrapper exit 2 与 kani 直接 exit 101 在 runner 看来都是 FAILED。

**推理链**：wrapper 选 exit 2 的语义意图（区分"wrapper 触发 reject" vs "kani 自身 error"）只在 raw stderr / stdout 里通过 `[kani-oracle] FAIL` 字符串体现。读者从 `results.json` 看不到 wrapper vs kani 的差异——除非读 raw stdout。

**决策性**：非决策点——runner 只看 exit code 二值化是宪法约定（[`architecture.md`](../../../design/architecture.md) §五 协议约定）；exit 2 vs 101 区分仅作 raw 输出层语义。

**建议**：可选——在 README 显式说明"wrapper exit 2 = oracle 触发的 FAILED；其他非 0 = kani 自身 reject"——便于后续 deep-reports 分类。

#2 (严重度: 中) — `command = ["${TS_PROJECT_ROOT}/tools/kani/kani-strict-wrapper.sh"]` 单元素 argv，依赖 cwd / 副本目录 cargo metadata

**现象**：[`tools/kani/tool.toml`](../../../../tools/kani/tool.toml) L27：

```
command = ["${TS_PROJECT_ROOT}/tools/kani/kani-strict-wrapper.sh"]
```

wrapper L93：`cargo kani --only-codegen --bin __ts_harness >"$out_file" 2>&1`——没有显式 `cd`。

**违反**：未违反——[`runner/src/exec.rs`](../../../../runner/src/exec.rs) L155 `command_builder.current_dir(&target_in_workdir)` 把 cwd 设为副本目录。wrapper 启动时 cwd 已正确。

**推理链**：wrapper 依赖隐式 cwd——若有用户手动跑 wrapper 来调试，必须先 `cd` 到 example workdir。可读性上不显式。

**决策性**：非决策点。

**建议**：可选——wrapper 头部加注释"cwd 由 runner 设为副本目录；手动调用前需 cd 到 example workdir"。

#3 (严重度: 高) — wrapper grep regex 仅抓 5 marker bullet line，未防御 kani 上游改 warning 格式

**现象**：[`tools/kani/kani-strict-wrapper.sh`](../../../../tools/kani/kani-strict-wrapper.sh) L108：

```bash
hit=$(grep -E '^[[:space:]]+-[[:space:]]+(TerminatorKind::InlineAsm|simd_cast|catch_unwind|ptr_mask|C string literal)\b' "$out_file" 2>/dev/null | head -5)
```

**违反**：[`tool-integration.md`](../../../design/tool-integration.md) §四-4.2"防漏报机制 + 反误报检查"——若 kani 上游改 warning 格式（如不用 bullet `- ` 而用其他形式），regex 不命中，silent partial 重新出现。

**推理链**：当前 regex 假设 kani 输出格式："` - <construct> (<count>)`"。这是 kani 当前版本（README 未 pin 到具体 commit）的输出格式。若 kani 升级，可能改 warning 渲染（如 JSON / 不同前缀）。

按 [`tool-integration.md`](../../../design/tool-integration.md) §四-4.4 漏报盲点诚实声明——README 应明示"5-marker grep 假设 kani warning 渲染格式不变"。

**决策性**：决策点——是否在 README 显式声明 grep 对 kani 输出格式的依赖。

**建议**：在 README L48 "漏报盲点" 段补"kani 未来若改 'Found the following unsupported constructs:' warning 渲染格式（如 JSON 输出 / 不同前缀），grep 需同步更新"。当前 README L48 已提"kani 未来新增 unsupported MIR 节点类别"，但未提"现有 marker 渲染格式变更"——补充更完整。

#4 (严重度: 中) — wrapper 后处理：`exit "$rc"` 把 kani 原 exit code 传出，但 exit 2 路径只在 exit 0 + grep 命中时触发

**现象**：[`tools/kani/kani-strict-wrapper.sh`](../../../../tools/kani/kani-strict-wrapper.sh) L99-103：

```bash
if [[ $rc -ne 0 ]]; then
    exit "$rc"
fi
```

**违反**：未违反——保持 kani 原 exit code（如 101 / 1）是合规的。但 wrapper 没有 catch "exit ≠ 0 + 仍有 5 marker" 的复合场景——理论上 kani 可能 exit 1 + 5 marker。

**推理链**：实际上 kani exit 1 时早已 abort codegen，不会再印 unsupported warning。所以 "exit ≠ 0 + 5 marker" 是空集——wrapper 没漏。但 wrapper 注释（L80）没明确这点。

**决策性**：非决策点。

**建议**：可选——wrapper 注释加一句"exit ≠ 0 路径：kani 已 abort codegen，无 5-marker warning 输出可能。"

#5 (严重度: 中) — `caller_location` / `foreign function` 排除论据强但缺更新机制

**现象**：[`tools/kani/kani-strict-wrapper.sh`](../../../../tools/kani/kani-strict-wrapper.sh) L34-41：

> These two are kani's *standard* handling of std internals and not a sign that user code triggered a hard-unsupported MIR construct. Rejecting on them would cause mass false positives (≥ 40% of SUCCESS turns FAILED for generic std-using code).

**违反**：未违反——按 [`tool-integration.md`](../../../design/tool-integration.md) §四-4.2 双向实测验证：60-63/144 SUCCESS 触发，排除合规。但 README L48 写"漏报盲点：`caller_location` / `foreign function` 在 kani 上仍 codegen 为 stub 但 oracle 不抓"——明确声明盲点。这符合 §四-4.4。

**推理链**：诚实声明已到位。但若 kani 上游修复 caller_location 真正 lower（不再 stub），现有"40% 假阳性"假设失效，oracle 应重新加入这两个 marker。当前没有自动检测机制。

**决策性**：决策点——是否预留 kani 上游修复检测。

**建议**：可选——在 README 加注释"若 kani 上游消除 caller_location / foreign function stub（实现真 lowering），oracle 应重新加入这两个 marker"——给后续维护者方向。

#6 (严重度: 中) — wrapper 用 `cargo kani` 但 PATH 未显式管控

**现象**：[`tools/kani/kani-strict-wrapper.sh`](../../../../tools/kani/kani-strict-wrapper.sh) L93：

```bash
cargo kani --only-codegen --bin __ts_harness >"$out_file" 2>&1
```

直接调用 `cargo kani`，依赖 PATH。`tool.toml` L27 也未注入 PATH——依赖 runner 继承的 shell PATH。

**违反**：未违反——按 [`tool-integration.md`](../../../design/tool-integration.md) §一，安装方式说明在 README 而非 wrapper。

**推理链**：若用户机器有多个 cargo（stable / nightly / rustup proxy），PATH 第一个 cargo 决定。kani-verifier 通过 `cargo install kani-verifier` 装到 `~/.cargo/bin/`，标准 PATH 应包含。

**决策性**：非决策点。

**建议**：无须改。

#7 (严重度: 低) — harness `#[kani::proof] fn ts_proof()` + `fn main() {}` 双重 entry

**现象**：[`tools/kani/harness.rs.tera`](../../../../tools/kani/harness.rs.tera)：

```rust
#[kani::proof]
fn ts_proof() {
    {{ target_crate_name }}::{{ entry_fn }}();
}

fn main() {}
```

**违反**：未违反——kani 需要 `#[kani::proof]` 入口；同时 bin target 需要 `fn main()`。空 main 合规。

**推理链**：kani 走 `#[kani::proof]` codegen，main 不参与 codegen。harness 形态合规。

**决策性**：非决策点。

**建议**：无须改。

#8 (严重度: 低) — README cc-reports 与 wrapper 注释互相引用 docs/fixes/oracle-leak-audit-2-2026-05-11.md

**现象**：[`tools/kani/kani-strict-wrapper.sh`](../../../../tools/kani/kani-strict-wrapper.sh) L3-6 与 [`tools/kani/README.md`](../../../../tools/kani/README.md) L43 都引用 [`docs/fixes/oracle-leak-audit-2-2026-05-11.md`](../../oracle-leak-audit-2-2026-05-11.md) §3.1。

**违反**：未违反——交叉引用一致。

**决策性**：非决策点。

**建议**：无须改。

## §4 决策点 vs 非决策点

- 决策点：3（README grep 格式依赖声明 / caller_location 重新加入机制 / wrapper exit 2 vs 101 在 cc-report 中是否标记）
- 非决策点：5

## §5 审查结论

kani 配置与 wrapper 经过 P12 / oracle-leak-audit-2 调整后已封堵 §C1 codegen-with-stub 漏报，论证扎实，引用完整。

最值得补强：

1. **README**：补一句"grep 假设 kani warning 渲染格式不变"——当前 5-marker grep 对 kani 输出 bullet 格式敏感。
2. **README**：补 caller_location / foreign function 重新加入的触发条件——kani 上游真正 lower 这两个 stub 时应同步更新 oracle。

整体属于"经过 2 轮 oracle audit 收紧后的稳态配置"，无 critical 问题。
