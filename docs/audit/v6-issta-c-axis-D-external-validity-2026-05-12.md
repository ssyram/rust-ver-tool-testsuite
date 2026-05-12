# Axis D (ISSTA External Validity) — v6 final / commit ebe6858

审查角色：ISSTA 2026 reviewer，按 **Wohlin Ch.8.7** External Validity 标准
审查日期：2026-05-12
审查方法：disprove-first，每候选先反挑刺、再反反 counter（精神性陈述检索宪法 + README + architecture）
审查输入：`feature-coverage-2026-05-12-v6.md` / `deep-reports/cc-reports/*.md` × 20 / `runs/run-1778560393-59119/report.md` / `principles.md` §四 B + §六 / `architecture.md` §一

---

## 0. 总览

External validity 中心问题：结果 generalize 到 (a) 更广 Rust 工具生态 / (b) 更广 Rust 代码生态吗？跨工具比较公平吗？

| # | 候选 | 状态 | 严重度 |
|---|---|---|---|
| EV-1 | 通过率排序表呈现 vs §四 B "不区分翻译深浅" 张力 | **PARTIAL VALID**（文案 vs 形态分裂） | high |
| EV-2 | §六 三路径（A/B/C）公平性（aeneas filter vs kani grep vs single-file 自然满足） | **NOT VALID**（counter 成立：路径选择是工具输出形态投影，不是偏袒） | low |
| EV-3 | bug detect = SUCCESS（P35）跨工具不可达性 | **PARTIAL VALID**（架构必然，但读者解读风险存在） | mid |
| EV-4 | 工具 toolchain pin / wrapper 性质混入"工具能力"读数 | **VALID**（prusti 老 nightly / kmir 官方 wrapper 易 crash 在主表不可见） | high |
| EV-5 | 161 entries → crates.io ecosystem generalizability | **PARTIAL VALID**（concurrency 多 primitive / FFI / proc-macro / build-time 覆盖弱） | mid |
| EV-6 | 通过率表 ranking bias 残留 | **VALID**（即使有 disclaimer，table 形态本身暗示 ranking） | high |

总计：3 high / 2 mid / 1 low。**不撼动数字正确性**，撼动数字**对外可读性**（external validity 核心）。所有候选过 R7-R12 反状态。

---

## 1. EV-1：通过率排序表 vs §四 B "不区分翻译深浅" 张力

### 1.1 现象

`feature-coverage-2026-05-12-v6.md` §3 表（line 103-124）按 success rate desc 排序：
- cargo-check 100% / miri 97% / charon × 2 95% / kani 93% — **浅** 工具占前 5
- verifast 8% / kmir 37% / verus 40% / aeneas × 3 60% — **深** 工具占后段

宪法 §四 B 等距推论（principles.md L75-76）：
> "测'接受 vs 不接受'，不区分翻译深浅（浅 syntactic 搬运 / 深 MIR 翻译 / verifier dialect 接受一律不加权）"

### 1.2 reviewer 挑刺

按 ISSTA Wohlin 8.7：若实验结果**呈现形态**与**声明的测量语义**不一致，结果对外不可解读（threat to external validity: "interaction of selection and treatment" — table 把异质 treatment 收编进单一 metric 形态）。

> 你既然不区分深浅，那把通过率排序展示有什么用？读者看到"cargo-check 100% > verifast 8%" 自然推断"cargo-check 比 verifast 强 12 倍"——但你 §四 B 又说不区分深浅。文案承诺与表格呈现互相打架。

### 1.3 counter（反反挑刺）

cargo-check 报告 §"矩阵角色"（cc-reports/cargo-check.md L14-21）已声明：
> "cargo-check 在 20 工具矩阵中的定位与其他 19 个验证工具不同 / 不参与'工具特性覆盖率'维度的比较 / 本报告刻意不与其他 19 工具做通过率排名"

主报告 §0 顶部 (L5) + 项目 README 顶部均挂"不构成工具能力客观评判"免责。

### 1.4 counter 是否站住？

**仅部分站住**。精神性陈述检索：

