# verifast readme Review

## §1 问题意识

verifast README 重点：(1) verifast 单文件模式与 cargo 解耦；(2) corpus 全 0 spec 注解状态 + 2026-05-08 oracle 修订；(3) 反作弊 `-verbose 1` user file 触及检测；(4) 形式严格性论证。

恶意角度考察：是否对 verifast 能力下绝对结论？历史口径修订是否诚实？

## §2 审查方法

参照源：

- [`docs/design/tool-integration.md`](../../../design/tool-integration.md) §五 / §六；
- [`tools/verifast/verifast-strict-wrapper.sh`](../../../../tools/verifast/verifast-strict-wrapper.sh)；
- [`deep-reports/cc-reports/verifast.md`](../../../../deep-reports/cc-reports/verifast.md)；
- [`docs/fixes/oracle-leak-audit-2026-05-08.md`](../../oracle-leak-audit-2026-05-08.md) §3.1。

## §3 审查现象

#1 (严重度: 低) — README 8 章节齐全

**现象**：[`tools/verifast/README.md`](../../../../tools/verifast/README.md) 完整。

**违反**：未违反。

**决策性**：非决策点。

**建议**：无须改。

#2 (严重度: 中) — README L40 "**预期 SUCCESS 数从 116 (79.5%) 降到 0–2 (0–1.4%)**——这是合理的"——通过率描述临近能力评判

**现象**：[`tools/verifast/README.md`](../../../../tools/verifast/README.md) L40：

> **2026-05-08 oracle 调整后**：[`verifast-strict-wrapper.sh`](verifast-strict-wrapper.sh) 用 `-verbose 1` 检测 symex 是否触及 user file，spec-less entry 在新 oracle 下被识别为 vacuous pass → FAILED。**预期 SUCCESS 数从 116 (79.5%) 降到 0–2 (0–1.4%)**——这是合理的，因为本 corpus 在当前 corpus 设计上就不应该让 verifast 有非 trivial 真 SUCCESS。

**违反**：[`tool-integration.md`](../../../design/tool-integration.md) §六禁忌——临近"对工具能力下绝对结论"。"这是合理的，因为本 corpus 在当前 corpus 设计上就不应该让 verifast 有非 trivial 真 SUCCESS" 是 corpus 设计 vs 工具能力的双向陈述——合规。

**推理链**：实测 116 → 0-2 是 oracle 修订前后实测对比，锚定 corpus 与时点。"这是合理的" 是基于"corpus 全 0 spec + verifast 设计需 spec" 的逻辑必然性陈述，非工具能力评判。

**决策性**：非决策点。

**建议**：可选——把"这是合理的" 改为"在 corpus 全 0 spec 状态下，verifast 实测 SUCCESS 应趋近 0—oracle 行为对齐 verifast 设计"——更精确。

#3 (严重度: 高) — README L60-64 "vacuous-pass 历史口径修订（重要）"——诚实历史声明优秀范例

**现象**：[`tools/verifast/README.md`](../../../../tools/verifast/README.md) L60-64：

> ### vacuous-pass 历史口径修订（重要）
> 2026-05-08 之前 README 把"`-skip_specless_fns` 让 spec-less entry 退化为 vacuous pass"称为"语义降级，不是漏报"。审计...按项目宪法 §6 反作弊原则口径校正：**SUCCESS = 工具完整完成它的工作单元**——`-skip_specless_fns` 让 entry 完全没经过 verify 阶段，本质就是 silent skip，符合 partial 定义，应封堵。
> 新 wrapper 落地这次口径校正。**当前实施下，SUCCESS = symex 真跑过用户代码**，与项目宪法对齐。

**违反**：未违反——优秀诚实历史声明。与 kmir L25-26 同模式——记录 oracle 修订路径。

**推理链**：诚实声明 + 锚定具体修订理由 + 引用 §6 反作弊原则 = 高诚实度。

**决策性**：非决策点——优秀范例。

**建议**：无须改。

#4 (严重度: 中) — README L46-57 形式严格性论证扎实

