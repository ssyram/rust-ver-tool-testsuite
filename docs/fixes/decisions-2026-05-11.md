# 决策点累积文档（2026-05-11，全自动 audit 流程产出）

> **整理版本**：R7（2026-05-12 晚 P27 落地后）。R3 + R5-cc 累积内容完整保留；新增 §0 / §0.5 阅读导航。**§0.0 落地总览**（2026-05-12 加入）记录 P27 修宪后哪些已 resolve、哪些仍待。

---

## §0.0 P27 落地总览（2026-05-12 / commit 28b4a03）

用户在 2026-05-12 提出 **「不引发纠纷 / 遵循社区惯例」** 作为根本问题意识，与原"不公平"并列为 §一 双根本问题。同时引入"本地性 / 当前性"原则凌驾于"社区惯例"之上：

> 测当下这个工具版本 + 它要求的 toolchain 能做什么。装对其要求的 toolchain 后仍不行 → 工具菜，FAILED 站得住。
>
> wrapper 是我们自己包装的 → 我们的问题（UNKNOWN）。
> 对面官方的 wrapper → 官方的锅（FAILED）。

按此新原则栈对所有未决策项重判：

### ✅ 已 resolve（不再是决策点）

| ID | 落地结果 | 提交 |
|---|---|---|
| **D1**（toolchain pinning，4 case）| FAILED 保持。工具自选老 toolchain = 其能力边界，本地性原则下 FAILED 站得住，工具开发者不能驳回 | P27 |
| **D2**（kmir wrapper，25 case）| FAILED 保持。kmir `cargo.py` 是 K Framework **官方** wrapper，官方代码失败 = 工具锅 | P27 |
| **D3.1**（ror "is not yet supported"，2 case）| 加 ror wrapper grep gate → FAILED | P27 |
| **D3.2**（aeneas inline-asm，4 case）| 加 4 个 aeneas wrapper charon stderr grep "is not supported" → FAILED | P27 |
| **D3.3**（aeneas Type error，3 case）| 加 4 个 aeneas wrapper charon stderr "^error:" 检测 → FAILED | P27 |
| **DP-4**（UNKNOWN schema first-class）| 删 oracle 规则 2/3/6（工具能力边界）+ 保留 1/4/5（我们这边问题）；UNKNOWN 严格语义入 §六 | P27 |
| **DP-5**（external_fault 独立字段）| moot：UNKNOWN 严格后只剩 3 类我们问题，独立字段无必要 | — |
| **DP-3**（oracle 规则上限）| moot：DP-4 严格后规则集稳定 | — |
| **DP-6**（规则 tool.toml 化）| moot：规则集稳定且数量少 | — |

### ⏸️ 仍待决策（数量很少）

| ID | 性质 | 操作建议 |
|---|---|---|
| **D3.4-3.6**（README "已知漏报盲点"补 3 类）| 文档补完，纯安全 | 直接补，无需用户裁决 |
| **DP-1**（principles 工具能力评估细则下沉 tool-integration）| 用户已表态接受 | 直接执行 |
| **DP-8**（detailed-design "runnable zero-arg" 措辞已失效）| 文档同步 | 直接执行 |
| **DP-9**（docs/design 文档章节号一致性）| 文档同步 | 直接执行 |
| **DP-11**（prusti README edition-2024 明示）| 文档补完 | 直接执行 |

### 🌫 长期，不抢核心承诺（按 CLAUDE.md §三）

- **DP-7** hax-lean-eval（P16-impl-B）— 次要模块
- **DP-10** vendor x509-parser lint 修源 — 改 vendored crate 破坏可复现性，不推
- **DP-12** Coq 8.19 + F* 装好后跑 typecheck — 次要模块本机依赖

### 总结

P27 落地后 **没有真正需要用户裁决的剩余决策点**。所有"决策"已派生自宪法 §一 双根本问题 + 本地性原则。下面 §1-§3.5 原始条目保留作为审计追溯，但**操作上以 §0.0 为准**。

---

## §0 用户回来阅读建议

### 0.1 5 分钟扫读路径

1. 读 §0.5 **决策点目录索引**（本表）—— 看一眼总数与优先级
2. 直接跳 §4 **操作可逆性 / 紧急度总表** —— 一张表看完所有 20 项决策点的 case 数、可逆性、紧急度
3. 想详看某条 → 跳到对应 §1 / §2 / §3 / §3.5 条目

### 0.2 30 分钟拍板路径

按推荐顺序读：§3.5 (D3.1 / D3.2 / D3.3 真漏报，高优先) → §1 (D1 / D2 候选误报，中优先) → §2 (DP-* 早期累积，低优先)。每条建议一次性裁决（不需要回拆细分类），decisions 标记为"已裁决"即可。

### 0.3 决策风格建议