- cargo-check 那 1 行已自陈 "baseline 角色"——✓
- 但其他 19 行（kani 93% / miri 97% / charon 95% 等）无类似自陈
- §四 B 在表中**未显式应用**——表没标注哪些是"前端深翻译"、哪些是"前端浅 typecheck"
- ISSTA 标准下：reviewer 没义务读 20 份 cc-reports 才能正确解读主表。**主表本身**必须自描述 enough，让 ISSTA reader 不被误导

**精神性 vs 直陈性**（charter-craft §4.5）：宪法精神已立"不区分深浅"，但表呈现层缺乏对应锚定。这不是宪法层缺陷，是 v6 主报告的**应用层缺陷**——精神入宪未传导到呈现。

### 1.5 修订建议（非决策点，文案级）

- 主报告 §3 表头加注："本表按通过率排序仅作浏览索引，不构成深浅工具对比；按 principles.md §四 B 等距推论，cargo-check / charon / miri 等前端 typecheck 工具与 aeneas / verifast / verus 等深翻译工具不可同维度比较"
- 或者：把 cargo-check 抽出独立标"baseline 角色"小节，与其他 19 工具表分离呈现

**状态**：PARTIAL VALID。不撼动 v6 数字。撼动 ISSTA 读者可读性。

---

## 2. EV-2：§六 三路径公平性（A/B/C）

### 2.1 现象

architecture.md §一 #§六 (L77-81) 落地三路径：
- 路径 A：aeneas 4 backend — charon stderr 带 source path → 路径前缀过滤
- 路径 B：kani — 5-markers 无 span → os.walk entry crate `src/` + 关键字正则反向证明
- 路径 C：verus / verifast / soteria / ror — 单文件 pipeline 不读 deps → 自然满足

### 2.2 reviewer 挑刺

路径 B 关键字 grep 可 false positive（注释 / 字符串字面量含 `asm!` 但实际不 codegen）/ false negative（macro 间接展开成 `asm!` 但源码无此 token）。这是给 kani **占便宜**（豁免范围偏宽）还是 **吃亏**（豁免范围偏窄）？

进一步问：路径分配本身是否偏袒？aeneas 用 filter（删 partial）vs kani 用反向证明（豁免 partial）——前者保守减 SUCCESS，后者保守加 SUCCESS。

### 2.3 counter

精神性陈索 architecture.md §一 #§六 (L80-81)：
> "路径 A 输出带 source path / 路径 B 输出无 span"

**路径选择不是设计偏袒，是工具输出形态的客观投影**：
- charon 输出格式确实带 `--> path:line:col` → 唯一合理实施是 path filter
- kani 5-markers 输出确实只 aggregate count 无 span → 只剩反向证明这条路

kani cc-report L62-63 已诚实声明：
> "P37 反向证明的关键字正则保守宽（偏向 false-positive FAILED 而非 false-negative SUCCESS），但非形式证明，仅实测校准"

即 kani 路径在边界上**偏向 conservative FAILED**——是吃亏不是占便宜。kani 98.8% 通过率不是路径 B 给的便利，是 5-markers 集合本身狭窄（5 个 markers vs verus 整套 VIR check）。

### 2.4 counter 是否站住？

**站住**。

- "路径分配 = 工具输出形态投影"——精神已明示（architecture §一 #§六 L78-81 文本）
- "保守偏 FAILED"——kani README 已诚实声明（cc-reports/kani.md L62）
- false positive / false negative 边界 R5 audit 已验证（详 P37 落地 docs）

剩余 ISSTA 关切（"读者无法独立验证'保守'声称"）属于 oracle 的**形式证明 vs 实测校准**张力——v6 已诚实标"实测校准非形式证明"，不是宪法层缺陷。

**状态**：NOT VALID（counter 成立）。

---

## 3. EV-3：bug detect = SUCCESS 跨工具不可达性

### 3.1 现象

architecture.md §一 #"bug detect 归 SUCCESS"（L86-101）适用工具：
- MIRI / soteria — 适用，wrapper 翻转
- verifast — corpus 0 spec 注解 → 0 触发
- kani / verus / creusot / prusti / ror — 设计跑前端层（`--only-codegen` / `--no-verify`）→ **0 触发**

→ MIRI 98% / soteria 78% 中含 bug-detect 翻转的 SUCCESS；kani / verus / creusot / prusti / ror 永远不在 bug-detect 通道获得 SUCCESS。

### 3.2 reviewer 挑刺