**现象**：[`tools/verifast/README.md`](../../../../tools/verifast/README.md) L55-58：

> - **partial 暴露机制**：rustc-verifast 任何 IR 构造失败（async / closure / 浮点 / const-generic 等）→ exit ≠ 0；symex verify err → exit 1；vacuous pass（symex 未触及 user file）→ wrapper 重写为 exit 2
> - **形式严格性 — 0 误报**：✅ 形式可证。verifast exit 0 + verbose 输出含 user-file 行 ⇔ symex 在用户代码上执行了至少 1 条 statement
> - **形式严格性 — 0 漏报**：✅ 实测 + 设计论证。`-skip_specless_fns` 跳过 user fn 时 verifast 不会发任何带用户源文件路径的 verbose 行（仅 prelude 行）—— 0 漏报由 verifast 设计强制
> - **反误报双向实测**：[`oracle-validation/`](oracle-validation/) 子目录含 spec-less + spec-bearing 两个 micro-test...

**违反**：未违反——按 [`tool-integration.md`](../../../design/tool-integration.md) §三/§四 形式严格性论证。

**推理链**：✅ 标注合理——`-verbose 1` 的 per-statement source-path 标签是 verifast 自身设计的稳定 signal，与 K Framework `#EndProgram` 同性质（工具自身设计层信号）。比纯 grep heuristics 强。

**决策性**：非决策点。

**建议**：无须改。

#5 (严重度: 低) — README L91-99 已知限制 / 坑 表格清晰

**现象**：[`tools/verifast/README.md`](../../../../tools/verifast/README.md) L91-99：

| 限制 | 说明 |
|------|------|
| 单文件模式 | verifast 直接处理 `src/lib.rs`...
| 无跨 crate 调用 | 不经 cargo 链接... |
| ... | ... |

**违反**：未违反——表格化已知限制清晰。

**决策性**：非决策点。

**建议**：无须改。

#6 (严重度: 低) — README L101-106 关联 sub-tests 说"plain Rust 样例在 `-skip_specless_fns` 下全部预期 SUCCESS（工具静默通过）"——但 oracle 修订后预期 FAILED

**现象**：[`tools/verifast/README.md`](../../../../tools/verifast/README.md) L106：

> plain Rust 样例在 `-skip_specless_fns` 下全部预期 SUCCESS（工具静默通过）。带正确 `//@ req/ens` 注解的样例进入 SMT 验证；带错误 spec 的样例预期 FAILED（exit 1）。

**违反**：[`tool-integration.md`](../../../design/tool-integration.md) §五——README 内部矛盾。L40 已说 oracle 修订后预期 SUCCESS 降到 0-2 (0-1.4%)——但 L106 说"plain Rust 样例在 `-skip_specless_fns` 下全部预期 SUCCESS"。L106 是旧 oracle 描述。

**推理链**：L106 是 2026-05-08 oracle 修订前的描述——应同步更新为"plain Rust 样例（无 spec）在新 oracle 下预期 FAILED（vacuous pass detection）"。

**决策性**：决策点——内部矛盾需修正。

**建议**：把 L106 改为：

> 在 2026-05-08 oracle 修订后：plain Rust 样例（无 `//@ req/ens` 注解）→ vacuous pass → FAILED；带正确注解 + 进入 symex → SUCCESS；带错误 spec → exit 1 → FAILED；触发 rustc-verifast IR 构造失败 → exit ≠ 0 → FAILED。

## §4 决策点 vs 非决策点

- 决策点：1（L106 内部矛盾）
- 非决策点：5

## §5 审查结论

verifast README **诚实历史声明优秀范例**：

- L60-64 历史口径修订诚实记录 oracle 演进；
- oracle-validation/ 独立目录是矩阵唯一；
- 形式严格性论证扎实（✅ 基于工具自身 verbose signal）。

**关键问题**：

1. **L106 内部矛盾**：与 L40 不一致——L106 是旧 oracle 描述，需同步更新。

整体属于"反作弊设计的杰出范例"，但 README L106 急需更新。
