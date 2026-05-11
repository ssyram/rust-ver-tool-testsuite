# verus readme Review

## §1 问题意识

verus README 重点：(1) 前端 / 后端切分（VIR vs AIR / Z3）；(2) 反作弊章节（"`mod __ts_inner` 必须放进 `verus! {}` 块内"）—— 是矩阵中唯一显式标"反作弊"章节的工具 README；(3) cargo-verus 不可用的诚实声明 + workaround；(4) 33% SUCCESS 率描述。

恶意角度考察：是否对 verus 能力下绝对结论？反作弊章节是否充分？形式严格性自陈是否扎实？

## §2 审查方法

参照源：

- [`docs/design/principles.md`](../../../design/principles.md) §六-4 反作弊推论；
- [`docs/design/tool-integration.md`](../../../design/tool-integration.md) §五 / §六；
- [`tools/verus/tool.toml`](../../../../tools/verus/tool.toml)；
- [`tools/verus/harness.rs.tera`](../../../../tools/verus/harness.rs.tera)；
- [`deep-reports/cc-reports/verus.md`](../../../../deep-reports/cc-reports/verus.md)。

## §3 审查现象

#1 (严重度: 中) — README 含"反作弊"专章——矩阵唯一

**现象**：[`tools/verus/README.md`](../../../../tools/verus/README.md) L53-76 "反作弊：`mod __ts_inner` 必须放进 `verus! {}` 块内" 章节详尽阐述宪法 §六-4 反作弊推论在 verus 上的落地。

**违反**：未违反——按 [`tool-integration.md`](../../../design/tool-integration.md) §五 (6) "本框架配置" 增补反作弊章节是合规且推荐的——verus 是矩阵中**唯一**显式带"反作弊"章节的工具 README。这是优秀范例。

**推理链**：tool-integration §五 (1-8) 章节清单允许在 (6) 本框架配置下扩展反作弊讨论。verus 把它作为一节单列——更清晰。

**决策性**：非决策点——优秀范例。

**建议**：无须改。

#2 (严重度: 低) — README L74 "verus 33% 的 SUCCESS 率才是 Verus 真实的接受 Rust 子集边界"——临近能力评判

**现象**：[`tools/verus/README.md`](../../../../tools/verus/README.md) L74：

> 所以矩阵里 verus 33% 的 SUCCESS 率才是 Verus 真实的"接受 Rust 子集"边界——它确实拒掉很多块外才合法的构造（闭包捕获 `&mut`、`Option::copied` 无 spec、`transmute` 无 spec 等）。

**违反**：[`tool-integration.md`](../../../design/tool-integration.md) §六禁忌——"对工具能力下绝对结论"。措辞"真实的接受 Rust 子集边界"带评判语气。但实际是反作弊论证的延续："如果作弊（mod 放块外），SUCCESS 会齐刷刷高分"——33% 是反作弊后的真实测量。

**推理链**：这条措辞是反作弊论证的实证——锚定 corpus 与 verus 0.2026.05.03，是合规的实测陈述。但 "真实的接受 Rust 子集边界" 应改为 "在本 corpus + verus 0.2026.05.03 + 本反作弊配置下的实测 SUCCESS 率"。

**决策性**：决策点——是否调整措辞。

**建议**：可选改"verus 真实的接受 Rust 子集边界"为"verus 在本 corpus + 本反作弊配置下的实测 SUCCESS 边界"。

#3 (严重度: 低) — README L30-32 形式严格性"形式可证"扎实

**现象**：[`tools/verus/README.md`](../../../../tools/verus/README.md) L30-32：

> - **partial 暴露机制**：Verus 任何 rejection（lifetime / type check / `assume_specification` 缺失 / `verus_builtin not imported` 等）→ exit ≠ 0
> - **形式严格性 — 0 误报**：✅ 形式可证。verus exit 0 ⇔ VIR 构造完成无错误
> - **形式严格性 — 0 漏报**：✅ 形式可证。Verus 任何 rejection 都通过 `dcx().emit` 触发 exit ≠ 0

**违反**：[`tool-integration.md`](../../../design/tool-integration.md) §三/§四——但 "形式可证" 应有源码层证据（如 aeneas 给的 Main.ml:773）。verus README 没明示源码行号——但提到 `dcx().emit` 是 rustc 错误通路，合规。

