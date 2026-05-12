# Glossary — Project Self-Defined Terminology

> 项目自创术语 → ISSTA 投稿术语对照。每个 term 在 paper 首次引入处必须显式定义。

## 核心 construct（必入 paper 引入段）

| 项目术语 | 英文 (paper register) | 定义 |
| --- | --- | --- |
| **特性覆盖广度** | feature acceptance breadth | the set of Rust language features that a tool accepts as input + produces a non-partial output for; deliberately **not** "verification correctness" |
| **前端测量** | front-end-cut measurement | restrict measurement to a tool's translation / type-checking / IR-construction phase, excluding back-end solving; per-tool cut point declared in `tools/<name>/README.md` |
| **当前 crate 焦点** | target-crate scope (width-cut) | partial signals occurring inside the target crate count as FAILED; partials in transitively imported deps (std / cargo registry / vendor) are tool-by-design opaqueification and do not count |
| **本地性原则** | locality / current-version principle | measurement reflects current tool version + its required toolchain; a tool failing on stable Rust under its own pinned toolchain *is* a capability boundary |
| **遵循社区惯例** | community-conventional usage | tools are exercised through their documented / recommended interfaces, not adversarial inputs |
| **最大善意** | best-effort host accommodation | install each tool's required toolchain, configure per docs, before observing failure |
| **三原则栈** | three-tier credibility stack | locality > community-conventional > best-effort; resolves conflicts in measurement framing |
| **工具触达其能力边界** | tool reaches its capability boundary | replaces colloquial "工具菜" — a tool's documented or implementation-level inability to handle a construct |
| **不公信** | (lack of) public credibility | one of the two root problems addressed by the framework; the other is **不公平** (unfairness) |
| **不公平** | (sample-tool) asymmetric bias | corpus or harness skewed toward one tool's sweet spot |

## Oracle 设计概念

| 项目术语 | 英文 | 定义 |
| --- | --- | --- |
| **0 误报形式可证** | formal-style argument for zero false positives | a single-exit-code-channel invariant argument; **not** machine-checked proof — softened in P38 to "by-design no-silent-skip + source-level argument" |
| **0 漏报实测 + wrapper 双通路封堵** | empirical + dual-channel wrapper gate evidence for zero false negatives | tool's own error channel + project-side wrapper grep gate, anchored on a v6 corpus run |
| **UNKNOWN 严格语义** | strict UNKNOWN semantics | UNKNOWN reserved for (a) global toolchain crash and (b) identified our-side problem with documented remediation plan |
| **官方 wrapper vs 我们 wrapper** | upstream wrapper vs project-maintained wrapper | failure attribution split: upstream-wrapper crash → tool capability boundary (FAILED); our-wrapper crash → UNKNOWN |
| **silent partial / silent skip** | silent partial / silent skip-item | tool emits a stub/opaque/placeholder but still exits 0; the project oracle is engineered to surface these as FAILED |
| **bug detect = SUCCESS** | bug-detection counts as SUCCESS | for abstract interpreters (MIRI) and symbolic executors (soteria), a self-disclosed user-code bug is a valid output form and counts as SUCCESS per §四 B |

## 切割维度概念

| 项目术语 | 英文 | 定义 |
| --- | --- | --- |
| **深度切割** | depth-cut | front-end vs back-end (per `principles.md` §六 "前端测量") |
| **宽度切割** | width-cut | target crate vs external deps (per §六 "当前 crate 焦点") |
| **三路径 (A / B / C)** | three implementation strategies for the width-cut | A: source-path filter (aeneas, P36); B: reverse-evidence keyword grep (kani, P37); C: single-file pipeline naturally satisfies (verus / verifast / soteria / ror) |

## Methodology 术语

| 项目术语 | 英文 | 定义 |
| --- | --- | --- |
| **c 路 / cc 路** | disprove-first round / counter-challenge round | per `charter-craft` §4.8: c-route is independent challenger finding flaws; cc-route counters each finding; only those that survive both rounds are landed |
| **时空锚定** | spatio-temporal anchoring | every reported number anchored to (run-id, tool-version, host, ISO timestamp, corpus commit hash) tuple |
| **本工具非静态原则** | tool-snapshot semantics | a tool's measured behavior reflects a frozen version + toolchain; not a long-term capability commitment |

## 模块定位

| 项目术语 | 英文 | 注 |
| --- | --- | --- |
| **核心模块 (runner / examples)** | first-class modules | long-term commitment under `principles.md` §三 |
| **次要模块 (tools / 实测报告)** | second-class modules | application snapshots, not long-term commitments — explains why `tools/<name>/` integration tuning is not a measurement-quality claim |

## paper 引入前必做

1. 主报告 §0.5 / paper §2 加 glossary 引用 (cross-reference 本文件)
2. 每条术语首次出现在 paper 主文时显式定义
3. 删除 / 替换文案中任何剩余口语化措辞 (P38 已替换"工具菜"，未来 audit 实测剩余)
