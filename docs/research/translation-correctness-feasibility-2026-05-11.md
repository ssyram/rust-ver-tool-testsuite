# 翻译产物运行一致性测试 — 可行性调研（2026-05-11）

> 调研对象：rocq-of-rust / hax-lean / hax-coq 三个 Rust → 证明助手翻译工具
> 调研目标：判断"翻译产物在对应 prover 里能否 typecheck / evaluate / 与 Rust 一致"的实测可行性
> 范围：可行性调研，**不**实施新测试框架（不改 tool.toml / runner / cc-report / docs/design/）
> 工作目录：`/Users/ssyram/workspace/rust-ver/rust-ver-tool-testsuite/`
> 实测临时目录：`/tmp/translation-feasibility-2026-05-11/`，`/private/tmp/hax/hax-lib/proof-libs/`，`/private/tmp/rocq-of-rust-clone/RocqOfRust/`

---

## §1 调研目标

现有 testsuite 测的是"工具前端接受度"（runner 跑工具 + oracle 检 `.v`/`.lean` 产物形状 + 失败 marker grep）；用户希望额外加"产物的语义可信度"。可信度按四档划：

| 档 | 含义 | 现状 |
|---|---|---|
| 档 0 | 工具自陈"前端接受" | **现有**，已覆盖 |
| 档 1 | 产物在对应 prover 里 typecheck / 编译通过 | 本调研主问题 |
| 档 2 | 产物 entry_fn 在 prover 里能 evaluate / Compute 出结果 | 本调研次问题 |
| 档 3 | evaluate 结果 == `cargo run` 结果（终极语义验证） | 本调研终极问题 |

档 1 是"产物语法 + 类型在 prover 严格性下立得住"；档 2 是"产物的运行时定义闭合（无 Admitted / sorry 拦路）"；档 3 是"翻译的语义保持"。三档对工具可信度的暴增方向：档 1 排除工具瞎写假装翻译；档 2 排除产物结构性死锁；档 3 是工具语义正确性的强经验证据。

---

## §2 本机 prover 可用性

实测命令：`which lean lake elan coqc rocq opam fstar.exe`、各自 `--version`、`opam switch list`。

| Prover | 状态 | 版本 | 路径 / 备注 |
|---|---|---|---|
| **Lean 4** | 装 | 4.29.1（项目要 4.29.0-rc1，rebuild Hax prelude 后 lake 自动拉 4.29.0-rc1 toolchain） | `/Users/ssyram/.elan/bin/lean` |
| **lake** | 装 | 5.0.0-src+f72c35b | `/Users/ssyram/.elan/bin/lake` |
| **elan** | 装 | 4.2.1 | `/Users/ssyram/.elan/bin/elan` |
| **Coq** | 装（多 switch） | **8.20.1**（在 opam switch `rocq9` / `rocq-vst`；非 active switch） | `/Users/ssyram/.opam/rocq9/bin/coqc` |
| **Rocq 9.0+** | **缺** | — | 本机无 Rocq ≥9.0；opam 仓库 `coq-released` 默认是 Coq 8.x |
| **F\*** | 缺 | — | `which fstar.exe` 无结果（本任务不需要） |
| opam | 装 | 2.5.0 | active switch = `soteria-install`；用 `opam exec --switch=rocq9` 切换 |

opam switches：`default`、`rocq9`（Coq 8.20.1）、`rocq-vst`（Coq 8.20.1）、`/Users/ssyram/.creusot`、`soteria-install`（active）。

**已装的额外 Coq lib**（调研中安装）：`coq-record-update 0.3.6`（在 `rocq9` switch；通过新增 `coq-released` opam repo 装的）。

**关键缺失**：
- **Rocq 9.0+**：rocq-of-rust runtime library `Require Export Stdlib.Strings.PrimString.` 等指向 Rocq 9 的新 `Stdlib.*` 命名空间，Coq 8.20 无该 namespace —— **rocq-of-rust 在本机不可能 typecheck**（除非装 Rocq 9.0+）

---

## §3 产物可编译性实测

### §3.1 实测 entry 集

通过 runner 在 9 个 task 跑 SUCCESS：

```bash
target/release/runner --tool rocq-of-rust --tool hax-lean --tool hax-coq \
  --entry 'hello/basic-hello/*' --entry 'int/checked/*' --entry 'int/wrapping/*'
```

结果：9/9 SUCCESS（exit 0、产物 shape OK、无 failure marker / sorry）。run_dir: `runs/run-1778464562-91578`。

