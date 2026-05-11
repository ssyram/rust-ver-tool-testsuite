# hax-lean 翻译产物运行一致性测试 — 框架化设计稿（2026-05-11）

> **状态**：设计稿，**不实施**。仅做设计层推导与决策记录。
> **前置**：[`../research/translation-correctness-feasibility-2026-05-11.md`](../research/translation-correctness-feasibility-2026-05-11.md) 已实测验证 hax-lean 三档可达（typecheck / evaluate / consistency 在 `custom_add` 一例上 OK）。本设计把这条实测路径**框架化**到 runner / corpus / tool.toml 层面，使其成为可重复执行的常规测试。
> **范围边界**：本设计仅处理 **hax-lean**。hax-coq / rocq-of-rust 的对应方案被显式排除（feasibility §6.1 已识别两者在本机不可达，阻塞在 prelude 编译，属工具上游问题，不在 testsuite 主线）。

---

## §0 设计任务总览与位阶澄清

### §0.1 一致性测试是"档 3"，与现有"档 0"并存而非替代

按 feasibility §1 的四档划分：

| 档 | 含义 | 测量者 |
|---|---|---|
| 档 0 | 工具前端接受 | 现有 `hax-lean` 已覆盖（cargo hax exit 0 + 产物无 term-position sorry） |
| 档 1 | 产物在 Lean 下 typecheck | 本设计的 stage 1（拷到 hax lean prelude 项目 `lake env lean`） |
| 档 2 | 产物 entry_fn `#eval` 可求值 | 本设计的 stage 2 |
| 档 3 | `#eval` 结果 == `cargo run` 结果 | 本设计的 stage 3（最终 oracle） |

**新 tool 与现有 hax-lean 并存**——不替代：

- 现有 `tools/hax-lean/` 仍跑全 corpus（146 entry），oracle 是档 0
- 新 `tools/hax-lean-eval/` 仅跑 corpus 中**标 `runnable`** 的子集（预计 10–25 个），oracle 是档 1+2+3 复合

两者在 results.json / report.md 里是**两列**独立工具，分别给出独立信号。读者可对比"档 0 SUCCESS 但档 3 FAILED"——这本身是有意义的差异信号（hax 前端接受了，但翻译语义在 Lean 跑出来与 Rust 不一致）。

### §0.2 设计层位阶（不动核心模块、不破宪法）

按 [`principles.md`](principles.md) §三模块定位：

- **核心模块 2（examples）** — 本设计要加 `runnable` 元数据 + 新增 ~15 entry，是核心模块的**自然扩展**，是 first-class 工作
- **次要模块 3（tools）** — 新增 `tools/hax-lean-eval/` 工具入口，是次要模块的**应用展示**

[`principles.md`](principles.md) §六硬指标全适用：新工具仍须满足 §六-2 不允许 partial（任何 stage failed → FAILED）、§六-1 前端测试范围（翻译类工具天然无求解层，stage 1+2 是 Lean 编译器跑 .lean，不算"工具自带求解层"）、§六-4 反作弊（stage 3 比对是新维度，不简单退化为 cargo-check）。

宪法 §四 三大派生原则全保留：

- **原则 A**：corpus `runnable` 元数据加在 `hirusttest.toml`（cargo 不读，行为字节级一致），不破"双方都不可侵入"
- **原则 B**：本设计仍是"必要条件测量"——测"hax-lean 是否能产出与 Rust 一致的语义"，**不**测语义对错本身（不证"翻译是正确的"，仅观察"当前实测一致")
- **原则 C**：异质性归配置——一致性测试逻辑封在 `tools/hax-lean-eval/wrapper.sh` + `hirusttest.toml` 新字段，runner 代码零改动

---

## §1 Corpus 适配策略

### §1.1 问题陈述

现有 142 entry / 146 个注册 entry fn 大多形态为 `pub fn xxx() {}`（零参、返回 `()`，body 内构造数据 + 调用 + `let _ =` 丢弃结果）。这种 entry **无法做一致性比对**——没值可比。即便强行 `#eval` 一个返回 `()` 的函数，得到 `RustM.ok ()` 与 Rust `cargo run` 出 stdout 任意 println 无可机械对位。

更严重的是：很多 entry 调用了 hax prelude 未实现的符号（`checked_add`、std::collections::HashMap、Vec、String、Rc/RefCell、thread::spawn 等）——这些 entry 走 stage 1 typecheck 就失败。把这类全标 runnable 会让 stage 1 漏报大幅膨胀。

**目标**：在 corpus 中**精确标注**哪些 entry 是"纯内部化、可一致性测试"——满足三条：

1. 返回非 `()` 的可比对类型（i*/u*/bool/tuple of these）
2. 实现仅用 hax prelude **已支持**的符号（arith / cmp / 控制流 / 自定义 ADT / 递归）
3. 行为是纯函数（无 IO、无 panic、无 unsafe、无 interior mutability）

### §1.2 方案 A/B/C 评估

#### 方案 A：现有 corpus 加注 — `hirusttest.toml` 加 `runnable` field

```toml
# examples/<feature>/<dir>/hirusttest.toml
entries = ["add_two", "fact"]

[runnable.add_two]
inputs = [[3, 4], [10, -7]]
expected = [7, 3]

[runnable.fact]
inputs = [[5], [6]]
expected = [120, 720]
```

- **优点**：不新增目录结构、不分裂 corpus、单一来源（一个 entry 的元数据集中在 hirusttest.toml）；现有 entry 想升级为 runnable 只加 `[runnable.<entry>]` 表；不破宪法 §四 原则 A 的"cargo 不读 hirusttest.toml"（仍是项目自有 schema）
- **缺点**：现有 145 个 hirusttest.toml 形态都是裸 `entries = [...]`，没 `[runnable.*]` 表——必须显式说明"runnable 字段缺省时该 entry 仅参与档 0 测试"；混淆点：同一 entry 同时承担"档 0 corpus member"和"档 3 corpus member"两职
- **兼容性影响**：不破——`runner/src/discover.rs:46-55` 的 `HirusttestToml` 结构用 `serde(default)` 接受未知字段（运行时验证：`HirusttestToml` 没设 `deny_unknown_fields`），多出 `[runnable.*]` 表会被 serde **忽略**。新工具 `hax-lean-eval` 用独立 parser 读 `[runnable.*]`

#### 方案 B：新增独立 corpus 子集 `examples/runnable/`

```
examples/runnable/
├── arith-add/
│   ├── Cargo.toml
│   ├── src/lib.rs       (pub fn add_two(a,b)->i32)
│   └── hirusttest.toml  (entries + inputs/expected)
├── arith-fact/
└── ...
```

