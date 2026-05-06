# Rust 工具特性能力测绘 testsuite —— 调研报告

> 阶段：调研（workflow §2.1）  
> 首批样本工具：Kani、MIRI（框架设计须对任意 Rust 验证/分析工具透明）  
> 本文档由**原则**驱动——每节从前节推导而来，不是平铺式设计清单。

---

## 1. 核心问题意识

这个项目的核心**不是**"做一个 testsuite"，**也不是**"集成几个工具"。它要回答的是一个结构性问题：

> 当一群**对"输入"的理解互不相通**的 Rust 验证/分析工具被同时摆在面前，如何用**一组对它们都中立**的输入去**筛选**它们？

这是一个**公平性问题**。绝对公平不可达——每个工具都希望按自己的协议被喂代码：MIRI 要 `fn main()`，Kani 要 `#[kani::proof]` 与 `kani::any()`，未来某 SMT-based 或 deductive 工具又会要另一套 attribute / 规约语法。任何**单一**形态的样例都暗中偏向某个工具的世界观。

设计的真正起点因此不是"取哪种胶水形态"，而是：

> **把不公平集中收敛到哪里？**

本项目的回答：

> **把不公平全部沉淀到一个透明、配置化、运行时的中介层；让样例和工具两端各自保持纯粹。**

具体地——

- **样例**不为任何工具修改、不引入任何工具相关依赖、不带任何工具特定标注；它是 100% 普通 Rust 代码。
- **工具**被当作黑盒，框架不去理解其内部如何工作，只观测它的实际输出。
- **框架**（中介层）承担所有 per-tool 差异——但以**配置 + 模板**而非**代码**的形式承担，使新工具的集成只需新增一份配置，不动框架。

这就是整个设计的"意识"。后文每一个具体决定——样例怎么写、工具怎么集成、运行时怎么组织、结果怎么分类——都是这个意识的展开，不是孤立选择。

读者读完本文档后应能不靠回头查表，自行从这条意识出发**预测**任何具体设计选择落点；如果某一具体设计无法被这条意识自然推出，那它就是该被剃刀的累积。

---

## 2. 三条派生原则

第 1 节的意识——"把不公平沉淀到中介层、两端保持纯粹"——分解为三条可操作的原则。它们不是独立公理，而是同一意识的三个面：**对样例端的纯粹要求**、**对工具端的观察立场**、**对中介层自身的形态要求**。

### 原则 A：双方都不可侵入

样例不为任何工具修改、不依赖任何工具相关 crate（如 `propverify`、`verifier` 之类的 helper crate）、不带任何 `#[<tool>::...]` 属性标注、不带任何 `#[cfg(<tool>)]` 分支。它是普通 Rust 代码，可独立 `cargo build`、被任意外部 crate path-dep 引用。

工具被当作黑盒——框架不解析其源、不模拟其内部、不假定其行为，只**观测**其实际输出。

**直接推论**：异质性既不能落到样例端（违反"纯粹"），也不能落到工具端（无法干预），**必然落到第三方——框架**。

### 原则 B：测必要条件，非语义对错

本项目是**筛选**，不是**认证**。

筛选只问一个问题——"工具能不能吃下这段代码并产出它该产出的输出形状"——即必要条件是否满足。"产出是不是语义正确的判定"是后续工业验证阶段才关注的事，**不是筛选阶段的关注点**。

**直接推论**：分类标签只描述"产出形状"的几种情况，不带"对错"评分。语义正确性可作可选附加观测（通过 ID-pattern 的 `expect` 声明），但不影响主分类。

### 原则 C：异质性归配置，框架代码同质

工具间的差异作为**声明数据**（per-tool 配置 + 模板）存在，不作为**控制流**（per-tool 代码分支）存在。

框架代码只知道**标准词汇**：标准 template 变量集（模板可用哪些变量）、标准 ResultClass（输出可分类为哪几种）、标准运行流程（怎么跑一次）。任何 `if tool == "kani"` 都被禁止出现在框架代码里。

**直接推论**：加新工具 = 写一份 `tools/<name>/{tool.toml + templates/...}`，**零代码改动**。配置语言必须足够表达——不能仅是平铺数据，需要条件分支与循环（详见后文模板引擎的推导）。

---

### 三条原则作为判据

任一具体设计争议，把三条过一遍即可：

- 是否要求样例改一行？→ 违反 A
- 是否要在框架代码里 `if tool == "kani"`？→ 违反 C
- 是否要框架自己判语义对错？→ 违反 B

