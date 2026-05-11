# rocq-of-rust-typecheck config Review

## §1 问题意识

rocq-of-rust-typecheck 是矩阵中**唯一档 1（产物在 Rocq 里 typecheck 通过）工具**——`tools/rocq-of-rust` 档 0 的严格上层包裹。Pipeline：rocq-of-rust translate → .v → coqc -R <runtime> RocqOfRust → .vo。oracle = 6 道门（档 0 复用）+ 3 道门（gate 7-9：coqc exit 0 + .vo 产生 + stderr 无 Error）。

恶意角度考察：
1. wrapper 复用 rocq-of-rust 6 道门是否真等价？
2. 9 个 RocqOfRust 核心 runtime .vo 的 bootstrap 机制是否真幂等？
3. opam switch 隔离是否合规？
4. coqc 调用 cwd / 路径处理？

## §2 审查方法

参照源：

- [`docs/design/principles.md`](../../../design/principles.md) §六-1 / §六-2；
- [`docs/fixes/rocq-of-rust-typecheck-implementation-2026-05-11.md`](../../rocq-of-rust-typecheck-implementation-2026-05-11.md)；
- [`docs/fixes/audit-2026-05-11/tools/rocq-of-rust/config-review.md`](../rocq-of-rust/config-review.md) 档 0 shared issues；
- [`tools/rocq-of-rust-typecheck/tool.toml`](../../../../tools/rocq-of-rust-typecheck/tool.toml)；
- [`tools/rocq-of-rust-typecheck/rocq-of-rust-typecheck-wrapper.sh`](../../../../tools/rocq-of-rust-typecheck/rocq-of-rust-typecheck-wrapper.sh)；
- Rocq 9.0.0 + ror-test opam switch。

## §3 审查现象

#1 (严重度: 高) — Stage 2 coqc typecheck 增加 3 道门——档 1 严格上层

**现象**：[`tools/rocq-of-rust-typecheck/rocq-of-rust-typecheck-wrapper.sh`](../../../../tools/rocq-of-rust-typecheck/rocq-of-rust-typecheck-wrapper.sh) L8-17：

```
# Oracle (gate 1-6 are inherited from tools/rocq-of-rust, gates 7-9 are new):
#   1-6  identical to tools/rocq-of-rust (exit 0, .v present, > 200 bytes,
#        no failure marker, entry_fn appears as Definition)
#   7.   coqc exits 0 on the .v product
#   8.   .vo file written next to .v product
#   9.   coqc stderr contains no "Error" line
```

**违反**：未违反——按 [`docs/design/hax-lean-consistency-design-2026-05-11.md`](../../../design/hax-lean-consistency-design-2026-05-11.md) 档位定义（档 0 → 档 1 → 档 2 → 档 3）。**矩阵中唯一档 1 工具**。

**推理链**：档 1 是档 0 的严格上层（任一 entry 档 1 SUCCESS ⇒ 档 0 SUCCESS）。形式严格性更强——coqc 是确定性算法。

**决策性**：非决策点——优秀范例。

**建议**：无须改。

#2 (严重度: 中) — Wrapper 复用 stage 1 logic 但**不是真的 import** rocq-of-rust wrapper

**现象**：[`tools/rocq-of-rust-typecheck/rocq-of-rust-typecheck-wrapper.sh`](../../../../tools/rocq-of-rust-typecheck/rocq-of-rust-typecheck-wrapper.sh) L112-143 与 rocq-of-rust wrapper 部分逻辑重复（gate 2-6）。但是**仅 N=1**——没有 N=7 attempt 循环。

**违反**：[`tool-integration.md`](../../../design/tool-integration.md) §四-4.2 防漏报机制——rocq-of-rust 档 0 用 N=7 封堵非确定性翻译路径漏报；rocq-of-rust-typecheck wrapper 只做 1 次 translate——理论上若 entry 触发非确定性 silent skip，typecheck 也可能 silent skip + 漏报。

**推理链**：但实际上 stage 2 coqc 是确定性的——若 stage 1 silent skip entry_fn（变体 B），stage 2 仍 typecheck 通过（因为 .v 不含 entry_fn 也合法），gate 6 抓 → exit 1。**所以 gate 6 复用就够了**。但单次 attempt 在 P(drop)=0.4 entry 上 60% SUCCESS / 40% FAILED——不稳定。

按 ror-gate6-fix-2026-05-11.md 的 P15-impl 发现，rocq-of-rust-typecheck 应同样跑 N=7——否则该工具在 `thread_local!` entry 上 silent flake。

**决策性**：决策点——是否给 rocq-of-rust-typecheck wrapper 加 N=7 循环。

**建议**：可选——把 rocq-of-rust 的 N-attempt 机制移植到 typecheck wrapper，或共享同一 stage 1 logic。当前 typecheck wrapper L113-114 只 translate 1 次——与 rocq-of-rust 反作弊设计有 gap。

#3 (严重度: 中) — Runtime bootstrap 机制 - 9 core .vo 幂等 build

**现象**：[`tools/rocq-of-rust-typecheck/rocq-of-rust-typecheck-wrapper.sh`](../../../../tools/rocq-of-rust-typecheck/rocq-of-rust-typecheck-wrapper.sh) L54-101 详尽 bootstrap：

```bash
CORE_VO=(
    "M.vo"
    "RocqOfRust.vo"
    ...
)
needs_build=0
for vo in "${CORE_VO[@]}"; do
    if [[ ! -f "$ROR_RUNTIME_PATH/$vo" ]]; then
        needs_build=1
        break
    fi
done
if [[ $needs_build -eq 1 ]]; then
    ...按依赖顺序 coqc build...
fi
```

**违反**：未违反——按 [`tool-integration.md`](../../../design/tool-integration.md) §五 自动 bootstrap 机制。幂等（仅缺失时 build）。

