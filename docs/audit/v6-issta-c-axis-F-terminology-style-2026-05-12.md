# v6 ISSTA c-axis F — 术语精确性 / 引用规范 / 学术写作风格审查

> **角色**：ISSTA 2026 reviewer，按 SIGSOFT camera-ready style + ACM 引用规范挑刺。
> **范围**：主报告 v6 + `docs/design/*.md` + `deep-reports/cc-reports/*.md` × 20 + `tools/*/README.md` × 20。
> **协议**：disprove-first，区分 (a) 项目自创术语在自家文档可用（R5/R11/R12 作者反驳点站得住）vs (b) paper 投稿前必须翻译 / 定义 / 引用补齐的 blocking issue。
> **限制**：本文档是审查产出，**不修改任何 design / tool 文档**——R7 修复落到 paper 草稿阶段。

---

## §1 术语精确性 — 挑刺清单

### 1.1 项目自创 / 非标准术语（paper 引入前需定义）

| 中文 / 当前用法 | 现状 | paper 英文建议 | 严重度 |
|---|---|---|---|
| **前端测量 / 前端边界**（principles §六）| 自创——PL 文献 "front-end" 通常指 lexer + parser + type-checker；本项目"前端 = parser + type-check + IR construction + 翻译 / 模型构造（除求解外）" | "front-end-only evaluation" 或 "pre-solver pipeline coverage"；定义需明示**排除 SMT / 模型检查求解层** | **高**——核心方法学术语 |
| **当前 crate 焦点（宽度切割）** | 自创 | "entry-crate-scoped measurement (width cut)" | **高**——核心方法学术语 |
| **深度切割 / 宽度切割** | 自创比喻 | "depth cut (pipeline-stage)" / "width cut (crate-scope)"；首次引入处给定义框 | 高 |
| **翻译深浅** | 自创（principles §四 B 推论） | "translation depth (syntactic / MIR-level / verifier-IR-level)"——首次给三档枚举 | 中 |
| **工具菜**（principles §一-2-1 / decisions-2026-05-11.md L11）| 口语，paper 投稿 blocker | "the tool fails on this feature under its own recommended toolchain" | **极高**——非学术口语 |
| **白嫖 / 水线**（grep 未命中，但记忆中提及）| 不在已审文档 | — | — |
| **不公平 / 不公信**（principles §一）| 自创问题意识标签 | "neutrality" / "credibility (defensibility against tool-author rebuttal)" | 高——双根本问题须正式命名 |
| **最大善意 / 本地性 / 社区惯例**（三原则栈）| 自创 | "principle of charity (good-faith effort)" / "locality (test-as-shipped)" / "community-convention alignment" | 中 |
| **partial / silent partial / silent skip / silent stub** | 半英 — verification 文献用 "incomplete translation" / "silently unsupported features" / "stub-and-continue" | 统一为 "silent partial (incomplete acceptance without diagnostic)" | 中 |
| **0 误报 / 0 漏报 形式可证**（principles §六 + 18 个 tool README）| "formally proven zero false positives / negatives"——**reviewer 必挑**：strong claim 须形式化定义"误报""漏报"语义 | 严格定义：FP = oracle 判 SUCCESS 但工具自陈 partial / FN = oracle 判 SUCCESS 但产物含 silent skip；"formally proven" 改为 "argued by single-channel exit-code invariant" 或 "single-signal-path argument" | **极高**——直接影响 reviewer 信任 |
| **形式严格性**（18 个 tool README §"SUCCESS 信号 + 形式严格性"）| 不存在英文对应——意指"基于源码层 invariant 而非实测统计" | "structural argument (vs empirical observation)" | 中 |
| **bug detect 归 SUCCESS**（architecture §一）| 自创 | "bug-detection counts as SUCCESS (tool finishes its job)" — 须解释为何不算 verification fail | 中 |

### 1.2 标准术语用对了吗（按工具核校）

