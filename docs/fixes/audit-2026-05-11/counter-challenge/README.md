# Counter-Challenge 综合 — Audit 自身的反向验证

## §0 元数据

- **Counter-Challenge ID**：`counter-challenge` (audit-2026-05-11 子审)
- **方法论**：disprove-first（默认 audit 错，找证据驳斥；找不到驳斥则 audit 成立）+ evidence-gated
- **范围**：5 个 audit 的高严重度问题 + 修宪议案
- **输出**：5 份 counter + 本 README

## §1 问题意识

5 个 audit agent 给 ~51 个高严重度问题 + 5 修宪议案 + ~93 决策点。审计本身可能：
- 过度推论（把合理设计决策误判为违反）
- 误读 design 意图
- 凑数 / 严重度通胀
- 事实错误
- 修复建议本身有问题

**用 disprove-first 反向验证**。

## §2 5 份 counter 文件索引

| Counter | 验证范围 | 文件 |
|---|---|---|
| Counter-1 | Audit-1 design 4 份 (13 高 + 4 修宪议案) | [`audit-1-counter.md`](audit-1-counter.md) |
| Counter-2 | Audit-2 20 工具 (15 高) | [`audit-2-counter.md`](audit-2-counter.md) |
| Counter-3 | Audit-3 examples (3 高 + 决策点抽样) | [`audit-3-counter.md`](audit-3-counter.md) |
| Counter-4 | Audit-4 reports/research/fixes (15 高) | [`audit-4-counter.md`](audit-4-counter.md) |
| Counter-5 | Audit-5 runner 实现 (5 高 + 3 冲突) | [`audit-5-counter.md`](audit-5-counter.md) |

## §3 5 个 audit 评级

| Audit | 评级 | 主要问题 |
|---|---|---|
| Audit-1 design | **B** | 系统性"过度修宪倾向"，4 修宪议案全部前提不成立 |
| Audit-2 tools | **B+** | 严重度通胀，事实层准确 |
| Audit-3 examples | **B** | 3 高严重度中 2 个事实错（#15 nightly-only / #20 vendor 25-crate） |
| Audit-4 reports | **B+** | 严苛度过度，方法论缺陷"未旁注就判高"未检查同份其他段落 |
| Audit-5 runner | **A-** | 最稳一份，8 条全成立或部分，0 错 |

## §4 各 audit "真正成立 vs 被驳" 分布

### Audit-1（13 高 + 4 修宪议案）

| 类别 | 成立 | 部分 | 不成立 / 事实错 |
|---|---:|---:|---:|
| 13 高严重 | 8 | 5 | 0 |
| 4 修宪议案 | **0** | 0 | **4（前提不成立）** |

**关键**：修宪议案全部否决，应降级为局部 fix（去 enumerate "19 工具" / 已有约束 / detailed-design 实施层）。

### Audit-2（15 高严重度抽样）

| 项 | 真伪 |
|---|---|
| aeneas wrappers `set -e` 冲突 (4 工具) | ✅ 真（bash 实测 3 次确认 oracle 不漏，仅诊断降级）|
| hax-coq README L22 vs L26 矛盾 | ✅ 真矛盾 |
| verifast README L106 vs L40 矛盾 | ✅ 真矛盾（L106 仍是 P12 前描述）|
| verus README L88 vs L23 矛盾 | ⚠ 部分（"硬矛盾"偏强，实际是 L88 缺限定条件）|
| charon `--target` 平台硬编码 | ✅ 成立但"违反 §七"定性偏强（应是"通用性受限"）|

### Audit-3（3 高严重度）

| 项 | 真伪 |
|---|---|
| §3.4-#18 industrial 违反双轨 | ⚠ **核心成立但有解释空间**（architecture.md §41 "仅 vendor/ 真本体" 留口）|
| §3.4-#20 vendor/sha2 25-crate 副作用 | ❌ **事实错**（cargo metadata 实测仅 11 crate，patches 在 industrial 不生效）|
| §3.3-#15 hax-limit nightly-only | ❌ **事实错**（stable rustc 1.95.0 已稳定 `unchecked_add`，cargo-check 实测 SUCCESS）|

### Audit-4（15 高严重度）