- **优点**：物理隔离，runnable 子集形态一目了然；新增不污染现有 corpus 的"feature 分类"语义（`int/checked` 测 i32::checked_add 不可能 runnable，把它标 runnable 没意义但污染语义）
- **缺点**：违反 [`principles.md`](principles.md) §三-原则 B"整体多样性"——把"是否 runnable"当作 corpus 顶层 feature 分类是错的，runnable 是**正交属性**而非 feature 类目；导致重复——一个"测 fact 递归"的 entry 同时落进 `examples/recursion/fact/` 和 `examples/runnable/fact/`，违反"一 entry = 一 lib crate"

#### 方案 C：entry 命名约定 `*-runnable`

```
examples/int/add-runnable/src/lib.rs
examples/recursion/fact-runnable/src/lib.rs
```

- **优点**：零 schema 变动；纯目录命名规则就能筛选
- **缺点**：把 runnable 元数据写死在目录名，没地方放 inputs/expected（仍要 schema 加字段），所以本质退化为方案 A 但额外多了命名约束；目录名重复 feature 信息（`int/add-runnable/` 与 `int/add/` 共存时 add_two 函数体可能完全一样，违反 "一 entry 测一特性"）

### §1.3 推荐：方案 A，加少量"语义友好型"新 entry

**推荐方案 A**，理由：

1. **正交属性归字段**：runnable 是 entry 的元数据维度，正确表达是 `[runnable.<entry>]` 字段而非顶层目录
2. **不破现有 145 个 hirusttest.toml**：新字段 serde 兼容（运行时验证：`HirusttestToml` 未设 `deny_unknown_fields`，多出表被忽略）
3. **避免重复**：递归 fact 已经隐式覆盖在 `int/`、`recursion/`（如有）类目里——新增"runnable 维度"不需要物理拷贝
4. **现有 corpus 中可直接升级的 entry 有限**：粗扫 142 entry 后，绝大多数（90%+）是 `pub fn xxx() {}` 返回 `()`，**不需要**重写——它们就只参与档 0。少数有返回值的（如 `int/wrapping/`、`int-width/arith-i8/`）部分可加 `[runnable.*]` 表升级为档 3
5. **新增 ~15 个 entry**：在 `examples/<feature>/<dir>/` 下新增（**不**新建 `examples/runnable/` 顶层目录），以 feature 为单位归位——如 `examples/int/add-pure/`、`examples/recursion/fact-pure/`、`examples/enum/option-unwrap-pure/`，每个都加 `[runnable.<entry>]` 表

### §1.4 纯内部化判定准则（清单）

按 hax-lean prelude（`hax/proof-libs/lean/Hax/{core_models,rust_primitives}/`）实际支持的符号集划界。**实测验证**通过 §4.1 micro-test。

#### 安全集（已实测 / 推定可达）

| 类目 | 示例 | 状态 |
|---|---|---|
| 基础算术 `+ - * /` （i8/i16/i32/i64/u8/u16/u32/u64） | `a + b`、`n * fact(n-1)` | ✅ 实测（`custom_add`） |
| 比较 `== != < <= > >=` | `if n <= 1 { ... }` | ✅ 实测 |
| 布尔操作 `&& \|\| !` | `flag1 && flag2` | ✅ 推定（prelude `rust_primitives.ops` 有） |
| `if`-`else` 控制流 | 见 fact | ✅ 实测 |
| `match` over int / bool / 自定义 enum | `match n { 0 => ..., _ => ... }` | ✅ 推定（hax 翻译 `match` 是常规路径） |
| 自定义 `struct` + `impl` 方法（无 generic、无 lifetime） | `struct Point { x: i32, y: i32 }; impl Point { pub fn norm(&self) -> i32 { ... } }` | ⚠️ 需 micro-test |
| 自定义 `enum`（payload 是 primitive / 自定义 struct）+ pattern match | `enum E { A(i32), B(i32, i32) }` | ⚠️ 需 micro-test |
| Tuple literal、tuple destructuring | `(a, b)`、`let (x, y) = pair;` | ⚠️ 需 micro-test |
| 固定长 Array literal `[i32; N]` | `[1, 2, 3]` | ⚠️ 需 micro-test（prelude `slice` / `sequence` 有但行为需验证） |
| `wrapping_add / wrapping_sub / wrapping_mul`（u8–u64） | `a.wrapping_add(b)` | ⚠️ 推定（prelude 实测有，但 #eval 行为待验证）|
| 递归（自递归、互递归） | `fact` / 互递归 even/odd | ✅ 实测自递归（互递归需 micro-test） |

#### 禁止集

| 类目 | 原因 |
|---|---|
| `checked_add / checked_sub / overflowing_*` | prelude 缺该符号 → typecheck 阶段 unknown identifier（feasibility §3.2.2 实测 `int_checked` 触发 sorry warning）|
| `i*::MAX / i*::MIN`（除非通过 `core_models.num.Impl_2.MAX` 等已支持的）| 实测 `int_checked` 编译报 unexpected token，是 hax printer bug 同时也是 prelude 边界 |
| `Vec<T> / VecDeque / HashMap / BTreeMap / HashSet / String` | prelude `core_models.alloc` / `core_models.std.collections` 完整度未验证，feasibility §6.2 列为"不适合" |
| `Box<T> / Rc<T> / Arc<T> / RefCell / Cell` | 同上，且引入 monadic effect 让 `#eval` 不确定 |
| `&mut T` 作为 entry 参数（entry 必须零参，所以这条更指 entry body 内的 mut ref） | hax limit (`hax-limit/` 类目说明) |
| `unsafe` block / raw pointer | hax 直接 reject |
| `panic! / unwrap() / expect() / assert!` | panic 会让 `cargo run` exit ≠ 0，没法和 `#eval` 字面比 |
| `println! / eprintln! / std::io::*` | IO 不可比 |
| `thread::spawn / async / await` | 同上 + hax 不支持 |
| 浮点 `f32 / f64`（除非显式建模为 epsilon 比对，初版不支持） | Rust `Debug` 与 Lean `#eval` 输出格式不一致 |
| `Trait` 多态分派（`Box<dyn Trait>` / generic fn with bound） | hax-lean 翻译形态尚未稳定，先不纳入 |

#### 灰色集（v1 不纳入，未来按 micro-test 结果分流）