| 工具 | 当前自陈 | 学术标准用语 | 状态 |
|---|---|---|---|
| **kani** | "有界模型检测器 bounded model checker" | bounded model checker ✓ | 正确 |
| **prusti** | "deductive verifier"（implicit） | deductive verifier (Viper-based) ✓ | 正确，建议显式标 "deductive verifier" |
| **verus** | "SMT-based spec verifier" | SMT-based deductive verifier ✓ | 正确 |
| **creusot** | 隐含 deductive | deductive verifier (Why3-based) ✓ | 正确，建议显式 |
| **verifast** | "Separation-logic verifier ... using symbolic execution and an SMT prover" | symbolic-execution-based separation-logic verifier ✓ | 正确——是审过最规范的一行 |
| **soteria** | "符号执行引擎 ... 原生 Tree Borrows" | symbolic execution engine ✓ + "with native Tree Borrows aliasing model" | 正确 |
| **miri** | "MIR-level 解释器"（implicit）+ "UB detection" | MIR interpreter / dynamic UB detector ✓ | 正确——但 README 未显式标"interpreter"，paper 须加 |
| **kmir** | "K Framework 上的 Rust MIR 形式化操作语义执行引擎" | executable operational semantics interpreter ✓ | 正确，但"形式化操作语义"应英化为 "executable operational semantics" |
| **charon** | "纯翻译工具" / "Rust → LLBC translator" | Rust-to-LLBC translator ✓ | 正确 |
| **aeneas** | "Rust → 多 PA 翻译器" | Rust-to-PA translator (functional translation) + "via symbolic interpretation + SymbolicToPure" | "多 PA" 应为 "proof assistant"——首次引入须展开 |
| **hax** | "Rust → 多个证明助手翻译器" | Rust frontend for verification (multi-backend translator) ✓ | 正确 |
| **rocq-of-rust** | "Rust → Rocq 自动翻译工具" / "syntactic transcoder" | shallow / syntactic translator ✓ | 正确——但 cc-report 用 "syntactic transcoder" 不是标准术语，建议 "shallow embedding into Rocq monadic IR" |
| **cargo-check** | "baseline / corpus 合法性兜底" | sanity baseline (rustc front-end) ✓ | 正确 |

**互换误用候选**：

- "verifier" vs "model checker" vs "static analyzer" — 当前文档**未严格区分**：kani 是 model checker，prusti/creusot/verus 是 deductive verifier，verifast 是 symbolic-exec-based verifier。已审 README 用对，**但主报告与 cc-reports 多处通用"verifier"称呼**——paper 须 reviewer-friendly 地全文统一三类分桶。
- "translator" 用于 charon/aeneas/hax/ror **统一且正确**。
- "abstract interpretation" — 全工具集**无一例**使用该术语——正确，因 corpus 内无 AI-based 工具。

### 1.3 "0 误报 / 0 漏报 形式可证" — reviewer 必挑的核心

ISSTA reviewer 看到 "formally proven 0 false positives / 0 false negatives" 会立刻要求：

1. **定义 FP / FN 的 ground truth**——本项目没有独立 oracle，FP/FN 是相对于 "工具自陈是否完整完成 entry crate 的工作"。这不是经典 FP/FN 语义。
2. **"形式可证" = formal proof?** ——当前实质是 "single-exit-code-channel invariant argument"，远弱于 formal proof。建议改：
   - "0 false positives" → "no SUCCESS without exit 0 AND no documented partial-emit marker"
   - "0 false negatives" → "every tool-self-disclosed partial (via exit-code or stderr marker) is captured by the oracle"
   - "formally proven" → "argued by structural single-channel invariant" 或 "audited via exhaustive source-level reject-path enumeration"

不改这一组表述，paper reviewer 第一轮就会 reject。

---

## §2 上游工具论文引用（20 工具）

### 2.1 应引核心 paper（venue + year + 缺失状态）