一个 verus 开发者看到"verus 41% / miri 98%"会怎么解读？两者通过率口径不可比——miri 跑完整解释 + UB detect + 翻 SUCCESS；verus 仅跑前端 + 不 verify。两者通过率在同一 metric 下呈现是**testing-treatment interaction**（Wohlin 8.7 经典 threat）。

### 3.3 counter

精神性陈索：
- principles.md §六 (L108-110)：前端测量深度切割——求解层不计入
- architecture.md §一 (L87-90)：bug detect = "工具完整跑完前端 + 求解，且自陈检出 bug 时归 SUCCESS"——这是 §四 B "测必要条件 / 非语义对错" 在符号执行类工具上的派生

精神层无张力：miri / soteria 是符号执行 / 解释引擎，其前端 = 全过程（cc-reports/miri.md L15 / cc-reports/soteria.md L31）。kani / verus / creusot / prusti / ror 设计为前端剪开，前端不含求解。两类工具的"前端边界"位置不同——这是**工具架构差异**，不是项目偏袒。

cc-reports/miri.md L17 已诚实声明：
> "MIRI 本身没有'前端 / 后端'分界——它就是 MIR 解释器，工作内容 = 解释执行 + UB 检测"

cc-reports/soteria.md L33-34 同样诚实：
> "soteria 没有'只翻译不执行'的 dry-run flag"

### 3.4 counter 是否站住？

**部分站住**。精神层不破，但 ISSTA 读者解读层仍有风险——主表 §3 表里不带"测量边界差异"列。

- ✓ 精神已明示（"前端边界 = 工具自身设计的前端"）
- ✗ 主报告 §3 通过率表未显式标"测量边界差异"——读者无法在主表层面区分 miri (interp 引擎全程) vs verus (前端 typecheck-only)
- 验证工具用户（verus / kani / creusot 开发者）会自然误读

**修订建议**（非决策点）：主报告 §3 表加列 "测量边界"（如 "rustc 前端" / "前端 typecheck" / "完整解释" / "前端 codegen + 反向证明"），或在表注里显式分组

**状态**：PARTIAL VALID（精神不破 / 呈现层风险存在）。

---

## 4. EV-4：toolchain / wrapper 性质混入"工具能力"读数

### 4.1 现象

精神性陈索 principles.md §六 "本地性原则" + architecture.md §一 P27 修宪后的 oracle 设计：
- prusti 锁 nightly-2023-08，拒 edition 2024 → FAILED（本地性原则下站得住）
- kmir 通过率 37% 主要因官方 `kmir/cargo.py` wrapper 易 crash → FAILED（按 D2 "官方 wrapper 失败 = 工具锅"）

### 4.2 reviewer 挑刺

ISSTA 读者看 prusti 44% / kmir 37% 会读成"工具特性覆盖差"——但实质是：
- prusti **能力本身**没差到 44%，是 2023-08 toolchain 不识 edition 2024 与新 std API
- kmir **能力本身**没差到 37%，是 stable-mir-json schema 漂移 + 官方 Python wrapper 维护滞后

这两个 conflation 在主表不可见。**threat to construct validity 同时也是 external validity**（"通过率"作为 construct 不能跨工具同 metric 解读）。

### 4.3 counter

精神性陈索：
- principles.md §一 #根本问题 2 + 三原则栈（L21-25）：本地性 / 当前性最高——"测当下这个工具版本 + 它要求的 toolchain 能做什么"
- principles.md §六 UNKNOWN 严格语义（L106）："工具自身能力边界（包括但不限于：官方 wrapper 失败 / 工具自选 toolchain 不支持新特性 / 工具单文件 pipeline 不读 Cargo.toml / 官方 wrapper 不传 --edition）一律 FAILED——按本地性原则 FAILED 站得住"

宪法**显式预见**这个 reviewer 关切并入宪给出立场：工具开发者选择 pin 老 toolchain / 自带 wrapper 易 crash 是工具的设计选择，结果反映到通过率上是"测量结果对工具版本快照诚实"——不是测量偏袒。

cc-reports/prusti.md L6-7 / cc-reports/kmir.md L30-34 都已逐工具诚实声明这些 conflation。

### 4.4 counter 是否站住？

**站住但不充分**。精神层立场清楚（本地性原则覆盖），但 ISSTA 读者解读层仍是问题：