- **真漏报（D3.1-D3.3）：先 review 修复方案 A / B 的反误报论证**——本会话中 cc 阶段已做"v5 corpus 实测"双向验证，但建议用户判定是否在更大 corpus（v6 扩 entry 后）再次双向 audit 后落地
- **候选误报（D1 / D2）**：精神模糊 — 用户拍板"toolchain pinning / wrapper 脆弱算外部根因吗？"即可推 R6 后续 fix agent 落地（按 §3.2.1 / §3.2.2 给出的 2 个立场二选一）
- **README 补完（D3.4-3.6）**：纯文档补丁，无技术风险，建议批准
- **DP-* 早期累积**：低紧急度，可推迟

---

## §0.5 决策点目录索引

| ID | 简述 | case | 来源 | 类别 | 紧急度 | §位置 |
|---|---|---:|---|---|---|---|
| **D1** | toolchain pinning 副作用是否算外部根因 | 4 | R2-cc | 候选误报 | 中 | §1.D1 |
| **D2** | 工具自带 wrapper 脆弱是否算外部根因 | 25 | R2-cc | 候选误报 | 中 | §1.D2 |
| **D3.1** | ror × 2 raw_ptr_const_match silent Pattern::Wild | 2 | R5-cc | 真漏报 | **高** | §3.5.D3.1 |
| **D3.2** | aeneas × 4 inline-asm charon stage 降级 opaque | 4 | R5-cc | 真漏报 | **高** | §3.5.D3.2 |
| **D3.3** | aeneas × 3 copy-deref-closure charon `error:` 但 exit 0 | 3 | R5-cc | 真漏报 | **高** | §3.5.D3.3 |
| D3.4 | kani README §漏报盲点补 concurrency 类 | 0 | R5-cc | 文档补完 | 中 | §3.5.D3.4 |
| D3.5 | soteria README §漏报盲点补 intrinsic 类 | 0 | R5-cc | 文档补完 | 中 | §3.5.D3.5 |
| D3.6 | verus README §漏报盲点补 derived auto-spec | 0 | R5-cc | 文档补完 | 中 | §3.5.D3.6 |
| DP-1 | principles 内容下沉 detailed-design / tool-integration | 0 | Agent X | 架构 | 低 | §2.DP-1 |
| DP-2 | 反误报方法论独立 first-class | 0 | Agent X | 架构 | 低 | §2.DP-2 |
| DP-3 | oracle 规则上限设计（N ≤ 10？双向实测程序约束？） | 0 | Agent X | 架构 | 低 | §2.DP-3 |
| DP-4 | UNKNOWN schema 升 first-class | 0 | Agent YZ | schema | 低 | §2.DP-4 |
| DP-5 | external_fault 独立字段 vs error 前缀 | 0 | Agent YZ | schema | 低 | §2.DP-5 |
| DP-6 | 规则声明 tool.toml vs hard-coded 中心化 | 0 | Agent YZ | schema | 低 | §2.DP-6 |
| DP-7 | hax-lean-eval tool 启动（P16-impl-B） | 新 entry | 外围 | 工具新增 | 低 | §2.DP-7 |
| DP-8 | detailed-design "runnable zero-arg" 措辞已失效 | 0 | 外围 | 文档 | 低 | §2.DP-8 |
| DP-9 | docs/design 文档章节号一致性（principles v7 重构后） | 0 | 外围 | 文档 | 中 | §2.DP-9 |
| DP-10 | vendor x509-parser lint 修源 | -2 UNKNOWN | 外围 | 源码 | 低 | §2.DP-10 |
| DP-11 | prusti README edition-2024 明示 | 0 | 外围 | 文档 | 低 | §2.DP-11 |
| DP-12 | 装 Coq 8.19 + F* 让 hax-coq / hax-fstar 跑后端 typecheck | 0 | 外围 | 工具增强 | 低 | §2.DP-12 |

**统计**：

- 高优先决策点：3 项（D3.1 / D3.2 / D3.3 真漏报，9 case）
- 中优先决策点：6 项（D1 / D2 候选误报 / D3.4-6 文档补完 / DP-9 文档一致性）
- 低优先决策点：11 项（DP-1 ~ DP-12 除 DP-9）
- 总：**20 项 / ≈ 38 case 影响**

---

## §1 R2-cc 候选误报决策点（精神模糊待裁决）

来源：[`audit-v5-cc-counter-challenge-2026-05-11.md`](audit-v5-cc-counter-challenge-2026-05-11.md) §3.2

### D1：toolchain pinning 副作用是否算外部根因？

**背景**：当 entry 使用的 Rust feature 在 mainline rustc 已 stable（cargo-check SUCCESS），但工具锁死的旧 toolchain 仍把它当 unstable feature gate，工具自身 rustc fork 拒绝。

**影响 case 数**：4 个