| 项 | 真伪 |
|---|---|
| reports H-1 typecheck baseline 旁注 | ❌ **驳**（typecheck.md:170-180 已解释 110 vs 109）|
| reports H-2 internal-roundup §7.3 自打脸 | ✅ 真 |
| reports H-3 ror "3 次" vs N=7 写作错误 | ✅ **真**（对照 ror-gate6-fix:78 N=7 是稳健阈值）|
| reports H-4/H-5 internal-roundup / v3 报告严重落后 | ✅ 真（具体行号验证）|
| reports H-7 kani "0 hard-reject" 措辞 | ❌ **驳**（cc-reports/kani.md:162 起首词"本次未触达"已锚定）|
| reports H-8 hax-lean silent-skip 漏审 | ✅ 真 |
| research H-1/H-3 "v2 推后未跟进" | ⚠ 部分（严重度夸大）|
| fixes H-3 / H-4 audit 脉络断 + hax-lean 未跟进 | ✅ 真 |

### Audit-5（5 高 + 3 冲突，去重 7 条）

| 项 | 真伪 |
|---|---|
| 5 完整成立 | #9 / #6 / #11 / #7（+ #6 冲突共享）|
| 3 部分成立 | #35 / #16 / #40 |
| 0 不成立 / 事实错 | — |

**关键**：#6 / #11 都有 design 明文支撑（detailed-design §三 / architecture §四 / detailed-design §七 错误处理表），audit-5 命中率极高。

## §5 综合 audit + counter 后的最终判定

之前 audit README §4 列了 16 个"跨层级关键发现"(F-1 ~ F-16)，counter-challenge 后修正如下：

### §5.1 真正成立（需要处理，共 11 条）

