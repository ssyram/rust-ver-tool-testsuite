# Publish-readiness — Audit Methodology + Checklist + Status

> 本文是项目**额外的质量保证层**。区别于：
>
> - `docs/design/principles.md` —— 宪法（What we must satisfy）
> - `docs/design/architecture.md` —— 架构（How we satisfy it）
> - `docs/audit/v6-law-*.md` —— 法律 audit（宪法 → 实施一致性）
> - `deep-reports/cc-reports/*` —— per-tool 实测 audit
>
> 本文加一层 **publish-readiness audit**：把项目状态对照 **学术发表标准**（ISSTA / ACM Artifact Badges / Wohlin Empirical SE）做 6-axis disprove-first 评估，记录方法学 + 当前快照 + 剩余 packaging gaps。Standing framework — 任何想投会（不限 ISSTA）时都可重用。
>
> **基础宪法**：`principles.md` §六 当前 crate 焦点 + 工具输出形态对称性 + 时空锚定 + 不构成长期承诺 + bug-detect=SUCCESS（架构 §一 派生）
>
> **审查协议**：宪法 §八 c+cc disprove-first（先反挑刺 + 后反 counter-challenge）

---

## §0 这层文档为什么存在

宪法层 + 法律层 + 实施层的 audit 已经回答："**项目内部是否自洽**"。但项目最终需要面对外部观察者——同行评审 / 论文 reviewer / 工具开发者 / 工业实践者。外部观察者的 standard **不来自宪法**——来自学术规范（empirical study 教科书 + ACM Artifact 标准 + venue-specific 要求）。

如果只走宪法层 audit，会发现：

- 项目自洽 ✓
- 但 reviewer "看不到对称性论证"（虽然项目内部已经对称）
- 但 reviewer "看不到 Threats to Validity 章节"（虽然项目散落多处声明）
- 但 reviewer "看不到 LICENSE / archive DOI"（虽然项目内容质量很高）

这一层 audit 的核心 reframe：

> **观察者视角 ≠ 设计者视角**——必须主动用观察者标尺测一遍。

---

## §1 Methodology — 6-Axis disprove-first 框架

按 **Wohlin et al. 2012 Experimentation in Software Engineering** Ch.8 + **ACM Artifact Review and Badging v1.1**：

| Axis | Standard | 检查什么 |
| --- | --- | --- |
| **A — Reproducibility** | ACM Available / Functional / Reusable | DOI / LICENSE / install instructions / 别人能跑出同结果 / schema 文档化够扩展 |
| **B — Construct Validity** | Wohlin Ch.8.5 | claim 是否精确测量了它声称的概念（"feature coverage" = 什么？切点对称？）|
| **C — Internal Validity** | Wohlin Ch.8.6 | 因果归属是否被混杂变量污染（harness instrumentation / oracle 切点选择 / corpus 污染）|
| **D — External Validity** | Wohlin Ch.8.7 | 结果能否 generalize（跨工具比较公平性 / 通过率 ranking 暗示 / 样本代表性）|
| **E — Conclusion Validity + ToV section** | Wohlin Ch.8.4 + SIGSOFT | 统计学严谨 + 是否有专章 Threats to Validity / Wilson CI / 多次跑确定性 |
| **F — Terminology + Citations** | ACM camera-ready style | 术语精确 / 上游论文 cite 齐全 / 项目自创术语 paper 引入前定义 |

### §1.1 c+cc 双层（disprove-first）

按宪法 §八：

1. **c 路（挑刺）**：每 axis 派 1 个 reviewer-style agent，按 venue 标准（ISSTA / Wohlin / ACM）找问题，**先不 self-counter**。
2. **cc 路（反质疑）**：对 c 路每个 finding 找推翻理由——
   - 项目宪法 / architecture 是否已 deliberate 选择？
   - reviewer 是否 misread 工具 nature / 切点设计？
   - 是否纯学术惯性套用（如把 multi-run 套到确定性程序）？
3. **只有 c+cc 都站得住的 finding 才落地修订**。
4. **审查 axis ROI**：见 `feedback_audit_axis_roi.md`——审"精神层" axis precision 8-30% / 审"传导链 + 学术规范" axis 67-80%。投投 c 路精力**按 axis ROI 调整**。