| 工具 | entry | stderr 摘要 |
|---|---|---|
| prusti | `float/round/float_round` | `error[E0658]: use of unstable library feature 'round_ties_even'` |
| prusti | `hax-limit/unsafe-block/hax_limit_unsafe_block` | `error[E0658]: use of unstable library feature 'unchecked_math'` |
| kmir | `creusot-limit/thread-local-ref/read_thread_local` | `Cargo compilation failed` — kmir cargo 用 pinned toolchain 编不了 thread_local 宏 |
| kmir | `lifetime/thread-local/thread_local_read` | 同上 |

**精神模糊原因**：宪法 §六 没明示区分"工具特定 toolchain pin 缺失"和"工具核心语义拒绝"。
- 立场 A（升 UNKNOWN）：cargo-check SUCCESS 证明 entry 主线合法；§六 时空锚定（工具版本是必然变量）支持这是 toolchain 时空错配
- 立场 B（保 FAILED）：工具就是选择了它的 rustc 版本；这是工具的设计选择；工具明确 reject 就是 reject

> 决策：是精神模糊，不过这里可能需要更多调研：
> - 到底 unstable feature 的语义有没有变化？如果语义一致我们可以认为**事实上支持**，不过是形式上多一个少一个 feature gate 的问题；如果语义不一致（比如 round_ties_even 之前是 round_ties_upward），那 prusti 的 reject 就是合理的。
> - 不过这里的拒绝也有理由：我们在当前这个时间点下这个工具版本的能力是 FAILED ，是完全合理的，这里看看到底最终目标是什么，我们的非误报的本质理由和原因是“降低纠纷”，即我们工具发布的时候不要惹来太多麻烦。从这个角度上来说也许采用上述的语义一致判定的 “最大善意” 角度会更好一些？这个可能得讨论。不过我觉得正经的研究者应该也会接受下面的拒绝理由。

**推荐讨论**：用户决定宪法 §六 是否新增一类外部根因 `toolchain_unstable_feature_gate` / `toolchain_pinning_mismatch`

**可逆性**：高（oracle 规则可加可撤）
**紧急度**：中（影响 4 个 case 的分类，不影响 v5.1 主体）
**影响范围**：prusti FAILED 数 / kmir FAILED 数（小）

**推荐裁决后路径**：
- 选立场 A → 加 oracle 规则 7 + 反误报论证 + 双向实测（按 tool-integration §4.2）
- 选立场 B → 不动 oracle，但在 prusti README + kmir README 补"toolchain pinning 与 mainline feature 错配是已知误报盲点"

---

### D2：工具自带 wrapper 自身脆弱是否算外部根因？

**背景**：kmir Python wrapper 在 `kmir/cargo.py:91` 解析 cargo --message-format=json 输出时，遇到非 JSON 行（warnings / 空行 / build-output）直接抛 JSONDecodeError，或 wrapper 调用内嵌 cargo 时遇到 deps 失败抛"Cargo compilation failed"。kmir 的 K interpreter 完全没机会执行。

**影响 case 数**：25 个

| 子桶 | 数量 | 典型 entry |
|---|---|---|
| 24 JSONDecodeError | 24 | arc/clone-drop, box/basic-alloc, charon-limit/box-branch-init 等简单合法 entry |
| 1 hashmap_basic 根因不明 | 1 | collections/hashmap/hashmap_basic — kmir 内嵌 cargo failed 但 stderr 不足以判定 |

**精神模糊原因**：
- 立场 A（升 UNKNOWN）：wrapper 失败模式与工具核心语义判定无关；wrapper 升级后同 entry 可能 SUCCESS；wrapper 是 harness 上游
- 立场 B（保 FAILED）：用户/runner 看到的是黑盒 kmir CLI；CLI 跑不下来就是工具能力问题；每个工具都可以宣称"我的 Python wrapper 崩了不算我"

> 决策：这个地方比较明确 -- wrapper 必定是用户使用体验的一部分，而且不是一个轻易可解决的 -- 上面的那个说是模糊地带是因为，作为用户，看到了这个提示只不过是加一行启动的问题，非常明确有提示，甚至可以加一个 #[cfg(prusti-testing)] 之类的配合注入的提示就可以的。总之这里可以有一个更高的原则就是看看社区研究者们会否接受这个作为 fail 的理由。wrapper 失败真的就是直接就是产品的问题了吧，这个没得洗了吧。前端管你哪个失败，不是合理理由。

**推荐讨论**：用户决定宪法 §六 是否区分"工具核心语义层 reject" vs "工具 wrapper 层 crash"

**可逆性**：高
**紧急度**：中（影响 kmir 25 个 case 的分类，但只影响 kmir 一个工具）
**影响范围**：kmir 100 FAILED → 75 FAILED（如选 A）

**附注**：hashmap_basic 子裁决——若选 A 一起升 UNKNOWN；若选 B 建议补一次 deep run 用 `RUSTFLAGS=--cap-lints=allow` 或直接调 `kmir/cargo.py` 查清根因。

