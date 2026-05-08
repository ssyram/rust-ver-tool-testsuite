# 宪法级精神原则

> 项目的精神宪法。所有架构、实现、配置都必须 100% 满足本文档。任何后续设计与代码以本文档为标准。如本文与下游设计或代码冲突，**改下游不改本文**；除非通过显式讨论修订本文。
>
> 本文按"问题意识 → 项目目标 → 模块定位 → 派生原则 → 硬指标 → 外围原则"的顺序展开（参见 `/principle-derivation` 与 `/workflow` 规范）。

---

## 一、根本问题意识

当前 Rust 验证工具生态已极度分化——Kani 写 `#[kani::proof]`、Verus 写 `verus! {}` 宏块、Prusti / Creusot 写 caller 上的 `#[ensures]`、MIRI 当解释器跑 `fn main()`、Charon 翻译到 LLBC、hax / aeneas / rocq-of-rust 翻译到 Lean / F\* / Coq……每个工具各自有声明、各自有测试、各自有 README 列出"支持哪些 Rust 特性"。

但**社区缺一个统一的特性覆盖率测试框架**——能让用户（选型）与开发者（自检）都**清晰看到工具的能力边界与发展趋势**，且在不同工具间**可比较**。

任一现存路径——把样例改成工具特定形态、或把样例围绕统一抽象 API（如 propverify / verifier crate）写——都把"不公平"沉淀进样例本身：样例污染、抽象成新依赖、每加一工具都要改样例。先行尝试 RVT (project-oak/rust-verification-tools)、soarlab/rust-benchmarks 都走这条路，都已归档或低活跃。

**本项目的核心意识**：把不公平沉淀到第三方中介层（runner + 配置 + 模板），让样例和工具两端各自纯粹。

---

## 二、项目目标

**构建一个工具特性覆盖率测试框架**，通过**样例驱动**的方式系统地测量和比较不同 Rust 工具的特性覆盖情况。

注意：**项目目标是"框架"，不是任何具体工具的测试报告**。测试报告是框架的应用产物，随工具版本浮动；框架本身才是项目的长期承诺。

---

## 三、模块定位与优先级（本文最关键约束）

项目分三个实现模块。**核心模块**（first-class）是项目长期承诺；**次要模块**（non-first-class）是核心模块的应用展示，不作为核心目标。

### 核心模块 1：测试运行与结果分析框架（`runner/`）

**职责**：自动化执行 (tool, entry) 三元组的隔离运行；采集 raw output；产出 `results.json` + `report.md`。

**原则：结果记录严谨全面**

必须客观认知测试结果的时空等基础测试属性，**尽可能穷尽**测试结果的时空等前提维度：

- **工具版本**：每次 run 跑 `version_command` 采集每工具的版本字符串，存入 `results.json` 顶部 metadata
- **测试环境**：host info（hostname / os / arch / kernel / cpu / memory / num_cpus）+ ISO 8601 UTC 起止时间戳
- **测试样例特征**：entry id 形如 `<feature>/<dir>/<entry-fn>`，可由路径自描述

**含义**：脱离时间 + 版本上下文的"工具能力"结论无可重现性、跨时段对比失效。`results.json` 是裸数据自描述，事后或第三方解读不依赖外部上下文。

### 核心模块 2：样例库（`examples/`）

**职责**：提供测试样例。每个样例是独立 cargo lib，含零参 `pub fn` 入口。

**原则 A：单性质检验追求纯净性**

每个样例尽量只检验一个特性，以便清晰地归因测试结果。具体落地：

- 一 entry = 一独立 lib crate；**禁止跨 entry 共享 helper**——共享出问题会让多个 entry 同 fail，丢失工具间分化信号
- 例子源码 **plain Rust**，零 verifier 标记（`#[kani::proof]` / `#[ensures]` / `verus! {...}` 等都不允许）
- 入口为零参 `pub fn`，无 parametrised 形态

**原则 B：整体追求多样性**

样例整体追求更多样更全面的测试特性覆盖——**从单性质到综合的较困难、更接近实际使用的样例都有**，形成一个丰富的样例库。具体梯队：

