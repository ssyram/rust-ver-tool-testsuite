# A Tool-Neutral Framework for Measuring Feature Acceptance Breadth in the Rust Verification Ecosystem: An Empirical Study of 20 Tools

> ISSTA 2026 投稿 draft（中文写作，camera-ready 前翻译）。所有数字锚定 run id `run-1778560393-59119` / 2026-05-12 / 工具版本快照 / corpus commit `936311b` / 161 entries × 41 features。术语对照见 `docs/publish/glossary.md`，引用见 `docs/publish/tool-citations.md`。
>
> Track 候选：Research Papers (empirical study) 或 Tools/Benchmark Suite。

---

## Abstract

Rust 验证工具生态高度分化：每个工具自带声明形式、推荐用法、特性覆盖说明，使得跨工具比较缺乏公共基准。既有 survey 多为 narrative-style 综述，依赖作者主观判读"工具 X 是否支持特性 Y"；既有 benchmark 又多为 tool-specific（工具开发者自维护的 `examples/`）。两类形态都无法支撑可重复的、tool-neutral 的特性覆盖测量。

本文提出一个工具无关的测量框架，把跨工具的不公平沉淀到第三方中介层（runner + tool-specific 声明数据 + harness 模板）而非样例本身。框架引入两层切割边界（深度：前端测量；宽度：当前 crate 焦点），以及三路径 oracle 实施策略，使"工具能不能吃下这段代码 + 产出非 partial 输出"成为可机检的二值信号。在此基础上，我们对 20 个 Rust 验证工具（涵盖 translator / verifier / abstract interpreter / symbolic executor 四类家族）跑 161 entry × 41 feature 的 corpus，得到 3220 个 (tool, entry) 观察，并按 95% Wilson 二项置信区间与 McNemar exact test 报告通过率与同族显著性。

本文主要 contributions：(1) 一个工具中立、可重现的特性覆盖测量 framework，含 runner / 声明式 tool 配置 / `.hirusttest` 信号 schema；(2) 对 20 个工具的实测快照，按家族（**不按通过率排序**）呈现以避免 ranking 暗示；(3) 一个公开 corpus + reproducible artifact；(4) oracle 设计方法学：UNKNOWN 严格语义 + 双切割 + 三路径 + bug-detect 派生 + 输出形态对称性论证。

---

## 1. Introduction

### 1.1 Problem Statement

Rust 安全保证的形式化验证近年涌现大量工具：演绎验证器（Prusti, Creusot, Verus, VeriFast, Kani）、抽象解释器（MIRI）、符号执行器（Soteria）、Rust → 证明助手 translator（Charon, Aeneas, Hax, Rocq-of-Rust）。各工具在 input 形态、推荐 toolchain、表达力边界、输出 protocol 上**高度异质**。

一个具体的 inability：用户 / 研究者 / 标准制定者无法回答以下问题：

- "在 2026 年的 Rust 生态中，哪些主流验证工具能接受 `impl Trait` return type？"
- "如果我有一段带 `async fn` 的代码，哪些工具会拒绝、哪些会沉默地跳过？"
- "MIRI 和 Soteria 的能力差异在哪些 feature 上 manifest？"

这些问题表面简单，但跨工具回答需要在每个工具的术语 / pipeline / config 形态下重新理解，不可能由非领域专家做出可靠判断。

### 1.2 Why Existing Approaches Are Insufficient

**Tool-specific benchmark**（每个验证工具自带 `examples/` 仓库）按工具自家形式写测试用例。这种 corpus 在工具内部具有 discriminative 价值，但**跨工具不可比较**：同一段算法逻辑在 Kani 形式（`#[kani::proof]` + assumption / property macros）和 Verus 形式（`verus! { ... requires / ensures }` 宏块）下是两份不同的代码，没有 fair 的 cross-tool delta。

**Narrative survey**（如 Pearce 系列的 Rust 验证综述 [pearce2024formal]）以学术论文体 + 描述性表格综合现状，但定性多于定量，且作者判读"X 支持 Y"在 surface-feature 层无法机检验证，时效性也快速衰减。

**Unified-API style** 思路（让所有工具围绕同一抽象 API 写 corpus）听起来公平，但实际把"特定工具的 mental model"嵌入 corpus 本身——例如某个 API 的语义按 Kani 偏好写，则 Verus 与 Creusot 自然在该 corpus 上"看起来劣势"。我们识别这是**根本问题 1：不公平**——把不公平沉淀进样例本身。

### 1.3 Approach

本文采用与上述两类思路正交的设计：把不公平沉淀到**第三方中介层**——一个统一的 runner + 每个工具自己的声明式配置（`tool.toml`）+ 每个工具自己的 harness 模板。样例与工具两端保持各自纯粹：样例是零工具依赖的 Rust lib crate，工具按其上游推荐姿势接入。

进一步，针对**根本问题 2：不公信**（测量姿势工具开发者可一句话驳回——"姿势不对"）我们设计了三原则栈作为测量姿势的合法性论据：**本地性**（最高）> **遵循社区惯例** > **最大善意**。装对工具要求的 toolchain、按其官方文档 / 推荐用法接入、尽最大可能配合工具——之后仍失败的 entry，FAILED 入册站得住。

### 1.4 Contributions

1. **Tool-neutral measurement framework**：runner + declarative `tool.toml` + Tera-based harness 模板 + `.hirusttest` 信号 schema。样例不为工具改写，工具不为框架适配，所有异质性归于第三方中介层。框架本身的核心代码无任何 `if tool == "kani"` 类后门。
2. **Empirical study of 20 tools**：在 161 entries × 41 features 的 corpus 上运行 20 个 Rust 验证工具的 v6 final 快照（2026-05-12），共 3220 个 (tool, entry) 观察，按 95% Wilson CI 与 McNemar exact test 报告 per-tool 与同族对比结果。
3. **Reproducible benchmark suite**：corpus + runner + tool integrations 全部公开在项目仓库（commit `936311b`）；每次 run 写入 `results.json` 含 host / 时间 / 工具版本三元组元数据，可独立复跑。
4. **Oracle 设计方法学贡献**：UNKNOWN 严格语义（仅两类，§3.4 详述）+ 双切割边界（深度 / 宽度）+ 三路径 oracle 实施（A / B / C，§3.5）+ bug-detect 派生 + 工具输出形态对称性论证——一组可被未来扩展 / 复用 / 挑战的设计 patterns。

