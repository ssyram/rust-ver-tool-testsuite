# 工具集成原则与 README 书写规范

> **前置约束**：本文是 [`principles.md`](principles.md) §六-本节自我性声明 + §三-3-原则 3 的展开——是**我方维护 tools 集成与实测报告**的具体实现原则。
>
> **自我性**：本文是我方在使用本框架时的方法学选择。**外来 tools 无须遵守**；他人复用本框架可做不同选择（如纳入 SMT、做下限测试等），仅需遵守 [`principles.md`](principles.md) §四 三大派生原则。
>
> 索引位置：[`architecture.md`](architecture.md) §八 下游索引。

---

## 一、工具版本锁定

每个 `tools/<name>/README.md` 必须明确：

- 上游仓库 URL
- 锁定的 commit hash / 版本号 / brew tap / nightly toolchain pin
- 安装方式（按上游文档自行安装；本项目不提供安装脚本——避免在工具版本变迁后误导）

`tool.toml` 必须含 `version_command`——runner 在每次 run 自动捕获工具版本字符串到 `results.json` metadata 段，按宪法 §五"工具非静态原则"裸数据自描述。

---

## 二、SUCCESS 信号的形式指标声明

每个 `tools/<name>/README.md` 必须明示"形式指标"——即 oracle 用什么具体可机检条件判定 SUCCESS。形式指标可以是：

- **exit code 单一信号**（如 cargo-check / kani / verus / aeneas / charon）
- **exit code + 产物字面 grep**（如 hax × 3 / rocq-of-rust）
- **exit code + stderr 模式 grep**（如 kmir 的 `#EndProgram ~> .K`）
- **多门组合**（如 rocq-of-rust 的 6 道门：exit 0 + 至少一个 .v + 无 0-byte + > 200B + 无 silent marker + entry_fn `Definition` 存在性；P15 引入 gate 6 + N=7 attempts AND-reduce）

工具用何种形式指标由工具自身设计决定（按宪法 §三-3-2.a），但必须在 README 显式列出。

---

## 三、0 误报的论证（必须）

**0 误报的定义**：oracle 判 SUCCESS 时一定是真 SUCCESS（不冤枉工具能力）。

**这是必须论证的硬指标**（按宪法 §三-3-2.b 上限保证）。论证形式不限定——**任何能提供足够证据的方式都可**：

- **反向证明**（最常见、最简单）：论证 oracle FAILED 时一定说明工具内部有问题，即 oracle FAILED ⟹ 真 partial。这种"satisfy 不一定无问题，但不 satisfy 一定有问题"的非对称是常见的工程论证形式
- **源码层穷尽**：grep 工具源码所有 oracle FAILED 触发点，每点对应真 partial
- **双向蕴含**：oracle SUCCESS ⇔ 工具内部某条断言成立（更强但代价大）
- **实测验证**：在足够多样的合法 SUCCESS 样例上验证 oracle 不误报（不构成形式证明，但提供足够经验证据时可接受）

**具体到不同形式指标**：

- **exit code 单一信号**：论证"工具的 exit ≠ 0 路径"上每个源码触发点都对应"真有 unsupported / reject"。例：
  - aeneas：`Main.ml:773` `if has_errors then exit 1`，`error_list` 仅由 `craise` 累加；论证 `craise` 只在识别 unsupported 时调用即可
  - charon `--abort-on-error`：`register_error!` 在第一次错误就 panic；论证 `register_error!` 只在真 unsupported 时调用即可
  - creusot：`crash_and_error / span_err / span_fatal` 把 unsupported 升 rustc error；论证这些 API 只在真 unsupported 时调用即可
- **exit code + 产物 grep**：论证 grep 命中位置都对应"真 partial"。例：
  - hax-lean：`text!("sorry")` 在 `lean.rs:1287, 2163` 调用——这两行都是 `PatKind::Error / error_node` 路径，确实是工具内部识别为 partial 时才触发
  - rocq-of-rust：explicit comment marker `(* Error / Unexpected ... *)` 由 `emit_warning_with_note` 在 thir 编译失败时生成，对应真 partial

每个 ⚠️ 通过实测（而非源码层穷尽）论证 0 误报的工具必须在 README 列出双向实测证据（按 §四-4.2 反误报检查）。

---

## 四、0 漏报的论证与防御（最好但不强求）

**0 漏报的定义**：oracle 判 SUCCESS 时真的没漏抓任何 partial（不高估能力）。

**这是软指标**（按宪法 §三-3-2.c 下限诚实）——**漏报是允许的**，只是尽量不要。优先级：能形式证明最好；不能则提供防漏报机制并声明盲点。

### 4.1 形式证明（最理想）

证明工具内部所有 unsupported 路径都被 oracle 抓住：

> 真 partial ⟹ oracle FAILED

或等价：

> oracle SUCCESS ⟹ 真 SUCCESS

适用工具：内部所有 unsupported 通过 exit 通路单一暴露的工具：

- **aeneas**：唯一 unsupported 入口是 `craise`，唯一 exit 决定是 `Errors.error_list` 非空 → exit 1，单一通路 ✅
- **charon `--abort-on-error`**：`register_error!` 是唯一 unsupported 入口，加 abort 后 panic 是唯一 exit 决定 ✅
- **creusot**：所有 unsupported 用 `crash_and_error / span_err / span_fatal` 升 rustc error，单一通路 ✅

### 4.2 防漏报机制 + 反误报检查（次优）