---

### D1 + D2 合并影响

若用户裁决 D1=A + D2=A：
- kmir 100 FAILED → 75 FAILED，UNKNOWN +25（D2）+ 2（D1）= 27
- 加上本次 R3 已实施的 §3.1.3 kmir 16 子集（如果实施）= 总 kmir UNKNOWN 43

R3 暂未实施 §3.1.3 kmir 16 子集——根据 R2-cc §4.1.3 推荐"R3 不动 kmir 系列，等 D1/D2 决策后再实施"。

---

## §2 早期累积决策点（盘点）

这些决策点来自更早阶段的 audit / 设计讨论。R3 不主动推进，只盘点提醒用户。

### 来自 Agent X（principles 重构 / 内容下沉）

#### DP-1：`principles.md` 中工具能力评估细则下沉到何处？

- 影响：principles.md / detailed-design.md / tool-integration.md 三层之间的章节切割
- 当前状态：principles 重构 v7 已稳定，但 DP-1/DP-2/DP-3 三个细化锚点是否需要进一步下沉到 detailed-design / tool-integration 待定
- 来源：principles 重构会话记忆
- 可逆性：中
- 紧急度：低（不影响 R3 实施）

> 决策：工具能力评估细则放到具体的 tool-integration.md 中更合适一些，principles.md 只保留高层原则，主要是目标性的。细化的设计原则则是派生一级别的**回答性**的，也就是说这里的最高原则本来就是**问题性**的，它是问题意识的解释和回答标准的制定 -- 如果对大型项目一定要细化的话。而回答性的部分则是设计原则，它们本质上是固定作为公理以回应问题，但是是不是唯一？恐怕不是，而是固定具体当前的选择 -- 不过这里要问到底“不得罪人”这个选择是问题性的还是回答性的？这里可能还是有个相对概念的 -- 它的答案可能是别人的问题，也就是要围绕继续细化的问题。也许本质不是谁是谁的回答/谁是谁的问题这么简单，而是原则的要遵守的优先度级别的问题？即冲突时消解到底属于哪个原则级别？总之对这个较大的项目而言，目标问题意识必定是最高级（principles.md 文件里），但是具体不得罪人这个算是某种围绕核心的工具展示的解答，还是一个独立的思考角度？

#### DP-2：tool-integration 中的反误报双向实测论证是否升 first-class 文档？

- 当前散落在 §4.2 内
- 是否独立为 `docs/design/false-positive-audit-methodology.md` 待定
- 可逆性：高
- 紧急度：低

> 决策：有足够的内容吗？当前有什么很好的方法论需要被固定？给个草稿看看？另外，记得从问题意识展开论证，就是这用来干什么的，解决了什么问题？-- 但是另外的是否有足够内容这一点倒无所谓了，本质上只是说有足够内容就可以独立一份，没有的话就放在 tool-integration 里也无所谓，所以是否有足够内容是是否形成单独的文件的理由，但是不是这个文件本身需要描述的内容里面要包含的部分 -- 是否有足够内容是它内容自证的，无须独立论证。

#### DP-3：architecture 中 oracle 分类逻辑的"5 + N 类规则"上限设计

- 当前 5 类 hard-coded（runnable / deps / edition / vendor / env），R3 加 2 类（edition_propagation / E0433 拓宽）变 6 类
- 是否在 architecture 中加"N ≤ 10"或"规则需在 tool-integration §4.2 双向实测后才能进 oracle"的程序性约束
- 可逆性：高
- 紧急度：低

> 决策：这是啥？详细解释。

---

### 来自 Agent YZ（P21 UNKNOWN 分类实现）

来源：[`oracle-unknown-classification-2026-05-11.md`](oracle-unknown-classification-2026-05-11.md)

#### DP-4：UNKNOWN schema 是否升 detailed-design first-class？

- 当前：UNKNOWN 在 results.json 中通过 `error: "external_fault: <tag>"` 字段表达
- 是否独立 `external_fault: Option<String>` 字段（与 `error` 解耦）
- 当前状态：未决策
- 可逆性：中（schema 改动需 v2 兼容）
- 紧急度：低（R3 实施未受阻）

> 决策：这要非常明确有个清晰的点 -- 什么叫 unknown ，它的自身语义精神是什么，每个工具实际上应该形式上是什么？这是**必要前提**。
> 有了这个必要前提也需要进一步论证必要性 -- 我看来 unknown 应该就是可修复的工具链崩溃 -- 连测都没测，全局 unknown 崩溃，事实上就是“工具没有配置成功导致工具没启动”，单个 entry unknown 说不通吧？除非说局部崩溃了，也就是“刚好测它的时候工具掉线了”。如果你说别的原因工具自身崩溃，那就是 fail 没什么好说的。

