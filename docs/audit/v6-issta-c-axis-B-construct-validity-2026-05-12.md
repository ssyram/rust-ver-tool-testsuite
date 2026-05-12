# Axis B: Construct Validity (Wohlin Ch.8 / ISSTA)

> 审查者立场：ISSTA 2026 program-committee reviewer，按 Wohlin et al. 2012 §8（construct validity = "do measurements actually measure the constructs they claim to"）独立审。
> 范围：v6 final / commit ebe6858，含 `principles.md` v8 / `feature-coverage-2026-05-12-v6.md` / 20 份 `deep-reports/cc-reports/*.md` / `tools/<name>/README.md`。
> 协议：disprove-first；先反挑刺、自带 R1/R2/R7/R11/R12 反查；下文 Findings 是 counter 之后仍站得住的部分。

---

## 一、"Feature Coverage" Construct

### 核心张力

`principles.md` §二明示 construct 是"工具特性覆盖"，但 operationalization 是 `pass-rate = SUCCESS / 161`。SUCCESS 的定义在 §四 B 与 §六 第一段被刻意降级为"**工具能不能吃下这段代码并产出预期形状的输出**"——这是 "tool acceptance on the corpus" 而**不是** "feature coverage" 的标准定义（feature coverage 通常意味着 mapping `feature ∈ Features → covered? ∈ {yes,no}`，需要 feature-level taxonomy + each-feature observable）。

corpus 是 161 entries × 41 features 的多对多结构（v6 report §0.2），但报告只汇总 entry-level pass rate，不 emit feature-level coverage 表。命名为 "feature coverage" 的报告标题与 operationalization 之间存在 construct-level slippage。

### Findings

- **Major B-1（construct-operationalization gap）**：报告名 `feature-coverage-*.md` 与 operationalized metric（entry-level SUCCESS count）之间无中间层。若 reviewer 严按 Wohlin §8.1 "mono-method bias" + "construct underrepresentation"，应额外给出 per-feature roll-up（41 features 各自的 SUCCESS / total entries），或把指标改名为 "entry-level tool acceptance rate"。**R11 反查**（文案 vs 实现）：principles.md §四 B "等距推论"承认"测接受 vs 不接受"——这是文案 vs 命名的不一致，挑刺站得住。

- **Major B-2（construct 异质聚合）**：cargo-check 100% 与其他 19 工具的 pass rate 同表呈现（v6 §3 表），但 `cargo-check.md` 自陈 cargo-check "**不测验证能力**、是 corpus 合法性 baseline"。两类不同 construct（rustc 合法性 vs verifier 接受度）混在一张比较表内，是构造混淆。**R12 作者反驳点**：作者可能反驳"我们顶部免责 + cargo-check 报告刻意不排名"——但 v6 §3 表里 cargo-check 排首位、读者会自然取 anchor。挑刺站得住。

- **Minor B-3（feature taxonomy 内部异质）**：41 features 既含 Rust 语法层（`enum/` / `trait/` / `lifetime/`）又含工具特定边界（`*-limit/` 八个）。"feature coverage" 在两类上语义不同：前者测语言子集吃下度，后者测工具自陈边界——construct 不纯。

---

## 二、"Front-end Measurement" Boundary

### 核心张力

`principles.md` §六"前端测量（深度切割）"宣称是宪法级 construct，但每工具切点截然不同：

| 工具 | "前端"切点 |
|---|---|
| kani | `cargo kani --only-codegen` exit + 5-marker grep + P37 source-walk |
| verus | `--no-verify` 同时切 AIR + Z3（不可分），VIR 即 "前端" |
| prusti | `PRUSTI_PRINT_HASH=true` short-circuit before Silicon |
| charon-* | 整条 LLBC pipeline 完成（无后端） |
| creusot | 默认无 subcommand → 停在 `.coma` 翻译 |
| aeneas-* | charon LLBC + aeneas OCaml engine + printer，完整 `.v` 落盘 |
| rocq-of-rust | "前端 = 全过程 = 翻译到 .v"（cc-report §pipeline 自述） |
| miri | `cargo miri run` 完整解释执行（"无前端 / 后端分界"，cc-report §pipeline 自述） |
| kmir | 三段全跑（cargo + stable-mir-json + K interpreter） |
| soteria | 完整符号执行（"无独立前端 / 求解层边界"，cc-report 自述） |
| verifast | 完整 symex + Z3 求解（`-skip_specless_fns` vacuous gate） |

### Findings