### 1.5 Scope and Disclaimer

本文的测量结果是 **2026-05-12 时间快照**，不构成对工具能力的长期承诺。Per 项目 README disclaimer：测试结果反映 "在这个时间点、这些工具版本、这个 corpus 下，可观察到这些数字"——工具升级后旧结果解释力衰减是必然的。**本文不对任何工具给出能力优劣排序结论**；§5 / §6 的呈现按工具家族分组而非按通过率 desc 排列，避免 ranking 暗示。

---

## 2. Background & Related Work

### 2.1 Rust 验证工具谱

Rust 验证工具按 pipeline 与表达力定位可大致分为四类：

- **Translator family**：把 Rust 源码翻译到证明助手 / 中间语言后由人类（或 oracle）在目标语言写证明。代表：Charon [ho2022aeneas]（Rust → LLBC），Aeneas [ho2022aeneas]（Charon + 4 个 backend printer：Coq / F* / Lean / HOL4），Hax [hax2024]（hax-engine + Coq / F* / Lean printer），Rocq-of-Rust [rocqofrust2024]（Rust → Rocq + 可选 typecheck stage）。
- **Verifier family (deductive)**：基于 SMT / BMC 的演绎验证。代表：Kani [vanhattum2022kani]（CBMC），Prusti [astrauskas2019leveraging]（Viper），Creusot [denis2022creusot]（Why3），Verus [lattuada2023verus]（Z3）。
- **Verifier family (separation logic)**：VeriFast [jacobs2011verifast]（Z3 with separation-logic predicates）。
- **Abstract interpreter / symbolic execution**：MIRI [jung2020stacked]（abstract interpretation + UB detection），Soteria [soteria2024]（symbolic execution + bug detection），KMIR [rosu2010k]（K Framework 解释 MIR）。

完整 cite list 见 `docs/publish/tool-citations.md`。注意：本文测的 corpus 与 oracle 框架是 tool-neutral 的，**这些工具被本文测量并不意味着我们对工具开发者的工作做了能力评判**——只是观察各工具在本 corpus 上的 feature 接受面。

### 2.2 Empirical Study of Verification Tools 既有形态

传统 empirical study of program verifiers 形态多分两支：

- **Soundness / completeness benchmarks**：SV-COMP [beyer2019svcomp] 是 C 验证最成熟范式——以年度竞赛 + 程序验证 task 形式，按 tool-specific 类目（reachability / memory-safety / 等）评通过率。SV-COMP 程序本身有 tool-neutral 的 oracle（`expected verdict` 写在 source），但 task 类目仍按 verifier 关注点分组。
- **Tool-specific case study**：每篇 verifier paper 报"我们在自家 examples/ 上验过 N 个 case"。这种形态对工具开发者自身有意义，跨工具不可比较。

本文的 differentiation：**测的是 feature acceptance breadth 而非 verification correctness**，因此与 SV-COMP 系列侧重点不同——SV-COMP 关心"工具能否给出正确 verdict"，本文关心"工具能否吃下这段代码 + 产出非 partial 输出"。前者要求 verdict 比对，后者不需要。

### 2.3 Wohlin 四类 Validity 框架与本文 Adaptation

本文借鉴 Wohlin et al. [wohlin2012experimentation] 的四类 validity threats 框架（Construct / Internal / External / Conclusion）评估自身可信度（§7）。但需注意：

- 本文测的 construct 是 **"feature acceptance breadth"**，不是 verifier soundness/completeness。Construct operationalization 与传统 verifier study 不同。
- 本文的实验对象（20 个工具）都是**确定性程序**——给同样 input + 同样 environment + 同样 toolchain，输出 byte-identical。因此 Wohlin 框架的 multi-run / RNG seed 类 conclusion-validity 条目不适用（详 §7.4 / cc-rebuttal §1）。

### 2.4 与 SV-COMP-style Benchmark 的对照

SV-COMP 是 tool-specific test categories 的 explicit 设计选择（每个 sub-category 测某 verifier 类型的强项）；本文是 tool-neutral corpus 的 explicit 设计选择（每个 entry 不为任一工具 sweet spot 写）。两者各有取舍：

| 维度 | SV-COMP-style | 本文 |
|---|---|---|
| Corpus 设计 | 按 verifier 类目分组 | 按 Rust feature 分组 (41 features) |
| Oracle | source 内 `expected verdict` 比对 | 工具自身 exit code + 项目维护 wrapper 信号 |
| Construct | verification correctness | feature acceptance breadth |
| 跨工具 fairness 沉淀 | task 类目设计 | 第三方中介层 (runner + tool.toml) |
| Tool 间 ranking | 默认呈现（按 score） | **deliberate 避免**（按家族分组） |

---

## 3. Framework Design

### 3.1 核心范式：(tool × entry) 矩阵

测量的原子是 (tool, entry) 二元组。每个 entry 是一个独立的 Rust lib crate（零 verifier 标记，零跨 entry 共享 helper），位于 `examples/<feature>/<dir>/`。每个 tool 由声明式 `tools/<name>/tool.toml` + Tera 模板 `harness.rs.tera` 描述。

一次运行枚举 161 entries × 20 tools = 3220 个独立任务。每个任务在独立的 work dir 内执行：cp 隔离副本 → patch `Cargo.toml`（按 `tool.toml::extra_cargo_deps` 注入）→ 渲染 harness（按 `entry_mode` 决定写 `src/bin/__ts_harness.rs` 或 `src/lib.rs`）→ spawn 工具进程 → 捕获 stdout / stderr / exit code → 清理。

这一范式直接服务于不公平问题的对策：runner 代码完全 tool-neutral（无 `if tool == "kani"` 类 if 分支），工具异质性通过声明数据 + 模板表达；样例不被工具污染（cargo / rustc 不读 `.hirusttest`，对 example 字节级无感知）。

### 3.2 双切割测量边界

按 `principles.md` §六，测量边界沿两个独立维度切割：

**深度切割（front-end-cut measurement）**：测量限于工具自身的前端阶段（parser / type check / 翻译 / 模型构造），求解层（SMT / BMC / Z3）排除在测量结论外。原因：

- 不同工具的求解层深度差异巨大，纳入会让深 verifier（Verus / Prusti）在含 BMC 超时 / SMT timeout 的 corpus 上 unfairly 劣势，混淆"特性接受"与"求解器超时"两件独立的事。
- 测量必须命中工具自身前端，否则 SUCCESS 信号退化为"rustc parses it"，工具间分化丢失。