#### DP-5：external_fault tag 是否 results.json 独立字段？

- 与 DP-4 重复但具体到字段名
- 当前：以字符串前缀混在 `error` 字段
- 推荐 schema：`{"status": "UNKNOWN", "error": null, "external_fault": "dependency_resolution"}`
- 可逆性：中
- 紧急度：低

> 决策：这是啥？为什么有 external_fault ？理论上存在，但是因为我们必须确保。cargo check 通过，也就是 examples 必对。那一切 external_fault 就是工具自身的 fault ，那唯一的 unknown 的理由（我可以想象的，如果你有更好的也行，说服我）是工具链没配置成功，跑不起来。其他有什么理由？

#### DP-6：5 条 hard-coded 规则是否声明在 `tool.toml` 中？

- 当前：5 条规则在 `runner/src/report.rs::classify_external_fault` 写死（R3 后变 6 类）
- 是否每个工具 `tool.toml` 声明 `external_fault_patterns = [...]`，让工具自陈而非 runner 中心化
- 反对论据：跨工具一致性（同一外部根因不应被不同工具描述成不同 tag）+ 防作弊（工具不能自己"赦免"自己的 partial）
- 推荐立场：保持 hard-coded
- 可逆性：低
- 紧急度：低

> 决策：所以 `classify_external_fault` 是啥，到底有什么理由？我感觉是不是有东西我没想象到？

---

### 来自外围 / 其他会话

#### DP-7：hax-lean-eval tool 启动（P16-impl-B）

- 来源：detailed-design 第 16 任务
- 当前状态：未实施
- 内容：在 hax-lean 后端基础上加 verifier-side eval（用 Lean 检查 hax-lean 输出能否 elaborate）
- 优先级：低（次要模块，按 CLAUDE.md §三 不抢核心承诺）

#### DP-8：detailed-design.md "runnable zero-arg FAILED 是已知代价" 已失效

- 来源：Task Y 修复后已治源
- 当前 detailed-design 仍有遗留陈述需更新
- 可逆性：高
- 紧急度：低（不影响实施，只是文档陈词需同步）

#### DP-9：docs/design 文档间一致性更新（principles 重构后引用旧章节号）

- principles v7 章节号变化后，architecture / detailed-design / tool-integration 中的"§N.M"引用可能失效
- 需全文档校对
- 可逆性：高
- 紧急度：中（避免后续 agent 引用错位）

#### DP-10：vendor x509-parser lint 修源？

- 当前 vendor/x509-parser 触发 `unused_qualifications` lint → oracle 规则 4 拦截
- 是否直接修 vendor 源码（移除 unused_qualifications 字面）以让所有走 cargo 的工具不再触发 lint
- 反对论据：动 vendor 破坏可复现性
- 可逆性：中
- 紧急度：低

#### DP-11：prusti README edition-2024 明示

- prusti 锁死 2023-08 toolchain，不识别 edition 2024
- 当前 oracle 规则 3（toolchain_edition_mismatch）正常工作，let-chains entry 已映射 UNKNOWN
- 但 README 是否明示"prusti 不支持 edition-2024 是已知误报盲点"
- 可逆性：高
- 紧急度：低

#### DP-12：装 Coq 8.19 + F* 让 hax-coq / hax-fstar 可跑后端 typecheck

- 当前 hax-coq / hax-fstar 只跑前端翻译，后端打印输出未经 typecheck
- 是否加 Coq 8.19 + F* 后端 typecheck oracle
- 优先级：低（次要模块）
- 可逆性：高（不破坏现有）

---

## §3 推荐处理顺序

由用户决定。本节给出推荐序：

1. **最高优先：D3.1 + D3.2 + D3.3**（R5-cc 真漏报，README "0 漏报形式可证"声明被实测推翻，影响 9 case，**修复方案 cc 阶段已写好双向验证论证**）
2. **次高优先：D1 + D2**（R2-cc 候选误报，最影响下次 matrix 的 case 总数；29 case 未修但已 audit 出）
3. **中：D3.4 + D3.5 + D3.6 + DP-9**（文档补完 / 文档一致性，长期诚实但不影响实施）
4. **再次：DP-1 / DP-2 / DP-3**（架构下沉，影响文档/代码分层的长期清晰度）
5. **再再次：DP-4 / DP-5**（schema 改动，但 R3 无阻碍）
6. **长期：DP-7 / DP-10 / DP-12**（次要模块工作，按 CLAUDE.md §三 不抢核心承诺）
7. **诚实声明类：DP-8 / DP-11**（文档措辞同步，不影响实施）

---

## §3.5 漏报决策点（R5-c + R5-cc 验证后真漏报，2026-05-11 追加）

来源：[`audit-v5-c-false-negative-2026-05-11.md`](audit-v5-c-false-negative-2026-05-11.md) → [`audit-v5-cc-false-negative-counter-2026-05-11.md`](audit-v5-cc-false-negative-counter-2026-05-11.md)

