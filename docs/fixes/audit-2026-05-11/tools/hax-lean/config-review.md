# hax-lean config Review

## §1 问题意识

hax-lean 是 hax 3 backend 中 oracle 设计最精巧的一个：(1) cargo hax exit 0 必要；(2) 产物 grep "sorry 在 term 位置" 抓 silent partial（rust-engine/src/backends/lean.rs:1287, 2163 的 `PatKind::Error / error_node` 路径 emit `text!("sorry")` 不发 Diagnostic）。oracle 写在 tool.toml 内联 `sh -c`——单行紧凑。

恶意角度考察：
1. 内联 `sh -c` 中的 awk + grep 正则正确（不抓 binder 位置 sorry）？
2. cargo hax 命令 `-C --lib ';' into lean` 中的 `';'` 是 hax 子参数终止符——是否被 shell 正确解读？
3. `${TS_HAX_ENGINE_BIN}` 展开（用 `${...}` 而非 `$VAR`）——为何不是 `$TS_ENTRY_FN` 模式？因为 TS_HAX_ENGINE_BIN 在 runner 启动时已有，TS_ENTRY_FN 是运行时注入。
4. wrapper 与 README 形式严格性自陈一致？

## §2 审查方法

参照源：

- [`docs/design/principles.md`](../../../design/principles.md) §六-2 / §六-4；
- [`docs/design/tool-integration.md`](../../../design/tool-integration.md) §三 / §四；
- [`tools/hax-lean/tool.toml`](../../../../tools/hax-lean/tool.toml)；
- [`docs/fixes/oracle-leak-audit-2-2026-05-11.md`](../../oracle-leak-audit-2-2026-05-11.md)；
- [`runner/src/discover.rs`](../../../../runner/src/discover.rs) expand_env；
- hax commit `30949eb87058895c24f963df90dd30ef11b0dc1a`。

## §3 审查现象

#1 (严重度: 中) — tool.toml command 用 inline `sh -c` 而非外部 wrapper 脚本

**现象**：[`tools/hax-lean/tool.toml`](../../../../tools/hax-lean/tool.toml) L14-17：

```toml
command = [
  "sh", "-c",
  "env HAX_ENGINE_BINARY=${TS_HAX_ENGINE_BIN} cargo +nightly-2025-11-08 hax -C --lib ';' into lean; rc=$?; if [ $rc -eq 0 ]; then if find proofs/lean/extraction -name '*.lean' -exec cat {} + 2>/dev/null | awk '{ sub(/--.*/, \"\"); print }' | grep -qE '(:=|pure|mk|,)[[:space:]]*sorry\\b|\\bsorry[[:space:]]*[,)\\]]'; then echo '[hax-lean-oracle] FAIL: silent partial — sorry in term position (lean.rs:1287/2163 PatKind::Error / error_node path)' >&2; rc=1; fi; fi; exit $rc"
]
```

**违反**：[`tool-integration.md`](../../../design/tool-integration.md) §四-4.2 防漏报机制——当前实施合规，但**长达 200+ 字符的内联 `sh -c` 不易审计**。kani / verifast / prusti / aeneas 都用外部 wrapper 脚本，hax-lean / hax-coq / hax-fstar 例外。

**推理链**：内联 `sh -c` 让 tool.toml 同时承担"调用配置"和"oracle 实施"两个角色，违反 "config / wrapper 分离" 隐式约定。修改时容易出错（如 quotes 转义问题）。

**决策性**：决策点——是否把 hax-* oracle 抽到独立 wrapper 脚本。

**建议**：可选——抽出 `hax-lean-wrapper.sh`（与 hax-coq / hax-fstar 共享逻辑），让 tool.toml 退化为单元素 command。当前内联实施有效但维护性低。

#2 (严重度: 中) — awk + grep 双向实测验证扎实，但 README 未列具体反向 case 表

**现象**：[`tools/hax-lean/tool.toml`](../../../../tools/hax-lean/tool.toml) L8-13 注释提及双向实测验证：

> grep 精准化（实测验证 0 误报 0 漏报）：
> - 必要：strip Lean `--` 行注释（hax 把 Rust doc comment 翻成 `--` 注释，注释里字面 sorry 不应触发 FAILED）
> - 必要：抓 sorry **只在 term 位置**（`:= sorry` / `pure sorry` / `mk sorry` / `, sorry` / `sorry,` / `sorry)` / `sorry]`），**不**抓 binder 位置（`let sorry :` 是用户合法变量名，不是 partial）

[`tools/hax-lean/README.md`](../../../../tools/hax-lean/README.md) L37-46 也提到实测验证。

**违反**：未违反——按 [`tool-integration.md`](../../../design/tool-integration.md) §四-4.2 双向实测验证已实施。但 README L38 "实测验证：用户合法 `let sorry : i32 := 5;` + doc comment 含 `sorry` 字面字符串都不触发 FAILED；真 partial（`(pure sorry)` / `mk sorry`）稳定触发 FAILED"——证据扎实但未引到独立的"oracle-validation" 目录（与 verifast oracle-validation/ 比对）。

**推理链**：verifast 有独立 `oracle-validation/` 目录存 micro-test 文件；hax-lean 没有，仅靠 README 文字陈述。

**决策性**：非决策点——实测证据已落到 README，独立目录是可选补强。