### §1.2 reviewer 经常 misread 的项目特点（cc-route 反击点）

由 v6 P31 法律 audit + P40 ISSTA audit 累积经验：

| 项目 deliberate 特点 | reviewer 易 misread | cc 反驳 |
| --- | --- | --- |
| **工具切前端层**（kani --only-codegen / verus --no-verify / etc.）| "你们没真测 verifier 求解能力" | 设计选择：为让所有工具在 feature-coverage layer 可比；solver 性能不属 measurement scope |
| **MIRI/soteria bug detect = SUCCESS** | "对 verifier 不公平" | 工具输出形态对称：MIRI 是 abstract interpreter 无 verifier 切点，bug-detect 是其有效输出形态之一；与 verifier deliberate 切前端对称 |
| **single-run** | "需要 multi-run validation" | 工具全确定性（no LLM, no RNG），multi-run = 浪费算力；唯一非确定性 ror 已用 N=7 处理 |
| **vendored crates corpus** | "vendor 自带 lint 影响数字" | P33 [env] schema 治源（RUSTFLAGS=--cap-lints=warn），不动 vendor 源；符合宪法 §四 A "信号文件不改 cargo 字节级" |
| **通过率排序** | "暗示工具好坏 ranking" | §3 表按家族分组（非按 pass rate desc），caption 明示不可比较性 |

### §1.3 venue-specific 调整

| Venue | 调整点 |
| --- | --- |
| **ISSTA / ASE / FSE** (软工顶会) | 主放 Empirical Study / Artifact track；强调 Threats to Validity + reproducibility |
| **POPL / PLDI / OOPSLA** (PL 顶会) | 主放 Tool / Experience paper；强调 formal definitions + 工具 cite 完整 |
| **CAV / TACAS** (验证顶会) | 主放 Tool Demo / Benchmark；强调 soundness/completeness 论证 + SV-COMP-style comparison |
| **ICSE SEIP** (industry track) | 强调 practitioner-facing + industrial 工业 crate 实测 |

---

## §2 Current Status Snapshot — v6 final (post-P41)

### §2.1 三层 readiness

| 层 | 完成度 | 评估 |
| --- | ---: | --- |
| **Content readiness** | **~97%** | 学术内容支撑充分。宪法 §一 + §六 + architecture §一 派生 + Wilson CI + McNemar + §11 ToV (Wohlin 4 类) + Glossary + tool-citations + cc-rebuttal 全在 |
| **Artifact infrastructure** | **~90%** | LICENSE + README submodule init + .env.example sync + runner 改 raw `${TS_*}` form + hostname=None + 0 private paths 残留。剩 Zenodo DOI + cross-machine validation |
| **Paper draft** | **~70%** | 中文 draft (522 行 8 章 + ToV + Refs)。剩英文翻译 + LaTeX + 20 per-tool capsules 补全 + BibTeX 实化 |

### §2.2 当前 commit 链（publish-readiness 相关）

```
e727b27 P41: artifact 实际 readiness 修订 — 强措辞同步 + private path anonymization
4287c32 P40: paper draft (中文 ISSTA-style 522 行)
936311b P39: glossary + tool-citations + §3 主表重设计 + Wilson CI + McNemar + §10/11
38fe549 P38: ISSTA publish-readiness 档 1 工程修 + 修宪明示对称性 + cc-rebuttal
138d4ee P31: v6 全面法律 audit (跨层一致性)
28b4a03 P27: 修宪 §一 双根本问题
```

### §2.3 Open gaps（剩余 packaging）

| Gap | 性质 | 工作量 | 何时做 |
| --- | --- | --- | --- |
| Zenodo / FigShare archive DOI | ACM Available 长期保存 | 15 分钟 | 投稿前 |
| Cross-machine reproduction 验证 | ACM Functional 一票否决项 | 半天-一天 | 投稿前 |
| paper.md → 英文 + LaTeX (venue template) | 必做 | 1-2 天 | 投稿前 |
| 20 per-tool capsule 补全 in paper §4.5 | camera-ready 前 | 半天 | camera-ready 阶段 |
| BibTeX 实化 + DOI verify（按 `tool-citations.md` 待确认列）| camera-ready 前 | 半小时 | camera-ready 阶段 |
| Per-feature roll-up table（可选 appendix）| 视 reviewer 反馈 | 半天 | 看 reviewer 提 |
| Anonymized fork（去 commit 中 ssyram 信息）| Double-blind 要求 | 半小时 | 投稿前 |