c 阶段 22 候选 → cc 验证 9 真漏报 + 13 驳回（设计选择）+ 3 文档盲点补完。下列 D3.x 留待用户裁决。

### D3.1：rocq-of-rust × 2 silent Pattern::Wild 漏报

**背景**：unsafe-ptr/raw-ptr-const/raw_ptr_const_match 在 ror + ror-typecheck 上 SUCCESS，但 stderr 7/7 attempts 都含 `warning: This kind of constant in patterns is not yet supported.`，源码 [`thir_pattern.rs:165-176`] `emit_warning_with_note + Rc::new(Pattern::Wild)` silent 替换为 wildcard。产物里 `DISGUISED_INT => 1` 被替换为 `_ => 1`，typecheck 通过但语义改变。

**影响 case 数**：2（ror + ror-typecheck，同 entry）

**ror README 自陈 vs 实测**：README §漏报盲点声明 "上游引入新 silent fallback 路径不带已知 markers（**实测在 examples corpus 0 现象**）"，但 v5 corpus 推翻 "0 现象"。

**修复路径建议**：gate 5 markers 加 wrapper grep stderr `"is not yet supported"`；或更精准 `"is not yet supported\."` 字面（避免合法字符串子串）

**修复风险**：
- 已知 silent path 命中（7/7 attempts）✅
- 反误报：v5 corpus 中其他 124 SUCCESS 全不触发 ✅；理论 thir_pattern.rs / thir_expression.rs 还有多处同 prefix emit point（Range/Never/Error/DerefPattern），未来 corpus 扩展可能触发更多 entries —— 不一定是误报，可能是真 silent path
- 关键反向：ror 上游是否在合法路径有 `is not yet supported` 字面 emit —— **本审范围未独立验证**

**可逆性**：高
**优先级**：**高**（README 声明被实测推翻，必须更新 README）

**用户决定**：是否加 marker？如加，是否先在更大 corpus（v5.1）上做反误报双向 audit？

---

### D3.2：aeneas × 4 backends — charon-limit/inline-asm/nop_via_asm 漏报

**背景**：aeneas × 4（coq / fstar / lean / hol4）在 inline-asm entry 上 SUCCESS。pipeline 实际：
1. charon `--preset=aeneas`（[`charon/options.rs:363`] 不设 abort-on-error）把 inline asm 包含的 entry_fn `nop_via_asm` 降级为 **opaque function**
2. charon stage exit 0
3. aeneas 接收 .llbc，`Translated opaque 1/1, transparent 0/0`，exit 0
4. **产物 entry_fn 仅 opaque 声明，无 body**

stderr 字面：`warning: Inline assembly is not supported` + `warning: The extraction generated 1 warnings`。

**影响 case 数**：4（4 个 backends × 1 entry）

**aeneas-* README 自陈 vs 实际**：README §"形式严格性 — 0 漏报" 声明 `✅ 形式可证。aeneas exit 0 ⇔ Errors.error_list 空` —— 但论证只覆盖 aeneas 本体（craise 唯一通路）、**未覆盖 charon 上游 stage**。pipeline-level 0 漏报论证存在缺口。

**修复路径建议**：

- **方案 A**：aeneas wrapper 在 charon 调用后 grep stderr：
  - `warning: The extraction generated [0-9]+ warnings` —— 命中 → FAILED
  - 或 `Inline assembly is not supported`
- **方案 B**：让 wrapper 在 charon 调用追加 `--abort-on-error`（覆盖 Preset::Aeneas 默认）

**修复风险**：
- 方案 A 反误报：v5 raw 中 7 entries 触发 "extraction generated.*warnings"（4 inline-asm + 3 copy-deref-closure，全 aeneas-coq/fstar/lean）—— 无合法 SUCCESS 触发
- 方案 B 反误报：可能伤"charon 半翻 + aeneas 仍翻部分"的真 charon-edge entries；v5 中未观察到

**可逆性**：高
**优先级**：**高**（pipeline-level 0 漏报论证已自陈缺口；aeneas × 4 都受影响）

**用户决定**：A or B？还是只更 README 不动 wrapper？

---

### D3.3：aeneas × 3 backends — charon-limit/copy-deref-closure 漏报

**背景**：aeneas-coq / aeneas-fstar / aeneas-lean 在 copy-deref-closure entry 上 SUCCESS。pipeline 实际：
1. charon stage 内部 `error: Type error after transformations: Found incorrect clause var: Bound(1, 0)` × 3 次（字面是 `error:` 而非 warning）
2. 但因 Preset::Aeneas 不带 abort-on-error，charon stage **exit 0**
3. aeneas 接收 partial .llbc，`Translated transparent functions: 6/6`，exit 0