| 类目 | 处理 |
|---|---|
| Closure (Fn / FnMut / FnOnce) | v1 不纳入；feasibility 未实测 |
| Generic fn `fn id<T>(x: T) -> T` | v1 不纳入；hax 处理 generic 形态复杂 |
| iterator methods（`map / filter / sum` 等）| v1 不纳入；需要 prelude `core_models.iter` 完整度验证 |

### §1.5 推荐 entry 设计目标（v1 实施时约 12–20 个）

按"覆盖纯内部化判定准则的代表性"原则：

| Entry id（建议） | 测的特性 |
|---|---|
| `int/add-pure/add_two` | i32 加法 + 多组输入 |
| `int/add-pure/add_u32` | u32 加法 |
| `int/mul-pure/mul_signed` | i32 乘法（含负数） |
| `int/sub-pure/sub_underflow` | i32 减法不下溢的边界 |
| `recursion/fact-pure/fact` | 自递归 + match-like if-else |
| `recursion/fib-pure/fib` | 自递归 + 加法 |
| `recursion/gcd-pure/gcd` | 自递归 + 余数 |
| `match/int-match-pure/classify` | match over i32 → i32 |
| `match/bool-match-pure/bnot` | bool match |
| `enum/option-pure/unwrap_or` | Option<i32> + match |
| `enum/option-pure/map_inc` | Option<i32> map 模拟 |
| `enum/custom-pure/area` | 自定义 enum + match → i32（去掉 feasibility 的 f64 版） |
| `struct/point-pure/manhattan` | struct + impl method → i32 |
| `tuple/swap-pure/swap_i32` | tuple destructuring + 返回 tuple |
| `wrapping/u8-wrap/wrap_add` | wrapping_add u8（验证边界包绕语义一致） |

每条都满足：返回类型 `i*/u*/bool`（或 tuple of these），参数 1–3 个，body 内无禁止集符号。Micro-test 前先各自 hax-lean 跑通档 1+2，再正式入 corpus。

### §1.6 现有 146 entry 中 runnable 估计

粗扫现有 entry 形态后估计：

- **明确不可 runnable**：约 130+ entry（返回 `()`、调 std collections、调 panic/unwrap、走 unsafe、走 trait obj、调 IO、属 `*-limit/` 类目）
- **可能可 runnable**：约 5–15 entry（如 `int/wrapping/int_wrapping` 改造、`int-width/arith-i8/*` 部分、少数 `int-width/checked-all-widths` 的 wrapping 分支）
- **直接可升级（不改 src/）**：约 0–3 entry（feasibility 实测中 hello/int_checked/int_wrapping 都返回 `()`，不可比，需改 src/。`int_wrapping` 若改成返回 `u8` 则不是单字段升级而是改 entry，违反原则 A "源码字面零修改"——因此只能新增 entry，不能"升级"现有）

**结论**：现有 corpus 几乎无可"无修改升级"的 runnable entry。**v1 实施需新增 ~15 个**——这是核心模块 2（examples）的扩展工作。

---

## §2 hirusttest.toml schema 扩展

### §2.1 字段设计

按 §1.3 推荐方案 A，扩展点为单文件轨 `hirusttest.toml`（目录轨同 schema，仅文件位置不同）：

```toml
# examples/<feature>/<dir>/hirusttest.toml

entries = ["add_two", "fact"]
target_path = "."                    # 现有字段，保持

# === 新增：[runnable.<entry_fn>] 表 ===

[runnable.add_two]
inputs = [[3, 4], [10, -7]]          # 必填：多组测试输入
expected = [7, 3]                     # 必填：对应预期输出（与 inputs 同长度）
input_types = ["i32", "i32"]          # 可选：参数类型注解（默认空，从 input toml 类型推断）
return_type = "i32"                   # 可选：返回类型（默认 "i32"）
rust_main_override = ""               # 可选：自定义 Rust 侧 main 模板（默认按 inputs 渲染 println!）
lean_eval_override = ""               # 可选：自定义 Lean 侧 #eval 表达式模板
compare_mode = "exact"                # 可选："exact" | "epsilon" | "structural"，默认 "exact"
compare_epsilon = 0.0                 # 可选：浮点 epsilon（compare_mode = "epsilon" 时用）

[runnable.fact]
inputs = [[5], [6]]
expected = [120, 720]
```

#### 字段语义

| 字段 | 类型 | 必填 | 含义 |
|---|---|---|---|
| `inputs` | Array of Arrays | 是 | 每个内 Array 是一组实参；inputs.len() = 测试组数 |
| `expected` | Array | 是 | 与 inputs 同长；每元素是该组对应的预期返回值 |
| `input_types` | Array of String | 否 | 参数 Rust 类型名（如 `["i32", "i32"]`）；用于 v2 严格类型断言或代码生成 |
| `return_type` | String | 否 | 返回 Rust 类型；默认 `"i32"`；用于 Lean `#eval` 输出 unwrap 策略选择 |
| `rust_main_override` | String | 否 | 整段 Rust `fn main()` body 字符串；非空时**完全替代**默认渲染（默认逐行 `println!("{}", entry_fn(input_args))`） |
| `lean_eval_override` | String | 否 | 整段 `#eval ...` 行（多行用 `\n`）；非空时**完全替代**默认 `#eval <crate>.<entry> ${inputs}` 渲染 |
| `compare_mode` | Enum String | 否 | `"exact"`（默认，字面相等）/ `"epsilon"`（浮点）/ `"structural"`（reserved） |
| `compare_epsilon` | Number | 否 | epsilon 模式的容差；其他模式忽略 |

#### v1 范围裁剪