每个工具的具体切点声明在 `tools/<name>/README.md` 中（详 §3.6）；典型 flag 包括 `kani --only-codegen`、`verus --no-verify --log vir`、`prusti PRUSTI_NO_VERIFY=false PRINT_HASH`（encoder runs but Silicon does not）等。

**宽度切割（target-crate scope）**：partial 信号只在 entry crate 自身代码内触发时计 FAILED；外部依赖（std / cargo registry / vendor）路径下的 opaque / skip / stub 不算 silent partial。原因：

- 把外部依赖翻译为 placeholder / opaque 是工具合法的设计选择（如 Aeneas 把 `std::vec::Vec::push` 保持为 opaque axiom 让人类在 Coq 端补 spec）。
- 当 entry crate 自己的 fn / type 被 silently opaque 时才是 silent partial，应升 FAILED。

### 3.3 三路径 oracle 实施 width-cut

宽度切割在 oracle 实施层因工具输出形态分三路径落地：

- **路径 A（source-path filter）**：工具输出 partial signal 后紧跟 `--> path` 行（如 Aeneas 4 backend 的 charon stage stderr）。Wrapper Python 段扫描每个 signal 后 60 行内的路径，按前缀 `/rustc/` / `/cargo/registry/` / `/vendor/` 判断；全 external 则 suppress 该 signal。落地于 4 个 `aeneas-*-wrapper.sh`（P36）。
- **路径 B（reverse-evidence keyword grep）**：工具输出聚合 count 不带 source span（如 Kani 5-markers warning）。无法按 path 过滤，改用反向证明：grep entry crate `src/` 文本是否含 marker 对应触发关键字（`asm!` / `simd_*` / `catch_unwind` / `::mask(` / `c"..."` 等）；含则 entry 自用 → FAILED，不含则必来自 deps → SUCCESS 豁免。落地于 `kani-strict-wrapper.sh`（P37），实测把 Kani 通过率从 93.8% 提升到 98.8%（10 个原 FAILED 中 8 个是 marker 来自 deps，2 个 entry 自用保持 FAILED）。
- **路径 C（natural satisfaction）**：单文件 pipeline 工具（Verus / VeriFast / Soteria / Rocq-of-Rust）根本不读 Cargo.toml deps，测的就是 entry crate 自己，§六 自然满足，oracle 不需要额外 filter。

### 3.4 UNKNOWN 严格语义

ResultClass 三分（SUCCESS / FAILED / UNKNOWN）。其中 UNKNOWN 严格限定为两类：

- (a) **全局工具链崩溃**（用户重装可修，如 Verus 缺 `verus-root`、Prusti `viper_tools` 丢失）
- (b) **我们这边可识别问题且暂未修**（如我们 harness 模板 bug → `runnable_harness_arg_mismatch`、我们 corpus 引入的 vendored crate lint → `vendor_lint_strictness`、我们环境损坏 → `environment_corruption`）

每类必附明确归因 + 会修计划。**工具自身能力边界一律 FAILED**——包括但不限于：官方 wrapper 失败、工具自选 toolchain 不支持新特性、工具单文件 pipeline 不读 Cargo.toml、官方 wrapper 不传 `--edition`。

这条严格化（P27 commit `28b4a03`）由"不公信"根本问题驱动：按本地性原则——测当下这个工具版本 + 它要求的 toolchain 能做什么；装对其要求 toolchain 后仍不行 → 工具触达其能力边界，FAILED 站得住——工具开发者不能以"姿势不对"驳回。

v5.1 → v6 演进中，这条严格化把 131 个 UNKNOWN 中 121 个重判为 FAILED（dependency_resolution / toolchain_edition_mismatch / edition_pipeline_propagation 三类规则被删）。v6 残留 10 个 UNKNOWN 全是 `vendor/x509-parser` 的 `#![deny(unused_qualifications)]` 在新 rustc 下触发，属 (b) 类。

### 3.5 工具输出形态对称性

不同工具的"前端"nature 不同——这是设计 oracle 时必须直面的不对称：

- **Verifier-style 工具**（Kani / Verus / Prusti / Creusot）有清晰的"前端 vs 求解层"二分切点，项目设计成跑前端层。
- **Abstract interpreter（MIRI）**和 **symbolic executor（Soteria）****没有这种切点**——它们的"前端"就是完整执行（abstract interpretation / 符号执行）一次走完。

对此，按 `principles.md` §四 B "测必要条件，非语义对错"，**bug-detect 归 SUCCESS** 作为一条派生原则：

- MIRI / Soteria 跑完一段代码 + 自陈"我在 entry 代码里检测到 UB / bug / counterexample" 时——这是工具最有价值的输出之一，按 §四 B 归 SUCCESS。
- FAILED 只用于"工具不能吃下这段代码"（不支持 / 自身 crash / 翻译失败 / 拒绝 partial）。

**对称性论证（防 reviewer 误读 unfair）**：bug-detect = SUCCESS 不是给 MIRI / Soteria "额外 reward"。原因：每个工具按它能给出的有效输出形态评 SUCCESS——MIRI / Soteria 的有效输出形态包括 bug-detect；verifier-style 工具被项目主动让其停在前端层（避免 BMC/SMT 求解开销），它们**本来就不在 bug-detect 路径**，不是被歧视。每个工具按它的输出 nature 评——对称。

详细论证见 `architecture.md` §一 末段。

### 3.6 工具集成"法律"

每个 tool 集成必须满足以下硬指标（`docs/design/tool-integration.md`）：

1. README 明示 pipeline 阶段图 + 前端/后端边界 + `tool.toml` 用什么 flag 实现该切割。
2. SUCCESS 信号的形式指标（exit code 单一信号 / exit code + 产物 grep / exit code + stderr grep / 多门组合）。
3. **0 误报论证**（硬指标）：oracle SUCCESS ⟹ 真 SUCCESS。论证形式不限——反向证明 / 源码层穷尽 / 双向蕴含 / 实测验证均可。
4. **0 漏报状态**（软指标）：能形式证明 0 漏报最好；不能则提供防漏报机制 + 实测有效性 + 漏报盲点诚实声明。
5. UNKNOWN (b) 子类的归因 + 会修计划。
6. 失败归因切分：**官方 wrapper 失败 = 工具锅 (FAILED)** / **我们维护的 wrapper 失败 = 我们锅 (UNKNOWN)**。

