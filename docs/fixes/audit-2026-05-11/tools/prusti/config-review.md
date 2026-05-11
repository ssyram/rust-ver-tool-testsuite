# prusti config Review

## §1 问题意识

prusti 是矩阵中前端切割最精细的工具——通过三个 env 实现"encoder 跑 / Silicon 不跑"的精确切线（PRUSTI_NO_VERIFY=false + PRUSTI_DUMP_VIPER_PROGRAM=true + PRUSTI_PRINT_HASH=true）。wrapper 在 cargo-prusti exit 0 后追加 `.vpr` 产物存在性 check（2026-05-08 oracle audit §3.6）。

恶意角度考察：
1. 三个 env 切线是否真按 README 描述工作？
2. wrapper 与 README 一致？
3. macOS arm64 `arch -x86_64` 强制 Rosetta 运行的 trap；
4. timeout_secs = 900 是否合理？

## §2 审查方法

参照源：

- [`docs/design/principles.md`](../../../design/principles.md) §六-1 / §六-2；
- [`docs/design/tool-integration.md`](../../../design/tool-integration.md) §三 / §四；
- [`tools/prusti/tool.toml`](../../../../tools/prusti/tool.toml)；
- [`tools/prusti/prusti-strict-wrapper.sh`](../../../../tools/prusti/prusti-strict-wrapper.sh)；
- [`docs/fixes/oracle-leak-audit-2026-05-08.md`](../../oracle-leak-audit-2026-05-08.md) §3.6；
- prusti v0.2.2 commit `a0681ee`。

## §3 审查现象

#1 (严重度: 高) — 三 env 切线设计精巧——按宪法 §六-1 前端 / 后端切分

**现象**：[`tools/prusti/tool.toml`](../../../../tools/prusti/tool.toml) L18-31 注释 + 实施：

```
# Mechanism (verified against prusti-utils/src/config.rs and
# prusti-server/src/process_verification.rs at commit a0681ee):
#   - PRUSTI_NO_VERIFY=false    -> verify(env, def_spec) is invoked, so
#                                  Encoder::process_encoding_queue runs over
#                                  every fn item collected by
#                                  CollectPrustiSpecVisitor (i.e. all fns,
#                                  no #[ensures] / use prusti_contracts needed)
#   - PRUSTI_DUMP_VIPER_PROGRAM -> after encoding, the resulting Viper program
#                                  is written to <log_dir>/viper_program/*.vpr
#   - PRUSTI_PRINT_HASH         -> process_verification_request returns
#                                  Success right after the dump and BEFORE
#                                  new_viper_verifier(), so no Silicon/Z3
#                                  process is ever spawned.
```

**违反**：未违反——按 [`principles.md`](../../../design/principles.md) §六-1 + [`tool-integration.md`](../../../design/tool-integration.md) §三-源码层穷尽。锚定 commit `a0681ee` 的 prusti-server/src/process_verification.rs 源码验证。**是矩阵中前端切割最精细的设计**。

**推理链**：与 kani `--only-codegen` 同性质——精确停在工具内部 pipeline 阶段。prusti 用 3 个 env 实现"encoder 跑完 + 写 .vpr + 短路 return Success"——比 kani 单 flag 复杂但同样精确。

**决策性**：非决策点——优秀设计。

**建议**：无须改。

#2 (严重度: 中) — wrapper 在 cargo-prusti exit 0 后追加 `.vpr` 存在性 check

**现象**：[`tools/prusti/prusti-strict-wrapper.sh`](../../../../tools/prusti/prusti-strict-wrapper.sh) L67-87：

```bash
log_dir="target/verify/log/viper_program"
vpr_count=$(find "$log_dir" -name '*.vpr' 2>/dev/null | wc -l | tr -d ' ')

if [[ "$vpr_count" -eq 0 ]]; then
    cat >&2 <<EOF
[prusti-oracle] FAIL: cargo-prusti exited 0 but produced 0 .vpr files in
...
EOF
    exit 1
fi
```

**违反**：未违反——按 [`tool-integration.md`](../../../design/tool-integration.md) §四-4.2 防漏报机制。封堵 README L62 documented oracle 与之前 tool.toml 仅看 exit code 的 gap（audit §3.6）。

