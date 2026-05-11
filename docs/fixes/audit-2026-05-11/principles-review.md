# 宪法审查 — `docs/design/principles.md`

> 综合审计 `audit-2026-05-11` / 第 1 组（宪法 + 设计原则文档层）
> 审查日期：2026-05-11
> 审查范围：`docs/design/principles.md`
> 跨文档矛盾汇总：本文 §4-cross-doc

---

## §1 问题意识

### 这份文档要审查什么？

`docs/design/principles.md` 是项目的**绝对宪法**——按项目根 CLAUDE.md，未经用户允许不可篡改、所有讨论与争议以此为准。文档篇幅 319 行，章节顺序"根本问题意识 → 项目目标 → 模块定位与优先级 → 三大派生原则 → 原则交集运行时投影 → 宪法级硬指标 → 外围原则 → 生效范围 → 下游关系"。

### 为什么要审查？

1. **宪法层一旦自相矛盾，整个下游设计都被污染** —— 因为派生链是"宪法 → 架构 → 细化 → 实现"，宪法本身有冲突时，下游永远在解不可解的约束系统。
2. **项目从 P4 → P16 经历快速增项**（引入 wrapper.sh、TS_* 环境变量、runnable corpus、第 20 个工具 ror-typecheck）—— 宪法没及时追上时，下游会与宪法暗中漂移。
3. **CLAUDE.md 明令"未经允许不可改宪法"**，所以如果发现宪法本身有问题，必须当作"修宪议案"标记给用户，**不能**自己直接改。

### 用什么方法 / 角度审查？

按 `/principle-derivation-v2` 的恶意角度：

- 假设宪法**有自相矛盾** —— 找哪几条条款互相约束、哪几条派生不顺
- 假设宪法**有未论证假设** —— 找"为什么这条作为基础"没原始论证的地方
- 假设宪法**有概念漂移** —— 同一概念在不同条款下定义有变
- 假设宪法**与当前实施脱节** —— P15 引入了 ror-typecheck（第 20 个工具）、P16 引入了 runnable corpus，宪法没明示

---

## §2 审查方法

### 参照源

- `docs/design/principles.md` 全文（行号引用）
- `docs/design/architecture.md`、`tool-integration.md`、`detailed-design.md`（用于交叉对照宪法是否被下游兑现）
- `runner/src/*.rs`（用于检查"宪法主张的运行机制"是否真实施）
- `tools/<name>/tool.toml`（用于检查"宪法引用的工具配置"是否对齐）
- 项目状态：从 git log 看到最新 commit `0019699 P7-P10`（即 P10），但下面发现 P11-P16 的实施物（`${TS_*}` 替换、wrapper.sh、ror-typecheck、runnable corpus）已在 working tree

### 恶意角度具体实施

- **自相矛盾搜索**：抓"位阶澄清"在 §四 与 §六 的相对位置；抓 §三-3 与 §六 自我性声明的范围矛盾；抓 §六-2 与 §三-3-2.b 是否真同义
- **未论证假设搜索**：检索"前端 vs 后端"切割的原始论证、检索"19 工具"枚举数字是否随实施变化
- **概念漂移搜索**：grep "前端 / 后端"、"形式指标"、"partial"在不同章节定义
- **与实施脱节搜索**：grep "runnable" / "ror-typecheck" / "TS_PROJECT_ROOT" / "wrapper.sh" 在宪法中的出现频次

### 诚实底线

每条问题给出**文件路径 + 行号 + 原文片段 + 推理链**。

---

## §3 审查现象

### #1（严重度: 高）—— "19 个工具"的硬编码枚举与实施漂移

**现象**：

- `principles.md` line 71：「**定位**：本项目目前接入的 **19 个**具体 Rust 验证工具（cargo-check / kani / miri / charon × 2 / creusot / hax × 3 / aeneas × 4 / prusti / verus / verifast / soteria / kmir / rocq-of-rust）的配置与实测，**居于次要地位**。」
- 工具实际目录：`ls tools/` 返回 **20 个**目录（多 `rocq-of-rust-typecheck`，P15 引入）
- 行 308 也写「19 工具配置示例」