这一组指标在每个 `tools/<name>/README.md` 中显式落地，作为 oracle 设计的可被审计的契约。

---

## 4. Implementation

### 4.1 Runner 架构

Runner 是 Rust binary（`runner/src/`），按四模块切分：

- **discover**：扫 `examples/` 与 `tools/`，产出 `Vec<Example>` 与 `Vec<Tool>`。双轨 example schema：单文件轨（`hirusttest.toml`，默认）+ 目录轨（`.hirusttest/config.toml`，仅用于 vendor crate 真实 codebase）。
- **exec**：执行一次 (tool, entry) 三元组，按上述 cp → patch → render → spawn 流程。子进程在独立 process group（`process_group(0)`），timeout 时 `kill(-pgid, SIGKILL)` 杀整个 group。
- **report**：把 `Vec<TaskResult>` 落盘为 `results.json`（机读，BTreeMap 排序保证确定性）+ `report.md`（按 feature 分组的人读矩阵）。
- **main**：CLI、调度、rayon 并发池（默认 `num_cpus`，`--parallel N` 可覆盖）。

### 4.2 `.hirusttest` Schema + `[env]` 机制

每个 entry 旁边一份 `hirusttest.toml`（或 `.hirusttest/config.toml`）声明：

```toml
target_path = "."          # entry 对应的 Cargo crate 相对路径
[[entries]]
entry_fn = "trigger_foo"   # 入口零参 fn，runner 用 TS_ENTRY_FN 注入 harness

[env]                      # 可选，runner 在 spawn 子进程时附加的 env
RUSTFLAGS = "--cap-lints=warn"
```

这一 schema 满足原则 A 的形式定义：cargo / rustc 都不读 `hirusttest.toml` 与 `.hirusttest/`——信号文件加入前后 example 的 cargo 行为字节级一致。

### 4.3 `tool.toml` + Tera Harness 模板

每个 tool 配置：

```toml
name = "kani"
command = ["bash", "tools/kani/kani-strict-wrapper.sh"]
timeout_secs = 300
entry_mode = "bin"
version_command = ["cargo", "kani", "--version"]
# extra_cargo_deps = ["creusot-contracts = \"0.5\""]  # 仅 Creusot 类工具需要
```

Tera 模板 `harness.rs.tera` 标准变量两个：`{{ target_crate_name }}` 与 `{{ entry_fn }}`。例如 Kani harness：

```rust
#[kani::proof]
fn __ts_harness() { {{ target_crate_name }}::{{ entry_fn }}(); }
```

整个工具异质性通过这两个声明文件表达——runner 代码不感知工具具体。

### 4.4 Wrapper 与 Wrapper 归属

20 工具中有项目维护 wrapper 的 7 个（Prusti / Aeneas × 4 / Rocq-of-Rust × 2 / VeriFast / Kani / MIRI / Soteria 的不同子集——详 `tools/<name>/README.md`）。Wrapper 在工具自身 stdout / stderr 之上加 oracle 信号——例如 Kani wrapper 跑 `cargo kani --only-codegen` 后 grep 5-markers 子集 + os.walk entry src 反向证明；Aeneas wrapper 在 OCaml 主程序之上扫 charon stage stderr。

按"失败归因切分"原则：我们维护的 wrapper 内部 bug → UNKNOWN；wrapper 转发的官方层错误 → FAILED。该切分在 v6 audit 中由独立 c 路 agent 主动 grep `unbound variable` / `TS_*: not set` / `command not found` / bash 语法报错 0 命中验证。

### 4.5 20 工具集成挑战汇报

工具集成是次要模块（按 `principles.md` §三），不是本文核心 contribution。但实际工程中我们碰到的几类典型挑战：

- **环境治源**：Verus / Prusti 的 binary 因 `/tmp/` 周期清理失效——治源迁移到 `~/.local/share/ts-tools/`。
- **Toolchain pin 副作用**：Prusti 锁 nightly-2023-08，拒新 edition feature——pass rate 含此 confounder（在 §7.3 ToV 明示）。
- **Silent partial 封堵**：Aeneas 上游 Warn 通道（非 craise/error_list）emit partial 而 exit 0 ——wrapper 加 stdout grep 4-pattern gate 封堵（P30）。
- **多门组合 oracle**：Rocq-of-Rust 单文件 pipeline + 翻译路径非确定性——7 道 gate + N=7 attempts AND-reduce。

20 工具每个的具体集成挑战 + oracle gate 设计详 `deep-reports/cc-reports/<tool>.md` 与 `tools/<name>/README.md`，由于篇幅本文不展开 per-tool capsule（建议未来 extended version 或 artifact appendix）。

---

## 5. Empirical Study

### 5.1 Corpus + 实验 Setup

- **Corpus**：161 entries，覆盖 41 features，按目录分组（如 `examples/closure/`, `examples/impl-trait/`, `examples/industrial/x509-parser/` 等）。Corpus 设计原则（`principles.md` §五）：单特性（per-feature minimal example）+ 边界（`*-limit/` 系列）+ 综合（`industrial/` 真实 vendor crate 多 entry）三梯队。
- **工具版本快照**：见项目 `docs/test-reports/feature-coverage-2026-05-12-v6.md` §0.2 与各 `tools/<name>/README.md`（Kani 0.67.0 / Verus 0.2026.05.03.8b81855 / 等）。
- **Host**：Apple M5 / macOS 25.4.0 / aarch64 10 cpus。
- **Run id**：`run-1778560393-59119`，ISO 时间 2026-05-12T12:33Z（19 tools）+ Verus rerun 12:58Z merge。Verus rerun 因路径修复后单独跑、已合并入 `results.json`。
- **Total tasks**：161 × 20 = 3220。
- **Determinism justification**：20 个工具均为确定性程序；唯一已知非确定性 `rocq-of-rust` 翻译路径已用 N=7 attempts AND-reduce 在 wrapper 中处理（详 `docs/fixes/ror-gate6-fix-2026-05-11.md`）。Multi-run 不提供额外信息——见 §7.4 ToV。

### 5.2 主结果（按家族分组，不按通过率排序）

数据源：`runs/run-1778560393-59119/results.json`（v6 final post-P37，总计 2219 SUCCESS / 1001 FAILED / 0 UNKNOWN = 68.91% 通过率）。注：与 §5.3 关键发现段共 19 工具的 UNKNOWN 已通过 P36 / P37 §六 路径 A + B 实施全部转入 FAILED 或 SUCCESS，仅 `vendor/x509-parser` 上 5 工具 × 2 entries = 10 个残留 UNKNOWN，已被另一统计角度（§5.4 详）单列。