- 通过率数字本身正确 ✓
- "本地性"立场宪法已立 ✓
- 但 ISSTA 读者**没有在主表层接触到这条立场**——只在每个工具 cc-report 里
- 主报告 §3 表无"toolchain pin 状态" / "wrapper 维护方"列

**reviewer 反反**："本地性 + FAILED 站得住"是项目对**测量公信力**的立场（不公信问题），不是对**reader 解读**的承诺。reviewer 关切是 reader 误读——两个是不同维度。

**修订建议**（非决策点）：主报告 §3 表加列 "toolchain pin"（如 prusti: nightly-2023-08）+ "官方 wrapper 维护状态"（如 kmir: stale）。或在 §0 元数据里集中列。

**状态**：VALID。

---

## 5. EV-5：161 entries → crates.io ecosystem generalizability

### 5.1 现象

161 entries 覆盖 41 features。Per-feature 通过率 §"Per-feature summary" (run report L65-) 反映各 feature 通过情况。

reviewer 关切：从 161 entries 能 generalize 到 crates.io 2026 200k+ crates 的"Rust 验证工具能力"吗？

### 5.2 reviewer 挑刺

抽样系统性问题（Wohlin 8.7 sampling representativeness）：
- concurrency 仅 40 task（4 entry × 10 tool）→ Atomics / Mutex / channels / async-runtime / rayon 等多 primitive 未细分
- unsafe pointers / raw pointer 操作覆盖薄
- FFI 仅 `kani-limit/extern-ffi/trigger_call_libc_abs` 一个 case
- proc-macros 未覆盖
- build-time codegen（`build.rs` / `include_bytes!`）未覆盖

→ 161 entries → wider Rust ecosystem 的 generalize 度有 sampling bias

### 5.3 counter

精神性陈索 principles.md §三 (L54-60)：
> "examples（样例库）：first-class / 长期承诺"
> "examples 多样性：样例覆盖单特性、边界（`*-limit/`）、综合（如 `industrial/`）多个梯队"

宪法立 examples 多样性 + 三梯队（单特性 / *-limit / industrial）——多样性精神已明示，但**没有承诺 crates.io 完整覆盖**。

principles.md §二 (L41)：
> "范围：项目目标是框架；测试报告是框架的应用产物，随工具版本浮动。工具能力的客观判定不在项目范围"

→ 项目**显式声明**不做工具能力客观判定。generalize 到 crates.io 不是项目承诺。

### 5.4 counter 是否站住？

**精神层站住**（项目自陈不做客观判定）。但 ISSTA 评审视角看：

- ✓ 项目自承范围 = 框架 + 应用产物快照
- ✓ corpus 三梯队架构合理
- 部分问题：concurrency / FFI / proc-macros 等 feature 覆盖仍薄。这是 corpus **进化**问题，不是 v6 撼动问题
- v6 主报告 §0.2 已锚定 corpus 大小 (161 entries × 41 features)——读者知 sample size

**修订建议**（非决策点 / 长期）：corpus 扩 concurrency primitive 多样性 / FFI / proc-macros 类别。

**状态**：PARTIAL VALID（精神层不破 / corpus 进化点存在）。

---

## 6. EV-6：通过率表 ranking bias 残留

### 6.1 现象

v6 主报告 §3 表按 rate desc 排序，cargo-check 100% 第一行。即使顶部有 "不构成工具能力评判" 声明，table 形态本身带 ranking 暗示。

### 6.2 reviewer 挑刺

Wohlin 8.7 conclusion validity & external validity 交叉：**呈现形态对读者解读**强于 disclaimer 文字。心理学经验：reader 看表第一眼提取的是 ranking，disclaimer 阅读率低。

→ "100% > 8%" 视觉投射 = 12.5× 能力差距 = 跨工具不公平比较——这是 §四 B "不区分翻译深浅" 的实际背叛。

### 6.3 counter

精神性陈索：
- principles.md §二 (L49)：测试结果可重现 / 锚定时间 + 版本
- principles.md §六 时空锚定（L98-100）："工具能力观察必须锚定 (工具版本, 时间) 二元组"
- v6 报告 §0.2 / 顶部声明锚定 run id + 工具版本快照