不能形式证明所有 silent path 都被覆盖时（如 grep-based oracle），可提供**防漏报机制**——产物 grep / 多门组合等启发式 —— 抓**已知** silent path。

**关键约束（与 §三 0 误报硬指标共享）**：防漏报机制**绝不能反向引入误报**。每个 grep marker 选取必须经过**双向实测**：

- 已知 silent path → grep 命中（防漏报有效）
- 合法代码 / 合法注释 / prelude / 用户合法字面 → grep 不命中（不引入误报）

**两边都通过**才能上线。例：hax-lean 的 grep `(:=|pure|mk|,)\s*sorry\b|\bsorry\s*[,)\]]` 经实测：
- `(pure sorry)` / `mk sorry` 命中 ✓
- 用户合法 `let sorry : i32 := 5;` + doc comment 含 sorry 字面 不命中 ✓

**不能两边都通过的 marker 不准上线**——宁可保留漏报盲点（按 §4.4 诚实声明），不可引入误报。

### 4.3 防漏报机制的意义辨析

防漏报机制（grep / 多门组合）的意义分两层：

- **理论意义有限**：它们**无法形式证明 0 漏报**——总有未观察到的 silent path 形式，机制只能覆盖**已知**的那些。理论上不构成 SUCCESS ⟹ 真 SUCCESS 的形式蕴含
- **实践意义存在**：在已知 silent path 上经过双向实测（按 §4.2）后，机制确实能命中——这是实测可验证的有效性

工程上完全值得做（比无机制好得多）。工具 README 应**清楚区分这两层**：陈述当前实测有效性，并同时按 §4.4 列出已知盲点。**避免**把实践有效性误述为形式保证。

### 4.4 漏报盲点的诚实声明（兜底）

任何工具不可形式证明 0 漏报、且防御机制覆盖不全时，必须在 README 列出**已知漏报盲点**——理论上可能存在但当前 oracle 抓不到的 silent path 类型。读者引用该工具支持率时知道有这些盲点。

例：

- **hax-lean**：hax engine 完全 skip item（item 既不写 sorry 也不发 Diagnostic，不出现在产物里）；上游引入新 silent path 而 grep 滞后
- **rocq-of-rust**：上游引入新 silent fallback 路径不带已知 markers；完全 skip item 类（合理 skip，不算漏报）

---

## 五、README 必含章节清单

每个 `tools/<name>/README.md` 必含以下章节（按此顺序）：

1. **简介** + GitHub 上游 URL + 工具的 elevator pitch
2. **本测试集中的"前端接受"定义** + pipeline 阶段图 + 我方切割点
3. **SUCCESS 信号** —— 按 §二 形式指标
4. **形式严格性** —— 按 §三 / §四 声明 0 误报 / 0 漏报状态 + 防漏报机制 + 漏报盲点
5. **安装** —— 锁定版本，按 §一
6. **本框架配置** —— `tool.toml` 关键参数 + `entry_mode` / `extra_cargo_deps` / `version_command` 等
7. **已知限制 / 坑** —— 平台限制、依赖限制、工具自身已知 bug
8. **关联 sub-tests** —— 是否有 `examples/<tool>-limit/` 类目

---

## 六、禁忌（自我性的体现）

工具 README **禁止**：

- 对工具能力下绝对结论（如"工具 X 不支持特性 Y"）—— 改为"我方测试方法学下，X 在本 corpus 的特性 Y 上 FAILED"
- 暗示客观真理（如"工具 X 是错的"）
- 跨工具能力排序（如"X 比 Y 更好"）—— 框架不评比，只测 entry 级二值信号

按 [`principles.md`](principles.md) §三-3-1 时效性，所有工具陈述都锚定具体时间 + 工具版本组合，不构成长期承诺。

---

## 七、实测报告原则

> 本节展开 [`principles.md`](principles.md) §三-3-原则 3 的"实测报告责任边界"。

### 7.1 约束对象明示（首要澄清）

`principles.md` / `docs/design/` 的"克制工具评判"精神，**约束对象**是：

- 工具集成的**设计认知** —— 避免把实测当作"工具能力客观判定"灌进设计层
- **后续基于本框架开发**的人 / AI —— 避免把数字硬塞进工具改进 / 认知模型
- 对工具开发者的**外溢免责** —— 挂在 [`README.md`](../../README.md) 顶部一次性陈述（避免无意得罪）

**不约束**：

- 报告书写者**对实测数据本身的诚实汇总与认知陈述**——基于数据归纳"X 在 corpus Y 下支持率 Z%，partial 模式集中在 W 类型"是诚实汇总，不是越界
- 报告书写者基于数据给出**自身认知与判断** —— 锚定 corpus / 时间 / 版本即可
- 报告**正文**无须反复自贬 / 加"不构成评判"声明 —— README.md 一次性免责已足够，正文写自然些

**不要**把"克制评判"误传到报告书写本身上，把报告写成无认知的快照陈列——那不是谨慎，是放弃汇总责任。

### 7.2 真实性诚实责任

- 数据基于 raw 可追溯（不读 raw 不写论断）
- 不夸大数字、不主观加权 / 排序
- 失败模式归类基于实读 stderr，不臆测

### 7.3 报告头部必含锚定信息

每份基于工具运行的结果性文档必须含：

- run id / ISO 8601 时间戳
- 工具版本（按 `results.json` metadata）
- corpus 范围（feature × entry 数）
- 链接到 [`README.md`](../../README.md) 顶部的一次性免责声明（正文无需重复）
