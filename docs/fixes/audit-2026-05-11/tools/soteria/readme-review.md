# soteria readme Review

## §1 问题意识

soteria README 重点：(1) Tree Borrows 原生支持的工具特性陈述；(2) exit code 0/1/2/3 四类 oracle 设计；(3) bug detect 也算 FAILED（按 §六-2 完整完成精神）；(4) HashMap aarch64 false-positive 等已知限制。

恶意角度考察：是否对 soteria 能力下绝对结论？

## §2 审查方法

参照源：

- [`docs/design/tool-integration.md`](../../../design/tool-integration.md) §五 / §六；
- [`tools/soteria/tool.toml`](../../../../tools/soteria/tool.toml)；
- [`deep-reports/cc-reports/soteria.md`](../../../../deep-reports/cc-reports/soteria.md)。

## §3 审查现象

#1 (严重度: 低) — README 8 章节齐全

**现象**：[`tools/soteria/README.md`](../../../../tools/soteria/README.md) 完整。

**违反**：未违反。

**决策性**：非决策点。

**建议**：无须改。

#2 (严重度: 低) — README L4 "第一个原生支持 Tree Borrows 的 Rust 符号执行引擎，对别名模型 bug 的检测比 Kani 更精确"——临近跨工具能力评判

**现象**：[`tools/soteria/README.md`](../../../../tools/soteria/README.md) L4：

> 第一个原生支持 Tree Borrows 的 Rust 符号执行引擎，对别名模型 bug 的检测比 Kani 更精确。

**违反**：[`tool-integration.md`](../../../design/tool-integration.md) §六禁忌——"跨工具能力排序（如'X 比 Y 更好'）"。"对别名模型 bug 的检测比 Kani 更精确" 是 elevator pitch 中的跨工具比较，触及禁忌线。

**推理链**：但 elevator pitch 是工具自陈（按上游 README 引用），不是测试结论。按 [`tool-integration.md`](../../../design/tool-integration.md) §五 (1) "简介 + GitHub 上游 URL + 工具的 elevator pitch"——elevator pitch 引用上游说法是合规的。

**决策性**：决策点——是否调整 elevator pitch 措辞。

**建议**：可选改"对别名模型 bug 的检测比 Kani 更精确" 为"针对别名模型 bug 提供 Tree Borrows 原生检测能力"——避免与 Kani 比较。当前 elevator pitch 措辞合规但临近禁忌。

#3 (严重度: 中) — README L25 "注：bug detect 在某些视角下可理解为'工具有效输出'"——按 §六-2 精神 FAILED

**现象**：[`tools/soteria/README.md`](../../../../tools/soteria/README.md) L25 / L42：

> 注：bug detect 在某些视角下可理解为"工具有效输出"，但按宪法精神（不允许 partial = 必须完整跑完）一律 FAILED。oracle（runner 仅按 exit code 判定）与宪法一致。
> ...
> 注：按工具自身语义 bug detect 是有效输出——但按"完整完成"精神，符号执行被 bug 中断 = 没完整跑完 → FAILED。

**违反**：未违反——明确诠释 §六-2 不允许 partial 精神。与 miri / verifast 同模式，多处重复诚实声明。

**推理链**：诚实声明优秀范例。

**决策性**：非决策点。

**建议**：无须改。

#4 (严重度: 低) — README L29 "**函数级对应性**：LLBC `.crate` 文件**逐字符给出函数路径 + 类型签名**——4 个翻译类工具中函数级映射最强"——临近跨工具能力评判

**现象**：[`tools/soteria/README.md`](../../../../tools/soteria/README.md) L29：

> **函数级对应性**：LLBC `.crate` 文件**逐字符给出函数路径 + 类型签名**——4 个翻译类工具中函数级映射最强

**违反**：[`tool-integration.md`](../../../design/tool-integration.md) §六禁忌——"跨工具能力排序"。"4 个翻译类工具中函数级映射最强" 直接评比。

**推理链**：这是与 charon / hax / aeneas / rocq-of-rust 的函数级映射强度对比——明确触及禁忌。但 LLBC 产物形态本身是 soteria 工具事实陈述（"逐字符给出函数路径 + 类型签名"），可保留；只删"最强" 评比即可。

**决策性**：决策点——需调整措辞。

**建议**：把 L29 改为"**函数级对应性**：LLBC `.crate` 文件含完整函数路径 + 类型签名——可作为 oracle 函数级对应性校验的基础"。

#5 (严重度: 中) — README L64 "**HashMap / collections 在 aarch64 产生 false-positive**" —— 实测 bug 描述

**现象**：[`tools/soteria/README.md`](../../../../tools/soteria/README.md) L64：

> **HashMap / collections 在 aarch64 产生 false-positive**：`std::collections::HashMap` 等在 aarch64 上使用 `stdarch` SIMD intrinsics，soteria-rust 将其报告为 dangling pointer 违规（即使代码完全正确）。受影响类目：`collections/hashmap` 等相关 entry，预期 FAILED。

**违反**：[`tool-integration.md`](../../../design/tool-integration.md) §六禁忌——"对工具能力下绝对结论"。"代码完全正确" 是绝对判断，"HashMap 是 false-positive" 是对 soteria 上游 bug 的明确归因。

**推理链**：但这是诚实声明上游 bug（aarch64 SIMD intrinsics 处理 bug）——按 [`tool-integration.md`](../../../design/tool-integration.md) §五 (7) 已知限制 / 坑允许描述工具自身已知 bug。"代码完全正确" 可改"按 Rust 语义完全合法"——更精确。

**决策性**：决策点——措辞调整。

**建议**：可选——把"代码完全正确" 改为"按 Rust 语义完全合法的 HashMap 操作"——避免绝对评判。

#6 (严重度: 低) — README L46-50 安装段 commit hash + OCaml 版本完整

**现象**：[`tools/soteria/README.md`](../../../../tools/soteria/README.md) L48：

> 本测试基线：soteria commit `3c21278187c60c99418fe2dabb03710ce4102896` + Obol commit `ddea5ca5da4c07301584f47f05ea8615fc365b41`（OCaml 5.4 + Z3，搭配 nightly-2026-02-07）。

**违反**：未违反——按 [`tool-integration.md`](../../../design/tool-integration.md) §一 commit hash 锁定。

**决策性**：非决策点。

**建议**：无须改。

#7 (严重度: 低) — README L72-74 关联 sub-tests "本工具未派生限制集"

**现象**：[`tools/soteria/README.md`](../../../../tools/soteria/README.md) L72-74。

**违反**：未违反——诚实声明。

**决策性**：非决策点。

**建议**：无须改。

## §4 决策点 vs 非决策点

- 决策点：3（L4 elevator pitch vs Kani / L29 "4 个翻译类工具中最强" / L64 "代码完全正确"）
- 非决策点：4

## §5 审查结论

soteria README 内容详尽，oracle 设计扎实。但**多处临近 §六禁忌跨工具能力排序的边界**：

1. **L4 elevator pitch "比 Kani 更精确"**：可调整为不引用 Kani；
2. **L29 "4 个翻译类工具中函数级映射最强"**：直接跨工具排序，需调整；
3. **L64 "代码完全正确"**：可改"按 Rust 语义完全合法"。

这是矩阵中**唯一显著触及多处跨工具能力评判**的 README。整体内容质量高，但措辞需调整以严格遵守 §六禁忌。
