# 审计报告 — 调研层（docs/research/）

## §1 问题意识

调研文档（`docs/research/*`）是"问题意识 → 项目目标 / 工具集成方法 / 可行性研究"的载体。按 `principles.md` §九"与下游文档的关系"，research 文档是外围参考——但它必须满足：(i) 反映项目当前方向；(ii) 调研结论被后续 commit 跟进，feasibility → impl 链完整；(iii) 调研发现的事实在后续 commit 实测推翻后，调研文档应有更新或交叉链接。

恶意角度：是否有"调研了但没做"项；是否有调研结论被实测 falsify 但调研文档未更新；是否有 P15 / P16 跟进的 research 与 impl 之间出现脱节。

## §2 审查方法

3 份调研文档：
- `docs/research/testsuite-research.md`（最早调研，2026-05-08 修订）
- `docs/research/translation-correctness-feasibility-2026-05-11.md`（feasibility）
- `docs/research/ror-runnable-deep-dive-2026-05-11.md`（深入调研）

逐一对照：调研结论 → 是否被 P15 / P16 跟进 → impl 实际实施 vs 调研结论是否对齐。

## §3 审查现象

### 3.1 高严重度（H）

**H-1：testsuite-research.md §3 "推后到 v2 的扩展点" 多条被实施跟进，但 research 文档未标更新**

第 36-43 行（"推后到 v2 的扩展点"）：

> 按 Occam 当前不做：
> - tool.toml.env / cwd 字段——目前 command = ["env", ...] 前缀已够
> - External_lib mode——样例作为 path-dep 时的工具行为差异
> - 期望比对（expect.<glob>）——SUCCESS / FAILED 进一步区分 verdict 正确性
> - bug_description 接 LLM 做 bug 描述匹配
> - 报告里 entry 名超链到 raw outputs
> - 跨运行对比（run N 与 N-1 差异）

但实际：
- P16 引入 `runnable` schema 在 hirusttest.toml（`runnable.<entry_fn>.inputs` + `expected`）—— 这本质就是"期望比对"的扩展（虽然方向不同，但 schema 思路重合）。research 文档的 §3 "推后" 没标 "→ P16 部分跟进"
- P15-impl 引入 `tools/rocq-of-rust-typecheck/` 作为档 1 独立工具，把 typecheck 纳入测试——这与"v2 扩展" 同方向但 research 文档没反向链接

读者读 research 时会以为这些 v2 项目都还未做。

**H-2：translation-correctness-feasibility-2026-05-11.md §7 "框架化设计建议" → P15-impl 部分跟进，但调研文档未指向 impl**

第 263-274 行（§7.1 multi-stage 模型）：

> 可行的最小做法：新增独立 tool 入口（每个对应一档可信度）：
> | hax-lean-typecheck | wrap 现有 hax-lean，跑完再把产物拷到 hax lean prelude 项目里 lake env lean | exit 0 + 无 error + 无 sorry warning |
> | hax-lean-eval | 在 typecheck 基础上 append #eval entry_fn <args> ... |
> | hax-lean-consistency | hax-lean-eval + cargo run + 比对 |
> 类似为 hax-coq、rocq-of-rust 各开三个；前提是档 1 的 prelude 装好。

但实际 P15-impl 实施了 `tools/rocq-of-rust-typecheck/` 而非 `tools/hax-lean-typecheck/`——和调研建议的"短期 hax-lean only yes / ror 短期 no" 顺序**反过来**。调研第 252-254 行 §6.3 明说：

> 短期（hax-lean only）yes：lake prelude 已 build OK + 产物可 typecheck/eval 实测验证 + 一致性 OK 实测验证。可以做。
> 短期 hax-coq / rocq-of-rust no：阻塞在 prelude / runtime 编译

P15-impl 实际选了 rocq-of-rust（被 [`ror-runnable-deep-dive-2026-05-11.md`](../../research/ror-runnable-deep-dive-2026-05-11.md) §3 实测推翻"短期不可达"），但 translation-correctness-feasibility-2026-05-11.md 没在文末加"已被 ror-runnable-deep-dive 推翻"标注。两份调研有交叉但缺反向链接。

**H-3：ror-runnable-deep-dive-2026-05-11.md §6.1 "档 1 可达 + 档 2/3 不推荐" → P15-impl 实施档 1 + P16 实施 hax-lean-eval (档 2/3)，调研结论部分反转**

ror-runnable-deep-dive §7.2 第 396-406 行：

> 中长期：档 2/3 不推荐
> 理由：
> - per-entry 工作量与 corpus 规模冲突：现 corpus ~150 entry，按"5-50 行手写证明"估，全 cover 是数周-数月的 proof engineering
> - 测试结果可信度不增
> - 不与 hax-lean 档 2/3 对等

而 P16 在 hax-lean 上做了档 2/3（hax-lean-eval 框架 + 15 个 runnable entry）—— 注意这与 ror 档 2/3 不冲突（ror-runnable-deep-dive 明确说"hax-lean 档 2/3 自动可达"）。但读者读 ror-runnable-deep-dive §7.2 单独读会以为"档 2/3 普遍不做"，而实际项目选择了"ror 不做但 hax-lean 做"——这种**工具间分化决策**调研文档没明示。

### 3.2 中严重度（M）

**M-1：testsuite-research.md §2 相关工作 vs principles.md §一 重叠但表述不一**