**任何"加东西"的提议先过这三关；任何"减东西"的提议——只要不削弱原则——倾向接受。** 这是奥卡姆剃刀在本项目里的具体形式。

---

## 3. 关键运行时投影

三条原则定下后，**两个二级推论**会在所有下游设计里反复出现。提前显式定下，避免在每个具体章节里重复推导。

### 3.1 A 的运行时投影：运行原子 = 单 entry

原则 A 要求双方都不可侵入。延伸到执行层面：

> **一个 entry 的命运不可与另一 entry 耦合。**

如果把同一样例的多个 entry 在一次工具调用里"批跑"——例如一次 `cargo kani` 同时枚举多个 `#[kani::proof]` harness、或一个 `fn main()` 串调多个 entry——那么任意一个 entry 上的失败（不支持、内部错误、超时）都会污染其他 entry 的可观测性：要么整批失败、要么输出难以拆分。

**操作原子因此降到 entry 层**：

- 一次 worktree、一次 harness 渲染、一次工具进程，对应**唯一一对** `(tool, entry, mode)`。
- 报告矩阵的行是 entry，**不是** example。
- 多 entry 共享一个 cargo crate 的好处仅止于"代码组织"（共享 src 的 helper、共享 dependencies），**不延伸到运行批处理**。
- cargo 编译缓存因此无法跨 entry 直接共享——**显式代价**，可接受：换来每个 entry 独立可观测、互不污染。

### 3.2 A ∩ B 的运行时投影：能力靠观测，不靠声明

原则 A 要求不替工具发言，原则 B 要求只看产出形状。两条相交：

> **工具的处理能力是框架通过实际跑然后 mapping 出的结果，不是框架通过 config 字段假设的输入。**

具体地：

- 框架**不预跳过**任何 `(tool, entry, mode)` 组合——所有组合实际跑。
- tool 配置里**没有** `supported_kinds = [...]` / `skip_entries = [...]` 这类**能力声明**或**排除清单**字段。一旦出现这种字段，就意味着框架替工具说"它做不了 X"——违反 A。
- 工具无法处理某情形时，"不能处理"这一事实通过两条**等价观测路径**之一浮现：
  1. **工具自身的拒绝输出**（如 MIRI 的 `unsupported operation: ...`），由 tool.toml 的 mapping 规则映射到 ResultClass。
  2. **per-tool 模板对自身确知做不了的情形主动产生有标识的失败**，例如：
     ```rust
     compile_error!("ts_unsupported: <reason>");
     ```
     同样由 mapping 规则映射回。
- 框架视角下两条路径**无差别**：跑 → 收输出 → 按 mapping 出标签。

### 3.3 这两条对下游章节的约束

| 下游章节 | 来自 3.1 的约束 | 来自 3.2 的约束 |
|---|---|---|
| §4 样例形态契约 | entry 是注册最小单位、每 entry 自有固定 ID | — |
| §5 工具集成契约 | — | tool.toml 不含 capability 声明 / 排除清单字段；模板需有条件分支能力 |
| §6 运行时机制 | worktree / 注入 / 进程都按 `(tool, entry, mode)` 笛卡尔积粒度调度 | 不预过滤组合；每组合都跑 |
| §7 结果分类与报告 | 矩阵行 = entry | 分类来自观测，每 cell 都有 ResultClass + 可选 reason |

---

## 4. 样例形态契约

样例（example）是被工具吃下去的那段 Rust 代码。本节定下样例长什么样、提供哪些信息、不提供哪些信息。

### 4.1 推自原则 A 的不可决项

| 推论 | 推导链 |
|---|---|
| 样例**不依赖**任何工具相关 crate（如 `propverify`、`verifier` 之类 helper crate） | A：引入即破中立 |
| 样例**不出现** `#[<tool>::...]` 属性、`#[cfg(<tool>)]`、`extern crate <tool>;` | A |
| 入口函数必须是 `pub fn` 可达、单态化的普通 Rust 函数 | A：中介层从外部调它，不能为单态化去改样例 |
| 入口函数是**零参** `pub fn name() -> R`（R 任意） | A + §3.1 + 测试入口范围划界（见 4.2） |
| ID 由 `<dir 路径> + <fn 名>` 自动派生，**不写** `id` 字段 | A：path 是事实，metadata 不重复声明 |

**副产物**：每个样例是 100% 普通 Rust 库——把 `testsuite.toml` 拿掉、剥离 `examples/` 上下文，每个样例仍可独立 `cargo build`、被任意外部 crate path-dep 引用。这是"侵入性零"的可观测证据。