---

## §3 Checklist（投稿前一项一项勾）

### §3.1 ACM Available

- [ ] LICENSE 文件 (`LICENSE`) — ✅ MIT OR Apache-2.0 已加
- [ ] Cargo.toml license 字段 — ✅ runner/Cargo.toml `license = "MIT OR Apache-2.0"`
- [ ] README license 段 — ✅
- [ ] Vendored crates license 注 — ✅ LICENSE 文件末尾说明
- [ ] **Zenodo / FigShare DOI** — ❌ **TODO 投稿前 15 分钟**
- [ ] Repository public + clone-able — ✅ (assuming user push)
- [ ] All `git submodule update --init` clean — ✅ verified P22+

### §3.2 ACM Functional

- [ ] `cargo build -p runner --release` clean checkout 通过 — ✅ 实测 1.3s exit 0
- [ ] README 初次 clone 流程文档 — ✅ P38 加 git submodule init 命令
- [ ] `.env.example` 与 tool.toml 引用一致 — ✅ P38 sync + P41 删 stale TS_ROCQ_OF_RUST_BIN
- [ ] runner CLI 可用（`--tool` / `--entry` / `report`）— ✅
- [ ] 20 工具 README 含 install 说明 + 上游 link — ✅（但 reviewer 装齐 20 工具仍困难，acknowledged limitation）
- [ ] **Cross-machine reproduction** — ❌ **TODO 投稿前半天-一天**
- [ ] Docker / Nix env (optional, 减少 reviewer 装齐 20 工具难度) — ❌ 未做（acknowledged limitation）

### §3.3 ACM Reusable

- [ ] hirusttest schema 文档化（detailed-design.md §一）— ✅ 含 `[runnable.*]` + `[env]` 扩展
- [ ] tool.toml schema 文档化（detailed-design.md §一 §五）— ✅
- [ ] harness.rs.tera 模板规约 — ✅ 3 个 Tera 变量明示
- [ ] Wrapper.sh 写作惯例 — ✅ tool-integration.md §四
- [ ] 添加新工具 step-by-step tutorial — ❌ **TODO**（README 现"按现有 tool 模仿"，可补 walk-through）
- [ ] 添加新 entry step-by-step — ✅ README "加一个新 example" 段
- [ ] results.json schema 完整文档化 — ✅ detailed-design.md §六（含 P41 anonymized 字段）
- [ ] **results.json 0 private paths** — ✅ P41 sed-anonymized + runner code 改用 raw `${TS_*}` form
- [ ] hostname 不持久化 — ✅ P41 改为 None

### §3.4 Construct Validity（Wohlin Ch.8.5）

- [ ] "feature coverage" construct 定义清晰（论文 §1 / §3）— ✅ paper draft §3
- [ ] "前端测量"边界跨工具一致（§六 deep cut + §六 width cut + §六 对称性）— ✅ P34 + P38
- [ ] 强措辞同步降级（"形式可证" → 实测 + 源码层论证 / by-design no-silent-skip）— ✅ P41 13 处全降
- [ ] 项目自创术语 paper 引入前定义（glossary 引用）— ✅ docs/publish/glossary.md
- [ ] bug-detect=SUCCESS 派生论证 + 对称性 — ✅ architecture §一 + cc-rebuttal

### §3.5 Internal Validity（Wohlin Ch.8.6）

- [ ] harness instrumentation 不为单工具偏 — ✅ §四 A 形式定义 + 每工具按上游惯例
- [ ] oracle 切点选择 deliberate 明示（每工具 README + cc-report）— ✅
- [ ] `[env]` schema 行为透明（hirusttest 不改 cargo 字节级）— ✅ P33 + detailed-design 文档化
- [ ] extra_cargo_deps 行为透明 — ✅ tool-integration.md
- [ ] Cargo.lock skip 行为说明 — ✅ detailed-design.md §四
- [ ] **Threats to Validity §11.2 包含全部 T-Ix** — ✅ 5 条