每行格式：`tool | n | S | F | rate [95% Wilson CI] | Measurement boundary`。

#### 5.2.1 Baseline

| tool | n | S | F | rate (95% CI) | Measurement boundary |
|---|---:|---:|---:|---|---|
| cargo-check | 161 | 161 | 0 | **100.0% [97.7, 100.0]** | rustc type/borrow check (single exit code) |

cargo-check 是 corpus 合法性 baseline——100% 通过反向证明 corpus 落在 stable Rust 接受面内，**不参与与其他工具的横向能力比较**。

#### 5.2.2 Translator family — single-stage (charon)

| tool | n | S | F | rate (95% CI) | Measurement boundary |
|---|---:|---:|---:|---|---|
| charon-mono | 161 | 153 | 8 | 95.0% [90.5, 97.5] | MIR → LLBC, `--abort-on-error` |
| charon-poly | 161 | 154 | 7 | 95.7% [91.3, 97.9] | MIR → LLBC, `--abort-on-error` |

McNemar exact test (mono vs poly): only-mono = 2, only-poly = 3, p = 1.0000 — **not significantly different**。

#### 5.2.3 Translator family — cascade (Aeneas backends, share charon stage 1)

| tool | n | S | F | rate (95% CI) | Measurement boundary |
|---|---:|---:|---:|---|---|
| aeneas-coq | 161 | 98 | 63 | 60.9% [53.2, 68.1] | charon + aeneas mid-end + Coq printer |
| aeneas-fstar | 161 | 98 | 63 | 60.9% [53.2, 68.1] | + F* printer |
| aeneas-lean | 161 | 98 | 63 | 60.9% [53.2, 68.1] | + Lean printer |
| aeneas-hol4 | 161 | 65 | 96 | 40.4% [33.1, 48.1] | + HOL4 printer |

McNemar：{coq, fstar, lean} 三两两 p = 1.0000 — **identical SUCCESS sets**（charon stage + aeneas mid-end 共享，printer 几乎不 fail）。{coq, fstar, lean} vs hol4: p = 0.0000 — **significantly different**（HOL4 backend 33 unique FAILED 来自 `trait_decl_kind_to_qualif` OCaml panic）。

#### 5.2.4 Translator family — cascade (Hax backends)

| tool | n | S | F | rate (95% CI) | Measurement boundary |
|---|---:|---:|---:|---|---|
| hax-coq | 161 | 113 | 48 | 70.2% [62.7, 76.7] | hax-engine + Coq printer |
| hax-fstar | 161 | 130 | 31 | 80.7% [74.0, 86.1] | hax-engine + F* printer |
| hax-lean | 161 | 127 | 34 | 78.9% [71.9, 84.5] | hax-engine + Lean printer |

McNemar: coq vs fstar p = 0.0000 (only-coq = 0, only-fstar = 17) — **F* printer SIG accepts more**; coq vs lean p = 0.0013 (only-coq = 2, only-lean = 16); fstar vs lean p = 0.5811 — **not significantly different**。

#### 5.2.5 Translator family — single-stage (Rocq-of-Rust)

| tool | n | S | F | rate (95% CI) | Measurement boundary |
|---|---:|---:|---:|---|---|
| rocq-of-rust | 161 | 123 | 38 | 76.4% [69.3, 82.3] | Rust → Rocq translator (7 wrapper gates) |
| rocq-of-rust-typecheck | 161 | 124 | 37 | 77.0% [69.9, 82.8] | + Stage 2 `coqc` typecheck (3 more gates) |

#### 5.2.6 Verifier family — deductive, sliced to front-end per §六

| tool | n | S | F | rate (95% CI) | Measurement boundary |
|---|---:|---:|---:|---|---|
| kani | 161 | 159 | 2 | **98.8% [95.6, 99.7]** | `--only-codegen` MIR → GotoC (5-marker gate + §六 reverse-evidence) |
| creusot | 161 | 121 | 40 | 75.2% [67.9, 81.2] | cargo-creusot → coma (no solver) |
| prusti | 161 | 71 | 90 | 44.1% [36.7, 51.8] | `PRUSTI_NO_VERIFY=false` + `PRINT_HASH` (encoder runs, Silicon does not) |
| verus | 161 | 66 | 95 | 41.0% [33.7, 48.7] | `--no-verify --log vir` (VIR built, AIR/Z3 skipped) |

#### 5.2.7 Verifier — separation logic (VeriFast)

| tool | n | S | F | rate (95% CI) | Measurement boundary |
|---|---:|---:|---:|---|---|
| verifast | 161 | 13 | 148 | 8.1% [4.8, 13.3] | full verification + corpus 0 `//@` annotations → vacuous-pass detection |

**Caveat**: verifast 8.1% **not comparable** to 其他 translator——measures "entries that compile + symex-touch user source without `//@` annotations". 13 SUCCESS are auto-generated struct predicates, not user-spec verification. 详 `deep-reports/cc-reports/verifast.md`。

#### 5.2.8 Abstract interpreter / symbolic execution — bug-detect path

| tool | n | S | F | rate (95% CI) | Measurement boundary |
|---|---:|---:|---:|---|---|
| miri | 161 | 158 | 3 | **98.1% [94.7, 99.4]** | abstract interpretation full execution + UB detection (bug-detect = SUCCESS) |
| soteria | 161 | 126 | 35 | 78.3% [71.3, 83.9] | symbolic execution + bug-detect = SUCCESS |
| kmir | 161 | 61 | 100 | 37.9% [30.8, 45.6] | K Framework interpreter (K-stuck detection) |

### 5.3 Cross-Family 描述性发现（不构成 ranking）

按家族汇总（**不构成跨家族 ranking**——见 §7.3 T-E2）：

- **Translator family** 在各自 boundary 上的接受面 ranges from 41-96%：
  - Single-stage 浅 translator（Charon）95% 量级——Rust → LLBC 翻译保留 MIR 大部分结构，rejected case 集中在 `inline_asm` / `simd_intrinsic` / `coroutine` 等"前端语法即拒"。
  - Cascade translator（Aeneas / Hax）通过率显著低于其 stage 1（charon stage 单独 95% → Aeneas + 3 printer 60.9% → HOL4 backend 40.4%）——cascade 末端 printer 引入额外 unsupported pattern；HOL4 OCaml panic 是 backend code 自身 bug，非 Rust 特性边界。
  - Hax 三 backend 的 printer 间分化（F\* / Lean 80% 量级 vs Coq 70%）反映 printer 端的 trait / generic 处理差异——同 cascade 不同 backend 在同 corpus 上不一致是 backend-level 而非 frontend-level 信号。