**比 D3.2 更严重**：charon 自己明确分类为 `error:` 但 exit 不升。

**影响 case 数**：3（aeneas-hol4 在此 entry 已 FAILED）

**修复路径建议**：同 D3.2（方案 A 双向验证更易：`error: Type error after transformations` 字面是 charon 强信号）

**修复风险**：v5 raw 中只有此 1 entry × 3 backends 触发此字面；反误报基本无风险

**可逆性**：高
**优先级**：**高**（charon 自陈 `error:` 仍 exit 0 是 pipeline 严重设计缺陷）

**用户决定**：同 D3.2 — 是否实施 A or B？是否覆盖 charon 上游 Preset 默认？

---

### D3.4：kani README §漏报盲点补完（concurrency 类）

**背景**：v5 raw 中 8 个 SUCCESS entries 含 `Kani currently does not support concurrency. The following constructs will be treated as sequential operations` warning（atomic_*/thread local/fence）。cc 阶段验证：

- kani-compiler 真 codegen 原子操作（atomic_block / SKIP / binop），不是 stub
- "treated as sequential" 是 BMC **单线程语义约束**（不模拟多线程交错），属求解层
- 按 §六-3 前端测量原则 + P13 audit-2 caller_location/foreign function 排除先例：**不抓 marker**，但 README §漏报盲点应明示

**影响 case 数**：0（实测）+ 8 个 entries 当前 SUCCESS 解释力依此调整

**修复路径建议**：kani README §漏报盲点条目 + "concurrency 类 warning 是 kani BMC 单线程语义约束 / 前端 codegen 完成 / 不属前端 partial"

**修复风险**：无（仅 README 补 text）

**可逆性**：高
**优先级**：**中**（不影响 v5 数据 / 不影响 oracle，但长期诚实）

---

### D3.5：soteria README §漏报盲点补完（intrinsic 类）

**背景**：v5 raw 中 4 个 SUCCESS entries 含 `An atomic intrinsic was encountered; it will be executed as sequential code` 或 `A complex floating point intrinsic was encountered; it will be executed with a significant over-approximation`。cc 阶段验证：

- atomic intrinsic 同 kani concurrency：符号执行单线程语义约束，前端完成
- complex float intrinsic over-approximation 是 soundness-preserving abstraction，不属 silent skip
- soteria README 当前声明 "漏报盲点：无" —— 不诚实

**影响 case 数**：0 oracle 改动 + 4 entries 当前 SUCCESS 解释力依此调整

**修复路径建议**：soteria README §漏报盲点 加条目 "atomic intrinsic sequential 替换 / complex float intrinsic over-approximation 是求解层语义近似 / 前端完成"

**修复风险**：无

**可逆性**：高
**优先级**：**中**

---

### D3.6：verus README §漏报盲点补完（derived auto-spec 类）

**背景**：v5 raw 中 aeneas-limit/float-types/make_measurement SUCCESS 含 `continuing, but without adding a specification for the derived Clone impl`。cc 阶段验证：

- "continuing" 表示 VIR 构造完成（前端完成）
- 缺的是 `#[derive(Clone)]` 的 auto-spec 生成 —— 属 verus → SMT 求解的中间步骤
- 按 §六-3 前端测量：spec gen 不属前端，verus README "无盲点"声明应补此条目

**影响 case 数**：0

**修复路径建议**：verus README §漏报盲点 加条目 "derive(Clone) 等部分 derive form 的 auto-spec 生成边界缺失 / VIR 构造完成 / 不影响 --no-verify 下前端测量"

**修复风险**：无

**可逆性**：高
**优先级**：**中**

---

## §4 操作可逆性 / 紧急度总表

