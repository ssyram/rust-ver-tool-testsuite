# prusti readme Review

## §1 问题意识

prusti README 重点：(1) MIR → Viper encoding 与 Silicon SMT 求解切线；(2) 三个 PRUSTI_* env 切线设计；(3) "与旧 PRUSTI_NO_VERIFY=true 配置的对比"段诚实记录 oracle 演进；(4) macOS arm64 Rosetta x86_64 锚定。

恶意角度考察：是否对 prusti 能力下绝对结论？历史口径修订诚实？

## §2 审查方法

参照源：

- [`docs/design/tool-integration.md`](../../../design/tool-integration.md) §五 / §六；
- [`tools/prusti/tool.toml`](../../../../tools/prusti/tool.toml)；
- [`tools/prusti/prusti-strict-wrapper.sh`](../../../../tools/prusti/prusti-strict-wrapper.sh)；
- [`deep-reports/cc-reports/prusti.md`](../../../../deep-reports/cc-reports/prusti.md)。

## §3 审查现象

#1 (严重度: 低) — README 8 章节齐全（但顺序与 §五 略不同）

**现象**：[`tools/prusti/README.md`](../../../../tools/prusti/README.md) 含简介、安装、本测试集前端接受、框架配置（前端-only 模式）、检测条件、形式严格性、与旧配置对比、已知限制、关联 sub-tests——但 "安装" 段在第 2 位（L12-19），而 [`tool-integration.md`](../../../design/tool-integration.md) §五 期望"安装" 在第 5 位。

**违反**：[`tool-integration.md`](../../../design/tool-integration.md) §五 章节顺序——但内容齐全，顺序差异。

**推理链**：实际上 §五 章节顺序是建议性的，"安装" 提前到第 2 位以便用户先安装—— UX 上合理，但不严格按推荐顺序。

**决策性**：非决策点——内容齐全，顺序略偏。

**建议**：可选——调整为 §五 推荐顺序。

#2 (严重度: 中) — README L70-81 "与旧 `PRUSTI_NO_VERIFY=true` 配置的对比" 表格——诚实历史声明

**现象**：[`tools/prusti/README.md`](../../../../tools/prusti/README.md) L70-81 详尽对比表格：

| 维度 | 旧（NO_VERIFY=true） | 新（DUMP+PRINT_HASH） |
| ... | ... | ... |
| "前端边界" 信号 | 全部 exit 0（rustc + macro 展开通过即视为接受，**未测 encoder**） | exit 0 仅当 encoder 真接受 |
| 旧配置定性 | 退化为 "rustc + prusti-contracts proc-macro pass"，与 `cargo-check` 重合 | 真实测到 MIR → Viper 这条 Prusti 独有的前端边界 |

**违反**：未违反——按 [`principles.md`](../../../design/principles.md) §三-3 / [`tool-integration.md`](../../../design/tool-integration.md) §四 诚实声明 oracle 演进。**这是矩阵中最详尽的 oracle 历史对比表**。

**推理链**：与 kmir L25 / verifast L60-64 同模式——诚实记录 oracle 修订。**优秀范例**。

**决策性**：非决策点——优秀范例。

**建议**：无须改。

#3 (严重度: 中) — README L82-85 通过率定性变化描述

**现象**：[`tools/prusti/README.md`](../../../../tools/prusti/README.md) L82-85：

> 预期通过率定性变化：旧配置下 Prusti 与 cargo-check 在通用 Rust feature 样例上几乎无差异，无法暴露 Prusti 的前端能力边界；新配置下 `prusti-limit/` 里的 entry 应稳定 FAILED，其它 entry 中触及 Prusti 不支持特性（async、复杂闭包、原始指针解引用、生命周期高阶用法等）的也会被识别出来。

**违反**：[`tool-integration.md`](../../../design/tool-integration.md) §六禁忌——"对工具能力下绝对结论（如'工具 X 不支持特性 Y'）"。"触及 Prusti 不支持特性（async、复杂闭包、原始指针解引用、生命周期高阶用法等）" 是绝对结论表述。

**推理链**：按 §六禁忌精神，应改为"在本测试方法学下，触发 Prusti encoder reject 的特性如 async / 复杂闭包 / 原始指针解引用 / 生命周期高阶用法等"。但该段是 "与旧配置对比" 的预期通过率描述，是方法学讨论而非直接对工具下结论——合规但临近禁忌。

**决策性**：决策点——措辞调整。

**建议**：可选——把"触及 Prusti 不支持特性"改为"触发 Prusti encoder reject 的特性"。

#4 (严重度: 低) — README L64-68 形式严格性论证扎实

**现象**：[`tools/prusti/README.md`](../../../../tools/prusti/README.md) L64-68：

> - **0 误报**：✅ 形式可证。cargo-prusti exit 0 ⇔ encoder 完整跑过且无 unsupported feature 报告...
> - **0 漏报**：✅ 形式可证 + wrapper 防御。Prusti 任何 unsupported feature → `[Prusti: ...]` marker + exit ≠ 0...
> - **漏报盲点**：无（NEW config + wrapper .vpr 检查双重保险下，encoder silent fast-path 已被 wrapper 闭环；剩余风险仅限于 encoder 内部 silent skip 单个 fn item 但仍写出非空 .vpr 的极端情形——理论窗口，实测 0 现象）

**违反**：未违反——✅ + wrapper 防御 + 理论窗口诚实声明。优秀范例。

**决策性**：非决策点。

**建议**：无须改。

#5 (严重度: 低) — README L88-97 已知限制 / 坑 macOS-specific 详细

**现象**：[`tools/prusti/README.md`](../../../../tools/prusti/README.md) L88-97 已知限制详细列出 5 条 macOS arm64 + Prusti closure ICE 等。

**违反**：未违反。

**决策性**：非决策点。

**建议**：无须改。

#6 (严重度: 低) — README L99-103 关联 sub-tests 描述准确

**现象**：[`tools/prusti/README.md`](../../../../tools/prusti/README.md) L99-103：

> `examples/prusti-limit/` 是本工具自声明的前端限制集——这些 entry 触发 Prusti encoder 真正拒绝的特性（closures、raw pointer deref、loan-crosses-loop 等），预期在新配置下 FAILED；旧 `NO_VERIFY=true` 配置下它们大多 PASS（因为 rustc 本身能编过），那是 false positive。

**违反**：未违反——"那是 false positive" 是 oracle 演进的诚实归因。

**决策性**：非决策点。

**建议**：无须改。

## §4 决策点 vs 非决策点

- 决策点：1（L82-85 措辞）
- 非决策点：5

## §5 审查结论

prusti README **优秀**：

- L70-81 "与旧配置对比" 表格是矩阵中最详尽的 oracle 历史对比——优秀范例；
- 形式严格性 ✅ + wrapper 防御 + 理论窗口诚实声明（L66-68）；
- 三个 PRUSTI_* env 切线设计文档化扎实。

最值得调整：L82-85 措辞"触及 Prusti 不支持特性"可改"触发 Prusti encoder reject 的特性"——更精确。

整体属于"高诚实度、高论证质量"的范例，与 verifast / kmir 同级。