精神层**承认时效性 + 不构成长期承诺**——但**没有反对 ranking 呈现**。

cargo-check cc-report L20-21 已自陈"baseline 角色 / 本报告刻意不与其他 19 工具做通过率排名"——但**那是 cc-report 单工具自陈，不传导回主表层**。

### 6.4 counter 是否站住？

**不站住**（candidate VALID）。

- 精神层有"时空锚定 + 不构成长期承诺"但**没明示反 ranking 呈现**
- ISSTA reviewer 关切：disclaimer 是法律层免责，table 形态是呈现层灌输，二者不对称——**呈现形态压过 disclaimer**
- cargo-check 单工具自陈"不参与排名"——但**实施层**它仍在表第一行，这是名义自陈与实施分裂
- 反 R11："文案 vs 实现不一致" 直接命中——disclaimer 文案承诺"不构成评判"，table 实施呈现 ranking ⇒ 文案 vs 实施分裂

**修订建议**（非决策点 / 文案 + 形态级）：

1. 主报告 §3 表改按**字母序**或**按工具集团分组**（前端 typecheck / 前端深翻译 / 完整解释 / 完整符号执行）排序，破 ranking 视觉
2. 或者：表头明示 "本表按通过率排序仅为浏览索引；按 §四 B 等距推论，工具间不构成 ranking"
3. cargo-check baseline 单独段落，不入主表

**状态**：VALID（文案与形态分裂是实测可见的 R11 命中）。

---

## 7. 反状态总结

| 反状态 | 触发 | 处理 |
|---|---|---|
| R7（弱化 reject） | 无候选被弱化重写 | ✓ |
| R11（文案 vs 实现不一致） | EV-6 命中：disclaimer "不构成评判" vs table ranking 呈现分裂 | 修订建议给出（非决策点 / 文案形态） |
| R12（作者反驳） | 作者可援引：（a）§三 模块定位"次要模块时效性"——通过率是次要模块快照，时效性 disclaimer 充分；（b）§六 时空锚定——锁定 (工具版本, 时间) 二元组解释力衰减是显式承诺 | 反反：本审查 ISSTA reviewer 视角强调 reader 解读层，作者反驳援引宪法精神**对内**有效但**对外** ISSTA reviewer 关切的是呈现层；二者不对称——作者反驳不撼动 EV-1 / EV-3 / EV-4 / EV-6 的呈现层关切 |

---

## 8. 落地建议（非决策点 / 文案形态级）

| 候选 | 状态 | 建议 |
|---|---|---|
| EV-1 | PARTIAL VALID | 主报告 §3 表头加 §四 B 注 + 按工具集团分组或字母序 |
| EV-2 | NOT VALID | 无 |
| EV-3 | PARTIAL VALID | 主报告 §3 加 "测量边界"列或表注分组 |
| EV-4 | VALID | 主报告 §3 加 "toolchain pin" + "官方 wrapper 维护状态" 列或 §0 集中列 |
| EV-5 | PARTIAL VALID | corpus 长期扩 concurrency primitive / FFI / proc-macros |
| EV-6 | VALID | 主报告 §3 表改字母序 / 集团分组排序 破 ranking 视觉；cargo-check baseline 单列 |

**4 个 high / mid 候选共同主题**：宪法精神（§四 B 不区分深浅 / §六 本地性 / 时空锚定 / cargo-check baseline 角色）入宪充分，**但传导到主报告 §3 表呈现层不充分**——这是 v6 主报告应用层缺陷，不是宪法层缺陷。

**修订动作**全在主报告 `feature-coverage-2026-05-12-v6.md` §3 文案 + 形态层，**不撼动**：
- v6 数字正确性（2202 / 1008 / 10）
- 宪法 §四 B / §六 / §一 任何条
- architecture.md §一 路径 A/B/C 分配
- 20 个 cc-reports 内容

---

## 9. 决策点说明

按 `/workflow` §决策点 vs 非决策点：

**无决策点**。所有 6 个候选都派生自既有宪法精神（§四 B 不区分深浅 / §六 时空锚定 + 本地性 / §三 模块定位）。修订建议都是文案形态层应用——不需要用户裁决。

建议作者按 §8 落地 3 个表层修订（§3 表分组排序 + 加测量边界列 + toolchain pin 列），其余 cc-report 内容保持不变。