但 runner cleanup work_dir（`runner/src/exec.rs:251`），产物不留盘。**手动 reproduce** 在 `/tmp/translation-feasibility-2026-05-11/{hello/basic-hello, int/checked, int/wrapping, custom_add}/`，外加 `custom_add`（pub fn add_two(a,b)->i32 + fact(n)->i32，故意构造有返回值便于一致性比对）。

### §3.2 三工具产物逐一编译实测

#### 3.2.1 rocq-of-rust

产物形态（`hello/basic-hello/rocq_translation/src/lib.v`）：

```coq
Require Import RocqOfRust.RocqOfRust.
Definition hello (ε : list Value.t) (τ : list Ty.t) (α : list Value.t) : M :=
  match ε, τ, α with
  | [], [], [] => ltac:(M.monadic (M.match_operator (| ... |)))
  | _, _, _ => M.impossible "wrong number of arguments"
  end.
Global Instance Instance_IsFunction_hello : M.IsFunction.C "lib::hello" hello. Admitted.
```

直接 `coqc lib.v`：

```
File "./lib.v", line 2, characters 15-36:
Error: Cannot find a physical path bound to logical path RocqOfRust.RocqOfRust.
```

→ 需要外部 `RocqOfRust` runtime library。

`RocqOfRust` runtime 在 `/private/tmp/rocq-of-rust-clone/RocqOfRust/`（1833 个 .v 文件，250M）。其入口 `RocqOfRust.v` 第一行：

```
Require Export Stdlib.Strings.Ascii.
```

`Stdlib.*` 命名空间是 **Rocq 9.0+** 才有，Coq 8.20 报 `Cannot find a physical path bound to logical path Stdlib.Strings.Ascii.` —— **rocq-of-rust runtime 在 Coq 8.20 不能编译**。

**结论**：rocq-of-rust 产物档 1 在**本机不可达**（需先装 Rocq 9.0+ 并编译完整 1833 个 .v 的 runtime library）。

#### 3.2.2 hax-lean

产物形态（`custom_add.lean`）：

```lean
import Hax
import Std.Tactic.Do
...
namespace custom_add
@[spec]
def add_two (a : i32) (b : i32) : RustM i32 := do (a +? b)
@[spec]
def fact (n : i32) : RustM i32 := do
  if (← (n <=? (1 : i32))) then do (pure (1 : i32))
  else do (n *? (← (fact (← (n -? (1 : i32))))))
partial_fixpoint
end custom_add
```

需要 hax lean prelude（`/private/tmp/hax/hax-lib/proof-libs/lean/`，lakefile.toml 项目，要 `leanprover/lean4:v4.29.0-rc1`）。

**编译 prelude**：`lake update` + `lake build` 实测**成功**（elan 自动拉 4.29.0-rc1 toolchain；91 个模块全 build；耗时几分钟）。

**把产物放进 prelude 项目 typecheck**（拷到 `proof-libs/lean/Test_<entry>.lean` 然后 `lake env lean Test_*.lean`）：

| entry | exit code | 备注 |
|---|---|---|
| `basic_hello.lean` | **0**（SUCCESS） | 干净 typecheck |
| `int_wrapping.lean` | **0**（SUCCESS） | 干净 typecheck |
| `int_checked.lean` | **0** 但有 error 输出 | `Test_int_checked.lean:22:41: error: unexpected token '←'; expected ':=' or '|'` + warn `declaration uses 'sorry'`（hax prelude 里 `checked_add` 是 sorry） |
| `custom_add.lean` | **0**（SUCCESS） | 干净 typecheck |

注意 `int_checked` 这一例 —— hax 翻译产物本身有**语法形态 Lean 拒绝**（line 22 `let _ : (core_models.option.Option i32) ←` 后接换行 + 表达式，Lean parser 不认）；不是 prelude 问题，是 **hax-lean printer 自身的 bug**（实测发现，应反馈上游）。`int_wrapping` 类似 do-block 形态却 OK，说明这是 hax-lean 对 `?`-suffix operator 在 stmt position 的部分形态 broken。

**结论**：hax-lean 档 1 **可达**（lake prelude 可编译 + 产物可 typecheck），但**实测发现 hax-lean printer 输出对部分构造（带类型注解的 `let ... ←`）有语法形态错**——这是个翻译质量问题，**也直接被档 1 测试发现**。

#### 3.2.3 hax-coq

产物形态（`Custom_add.v`）：

```coq
From Coq Require Import ZArith.
...
From RecordUpdate Require Import RecordSet.
From Core Require Import Core.
Definition add_two (a : t_i32) (b : t_i32) : t_i32 := f_add (a) (b).
Fixpoint fact (n : t_i32) : t_i32 :=
  if f_le (n) ((1 : t_i32)) then (1 : t_i32)
  else f_mul (n) (fact (f_sub (n) ((1 : t_i32)))).
```