- **Major B-4（construct 异质性 fatal）**：所谓"前端"在 7 个工具上是 "translation stage exit"，在 4 个工具上是 "tool 跑完全过程的 exit"——这两类不是同一 construct。把它们都标为"前端测量"违反 Wohlin §8 "single-construct consistency"。**R2 反查**（已文档化设计选择）：principles.md §六明示"前端 = 工具自身前端（parser / 类型检查 / 翻译 / 模型构造），求解层不计入"——但 miri / soteria / kmir 没有"前端 vs 求解层"的可切分边界（cc-reports 三家都自陈无此切割），它们走完整执行——这就让"前端 = 不进求解"的精神在这些工具上落空。已文档化也不能消解 construct 漂移。挑刺站得住。

- **Major B-5（公平性 invariant 受冲击）**：kani / prusti / verus 通过显式 `--only-codegen` / `--no-verify` / `PRINT_HASH=true` flag **主动**切掉求解；miri / kmir / soteria 是 SMT/symex 在小 entry 上**自然秒过**。这两种"前端等价"在被求解开销 dominate 的 industrial 样例上行为发散——v6 §3 排序里 verus（强切）40% vs miri（自然完成）97%——读者会把这读成"miri 比 verus 强"，但这只是切点宽窄不同。**R12 反驳点**：作者会说"miri 的 SUCCESS 包含 bug-detect 翻转、verus 是纯 type/lifetime check"——但这恰恰证明二者 construct 不同，不可同列比较。

- **Minor B-6（rocq-of-rust "前端 = 全过程"自陈与精神冲突）**：cc-report rocq-of-rust.md §pipeline 直陈"本工具前端 = 全过程 = 翻译到 .v"——而 `principles.md` §六说"前端 / 后端"是有切割的；rocq-of-rust 明明 archive-only，无后端可切，把它纳入"前端测量" construct 是定义上的 vacuous truth。建议改 construct 名为"工具流水线 acceptance"，分别声明每工具流水线终点。

---

## 三、Over-strong Claims (formal vs empirical)

### 核心张力

20 cc-reports 里 "形式可证" / "形式严格性" / "不可形式证明" 三种措辞密度极高（grep 命中 65 次跨 20 文件）。R7 反查（弱化后不得恢复强措辞）是 v6 cc-route audit 在 aeneas 4 backend 已经做过的——README 把 "形式可证 0 漏报" 改为 "实测 + wrapper 双通路封堵"（v6 report §5.2）。但其他工具的强措辞仍在。

### Findings

- **Major B-7（charon-mono / charon-poly 强自陈"形式可证 0 漏报"）**：charon-mono.md §SUCCESS 写"**0 漏报：形式可证（单一通路 — 见 tool-integration.md §四.1）。论证：`charon-driver/driver.rs:143` 设 `error_ctx.continue_on_failure = false`，... 单一通路覆盖所有 silent path**"。此声明是**源码级断言**但只引一行 driver.rs 设置，不构成 silent-path closed-form proof（外部依赖 opaque 化路径未被 register_error! 这一点 README 自己也承认）。这与 aeneas R7 修正前的"形式可证 0 漏报"是同 pattern。**R11 反查**（文案 vs 实现）：aeneas R7 audit 已经推翻类似措辞，charon README 应一并降级——否则破 R7 invariant（弱化后不得恢复强措辞）。挑刺站得住，建议同步降级。

- **Major B-8（cargo-check "0 漏报形式可证"过强）**：cargo-check.md §SUCCESS 写"**0 漏报：形式可证。rustc 不存在 silent skip 路径**"——这是对 rustc 整体行为的全局断言，需要 rustc source-level proof。construct validity 角度：rustc bug history（含 ICE）可反例化此断言，建议改 "0 漏报：实测可信 + rustc 设计目标"。**R2 反查**：principles.md §六 "Oracle 责任"要求"已知漏报盲点必须文档化"——cargo-check.md 写"漏报盲点：无"是孤例，与 §六精神（"不藏"）相冲突（rustc ICE 偶发是公开事实）。挑刺站得住。

- **Minor B-9（verus / kani / soteria 等"实测验证"措辞已诚实）**：以下文件已用"实测验证 / 不可形式证明"双标声明，符合 R7：kani.md §SUCCESS / verus.md §SUCCESS / soteria.md §SUCCESS / hax-lean.md §SUCCESS / rocq-of-rust.md §SUCCESS。这些是 baseline 良性样本。

- **Minor B-10（cc-reports 散见"反向证明"被滥用为"证明"）**：kani.md "P37 §六 反向证明"实际是 source-walk + regex match，应叫 heuristic source-filter，称"证明"会让 reader 误解为形式 proof。

---

## 四、Localness / Tool-blame Wording

### 核心张力

`principles.md` §一-1 第 23 行原文："**装对其要求的 toolchain 后仍不行 → 工具菜，FAILED 站得住**"。"工具菜"（口语贬义）出现在宪法级文档里，是 reviewer 易抓的把柄。同样的措辞在 v6 主报告 §2.2 第 81 行复述。

