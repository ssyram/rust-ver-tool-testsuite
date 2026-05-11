# rocq-of-rust-typecheck readme Review

## §1 问题意识

rocq-of-rust-typecheck README 重点：(1) 档位定义（档 0 / 档 1 / 档 2 / 档 3）的清晰划分；(2) 与 rocq-of-rust 档 0 的严格上层关系；(3) 档 2/3 不实施的工程理由（ror deep embedding 设计）；(4) runtime bootstrap 自动幂等机制；(5) 形式严格性 ✅ 基本可形式证明。

恶意角度考察：档位定义是否合规？是否对 rocq-of-rust 能力下绝对结论？

## §2 审查方法

参照源：

- [`docs/design/tool-integration.md`](../../../design/tool-integration.md) §五 / §六；
- [`tools/rocq-of-rust-typecheck/tool.toml`](../../../../tools/rocq-of-rust-typecheck/tool.toml)；
- [`tools/rocq-of-rust-typecheck/rocq-of-rust-typecheck-wrapper.sh`](../../../../tools/rocq-of-rust-typecheck/rocq-of-rust-typecheck-wrapper.sh)；
- [`docs/fixes/rocq-of-rust-typecheck-implementation-2026-05-11.md`](../../rocq-of-rust-typecheck-implementation-2026-05-11.md)；
- [`docs/design/hax-lean-consistency-design-2026-05-11.md`](../../../design/hax-lean-consistency-design-2026-05-11.md) 档位定义。

## §3 审查现象

#1 (严重度: 低) — README 8 章节齐全

**现象**：[`tools/rocq-of-rust-typecheck/README.md`](../../../../tools/rocq-of-rust-typecheck/README.md) 完整。

**违反**：未违反。

**决策性**：非决策点。

**建议**：无须改。

#2 (严重度: 低) — README L9-19 档位定义清晰 + 档 2/3 不实施的工程理由扎实

**现象**：[`tools/rocq-of-rust-typecheck/README.md`](../../../../tools/rocq-of-rust-typecheck/README.md) L9-19：

> | 档 | 含义 | 本工具 |
> |---|---|---|
> | 档 0 | 工具自陈"前端接受"（`.v` 产物落盘 + 无显式 failure marker）| `tools/rocq-of-rust` |
> | **档 1** | **产物在 Rocq 里 typecheck 通过** | **本工具** |
> | 档 2 | 产物 entry_fn `Compute` 出值 | n/a（ror 设计上不支持）|
> | 档 3 | evaluate 结果 == `cargo run` 结果 | n/a（ror 设计上不支持）|
> 档 2/3 不实施的理由：ror 产物是 deep embedding `M = LowM.t (Value.t + Exception.t)`，每个 op 是 inductive constructor，无 native compute；上游设计上把"语义"留作用户 proof obligation...

**违反**：未违反——档位定义来源于 [`hax-lean-consistency-design-2026-05-11.md`](../../../design/hax-lean-consistency-design-2026-05-11.md)。档 2/3 不实施的理由扎实——锚定 ror 设计（deep embedding）。

**推理链**：与 [`tool-integration.md`](../../../design/tool-integration.md) §五 (3) "SUCCESS 信号 — 形式指标" 一致——档 1 的形式指标是 coqc exit 0 + .vo 产物。

**决策性**：非决策点——优秀范例。

**建议**：无须改。

#3 (严重度: 中) — README L85-89 形式严格性 ✅ 基本可形式证明

**现象**：[`tools/rocq-of-rust-typecheck/README.md`](../../../../tools/rocq-of-rust-typecheck/README.md) L87-89：

> **0 误报**：✅ **基本可形式证明**。gate 7（coqc exit 0）即 Rocq 9 typecheck 完整通过——Rocq typecheck 是确定性算法，无随机性、无 silent partial 路径。gate 8/9 仅是 coqc exit 0 的 belt-and-braces 复核...
> **0 漏报**：✅ **基本可形式证明**。coqc 对任何 typecheck 失败必 exit ≠ 0；gate 7 直接捕获。理论上 ror 翻译可能利用 axiom / `Admitted` 让产物 typecheck 通过但语义不真——本工具档 1 边界明确**只**保证 typecheck 通过，**不**保证"语义正确"或"evaluate 一致"。

**违反**：未违反——按 [`tool-integration.md`](../../../design/tool-integration.md) §四-4.1 形式证明 + §四-4.4 漏报盲点诚实。"基本可形式证明" 是精确措辞——coqc 是确定性算法（无 silent partial），但 stage 1 仍依赖 rocq-of-rust 的产物 grep（不可形式证明）。