| 工具 | 应 cite paper | venue / year | DOI/arXiv 估算 | 当前 README 引用状态 |
|---|---|---|---|---|
| **kani** | "Kani Rust Verifier" — Vanhattum et al.（AWS 团队 SAS / ICST workshop / arXiv） | arXiv 2208.05545 + SAS'23 workshop tutorial | arXiv:2208.05545 | **缺失**——仅有 GitHub URL |
| **prusti** | "Leveraging Rust Types for Modular Specification and Verification" — Astrauskas, Müller et al. | OOPSLA 2019 / ECOOP follow-ups | DOI 10.1145/3360573 | **缺失** |
| **creusot** | "Creusot: a Foundry for the Deductive Verification of Rust Programs" — Denis, Jourdan, Marché | VSTTE 2022 | DOI 10.1007/978-3-031-25803-9_6 | **缺失** |
| **verus** | "Verus: Verifying Rust Programs using Linear Ghost Types" — Lattuada, Hance, Cho et al. | OOPSLA 2023 | DOI 10.1145/3586037 | **缺失** |
| **aeneas** | "Aeneas: Rust Verification by Functional Translation" — Ho, Protzenko | ICFP 2022 | DOI 10.1145/3547647 | **缺失**（4 个 backend README 均缺）|
| **charon** | 无独立 paper（Aeneas 论文内介绍）；可 cite Aeneas + GitHub | — | — | 引用 Aeneas 即可 |
| **hax** | "Tech Report: Hax — Translating Rust into formal languages" + Bhargavan et al. (hacspec)；目前 arXiv | arXiv 2410.x（hax 技术报告）/ hacspec POPL workshops | arXiv:2410.18526 (hax) | **缺失** |
| **miri** | Ralf Jung's PhD thesis + "Stacked Borrows" POPL 2020 — Jung, Dang, Kang, Dreyer | POPL 2020 + PLDI 2018 "RustBelt: securing the foundations" | DOI 10.1145/3371109 (SB) | **缺失** |
| **kmir** | "K — A Semantic Framework for Programming Languages and Formal Analysis Tools" — Roşu, Şerbănuţă | JLAMP 2010 + RV 系列；mir-semantics 应 cite RV / Rust foundation post | — / GitHub | **缺失** — 仅 GitHub |
| **verifast** | "VeriFast: A Powerful, Sound, Predictable, Fast Verifier for C and Java" — Jacobs, Smans, Philippaerts, Vogels, Penninckx, Piessens | NFM 2011 | DOI 10.1007/978-3-642-20398-5_4 | **缺失** |
| **rocq-of-rust** | Claret et al. / Formal Land blog post + arXiv？目前 arXiv 上未见明确 paper | (尚未正式发表 / 仅 GitHub) | — | 引用 GitHub OK，但建议 cite Formal Land 团队 blog 系列文章 |
| **rocq-of-rust-typecheck** | 同 ror（项目自家 tier-1 wrapper，无独立 paper） | — | — | N/A |
| **soteria** | 无 paper / 仅 GitHub（soteria-tools 组未发表）；可注 "no published paper as of 2026-05" | — | — | 引 GitHub + 注明未发表 |
| **cargo-check** | rustc / cargo —— Rust foundation；可不 cite | — | — | N/A |

### 2.2 paper-level 引用建议

- 主报告（v6 final）在 §1 TL;DR 后加一节 "Tools under evaluation"，列 20 工具 × (name, version commit hash, primary paper citation)。
- 每个 tool README 顶部"简介"段须加 [Citation] 字段——当前所有 20 个 README **均未引用学术论文**，仅 GitHub URL，paper 投稿 blocker。
- "rocq-of-rust" / "soteria" 暂无正式 paper，cite 形式："Formal Land, *rocq-of-rust* (GitHub), accessed 2026-05-12, commit `a8a76a4d`."——ACM 接受。

---

## §3 数据呈现 — reviewer 必挑

### 3.1 数字精度过度

主报告 v6 §3：

- `68.39%` / `60.87%` / `77.0%` / `8.1%` — 4 位有效数字。ISSTA reviewer 一定挑：百分比基于 161 entries / 20 工具 = 3220 task，**无 CI、无重复 run、无 sample variance**——4 位精度暗示精度不可承诺。
- 建议：所有通过率 round 到整数 + 显式给 raw count "S=98 / n=161"。`60.87%` → "61% (98/161)"。

### 3.2 排序暗示 ranking