**推理链**：合规——bootstrap 是 wrapper 内自动化的 self-contained 流程，避免用户手动 setup。

**决策性**：非决策点。

**建议**：无须改。

#4 (严重度: 中) — `opam env --switch=${ROR_TYPECHECK_SWITCH} --set-switch` 隔离 switch

**现象**：[`tools/rocq-of-rust-typecheck/rocq-of-rust-typecheck-wrapper.sh`](../../../../tools/rocq-of-rust-typecheck/rocq-of-rust-typecheck-wrapper.sh) L45-47：

```bash
if command -v opam >/dev/null 2>&1; then
    eval "$(opam env --switch="$ROR_TYPECHECK_SWITCH" --set-switch 2>/dev/null)" || true
fi
```

**违反**：与 aeneas wrapper 同 issue——`|| true` 容许 opam 不存在，但 coqc 找不到则后续 fail。L49-52 显式 check `coqc` on PATH。

**推理链**：合规——比 aeneas wrapper "硬编码 default switch" 更通用（用户可指定自己的 switch 名通过 TS_ROR_TYPECHECK_SWITCH）。

**决策性**：非决策点——优于 aeneas 实施。

**建议**：无须改。

#5 (严重度: 中) — coqc 调用 `cd "$PRODUCT_DIR"` 后 `coqc -R "$ROR_RUNTIME_PATH" RocqOfRust -impredicative-set "$PRODUCT_BASE"`

**现象**：[`tools/rocq-of-rust-typecheck/rocq-of-rust-typecheck-wrapper.sh`](../../../../tools/rocq-of-rust-typecheck/rocq-of-rust-typecheck-wrapper.sh) L160-166：

```bash
echo "[ror-typecheck-wrapper] coqc -R $ROR_RUNTIME_PATH RocqOfRust -impredicative-set $PRODUCT_V" >&2
...
( cd "$PRODUCT_DIR" && coqc -R "$ROR_RUNTIME_PATH" RocqOfRust -impredicative-set "$PRODUCT_BASE" ) \
    >&2 2>"$coqc_err"
```

**违反**：未违反——`$ROR_RUNTIME_PATH` 必须为绝对路径，wrapper 注释 L140-141 提示。subshell cd 隔离不影响主 wrapper cwd。

**推理链**：合规。subshell 避免影响后续 logic。

**决策性**：非决策点。

**建议**：无须改。

#6 (严重度: 低) — Gate 8 / Gate 9 belt-and-braces 防御

**现象**：[`tools/rocq-of-rust-typecheck/rocq-of-rust-typecheck-wrapper.sh`](../../../../tools/rocq-of-rust-typecheck/rocq-of-rust-typecheck-wrapper.sh) L176-188：

```bash
# Gate 8: .vo file produced
if [[ ! -f "$PRODUCT_DIR/$PRODUCT_STEM.vo" ]]; then
    echo "[ror-typecheck-oracle] FAIL: coqc exit 0 but no .vo at $PRODUCT_DIR/$PRODUCT_STEM.vo" >&2
    exit 1
fi
# Gate 9: stderr has no "Error" line (defence in depth — coqc usually
# exits non-zero on errors...
if grep -qE '^Error' "$coqc_err" 2>/dev/null; then
    ...
fi
```

**违反**：未违反——按 [`tool-integration.md`](../../../design/tool-integration.md) §四-4.2 belt-and-braces 防 coqc 假象 exit 0。

**决策性**：非决策点——优秀防御。

**建议**：无须改。

#7 (严重度: 低) — harness 与 rocq-of-rust 完全相同（`pub fn entry_fn() -> u32 { 42 }`）

**现象**：[`tools/rocq-of-rust-typecheck/harness.rs.tera`](../../../../tools/rocq-of-rust-typecheck/harness.rs.tera) 与 [`tools/rocq-of-rust/harness.rs.tera`](../../../../tools/rocq-of-rust/harness.rs.tera) 文字完全相同。

**违反**：未违反——harness 不被 cargo 编译（rocq-of-rust 直接读 src/lib.rs）。

**决策性**：非决策点。

**建议**：可选——同 rocq-of-rust review 建议，改 `fn main() {}`。

#8 (严重度: 低) — timeout 180s + bootstrap 自动幂等

**现象**：[`tools/rocq-of-rust-typecheck/tool.toml`](../../../../tools/rocq-of-rust-typecheck/tool.toml) L48 + 注释 L36-39：

> timeout: 180 s ≈ rocq-of-rust (typically < 5 s) + coqc on a single ror product (< 1 s after runtime pre-built; first-call bootstrap adds ~3 s).

**违反**：未违反——按 README 工程估算合理。

**决策性**：非决策点。

**建议**：无须改。

## §4 决策点 vs 非决策点

- 决策点：1（是否给 typecheck wrapper 加 N=7 attempt）
- 非决策点：7

## §5 审查结论

rocq-of-rust-typecheck 配置是**矩阵中唯一档 1（typecheck）工具**——pipeline 严格上层，gate 7-9 增加 coqc 验证。优秀实施：

- runtime bootstrap 幂等机制（9 core .vo 自动 build）；
- opam switch 隔离 + 用户可定制（TS_ROR_TYPECHECK_SWITCH）；
- gate 8 / 9 belt-and-braces 防 coqc silent path。

最值得补强：

1. **N=7 attempt**：当前 stage 1 仅 1 次 translate——理论上对 `thread_local!` 类 entry 仍会 silent flake。建议复用 rocq-of-rust 的 N-attempt 机制。

整体属于"档 1 严格上层包裹的优秀实施"，与 rocq-of-rust 一并构成档 0 → 档 1 双层 oracle。