| 编号 | 内容 | counter 验证 |
|---|---|---|
| **F-4** | internal-roundup 落后于 P16 数据 | ✅ Counter-4 具体行号验证 |
| **F-5** | v3 测试报告也落后 | ✅ Counter-4 验证 |
| **F-6** | ror cc-report N=7 vs "3 次" 自打脸 | ✅ Counter-4 对照 ror-gate6-fix:78 |
| **F-7** | internal-roundup §7.3 kani-limit 7/7 vs §3 6/7 自打脸 | ✅ |
| **F-9** | hax-lean silent-skip-item 漏审 | ✅ |
| **F-10** | hax-coq / verifast README 内部矛盾 | ✅（verus 改"措辞偏强"） |
| **F-11** | ror-gate6-fix vs rocq-of-rust-typecheck-impl 根因冲突 | ✅ |
| **F-12** | audit 4 次 falsify 记录脉络断 | ✅ |
| **F-13** | runner 3 处 design-impl 冲突 (#6/#11/#7) | ✅ Counter-5 完全成立 |
| **F-14** | industrial 三件套违反双轨 | ⚠ 核心成立但有解释空间（D1 保留高优） |
| **F-15** | aeneas wrappers `set -e` 诊断质量降级 | ✅ Counter-2 bash 实测 |

### §5.2 撤回 / 重新定性（4 条）

| 编号 | 内容 | counter 判定 |
|---|---|---|
| **F-1** | "19 工具"硬编码漂移 | 🔄 修复方向反 — 应**去 enumerate**（结构修复），不补 1 行 |
| **F-3** | P11-P16 实施物在 design 主线缺失 | 🔄 部分误读 — design 分层故意写 / detailed-design 已写部分 |
| **F-2** | ror 道门数 5 vs 6 | ✅ 真（统一为 6，非决策点）|
| **F-8** | cc-reports/rocq-of-rust v4 vs typecheck baseline 差 1 | ❌ 撤回（typecheck.md:170-180 已解释，audit-4 H-1 被驳）|

### §5.3 Audit 事实错（2 条撤回）

| 编号 | 内容 | counter 判定 |
|---|---|---|
| Audit-3 #15 hax-limit nightly-only | ❌ **事实错**（stable rustc 1.95.0 已稳定 `unchecked_add`，cargo-check 实测 SUCCESS）|
| Audit-3 #20 vendor/sha2 25-crate 副作用 | ❌ **事实错**（cargo metadata 实测仅 11 crate）|

### §5.4 措辞 / 定性偏强（保留但降级）

| 项 | counter 调整 |
|---|---|
| Audit-2 verus L88 矛盾 | "硬矛盾" → "措辞缺限定条件" |
| Audit-2 charon `--target` "违反 §七" | → "通用性受限"（非违反）|
| Audit-3 #18 industrial 违反双轨 "宪法级违反" | → "核心成立但有解释空间"（精神派 vs 字面派分歧）|
| Audit-4 H-1 / H-7 | 撤回（已被 Counter-4 驳）|

### §5.5 长期项（不变）

| 编号 | 内容 |
|---|---|
| **F-16** | 多平台 macOS arm64 硬编码 |

## §6 修宪议案最终判定（全部否决）

| 议案 | audit-1 提出 | counter-1 判定 |
|---|---|---|
| **A** | 吸纳 ror-typecheck + runnable corpus + `${TS_*}` 替换为正式宪法条款 | ❌ **否决** — 应降级为局部 fix（去 enumerate "19 工具"）|
| **B** | 原则 A 形式定义明示作用域 | ❌ **否决** — line 142 / 173 现有约束已涵盖 |
| **C** | 三分类入宪法层 | ❌ **否决** — 属 detailed-design 实施层 |
| **D** | TS_* envvar 入宪法接口契约 | ❌ **否决** — `.env.example` 已文档化 strip 规则 |
| **E** | industrial 三件套双轨违反松绑宪法或迁移 | ⚠ 保留作为 F-14 (D1 决策点)|

**结论：不需要修宪**。原 audit README §5 推荐的 5 条修宪议案全部降级为文档同步 / 文字 fix / 局部修复。

## §7 决策路径修订（vs 原 audit README §6）

原 audit README §6 五组决策路径，counter-challenge 修订：

### §7.1 短期修复（非决策点，可批量修）

**保留**所有原 §6.1 项 + 加入：
- ror cc-report N=7 / 3 次写作错误（F-6）
- internal-roundup §7.3 老结论（F-7）
- 3 工具 README 内部矛盾（F-10 hax-coq + verifast；verus 降级）
- aeneas wrappers `set -e` 修（F-15）
- Audit-3 #15 / #20 **撤回** — 不要按 audit 建议改

### §7.2 报告同步

**保留**：重写 v4 系统报告 + 同步 internal-roundup（F-4 / F-5）。

### §7.3 实施层 fix

**保留** + 调整：
- runner 3 处 design-impl 冲突（F-13）— Counter-5 验证 design 有明文，**高优**
- hax-lean silent-skip-item P17 审计（F-9）— 仍保留
- F-11 / F-12 fixes 链补脉络

### §7.4 修宪议案审议

**全部否决**（counter §6）。修复方向调整：
- "19 工具" 去 enumerate（局部 fix，非修宪）
- A 形式定义 / 三分类 / TS_* 契约 — 现有约束已涵盖
- industrial 双轨（F-14）— 单独议案 D1，**精神派 vs 字面派分歧需用户拍板**：是迁移 industrial 到目录轨，还是松绑 architecture §41 字面口

### §7.5 长期项（不变）

- F-16 多平台支持

## §8 整体评价（修正版）

原 audit README §7 整体评价："实施快于文档"——counter-challenge 后修订：

- 实施层（runner / wrapper）**质量高于 audit 估计**（Audit-5 命中率 100% 但 audit-3 命中率仅 33%）
- 主线 design 文档（principles/architecture/tool-integration/detailed-design）**比 audit-1 估计更稳健**（4 修宪议案全前提不成立 → design 没那么糟）
- 真正需要做的事：**报告同步 + 个别文字 fix + hax-lean P17 + runner 3 处实施层小修**
- **不需要修宪**

**修正后真正紧急问题** top 5：
1. F-4 / F-5 重写 v4 系统报告 + 同步 internal-roundup（数据严重过时）
2. F-13 runner 3 处 design-impl 冲突修复（design 已明文要求）
3. F-9 hax-lean silent-skip-item 审计（P13 漏审）
4. F-6 / F-7 文字写作错误 fix
5. F-14 industrial 双轨问题用户拍板（精神派 vs 字面派）

## §9 方法学反思（meta-meta）

**为什么 audit 自身有错？**

- Audit-1：把"宪法 → 架构 → 细化"层级误读为"任何实施细节都该入宪"
- Audit-3：版本性 / cargo resolve 类判断未现场实测，依赖训练时知识
- Audit-4：未检查"同份不同段落 / 起首词是否已锚定语义"就判高
- Audit-2：严重度标签语义混（"高" = 真违反 / 也是"关键设计"）

**Counter-Challenge 的价值**：
- disprove-first 系统性过滤"过度推论"
- 现场实测（Counter-2 跑 bash / Counter-3 跑 cargo metadata / Counter-5 构造 entries=[]）独立证伪
- 把 audit 的"看似很多问题"过滤为"真正要做的事"

**audit + counter 的最终输出**：~290+ 原始问题 → ~50 真正成立 → ~15 紧急 → ~3-5 用户拍板。