- **单特性 entry**（基础层）：每个 Rust 特性一个最小 entry（int / vec / closure / lifetime / trait 等）
- **`*-limit/` 类目**（边界层）：故意触发某工具自声明的"不支持"特性
- **综合样例**（应用层）：多特性组合、接近实际使用的较复杂代码（如 `industrial/` 下的 vendor crate）

新增样例必须明确归属哪个梯队，并满足该梯队的纯净度要求。

### 次要模块 3：市面工具样例（`tools/` + 实测报告）

**定位**：本项目目前接入的 19 个具体 Rust 验证工具（cargo-check / kani / miri / charon × 2 / creusot / hax × 3 / aeneas × 4 / prusti / verus / verifast / soteria / kmir / rocq-of-rust）的配置与实测，**居于次要地位**。

提供非官方性质的测试，本质作为**工具使用展示** + **初步调研结果**——主要为了丰富样例库、展示测试框架的应用价值，**不作为项目核心目标**。

#### 原则 1：承认时效性

仅作为**当下**工具状态的快照，**锚定具体版本**：

- 每工具的 README 锁定 commit hash / brew tap version / nightly toolchain pin
- 每次 run 的 `results.json` metadata 段记录跑时具体版本
- 工具升级后，旧测试结果的解释力线性衰减——这是必然的，不是缺陷

测试报告本身不构成项目对工具能力的长期承诺；仅是某时刻某版本的实测快照。

#### 原则 2：诚实的测试范围

##### 2.a 以形式指标为最终解释

每项工具特性覆盖率的评判以**形式指标**（formal indicator）为核心——所谓形式指标即 oracle 用的具体可机检条件：

- exit code（cargo-check / kani / verus 等）
- 产物存在性 + 大小阈值（hax / aeneas）
- 产物字面 grep（hax-lean 的 `sorry`、rocq-of-rust 的 explicit comment markers）
- stderr 字面 grep（kmir 的 `#EndProgram ~> .K`）
- 多门组合（rocq-of-rust 6 道门）

**各项工具因其设计不同，无可避免取不同的形式指标**——SAT-based / SMT-based / 翻译类 / 解释执行类各自能暴露 partial 的方式不同，oracle 必须按工具自身设计精确切割。

**测试的能力边界 = 形式指标定义的诚实边界**。读者引用某工具的支持率时，必须知道该数字是基于哪个形式指标定义的边界。

##### 2.b 上限保证（不冤枉能力）

**oracle 0 误报第一**——把某 entry 标 SUCCESS 时**必须**是真 SUCCESS（工具确实接受了这段代码、未发生任何 partial）；不允许冤枉工具能力（不允许把真 SUCCESS 误判为 FAILED）。

由此推导："**测试支持率是真实能力的严格上限**"——工具实际能力 ≤ 测得支持率。读者拿到测试报告后**不会高估**工具能力的上限，工具发布人也**不会被冤枉**。

##### 2.c 下限诚实（不高估能力）

理想是**尽量无漏报**——把真 partial 错判为 SUCCESS 是漏报，会高估能力。

但**诚实承认某些工具确实没有提供足够的信号来完全排除 silent path 的潜在漏报**——典型：

- 翻译类工具中走产物字面 grep 检测的（hax × 3 / rocq-of-rust）：grep 模式经过实测验证 0 误报，但**不可形式证明**所有未来 silent path 都被覆盖
- 上游工具自身可能引入新的 silent path，oracle 滞后于上游修复

**通过实测验证来增强对漏报盲点的信心**——构造已知会触发 / 不会触发的 entry，验证 grep 在两类样例上分别命中与否。

**每个工具的 README 必须明确声明形式严格性状态**：(a) 是否能形式证明 0 误报，(b) 是否能形式证明 0 漏报，(c) 已知漏报盲点（如有）。这样读者在引用某工具的支持率时，知道该数字的可信度边界。

#### 原则 3：实测报告的责任边界