### Findings

- **Major B-11（"工具菜"等口语化措辞破学术 register）**：ISSTA reviewer 会直接画红线——这是 register mismatch，影响 construct 在外部观察者那里的可信度。**R12 反驳点**：作者可能反驳"宪法是项目内部 prompt，不是 paper"——但 ISSTA 评审范围是 v6 final commit ebe6858 整套（含 principles.md），文档化的口语化措辞构成 construct 表达层的 noise。建议替换为：

  | 旧措辞 | 推荐学术措辞 |
  |---|---|
  | 工具菜 | capability boundary reached / unsupported construct |
  | 工具自陈 | tool's own self-reported diagnostic |
  | 工具的锅 | upstream-attributable failure |
  | 站得住 | warranted under the local-currency principle |

  这条建议是 register-level，不要求改 construct 内涵。挑刺站得住（R1 现象 ≠ 缺陷，但学术 register 错位本身就是 construct 表达缺陷）。

- **Minor B-12（"本地性"作为 construct 锚点缺学术先验链接）**：principles.md §一 提出"本地性 / 当前性 / 社区惯例 / 最大善意"四原则栈是项目自创，未对接 Wohlin §8 / 软件工程实验文献中的成熟构造（如 "as-tested capability" / "configuration boundary"）。学术读者会要求引用支持。

---

## 五、Falsifiability of 0-FP / 0-FN

### 核心张力

20 工具中："0 误报 + 0 漏报"声称形式可证的有 cargo-check / charon-mono / charon-poly / verus（仅 0 误报形式 / 0 漏报有盲点）；声称实测的有 kani / prusti / aeneas-4 / hax-3 / rocq-of-rust×2 / kmir / soteria / miri / verifast。Construct validity 关键问题：falsifiability。

### Findings

- **Major B-13（"反向证明"在 oracle 设计意义上不可证伪）**：kani P37 source-walk + 正则关键字匹配 `\b(asm|global_asm)\s*!`，自陈"保守策略偏向 false-positive FAILED"。但实测 161 entries 里 0 反例——这是 absence of evidence，不是 evidence of absence。reviewer 角度：oracle 在 corpus 内 0 反例 ≢ oracle 形式正确。**R12 反驳点**：kani.md 自陈"非形式证明，仅实测校准"——挑刺已被 README 自承认，不能再加重；但报告主表（v6 §3）的 SUCCESS / FAILED 数字直接采用此 oracle 输出，没传播这层 uncertainty。建议在 v6 主报告 §1 TL;DR 之后加一节 "Oracle Trust Level"，分工具列三级（形式可证 / 实测+wrapper / 实测） — 让读者按层次解读 pass rate。

- **Major B-14（charon "0 漏报形式可证"未做"silent skip-item"全枚举）**：见 B-7。即使 `continue_on_failure=false`，仍需证明所有 unsupported codepath 都走 `register_error!` 而非 silent return。README 只引一处 grep + 一处 driver.rs:143，未做穷举。

- **Minor B-15（verifast vacuous-pass gate 是 oracle 设计层创举，但 0 漏报论证仅在 corpus 0 spec 这个特殊情况下成立）**：verifast.md 自陈"corpus 0 entry 含 //@ req/ens"——当未来 corpus 加入 spec-bearing entry，gate 假设失效。这条 falsifiability 已诚实声明（"规则在 spec corpus 上行为待实测"），属合规盲点。

- **Minor B-16（miri "wrapper 不区分 UB 在 entry 还是 vendored crate"）**：miri.md §漏报盲点 3 自陈"本测试 corpus 暂无后者实例，但若将来引入，wrapper 仍会翻 SUCCESS"——这是 construct 的边界，与 §六"当前 crate 焦点"潜在冲突（bug-detect 在 vendored crate 时 SUCCESS 是否合理？P35 派生未明示该边界）。

---

## 六、Bug-detect = SUCCESS Consistency

### 核心张力

`architecture.md` §一 P35 派生：MIRI 检出 UB / soteria 检出 bug → SUCCESS。这是 v6 新引入的 construct 扩展（原本 SUCCESS = "工具吃下 feature 不报错"，现在 = "吃下 feature 不报错 OR 吃下 feature 并报告 bug"）。该派生与 "feature coverage" 这个原始 construct 之间的距离值得审。

### Findings