形态比 rocq-of-rust 干净——直接函数式 + native 调用 hax Coq Core 库定义的 `t_i32 / f_add / f_mul`。

直接 `coqc Custom_add.v`：

```
File "./Custom_add.v", line 10, characters 33-42:
Error: Cannot find a physical path bound to logical path RecordSet with prefix RecordUpdate.
```

→ 装 `coq-record-update`（opam 通过 `coq-released` repo 装，0.3.6 成功）。

再编 hax Core 库（`/private/tmp/hax/hax-lib/proof-libs/coq/coq/generated-core/src/`，59 个 .v 文件）：

- 用 `coq_makefile -f _CoqProject -o Makefile` + `make` 失败（Makefile target `.Makefile.d` 在 8.20 的 `coq_makefile` 模板下不自动生成）
- 改用 topo 排序 + 逐文件 coqc，结果：
  - **18/59 文件 typecheck 通过**
  - **41/59 失败**，主要两类硬错：
    1. `Core_Marker.v`：`Warning: Automatically putting t_Copy in Prop` 在 Coq 8.20 已变 hard error（`-w "-elaboration-warning,all"` 可绕过 → 18 个里的 1 个就这么救的）
    2. `( _ | _ )` Notation deprecated → hard error in Coq 8.20（无法用 `-w` 绕过；hax Core 库源码本身用了 8.19 的 pattern syntax，**8.20 不兼容**）
    3. `Core_Base_Spec.v` 在 `generated-core/spec/` 子目录但 `_CoqProject` 没 -R 进来（**hax-coq Core 库自身的 build 配置 bug**——`spec/` 路径漏配）

→ hax Core 库**不能在本机 Coq 8.20 完整编译**。即便 `coq-record-update` 装上、`-w` 绕开 warning-as-error，`( _ | _ )` 这一硬冲突需要 hax 上游修 / 切到 Coq 8.19 才能解。

**结论**：hax-coq 档 1 **本机不可达**。需要 (a) 装 Coq 8.19 或 (b) hax 上游修复 Core 库对 8.20 / 9.0 的兼容。

---

## §4 产物可 evaluate 性实测

### §4.1 hax-lean — 完整跑通

把 `custom_add.lean` 拷到 prelude 项目里加 `#eval` 语句：

```lean
#eval custom_add.add_two 3 4         -- → RustM.ok 7
#eval custom_add.add_two 10 (-7)     -- → RustM.ok 3
#eval custom_add.fact 5              -- → RustM.ok 120
#eval custom_add.fact 6              -- → RustM.ok 720
```

`lake env lean Test_custom_add.lean` exit 0，stdout 输出四个 `RustM.ok <n>`。**hax-lean 档 2 实测命中**。

`#eval basic_hello.hello rust_primitives.hax.Tuple0.mk` → `RustM.ok {  }`（无返回值的 entry 也能 evaluate，但比对意义弱）。

### §4.2 hax-coq / rocq-of-rust — 未实测

- **hax-coq**：档 1 未达 → 档 2 不可测。但产物形态干净（`Definition add_two (a b : t_i32) : t_i32 := f_add a b`），档 1 一旦达到（修 Core 库），`Compute add_two ((1 : t_i32)) ((2 : t_i32))` 理论上可行；前提 `f_add` 在 Core 库里是 `Definition`（reducible）非 `Parameter / Axiom`。
- **rocq-of-rust**：档 1 在本机不可达；即便达到（装 Rocq 9 + 编完 1833 个 .v），产物形态是 deep embedding —— `add_two [] [] [Value.Integer IntegerKind.I32 3; Value.Integer IntegerKind.I32 4]`，输出是 `M` (= `LowM.t (Value.t + Exception.t)`)，要 Compute 必须自己写 evaluator 把 `M` 跑到 `Value.t`。`Global Instance Instance_IsFunction_X : ... Admitted.` 不影响 `Compute X args`（Admitted 是 typeclass instance 证明 admitted，函数定义本身是 `Definition` 完整的）。

---

## §5 一致性比对实测

仅 **hax-lean × custom_add** 一组实测可比，因为：
- hello/int_checked/int_wrapping 三个原 corpus entry **返回 `()`** —— Rust 端跑也只能 `println!("ran")`，无值可比
- hax-coq / rocq-of-rust 档 1 未达，无 evaluate 结果