§3 表 "按通过率排序 desc" — paper reviewer 直接读为 "tool ranking"。本项目宪法明示**不做工具能力评判**——表头排序违 §一 / §二 不可妥协价值条 3 + tool-integration 中立性。建议：

- paper 中按字母序或按工具类别（model checker / deductive verifier / translator / interpreter）分组排列
- caption 明示 "ordered by acceptance rate for readability; not an authoritative ranking of tool capability"

### 3.3 numbered figures / tables 缺失

当前 markdown **无 Table 1 / Figure 1 编号**。paper 形态须：

- 主报告 §3 大表 → "Table 1: Per-tool acceptance rates ..."
- §3.1 自洽性 → 可作 inline table 或 Table 2
- §3.2 UNKNOWN 10 个全列 → "Table 3: Residual UNKNOWN cases"

---

## §4 写作风格 — paper 投稿 blockers

### 4.1 中文 markdown → 英文翻译时最难处理的 5 类

| 项目自创术语 | 翻译难度 | 建议 paper 章节首次定义 |
|---|---|---|
| "前端 / 前端测量 / 前端边界" | **极高**——与 PL 文献 "front-end" 冲突 | §3.1 Methodology — Front-end Evaluation: A Definition |
| "宽度切割 / 深度切割" | 高——比喻须直译为 cuts | §3.2 Two Cuts: Depth (pipeline-stage) and Width (crate-scope) |
| "0 误报 / 0 漏报 形式可证" | **极高**——strong claim, reviewer 必挑 | §3.3 Oracle Strictness: Single-channel-exit Invariants |
| "本地性 / 社区惯例 / 最大善意" | 中——三原则栈须命名 | §3.4 The Three-Principle Stack for Tool-Author Defensibility |
| "工具菜 / 水线 / 白嫖" 等口语 | 极高——投稿 blocker | 删除 / 替换为正式表述 |

### 4.2 标准 paper 章节 — 当前散落需汇总

ISSTA paper 标准结构 vs 当前文档分布：