### 4.2 范围划界：测试入口 vs 参数入口

本项目的目标是测试工具对 **Rust 特性的覆盖广度**——开放式枚举、随发现增长。每个 entry 之所以存在，是因为它的函数体**承载了若干具体 Rust 特性**；"工具能不能处理它" = "工具能不能处理含这些特性的代码"。

样例提供的是 **测试入口（test entries）**——零参 `pub fn`，函数体以侵入式方式写入要测的代码。

样例 **不提供"参数入口"**——即 `fn(args)` 形式的、需要外部喂参数才能调用的 entry。理由：

- 参数化测试本质是**工具功能测试**，不是覆盖测试，类目错位
- "如何构造参数 / 如何喂参数"是**工具选择阶段**的事，不在本项目范围
- 想覆盖"在某具体输入下使用了某特性"的场景？作者写零参 wrapper：
  ```rust
  pub fn vec_with_specific_seq() {
      let mut v = Vec::new();
      v.push(1); v.push(2); let _ = v.pop();
  }
  ```
  注册它为 entry——对框架仍是"零参 `pub fn`"，对工具仍是"我能不能吃下这段代码"。

### 4.3 已确立的设计选择

- **永远 cargo 项目**——每个样例是独立的 cargo 项目；不接受单文件 `.rs` 形态。推自 A——避免 runner 里堆"单文件 vs cargo"双路径分支。
- **样例必须是 lib crate**——目标 crate 必须含 `src/lib.rs`，entry 以 `pub fn` 出现在 lib 中。原因：harness 在 `src/bin/__ts_harness.rs` 中通过 `<crate>::<entry_fn>()` 调用 entry，需 lib 暴露之。
- **多 entry per example 允许**——entry 是平等独立、各自有固定 ID 的；example 只是它们共享 helpers / dependencies 的 cargo 容器。
- **每个 example 是独立 cargo 项目**，不属于全局 workspace——避免跨 example 的依赖耦合。
- **多 crate 样例**通过 `target_path` 字段声明被测 crate（相对样例根的路径，默认 `.`）。

### 4.4 ID 派生

ID 格式：

```
<feature>/<dir>/<entry-fn>
```

- `feature` 取自 `examples/` 下第一级目录名，自由字符串（建议在 `docs/feature-tags.md` 维护一致性）
- `dir` 取自第二级目录名（即样例 cargo 项目根目录名）
- `entry-fn` 取自 `testsuite.toml` 中 `entries` 注册项

示例：

```
vec/basic-ops/vec_push_pop_basic
vec/basic-ops/vec_drain_partial
unsafe-ptr/use-after-free-1/buggy_call
```

ID 由路径派生意味着：路径或 fn 改名 → ID 随之改。这是预期行为——"重组目录"是有意义的结构变更，理应反映到 ID。"固定 ID"指**只要不重组就稳定**，不是指 ID 与磁盘解耦。

不再设 `clean | bug` 的 category 二分前缀（按 §5 通用性论证下的极简方向）。bug 类样例的"含 bug"语义靠 src + git log + 描述性 dir 名（如 `unsafe-ptr/use-after-free-1`）承载。

### 4.5 testsuite.toml schema

最小契约——两个字段：

```toml
# entries 必填——零参 `pub fn` 的函数名列表
entries = ["vec_push_pop_basic", "vec_drain_partial"]

# 仅多 crate 样例需要——被测 crate 相对样例根的路径，默认 "."
target_path = "crates/core"
```

**没有的字段**（每条对应一次剃刀）：

- 无 `id`——派生自路径
- 无 `category` / `feature`——派生自路径
- 无 `params` / `return_type` / `signature`——entry 是零参，框架不解析签名
- 无 `entry_kind`——所有 entry 等价，无 fixed / parametrised 二分
- 无 `expected_verdict`——期望走 `tool.toml` 的 ID-glob，不进 example metadata
- 无 `bug_description`——bug 语义靠 src + git log + ID 名承载（未来若接 LLM 做匹配可考虑加）

### 4.6 一个完整样例的形态

```
examples/vec/basic-ops/
├── Cargo.toml
├── src/
│   └── lib.rs
└── testsuite.toml
```

`lib.rs`：

```rust
pub fn vec_push_pop_basic() {
    let mut v = Vec::new();
    v.push(1);
    v.push(2);
    let _ = v.pop();
}

pub fn vec_drain_partial() {
    let mut v = vec![1, 2, 3, 4, 5];
    let _: Vec<_> = v.drain(1..3).collect();
}
```