| entry | 输入 | Rust 直接 `cargo run` | hax-lean `#eval` | 一致？ |
|---|---|---|---|---|
| `add_two(a, b)` | a=3, b=4 | `7` | `RustM.ok 7` | **是** |
| `add_two(a, b)` | a=10, b=-7 | `3` | `RustM.ok 3` | **是** |
| `fact(n)` | n=5 | `120` | `RustM.ok 120` | **是** |
| `fact(n)` | n=6 | `720` | `RustM.ok 720` | **是** |

**hax-lean 档 3 实测命中**（在 custom_add 这一例上）。需要 unwrap `RustM.ok` 才能字符串比对，但语义已对上。

---

## §6 整体可行性判断

### §6.1 三工具档位实测判断（2026-05-11 本机快照）

| 工具 | 档 0 (前端) | 档 1 (typecheck) | 档 2 (evaluate) | 档 3 (一致) | 备注 |
|---|---|---|---|---|---|
| **hax-lean** | ✅ 已覆盖 | ✅ **可达**（lake prelude rebuild OK + 产物可 typecheck） | ✅ **可达** | ✅ **可达** | 例外：`int_checked` 形态有 printer bug；prelude 部分 method 是 `sorry`（如 `checked_add`），命中这些会 typecheck pass 但 evaluate 结果不可信 |
| **hax-coq** | ✅ 已覆盖 | ❌ **本机不可达** | ❌ | ❌ | 阻塞：Core 库源 `( _ | _ )` 8.20-incompatible + `Core_Base_Spec` 路径 -R 配置 bug + `coq-record-update` 需手动装 |
| **rocq-of-rust** | ✅ 已覆盖 | ❌ **本机不可达** | ❌ | ❌ | 阻塞：runtime 要 Rocq 9.0+ `Stdlib.*` namespace；本机仅 Coq 8.20.1。即便装 Rocq 9.0，1833 个 .v 全 build 是大工程 |

注："可达"是**实测过 / 已知可通过本机操作完成**；"不可达"是**本调研未完成 + 已识别明确阻塞**，不是"工具产物根本不能 typecheck"。

### §6.2 适合做一致性测试的 entry 类型

**适合**（hax-lean 实测验证）：
- 纯函数 `pub fn f(args...) -> primitive { ... }`，参数和返回是 `i*/u*/bool`
- 无 panic、无 unsafe、无 mut ref、无 IO、无依赖外部 crate
- 递归 OK（`fact` 实测 `partial_fixpoint` 跑通）
- 形态：仅 arith / cmp / `if`-`else` / `match` / let-binding / 简单 recursion

**不适合**：
- 返回 `()` 或副作用型 entry（无值可比）
- mut ref / interior mutability（`RefCell`、`Cell`）—— hax prelude 里多半是 sorry
- panic / unsafe block（hax 直接 reject / sorry）
- 大状态 / `Vec` / `HashMap` —— hax-lean prelude `core_models` 部分功能未完成
- std 库的 `checked_*` / `wrapping_*` 之外的稀有 API（命中 prelude sorry 路径）
- 浮点（Coq 端要 `Coq.Floats.Floats`，hax-coq 路上还有更多依赖陷阱）

corpus 推荐子集（先优先 hax-lean）：约 5–10 个**有返回值、纯计算**的 entry，类型局限 i32/u32/bool/tuple、签名 1–3 参、行为可枚举（递归 ≤ 几层 / 小 input space）。当前 corpus 大部分 entry 是 `() -> ()`，需**新增一批**有返回值的"语义比对友好型"entry（不污染现有特性覆盖广度 corpus）。

### §6.3 框架化的总体判断

**短期（hax-lean only）yes**：lake prelude 已 build OK + 产物可 typecheck/eval 实测验证 + 一致性 OK 实测验证。可以做。

**短期 hax-coq / rocq-of-rust no**：阻塞在 prelude / runtime 编译，且阻塞**不在 testsuite 范围**——需要 hax 上游 / Rocq 9 装机。即便上游修了，rocq-of-rust 还有 deep-embedding M monad 的 evaluator 写作问题，比 hax-lean 至少多一步。

**中期可解**：hax-coq 装 Coq 8.19 switch 或等上游修 `( _ | _ )` notation；rocq-of-rust 装 Rocq 9 switch + 编 runtime。但这俩都是工具基础设施工作，不应是 testsuite 主线。

---

## §7 框架化设计建议（若推进）

### §7.1 multi-stage 模型

按 `principles.md` §三的"非首要模块"原则，框架化属次要模块（工具集成深度调优），**不应抢核心模块（runner + examples）优先级**。但若推进，最小设计如下：