**推理链**：reverse-FP 论证 wrapper L33-46 扎实：在 NEW config 下，任何 exit 0 都必经过 dump 站点产生 ≥ 1 .vpr——所以"exit 0 + 0 .vpr" 状态不存在于合法 SUCCESS。

**决策性**：非决策点——优秀范例。

**建议**：无须改。

#3 (严重度: 中) — `arch -x86_64` 在 wrapper 内 hardcode—— macOS arm64 only

**现象**：[`tools/prusti/prusti-strict-wrapper.sh`](../../../../tools/prusti/prusti-strict-wrapper.sh) L58：

```bash
arch -x86_64 "$CARGO_PRUSTI"
```

**违反**：[`tool-integration.md`](../../../design/tool-integration.md) §一 macOS arm64 only。Linux 用户无 `arch` 命令。README L89 提到"macOS arm64 上必须 `arch -x86_64` 整体以 Rosetta 跑"——诚实声明。

**推理链**：合规但平台限定。

**决策性**：非决策点——README 已诚实声明。

**建议**：无须改——可考虑 `if [[ "$(uname)" == "Darwin" && "$(uname -m)" == "arm64" ]]; then arch -x86_64 ...; else "$CARGO_PRUSTI"; fi` 兼容 Linux。

#4 (严重度: 中) — tool.toml command 是 `["env", "K1=V1", ..., "wrapper.sh"]` 长 argv——可读性低

**现象**：[`tools/prusti/tool.toml`](../../../../tools/prusti/tool.toml) L1-41，command 含 12 个 K=V + 1 wrapper 路径。

**违反**：未违反——`env` 是 POSIX 标准接受 KEY=VAL 前缀。但 12 个 K=V 在 argv 中可读性低。

**推理链**：替代方案：在 wrapper 内 export env。但当前实施让 tool.toml 显式列出所有 env—— audit-able。trade-off。

**决策性**：非决策点。

**建议**：无须改。

#5 (严重度: 低) — timeout_secs = 900 对应 prusti JVM bootstrap + encoder 时长

**现象**：[`tools/prusti/tool.toml`](../../../../tools/prusti/tool.toml) L42：`timeout_secs = 900`。

**违反**：未违反——按 README "All 56 prusti SUCCESS run ≥ 13 s wall, dominated by JVM bootstrap + encoder"。900s 足够。

**决策性**：非决策点。

**建议**：无须改。

#6 (严重度: 低) — `CARGO_PRUSTI` 重导出（与 verifast / aeneas 同模式）

**现象**：[`tools/prusti/tool.toml`](../../../../tools/prusti/tool.toml) L34-37：

```
# Re-export TS_CARGO_PRUSTI as CARGO_PRUSTI for the wrapper. The runner
# strips all TS_* envvars before spawning children (runner/src/exec.rs:165).
"CARGO_PRUSTI=${TS_CARGO_PRUSTI}",
```

**违反**：未违反。

**决策性**：非决策点。

**建议**：无须改。

#7 (严重度: 中) — harness 是 bin 形态——但 prusti 默认对所有 fn item collect

**现象**：[`tools/prusti/harness.rs.tera`](../../../../tools/prusti/harness.rs.tera)：

```rust
fn main() {
    {{ target_crate_name }}::{{ entry_fn }}();
}
```

**违反**：未违反——按 [`tools/prusti/README.md`](../../../../tools/prusti/README.md) L51 `entry_mode = "bin"`。tool.toml 注释 L23-26 说 `CollectPrustiSpecVisitor` 默认收集所有 fn item（无需 entry 加 `use prusti_contracts::*`）—— harness bin 形态足够。

**推理链**：合规。

**决策性**：非决策点。

**建议**：无须改。

## §4 决策点 vs 非决策点

- 决策点：0
- 非决策点：7

## §5 审查结论

prusti 配置是**矩阵中前端切割最精细的设计**——3 个 PRUSTI_* env 实现"encoder 跑 / Silicon 不跑"的精确切线，wrapper 追加 `.vpr` 产物存在性 check。论据扎实（锚定 prusti commit `a0681ee` 的 prusti-utils/src/config.rs + prusti-server/src/process_verification.rs 源码）。

无 critical 问题；整体属于"oracle 设计的杰出范例"，与 verifast / verus 同级。
