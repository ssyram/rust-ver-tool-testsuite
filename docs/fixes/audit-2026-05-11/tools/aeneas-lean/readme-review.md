# aeneas-lean readme Review

## §1 问题意识

aeneas-lean README 是 4 个 aeneas backend README 的"模板"——它最完整（其他 3 个 README 大量复用其结构）。审查重点：(1) 是否准确传达 aeneas "exit 0 = 完整翻译"的形式证明级 oracle 设计；(2) `Generated the partial file` 路径的正确诠释（partial 仍 FAILED）；(3) 与 charon stage 的合作（charon 失败 → aeneas 不启动）；(4) Lean backend specific 内容（`Primitives.lean` 100 个 Axiom 与翻译质量无关的澄清）。

恶意角度考察：
1. 是否对工具能力下绝对结论？
2. 与 cc-report 是否一致？
3. 形式严格性 "形式可证" 是否扎实？

## §2 审查方法

参照源：

- [`docs/design/tool-integration.md`](../../../design/tool-integration.md) §五 必含章节、§六 禁忌、§三 / §四 形式严格性；
- [`tools/aeneas-lean/aeneas-lean-wrapper.sh`](../../../../tools/aeneas-lean/aeneas-lean-wrapper.sh) 实施对照；
- [`deep-reports/cc-reports/aeneas-lean.md`](../../../../deep-reports/cc-reports/aeneas-lean.md)；
- 其他 aeneas backend README 对照体例一致性。

## §3 审查现象

#1 (严重度: 低) — README L26-32 多种判定语义解释清晰

**现象**：[`tools/aeneas-lean/README.md`](../../../../tools/aeneas-lean/README.md) L26-32：

```
- **exit 0** = aeneas 全程跑通，产物 `lean-out/<Mod>.lean` 写出且完整 → SUCCESS
- **exit 1 + `Generated the partial file (because of N errors)` 路径** = ... → FAILED（按宪法 §六-2 不允许 partial）
- **exit 2** = OCaml panic / 内部异常，无产物 → FAILED
- **charon stage 失败** = `<crate>.llbc` 没产出，aeneas stage 不启动 → FAILED
```

**违反**：未违反——精确实测语义清晰列出。

**推理链**：判定层级 4 类 + oracle 行为 1 类（`exit 0` ⇔ SUCCESS）—— 工具自陈"我没全干完"必须被尊重。这是 [`principles.md`](../../../design/principles.md) §六-2 精神的清晰落地。

**决策性**：非决策点。

**建议**：无须改。

#2 (严重度: 中) — README L34"成功 entry 产物里**零 sorry / Admitted**" —— 对 Lean backend 是断言性的，但 oracle 不查产物 sorry

**现象**：[`tools/aeneas-lean/README.md`](../../../../tools/aeneas-lean/README.md) L34：

> 成功 entry 产物里**零 sorry / Admitted**——aeneas 的 backward function 模型（`fn f(x: &mut T) -> U` ⟼ `f : T → U × T`）是纯函数化，**不引入证明义务**。`Primitives.lean` 含约 100 条 `Axiom`...

**违反**：[`tool-integration.md`](../../../design/tool-integration.md) §二 "SUCCESS 信号的形式指标声明"——这条 README 断言"成功 entry 零 sorry / Admitted"，但 wrapper 不查 sorry。所以这是基于 aeneas 设计的**论据**而非 oracle 实施。

**推理链**：与 hax-lean 形成对比——hax-lean 走 grep sorry 抓 silent partial；aeneas-lean 不需要因为 aeneas 设计上不存在 silent sorry path。README 这条声明是 aeneas 的设计保证，非 oracle 实施。措辞需更清楚区分。

**决策性**：非决策点。

**建议**：可选——把 L34 改为："aeneas 设计上不存在 sorry / Admitted silent path——`Errors.error_list` 单一通路 + 完成态产物零证明义务（aeneas 用 backward function 模型把 `&mut` 转纯函数）。因此 oracle 不需 grep sorry。"