**关键判断**：当前 runner 的 task 模型已经支持多 entry × 多 tool，每个 task 是一个 `tool.command`。**typecheck / evaluate 可以建模为独立 tool 而不是 runner 改造** —— 这符合宪法 §六"不允许 partial / 形式指标为最终解释"的精神，且**不需要修 runner**。

可行的最小做法：**新增独立 tool 入口**（每个对应一档可信度）：

| 假想 tool | 作用 | oracle |
|---|---|---|
| `hax-lean-typecheck` | wrap 现有 hax-lean，跑完再把产物拷到 hax lean prelude 项目里 lake env lean | exit 0 + 无 error + 无 sorry warning |
| `hax-lean-eval` | 在 typecheck 基础上 append `#eval entry_fn <args>` 然后 lake env lean，stdout 抓 RustM.ok | exit 0 + stdout 含 `RustM.ok <expected>` |
| `hax-lean-consistency` | hax-lean-eval + cargo run + 比对 | stdout 完全相同（unwrap `RustM.ok`） |

类似为 hax-coq、rocq-of-rust 各开三个；前提是档 1 的 prelude 装好。

**`<args>` / `<expected>` 怎么来**：在 corpus 的 `hirusttest.toml` 加一个新 field 比如 `inputs = [...]` 和 `expected = [...]`，或者引入 `oracle.toml` 单独存"运行一致性 oracle"。这样不污染现有 corpus 的"前端接受"语义。

### §7.2 entry 标注

corpus 标注哪些 entry 适合一致性测试。新增一个 manifest field（如 `runnable = true` + `inputs`/`expected`），只跑那些标注的。其他 entry 仍只跑档 0。

### §7.3 oracle 设计

档 1 的 oracle：`lean / coqc / rocq` 退出码 0 且 stderr 无 `error:` / `warning: declaration uses 'sorry'`（后者抓 prelude 内 sorry 命中，避免假信号）。

档 2 的 oracle：`#eval` / `Compute` 输出非空 + 不含 `sorry` / `M.impossible` / `Inhabited.default`。

档 3 的 oracle：unwrap monad wrapper（如 `RustM.ok` / `M.ok` / `Value.Integer ... <n>`）后字符串相等 `cargo run` 输出。需要 per-tool 的 unwrap 规则。

### §7.4 工业三件套是否适用

**几乎不行**：industrial corpus 的 `rsa / sha2 / x509-parser` 都用了大量 std::collections、bytes、外部 crate；hax 即便 typecheck 过，evaluate 大概率命中 prelude 内 sorry。一致性测试天然只能跑"纯算 + 小输入 + 不依赖未实现 std"的 entry。

---

## §8 实施成本估计

| 阶段 | 工作 | 估时 |
|---|---|---|
| **立即可做**（本调研已完成的可重复部分） | 写 5–10 个 i32/u32/bool 返回值的 pure entry + 手工跑 hax-lean SUCCESS 路径上的一致性验证 | 半天 |
| **短期（hax-lean 框架化）** | 新增 tool `hax-lean-typecheck` / `hax-lean-eval` / `hax-lean-consistency` + corpus `inputs`/`expected` field + runner 跑、oracle 比对 | 1–2 天（取决于 corpus inputs 编辑量） |
| **中期（hax-coq 解阻）** | opam 装 Coq 8.19 switch（或等上游修 8.20 兼容）+ build Core 库 + tool 加 hax-coq-{typecheck,eval,consistency} | 半天–几天（看上游修复时间） |
| **长期（rocq-of-rust 解阻）** | opam 装 Rocq 9.0+ switch + build 1833-file runtime + 写 M monad evaluator（把 `M` 跑到 `Value.t`） | 几天–一周（runtime build 风险高） |
| **corpus 标注** | 跨整个 examples/ 标注哪些 entry runnable + 加 inputs/expected | 1–2 天 |

**immediate action 推荐**（若用户批准）：
1. 先在 examples 下新增一个 `examples/consistency/`（或者复用 `examples/int/` 加 `runnable` 标注），写 5–10 个有返回值的 pure entry
2. 手工 reproduce §5 的一致性验证流程（不入 runner），先确认 hax-lean 一致性测试在 corpus 多样 entry 下不退化
3. 然后再决定是否值得为这个测试做 runner 集成（vs 留作半自动手工跑的次要工具）

按 `feedback_no_premature_extensibility.md` 精神：**先用 5 个手工 entry 把"hax-lean 一致性测试"真值验证出来，再讨论是否值得 runner 集成**；不预设要全自动框架。