**违反 / 嫌疑**：

- 宪法层"模块定位"段把 corpus 数硬编码到正文，这条数字会随每次集成 / 弃用工具变化——宪法不该用硬数字，应说"目前所接入的若干个具体 Rust 验证工具（清单见 `tools/`）"
- 这条产生了**实施漂移**：P15 已引 ror-typecheck（第 20 个），宪法没追上。下游 `architecture.md` line 84 / `principles.md` line 308 都用 "19" 一致复现这条漂移

**推理链**：

- 项目目标（§二）是"框架"不是"19 工具实测报告"——所以宪法 §三应只描述模块语义（次要模块 3 = 工具集成与实测），不该 enumerate 具体工具数
- 当 P15 引入新工具时，没有原则告诉作者"是否需要更新宪法 §三 中的数字"——本身就是宪法书写质量缺陷

**决策性**：☑ 非决策点（数字小修，把硬编码 19 改成"若干"或保持随实施动态）

**建议**：line 71 改为"本项目目前接入的若干个具体 Rust 验证工具（cargo-check / kani / miri / charon × 2 / creusot / hax × 3 / aeneas × 4 / prusti / verus / verifast / soteria / kmir / rocq-of-rust / rocq-of-rust-typecheck，详细清单见 `tools/`）"。注意：宪法虽然原则上不该频繁修订，但**纯计数的小修**属于"反映实施现状"，不构成实质宪法修订。

---

### #2（严重度: 高）—— P15 / P16 引入物未在宪法中确认地位

**现象**：

- `grep "runnable" docs/design/principles.md` → **0 hits**
- `grep "rocq-of-rust-typecheck\|ror-typecheck" docs/design/principles.md` → **0 hits**
- `grep "TS_\|\${VAR}" docs/design/principles.md` → **0 hits**
- `grep "wrapper.sh" docs/design/principles.md` → **1 hit**（仅 line 188 一次性说"tool.toml、harness.rs.tera、wrapper.sh 等"列举工具集成器边界，未做地位声明）

但实际 working tree：

- `tools/*/wrapper*.sh` 共 9 个 wrapper（aeneas × 4 / kani / prusti / verifast / rocq-of-rust × 2）
- `tools/rocq-of-rust-typecheck/` 是新增第 20 工具入口（按宪法 §六-1 表分类是"翻译类典型代表新增形式"）
- `examples/*/<dir>/hirusttest.toml` 中已有 `[runnable.<fn>]` 段（按 `hax-lean-consistency-design-2026-05-11.md` §1.2 方案 A）
- 主线 design 之外有独立设计稿 `docs/design/hax-lean-consistency-design-2026-05-11.md` 提到这些扩展

**违反 / 嫌疑**：

- CLAUDE.md §1.3 文档优先原则：「发现需求或问题 → 先看 principles.md 是否覆盖；不覆盖则提议修订宪法」—— P15/P16 都是"实施先行、宪法未追"的反向漂移
- 宪法 §六 自我性声明（line 267-271）：「这一节是强约束，新增工具 / 修改 oracle / 改动配置时必须满足。违反者必须在 PR 描述中明确说明修订理由并经讨论确认」——P15 引 ror-typecheck 作为新工具入口属于"新增工具"，应进入宪法 §三-3 工具枚举与 §六-1 表格更新，但未发生

**推理链**：

- 宪法 §六-1 表（line 235-245）枚举每个工具的"前端 / 后端切割方式"，但表中只列了 19 工具的旧列表，未加 rocq-of-rust-typecheck 这一行
- runnable 字段（按 hax-lean-consistency §1.2 方案 A 加进 hirusttest.toml）是 example schema 扩展，按宪法 §四 原则 A 形式定义（line 146-157）「加入框架信号文件前后行为字节级一致」，runnable 字段满足这一形式定义（cargo 仍不读 hirusttest.toml），所以**理论上不破宪法**——但这条派生应明确写进宪法，否则未来读者不知"宪法层是否允许 hirusttest.toml schema 自由扩展"
- `${TS_*}` 环境变量替换 / `TS_PROJECT_ROOT` / `TS_ENTRY_FN` 注入子进程是宪法 §四 原则 C「异质性归配置」的具体兑现机制——但宪法未明示这一机制存在