### §3.6 External Validity（Wohlin Ch.8.7）

- [ ] §3 主表按家族（非 pass-rate）排序 — ✅ P39 重设计
- [ ] §3 表加 Measurement boundary 列 — ✅
- [ ] §3 表加 Wrapper status 列 — ✅
- [ ] Pass-rate 不暗示 ranking（caption + footnote）— ✅
- [ ] 跨家族比较 caveat 明示 — ✅ §3.3 verifast caveat + §6 Discussion
- [ ] Corpus generalize 限制声明 — ✅ §11.3 T-E3
- [ ] **Threats to Validity §11.3 包含全部 T-Ex** — ✅ 5 条

### §3.7 Conclusion Validity（Wohlin Ch.8.4）

- [ ] Wilson 95% 二项 CI（per-tool）— ✅ P39 §3 全 20 工具
- [ ] McNemar exact test（同族对比）— ✅ aeneas × 4 / hax × 3 / charon × 2
- [ ] Single-run determinism 论证（why no multi-run）— ✅ §11.4 T-V1
- [ ] Sample size 论证（n=161）— ✅ §11.4 T-V4
- [ ] **§11 专门 Threats to Validity 章节** — ✅ Wohlin 4 类全

### §3.8 Terminology + Citations

- [ ] Glossary 文件 — ✅ docs/publish/glossary.md
- [ ] 20 工具 cite list — ✅ docs/publish/tool-citations.md（11 distinct + verify list）
- [ ] BibTeX 实化（camera-ready 前 DOI verify）— ❌ **TODO camera-ready**
- [ ] "工具菜"等口语清理 — ✅ P38 替换为 "capability boundary reached"
- [ ] Per-tool capsule 在 paper 中 — ⚠️ paper draft §4.5 有 4 条 illustrative，**TODO 补全 20 个**

### §3.9 Pre-submission packaging

- [ ] paper.md 翻译英文 — ❌ **TODO 1-2 天**
- [ ] paper LaTeX (按 venue template) — ❌ **TODO**
- [ ] 匿名化 commit history（去 ssyram user info）— ❌ **TODO 半小时**
- [ ] 选 venue + check deadline / page limit — ❌ **TODO**
- [ ] artifact 包（含 anonymized v6 final run）一并上 Zenodo — ❌ **TODO**

---

## §4 Re-audit Triggers

什么情况下重跑 publish-readiness audit？

### §4.1 Trigger A — 准备投另一个 venue
- 不同 venue 标准不同（ISSTA artifact eval vs PLDI tool track vs CAV bench）
- 跑一遍 axis-by-axis 用新 venue checklist 验

### §4.2 Trigger B — 大型设计变化
- 宪法修订（如 P27 双根本问题 / P34 §六 当前 crate 焦点 / P35 bug detect）
- corpus 大改（如加 100 entries / 引入新 feature 类目）
- 新工具集成（21 个工具 + 起）
- runner schema 改动（如未来添加 [env] 类的新字段）

新增内容应 cross-axis 重审，特别是 Construct + External Validity（reviewer 看不到的隐式变化）。

### §4.3 Trigger C — Reviewer feedback after submission

收到 reviewer 意见后：

- 区分 review finding 是 **不成立 (cc-route 反驳)** vs **成立**
- 成立的 → revision letter + 项目修
- 不成立的 → revision letter 引项目 cc-rebuttal 文档反驳
- 不要 over-revise（charter-craft §4.8.1 实用主义：reviewer 单边反驳 precision 1/3）

### §4.4 Trigger D — Periodic refresh

至少每年一次：

- ACM Artifact 标准更新（v1.x → 新版）
- 上游工具大版本变化（如 kani 2.x / verus 1.0 release / 等）
- venue 大版本变化（ISSTA submission guideline 更新）

---

## §5 Audit Trail

完整 audit 历史（截至 P41）：