- **Verifier family (deductive)**：在前端切到 codegen / VIR / encoder 后通过率 41-98.8%——分化主要来自 toolchain pin（Prusti 锁 nightly-2023-08 拒 edition 2024 / `let-else` 等新 feature 是 confounder）与单文件 pipeline 设计（Verus 不读 Cargo.toml deps）。
- **Verifier (separation logic)**：VeriFast 8.1% 是 corpus 0 spec annotation 设计下的低 bound——本 corpus deliberate 不含 `//@ req/ens`，verifast 不进入真 verification 路径，13 SUCCESS 全为 auto-generated struct predicate。这一数字不应解读为 VeriFast 能力——需要 spec-bearing corpus 重测才有可比意义（future work）。
- **Abstract interpreter / symbolic executor**：MIRI 98.1% / Soteria 78.3%——它们的 measurement boundary 是完整执行 + bug-detect = SUCCESS，与 verifier-style 工具的前端切点不可直接比较（见 §3.5 对称性论证）。

### 5.4 Vendor-Lint Strictness（10 个 UNKNOWN）

v6 final 残留 10 UNKNOWN 全部在 `industrial/x509-parser/cert-parse/` 上：aeneas-hol4 / hax-coq / hax-fstar / hax-lean / kani 五工具各 2 entries（`x509_parse_der` + `x509_subject_extensions`）。归因：vendored crate `vendor/x509-parser` 含 `#![deny(unused_qualifications)]`，在更严 rustc 下触发 lint denial。属 UNKNOWN (b) 类（我们 corpus 引入的 vendored crate lint），修复路径是 vendor crate 治源（DP-10，长期，未实施——动 vendored crate 破坏可复现性）。

### 5.5 0 误报 / 0 漏报 状态（实测 + 独立 audit 验证）

按 `principles.md` §八 disprove-first 审查协议，v6 final 经独立 c-route + cc-route 双轮审查：

- **0 误报**：1001 FAILED 中独立 agent grep wrapper 内部 bug 信号（`unbound variable` / `command not found` / `viper_tools` / `core dumped`）**0 命中**；按本地性 / 社区惯例 / 最大善意三原则栈逐类排除——5 类候选 0 个落地（详 §4 audit 闭环）。
- **0 漏报**：c-route 在 2202 SUCCESS pool 中发现 4 个候选（aeneas 3 entries / ror-typecheck 1 entry 自陈 Warn 通道 partial），cc-route 反挑后全 4 成立，落地 R7 fix（aeneas 4 wrapper 加 Warn 通道 grep gate / ror-typecheck wrapper 同步 D3.1 gate）。重跑后实测 6 个 SUCCESS 翻转为 FAILED（aeneas-hol4 因独立 OCaml panic 已 FAILED 故新 gate 无新增翻转）。

完整 audit 链 + counter-rebuttal 见 `docs/audit/v6-issta-cc-rebuttal-2026-05-12.md`。

---

## 6. Discussion

### 6.1 工具实用性 ≠ Pass Rate

本文测的 pass rate 是**单一切点上的 feature acceptance**，**不**等于工具对最终用户的实用性。例：

- VeriFast 8.1% 在本 corpus 下低——但 VeriFast 在 spec-bearing C / Rust 代码上的真 verification 能力是 well-established 的。本 corpus 0 `//@` annotation 是 deliberate corpus design 选择，使 VeriFast 不进入真 verification 路径——这测的是"在 spec-less corpus 上能否做 vacuous-pass detection"而非"VeriFast 验证能力"。
- Prusti 44.1% 包含 toolchain pin (nightly-2023-08) 的 confounder——Prusti 在其原 toolchain 支持的 Rust 子集内能力远不止 44.1%。

这一限制在 §7.3 T-E5 / T-E1 中正式列入 External Validity threats，并在 §5.3 描述时一律避免跨家族 ranking 措辞。

### 6.2 跨工具比较的边界

按 `principles.md` §四 B 等距推论："测接受 vs 不接受，不区分翻译深浅"——syntactic 搬运 / 深 MIR 翻译 / verifier dialect 接受一律不加权。但读者仍可能直觉对比"浅 translator 95% vs 深 verifier 41%"——这是 reading hazard，不是 data hazard。Mitigation：

- §5.2 表按家族分组而非按通过率 desc 排序。
- 每个 sub-table 加 Measurement boundary 列明示切点 nature 异质。
- §3.5 输出形态对称性论证写入 paper 主文，让读者理解 bug-detect SUCCESS / 前端切点是 deliberate symmetric design。

### 6.3 Findings 摘要

- **Vendor lint 治源价值**：10 UNKNOWN 全集中在一个 vendored crate 的 lint denial——一处 corpus-level 修源可消除整批 UNKNOWN。说明 corpus 自身的 vendor crate 选择对测量结果有显著影响（External Validity T-E3 相关）。
- **Cascade translator 的 backend-level 分化**：Aeneas 同一 charon + mid-end 接 4 不同 printer 通过率从 60.9% 到 40.4%；Hax 同一 hax-engine 接 3 不同 printer 通过率从 70.2% 到 80.7%。这一分化是 printer-end 工程实现差异，非 Rust feature 边界差异——对工具开发者有 actionable 价值（指向 printer code 的 unsupported pattern）。
- **bug-detect 对称性的实证有效性**：MIRI 98.1% / Soteria 78.3% 通过 §3.5 对称性论证可被 paper reviewer 接受为 fair——cc-route 中 reviewer C 提出"非对称"finding 被反驳成立（详 `v6-issta-cc-rebuttal-2026-05-12.md` §2）。
- **Wrapper 归属切分（FAILED vs UNKNOWN）的实证有效性**：v6 1001 FAILED 中 0 误报候选——本地性原则下 FAILED 入册经独立 agent 双轮审查不可驳回。

---

## 7. Threats to Validity

按 Wohlin et al. [wohlin2012experimentation] 四类 validity 框架 + 本文项目特殊的对称性论证。

### 7.1 Construct Validity