#3 (严重度: 低) — README L36-46 "SUCCESS 信号 / partial 暴露机制 / 形式严格性" 结构标准

**现象**：[`tools/aeneas-lean/README.md`](../../../../tools/aeneas-lean/README.md) L36-46 完整按 [`tool-integration.md`](../../../design/tool-integration.md) §二 / §三 / §四 体例。

**违反**：未违反。

**决策性**：非决策点。

**建议**：无须改——好范例。

#4 (严重度: 低) — L50-54 安装段 commit hash + charon-pin 信息清晰

**现象**：[`tools/aeneas-lean/README.md`](../../../../tools/aeneas-lean/README.md) L50-54：

> 本测试基线：aeneas commit `a14083a6` + 自家 charon v0.1.184（commit `ed22146b`，由 `charon-pin` 锁定）。

**违反**：未违反——按 [`tool-integration.md`](../../../design/tool-integration.md) §一 锁定 commit hash 要求。

**决策性**：非决策点。

**建议**：无须改。

#5 (严重度: 中) — README L66-72 已知限制 / 坑 提到 issues #960, #961——是否对工具能力下绝对结论需审视

**现象**：[`tools/aeneas-lean/README.md`](../../../../tools/aeneas-lean/README.md) L72：

> FnMut 闭包返回 `()`、trait `&mut` 实例化参数不匹配等场景下 aeneas 能产出文件但 Lean 类型检查会失败（issues #960, #961）

**违反**：[`tool-integration.md`](../../../design/tool-integration.md) §六禁忌——"对工具能力下绝对结论"。这条措辞描述"aeneas 能产出但 Lean type check 失败"——是工具实际行为陈述，不算绝对结论；并锚定 upstream issue 编号（诚实指向已知 bug），合规。

**推理链**：[`tool-integration.md`](../../../design/tool-integration.md) §五 (7) "已知限制 / 坑：平台限制、依赖限制、工具自身已知 bug"。这条满足。

**决策性**：非决策点。

**建议**：无须改。

#6 (严重度: 低) — README L74-76 关联 sub-tests 描述统一

**现象**：[`tools/aeneas-lean/README.md`](../../../../tools/aeneas-lean/README.md) L76：

> `examples/aeneas-limit/` 是 Aeneas（不分 backend）自声明的限制集——故意触发已知"不支持"特性（...），期望本 backend 在这些 entry 上 FAILED。

**违反**：未违反——按 [`tool-integration.md`](../../../design/tool-integration.md) §五 (8) "关联 sub-tests"。

**推理链**：4 个 aeneas backend 共享同一 limit 类目，README 措辞一致。

**决策性**：非决策点。

**建议**：无须改。

#7 (严重度: 低) — README 在多处使用 `^Errors.error_list` 引用 + `Main.ml:773` 行号

**现象**：[`tools/aeneas-lean/README.md`](../../../../tools/aeneas-lean/README.md) L40-44 多次引用上游源码行号：

> `Main.ml:773` `if has_errors then exit 1`
> aeneas 用 `craise` 把所有 unsupported 项 push error_list

**违反**：未违反——[`tool-integration.md`](../../../design/tool-integration.md) §三-源码层穷尽 / §四-4.1 形式证明的标准做法。

**推理链**：行号锚定于 commit `a14083a6`——若 aeneas 上游修改，行号可能漂移。README 应说明"以下行号锚定 commit a14083a6"。L52 已说明 commit。

**决策性**：非决策点。

**建议**：无须改。

## §4 决策点 vs 非决策点

- 决策点：0
- 非决策点：7

## §5 审查结论

aeneas-lean README 是矩阵中**最完整、最严谨的 README**之一——形式严格性"形式可证" 论证扎实（Errors.error_list + Main.ml:773 单一通路），与 wrapper 完全一致。

可微调（非必须）：

- L34 "成功 entry 产物里零 sorry / Admitted" 应澄清这是 aeneas 设计保证而非 oracle 实施——避免读者误以为 wrapper 走 sorry grep。

无 critical 问题。可作为其他工具 README 的参考体例。
