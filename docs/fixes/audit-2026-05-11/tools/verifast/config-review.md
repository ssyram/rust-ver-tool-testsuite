# verifast config Review

## §1 问题意识

verifast 是矩阵中最特殊的工具——separation logic verifier，**不通过 cargo**，直接读 src/lib.rs。corpus 全 0 spec 注解的现状下，`-skip_specless_fns` 让所有用户函数被跳过——这是 silent skip 走 silent SUCCESS 的典型情景。2026-05-08 oracle audit-1 引入 wrapper 用 `-verbose 1` 检测 symex 是否触及 user file，把 vacuous pass 翻为 FAILED。

恶意角度考察：
1. wrapper 实施是否真按 README 描述工作？双向实测 oracle-validation/ 是否存在？
2. `-skip_specless_fns` + `-disable_overflow_check` + `-ignore_unwind_paths` 三件 flag 是否合规？
3. wrapper 与 README 形式严格性自陈一致？

## §2 审查方法

参照源：

- [`docs/design/principles.md`](../../../design/principles.md) §六-2 / §六-4；
- [`docs/design/tool-integration.md`](../../../design/tool-integration.md) §三 / §四；
- [`tools/verifast/tool.toml`](../../../../tools/verifast/tool.toml)；
- [`tools/verifast/verifast-strict-wrapper.sh`](../../../../tools/verifast/verifast-strict-wrapper.sh)；
- [`tools/verifast/oracle-validation/`](../../../../tools/verifast/oracle-validation/)；
- [`docs/fixes/oracle-leak-audit-2026-05-08.md`](../../oracle-leak-audit-2026-05-08.md) §3.1；
- verifast 26.01。

## §3 审查现象

#1 (严重度: 高) — wrapper `-verbose 1` 检测 user file 触及——精确反作弊机制

**现象**：[`tools/verifast/verifast-strict-wrapper.sh`](../../../../tools/verifast/verifast-strict-wrapper.sh) L86-129：

```bash
"$VERIFAST_BIN" \
    -verbose 1 \
    -target macOS \
    -shared \
    -skip_specless_fns \
    ...
user_lines=$(grep -cE 'src/lib\.rs\(' "$out_file" 2>/dev/null || true)
if [[ "$user_lines" -eq 0 ]]; then
    cat >&2 <<EOF
[verifast-oracle] FAIL: vacuous pass — symex executed 0 statements in src/lib.rs.
...
EOF
    exit 2
fi
```

**违反**：未违反——按 [`tool-integration.md`](../../../design/tool-integration.md) §四-4.2 双向实测验证 + [`principles.md`](../../../design/principles.md) §六-4 反作弊推论。`-verbose 1` 的 per-statement source-path 标签是 verifast 自身设计 signal，grep 抓 user file 行——可信。

**推理链**：wrapper L41-52 反误报论证：spec-bearing SUCCESS ⇒ ≥ 1 verbose line mentioning user file，所以 reject 条件 (0 mentions) 对 real SUCCESS 不可达——0 误报。

**决策性**：非决策点——优秀反作弊设计。

**建议**：无须改。

#2 (严重度: 中) — oracle-validation/ 独立目录——矩阵中**唯一**

**现象**：[`tools/verifast/`](../../../../tools/verifast/) 含 `oracle-validation/` 子目录（wrapper L54-62 引用）：