核心 construct："feature acceptance breadth"。Threats：

- **T-C1: "feature coverage" 命名 vs operationalization**：oracle 在 entry 粒度判定，未做 feature-level roll-up。pass rate 数字反映 (tool, entry) 对的接受率；feature 层"覆盖"含义需读者通过 corpus feature directory 自行归纳。**Mitigation**：corpus 按 feature 分目录（41 features），每 feature ≥ 1 entry；§5.2 给 per-tool breakdown 而非 per-feature roll-up，避免过度归约。
- **T-C2: "前端测量"切点异质**：每工具切点 nature 不同（kani `--only-codegen` / verus `--no-verify` / verifast 完整 verification + corpus 0 spec annotation / MIRI 抽象解释整段执行 / Soteria 符号执行整段）。**Mitigation**：§5.2 表加 Measurement boundary 列显式标注；§3.5 输出形态对称性论证写入 paper 主文。
- **T-C3: "形式可证 0 误报 / 0 漏报" 措辞**：实质是 single-exit-code-channel invariant argument + 实测验证，**非 machine-checked proof**。已 in P38 把 charon × 2 + cargo-check 措辞降级为 "by-design no-silent-skip + source-level argument"。Aeneas / Hax / Kani / Verus / Prusti / Creusot 已在 P30 + P32 + P35 + P37 cc-route audit 落地诚实声明。

### 7.2 Internal Validity

核心问题：测量结果归因到工具能力是否被混杂变量污染？Threats：

- **T-I1: Harness instrumentation effect**：每工具 harness 模板不同（Verus 必须 `verus! {}` 包 / kmir 用 K-stuck grep / 等）。**Mitigation**：宪法 §四 A "信号文件加入前后 cargo 字节级一致"形式约束；harness 模板按工具自身惯例写（如 Verus 不 `verus! {}` 包工具就拒绝，是工具 contract 而非框架 instrumentation）。
- **T-I2: Oracle 切点选择**：kani `--only-codegen` / verus `--no-verify` / prusti env trick 等切点是项目 deliberate design choice（per §六 前端测量）。**Mitigation**：每个工具 README 明示切点 + cc-report 详细论证切点合理性；P36 / P37 §六 三路径补对称性。
- **T-I3: `[env]` 修改 cargo build 行为**：x509-parser entry 加 `RUSTFLAGS=--cap-lints=warn` (P33)——改 vendor crate lint denial 的 build-time 行为，是 corpus 适配？**Mitigation**：宪法 §四 A 形式定义只要求 cargo 字节级——`[env]` 由 runner 在 spawn 时附加，cargo 自身不读 hirusttest，符合 A 精神；但 reviewer 仍可挑刺——明确列入 ToV 接受为 limitation。
- **T-I4: `extra_cargo_deps` (Creusot)**：creusot tool.toml 让 runner 把 `creusot_contracts` 依赖 inject 到工作副本 Cargo.toml——局部为工具改 deps。**Mitigation**：宪法 §四 C 异质性归配置（声明数据，非框架代码 if 分支）；Creusot README 明示此契约；与每个 entry 自身 src 代码无关。
- **T-I5: `Cargo.lock` 跳过**：runner 跳过 Cargo.lock copy 以避免 Prusti pinned old toolchain 不识 v4 lockfile。**Mitigation**：feature-coverage screening 不需要跨 run 锁 dep；明示 in `detailed-design.md` §四 隔离机制。

### 7.3 External Validity

核心问题：结果能 generalize 吗？跨工具比较是否暗示公平 ranking？Threats：

- **T-E1: Pass-rate 排序的 ranking 暗示**：早期 v5/v6 报告按 pass rate 排序大表会给读者"工具 X 优于工具 Y"误读。**Mitigation**：P38 起 §5.2 按家族（baseline / translator subtypes / verifier subtypes / abstract interpreter / kmir）分组，**不按 pass rate desc**；每表加 caption + footnote 提示不可比较性。
- **T-E2: 不同深度工具同表呈现**：浅 translator (Charon 95.7%) vs 深 verifier (Verus 41.0%) 同表会让读者直觉对比。**Mitigation**：§六 "不区分翻译深浅" 原则 + §5.2 表按家族分组 + Measurement boundary 列明示每工具切点。
- **T-E3: 161 entries → crates.io 2026 generalize**：corpus 是项目作者选材，未做 representative sampling。**Mitigation**：宪法 §五 "examples 多样性" 设计（单特性 / 边界 `*-limit/` / 综合 `industrial/` 多梯队）；project 主动声明"测试报告不构成对工具能力的长期承诺"。
- **T-E4: bug-detect 跨工具非对称**：MIRI / Soteria 触发 bug-detect path，其他工具 deliberate 切前端不进入此 path。这非 unfair——见 §3.5 对称性论证 + cc-rebuttal §2；§5.2.8 / §5.2.6 表布局清晰区分这两类工具。
- **T-E5: Toolchain pin 副作用**：Prusti 锁 nightly-2023-08，拒新 edition feature。Pass rate 含此 confounder。**Mitigation**：§5.2 Measurement boundary 列加 toolchain 信息；Prusti README 明示。

### 7.4 Conclusion Validity

核心问题：每个数字 claim 是否 statistically sound？Threats：

- **T-V1: Single-run determinism**：20 工具是确定性程序——给同样 input + env，输出 byte-identical。Single run 足够；Multi-run 不会给出额外信息。**Mitigation**：唯一已知非确定性 Rocq-of-Rust（翻译路径），项目已用 N=7 attempts AND-reduce 处理；其他 19 工具切前端层不进入 BMC/SMT 求解，无 timeout flip 风险。完整 reviewer rebuttal 见 `docs/audit/v6-issta-cc-rebuttal-2026-05-12.md` §1。
- **T-V2: Pass-rate 无置信区间**：v5.1 仅给 % 不给 CI。**Mitigation**：P39 §5.2 加 Wilson 95% 二项 CI（per tool n = 161，CI 一般 ±5-10pp）。
- **T-V3: 同族对比无显著性检测**：Aeneas × 4 / Hax × 3 / Charon × 2 同表比较是否 SIG？**Mitigation**：P39 §5.2 加 McNemar exact p-value（Aeneas {coq, fstar, lean} = identical / hol4 SIG; Hax F* / Lean printer SIG accepts more than Coq）。
- **T-V4: 样本量 161 是否足**：41 features 平均 ~4 entries/feature。对 statistical significance 二项 CI ±5-10pp 已可接受；对 feature-level saturation 论证不充分。**Mitigation**：corpus 设计单特性 + 边界 + 工业三梯队，每梯队针对不同维度；扩 corpus 是 long-term plan。