**决策性**：☑ **决策点** —— 需要用户审议：

- 是否把 P15/P16 引入物正式吸纳进宪法？（特别是 ror-typecheck 是否承认为正式集成工具）
- runnable 字段是否需要宪法明示其符合原则 A 形式定义？
- `${TS_*}` 替换机制是否要在宪法层明示其符合原则 C？

**建议**：在宪法的 §三-3 工具枚举更新 + §六-1 表加 rocq-of-rust-typecheck 行 + §四 A 形式定义后注明"hirusttest.toml 内 `[runnable.*]` 等子表扩展不破形式定义"。这是修宪议案——必须经用户确认。

---

### #3（严重度: 中）—— "形式指标"概念漂移：§三-3-2.a vs §六-2

**现象**：

- `principles.md` line 87-99 「§三-3-2.a 以形式指标为最终解释」：列了 exit code / 产物存在性 + 大小阈值 / 产物字面 grep / stderr 字面 grep / 多门组合五类形式指标；line 95 说「多门组合（rocq-of-rust 6 道门）」
- `principles.md` line 254-256 「§六-2 不允许 partial」：「`tool.toml` 的 oracle 必须按工具内部 pipeline 设计精确停在自带后端之前，但完整跑完前端」

**违反 / 嫌疑**：

- §三-3-2.a 把"形式指标"定义为"oracle 用的具体可机检条件"——多门组合（rocq-of-rust 6 道门）也算
- §六-2 说「oracle 必须...精确停在自带后端之前」——"停在后端之前"和"完整跑完前端"是 oracle 的**行为切割定义**，与§三-3-2.a 的"判定条件"是两个不同维度
- 一个 oracle 同时需要做两件事：**(a) 让工具跑哪一段（行为切割）**、**(b) 用什么条件判定 SUCCESS（判定条件）**。宪法把这两件混在"形式指标"一个词里有概念漂移嫌疑
- 此外 §三-3-2.a line 95 写"rocq-of-rust 6 道门"——这与 `tool-integration.md` line 30 写的"rocq-of-rust 的 5 道门"直接冲突（见 §4-cross-doc #X1）

**推理链**：

- 同一文档内 §三-3-2.a "多门组合"作为"形式指标"分类（即判定条件分类）→ 这是 oracle 的**输入**
- §六-2 "oracle 必须精确停在自带后端之前" → 这是 oracle 的**行为切割**
- 一份严谨宪法应区分这两个维度。当前文字混在一起，潜在让读者把"判定条件多门"和"行为切割停在后端"误以为是同一件事

**决策性**：☐ 非决策点（文字小修，澄清两概念区别）

**建议**：在 §三-3-2.a 加一句"注意'形式指标'专指 oracle 的**判定条件**——oracle 的**行为切割**（让工具跑哪段）见 §六-1 前端支持性观察"；并补充 §三-3-2.a 与 §六-1 的明确互引。

---

### #4（严重度: 中）—— rocq-of-rust 道门数"6 vs 5"的自相矛盾

**现象**：

- `principles.md` line 95：「多门组合（rocq-of-rust **6 道门**）」
- `tool-integration.md` line 30：「**多门组合**（如 rocq-of-rust 的 **5 道门**：exit 0 + 至少一个 .v + 无 0-byte + > 200B + 无 silent marker）」
- 实际 `tools/rocq-of-rust/rocq-of-rust-wrapper.sh` 注释：「Oracle (still **6 gates** — wrapper is gate-equivalent to the old tool.toml...)」（第 55-61 行）
  - gate 1: exit 0
  - gate 2: ≥ 1 .v product
  - gate 3: no zero-byte
  - gate 4: ≥ 1 product > 200 B
  - gate 5: no failure marker
  - gate 6: entry_fn appears as `Definition <fn>` —— P15 加入的反 silent-skip 闸门

**违反 / 嫌疑**：