**建议**：可选——加 `tools/hax-lean/oracle-validation/` 目录存若干 lean 文件，方便后续维护者重跑验证。

#3 (严重度: 高) — grep 正则 `(:=|pure|mk|,)\s*sorry\b|\bsorry\s*[,)\]]` 与 hax 上游产物的对应性是 brittle 依赖

**现象**：[`tools/hax-lean/tool.toml`](../../../../tools/hax-lean/tool.toml) L16 正则。

**违反**：[`tool-integration.md`](../../../design/tool-integration.md) §四-4.3 防漏报机制的意义辨析——"理论意义有限：grep-based 机制**无法形式证明 0 漏报**——总有未观察到的 silent path 形式"。

**推理链**：hax 上游 commit `30949eb` 在 lean.rs:1287, 2163 emit `text!("sorry")`——README L36 引用。若 hax 上游升级（例 PR #1672 合并），sorry path 消失——grep 自然失效（无害）。但若 hax 引入新 sentinel marker（如 `unimplemented_value` / `partial_marker`），grep 不抓。

README L42-44 已声明这条漏报盲点。合规但 brittle。

**决策性**：决策点——是否补强（加 entry_fn 存在性 grep 作为额外封堵）。

**建议**：与 hax-coq / hax-fstar 不同——它们已经加了 entry_fn 存在性 grep（封堵 silent skip-item path）。hax-lean 没有。若 hax 在 lean backend 引入 silent skip-item path（不写 sorry 也不发 Diagnostic），oracle 漏报。可选——加 entry_fn `let` / `def` 存在性 grep。

#4 (严重度: 低) — `${TS_HAX_ENGINE_BIN}` 用 `${...}` 形式——与 hax-coq / hax-fstar 模式不同

**现象**：[`tools/hax-lean/tool.toml`](../../../../tools/hax-lean/tool.toml) L16 用 `${TS_HAX_ENGINE_BIN}`（runner expand_env 在 read tool.toml 时展开，因 TS_HAX_ENGINE_BIN 在 runner 启动时由 .env source）。

**违反**：未违反——按 [`runner/src/discover.rs:294-335`](../../../../runner/src/discover.rs) `expand_env` 行为。这与 hax-coq / hax-fstar 中的 `$TS_ENTRY_FN`（无大括号）trap 不同——TS_ENTRY_FN 是 runner 在 spawn child 前才注入 child env，所以必须用 `$VAR` 让 shell 在运行时展开，而非 runner expand_env 在启动时展开。

**推理链**：hax-lean 不使用 `TS_ENTRY_FN` 因为它没加 entry_fn 存在性 grep。

**决策性**：非决策点。

**建议**：无须改。

#5 (严重度: 低) — version_command 多 line 合理

**现象**：[`tools/hax-lean/tool.toml`](../../../../tools/hax-lean/tool.toml) L19-24：

```toml
version_command = [
  "env",
  "HAX_ENGINE_BINARY=${TS_HAX_ENGINE_BIN}",
  "cargo", "+nightly-2025-11-08",
  "hax", "--version"
]
```

**违反**：未违反——通过 `env` 注入 HAX_ENGINE_BINARY，让 hax 找到 engine binary。

**决策性**：非决策点。

**建议**：无须改。

#6 (严重度: 低) — timeout_secs = 600 与 hax-coq 一致

**现象**：[`tools/hax-lean/tool.toml`](../../../../tools/hax-lean/tool.toml) L18：`timeout_secs = 600`。

[`tools/hax-fstar/tool.toml`](../../../../tools/hax-fstar/tool.toml) L24：`timeout_secs = 300`——差异。

**违反**：未违反——hax 不同 backend 提取速度差异，hax-lean / hax-coq 用 600s，hax-fstar 用 300s。README hax-fstar L57 说明 "F\* 提取比 Lean/Coq 快"。

**决策性**：非决策点。

**建议**：无须改。

#7 (严重度: 低) — harness 朴素 bin + comment 说明 hax `-C --lib ;` 跳过 bin

**现象**：[`tools/hax-lean/harness.rs.tera`](../../../../tools/hax-lean/harness.rs.tera)：

```rust
// hax-lean: harness is bin target; hax only extracts the lib (-C --lib ;).
fn main() {
    {{ target_crate_name }}::{{ entry_fn }}();
}
```

**违反**：未违反——hax `-C --lib` 限制只翻译 lib，harness（bin target）被跳过。

**决策性**：非决策点。

**建议**：无须改。

## §4 决策点 vs 非决策点

- 决策点：3（内联 sh -c vs 外部 wrapper / entry_fn 存在性 grep / grep 正则维护机制）
- 非决策点：4

## §5 审查结论

hax-lean 配置 oracle 设计精巧（awk strip 注释 + 精准 grep），符合 [`tool-integration.md`](../../../design/tool-integration.md) §四-4.2 双向实测验证。

最值得补强：

1. **内联 sh -c 抽外部 wrapper**：与 hax-coq / hax-fstar 一并重构，减少 tool.toml 复杂度；
2. **加 entry_fn 存在性 grep**：与 hax-coq / hax-fstar 对齐——防 hax 引入 lean backend silent skip-item path（如 fstar_backend.ml:1771 `Use _ | NotImplementedYet -> []` 在 lean backend 的等价路径）。

整体 oracle 设计扎实，但维护性可改进。
