# kmir readme Review

## §1 问题意识

kmir README 重点：(1) K Framework 三层工具链（K Framework + Python kmir + stable-mir-json）的复杂安装；(2) K-stuck 反作弊关键 oracle 设计（旧 oracle 52/102 SUCCESS 假阳性的诚实历史）；(3) 与 miri 的对比（都是解释执行类）。

恶意角度考察：是否对 kmir 能力下绝对结论？K-stuck 论据是否扎实？

## §2 审查方法

参照源：

- [`docs/design/tool-integration.md`](../../../design/tool-integration.md) §五 / §六；
- [`tools/kmir/tool.toml`](../../../../tools/kmir/tool.toml)；
- [`deep-reports/cc-reports/kmir.md`](../../../../deep-reports/cc-reports/kmir.md)。

## §3 审查现象

#1 (严重度: 低) — README 8 章节齐全

**现象**：[`tools/kmir/README.md`](../../../../tools/kmir/README.md) 完整。

**违反**：未违反。

**决策性**：非决策点。

**建议**：无须改。

#2 (严重度: 中) — README L25 "历史问题"段坦诚——实测 52/102 假阳性是显著修复案例

**现象**：[`tools/kmir/README.md`](../../../../tools/kmir/README.md) L25-26：

> > **历史问题**：旧版 tool.toml 仅看 `kmir run` exit code，但 K interpreter 卡 stuck 时 CLI 仍 exit 0。实证扫到 102 个原始 SUCCESS 中 **52 个是 K-stuck 假阳性**（如 `charon-limit/inline-asm` 在 miri 上 FAILED unsupported asm，旧 oracle 在 kmir 上 SUCCESS——同一构造结论相反）。新 oracle 通过 grep `#EndProgram ~> .K` 终结模式翻转此类假阳性，真实"工具完成 K 语义"率约 32%（46/146）。

**违反**：未违反——按 [`principles.md`](../../../design/principles.md) §三-3 诚实测试范围、[`tool-integration.md`](../../../design/tool-integration.md) §四-4.2 双向实测。**这是矩阵中诚实历史声明的优秀范例**——明确说明旧 oracle 假阳性比例 + 修复后真实通过率。

**推理链**：诚实声明历史 bug + 锚定具体数字（52/102 / 32%）= 高诚实度。

**决策性**：非决策点——优秀范例。

**建议**：无须改。

#3 (严重度: 中) — README L29 "FAILED 来源（实测分布）"段——临近"工具能力评判"

**现象**：[`tools/kmir/README.md`](../../../../tools/kmir/README.md) L29：

> - **FAILED 来源（实测分布）**：当前 FAILED 集合主要来自上游 **cargo + stable-mir-json** 编译失败（`json.JSONDecodeError: Expecting value: line 1 column 1` 链）—— 触发于含 Arc / BTreeMap / HashMap / 线程 / Mutex / 第三方 crate（rsa / sha2 / x509-parser）的样例。这与"K rule 缺失"的旧 README 描述**不同**：实际是上游 build 阶段就退出，K 语义未介入

**违反**：[`tool-integration.md`](../../../design/tool-integration.md) §六禁忌——"对工具能力下绝对结论"。L29 说"FAILED 主要来自 stable-mir-json 编译失败"是实测归因——锚定具体 stable-mir-json commit + corpus，合规。"这与'K rule 缺失'的旧 README 描述**不同**" 是 README 自我修正——优秀诚实。

**推理链**：实测归因 + 自我修正 = 合规。

**决策性**：非决策点。

**建议**：无须改。

#4 (严重度: 低) — README L34-41 形式严格性 ✅ 形式可证

**现象**：[`tools/kmir/README.md`](../../../../tools/kmir/README.md) L34-41：

> - **形式严格性 — 0 误报**：✅ 形式可证。`#EndProgram ~> .K` 是 K Framework 解释器的稳定终止 signature——K cell 化简到此 ⇔ 解释执行完整完成
> - **形式严格性 — 0 漏报**：✅ 形式可证。K-stuck（K cell 卡在 unsupported terminator）grep 已封死 silent path；cargo + stable-mir-json 编译失败也直接 exit ≠ 0
> - **漏报盲点**：无

**违反**：与 hax-coq / hax-lean 措辞对比——kmir 用 ✅，hax-lean 用 ⚠️。kmir K-stuck grep 是基于 K Framework 终止 signature 的论证，比纯 grep heuristics 强（K 语义稳定 signature）。

**推理链**：合规——K Framework 的 `#EndProgram` 是稳定终止公约（K 语义层定义），grep 不是启发式而是查 K 语义的稳定标记。✅ 可接受。

**决策性**：非决策点。

**建议**：无须改。

#5 (严重度: 低) — README L43-49 安装段三层版本锁定清晰

**现象**：[`tools/kmir/README.md`](../../../../tools/kmir/README.md) L46-47：

> 本测试基线：mir-semantics commit `84bea09` + stable-mir-json commit `62a239d7`，配 K Framework v7.1.282。

**违反**：未违反——按 [`tool-integration.md`](../../../design/tool-integration.md) §一 commit hash 锁定，三层工具链都 pin。

**决策性**：非决策点。

**建议**：无须改。

#6 (严重度: 低) — README L62-67 已知限制 / 坑 详细

**现象**：[`tools/kmir/README.md`](../../../../tools/kmir/README.md) L62-67 列出 5 条 kmir 特有的安装坑（llvm-kompile-clang patch / stable-mir-json 版本 lock / K state vs stdout / per-program kompile 开销 / nightly-2024-11-29 专用）。

**违反**：未违反——按 [`tool-integration.md`](../../../design/tool-integration.md) §五 (7) 已知限制。

**决策性**：非决策点。

**建议**：无须改。

#7 (严重度: 低) — 关联 sub-tests "本工具未派生限制集 agent，无 `examples/kmir-limit/`"

**现象**：[`tools/kmir/README.md`](../../../../tools/kmir/README.md) L69-73。

**违反**：未违反——诚实声明。

**决策性**：非决策点。

**建议**：无须改。

## §4 决策点 vs 非决策点

- 决策点：0
- 非决策点：7

## §5 审查结论

kmir README 内容详尽，**特别是 L25-26 "历史问题" 段是矩阵中诚实历史声明的优秀范例**——明示 52/102 假阳性 + 修复后通过率 + 与 miri 对比（同一构造结论相反的诊断信号）。

无 critical 问题；整体属于"经历显著 oracle 修复后稳态"的高诚实度配置。