testsuite-research.md §2"相关工作与 Gap"列 RVT / soarlab / arXiv 2410.01981 / AWS verify-rust-std / SV-COMP；principles.md §一也提到 RVT / soarlab "都已归档或低活跃"。表述基本一致，**但 testsuite-research.md 没明示"以下结论已纳入 principles.md §一"**。

**M-2：testsuite-research.md §4 "进入下一阶段" 引用 docs/design/architecture.md / detailed-design.md 但不引 principles.md**

第 48-50 行：

> 调研收尾。架构层与细化层落地于：
> - docs/design/architecture.md
> - docs/design/detailed-design.md
> - docs/test-reports/

→ 漏 `docs/design/principles.md`（项目宪法）和 `docs/design/tool-integration.md`（次要模块原则）。这与 `principles.md` §九"与下游文档的关系"建议的层次结构不一致。读者从 research 文档进入 design 文档时会先看到 architecture 而非 principles。

**M-3：translation-correctness-feasibility-2026-05-11.md "本机当前快照" vs P15 调研推翻的 rocq-of-rust 阻塞**

第 220-227 行（§6.1 三工具档位实测判断）：

> | **rocq-of-rust** | ✅ 已覆盖 | ❌ **本机不可达** | ❌ | ❌ | 阻塞：runtime 要 Rocq 9.0+ Stdlib.* namespace ...

但 `ror-runnable-deep-dive-2026-05-11.md` §3 实测装 Rocq 9.0.0 isolated switch 成功 + 编 runtime 通过 + 档 1 4/4 PASS。**已经把 "本机不可达" 推翻**——但 translation-correctness-feasibility-2026-05-11.md 没在第 6.1 表格旁加"⚠️ 见 ror-runnable-deep-dive，本机已装 ror-test switch 后可达"。

读者按时间顺序读这两份 research 时确实会发现，但若只读前者会误以为"ror 档 1 不可达"。

### 3.3 低严重度（L）

**L-1：testsuite-research.md §3 "扩展点" 不清晰区分"v2 推后 vs 已弃"**

第 36-43 行列了 6 项"v2 推后"。但其中 `bug_description 接 LLM 做 bug 描述匹配` —— project 走向上现在更倾向"形式指标"（principles.md §三-3-2.a），LLM-based bug 匹配可能已被 principle B（测必要条件，非语义对错）排除而非"推后"。研究文档应区分。

**L-2：ror-runnable-deep-dive §8 "短期不可行点（档 2/3 specific）" 第 437-442 行结论与 P16 hax-lean-eval-corpus-baseline.md 第 65 行的限制相同但未交叉引用**

ror-runnable-deep-dive §8 第 437 行：

> 4. stdlib method 的 linker 依赖：int_wrapping 用 u8::wrapping_add，要求 ror runtime 的 core/num/links/*.v 提供 Run.Trait for wrapping_add

`hax-lean-eval-corpus-baseline.md:65`（design §1.4 禁止集）：

> i*::checked_* / overflowing_*：design §1.4 禁止集——hax-lean prelude 未实现，翻译产物会含 sorry → 故意未纳入 corpus

→ 两份文档都识别到"stdlib method 缺 linker / spec"问题。一份是 ror 的 runtime，一份是 hax-lean 的 prelude——属"工具间对偶现象"。可以加交叉引用增进读者理解。

## §4 决策点 vs 非决策点

| 项 | 类型 | 理由 |
|---|---|---|
| H-1 | 决策点 | research 文档有 "推后到 v2" 节而后续实施部分跟进；需要在文档末尾加"→ 已部分跟进，详 docs/fixes/" |
| H-2 | 决策点 | 两份 research 之间缺反向链接，导致读者误认 P15-impl 路径 |
| H-3 | 决策点 | 工具间档 2/3 决策分化（ror no / hax-lean yes）调研文档应明示 |
| M-1, M-2 | 决策点（低） | 引用层级 / 标注，易改 |
| M-3 | 决策点 | "本机当前快照"被后续实测推翻，建议加交叉链接 |
| L-1 | 非决策点 | "v2 推后" vs "已弃"区分语义性，可不区分 |
| L-2 | 非决策点 | 跨工具对偶引用可加可不加 |

## §5 结论

3 份调研文档质量较高、问题意识清晰、相关工作识别准确。

**最严重 3 处**：
1. **H-1**：testsuite-research.md §3 "推后到 v2" 已被 P15-impl + P16 部分跟进但 research 文档未更新——读者会误以为这些项还未做
2. **H-2 + H-3**：translation-correctness-feasibility 与 ror-runnable-deep-dive 之间的交叉引用不完整，P15-impl 实际选择路径与 feasibility 调研推荐顺序"反过来"，需要标注
3. **M-3**：feasibility §6.1 "本机不可达"被 deep-dive 实测推翻但前者没加反向链接

**调研→实施未跟进项**：
- testsuite-research §3 "外部 lib mode" / "bug_description LLM 匹配" / "跨运行对比（run N 与 N-1 差异）"—— 完全未做，但未明确"放弃 / 推后"
- translation-correctness-feasibility §7.1 "hax-coq-typecheck / hax-lean-typecheck 独立 tool" 路径未实施，P15-impl 选 ror 路径而非 hax-lean——这是"实际 commit 与调研建议优先级反过来"的决策

**高严重度：3 / 中：3 / 低：2**。