### 7.5 时空锚定 + 不构成长期承诺

按 `principles.md` §三 模块定位（次要模块时效性）+ §六 时空锚定：

- 所有数字锚定 (run id `run-1778560393-59119`, host = Apple M5 / macOS 25.4.0 / aarch64 10 cpus, ISO 时间 2026-05-12, commit `936311b`)。
- 工具版本快照见 `docs/test-reports/feature-coverage-2026-05-12-v6.md` §0.2。
- 工具升级后旧结果解释力衰减是必然的、不是缺陷。
- 测试报告**不构成对工具能力的长期承诺**——本快照仅说"在这个时间点 + 这些工具版本 + 这个 corpus 下，可观察到这些数字"。

---

## 8. Conclusion

本文提出一个 tool-neutral 测量框架解决 Rust 验证工具生态的不公平 / 不公信两大根本问题。框架核心 contributions：(1) 把工具异质性沉淀到第三方中介层（runner + 声明式 tool.toml + Tera 模板），样例与工具两端各自纯粹；(2) 双切割边界（深度：前端测量 / 宽度：当前 crate 焦点）+ 三路径 oracle 实施（A: source-path filter / B: reverse-evidence grep / C: 单文件自然满足）+ bug-detect 派生 + 工具输出形态对称性论证，使"特性接受面"可被 fair 测量；(3) 20 工具 × 161 entries × 41 features 的 v6 final 快照，按家族分组 + Wilson CI + McNemar exact test 呈现，公开 reproducible artifact；(4) UNKNOWN 严格语义 + wrapper 归属切分 + disprove-first 审查协议，使 oracle 设计可被审计、challenge、扩展。

**Future work**：

- 扩 corpus 到 spec-bearing 形态（含 `//@ req/ens`），让 VeriFast / 完整 verification 路径有 fair measurement boundary——本文当前 corpus 0 spec annotation 设计是 deliberate trade-off。
- 接入更多工具（MIRAI / ESBMC-Rust / RustHorn / 等），按 `architecture.md` §二 通用性论证扩 token 形态覆盖域。
- Per-feature roll-up 报告（T-C1）——把 (tool, entry) 矩阵 aggregate 到 (tool, feature) 矩阵，给读者 feature-level 视角。
- 接入 SV-COMP-style verdict 比对作为可选 oracle dimension——保留本文 feature-acceptance 主轴的同时给 verifier-family 工具一个独立的 correctness 维度。

---

## References

按 `docs/publish/tool-citations.md` 整理 11 distinct citations + Wohlin / SV-COMP 共约 13 条。Camera-ready 前会扩为完整 BibTeX entries 与 DOI 校验。

```
[ho2022aeneas]       Son Ho and Jonathan Protzenko. "Aeneas: Rust Verification by
                     Functional Translation." Proc. ACM Program. Lang. (ICFP 2022).
                     doi:10.1145/3547647

[vanhattum2022kani]  Alexa VanHattum et al. "Kani Rust Verifier." CAV 2022.
                     arXiv:2208.05545

[astrauskas2019leveraging]  Vytautas Astrauskas, Peter Müller, Federico Poli,
                            Alexander J. Summers. "Leveraging Rust Types for Modular
                            Specification and Verification." OOPSLA 2019.
                            doi:10.1145/3360573

[denis2022creusot]   Xavier Denis, Jacques-Henri Jourdan, Claude Marché. "Creusot: a
                     foundry for the deductive verification of Rust programs." VSTTE
                     2022. doi:10.1007/978-3-031-25803-9_6

[lattuada2023verus]  Andrea Lattuada et al. "Verus: Verifying Rust Programs using
                     Linear Ghost Types." OOPSLA 2023.

[jacobs2011verifast] Bart Jacobs et al. "VeriFast: A Powerful, Sound, Predictable,
                     Fast Verifier for C and Java." NASA Formal Methods 2011.
                     doi:10.1007/978-3-642-20398-5_4

[rosu2010k]          Grigore Roşu, Traian Florin Şerbănuţă. "An overview of the K
                     semantic framework." J. Logic Algebra Program. 79(6), 2010.
                     doi:10.1016/j.jlap.2010.03.012

[jung2020stacked]    Ralf Jung, Hoang-Hai Dang, Jeehoon Kang, Derek Dreyer. "Stacked
                     Borrows: An Aliasing Model for Rust." POPL 2020.
                     doi:10.1145/3371093

[hax2024]            Cryspen et al. "hax: a tool for high-assurance translation of
                     Rust." GitHub: hacspec/hax (commit 30949eb87058895c24f963df90dd30ef11b0dc1a).

[rocqofrust2024]     Formal Land. "rocq-of-rust: Translating Rust to Rocq." GitHub:
                     formal-land/rocq-of-rust (commit a8a76a4d).

[soteria2024]        Soteria Tools. "Soteria: Symbolic Execution for Rust." GitHub:
                     soteria-tools/soteria-rust (commit 3c21278187c60c99418fe2dabb03710ce4102896).

[wohlin2012experimentation]  Claes Wohlin et al. Experimentation in Software
                             Engineering. Springer, 2012.

[beyer2019svcomp]    Dirk Beyer. "SV-COMP: International Competition on Software
                     Verification." TACAS 系列年度论文.

[cargo]              Rust Project. "Cargo: the Rust package manager."
                     https://doc.rust-lang.org/cargo/
```

---

## Artifact Availability

本文 reproducible artifact：

- 项目仓库 commit `936311b`（含 runner / corpus / 20 tool integrations）
- Run data: `runs/run-1778560393-59119/results.json`（含 host / 时间 / 工具版本 metadata）
- Corpus: `examples/`（161 entries × 41 features）
- Per-tool reports: `deep-reports/cc-reports/*.md`（20 reports，每报告含 pipeline + oracle 设计 + 失败分桶）
- Audit chain: `docs/audit/v6-issta-cc-rebuttal-2026-05-12.md`（disprove-first 协议双轮记录）

按 ISSTA artifact evaluation 标准：runner 可在 macOS / Linux 上独立运行（cargo 依赖 + 各工具按上游推荐姿势装）；single full run 约 30-60 min on Apple M5 / 10 cpus。