**推理链**：合规——明确档 1 边界"**只**保证 typecheck 通过，**不**保证语义正确" 是严格诚实声明。

**决策性**：非决策点。

**建议**：无须改。

#4 (严重度: 低) — README L92-95 漏报盲点诚实

**现象**：[`tools/rocq-of-rust-typecheck/README.md`](../../../../tools/rocq-of-rust-typecheck/README.md) L92-95：

> - **`Admitted` 一通到底**：ror 翻译生成的 `Global Instance Instance_IsFunction_<fn> : ... Admitted.` 是 ror 的设计选择...这**不算漏报**，是档 1 的诚实边界："产物能在 Rocq 中编译通过"包含 `Admitted` 路径。本工具不评判 ror 设计选择，只测产物可编译性
> - **ror 翻译走 axiom**：若 ror 上游引入 `Axiom <name> : ...`（理论上 typecheck 仍过），同上不算漏报
> - 本工具档 1 边界**不**等同档 2/3（evaluate / 一致性），不构成对工具语义正确性的判断

**违反**：未违反——按 §四-4.4 + §六禁忌"不构成对工具语义正确性的判断"。严格档 1 边界声明。

**决策性**：非决策点。

**建议**：无须改。

#5 (严重度: 低) — README L97-125 安装段详细——隔离 opam switch 创建 + Rocq 9 install + ror runtime 依赖 + clone 上游

**现象**：[`tools/rocq-of-rust-typecheck/README.md`](../../../../tools/rocq-of-rust-typecheck/README.md) L103-122 详细 bash 命令清单。

**违反**：未违反——按 [`tool-integration.md`](../../../design/tool-integration.md) §一 锁定版本 + 安装说明。注释"本项目不提供安装脚本或步骤教程"——但这里 README 提供了详细命令，与该规则 tension。

**推理链**：[`tool-integration.md`](../../../design/tool-integration.md) §一 "安装方式（按上游文档自行安装；本项目不提供安装脚本——避免在工具版本变迁后误导）"。README L103 提供 bash 命令是 informative 而非"安装脚本"——介于 guideline 边界。

**决策性**：决策点——是否把 L103-122 详细命令删除或简化为"按上游 README 自行安装"。

**建议**：可选——保留作为示例，但加注"以下命令仅为参考，最新安装方式请按上游 README"。当前实施较强引导，但用户可调整。

#6 (严重度: 低) — README L138-145 已知限制 / 坑

**现象**：[`tools/rocq-of-rust-typecheck/README.md`](../../../../tools/rocq-of-rust-typecheck/README.md) L138-145 列 6 条（runtime path 绝对路径 / switch 名 / runtime bootstrap destructive / coqc 来源 / macOS-only / 不测 evaluate）。

**违反**：未违反。

**决策性**：非决策点。

**建议**：无须改。

#7 (严重度: 低) — README L147-151 关联 sub-tests 强调与档 0 的关系

**现象**：[`tools/rocq-of-rust-typecheck/README.md`](../../../../tools/rocq-of-rust-typecheck/README.md) L147-151：

> 本工具未派生限制集 agent，无 `examples/rocq-of-rust-typecheck-limit/`。
> 跟 ror 档 0 的 corpus 重合：在 `tools/rocq-of-rust` 标 SUCCESS 的 entry 集合上跑本工具，对比通过率差——差额暴露 ror 翻译的"产物能 emit、但 Rocq 不接受"的 silent typecheck bug。

**违反**：未违反——清晰说明档 1 上层关系 + 差额暴露 silent typecheck bug。

**决策性**：非决策点——优秀范例。

**建议**：无须改。

## §4 决策点 vs 非决策点

- 决策点：1（L103-122 详细安装命令 vs §一 "不提供安装脚本"）
- 非决策点：6

## §5 审查结论

rocq-of-rust-typecheck README **优秀**：

- 档位定义清晰（档 0 / 档 1 / 档 2 / 档 3）；
- 档 2/3 不实施工程理由扎实（ror deep embedding）；
- 形式严格性 ✅ "基本可形式证明" 措辞精确；
- 漏报盲点诚实声明（Admitted / Axiom 不算漏报，因档 1 边界明确）；
- runtime bootstrap 自动化文档完善。

整体属于"档 1 严格上层的杰出文档范例"——清晰传达档位语义边界。