- 宪法 line 95 准确（6 道门），但 `tool-integration.md` line 30 落后（5 道门——P15 引入 gate 6 之前的旧实现）
- 这条不属宪法层面的自相矛盾（宪法自身正确），但属"宪法与下游不一致"——下游 tool-integration 应改

**推理链**：

- 道门 6（entry_fn 出现校验）是 P15 引入物（按 `docs/fixes/oracle-leak-rules-implementation-2-2026-05-11.md` §3.2）
- 宪法显然在 P15 之后更新过（保留 "6 道门"），但 `tool-integration.md` 没追

**决策性**：☐ 非决策点（tool-integration 小修）

**建议**：本条不需改宪法，但需提示用户改 `tool-integration.md` line 30 的 "5 道门" → "6 道门"，并补 gate 6 描述。

---

### #5（严重度: 中）—— 派生原则 A 的"形式定义"违反自身定义？

**现象**：

- `principles.md` line 146-157 § A 的形式定义：「一个 example 是"自身完善独立"的当且仅当——加入框架信号文件（`hirusttest.toml`）**之前与之后**，example 自身的行为**完全 100% 一致**」
- 同段 line 157 应用举例：「Cargo.toml 内嵌方案**违反**该形式标准（cargo 读 Cargo.toml 时会解析全部内容）」

**违反 / 嫌疑**：