本项目产出的所有基于工具运行的结果性文档——`docs/test-reports/`、`deep-reports/cc-reports/`、`runs/<id>/report.md`——遵循：

1. **真实性诚实责任**：我方对报告内容的真实性负责，不夸大、不主观加权 / 排序，所有数据基于 raw 可追溯
2. **非评判性**：报告内部分析（特别是失败模式归类、跨工具发现等）**不构成对工具的实质评判**——
   - 不是工具本身的责任，工具开发者无须为内容负责
   - **不纳入**任何工具设计与实现考量
   - 都是**外来的**——某具体时间 + 工具版本 + corpus 下的观察快照

工具能力的客观判定**不在本项目范围**。详见 [`tool-integration.md`](tool-integration.md) §九"实测报告原则"。

---

## 四、三大派生原则

把"不公平沉淀到中介层、两端保持纯粹"这一意识落在三个面上时分别长成三条原则。

### 原则 A：双方都不可侵入

样例不是为某工具改的、工具被当作黑盒。异质性既不能落到样例端（破中立）也不能落到工具端（无法干预），必然落到第三方框架。

**位阶澄清**：A 保护的是**原始磁盘的样例源码**——`src/`、entry 函数体、属性标注一律不为工具改动；隔离副本上的 manifest 注入（`extra_cargo_deps`）与 entry 入口替换（`entry_mode = "lib"`）属于"中介层声明式填充"，是 runner 在副本上做的工具特定准备，不是样例端为工具让步。

承诺从天真版"样例对所有工具完全无感"降级为：**"原始磁盘字面零修改 + 隔离副本上声明式工具特定填充"**。

#### A 的形式定义（最精确可机械验证判据）

一个 example 是"自身完善独立"的当且仅当——加入框架信号文件（`hirusttest.toml`）**之前与之后**，example 自身的行为**完全 100% 一致**。如果不能保证完全一致，则该改动违反原则 A。

**等价表述**：信号文件必须对 cargo / rustc / 任何 verifier 工具**完全不可见**——加不加它，对 `cargo build` / `cargo check` / `cargo run` / `cargo test` 等任意 cargo 子命令的输出**字节级一致**。

**该判据的实现保证**：

- `hirusttest.toml` 是**项目自有 schema**，不在 cargo / rustc 识别的文件名集合（不是 `Cargo.toml` / `*.rs` / `build.rs` / `.cargo/config.toml` 等）
- 任何 verifier 工具通过 cargo 触达——cargo 不读它，工具也就不读它

**判据的应用举例**：未来若有人提议在 examples 端加入新的配置（如 `[package.metadata.hirusttest]` 内嵌进 Cargo.toml 的方案），必须先验证"加入前后行为是否字节级一致"。**Cargo.toml 内嵌方案违反该形式标准**（cargo 读 Cargo.toml 时会解析全部内容）——所以必须用独立文件 `hirusttest.toml`，禁止内嵌。

#### A 形式定义下的双轨 schema

按 example 自身的复杂度，配置 schema 分两轨。两轨都满足上述形式定义（cargo / rustc 不读它们，行为字节级一致）：

| 轨 | 适用场景 | 信号位置 |
|---|---|---|
| **单文件轨**（默认） | 简单单特性测试——本项目自己写的、纯粹为测某个 Rust 特性而创建的 lib crate（如 `examples/closure/fn-fnmut/`） | `<example_dir>/hirusttest.toml` |
| **目录轨**（仅外部综合项目）| 外部综合项目——非本项目原创、可能是 git submodule 或 vendor crate 的真实 codebase（如 `vendor/x509-parser/`、`vendor/openssl/`），单文件 schema 无法承载所需配置（多 entry 独立配置、per-entry verifier 专用辅助文件等）| `<example_dir>/.hirusttest/config.toml` + 同目录下可选辅助文件 |

**双轨的强制边界**（防止双轨退化为混乱）：