按 [`principles.md`](principles.md) §七 Occam 砍项 + [`feedback_no_premature_extensibility.md`](https://) 不预设可扩展性：v1 只实现 `inputs / expected / return_type / compare_mode = "exact"`。其余字段为预留 schema 占位，v1 不读取（按 hirusttest.toml schema 兼容性原则——多字段 serde 忽略，不破坏）。

### §2.2 多组 input → 多组比对的语义

一条 `[runnable.<entry>]` 实例 = **一次 task**（runner 角度）。task 内部跑全部 inputs；任一组 input 的 Lean / Rust 输出不一致 → 整条 task FAILED。**单个 entry 不细分多 task**（避免破坏 [`detailed-design.md`](detailed-design.md) §二 ID 唯一性约定）。

raw 输出存全量：`raw/hax-lean-eval/<slug>.stdout` 含每组输入的 Rust 端 stdout + Lean 端 stdout，便于 post-mortem。

### §2.3 类型 mapping（Rust → Lean）

v1 仅支持以下类型 mapping（按 hax 翻译产物形态）：

| Rust 类型 | Lean 类型（hax 翻译后） | `#eval` 输出形态 | Unwrap 策略 |
|---|---|---|---|
| `i8/i16/i32/i64/i128` | `i8 / i16 / i32 / i64 / i128`（hax prelude 类型） | `RustM.ok <n>` | 字面提取 `RustM.ok (\d+)` → `<n>` |
| `u8/u16/u32/u64/u128` | 同上 u-version | 同上 | 同上 |
| `bool` | `Bool` | `RustM.ok true` / `RustM.ok false` | regex 提取 `RustM.ok (true|false)` |
| `(i32, i32)` 等 tuple | hax tuple struct | `RustM.ok ⟨3, 4⟩` 或 `RustM.ok (3, 4)`（待 micro-test） | structural normalize（v1 不支持，归为灰色集） |
| `()` | `rust_primitives.hax.Tuple0` | `RustM.ok {  }` | 不允许（无返回值的 entry 不应 runnable） |

**关键风险**：feasibility §5 实测 `RustM.ok 7` 是确实出现的形态，但 hax 上游产物格式可能演化（PR #1672 之类）——unwrap regex 需在 §4 oracle 设计中显式 micro-test 双向验证。

### §2.4 默认 Rust main / Lean #eval 渲染规则

默认情况（无 override）：

#### Rust side（runner 在隔离副本生成 `src/main.rs`）

```rust
// 渲染模板（伪 tera）
use {{ target_crate_name }}::*;
fn main() {
{% for input in inputs %}
    println!("{}", {{ entry_fn }}({{ input | join(sep=", ") }}));
{% endfor %}
}
```

执行：`cargo run --quiet` → stdout 每行一个返回值字面（`Display` trait 输出）。

#### Lean side（runner 拷产物到 hax prelude 项目，生成 `Test_<slug>.lean`）

```lean
import Hax
import Std.Tactic.Do
import Std.Do.Triple
import Std.Tactic.Do.Syntax
open Std.Do
open Std.Tactic

set_option mvcgen.warning false
set_option linter.unusedVariables false

-- 把 hax 翻译产物 append 进来
{{ translated_lean_content }}

-- runner 生成的 #eval block
{% for input in inputs %}
#eval {{ target_crate_name }}.{{ entry_fn }} {{ input | join(sep=" ") }}
{% endfor %}
```

执行：`lake env lean Test_<slug>.lean` → stdout 每个 `#eval` 一行 `RustM.ok <value>`。

#### 比对

```
rust_lines     = cargo run stdout 按行拆
lean_lines     = lake env lean stdout 按行拆，每行 unwrap_rust_m_ok(.) → value_str
expected_lines = expected toml field 渲染为字符串

assert len(rust_lines) == len(lean_lines) == len(expected_lines)
for r, l, e in zip(rust_lines, lean_lines, expected_lines):
    if compare_mode == "exact":
        assert r == l == e
    elif compare_mode == "epsilon":
        assert |float(r) - float(l)| < eps and |float(r) - float(e)| < eps
```

任一断言失败 → FAILED。全过 → SUCCESS。

---

## §3 新工具入口 `tools/hax-lean-eval/`

### §3.1 设计：四 stage pipeline 封在单 wrapper（不改 runner）

按 [`architecture.md`](architecture.md) §三模块切分 + [`detailed-design.md`](detailed-design.md) §五.charon 的 `sh -c` 单行 wrapper 范式——**整个 pipeline 封在 wrapper.sh 内**，runner 只看到一个 tool 进程的 exit code。

```
tools/hax-lean-eval/
├── tool.toml
├── harness.rs.tera            # 复用 hax-lean 的 bin harness（哑实现，hax 用 --lib 跳过）
├── wrapper.sh                 # 四 stage pipeline 主驱动
├── render_rust_main.sh        # 子步骤：渲染 src/main.rs（按 [runnable.<entry>]）
├── render_lean_eval.sh        # 子步骤：渲染 Test_<slug>.lean
├── compare.py                 # 子步骤：unwrap RustM.ok + 比对（Python 易写 + macOS 自带）
└── README.md                  # 按 tool-integration.md §五 章节清单
```

### §3.2 wrapper.sh 四 stage 逻辑

```bash
#!/usr/bin/env bash
set -uo pipefail
# args: $1 = entry_fn （runner 通过 TS_ENTRY_FN 注入）
# env:  TS_HIRUSTTEST_TOML, TS_ENTRY_FN, TS_TARGET_CRATE, TS_HAX_LEAN_PRELUDE_DIR, TS_HAX_ENGINE_BIN

ENTRY=$TS_ENTRY_FN
CRATE=$TS_TARGET_CRATE

# === Stage 0: 解析 [runnable.<entry>] 表 ===
# 失败：runnable 表缺失 → 该 entry 不应被 hax-lean-eval 跑，记 FAILED + reason "not_runnable"
# (因 runner 不感知 runnable 标记，会把所有 entry 都丢给 hax-lean-eval；wrapper 内自筛)

if ! python3 -c "import tomllib; ..." > /tmp/runnable.json; then
  echo "[hax-lean-eval] FAIL: entry not marked [runnable.$ENTRY] in hirusttest.toml" >&2
  exit 1
fi

# === Stage 1: cargo hax into lean (复用现有 hax-lean 逻辑) ===
env HAX_ENGINE_BINARY=$TS_HAX_ENGINE_BIN cargo +nightly-2025-11-08 hax -C --lib ';' into lean
RC=$?
if [ $RC -ne 0 ]; then
  echo "[hax-lean-eval] FAIL: stage 1 cargo hax exit $RC" >&2
  exit 1
fi
# 复用 hax-lean oracle 的 silent partial 检测
if find proofs/lean/extraction -name '*.lean' -exec cat {} + 2>/dev/null \
   | awk '{ sub(/--.*/, ""); print }' \
   | grep -qE '(:=|pure|mk|,)[[:space:]]*sorry\b|\bsorry[[:space:]]*[,)\]]'; then
  echo "[hax-lean-eval] FAIL: stage 1 silent partial (sorry in term position)" >&2
  exit 1
fi

# === Stage 2: 在 hax prelude 项目里 lake env lean (typecheck + #eval) ===
PRELUDE=$TS_HAX_LEAN_PRELUDE_DIR
SLUG=$(echo "$CRATE-$ENTRY" | tr / _)
TEST_LEAN="$PRELUDE/Test_${SLUG}.lean"
bash render_lean_eval.sh "$CRATE" "$ENTRY" "$PRELUDE" "$TEST_LEAN" /tmp/runnable.json
(cd "$PRELUDE" && lake env lean "Test_${SLUG}.lean") > /tmp/lean.stdout 2> /tmp/lean.stderr
RC=$?
if [ $RC -ne 0 ]; then
  echo "[hax-lean-eval] FAIL: stage 2 lake env lean exit $RC" >&2
  cat /tmp/lean.stderr >&2
  exit 1
fi
# 关键反误报：lake exit 0 不蕴含产物无 sorry，要 grep stderr 是否含 sorry warning / error
if grep -qE 'declaration uses .sorry.|error: unexpected token' /tmp/lean.stderr; then
  echo "[hax-lean-eval] FAIL: stage 2 typecheck pass but uses sorry / printer bug" >&2
  exit 1
fi

# === Stage 3: 渲染 Rust main + cargo run ===
bash render_rust_main.sh "$CRATE" "$ENTRY" /tmp/runnable.json > src/main.rs
cargo run --quiet > /tmp/rust.stdout 2> /tmp/rust.stderr
RC=$?
if [ $RC -ne 0 ]; then
  echo "[hax-lean-eval] FAIL: stage 3 cargo run exit $RC (rust side panic / type error)" >&2
  cat /tmp/rust.stderr >&2
  exit 1
fi

# === Stage 4: compare (unwrap RustM.ok + diff) ===
python3 compare.py /tmp/runnable.json /tmp/rust.stdout /tmp/lean.stdout
exit $?
```

### §3.3 各 stage 失败的 oracle 语义（按 [`tool-integration.md`](tool-integration.md) §二 形式指标）

| Stage | 失败信号 | runner 看到 |
|---|---|---|
| Stage 0 (parse runnable) | wrapper exit 1 + stderr `not marked [runnable.<entry>]` | FAILED |
| Stage 1.a (cargo hax exit ≠ 0) | wrapper exit 1 + 复用 hax-lean 的诊断 | FAILED |
| Stage 1.b (silent sorry) | wrapper exit 1 + grep 命中 | FAILED |
| Stage 2.a (lake env lean exit ≠ 0) | wrapper exit 1 + 抓 lean stderr | FAILED |
| Stage 2.b (lake exit 0 但用 sorry / printer bug) | wrapper exit 1 + grep 命中 `declaration uses 'sorry'` 或 `unexpected token` | FAILED |
| Stage 3 (cargo run exit ≠ 0) | wrapper exit 1 | FAILED |
| Stage 4 (output mismatch / unwrap failure) | wrapper exit 1 + diff dump | FAILED |
| 全过 | wrapper exit 0 | SUCCESS |

按宪法 §六-2 不允许 partial：**任一 stage 失败 = FAILED**。runner 不区分 stage——只看 exit code（这维持核心模块 1 简单）。**stage 区分通过 stderr 字面**记录到 raw 文件，cc-report 时人读 stderr 归类。

### §3.4 tool.toml 设计

```toml
# tools/hax-lean-eval/tool.toml
command = [
  "sh", "-c",
  "bash ${TS_PROJECT_ROOT}/tools/hax-lean-eval/wrapper.sh"
]
timeout_secs = 900            # stage 2 lake build 慢；预留 15 min
entry_mode   = "bin"          # 与 hax-lean 一致；harness 是 bin，hax 用 --lib 跳过

# 新工具自身没专门的 version 命令；汇总 hax + lean + lake 三者版本
version_command = [
  "sh", "-c",
  "echo hax=$(cargo +nightly-2025-11-08 hax --version 2>&1 | head -1); echo lean=$(lean --version 2>&1 | head -1); echo lake=$(lake --version 2>&1 | head -1)"
]
```

### §3.5 .env 新增

```
# ─── hax-lean-eval ───────────────────────────────────────────────────────────
# hax lean prelude 项目路径（包含 lakefile.toml + Hax.lean）
# 上游：hacspec/hax tree path proof-libs/lean
# 测试基线：commit 30949eb87058895c24f963df90dd30ef11b0dc1a，lean 4.29.0-rc1
TS_HAX_LEAN_PRELUDE_DIR=/private/tmp/hax/hax-lib/proof-libs/lean
```

runner 的 `${VAR}` 展开机制（见 [`detailed-design.md`](detailed-design.md) §一 command 字段说明 + `.env.example`）已支持。

### §3.6 Tool 与 corpus 的耦合处理

runner 不感知 `[runnable.*]`——它把所有 146 entry 都丢给 hax-lean-eval。**自筛逻辑在 wrapper 内**：未标 runnable 的 entry stage 0 直接 FAILED with reason "not_runnable"。

这会让 report.md 出现大量 `not_runnable` 的 FAILED——**这是有意为之**的诚实表态：hax-lean-eval 在这些 entry 上"没有能给出判断"，按宪法 §三-3-2.a 形式指标精神，FAILED 是真实信号。

替代方案（被排除）：让 runner 支持 per-tool 的 entry filter——破宪法 §四-A∩B "能力靠观测，不靠声明"。runner 不能预跳过任何 (tool, entry) 组合。

---

## §4 Oracle 反误报论证（按 [`tool-integration.md`](tool-integration.md) §四）

按 §三的 oracle 形式指标精神，逐 stage 提供"防漏报 + 反误报"论证。

### §4.1 Stage 1 — 复用 hax-lean 现有论证

stage 1 oracle 与现有 `tools/hax-lean/tool.toml` 完全一致（cargo hax exit 0 + 产物无 term-position sorry）。**复用** [`tools/hax-lean/README.md`](../../tools/hax-lean/README.md) "形式严格性 — 0 误报 / 0 漏报"段落的实测论证，不重复。**信心等级**：⚠️ 实测验证 0 误报 / 0 漏报，不可形式证明（同 hax-lean）。

### §4.2 Stage 2 — lake env lean

**防漏报**：

- 已知不一致场景：产物用 `sorry` 占位（prelude 缺符号） → `lake env lean` 仍 exit 0（Lean 接受 sorry term）但 stderr 含 `declaration uses 'sorry'` → grep 命中
- hax printer bug（feasibility §3.2.2 实测 `int_checked` 触发 `unexpected token '←'`） → stderr 含 `error: unexpected token` → grep 命中
- 真正的 typecheck 失败（类型不匹配等） → exit ≠ 0 → 直接 FAILED

**反误报**：合法 SUCCESS 路径——`add_two`、`fact` 都是 prelude 完全支持的符号 → lake 0 + stderr 无 warning → 不命中 FAILED。**micro-test**（v1 实施时跑）：

| Micro-test entry | 预期 stage 2 信号 |
|---|---|
| `int/add-pure/add_two` | SUCCESS（lake 0 + stderr 无 sorry/error） |
| `int/checked-broken/int_checked`（故意构造调 checked_add） | FAILED（stderr 含 `declaration uses 'sorry'` 或 `unknown identifier`） |
| `int/printer-bug/int_checked_form`（实测 feasibility 的 `let _ : Option ←` 形态） | FAILED（stderr 含 `unexpected token`） |
| `enum/option-pure/unwrap_or` | 待 micro-test（v1 验证 prelude 对 Option 的支持） |

每个 micro-test 都先单独跑确认双向有效后才入 corpus。**信心等级**：⚠️ 实测验证（待 v1 跑 micro-test）；不可形式证明（Lean 错误信号枚举无穷开放）。

### §4.3 Stage 3 — cargo run

**防漏报**：

- panic / unwrap on None / index out of bounds → Rust runtime exit ≠ 0 → FAILED
- 编译错（不太可能，因为产物 src/lib.rs 已通过 hax 的 cargo check 隐式） → cargo run exit ≠ 0 → FAILED

**反误报**：合法 SUCCESS 路径——纯函数 + 小输入 + 已类型检查 → cargo run exit 0 + stdout 含返回值字面 → 不误报。**信心等级**：✅ 高（cargo run 是 stock Rust 工具，exit 0 蕴含进程正常退出，无 silent partial 可能）。

### §4.4 Stage 4 — compare（最关键，最易误报）

**输出 normalize 规则**（risk 集中点）：

#### Rust 端

`Display` trait 输出。对 i*/u*/bool 简单：`3`、`-7`、`120`、`true`。

⚠️ 风险：Rust `println!` 的 trailing newline。`compare.py` 须 `.strip()` 后比对。

#### Lean 端

`#eval` 输出格式：feasibility §4.1 实测得 `RustM.ok <value>` 一行。

⚠️ 风险点 1：**hax 上游格式演化**。若未来 `#eval` 输出变为 `Except.ok <value>`、`{ ok := <value> }`、`<value>` 裸值——unwrap regex 失效。**应对**：v1 用宽松 regex `(?:RustM\.ok|Except\.ok|ok)\s+([\-\d]+|true|false)`，并把"unwrap failure"作为 stage 4 sub-failure 显式记 stderr，便于人工诊断；这条本质是**已知漏报盲点**，必须在 README 显式声明。

⚠️ 风险点 2：**`#eval` 多行输出**（如 tuple 展开、负数括号包裹）。v1 不支持 tuple，先回避；负数实测形态是 `RustM.ok -7`（feasibility 验证），不是 `RustM.ok (-7)`——但 lean 端入参时 `#eval add_two 10 (-7)`，输出 `RustM.ok 3` 反而稳定。这是 lean #eval 的左结合 application + 字面表达式的不对称——**正式入 corpus 前每个 entry 单跑确认**。

⚠️ 风险点 3：**字段顺序 / struct printing**。Lean 4 `#eval` 对 struct/inductive 默认用 `mk` 形态打印（如 `Point.mk 3 4`），与 Rust `Debug` 的 `Point { x: 3, y: 4 }` 不同。v1 **不支持** struct 比对（compare_mode = "exact" 不能跨这层差异）—— `struct/point-pure/manhattan` 的实际做法是返回 `i32`（`norm` 是 i32→i32），而非返回 struct 本身。

⚠️ 风险点 4：**浮点比较**。`compare_mode = "epsilon"` 在 v1 不实施。所有候选 entry 强制 i*/u*/bool/tuple-of-these。

**防漏报**：输出不一致（语义错误） → diff 必命中 → FAILED。

**反误报**：

- 必须确保 normalize 规则**不引入错误对位**——如把 Rust 的 `3` 与 Lean 的 `RustM.ok 3` 错配 normalize 成 `3` vs `3` 是**正确**对位（这是设计目的）；但若把 Lean 输出错读为 `3, 4` 但实际是 `30, 4`（行边界误判），就是误报
- **micro-test**：实施时每个 candidate entry 跑前先 `compare.py /tmp/rust.stdout /tmp/lean.stdout` 单独验证，确认人工眼读结果与 compare.py 输出一致才入 corpus

**信心等级**：⚠️ 实测验证（micro-test 范围内）；不可形式证明（Lean 输出格式开放、上游可演化）。

### §4.5 信心等级总表

| Stage | 防漏报方式 | 反误报方式 | 形式严格性 |
|---|---|---|---|
| 1 (hax → lean) | 复用 hax-lean | 复用 hax-lean | ⚠️ 实测，不可形式 |
| 2 (lake env lean) | exit ≠ 0 + stderr grep `declaration uses sorry` / `unexpected token` | micro-test 双向 | ⚠️ 实测，不可形式 |
| 3 (cargo run) | exit ≠ 0 | stock Rust 工具行为已知 | ✅ 高 |
| 4 (compare) | diff | normalize 规则 + micro-test 双向 | ⚠️ 实测，不可形式（最弱一环） |

**已知漏报盲点（必须声明在 README）**：

- hax 输出 `#eval` 格式上游演化 → unwrap regex 失效；当前 unwrap 用宽松 regex 兜底
- Lean prelude 对 v1 未覆盖类型（tuple printing / struct / 浮点）的输出格式
- 极端 input space 未覆盖（hax-lean-eval corpus 是离散小样本，不做 fuzzing / property-based 测试）—— 这条**不算漏报**，按 [`principles.md`](principles.md) §三-原则 B 是"必要条件测量"的本性，但读者引用支持率时应知道

---

## §5 Runner 是否需要扩展？

### §5.1 两个选项

**选项 a：multi-stage 封 wrapper（推荐）**

- runner 看到一个 tool 进程 + 一个 exit code
- 所有 stage 区分由 wrapper 在 stderr 写诊断字面，cc-report 时归类
- runner 代码**零改动**

**选项 b：runner 扩展支持 pipeline tool**

- runner 引入 `[tool.stages]` 配置，每 stage 各自 oracle，runner 串起来跑
- raw 输出按 stage 拆分目录
- runner 代码大改 + 增加非必需复杂度

### §5.2 推荐选项 a，理由

按 [`principles.md`](principles.md) §四原则 C "异质性归配置，框架代码同质"：**runner 代码不为某工具开后门**。多 stage pipeline 是 hax-lean-eval 自身的复杂度，应该封在工具适配层，而非污染 runner 代码。

按 [`feedback_no_premature_extensibility.md`](https://) 不预设可扩展性：只有这一个工具需要多 stage（hax-coq / rocq-of-rust 中期解阻才可能各自需要类似机制），而它们各自的 pipeline 形态不一样（hax-coq 是 `coqc` + `Compute`、rocq-of-rust 是写 M monad evaluator 求值），抽象成统一 stage 接口现在没收益。

按 [`detailed-design.md`](detailed-design.md) §五 charon / prusti / creusot 都用 `sh -c` 单命令封多步——这是现有项目的成熟做法，保持一致。

stage 区分通过 stderr 字面（如 `[hax-lean-eval] FAIL: stage N ...`）记到 raw 文件——cc-report 时人读 stderr 做归类，与现有 hax-lean 的 silent partial 诊断处理方式一致。

### §5.3 选项 a 的代价

- runner 看不到"stage 1 SUCCESS、stage 2 FAILED" 这种细粒度信号——但**这不重要**：宪法 §六-2 不允许 partial，任一 stage failed = 工具未完整完成 = FAILED。细粒度仅服务 cc-report 归类，可从 raw stderr 复原
- wrapper.sh 内部复杂度上升，需测试覆盖——这是次要模块工作，可承担

---

## §6 工作量估计

按 [`principles.md`](principles.md) §三-核心模块 vs 次要模块优先级 + [`tool-integration.md`](tool-integration.md) §六禁忌的"实测报告非长期承诺"精神，工作量分核心 / 次要双轨估计。

### §6.1 任务分解

| 工作项 | 模块归属 | 估时（小时） |
|---|---|---|
| corpus 新增 ~15 个 runnable entry（src/lib.rs + Cargo.toml + hirusttest.toml） | 核心模块 2 | 8–12（每个 ~30 min，含 hax-lean 跑通确认） |
| hirusttest.toml schema 扩展（design 层已完成，实施需 examples 写入） | 核心模块 2 | 与上一项合并 |
| `tools/hax-lean-eval/` 工具入口（tool.toml + wrapper.sh + render_*.sh + compare.py + README.md） | 次要模块 3 | 10–14 |
| Stage 2 反误报 micro-test 套件（构造 3–5 个故意 broken entry 验证 oracle 命中） | 次要模块 3 | 4–6 |
| Stage 4 normalize 规则的 unwrap regex 反误报 micro-test | 次要模块 3 | 2–4 |
| 全 corpus 跑一次 hax-lean-eval + 写测试报告（按 [`tool-integration.md`](tool-integration.md) §七实测报告原则） | 次要模块 3 | 4–8 |
| docs 更新（README.md + 工具自身 README + 一致性测试在项目层的位置阐释） | 文档 | 4 |
| **总和** | | **32–48 小时（约 4–6 个工作日）** |

### §6.2 阶段化建议

按 [`workflow.md`](https://) 三阶段精神，先核心后次要：

1. **阶段 1：corpus 扩展 + schema 落地**（核心模块 2，~12h）
   - 设计 ~15 entry 的 src/lib.rs + hirusttest.toml
   - 手工 hax-lean 跑通每个 entry 确认产物档 1 OK
   - 此阶段产出**先于** tool 实施——按 testsuite 精神，corpus 是 first-class，先有 corpus 再有工具

2. **阶段 2：tool 入口实施**（次要模块 3，~20h）
   - wrapper.sh + render_*.sh + compare.py
   - micro-test 套件验证 oracle 双向有效
   - 全 corpus 跑一次出 results.json

3. **阶段 3：测试报告**（次要模块 3，~12h）
   - 按 [`tool-integration.md`](tool-integration.md) §七 写 cc-report
   - 不在本设计稿范围

阶段 1 完成后**可以独立交付**——即便 tool 不做，corpus 中的 runnable entry 仍提供"语义友好型"测试材料，给现有 hax-lean / charon / kani / verus 等也用得上（kani 可对 `add_two(3, 4) == 7` 写 `#[kani::proof]` 断言）。

---

## §7 与现有体系的关系

### §7.1 现有 19 工具的 oracle / cc-report 不受影响

按 §3.6，runner 把所有 entry 都丢给所有工具——hax-lean-eval 是**新增**第 20 个工具列，不替代既有任何工具。已有 19 工具的 tool.toml / oracle 完全不动；其 cc-report 也不动（仍按既有 corpus 跑）。

### §7.2 文档层级别更新建议

#### 必须更新

- **`README.md`**（主入口）：在 19 工具列表后追加第 20 列 hax-lean-eval；在"测试覆盖维度"段加一节"档 3 一致性测试维度（仅 hax-lean）"
- **`tools/hax-lean-eval/README.md`**（新建）：按 [`tool-integration.md`](tool-integration.md) §五章节清单写——简介 / 前端接受定义 / SUCCESS 信号 / 形式严格性 / 安装 / 本框架配置 / 已知限制 / 关联 sub-tests

#### 不必更新

- **`docs/design/principles.md`** — 不动。本设计在 §0.2 已论证一致性测试**没新增宪法级原则**，仍是"必要条件测量 + 异质性归配置"，原则 A/B/C 全保留
- **`docs/design/architecture.md`** — 不动。本设计选项 a 没扩展 runner，没改模块切分
- **`docs/design/detailed-design.md`** — 不动。schema 扩展是向后兼容的字段追加，serde 默认接受，不改函数级前后置
- **`docs/design/tool-integration.md`** — 不动。本设计是"使用现有 tool-integration 框架的另一个工具"，方法学全继承

#### 可选更新（讨论后决定）

- 若希望把"档 3 一致性测试"作为一个新的可信度维度纳入项目长期叙事，可在 [`principles.md`](principles.md) §一根本问题意识下追加一段"覆盖度的多维性——除前端接受外，对翻译类工具可观察的'语义一致性'是另一独立维度"——但这**必须**先与用户讨论是否值得修宪（按 CLAUDE.md §1.1 宪法级修订规则）。**默认建议不修宪**：把档 3 视为框架的应用展示，仍属次要模块 3 范围

### §7.3 模块优先级影响

按 [`principles.md`](principles.md) §三 三模块定位：

- **核心模块 2（examples）扩展**：新增 ~15 runnable entry 是 first-class 工作——是"原则 B 整体追求多样性"的自然延伸（从"单特性 entry"梯队扩展到"语义可比对的纯函数 entry"子梯队）
- **次要模块 3（tools）扩展**：`tools/hax-lean-eval/` 是次要工作，不抢核心优先级——按 [`feedback_no_premature_extensibility.md`](https://) 不预设全自动框架，若 corpus 阶段后用户判断"hax-lean-eval 工具入口暂不值得做"，corpus 仍交付，工具留作半自动手工跑

### §7.4 与 P13-A / P15 工作的边界

按用户硬约束：

- **不动 tools/{kani, hax-fstar, hax-coq}**（P13-A 工作中）—— hax-coq 即便未来一致性测试可达也不在本设计稿范围
- **不动 tools/rocq-of-rust 或新 ror-eval 入口**（P15 工作中）—— rocq-of-rust 在本机不可达（feasibility §6.1），即便未来可达也单独设计，不在本稿

本设计稿仅覆盖 `tools/hax-lean-eval/`（新建），与上述并行工作物理隔离。

---

## §8 设计层风险与未决事项

### §8.1 已识别风险（已在前文给出对策）

| 风险 | 位置 | 对策 |
|---|---|---|
| `#eval` 输出格式上游演化 | §4.4 风险点 1 | 宽松 unwrap regex + README 显式声明盲点 |
| hax printer bug 对带类型注解 `let ... ←` 形态 | §4.2 / feasibility §3.2.2 | stderr 含 `unexpected token` → FAILED（已实测有效） |
| corpus 中 runnable entry 数量稀少 | §1.6 | 新增 ~15 entry；不依赖现有 corpus 的"原样升级" |
| Lean prelude 演化引入新 sorry path | §4.1 同 hax-lean | 复用 hax-lean 既有漏报盲点声明 |

### §8.2 未决事项（设计层硬伤候选）

经全面推导，**未发现设计层硬伤**。具体说：

- `#eval` 输出格式**不是**硬伤——经 feasibility §5 实测，`RustM.ok <int>` 与 `RustM.ok <bool>` 格式稳定，正则化可机械实施；上游演化是已知盲点而非硬伤
- "纯内部化判定"**不是**硬伤——准则可枚举（§1.4 清单），实施时按清单筛
- runner 不感知 runnable 标记**不是**硬伤——选项 a 的自筛逻辑在 wrapper 内，符合宪法原则 C
- tuple / struct 输出对位**是 v1 不支持的范围**而非硬伤——v1 强制 entry 返回 i*/u*/bool（不含 tuple），即可绕过

唯一**软约束**：本设计强依赖 hax-lean 上游产物 `#eval` 行为格式的相对稳定。若 hax 上游短期内大改产物形态，本设计需要 stage 4 normalize 规则跟进。但这与现有 hax-lean 工具的 grep 模式同样依赖上游不变，是同类风险，可承担。

### §8.3 实施前需用户确认的设计抉择

- **§1.3 推荐方案 A（schema 加注）**：用户是否同意，而非方案 B/C？
- **§3.5 .env 新增 `TS_HAX_LEAN_PRELUDE_DIR`**：是否需要更细的 prelude 版本锁定（如自动 git clone 到固定路径）？
- **§7.2 是否修宪**：是否要把"档 3 一致性测试"提升为宪法层新维度？默认建议不修
- **§6.2 阶段化交付**：用户是否同意分阶段（先 corpus / 后 tool）？

---

## §9 决策溯源表（按 [`principles.md`](principles.md) §七 Occam）

| 决策 | 推自 | 备选 | 选择理由 |
|---|---|---|---|
| schema 加 `[runnable.<entry>]` 表（方案 A） | 原则 A 双轨 + Occam | B 顶层 runnable/ 目录 / C 命名约定 | 不破现有 corpus、不破 cargo 不可见性、字段级正交 |
| multi-stage 封 wrapper.sh（选项 a） | 原则 C 异质性归配置 | b runner 扩展支持 pipeline | runner 简单优先；与现有 charon / prusti 等成熟做法一致 |
| `[runnable.*]` 缺省时 stage 0 FAILED（不预跳过） | 原则 A ∩ B "能力靠观测" | runner 加 entry filter | 不破宪法投影 |
| v1 仅支持 i*/u*/bool 返回类型 | Occam + 不预设可扩展性 | v1 全类型 mapping | tuple / struct / float 引入 normalize 复杂度，先验证最简形态 |
| v1 不实施 epsilon 浮点比对 | Occam | 全比对模式 | corpus 候选 entry 无浮点必要 |
| 工具命名 `hax-lean-eval`（不叫 `hax-lean-consistency`） | 简短 | `hax-lean-consistency` / `hax-lean-runtime` | "eval" 字面对应 Lean `#eval`，更短，与 cargo run output 对位 |
| 用 Python 写 compare.py（不用 shell awk） | 实用 | 纯 shell | macOS / Linux 都自带 python3，正则比 shell 易写易测 |
| stage 区分通过 stderr 字面而非 exit code 编码 | runner 不解读子进程内部 | 多级 exit code | 与现有 charon `[hax-lean-oracle] FAIL: ...` 一致 |
| 实施分阶段（corpus 优先） | [`principles.md`](principles.md) §三模块优先级 | 一并交付 | 核心模块 2 是 first-class，独立可交付 |
| 不修宪法 / 不修 architecture / 不修 detailed-design | CLAUDE.md §1.1-§1.3 | 加新宪法条 | 本设计完全在现有原则空间内 |

---

## §10 附录：与 feasibility 报告的对应

feasibility 报告 §7 已给出"框架化设计建议雏形"——本设计稿是其完整展开。映射关系：

| feasibility §位置 | 本设计 §位置 | 关系 |
|---|---|---|
| §1 四档划分 | §0.1 | 继承 |
| §3.2.2 hax-lean 档 1 实测 | §3.2 Stage 1 / §4.1 | 复用 oracle 论证 |
| §3.2.2 `int_checked` printer bug | §4.2 micro-test 选项 | 转化为反误报证据 |
| §4.1 hax-lean 档 2 实测 | §3.2 Stage 2 / §4.2 | 框架化 |
| §5 一致性比对实测 | §3.2 Stage 4 / §4.4 | 框架化 + 风险展开 |
| §6.2 适合 entry 类型 | §1.4 纯内部化判定准则 | 系统化 + 禁止集明示 |
| §7.1 multi-stage 模型 | §5 选项 a vs b | 完整决策记录 |
| §7.2 entry 标注 | §1.3 推荐方案 A + §2 schema | 完整展开 |
| §7.3 oracle 设计 | §4 反误报论证 | 按 [`tool-integration.md`](tool-integration.md) §三/§四 重写 |
| §7.4 工业三件套 | §1.4 禁止集 | 显式排除 |
| §8 成本估计 | §6 工作量估计 | 重估 |

---

**完。** 本设计稿不进入实施；待用户确认 §8.3 抉择后转工作流"细化阶段"产出 README + 实施说明，再写代码。