- 形式定义本身严格自洽，无矛盾
- 但 line 173："`.hirusttest/` 仅可包含项目自有 schema 的辅助文件（如 per-entry verifier stub、proof annotation snippet 等）；这些辅助文件**可被工具读取**（**这是受控的工具适配，发生在 framework 中介层调度时**），但 example 自身的 src/ 与 Cargo.toml 仍需满足形式定义"
- 这条引入了"辅助文件可被工具读取"——但形式定义说"加入前后行为字节级一致"。如果工具会读 `.hirusttest/<entry>-spec.rs`（如 line 103 描述：「per-entry 的 verifier spec stub，runner 在隔离副本上把这类 stub 注入 src/」），那 "stub 注入 src/" **本身就是修改 src/**，原则 A 形式定义就破了

**推理链**：

- 形式定义 line 146-150 说"信号文件对 cargo / rustc / 任何 verifier 工具完全不可见——加不加它，对 cargo build / cargo check / cargo run / cargo test 等任意 cargo 子命令的输出字节级一致"
- 但 line 173 说"per-entry verifier stub"可被工具读取——而 line 103（detailed-design.md）说"runner 在隔离副本上把这类 stub 注入 src/"
- 也就是说，整流程是：原始磁盘 example src/ 字节级不变，但**隔离副本** src/ 会被 stub 注入。这件事按 line 142 的"位阶澄清"是允许的（"原始磁盘字面零修改 + 隔离副本上声明式工具特定填充"），但 line 146-157 的形式定义没明示**"作用于副本"vs"作用于原始磁盘"的区分**
- 结果：line 146-157 的形式定义读起来覆盖整个流程（包括副本），但 line 173 又允许副本被工具读取（破形式定义）

**决策性**：☑ **决策点** —— 需要用户审议：

- 是否在 line 146-157 形式定义里明确加入"**对原始磁盘 example 目录的 cargo 行为**字节级一致"限定语？

**建议**：line 146 改为「一个 example 的**原始磁盘目录**是"自身完善独立"的当且仅当——在该目录下加入框架信号文件 `hirusttest.toml` 之前与之后，**该原始磁盘目录上的任意 cargo 子命令输出**完全 100% 一致」。这条把作用域明示为"原始磁盘 + cargo 子命令"，与 line 142"位阶澄清"互补。

---

### #6（严重度: 中）—— 自我性声明的范围模糊：§六自我性 vs §三-3 自我性

**现象**：

- `principles.md` line 267-271 § 六自我性声明：「§三-3'市面工具样例'的诚实测试范围 + §六 宪法级硬指标 + 派生的工具 README 书写规范是**我方使用本框架做 tools 集成时的方法学选择**。... 外来 tools 无须遵守本节；框架本身只要求 §四」
- `principles.md` line 31-33 § 三模块定位：「**核心模块**（first-class）是项目长期承诺；**次要模块**（non-first-class）是核心模块的应用展示」
- `principles.md` line 297-300 § 八生效范围：「3. **次要模块 3（市面工具样例）的工具集成** — tools/<name>/{tool.toml, harness.rs.tera, README.md} 必须满足硬指标 §六-1 至 §六-4 + 时效性 + 诚实测试范围」

**违反 / 嫌疑**：

- 同时 (a) 「§六 自我性声明：外来 tools 无须遵守本节」和 (b) 「§八：次要模块 3 的工具集成必须满足硬指标 §六-1 至 §六-4」——两条同时正确，但区分不清"哪些工具集成需要遵守 §六、哪些不需要"
- 解读：本项目自己集成的 19/20 个工具必须遵守 §六；其他人复用本框架做自己的工具集成时无须遵守 §六。这条解释合理，但宪法文字让读者要花心思才能解出来

**推理链**：

- 当前文字下，"外来 tools" 指"复用本框架的第三方人员"——但读者可能误读为"本项目集成的 tools 中外来的"（这是错的，因为本项目所有 tools 都"外来"——cargo-kani / cargo-prusti 等都是外部上游工具）
- 这是 self-confusing 措辞，宪法层 absolute clarity 标准下该改

**决策性**：☐ 非决策点（文字澄清，意图明确）

**建议**：line 267 改为「§三-3 + §六 + 工具 README 书写规范是**本项目维护 tools 集成与实测报告时的方法学选择**。其他人复用本框架做自己的工具集成与方法学选择时无须遵守本节；框架本身（runner 实现 + 双轨 schema）只要求 §四」

---

### #7（严重度: 低）—— 派生原则 A 5 层精细化与§三-3-2 的派生关系不显式

**现象**：

- `principles.md` line 184-193 § A 5 层精细化（Examples 自身完善独立 → 极简解耦 → Examples 端标识性信号的上限 → Tools 端绝不为框架适配 → 框架负责一切信号转换）
- `principles.md` line 102-118 §三-3-2.b/c 上限保证 + 下限诚实

**违反 / 嫌疑**：

- A 5 层是"双方不可侵入"的精细化——主要约束 examples ↔ tools 的解耦
- §三-3-2 是"诚实的测试范围"——主要约束 oracle 的可信度边界
- 两者本应是正交的（A 管"解耦"，§三-3-2 管"诚实"），但宪法没说明它们的关系
- 潜在风险：未来读者可能把 A-4「Tools 端绝不为框架适配」误读为"oracle 不能加约束"——其实 oracle 是 tools/<name>/tool.toml 层做的，不是 tool 本身做的

**推理链**：

- 宪法 line 188 A-4：「tools 是黑盒。tools/<name>/ 目录下的所有文件...是**集成者描述工具自身行为**的桥接信号——**不是工具自身的修改**。tool 本身（cargo-kani / cargo-prusti / cargo-creusot 等）作为黑盒，绝不为框架做任何代码或行为上的适配。」
- 这条已经明示"集成者描述 vs 工具自身"，但没把 oracle 这一具体集成行为列出来
- §三-3-2 的"诚实测试范围"建立在 oracle 的可信度边界上，所以两条紧密相关

**决策性**：☐ 非决策点（文字小补，明示关系）

**建议**：line 193 后补一段「§三-3-2 的诚实测试范围是 A-4 'Tools 端绝不为框架适配' 在 oracle 层的具体派生——oracle 作为 tool.toml 的集成者描述，不为工具适配，反过来必须**诚实地表达工具的实际边界**，这就要求 0 误报（不冤枉）+ 尽量 0 漏报（不高估）+ 已知盲点声明。」

---

### #8（严重度: 低）—— "性能不算问题"vs"硬指标 1 前端切割"潜在张力

**现象**：

- `principles.md` line 278-284 § 七 性能不算问题：「资源用量、性能开销、缓存效率本身不构成'问题'。只有当某性能/资源失控让功能层面失效时它才算问题」
- `principles.md` line 230-236 § 六-1 前端支持性观察：「两件事在量级上完全分开：前端通常秒级、求解层可能小时级或不收敛。把求解层一并测会让对比失公」

**违反 / 嫌疑**：

- 表面看 §七 性能不算问题 vs §六-1 量级分开是张力——§六-1 实际上是在说"求解层时间太长导致对比失公"，这其实**是**性能问题
- 解读：§七 性能不算问题指的是"runner 内部的资源/缓存"；§六-1 量级分开指的是"工具自身行为的可控范围"。两者实际不冲突
- 但宪法没明示这一区分

**推理链**：

- §七 的"性能"举例：subprocess stdout 一次性读到内存的 OOM、Cargo.lock 副本的延迟、cargo registry flock 在并发首次 fetch——都是 runner 内部的资源/性能
- §六-1 的"量级分开"是工具求解层 vs 前端层的语义分割
- 两者是不同对象的"性能"，文字没明示对象差别

**决策性**：☐ 非决策点（文字小补）

**建议**：line 278 改为「runner / 隔离机制 / 缓存等**框架自身**的资源用量、性能开销、缓存效率本身不构成'问题'。」明示"框架自身"。

---

### #9（严重度: 低）—— "硬指标 3 不区分翻译深浅"的诚实表达 vs "测试报告归责" 张力

**现象**：

- `principles.md` line 258-260 § 六-3 不区分翻译深浅：「两个工具同对一段 Rust 标 SUCCESS 不蕴含它们后续做的东西等价——这是覆盖度测量的本性，不是缺陷」
- `principles.md` line 122-130 § 三-3-原则 3 实测报告责任边界：「2. 非评判性：报告内部分析（特别是失败模式归类、跨工具发现等）不构成对工具的实质评判」

**违反 / 嫌疑**：

- § 六-3 默认"覆盖度数字不蕴含等价"——是诚实警告
- § 三-3-原则 3 默认"报告内部分析非评判"——也是诚实警告
- 两者实际互补但宪法没明示

**推理链**：

- 读者拿到 report.md 看到两工具 SUCCESS 数字接近，可能误以为"两工具能力相当"——§六-3 警告这种推断不成立
- 报告作者写"X 在 Y feature 上 partial 比 Z 多"，可能被误读为"X 比 Z 差"——§三-3-原则 3 警告这是非评判
- 两者其实是同一精神的两面

**决策性**：☐ 非决策点（文字小补，连接两条）

**建议**：在 § 六-3 末尾加一句「这条与 §三-3-原则 3 实测报告非评判性互补：SUCCESS 数字的对比不蕴含工具能力对比，报告内部分析不构成工具能力评判。」

---

## §4-cross-doc 跨文档矛盾

### #X1（严重度: 高）—— rocq-of-rust 道门数：principles 6 vs tool-integration 5

**对照源**：

- `principles.md` line 95：「多门组合（rocq-of-rust **6 道门**）」
- `tool-integration.md` line 30：「多门组合（如 rocq-of-rust 的 **5 道门**：exit 0 + 至少一个 .v + 无 0-byte + > 200B + 无 silent marker）」
- 实际 wrapper `tools/rocq-of-rust/rocq-of-rust-wrapper.sh` line 55-61：6 gates，其中 gate 6 是 P15 引入

**推理链**：宪法 line 95 已对齐 P15 实施（6 道门），但 tool-integration line 30 落后（仍 5 道门）。

**决策性**：☐ 非决策点（修 tool-integration）

**建议**：tool-integration.md line 30 改 6 道门 + 补 gate 6 描述。

---

### #X2（严重度: 高）—— "19 工具"硬编码在 principles + architecture + detailed-design

**对照源**：

- `principles.md` line 71 / 308：「19 个」/「19 工具配置示例」
- `architecture.md` line 84 / 294：「全在内（19 工具）」/「19 工具配置示例」
- `detailed-design.md` line 335：「19 个工具的完整 tool.toml / harness.rs.tera 配置」
- 实际 `tools/` 目录：20 个

**推理链**：3 份 design 文档同步漂移——P15 引入 rocq-of-rust-typecheck 后未级联更新。

**决策性**：☐ 非决策点（统一改为"若干"或动态枚举）

---

### #X3（严重度: 中）—— P15/P16 引入物在 design 主线的覆盖缺失

**对照源**：

- `principles.md` / `architecture.md` / `tool-integration.md` / `detailed-design.md` 主线四文档
- `grep "runnable"` → 仅 detailed-design 命中（line 30-80）
- `grep "rocq-of-rust-typecheck"` → 0 hits
- `grep "wrapper.sh"` → principles line 188 一次性提及
- `grep "TS_PROJECT_ROOT\|\${TS_"` → 0 hits in design docs

**推理链**：

- P15（ror-typecheck 新工具入口、wrapper.sh 机制成熟、`${TS_*}` 环境变量替换层）
- P16（runnable corpus 的 `[runnable.<fn>]` schema 扩展）

两条 P 都没在 design 主线吸纳——detailed-design.md 仅补了 `[runnable.<fn>]` 段定义，但 architecture / principles / tool-integration 三份都未追。

**决策性**：☑ **决策点** —— 需要用户审议这些扩展的宪法地位（详见本文 §3 #2）

---

### #X4（严重度: 中）—— detailed-design.md §五 工具示例与实际 tool.toml 整体不同步

**对照源**：

- `detailed-design.md` line 363-367（Kani 示例）：`command = ["cargo", "kani", "--only-codegen", "--bin", "__ts_harness"]`
- 实际 `tools/kani/tool.toml` line 27：`command = ["${TS_PROJECT_ROOT}/tools/kani/kani-strict-wrapper.sh"]`
- `detailed-design.md` line 398（Charon-poly 示例）：`["/tmp/ts-tools-install/charon/bin/charon", "cargo", "--abort-on-error", "--", "--lib", "--target", "aarch64-apple-darwin"]`
- 实际 `tools/charon-poly/tool.toml`：`["${TS_CHARON_BIN}", "cargo", "--abort-on-error", "--print-llbc", "--", "--lib", "--target", "aarch64-apple-darwin"]`（多 `--print-llbc`，硬编码路径换 `${TS_CHARON_BIN}`）
- `detailed-design.md` line 425-440（Prusti 示例）：硬编码 env list
- 实际 `tools/prusti/tool.toml`：用 wrapper.sh + `${TS_PRUSTI_*}` 环境变量

**推理链**：detailed-design.md §五 写"19 个工具的完整配置"但**实际只列 6 个示例**，且这 6 个都已落后于 working tree 的 tool.toml。

**决策性**：☐ 非决策点（detailed-design 同步实施）

**建议**：要么把 §五 "已集成工具配置示例"明示为"示意性历史快照，权威配置见各工具 tools/<name>/tool.toml"；要么去掉硬编码示例，仅列引用。

---

### #X5（严重度: 中）—— runner 代码注释引用的 design 章节锚点不存在

**对照源**：

- `runner/src/discover.rs` line 257：「`(see §7.5)`」
- `runner/src/exec.rs` line 250：「`(see §7.6 spec note)`」
- `detailed-design.md` 章节：用的是 §一/§二/.../§七（中文数字）
- 找 §7.5 / §7.6 → **不存在**

**推理链**：runner 实现写作时引用的章节号是"7.5/7.6"（阿拉伯数字+子节），但 design 文档已改为中文一级章节"§七 错误处理策略"——子节标题"超时"、"异常"、"cleanup work_dir 失败"等没编号。runner 注释的引用 dangle 了。

**决策性**：☐ 非决策点（注释小修，或 design 给章节加阿拉伯子节号）

---

### #X6（严重度: 低）—— version_command 在 tool-integration 是"必含"，在 detailed-design 是"可选"

**对照源**：

- `tool-integration.md` line 19：「`tool.toml` **必须含** `version_command`」
- `detailed-design.md` line 121-130：「`version_command  # 可选，默认 []`」

**推理链**：tool-integration 是工具集成方法学（**我方**集成时必须含）；detailed-design 是 runner schema（接受空数组）。两者表面冲突，实际是位阶不同——但宪法/位阶约束没明示这种"软约束 vs 硬 schema"区分。读者可能困惑：到底必填还是可选？

**决策性**：☐ 非决策点（文字澄清）

**建议**：detailed-design line 130 加一句「按 [tool-integration.md](tool-integration.md) §一，本项目自己集成的所有工具的 tool.toml 必须配置非空 version_command；runner schema 接受空数组是为了让第三方复用框架时灵活配置」

---

## §4 决策点 vs 非决策点汇总

### 决策点（需用户审 / 拍板）

| # | 摘要 |
|---|---|
| #2 | P15/P16 引入物（ror-typecheck、runnable、TS_* 替换、wrapper.sh）的宪法地位 |
| #5 | A 形式定义是否要明示"作用于原始磁盘 example 目录的 cargo 行为" |
| #X3 | design 主线四文档对 P15/P16 的覆盖更新（与 #2 同源） |

### 非决策点（局部 fix）

| # | 摘要 |
|---|---|
| #1 | 19 工具硬编码改"若干" |
| #3 | 形式指标"判定条件 vs 行为切割"概念澄清 |
| #4 | rocq-of-rust 6 道门 in tool-integration |
| #6 | 自我性声明"外来 tools"措辞澄清 |
| #7 | A 5 层 ↔ §三-3-2 派生关系明示 |
| #8 | "性能不算问题"明示对象为"框架自身" |
| #9 | "不区分翻译深浅"与"实测报告非评判"互补连接 |
| #X1 | tool-integration 道门数从 5 改 6 |
| #X2 | 19 工具枚举 → 动态/若干 |
| #X4 | detailed-design §五 工具配置示意化 |
| #X5 | runner 注释引用章节锚点修复 |
| #X6 | version_command 必/可选位阶澄清 |

---

## §5 审查结论

### 总体判断

`principles.md` 作为宪法**主体精神正确、自洽性强**——三大派生原则（A 双方不可侵入 / B 测必要条件 / C 异质性归配置）的派生链清晰，§六硬指标的论证逻辑严密。但存在三个层面的问题：

1. **实施漂移**（高严重度）：P15/P16 已引入物（第 20 个工具 ror-typecheck、runnable corpus、TS_* 替换、wrapper.sh 机制）未在宪法/架构/工具集成原则三份主线 design 中正式吸纳——这是 CLAUDE.md §1.3 "文档优先原则"被反向破坏。
2. **概念漂移**（中严重度）：形式指标的"判定条件"vs"行为切割"两维混在一个词里；自我性声明"外来 tools"措辞模糊；A 形式定义对"原始磁盘 vs 隔离副本"作用域未明示。
3. **小漂移**（低严重度）：跨文档的硬编码数字（19 工具、6/5 道门）、runner 注释引用章节锚点 dangle、version_command 必/可选位阶不清。

### 严重度分布

- 高严重度：3 条（#1 + #2 + #X1 + #X2 视为同源 = 1 决策项 + 2 同步项）
- 中严重度：4 条（#3 + #5 + #6 + #X3 + #X4 + #X6 = 6 条按"独立性"算）
- 低严重度：4 条（#7 + #8 + #9 + #X5）

### 关键风险

最严重风险是 **#2 / #X3**：宪法/架构/工具集成原则三份主线 design 在 P12 后停止追实施。如果 P17+ 继续引入新机制（如 hax-lean-eval 一致性测试），design 主线会越来越脱节——届时新人接手项目时只能读源码理解，违反 CLAUDE.md "项目以 docs/design 为中心"的根本约束。

### 是否发现需要修宪的问题？

**是，标记修宪议案**：
- **议案 #2-A**：吸纳 ror-typecheck 为正式集成工具（更新 §三-3 工具枚举 + §六-1 表）
- **议案 #2-B**：明示 hirusttest.toml 子表扩展（如 `[runnable.*]`）符合原则 A 形式定义
- **议案 #2-C**：明示 `${TS_*}` 环境变量替换层符合原则 C
- **议案 #5**：A 形式定义加"作用于原始磁盘 example 目录的 cargo 行为"限定语

这些都不是单纯文字小修，**必须经用户确认**才能改。