1. 简单单特性测试**不允许**升级到目录轨——必须用单文件
2. 外部综合项目**不允许**降级到单文件轨——必须用目录（即便目前只有一个 entry，也用目录轨以预留扩展空间）
3. 同一目录**禁止同时存在** `hirusttest.toml` 与 `.hirusttest/`——runner 须把这种情况报为发现阶段错误
4. `.hirusttest/` 仅可包含项目自有 schema 的辅助文件（如 per-entry verifier stub、proof annotation snippet 等）；这些辅助文件可被工具读取（**这是受控的工具适配，发生在 framework 中介层调度时**），但 example 自身的 src/ 与 Cargo.toml 仍需满足形式定义"cargo 行为字节级一致"

**discover 双轨判定**（runner 实现责任）：给定 `<example_dir>`，按以下顺序：
- 若 `.hirusttest/` 存在且 `.hirusttest/config.toml` 存在 → 走目录轨
- 否则若 `hirusttest.toml` 存在 → 走单文件轨
- 否则该目录不是 example
- 双方同时存在 → 报错

#### A 的 5 层精细化（解耦层级）

A 在实现上分五个层级，每一层都是约束：

1. **Examples 自身完善独立**（最严）—— examples 代码层面**完全不需要意识到自身将被测试**。entry fn body / src/ 任何源码是 plain Rust，零 verifier 标记。**侵入和调动测试都是框架的事**。
2. **极简解耦** —— examples 与 tools 之间**没有任何直接连接路径**——所有连接由框架（runner）承担。这是项目"把不公平沉淀到中介层"意识的最强落地形式。
3. **Examples 端标识性信号的上限** —— 如果必须让 examples 端为框架提供少量元数据，**最大限度允许独立的 `hirusttest.toml`**（满足上述形式定义）。**禁止**触及 src/ 源码内容；**禁止**通过 Cargo.toml 内嵌（cargo 会读，违反形式定义）。
4. **Tools 端绝不为框架适配** —— tools 是黑盒。`tools/<name>/` 目录下的所有文件（`tool.toml`、`harness.rs.tera`、wrapper.sh 等）是**集成者描述工具自身行为**的桥接信号——**不是工具自身的修改**。tool 本身（cargo-kani / cargo-prusti / cargo-creusot 等）作为黑盒，绝不为框架做任何代码或行为上的适配。框架理解工具的命令、版本输出、产物路径、退出码语义等作为"信号"，做翻译与连接，但不预设工具的内部细节。
5. **框架负责一切信号转换**（信号驱动的解耦）—— runner 只与"信号"打交道，不与具体的 examples 或 tools 打交道：
   - examples 端的信号 = `hirusttest.toml` 的声明 + `Cargo.toml` 自身原生内容
   - tools 端的信号 = `tool.toml` 描述的工具行为 + `harness.rs.tera` 描述的入口形态 + 工具自身的退出码 / stdout / stderr
   - 框架是双方信号的**中介翻译层**：把 examples 的标识转化为对工具的调用，再把工具的输出转化为可记录的结果

### 原则 B：测必要条件，非语义对错

本项目是**筛选**不是**认证**——只问"工具能不能吃下这段代码并产出预期形状的输出"，不问"产出语义是否正确"。分类标签只描述产出形状，不带对错评分。

### 原则 C：异质性归配置，框架代码同质

工具间差异以声明数据形式（`tool.toml + tera 模板`）存在，不作为框架代码的 if 分支。任何 `if tool == "kani"` 都被禁止。

---

## 五、原则交集的运行时投影

### A 的投影：运行原子 = 单 entry

如果把同样例多 entry 在一次工具调用里批跑，任意一 entry 失败会污染其他——要么整批失败，要么输出难拆。所以一次操作对应唯一 (tool, entry) 三元组：一次隔离副本、一次 harness 渲染、一次工具进程、一份 raw output。

### A ∩ B 的投影：能力靠观测，不靠声明

框架不预跳过任何 (tool, entry) 组合——所有组合都跑。tool 配置里禁止 `supported_kinds = [...]` 这类字段——一旦出现就意味着框架替工具说"它做不了 X"，既违反 A（替工具发言）也违反 B（不该靠 config 假设而该靠跑后看结果）。