**推理链**：与 aeneas / charon 同模式——单一通路（rustc error → exit ≠ 0）。但 verus 没给具体行号。可接受。

**决策性**：非决策点。

**建议**：可选——补 verus 源码层引用（如 `rust_verify/src/lifetime_emit.rs` 中 `dcx().emit` 行号）。

#4 (严重度: 低) — README L77-82 已知限制 / 坑 描述准确

**现象**：[`tools/verus/README.md`](../../../../tools/verus/README.md) L78-82：

> - **cargo-verus 不可用**：`cargo-verus` 作为 `RUSTC_WRAPPER` 尝试从 crates.io 重编译 vstd，但 `0.2026.05.03` 发布版 binary 与 crates.io 任何版本的 vstd 均不兼容...
> - **工具链版本锁定**：verus binary 需要精确的 rustup 工具链版本（此版本为 `1.95.0-aarch64-apple-darwin`）...

**违反**：未违反——锚定版本的诚实坑陈述。

**决策性**：非决策点。

**建议**：无须改。

#5 (严重度: 低) — README L82 "binary 路径"提到 `/tmp/ts-tools-install/...`（临时目录）

**现象**：[`tools/verus/README.md`](../../../../tools/verus/README.md) L82：

> **binary 路径**：当前 `tool.toml` 指向 `/tmp/ts-tools-install/...`（临时目录）。机器重启后需重新下载或移至持久路径（如 `~/.verus/`）。

**违反**：未违反——`.env` 配置 path 灵活，README 提醒持久化。但 tool.toml 本身用 `${TS_VERUS_BIN}` 不写 `/tmp/...` 字面——README L82 这条陈述应澄清"tool.toml 通过 `.env` 指向"。

**推理链**：tool.toml 读 [`tools/verus/tool.toml`](../../../../tools/verus/tool.toml) L2 是 `"${TS_VERUS_BIN}"` —— 实际路径由 `.env` 决定。L82 提到 `/tmp/ts-tools-install/...` 只是当前 .env 配置——不是 tool.toml 字面。

**决策性**：非决策点——README 措辞稍模糊但内容正确。

**建议**：可选改"当前 .env 配置 `TS_VERUS_BIN` 指向 `/tmp/ts-tools-install/...`（临时目录）"。

#6 (严重度: 低) — 关联 sub-tests 段说"本工具未派生限制集 agent"

**现象**：[`tools/verus/README.md`](../../../../tools/verus/README.md) L85-89：

> 本工具未派生限制集 agent，无 `examples/verus-limit/`。
> plain Rust 样例预期 SUCCESS（`0 verified, 0 errors`，exit 0）。类型错误样例预期 FAILED（exit 1）。

**违反**：未违反——但 verus 拒收很多 std API（README L23），实际 plain Rust 样例预期会大量 FAILED。L88 措辞"plain Rust 样例预期 SUCCESS" 与 L23 "未加 `assume_specification` 的标准库 API ... 整体被拒"矛盾。

**推理链**：plain Rust 不含 std API 调用时 SUCCESS；含 String / Box / Iterator 等会 FAILED。L88 措辞过乐观。

**决策性**：决策点——README 内部矛盾 L23 vs L88。

**建议**：把 L88 改为 "plain Rust 样例预期在不依赖标准库 API 时 SUCCESS；调用 std 类型方法时 Verus 因缺 `assume_specification` 而 FAILED——见 L23"。

## §4 决策点 vs 非决策点

- 决策点：2（L74 措辞 / L88 与 L23 内部矛盾）
- 非决策点：4

## §5 审查结论

verus README **优秀**：

- 反作弊章节是矩阵唯一显式标注（L53-76）——优秀范例；
- 形式严格性形式可证论证扎实；
- cargo-verus 不可用的诚实声明 + workaround 完整。

**关键问题**：

1. **L88 与 L23 内部矛盾**：L88 "plain Rust 样例预期 SUCCESS" 与 L23 "Verus 拒收无 spec 的标准库 API" 不一致——需修；
2. **L74 措辞**：可调整"verus 真实的接受 Rust 子集边界" 为更中性表述。

整体属于"反作弊设计的优秀范例"，cc-report 与 README 高度对齐。