> Validation evidence (tools/verifast/oracle-validation/)
> Two micro-tests, each fed to verifast as src/lib.rs (matching the runner's invocation form):
>   spec_less_baseline.rs (no spec, single `pub fn add_one`):
>     EXIT 0, 0 verbose lines mentioning src/lib.rs   → wrapper rejects ✓
>   spec_bearing_add_one.rs (`fn foo` with `//@ req true; //@ ens ...`):
>     EXIT 0, 10 verbose lines mentioning src/lib.rs  → wrapper accepts ✓

**违反**：未违反——这是矩阵中**唯一**带 oracle-validation 独立目录的工具。按 [`tool-integration.md`](../../../design/tool-integration.md) §四-4.2 双向实测验证扎实落实。

**推理链**：优秀范例。其他工具如 hax-lean / rocq-of-rust 可参考——把双向实测 micro-test 落入独立目录便于重跑。

**决策性**：非决策点——优秀范例。

**建议**：无须改。可考虑——其他工具（hax-lean / rocq-of-rust）也建 oracle-validation 目录与 verifast 体例对齐。

#3 (严重度: 中) — `-target macOS` 硬编码——与 charon-mono / charon-poly 同问题

**现象**：[`tools/verifast/verifast-strict-wrapper.sh`](../../../../tools/verifast/verifast-strict-wrapper.sh) L88：`-target macOS`。

**违反**：[`tool-integration.md`](../../../design/tool-integration.md) §一——平台硬编码。Linux 用户无法直接跑。

**推理链**：README L80 注释"指定 LP64/arm64-apple-macosx 平台"。可考虑参数化为 `$TS_VERIFAST_TARGET`。

**决策性**：决策点。

**建议**：可选——参数化平台 flag。

#4 (严重度: 中) — `-skip_specless_fns` + corpus 全 0 spec 状态——2026-05-08 之前的"语义降级"误读

**现象**：[`tools/verifast/README.md`](../../../../tools/verifast/README.md) L60-64：

> ### vacuous-pass 历史口径修订（重要）
> 2026-05-08 之前 README 把"`-skip_specless_fns` 让 spec-less entry 退化为 vacuous pass"称为"语义降级，不是漏报"。审计...按项目宪法 §6 反作弊原则口径校正：**SUCCESS = 工具完整完成它的工作单元**——`-skip_specless_fns` 让 entry 完全没经过 verify 阶段，本质就是 silent skip，符合 partial 定义，应封堵。

**违反**：未违反——README 显式标"历史口径修订（重要）"，记录 oracle 演进路径。**优秀诚实声明**。

**推理链**：与 kmir L25-26 "历史问题"段同模式——诚实记录 oracle 修复路径。

**决策性**：非决策点——优秀范例。

**建议**：无须改。

#5 (严重度: 高) — wrapper exit 2 路径——runner 二值化处理

**现象**：[`tools/verifast/verifast-strict-wrapper.sh`](../../../../tools/verifast/verifast-strict-wrapper.sh) L128：`exit 2`。runner 二值化（非 0 = FAILED），exit 2 与 exit 1 同 FAILED。

**违反**：未违反——同 kani wrapper exit 2 模式。

**决策性**：非决策点。

**建议**：无须改。

#6 (严重度: 低) — tool.toml env 重导出 VERIFAST_BIN

**现象**：[`tools/verifast/tool.toml`](../../../../tools/verifast/tool.toml) L48-51：

```toml
command = [
  "env",
  "VERIFAST_BIN=${TS_VERIFAST_BIN}",
  "${TS_PROJECT_ROOT}/tools/verifast/verifast-strict-wrapper.sh"
]
```

注释 L11-14："runner strips all TS_* envvars before spawning children (runner/src/exec.rs:165), so we re-export TS_VERIFAST_BIN as the non-prefixed VERIFAST_BIN"。

**违反**：未违反——按 [`runner/src/exec.rs`](../../../../runner/src/exec.rs) L165 strip TS_*，env 重导出是规范做法。同 aeneas-* / prusti / rocq-of-rust-typecheck。

**决策性**：非决策点。

**建议**：无须改。

#7 (严重度: 低) — harness 是最小 placeholder

**现象**：[`tools/verifast/harness.rs.tera`](../../../../tools/verifast/harness.rs.tera)：

```rust
// VeriFast harness placeholder — intentionally minimal.
// VeriFast is invoked directly on src/lib.rs (not via cargo)...
fn main() {}
```

**违反**：未违反——verifast 只读 src/lib.rs，bin harness 被忽略。harness 仅满足框架契约。

**决策性**：非决策点。

**建议**：无须改。

#8 (严重度: 低) — version_command 用 `verifast -help`

**现象**：[`tools/verifast/tool.toml`](../../../../tools/verifast/tool.toml) L55-58。

**违反**：未违反——verifast 无 `--version`，用 `-help` 首行含版本字符串。

**决策性**：非决策点。

**建议**：可选——用 `${TS_VERIFAST_BIN} -version` 若 verifast 26.01 支持该 flag。

## §4 决策点 vs 非决策点

- 决策点：2（平台 flag 参数化 / oracle-validation 目录扩散到其他工具）
- 非决策点：6

## §5 审查结论

verifast 配置是矩阵中**反作弊设计的杰出范例**：

- wrapper `-verbose 1` 检测 user file 触及 + reverse-FP 分析扎实；
- **oracle-validation/ 独立目录是矩阵唯一**——双向 micro-test 落地；
- README L60-64 "历史口径修订（重要）" 诚实记录 oracle 演进。

无 critical 问题；整体属于"反作弊设计的杰出范例"，可作为其他工具参考。