### A ∩ B ∩ C 的投影：工具非静态原则

任何 (tool, entry) 上的观察都是**对那个时刻、那个版本组合下的工具的观察**——这条对应核心模块 1 的"结果记录严谨全面"原则。

---

## 六、宪法级硬指标

> 这一节是**强约束**，新增工具 / 修改 oracle / 改动配置时必须满足。违反者必须在 PR 描述中明确说明修订理由并经讨论确认。

### 硬指标 1：前端支持性观察原则

**核心思想：测工具理论上能"接受"的代码范围，不测它实际能"处理完成验证"的范围。**

- "接受范围" = 工具的前端（parser / 宏展开 / type-check / 翻译 / 模型构造）能不能吃下这段 Rust 并形成内部表示
- "处理范围" = 后端求解器（SAT / SMT / 模型检查器）能不能在该 IR 上完成验证

**两件事在量级上完全分开**：前端通常秒级、求解层可能小时级或不收敛。把求解层一并测会让对比失公——同一组样例上 kani 因 cbmc 路径爆炸 timeout、charon / hax 仅翻译几秒；同样标 FAILED 但根因不同维度。

**配置落地**——每个工具找一个把求解层关掉**且仍走自身前端**的 flag：

| 工具 | 落地方式 |
|---|---|
| kani | `--only-codegen`（MIR → GotoC + 类型 / 模型检查，不 invoke cbmc）|
| charon × 2 | `--abort-on-error`（翻译报错暴露但不进 verifier 后端）|
| prusti | env `PRUSTI_NO_VERIFY=false` + `PRUSTI_DUMP_VIPER_PROGRAM=true` + `PRUSTI_PRINT_HASH=true`（触发 encode 并 dump Viper 程序后提前返回，不调 Silicon SMT）|
| verus | argv `--no-verify`（parse + verus! 宏展开 + type-check，不调 Z3）|
| verifast | `-skip_specless_fns` |
| creusot | `cargo creusot`（默认仅翻译到 Coma；显式 `cargo creusot prove` 才进 SMT）|
| soteria | 默认 `--step-fuel` / `--branch-fuel` 边界（bounded 符号执行，不无限求解）|
| miri / kmir | 真解释执行类，按 MIR / SMIR 走代码——属解释器范畴而非求解器，秒级开销可接受 |
| 翻译类（cargo-check / charon × 2 / hax × 3 / aeneas × 4 / rocq-of-rust）| 天然只翻译，无求解层 |

**每个工具的"前端测试范围"由工具自身设计决定，没有统一标准**——只能根据该工具的内部 pipeline 设计精确切割（与 §三-3-2.a"形式指标"对应）。新增工具必须在该工具 README 中明确：(a) pipeline 阶段图、(b) 前端 / 后端边界落点、(c) `tool.toml` 用什么 flag 实现该切割。

### 硬指标 2：不允许 partial

**SUCCESS = 工具完整完成它的工作单元，不允许任何 partial / silent skip / 半翻译**。即便 partial 产物落盘也是 FAILED——工具自陈"我没全干完"必须被尊重（如 aeneas 的 `Generated the partial file (because of N errors)` 路径，hax 的 `text!("sorry")` silent path）。

`tool.toml` 的 oracle 必须按工具内部 pipeline 设计精确停在自带后端之前，但完整跑完前端。

这条硬指标与 §三-3-2"诚实的测试范围"是同义的两个角度——前者从 oracle 行为角度说，后者从测试结果可信度角度说。

### 硬指标 3：不区分翻译深浅

"接受"指工具自身前端在册收下，至于工具内部对该构造做的是浅 syntactic 搬运、MIR-level 深度翻译、还是 verifier 的 dialect 接受，本测试一律不区分、不加权。两个工具同对一段 Rust 标 SUCCESS 不蕴含它们后续做的东西等价——这是覆盖度测量的本性，不是缺陷。

### 硬指标 4：反作弊推论