| commit | 内容 |
| --- | --- |
| `e727b27` P41 | artifact 实际 readiness 修订 — 13 处"形式可证"同步降级 + private path anonymization + runner code 改 raw `${TS_*}` |
| `4287c32` P40 | paper draft (中文 ISSTA-style 522 行 8 章 + ToV + Refs) |
| `936311b` P39 | glossary + tool-citations + §3 主表重设计 + Wilson CI + McNemar + §10 Related Work + §11 ToV |
| `38fe549` P38 | LICENSE + README submodule init + 修宪 §六 对称性 + arch §一 P35 论证 + cc-rebuttal |
| `138d4ee` P31 | v6 全面法律 audit (5 c + 4 cc) — 跨层一致性 |
| `28b4a03` P27 | 修宪 §一 双根本问题 |

### §5.1 Audit 中间产物（按 axis 索引）

- `docs/audit/v6-issta-c-axis-{A,B,C,D,E,F}-*.md` × 6 — ISSTA 角度 c 路 findings
- `docs/audit/v6-issta-cc-rebuttal-2026-05-12.md` — cc 路反驳 C+E findings
- `docs/audit/v6-law-c-axis-{A,B,C,D,E}-*.md` × 5 — 法律层 c 路 findings (P31)
- `docs/audit/v6-law-cc-axis-{A,B,C,D}-*-counter-*.md` × 4 — 法律层 cc 路
- `docs/audit/v6-law-audit-2026-05-12-final.md` — 法律层综合
- `docs/audit/v6-cc-report-rewrite-2026-05-12.md` — cc-report rewrite 综合
- `deep-reports/cc-reports/*.md` × 20 — per-tool 实测 cc audit

### §5.2 Methodology references

- **charter-craft skill**：`~/workspace/ai-tools/prompts/current/charter-craft.md` — 制宪与修宪方法学（c+cc disprove-first §4.8）
- **principle-derivation-v1**：事后整理姿态（用于 audit 报告写作）
- **principle-derivation-v2**：邀请展开姿态（不用于 audit，用于教学）
- **workflow-audit**：multi-direction PR audit 方法学（本项目 adapt 用于学术 publish-readiness audit）

---

## §6 关键洞察 — c+cc 协议在跨学科 audit 中的特殊价值

经过 P31 法律 audit + P40 ISSTA audit 两次大规模 c+cc：

- **单走 c 路会过度套用方法学到 conceptually 不匹配场景**（如 reviewer E 把 empirical study multi-run 套用确定性 build pipeline）
- **cc 路 precision 在跨学科审查中尤其高**——reviewer 用学科默认套路看项目时，常 misread 项目的 deliberate design choice
- **cc 路反驳 reviewer 时必须诉诸**：(a) 宪法明示精神 (b) 工具 nature 事实 (c) 项目历史 deliberate 选择记录
- **审 axis ROI 不对称**：cc-A (宪法层) precision ~8% / cc-D (传导链) precision ~67% / cc-E (统计学规范) precision ~50%——下次大 audit 据此分配精力

---

## §7 Long-term Quality Assurance Layer Map

本项目质量保证目前由四层叠加（见 §0 顶部）：

```
┌─────────────────────────────────────────────────────────┐
│  L4: publish-readiness 层 (本文档)                       │ ← venue / 外部观察者标准
├─────────────────────────────────────────────────────────┤
│  L3: 法律 audit 层 (docs/audit/v6-law-*)                 │ ← 宪法 → 实施一致性
├─────────────────────────────────────────────────────────┤
│  L2: per-tool 实测 audit 层 (deep-reports/cc-reports/)   │ ← 每工具 oracle 严谨性
├─────────────────────────────────────────────────────────┤
│  L1: 宪法 / 架构 / 实施 (docs/design/ + runner + tools)  │ ← 内部 spec
└─────────────────────────────────────────────────────────┘
```

每层独立运行 + 互补：
- L1 故障 → L2 (cc-report) 先发现
- L2-L1 不一致 → L3 (法律 audit) 后发现
- L3-L1 都自洽但仍可能 reviewer reject → L4 (本层) 发现
- L4 也通过 → 可走 submission

每层都按 c+cc disprove-first 协议跑（宪法 §八）——这是项目质量保证体系的**统一 idiom**。