| ID | 描述 | case 影响 | 可逆性 | 紧急度 | 影响范围 |
|---|---|---|---|---|---|
| D1 | toolchain pinning 副作用 | 4 | 高 | 中 | prusti / kmir 局部 |
| D2 | wrapper 脆弱 | 25 | 高 | 中 | 仅 kmir |
| **D3.1** | **ror × 2 Pattern::Wild silent skip** | **2** | **高** | **高** | rocq-of-rust + ror-typecheck |
| **D3.2** | **aeneas × 4 charon inline-asm 漏报** | **4** | **高** | **高** | aeneas × 4 backends |
| **D3.3** | **aeneas × 3 charon type error 不升 exit** | **3** | **高** | **高** | aeneas × 3 backends（hol4 已 FAILED） |
| D3.4 | kani README 盲点补 concurrency 类 | 0 | 高 | 中 | kani README |
| D3.5 | soteria README 盲点补 intrinsic 类 | 0 | 高 | 中 | soteria README |
| D3.6 | verus README 盲点补 derived auto-spec | 0 | 高 | 中 | verus README |
| DP-1 | principles 内容下沉 | 0（文档） | 中 | 低 | 全文档 |
| DP-2 | 反误报方法论独立 | 0 | 高 | 低 | tool-integration |
| DP-3 | oracle 规则上限设计 | 0 | 高 | 低 | architecture |
| DP-4 | UNKNOWN schema first-class | 0 | 中 | 低 | results.json schema |
| DP-5 | external_fault 独立字段 | 0 | 中 | 低 | results.json schema |
| DP-6 | 规则声明 tool.toml | 0 | 低 | 低 | 每个工具 tool.toml |
| DP-7 | hax-lean-eval 启动 | +N 个新 entry | 高 | 低 | hax-lean-eval/ 新目录 |
| DP-8 | detailed-design 措辞 | 0 | 高 | 低 | detailed-design.md |
| DP-9 | 文档章节号一致性 | 0 | 高 | 中 | 全 docs/design |
| DP-10 | vendor x509-parser 修源 | -2 个 UNKNOWN？ | 中 | 低 | vendor/x509-parser |
| DP-11 | prusti README 明示 | 0 | 高 | 低 | prusti README |
| DP-12 | Coq + F* 后端 typecheck | 提升 hax oracle 精度 | 高 | 低 | hax × 2 wrapper |

---

## §5 备注

- 本文件 R3 创建，R5-cc 累积 D3.* 漏报决策，R6 整理目录索引与阅读导航
- 未来累积决策点的 agent 应**追加而不擅自结案**
- 用户在裁决某条后，建议在该条标记 `[已裁决 → 立场 X，由 Agent <name> 实施于 <date>]` 而保留历史（按时间累积，便于后续 review）
- 决策点的"精神模糊"判定遵循 charter-craft §4.8.3：c 阶段挑刺 + cc counter-challenge 后仍不能从宪法 / 下游设计中直接溯源
- D1 / D2 已经过 R2-cc 阶段双 agent 反复 counter-challenge（c 阶段 64 candidate + cc 35 非决策 / 29 决策切分），裁决质量高
- D3.1 / D3.2 / D3.3 已经过 R5-cc 阶段独立 agent 验证（22 候选 → 9 真漏报，13 驳回率 59%，符合 disprove-first 经济性）

---

## §X 如何 review 决策点（短指南）

按 [`principles.md`](../design/principles.md) §八 审查协议 + [`charter-craft`](https://) §4.8.3 决策点判据：

### X.1 每条决策点的 review 流程

1. **读决策点描述**（§1 / §2 / §3.5 各条）
2. **判精神是否真模糊**：
   - 如果用户认为宪法 §六 / `tool-integration.md` 实际已经明示某立场 → 走非决策点路径直接落地
   - 否则 → 走决策点路径，用户拍板立场 A / B
3. **拍板时考虑**：
   - 反误报双向实测论证是否通过（来自 cc 阶段已附）
   - 可逆性（修了能撤吗）
   - 影响范围（伤几个工具 / 几个 entry）
   - 是否需要先在更大 corpus 验证

### X.2 review 后的落地约束（按 tool-integration §4.2）

任何 oracle 规则上线必须满足：

- **真 partial 路径上 grep 命中**（防漏报有效）
- **合法 SUCCESS / 合法注释 / 用户字面上不命中**（不引误报）
- **两边都过才能上线**

不能同时满足 → 保留漏报盲点（按 §4.4 诚实声明），不可引入误报。

### X.3 决策点不进 commit 时的处理

按 `principles.md` §八 + CLAUDE.md：

- 决策点本身就是"不擅自落地"的标记 —— 文档保留即可
- 用户拍板后落地的 commit 应在 message 注明"裁决 D-X 立场 Y"溯源
- 决策点保留历史而不删 —— 已裁决的标记 `[已裁决]` 但保留全文

### X.4 推荐 review 节奏

- **Day 1 (用户回来当天)**：扫 §0.5 目录 + 决定 D3.1 / D3.2 / D3.3 三个高优先漏报的方案 A / B / "只更 README 不动 wrapper"
- **Day 2-3**：决定 D1 / D2 candidate 误报立场
- **Week 1**：处理 D3.4-3.6 README 补完（无技术风险）
- **Week 2+**：DP-* 长期项按宪法 §三 模块优先级处理（次要模块不抢核心）

---

## §6 文档变更履历

| Round | 日期 | 内容 |
|---|---|---|
| R3 | 2026-05-11 上午 | 初版 — §1 (D1 / D2) + §2 (DP-1 ~ DP-12) + §3 推荐顺序 + §4 总表 |
| R5-cc | 2026-05-11 晚 | 追加 §3.5 (D3.1-3.6 漏报决策点) + §4 总表加 D3.* 行 |
| **R6** | **2026-05-11 晚** | **整理 — 新增 §0 阅读建议 / §0.5 目录索引 / §X review 指南 + §6 履历表；不剃任何原内容** |
