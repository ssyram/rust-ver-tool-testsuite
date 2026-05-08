# Rust 验证工具特性覆盖测试 — 调研报告

## 1. 问题意识

一群对"输入"理解互不相通的 Rust 验证工具（Kani 写 `#[kani::proof]`、Verus 写 `verus! { fn p() requires/ensures }` 宏块、Prusti / Creusot 写 caller 上的 `#[ensures]` attr、MIRI 当解释器跑 `fn main()`、Charon 翻译到 LLBC）被同时摆出来选型。**问题**：用什么样的输入对它们都中立、能筛选哪个工具吃得下哪种 Rust 特性？

任一现存路径——把样例改成工具特定形态，或把样例围绕统一抽象 API（propverify / verifier crate）写——都把"不公平"沉淀进样例本身：样例污染、抽象成新依赖、每加一工具都要改样例。

本项目的意识是把不公平沉淀到第三方中介层（runner + 配置 + 模板），让样例和工具两端各自纯粹。这一意识在派生原则与具体设计中展开，详见 [`design/architecture.md`](../design/architecture.md)。

## 2. 相关工作与 Gap

跨工具 Rust 验证 feature 覆盖测试的 gap 是真实的、被综述明确点名的、且未被填补。

**RVT (project-oak/rust-verification-tools)**：用 `propverify` / `verification-annotations` 包多个 verifier。要求样例侵入式使用其 API。2023 年 7 月归档。

**soarlab/rust-benchmarks**：跨工具 benchmark 仿 SV-COMP（assert/assume/nondet）。仍要求样例使用 `verifier` crate。低活跃。

**arXiv 2410.01981 "Surveying the Rust Verification Landscape" (2024)**：覆盖 Prusti / Creusot / Aeneas / Verus / Kani / SMACK / SeaHorn / Gillian-Rust / RefinedRust 等。明确指出社区**不存在**跨工具 benchmark 或系统 feature-coverage testsuite，工具选择只能 "manually evaluate"。

**AWS verify-rust-std (Rust Project Goal 2024h2)**：把多个 verifier 应用到 fork 的 std 上做"Challenges"——真实代码上的应用研究，与本项目互补不冲突。

**SV-COMP**：C / Java 跨工具竞赛 14 届，**未设 Rust 类目**。

| 维度 | RVT / soarlab | 本项目 |
|---|---|---|
| 样例对工具的依赖 | 必须用 propverify / verifier crate | 零依赖、plain Rust |
| 工具适配方式 | Rust 代码 + crate + FFI shim | 纯配置：`tool.toml + harness.rs.tera` |
| 评测目标 | 同语义跑通 / 求解结果对比 | feature 覆盖广度 |
| 状态 | 归档 / 低活跃 | greenfield |

两次先行尝试都走"统一 verifier API + 样例侵入"路径并停滞——这条路径锁工具、累样例。本项目放弃 SV-COMP 风格的语义统一抽象，换"必要条件 + 配置驱动"的窄目标，与已知先行尝试不冲撞。

## 3. 推后到 v2 的扩展点

按 Occam 当前不做：

- `tool.toml.env` / `cwd` 字段——目前 `command = ["env", ...]` 前缀已够
- External_lib mode——样例作为 path-dep 时的工具行为差异
- 期望比对（`expect.<glob>`）——SUCCESS / FAILED 进一步区分 verdict 正确性
- `bug_description` 接 LLM 做 bug 描述匹配
- 报告里 entry 名超链到 raw outputs
- 跨运行对比（run N 与 N-1 差异）

## 4. 进入下一阶段

调研收尾。架构层与细化层落地于：

- [`docs/design/architecture.md`](../design/architecture.md) — 三原则、运行时投影、通用性论证、模块切分、接口规约
- [`docs/design/detailed-design.md`](../design/detailed-design.md) — schema 完整、函数级前后置、运行时伪码、19 工具配置实例
- [`docs/test-reports/`](../test-reports/) — 实测数据快照（时效性强，非长期承诺）

修正方案文档归档在 [`docs/fixes/`](../fixes/)。