`testsuite.toml`：

```toml
entries = ["vec_push_pop_basic", "vec_drain_partial"]
```

每个 entry 的函数体即测试本身。无 `panic!`、无 `assert!`——只关心"工具能不能吃下这段代码"。

---

## 5. 正确性论证：通用性

本节回答一个根本问题：本框架对市面上所有 Rust 嵌入式验证工具是否通用？如果不通用，框架就不该按这个形态走。

### 5.1 嵌入式验证的不可逃避核心

"嵌入式验证"指验证工具在**被验证的源码项目之内**运行——读 cargo 项目、识别源码内的标记/规约、产出诊断。与之对立是"外部验证"——读独立 spec 文件或编译产物。

**Rust 验证生态的主流工具皆为嵌入式**：Kani、MIRI、Verus、Prusti、Creusot、MIRAI、ESBMC-Rust、`cargo check`——都通过 cargo 子命令运行、都读 Rust 源码项目、都在源码中识别标记。

任一嵌入式验证活动可分解为三元组：

```
(被分析的代码 P,  指示"分析什么"的 token T,  工具的执行命令 C)
```

- **P**：cargo 项目（Rust 嵌入式验证的事实标准）
- **T**：以工具能识别的源码形式存在于 P 之内（attribute / 函数 / 宏调用 / 测试标记 / 隐式入口如 `fn main`）
- **C**：作用于 P 的命令行调用

任何嵌入式验证 = 此三者的组合。

### 5.2 工具具体做什么

不严格地说，工具的工作流是：

1. 接受 cargo 项目 P
2. 在 P 内识别 token T（在源码扫描或编译阶段完成）
3. **沿 T 出发的调用图分析所到达的代码**
4. 输出诊断 / 判定 / 反例

第 3 步的"沿调用图"是关键——只要 entry function 在 T 的可达图上，entry 的 body 就会被工具分析。这是我们能测"feature 是否被工具支持"的根本机制。

### 5.3 必须包含哪些信息

对具体的 (tool, entry) 测试，必须满足：

- **代码本身**：以 cargo 项目形式存在
- **token T**：以工具识别的源码形式存在
- **T → entry 的可达性**：T 出发能在调用图上到达 entry
- **执行命令**：tool-specific 命令字符串

缺任何一项，工具就无法对 entry 实施嵌入式验证。

### 5.4 我们的方法逐项提供

| 必须信息 | 我们如何提供 |
|---|---|
| 代码本身（cargo 项目） | example 自身就是 cargo 项目（§4） |
| token T | per-tool `harness.rs.tera` 渲染到新增的 `src/bin/__ts_harness.rs`——T 在该文件内用工具自己的语法表达 |
| T → entry 可达 | harness 内 `{{ target_crate_name }}::{{ entry_fn }}()` 直接调用，绑定关系平凡 |
| 执行命令 | `tool.toml.command` 原文配置 |

**关键约束**（与原则 A 一致）：T **完全通过新增 `src/bin/__ts_harness.rs` 一个 file 携带**；example 现有源码不动。

### 5.5 关键命题：所有主流工具的 T 都能放进单个新增 file 内

本方法的**充分条件**是：每个工具的 token T 形式都能在单个新增 `.rs` file 内表达。逐一审视：

| 工具 | Token T 形式 | 单 file 可表达 |
|---|---|---|
| Kani | `#[kani::proof] fn p() { ... }` | ✓ proof + main 都在新 file |
| MIRI | `fn main()`（标准 Rust 入口） | ✓ |
| Verus | `verus! { fn p() requires/ensures ... }` 宏块 | ✓ external function 引用 |
| Prusti | `#[ensures(...)]` 等 attr 在 caller 上 | ✓ caller 在新 file 内调 entry |
| Creusot | `#[ensures(...)]` 等 attr 在 caller 上 | ✓ 同上 |
| MIRAI | `mirai_annotations::*` 调用 | ✓ |
| ESBMC-Rust | harness fn + assertion | ✓ |
| `cargo check` | 默认编译，无显式 token | ✓ `fn main()` 足矣 |

**结论**：所有当前主流 Rust 嵌入式验证工具的 T 都可在单个新增 `.rs` file 内表达。

### 5.6 反例域：本方法覆盖不到的工具类型

为完整性列出**无法覆盖**的工具类型：