- **Major B-17（bug-detect 不是 "feature coverage"，归 SUCCESS 让 construct 复杂化）**：reviewer 角度——"feature coverage" 是 "tool can ingest construct"，"bug-detect" 是 "tool can ingest AND complete verification AND report violation"。后者**强于**前者，归同一个 SUCCESS 桶让汇总数字含义模糊。v6 §3 排序里 miri 97% 与 cargo-check 100% 同列，但 cargo-check 的 SUCCESS = "rustc accepts"、miri 的 SUCCESS 含 "rustc accepts AND miri executes AND (no UB OR UB found in entry)"——三种 construct 同表呈现读者无法解开。**R12 反驳点**：架构 §一 P35 表只列出 miri / soteria 真正受影响，verifast / kani 等列 "0 触发"——但凡有 1 触发就破 construct 单一性。建议：在 v6 主报告 §3 表后加一栏 "SUCCESS includes bug-detect?"（miri / soteria 标 yes，其余 no），或把 bug-detect SUCCESS 单独抽出。

- **Minor B-18（P35 派生与 §六"前端测量"在 miri / soteria 上的内部张力）**：principles.md §六说"求解层不计入"，但 miri / soteria 的 bug-detect 是求解层 / symex 层产物——P35 显式让求解层结果反向影响 SUCCESS。architecture.md §一第 101 行自陈"这条派生与 §六 不冲突，是 §四 B 在 v6 corpus 上的具体投影"——但 §六 是 "前端测量"，§四 B 是 "测必要条件"，两者在 P35 这里冲突而非互补。**R2 反查**：已文档化的 contract 不等于内部一致——这是宪法内部的张力，应该在 principles.md 显式调和（要么 §六允许 bug-detect 例外、要么 §四 B 不延伸到 bug-detect）。挑刺站得住。

---

## Summary

### 总计 18 条 Findings

| 严重度 | 编号 | 主题 |
|---|---|---|
| Major | B-1 | feature coverage 命名与 operationalization 不一致 |
| Major | B-2 | cargo-check 与 verifier 工具同表对比 construct 混淆 |
| Major | B-4 | "前端" construct 跨工具异质性（7 切点 vs 4 全过程） |
| Major | B-5 | 求解切点宽窄差异 confound pass-rate 解释 |
| Major | B-7 | charon-mono / poly "形式可证 0 漏报"措辞过强（R7 应一并降级） |
| Major | B-8 | cargo-check "形式可证 0 漏报 + 漏报盲点：无"过强、违 §六"不藏" |
| Major | B-11 | "工具菜"等口语 register 破学术外观 |
| Major | B-13 | kani P37 "反向证明"实测 0 反例 ≢ 形式正确，未传播至主表 |
| Major | B-14 | charon 0 漏报论证未做 silent skip-item 穷举 |
| Major | B-17 | bug-detect = SUCCESS 让 SUCCESS construct 异质聚合 |
| Minor | B-3 | feature 类目内部异质（语言层 vs 工具边界混合） |
| Minor | B-6 | rocq-of-rust "前端 = 全过程"自陈与 §六前后端二分冲突 |
| Minor | B-9 | （正向 baseline）多工具已采"实测 + 不可形式证明"双标 |
| Minor | B-10 | "反向证明"应改名 source-filter heuristic |
| Minor | B-12 | "本地性"四原则栈缺学术先验引用 |
| Minor | B-15 | verifast vacuous-pass gate 仅在 corpus-0-spec 假设下 0 漏报 |
| Minor | B-16 | miri UB-in-vendored-crate 边界未明示 |
| Minor | B-18 | P35 bug-detect 派生与 §六 前端测量原则内部张力未调和 |
| Style | （含 B-11 register 建议表） |  |

**Major = 10 / Minor = 8 / Style = 1（合在 B-11 内）**

### 评审建议

**Major revision.**

理由：
1. construct 异质聚合（B-2 / B-4 / B-17）是 ISSTA reviewer 最直接的攻击面——主表把不同 construct 的数字同列呈现，读者会取出系统性误读的结论。这超出 R1（现象 ≠ 缺陷）；不是观察现象，而是 construct 表达层的设计问题。
2. 强措辞遗留（B-7 / B-8）违反作者自己已确立的 R7 invariant（弱化后不得恢复强措辞）——既然 aeneas 4 backend 在 R7 已经降级，charon × 2 与 cargo-check 同 pattern 应同步处理，否则破 self-consistency。
3. register-level 问题（B-11）单独看是 Minor，但密度高（principles.md / v6 主报告 / 部分 cc-reports 都有），构成系统性 noise。
4. 多数 Minor 已被 cc-reports 内部 R7 audit 抓到并诚实声明（B-9 baseline 是正向证据）——这说明项目有 self-correction 能力。建议 PC 给一轮 major revision 而非 reject。

**不建议 reject 的理由**：principles.md §六 "Oracle 责任 / 不冤枉 / 不藏" + R7 cc-route audit 历史 + cc-reports 多家诚实声明盲点——construct 表达层的瑕疵都有作者侧的自检机制，作者已表现出愿意按反馈降级措辞 / 增补 disclaimer 的 track record。Major revision 后可接受。