| ISSTA paper 章节 | 当前散落位置 | 整合建议 |
|---|---|---|
| **1. Introduction** | `principles.md §一`（双根本问题）+ README.md | 直接重写：motivation = neutrality + credibility 双问题 |
| **2. Background** | `cc-reports/*.md` pipeline 段（散在 20 文件中）| 抽 5-page background：Rust verification landscape + 20 tools 分类 |
| **3. Methodology / Framework** | `architecture.md` + `tool-integration.md` + `detailed-design.md` | 汇总：runner + isolation + harness + oracle |
| **4. Evaluation Setup** | 主报告 §0 元数据 | 直接复用 |
| **5. Results** | 主报告 §3 + §6 | 直接复用，加 numbered table |
| **6. Discussion / Threats to Validity** | 主报告 §0.3 修宪 + §3.1 自洽性 + 散在 audit/*.md | **缺失专章**——须新写 "Threats to Validity"（时空锚定 / oracle 严格性 / 不构成 ranking 等） |
| **7. Related Work** | **完全缺失** | 须新写：之前的 Rust verifier surveys (e.g., RustBelt 系列 / Astrauskas survey) |
| **8. Conclusion** | 主报告 §6 关键发现 | 直接复用 |

**最影响 publish 的 style 问题排序**（top 3 blockers）：

1. **"0 误报 / 0 漏报 形式可证"必须降级表述**——reviewer 必首挑（极高优）
2. **20 工具学术论文引用全缺**——reviewer 必首挑（极高优）
3. **缺 Threats to Validity 专章 + 缺 Related Work 专章**——结构性 blocker

---

## §5 可索引性 / artifact link

### 5.1 当前状态

- 项目 commit hash：主报告 §8 commit 链有列出，但**未在主报告顶部 banner 醒目锚定**
- DOI / artifact link：**全缺**——paper 形态须 Zenodo 或类似存档
- corpus / runner / cc-reports 在 paper 中引用方式：当前无规范

### 5.2 paper 形态建议

- 主报告 §0 元数据后加 "Artifact" 段：列 (a) corpus snapshot Zenodo DOI / (b) runner repo commit `ebe6858` / (c) results.json 原始 run id `run-1778560393-59119`
- ACM artifact badges 申请：functional / reusable / available 三档——本项目结构上对齐 "available + reusable"

---

## §6 R5 / R11 / R12 作者反驳点核校

| 反驳点 | 是否成立 | 说明 |
|---|---|---|
| **R5：自创术语在自家文档可用** | ✓ 成立 | 项目宪法 / 架构 / tool README 内部使用"前端 / 宽度切割 / 不公平"等自创术语合理——本审查**不要求修改这些文档** |
| **R11：tool README 不必引用学术 paper** | ✗ 不成立 | 即使内部文档不引，paper 投稿时**每个工具必须 cite**——这是 ACM 引用规范硬要求。建议在 paper 草稿中新建 references.bib，不污染 tool README |
| **R12：测试报告精度 4 位是工程精确性** | 部分成立 | 工程上可以记录精确数；但 paper 表述时须 round + 加 raw count——这是社区惯例 |

**结论**：R5 全过，R11 部分不过（paper 须补），R12 部分不过（paper 须 round）。

---

## §7 完成回 — 决策点 vs 非决策点

### 7.1 非决策性观察（不必打扰用户）

- 字段命名 / 表格格式 / numbered figures——paper 草稿期任由作者偏好
- README 内部使用 "工具菜"、"形式可证" 等自创术语——**自家文档可用，paper 投稿前再统一翻译**

### 7.2 真决策点（待用户裁决）

| 决策点 | 选项 | 建议 |
|---|---|---|
| **DP-F1**：是否在 paper 投稿期**预先维护一份英文 abstract + key-term glossary**（避免投稿前突击翻译错术语）| (a) 不维护，投稿时一次性翻；(b) 维护一份 `docs/paper-draft/glossary.md` 内含中-英双语 key terms | (b)——key terms ~15 个，提前对齐避免翻译歧义 |
| **DP-F2**：是否在 tool README 顶部**加 `## Citation` 段**（即使非 paper 投稿期）| (a) 不加，paper 草稿期专门收集；(b) 立即加，每个 README +5 行 | (a)——按 CLAUDE.md "tools 集成是次要模块，不抢核心优先级"，paper 草稿期统一收集即可 |
| **DP-F3**：是否在主报告 §3 大表 caption 显式声明"不构成 ranking"（已在 §0.3 / README disclaimer 顶部声明，但表头本身仍 desc-sort）| (a) 保持现状 + 信赖顶部 disclaimer；(b) 改表头为字母序 + caption 重复 disclaimer | (a)——已多处 disclaimer，重复无新增信息 |

### 7.3 不属本审查范围

- README 改写 / cc-report 改写——按 CLAUDE.md §一."文档优先原则"，需先动 principles → architecture → tool-integration 再动 tool README，paper 形态前不必动
- 实际 paper 撰写——超出 testsuite 项目范围

---

## §8 总结（300 token 内）

**核心发现**：

1. **20 工具学术论文引用 100% 缺失**——paper 投稿前必须补齐 references.bib（venue + DOI / arXiv 估算见 §2.1 表）
2. **"0 误报 / 0 漏报 形式可证"是 reviewer 必挑的 strong claim**——须降级为 "single-channel exit-code invariant argument"
3. **15 个项目自创术语** paper 引入前须定义：top 5 = 前端 / 宽度-深度切割 / 0 误报形式可证 / 三原则栈 / 工具菜
4. **paper 结构 blocker**：缺 Threats to Validity 专章 + 缺 Related Work 专章
5. **数据呈现**：4 位百分比 → round + raw count；desc-sort 表加 "not a ranking" caption

**R5 / R11 / R12 核校**：R5 全过（自家文档可用自创词）；R11 部分不过（paper 须 cite 上游）；R12 部分不过（paper 须 round）。

**真决策点 3 个**（DP-F1 / F2 / F3，见 §7.2）——其中 DP-F1 推荐 (b) 维护 glossary 草稿，其余维持现状。

**本审查不修改任何 design / tool 文档**——按 R5，所有发现均为 paper 草稿阶段 future-work。