- **A. 要求修改 entry 函数自身**——例如假想工具要求规约 attr 必须直接在 `pub fn entry` 上、不允许在 caller 上加。与原则 A 直接冲突。**Rust 主流生态中无此类**——这种设计会让工具无法分析任何外部库代码（库作者不会为某验证工具加 attr）。
- **B. 不通过 cargo 运行**——例如直接读单个 `.rs` 源文件的独立工具。**Rust 严肃验证工具普遍以 cargo 子命令为入口**。
- **C. 要求多文件协同特定布局**——例如要求 `code.rs` 与 `spec.rs` 在同包内特定相对位置。**未见实例**。

落入 A/B/C 的工具是把自己排除在"中立测试"之外——这不是本框架的不足，是该工具的设计选择。

### 5.7 一处技术风险与防御

**风险**：若某工具只分析"活码"（post-DCE 后剩下的可达代码），即便 harness 直接调 entry，也可能因调用被消除而 entry body 未被实际分析。

**现状**：Rust 主流验证器都在 **pre-optimization MIR / HIR** 层面分析，DCE 后置。entry body 被分析这一点稳健。

**防御**（如未来需）：在 harness 内对 entry 调用结果加 `std::hint::black_box(...)` 抑制 DCE。属 `tool.toml + harness.rs.tera` 的小幅调整，**不波及框架内核**。

### 5.8 形式结论

> 本方法的覆盖域：  
>
> `{ tool : tool 的 token T 可在单个新增 .rs file 内表达 ∧ tool 通过 cargo 子命令运行 ∧ tool 不要求修改 entry 自身 }`  
>
> 此集合 = **当前所有 Rust 主流嵌入式验证工具**。

具体每个工具的边界细节（如 Kani 在 bin target 内 proof 的具体支持范围、Verus external function 的引用语法）在**实施第一个工具集成时实跑实证**；如有偏差，调整对应 `tool.toml + harness.rs.tera`——**不波及框架内核**。这正是"配置 vs 代码"分离带来的好处：边界探测的修正局部化在工具配置层，框架结构稳定。

---

## 6. 工具集成契约（极简版）

工具集成纯由配置驱动——加新工具 = 写一份 `tools/<tool>/`，**零框架代码改动**。这是原则 C 的具体落实。

### 6.1 目录结构

每个工具一个目录，含两个文件，文件名固定：

```
tools/<tool-name>/
├── tool.toml
└── harness.rs.tera
```

### 6.2 `tool.toml` schema

```toml
command      = ["cargo", "kani", "--bin", "__ts_harness"]   # 必填
timeout_secs = 300                                           # 可选，默认 300
```

- **`command`**：字符串数组，runner 原样作为 argv 传给进程启动 API。第一个元素是可执行文件名（或在 PATH 中可解析），其余为参数。
- **`timeout_secs`**：可选整数，单位秒。框架默认 300——但不同工具实际耗时差异大（`cargo check` 几秒、Kani 验证可能数分钟到上不封顶），按需 override。

其他字段（`env` / `cwd` 等）目前不开，按 Occam 推后到真用上时再加（schema 添加是向后兼容的）。

### 6.3 标准 template 变量集

`harness.rs.tera` 可用变量只有两个：

| 变量 | 类型 | 含义 |
|---|---|---|
| `target_crate_name` | string | 样例 `Cargo.toml` 中 `[package].name`，用于 `<crate>::<fn>` 路径 |
| `entry_fn` | string | `testsuite.toml` 中 `entries` 注册的函数名 |

变量集**固定**，不可由 tool config 扩展（推自 C：契约对所有工具同义）。

### 6.4 框架内置约定

- **渲染目标固定**：`<target_crate>/src/bin/__ts_harness.rs`（cargo auto-discover bin target，零 Cargo.toml 修改）
- **命令的 cwd 固定**：`<target_crate>` 目录（即 `target_path` 解析后的目录）
- **进程启动**：以子进程方式起，stdout / stderr / exit code 全部捕获

### 6.5 三个工具的完整集成

**Kani**

```toml
# tools/kani/tool.toml
command = ["cargo", "kani", "--bin", "__ts_harness"]
```

```rust
// tools/kani/harness.rs.tera
#[kani::proof]
fn ts_proof() {
    {{ target_crate_name }}::{{ entry_fn }}();
}

fn main() {}
```

**MIRI**

```toml
# tools/miri/tool.toml
command = ["cargo", "+nightly", "miri", "run", "--bin", "__ts_harness"]
```

```rust
// tools/miri/harness.rs.tera
fn main() {
    {{ target_crate_name }}::{{ entry_fn }}();
}
```