dry-run flag 必须使工具**真的拿样例代码喂自己的前端**，不能把代码绕路给 stock rustc 走 cargo-check 等价路径。否则 SUCCESS 信号退化为"rustc parses it"——所有工具会齐刷刷高分，丢失工具间的特性子集分化信号。

最典型例子：Verus harness 的 `mod __ts_inner` **必须**写在 `verus! { }` 块内；写在块外，Verus 把 mod 内容直接当 stock Rust 透传 rustc，等于把它降级成 cargo-check（详见 `tools/verus/README.md`）。

### 本节自我性声明

§三-3"市面工具样例"的诚实测试范围 + §六 宪法级硬指标 + 派生的工具 README 书写规范（详见 [`tool-integration.md`](tool-integration.md)）是**我方使用本框架做 tools 集成时的方法学选择**。

在遵守 §四 三大派生原则（A 双方不可侵入 / B 测必要条件 / C 异质性归配置）的前提下，他人复用本框架可做其他方法学选择——纳入 SMT 求解、选反向权衡（无漏报 + 允许误报）、接受 partial 等。**外来 tools 无须遵守本节**；框架本身只要求 §四。

---

## 七、外围原则

### 性能不算问题（除非升级为功能问题）

**资源用量、性能开销、缓存效率本身不构成"问题"**。只有当某性能/资源失控让功能层面失效时它才算问题。

- 无 timeout 让工具 hang 时整 run 跑一整天 → 升级为功能问题，必须修
- 子进程 stdout/stderr 一次性读到内存的 OOM 风险、`Cargo.lock` 副本带来的 cargo re-resolve 延迟、cargo registry flock 在并发首次 fetch 时序列化 → 纯性能问题，不修

这条精神是 challenge 与 audit 时的判据。

### Occam 砍项

砍掉非必需复杂度。每条决策必须能溯源到一条原则。已 v1 砍项：external_lib mode、verdict 期望比对、category clean/bug 二分、`tool.toml.env` / `cwd` 字段（用 `command = ["env", ...]` 前缀代替）。

---

## 八、原则的生效范围

这些原则约束：

1. **核心模块 1（runner）的架构与实现** — `architecture.md`、`detailed-design.md`、`runner/src/*` 必须满足
2. **核心模块 2（examples）的样例设计** — 新增样例必须满足原则 A 单性质纯净性 + 原则 B 整体多样性的梯队归属
3. **次要模块 3（市面工具样例）的工具集成** — `tools/<name>/{tool.toml, harness.rs.tera, README.md}` 必须满足硬指标 §六-1 至 §六-4 + 时效性 + 诚实测试范围（形式指标 / 上限保证 / 下限诚实）
4. **测试报告** — 必须明确锚定时间 + 工具版本组合，不构成对工具能力的长期承诺

任何后续讨论、PR、设计变更都从本文档的原则出发——以本文为标准。

---

## 九、与下游文档的关系

- **下游：`architecture.md`** — 在本文原则约束下做核心模块 1 / 2 的具体架构设计；含模块切分、模块功能规约、模块间接口规约。是 design 的"lib 入口"，索引其他子模块文档。
- **下游：`detailed-design.md`** — 函数级细化，schema 完整定义、运行时机制、19 工具配置示例。
- **下游：`tools/<name>/README.md`** — 次要模块 3 的每工具按硬指标 + 诚实测试范围声明前端边界、形式指标、SUCCESS 信号、形式严格性、漏报盲点。
- **下游：`runner/src/*`** — 核心模块 1 的实现，必须满足模块功能规约 + 接口规约。
- **下游：`examples/<feature>/<dir>/`** — 核心模块 2 的样例，必须满足纯净性与多样性梯队归属。

外围参考：

- **`research/testsuite-research.md`** — 调研报告（问题意识来源 + 先行尝试对比）
- **`test-reports/*`** — 次要模块 3 的实测数据快照（时效性强，非长期承诺）
- **主 `README.md`** — 用户视角的入口文档，引用本文。

如本文与任何下游文档冲突，**改下游不改本文**——除非通过显式讨论修订本文，并相应级联更新所有下游。