**cargo check（parse 类）**

```toml
# tools/cargo-check/tool.toml
command = ["cargo", "check", "--bin", "__ts_harness"]
```

```rust
// tools/cargo-check/harness.rs.tera
fn main() {
    {{ target_crate_name }}::{{ entry_fn }}();
}
```

### 6.6 加新工具的工作量

加一号新工具 = 创建 `tools/<name>/`、写一行 toml + 三五行 tera——**零框架代码改动**。这是原则 C 的可观测兑现。

---

## 7. 运行时机制

本节落实如何把 §4 / §6 的契约组合成实际可跑的流程。

### 7.1 总体调度

启动一次完整 run：

1. 扫描 `examples/` 与 `tools/`，构造笛卡尔积 `{(tool, entry)}`
2. 创建 `runs/<ts>/` 根目录
3. 并发派发每对 `(tool, entry)` 到独立执行单元（见 7.2）
4. 每对完成后，原始输出落盘 `runs/<ts>/raw/<tool>/<entry-id-slug>.{stdout,stderr,exit}`
5. 全部完成后写汇总（§8）

### 7.2 单次执行单元

对一对 `(tool, entry)`：

```
1. exec_id = <tool>__<entry-id-slug>
2. work_dir = runs/<ts>/work/<exec_id>/
3. git worktree add <work_dir> HEAD
4. cd <work_dir>/<example-dir>/<target_path>/
5. 渲染 tools/<tool>/harness.rs.tera 用 (target_crate_name, entry_fn) → src/bin/__ts_harness.rs
6. 起子进程跑 tool.toml.command，捕获 stdout/stderr/exit/wall-time，应用超时
7. 落盘 raw outputs
8. git worktree remove <work_dir>
```

### 7.3 worktree 管理

- 用 `git worktree add <dir> HEAD` 创建——HEAD 指向 testsuite 仓库当前提交，原子快照
- 完成后 `git worktree remove <dir>` 清理
- 异常退出未清理的 worktree：下次启动时检测 `runs/` 下残留并 prune

### 7.4 并发

v1 即支持并发——每个执行单元用独立 worktree、互不干扰。两个并发轴：

- **同工具不同 entry 并行**
- **不同工具并行**（同 entry 也可、不同 entry 也可）

并发度由命令行参数控制（如 `--parallel N`），默认值在 v1 实施时按机器测出合理上限定。

### 7.5 超时

- 框架默认 **300 s**
- `tool.toml.timeout_secs` 可 override
- 超时则向子进程发 SIGTERM、5 秒后未退发 SIGKILL；分类 FAILED 并在 raw output 中记录 `__ts_timeout` 标记

### 7.6 崩溃与异常

- 子进程被信号杀（SIGSEGV / SIGABRT 等）→ 非零 exit → FAILED（exit code 在 raw output 中可查）
- worktree 创建失败、模板渲染失败、文件 IO 失败 → runner 自身 panic（这些是 runner 实施 bug，不该静默掉）
- 超时见 7.5

---

## 8. 结果分类与报告

每次 run 产生三类输出：raw（原始）、structured（机读）、summary（人读 + console）。

### 8.1 结果分类

ResultClass 二分：

| Class | 触发条件 |
|---|---|
| `SUCCESS` | 子进程 exit code == 0 |
| `FAILED` | 子进程 exit code != 0、被信号杀死、或超时 |

不再细分 verified / counterexample / parse_passed / rejected / internal_error / timeout / compile_error 等。这些细节通过保留 raw stdout/stderr 让人事后查；分类只承担"工具吃下来了吗"这个唯一信号——这是原则 B 在分类层的兑现。

### 8.2 Raw outputs

每对 (tool, entry) 落盘 4 个文件：

```
runs/<ts>/raw/<tool>/<entry-id-slug>.stdout
runs/<ts>/raw/<tool>/<entry-id-slug>.stderr
runs/<ts>/raw/<tool>/<entry-id-slug>.exit       # 退出码（数字）或信号名（如 "SIGTERM"）
runs/<ts>/raw/<tool>/<entry-id-slug>.timing     # wall-time ms
```

`<entry-id-slug>` = entry-id 中 `/` 替换为 `__`（如 `vec__basic-ops__vec_push_pop_basic`）。

raw outputs 是诊断的最终来源——任何下游分类、报告都从这里推出。

### 8.3 `results.json`（机读）

汇总 JSON，UTF-8，缩进 2 空格：

```json
{
  "run_id": "2026-05-06T12-34-56",
  "started_at": "2026-05-06T12:34:56Z",
  "results": [
    {
      "entry_id": "vec/basic-ops/vec_push_pop_basic",
      "tool": "kani",
      "status": "SUCCESS",
      "exit_code": 0,
      "duration_ms": 4210,
      "raw_stdout": "raw/kani/vec__basic-ops__vec_push_pop_basic.stdout",
      "raw_stderr": "raw/kani/vec__basic-ops__vec_push_pop_basic.stderr"
    }
  ]
}
```

下游脚本基于此做任意切片 / 统计。

### 8.4 `report.md`（人读）

按 feature 分组——每个 feature 一个小表，行 = 该 feature 下 entry，列 = tool：

```markdown
# Run 2026-05-06T12-34-56

## vec

| entry | kani | miri | cargo-check |
|---|---|---|---|
| basic-ops/vec_push_pop_basic | SUCCESS | SUCCESS | SUCCESS |
| basic-ops/vec_drain_partial | FAILED (101) | SUCCESS | SUCCESS |

## unsafe-ptr

| entry | kani | miri | cargo-check |
|---|---|---|---|
| use-after-free-1/buggy_call | SUCCESS | FAILED (1) | SUCCESS |

---
Total: 4 succeeded / 2 failed / 6 total
```

单元格 = `SUCCESS` 或 `FAILED (<exit_code>)`。点 entry 名超链到 raw outputs 是可选实现项（v1 未必加）。

### 8.5 Console output

run 期间每对 (tool, entry) 完成时打一行：

```
[SUCCESS] kani vec/basic-ops/vec_push_pop_basic (4210ms)
[FAILED ] miri unsafe-ptr/use-after-free-1/buggy_call (89ms, exit=1)
```

末尾汇总：

```
---
Total: 4 succeeded / 2 failed / 6 total
report: runs/2026-05-06T12-34-56/report.md
```

---

## 9. 现有相关工作

调研覆盖四个直接相关项目 / 文献，以及一个方法学源头。**结论**：跨工具 Rust 验证 feature 覆盖测试这一 gap 是真实的、被综述明确点名的、且未被填补。

### 9.1 project-oak/rust-verification-tools (RVT)

Google project-oak 出品，2020 启动，**2023 年 7 月归档**，不再维护。

- 路径：用 Rust 抽象层包多个 verifier——`propverify` 模拟 proptest API、`verification-annotations` 是 KLEE 的 FFI shim
- 入口：`cargo-verify --backend klee|seahorn|crux-mir`
- 测试集 `compatibility-test`：跑同代码在 proptest vs propverify 验证行为一致——**不是 feature-coverage 筛选**

**与本项目关键差异**：RVT 要求样例**侵入式使用 propverify API**；本项目要求样例是 plain Rust，由 runner 在外部生成胶水。

### 9.2 soarlab/rust-benchmarks

明确以"跨工具 Rust 验证 benchmark"为目标，接口仿 SV-COMP（assert/assume/nondet）。

- 41 commits、4 stars、主要演示 MIRAI
- 仍要求样例使用 `verifier` crate（assert/assume/nondet）
- README："enable for various verifiers to be easily applied on the benchmarks without having to make any updates to the benchmarks themselves"——**愿景与本项目一致，但实际仍走 verifier crate 抽象**

### 9.3 arXiv 2410.01981 "Surveying the Rust Verification Landscape" (2024)

覆盖 Prusti、Creusot、Aeneas、Verus、Kani、SMACK、SeaHorn、Gillian-Rust、RefinedRust 等近十个工具的综述。

**明确指出**：当前社区**不存在**跨工具 benchmark 或系统的 feature-coverage testsuite；工具选择只能 "manually evaluate"。

这正是本项目要做的——文献层面已被点名为 gap。

### 9.4 AWS verify-rust-std (Rust Project Goal 2024h2)

不是 benchmark，是把多个 verifier 应用到 fork 的 std 库上，以 "Challenges" 形式做实际验证。目标"找出工具共同的不足"。

**与本项目差异**：他们是真实代码上的应用研究；本项目是合成 micro-benchmark 做工具筛选。**互补、不冲突**。

### 9.5 SV-COMP

C / Java 跨工具竞赛体系已 14 届，**未设 Rust 类目**。RVT、soarlab 都借鉴其 assert/assume/nondet 抽象——但这条路径在 Rust 生态中两次尝试都未成熟。

### 9.6 定位

**Gap 是真实的、被综述明确点名的、且未被填补**。两次先行尝试（RVT、soarlab）都走"统一 verifier API + 样例侵入"路径，要求样例围绕抽象写——代价是样例污染 + 抽象本身是新依赖 + 每工具都要适配抽象。两者都停滞。

本项目的差异点：

| 维度 | RVT / soarlab | 本项目 |
|---|---|---|
| 样例对工具的依赖 | 必须用 propverify / verifier crate | 零依赖、plain Rust |
| 工具适配方式 | Rust 代码 + crate + FFI shim | 纯配置：`tool.toml + harness.rs.tera` |
| 评测目标 | 同语义跑通 / 求解结果对比 | feature 覆盖广度 |
| 状态 | 归档 / 低活跃 | greenfield |

这条路径在已知先行尝试里**没人走过**——降低样例与工具双方的耦合代价，代价是放弃"统一 verifier API"那种 SV-COMP 风格的标准化。这与本项目"必要条件 + 配置驱动 + 不锁工具"的目标对路。

---

## 10. 决策记录与待办

### 10.1 关键决策记录

| 决策 | 推自 | 状态 |
|---|---|---|
| 样例为独立 cargo lib crate，零工具依赖 | A | 锁定 |
| 测试入口为零参 `pub fn`，无 parametrised 形态 | §1 + A | 锁定 |
| ID 格式 `<feature>/<dir>/<entry-fn>`（无 category） | C + Occam | 锁定 |
| 工具集成 = `tool.toml + harness.rs.tera` 两文件 | C | 锁定 |
| 标准 template 变量 = `target_crate_name`、`entry_fn` 两个 | C | 锁定 |
| 渲染目标固定 `src/bin/__ts_harness.rs` | A + cargo auto-discover | 锁定 |
| 隔离机制：per-execution `git worktree add` | A | 锁定 |
| 运行原子 = 单 entry，不批跑 | §3.1 | 锁定 |
| ResultClass 二分（SUCCESS / FAILED） | B + Occam | 锁定 |
| 不预跳过任何 (tool, entry) 组合（capability 靠观测） | A ∩ B | 锁定 |
| 砍掉 external_lib mode | Occam | v1 锁定 |
| 砍掉 verdict 期望比对（`expect.<glob>`） | B | v1 锁定 |
| 砍掉 category clean/bug 二分 | Occam | v1 锁定 |

### 10.2 推后到 v2 的扩展点

不是被砍死，而是当前需求下 Occam 推后；未来真用上再加：

- **`tool.toml.env`** 字段——某工具需 `MIRIFLAGS` / `RUSTFLAGS`
- **`tool.toml.cwd`** 覆盖——若有工具不在 target_crate dir 跑命令
- **External_lib mode**——测样例作为 path-dep 时的工具行为差异
- **期望比对（`expect.<glob>`）**——从 SUCCESS / FAILED 进一步区分 verdict 正确性
- **`bug_description`**——接 LLM 做 bug 描述匹配（用户 2026-05-06 提及）
- **报告里 entry 名超链到 raw outputs**
- **跨运行的对比**——run N 与 run N-1 的差异分析

### 10.3 实施前必跑的经验探针

实施第一个工具集成时实跑实证，如有偏差，调整对应 `tool.toml + harness.rs.tera`，**不波及框架内核**：

- **Kani**：bin target 内的 `#[kani::proof] fn` 是否完全等价于 lib 内？`cargo kani --bin __ts_harness` 是否启该 bin 内的所有 proof？
- **MIRI**：`cargo +nightly miri run --bin __ts_harness` 在仅 lib 的 crate 加 `src/bin/` 后能否跑？
- **cargo-check**：`--bin` 选项的退出码语义是否仅与编译相关？
- **Verus**（v2）：external function 引用的具体语法、宏块内 fn 是否可调外部 crate 函数？

### 10.4 进入下一阶段

调研报告完成。**workflow §2.4 设计审查**通过的前提是这三条：

1. 架构能实现原定目标——§1（核心问题意识）+ §5（通用性论证）已论证
2. 各模块设计满足前置规约——§4（样例契约）+ §6（工具契约）+ §7（运行时机制）+ §8（结果与报告）落地
3. 文档无歧义、可直接用于开发实现——§10.1 决策记录可作 cross-check 表

下一步进入 **workflow §2.2 架构阶段**。考虑到本项目体量小、§4–§8 已基本覆盖架构所需的模块边界与契约规约，预计架构文档可大幅 reuse 本调研报告 + 补少量"模块功能规约"形式化即可。
