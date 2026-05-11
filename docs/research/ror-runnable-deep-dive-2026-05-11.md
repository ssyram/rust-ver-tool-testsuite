# rocq-of-rust 产物可运行性深入调研（2026-05-11）

> 调研对象：rocq-of-rust（formal-land 出品，本测试基线 commit `a8a76a4d`）
> 调研目标：判断"ror 产物在 Rocq 中能否 typecheck / evaluate / 与 Rust 输出一致"
> 上层调研基线：`docs/research/translation-correctness-feasibility-2026-05-11.md`（认为 ror 短期不可达）
> 工作目录：`/Users/ssyram/workspace/rust-ver/rust-ver-tool-testsuite/`
> 实测临时目录：
>   - 上游 ror 仓库镜像：`/private/tmp/rocq-of-rust-clone/`（已在上层调研中 clone）
>   - 编译实验：`/tmp/ror-eval-test/` + `/tmp/ror-eval-test/manual_test/`
>   - 隔离 opam switch：`ror-test`（Rocq 9.0.0 + 全部依赖；下文§3 详述）

---

## §1 调研目标

依上层调研的"档位"模型（`translation-correctness-feasibility-2026-05-11.md` §1）：

| 档 | 含义 | 本次目标 |
|---|---|---|
| 档 0 | 工具自陈"前端接受" | 已覆盖（runner）|
| **档 1** | 产物在对应 prover 里 typecheck / 编译通过 | **是否本地可达** |
| **档 2** | 产物 entry_fn 在 prover 里 Compute / evaluate 出值 | **是否可达 + 工作量** |
| **档 3** | evaluate 结果 == `cargo run` 结果 | **是否可达 + 工作量** |

上层调研认为 ror 短期不可达，阻塞点：(a) 本机无 Rocq 9.0+；(b) 产物是 `M : LowM.t (Value.t + Exception.t)` deep embedding，`Compute` 拿不到 native 值。

本次深入调研做四件事：(i) 在隔离 opam switch 装 Rocq 9.0；(ii) 编 ror runtime 库；(iii) 手工跑 micro entry 的档 1–3；(iv) 评估 short-term / mid-term 实施可行性 + 工作量。

**关键发现总览**：
- **档 1**：可达。在 `ror-test` switch（Rocq 9.0.0 + 必备 lib）下，4/4 实测产物（custom_add / basic_hello / int_wrapping / int_checked）独立编译通过，runtime 部分依赖（M / RocqOfRust / lib/lib / RecordUpdate）几秒可建好。完整 runtime 库（1509 个 .v）3 分钟内编了 1253 个 .vo（-j 4），预计 10–15 分钟全建完
- **档 2 / 档 3**：**架构上**ror 设计就**不支持**对自动生成产物直接`Compute`到 native 值；产物的 monad 是 syntactic deep embedding，所有 op（`alloc`/`read`/`call_closure`/`call_primitive`）都是 `LowM.t` inductive constructor，没有 Compute 语义。**真正的"evaluate"**走上游的 link + simulate 两阶段，**需要人工**为每个 entry 写：
  1. `Instance run_<fn> ... : Run.Trait <fn> ... := { run_f := ltac:(constructor; run_symbolic) }`（非递归 fn 可全自动）
  2. `Definition <fn>_simulated ... : <return_ty> := <pure Rocq expr>`（**手写**纯 Rocq 模拟版本）
  3. `Lemma <fn>_eq ... : SimulateM.eval_f (run_<fn> args) Nil 🌲 (Output.Success (<fn>_simulated args), Nil)`（非递归 fn 用 `repeat (eapply Run.Call || apply Run.Pure)` 一行 tactic 即可关闭；递归 fn 需要 induction）
- 这与 hax-lean 一致性测试**根本不同**：hax-lean 自动生成的 `RustM` monad 含 native eval（`do (a +? b)` 在 lean 里直接 `#eval`），而 ror 产物的 monad 必须配 hand-written 证明对象才能"运行"

---

## §2 上游 evaluator 调研（Q1）

### §2.1 上游架构（3 阶段 pipeline）

ror README 明确（`/private/tmp/rocq-of-rust-clone/README.md`）：

> The translation is in three steps:
>   1. Import of the THIR representation of Rust to Rocq, running `cargo rocq-of-rust`.
>   2. Type-checking and trait inference with native Rocq types and typeclasses (we call it linking).
>   3. Representation of the control-flow and memory manipulations in a purely functional style (we call it simulation).

只有 step 1 是工具自动的；step 2 / 3 是**人工证明工作**。README 头号示例 `add_one` 表明这一点（`/private/tmp/rocq-of-rust-clone/RocqOfRust/examples/default/examples/custom/`）：

- **translate（自动）**：`add_one.v` 写出 `Definition add_one (ε τ α) : M := ...`
- **link（人工）**：`links/add_one.v` 手写
  ```coq
  Instance run_add_one (x : u32) : Run.Trait add_one [] [] [φ x] u32.
  Proof. constructor. run_symbolic. Defined.
  ```
- **simulate（人工）**：`simulate/add_one.v` 手写
  ```coq
  Definition add_one (x : u32) : u32 := x +i 1.
  Lemma add_one_eq (x : u32) :
    {{ SimulateM.eval_f (run_add_one x) []%stack 🌲
       (Output.Success (add_one x), []%stack) }}.
  Proof. repeat (eapply Run.Call || apply Run.Pure). Qed.
  ```

### §2.2 evaluator 存在吗？

**有**，名字是 `SimulateM.eval` / `SimulateM.eval_f`（`/private/tmp/rocq-of-rust-clone/RocqOfRust/simulate/M.v:343 + 445`）：

```coq
Fixpoint eval {R Output : Set}
    (e : LinkM.t R Output) (stack : Stack.t)
    {struct e} :
  t (Output.t R Output * Stack.t).
... (* 9 case destruct on LinkM.t *)
Defined.

Definition eval_f {f : PolymorphicFunction.t} {ε τ α} {Output : Set} `{Link Output}
    (run : Run.Trait f ε τ α Output) :
    Stack.t -> t (Output.t Output Output * Stack.t) :=
  eval (links.M.evaluate run.(Run.run_f)).
```

但**有三个关键约束**让它不能直接 Compute 出值：

1. **输入不是产物 `M` 而是 `LinkM.t R Output`**：`eval_f` 需要 `Run.Trait f ε τ α Output` 类型类实例。这个实例的 `run_f` 字段是一个 `Run.t` proof tree（`links/M.v:830` 的 inductive），用户必须用 `run_symbolic` tactic 推导出来。无 `Run.Trait` 实例 → 无法调 `eval_f`
2. **`SimulateM.t` 本身是 inductive，不是 `Z`**：`eval_f` 返回 `SimulateM.t (Output.t Output Output * Stack.t)`，是一个含 5 个 constructor（`Pure / GetCanAccess / Let / Call / Impossible`）的 inductive，**不是 native Coq 值**。其中 `Call` constructor 携带一个 `Run.t` 证明对象与续延，**Compute 无法将其展开成原子结果**——除非递归 `eval` 在 `Run.t` proof 上做 induction（即 lemma `Run.Call` rule）
3. **`Run.t` 关系是 propositional，不是 functional**：`eval_f` 与"value"的关系是 propositional 的（`🌲` notation 是 `Run.t value (eval_f run stack)` 的 sugar），用 `repeat (eapply Run.Call || apply Run.Pure)` tactic **证明**——不是 `Compute` 后得 value

**结论**：上游有 evaluator，但它是**用于 proof-relevant trace 的 propositional 解释器**，不是计算到数值的 native evaluator。`Compute (SimulateM.eval_f ...)` 给出 `SimulateM.t` syntactic tree（含 `Call ... fun ...`），不是 `Z`。

### §2.3 实测：`cbn` 在简单 add_two 上的行为

实测（`/tmp/ror-eval-test/manual_test/custom_add_simulate.v`）`Eval cbn in result_3_4` 其中 `result_3_4 = SimulateM.eval_f (run_add_two a3 b4) Nil`，`a3 = {| Integer.value := 3 |}`、`b4 = {| Integer.value := 4 |}`：

```coq
= SimulateM.Call []%stack
    (Run.PureSuccess (Integer.t IntegerKind.I32)
       (Integer.t IntegerKind.I32) (Value.Integer IntegerKind.I32 7)
       {| Integer.value := 7 |} OfValueWith.of_value_with)
    (fun '(output, stack) =>
     SimulateM.Pure (Output.Success output, stack))
```

**注**：(a) 加上 `Transparent Z.add Z.sub Z.mul Z.modulo Z.pow.` 才能让 `Integer.normalize_wrap I32 (3 + 4)` cbn 到 `7`；不 transparent 时 `cbn` 得 `(3 + 4 + 2 ^ 31) mod 2 ^ 32 - 2 ^ 31`。`simulate/RocqOfRust.v` 顶部把这些 `Z.*` 标 `Global Opaque`（`/private/tmp/rocq-of-rust-clone/RocqOfRust/simulate/RocqOfRust.v:8`）；(b) 结果**仍然是 `SimulateM.Call` constructor**，不是 `(Output.Success 7, Nil)` —— `Call` 持有 `Run.t` 证明对象 + 续延，**Compute 不会 apply 续延**

可以看到值 `7` 是出现在结构里的（`Value.Integer IntegerKind.I32 7` + `{| Integer.value := 7 |}`），但作为 proof-tree 的字段，不是被 evaluator 输出的"结果"。

`vm_compute` / `native_compute` 直接 SIGSEGV（exit 139）——`Run.t` proof tree 里含 axiom（如 `links/M.v:1659 axiom is_discriminant_tuple_eq`、之前的 `IsTraitAssociatedType_eq`），native_compute 在遇到 axiom 时会崩

---

## §3 Rocq 9 安装实测（Q2）

### §3.1 安装命令链（成功路径）

```bash
# 创建隔离 switch（不动 default / rocq9 / rocq-vst / soteria-install / .creusot）
opam switch create ror-test --empty

# 加 coq-released opam repo（已默认）
opam repo add coq-released https://coq.inria.fr/opam/released --switch=ror-test --rank 1

# 安装 ocaml 编译器 + dune（Rocq 依赖）
opam install --switch=ror-test ocaml-base-compiler.5.2.0 dune.3.18.2 -y
# → 耗时 ~2 分钟，installed: ocaml.5.2.0 + dune.3.18.2 + 基础包

# 安装 Rocq 9.0.0（rocq-core + rocq-stdlib）
opam install --switch=ror-test rocq-core.9.0.0 rocq-stdlib.9.0.0 -y
# → 耗时 ~10 分钟（rocq-stdlib 含 Reals 等较慢的模块）
# installed: rocq-runtime.9.0.0 + rocq-core.9.0.0 + rocq-stdlib.9.0.0

# 安装 ror runtime 依赖的额外 lib：smpl + hammer + coqutil
opam install --switch=ror-test rocq-smpl coq-hammer.1.3.2+9.0 coq-coqutil -y
# → 耗时 ~3 分钟
# installed: rocq-smpl.9.0 + coq-hammer.1.3.2+9.0 + coq-hammer-tactics.1.3.2+9.0
# + coq-coqutil.0.0.7 + coq.9.0.0 (兼容元包，提供 coqc 别名)
```

注 `coq-hammer.1.3.2+9.0` 是 hammer 上游为 Rocq 9.0 发布的 tag。

### §3.2 实测结果

| 阶段 | 时间 | 大小 | 状态 |
|---|---|---|---|
| switch 创建 + ocaml + dune | ~2 min | ~500 MB | ✅ |
| Rocq 9.0.0 + stdlib | ~10 min | +900 MB | ✅ |
| smpl + hammer + coqutil | ~3 min | +200 MB | ✅ |
| **总计** | **~15 min** | **~1.6 GB** | ✅ |

`/Users/ssyram/.opam/ror-test/bin/{rocq, coqc}` 可用，version `The Rocq Prover, version 9.0.0`（OCaml 5.2.0）。

### §3.3 已用 / 不用对照

| 操作 | 是否影响默认环境 |
|---|---|
| 装 ocaml 5.2.0 + Rocq 9.0.0 在 `ror-test` switch | **无影响**（隔离） |
| `opam env --switch=ror-test --set-switch` | 只改当前 shell session 的 PATH，每次工作前 eval |
| 没有改 default switch 的任何包 | ✅ |
| 没有改 `rocq9` switch（Coq 8.20.1）的任何包 | ✅ |
| 没有改 `brew` / 系统 PATH / dotfile | ✅ |

**清理方案**：`opam switch remove ror-test`（一次性 release 1.6GB）。

---

## §4 runtime 编译实测（Q3）

### §4.1 最小核心 runtime（手工逐文件）

为支持档 1 实测，先编最小子集（手工 `coqc -R . RocqOfRust -impredicative-set <file>`）：

| 顺序 | 文件 | 耗时 | 状态 |
|---|---|---|---|
| 1 | `RecordUpdate.v` | < 1 s | ✅ |
| 2 | `M.v`（核心 monad 定义 + Primitive / LowM / Value / IsFunction 等） | < 1 s | ✅ |
| 3 | `lib/lib.v`（`BinOp.Wrap.add` 等基础 closure） | < 1 s | ✅ |
| 4 | `RocqOfRust.v`（顶层 re-export） | < 1 s | ✅ |
| 5 | `links/M.v`（`Run.Trait` typeclass + `LinkM.t` + `links.M.evaluate` + `run_symbolic` tactic） | < 1 s | ✅ |
| 6 | `links/RocqOfRust.v` | < 1 s | ✅ |
| 7 | `lib/simulate/lib.v` | < 1 s | ✅ |
| 8 | `simulate/M.v`（`SimulateM.t` + `eval` + `eval_f` + 命题 `Run.t`） | < 1 s | ✅ |
| 9 | `simulate/RocqOfRust.v` | < 1 s | ✅ |

**总 9 个文件，全部秒级编译通过**。这是档 1 + 档 2 实测所需的最小依赖闭包。

### §4.2 完整 runtime build（make all -j 4）

后台跑 `make all -j 4`，分两段：

- 第一段（被 timeout 中断后已恢复运行）：3 分钟 → 1253 / 1509 个 .vo
- 第二段（继续后台跑）：~1297 / 1509 时仍在编 `revm/revm_interpreter/instructions/links/*.v` 系列（REVM 链路最重）

按目前速率，完整 runtime 全 build 预估 **15–20 分钟**（host: Apple M5, -j 4）。本调研未等到 100% build 完成，但**关键发现**：所有 ror 产物档 1 + 简单档 2 实测**只需 §4.1 的最小核心 runtime**，**不需要全建 1509 个**。REVM 等大模块**与本调研无关**。

### §4.3 已知 build 难点

- `blacklist.txt` 标了若干超慢 / 在 8.x 留下的文件（`revm/types.v` "ten minutes"、`move_sui/links/*` "links not working yet" 等）—— 这些在 `make all` 自动 skip
- 没有遇到任何 hard error / 兼容性 bug，对比 hax-coq 的 8.20-incompat `( _ | _ )` notation 问题更顺畅

---

## §5 档 1–3 实测（Q4）

### §5.1 档 1（typecheck）

实测 4 个 entry（产物已存档于 `/private/tmp/translation-feasibility-2026-05-11/`，上层调研 §3 同一批）：

| entry | 产物路径 | coqc 命令 | 状态 | 时间 |
|---|---|---|---|---|
| `custom_add` | `custom_add/rocq_translation/src/lib.v` | `coqc -R /private/tmp/rocq-of-rust-clone/RocqOfRust RocqOfRust -impredicative-set custom_add.v` | ✅ | < 1 s |
| `basic_hello` | `hello/basic-hello/.../lib.v`（重命名 `basic_hello.v` 避免 `-` 影响 Rocq identifier） | 同上 | ✅ | < 1 s |
| `int_wrapping` | `int/wrapping/.../lib.v` | 同上 | ✅ | < 1 s |
| `int_checked` | `int/checked/.../lib.v` | 同上 | ✅ | < 1 s |

**4/4 PASS**。这与上层调研 §3.2.1 的结论"档 1 在本机不可达"对照 ── **不再不可达**，是装 Rocq 9 后可达。

### §5.2 档 2（Compute / evaluate 出值）

#### 5.2.1 直接 Compute 在 raw M 上 — 不可行（如预期）

```coq
Definition test_add_3_4 : M :=
  add_two [] [] [Value.Integer IntegerKind.I32 3; Value.Integer IntegerKind.I32 4].
Eval cbv [add_two test_add_3_4] in test_add_3_4.
```

输出：

```
= let* ' a0 := alloc (Ty.path "i32") (Value.Integer IntegerKind.I32 3)
  in let* ' b0 := alloc (Ty.path "i32") (Value.Integer IntegerKind.I32 4)
    in let* ' v := read a0
      in let* ' v0 := read b0
        in call_closure (Ty.path "i32") BinOp.Wrap.add [v; v0]
: M
```

**确认**：raw `M` monad 的 `alloc / read / call_closure` 都是 `LowM.CallPrimitive / LowM.CallClosure` constructor，**无 Compute 语义**。这是 deep embedding 的固有结构，**不是工具 bug**。

#### 5.2.2 走 link + simulate 路径 — 部分可行（要人工）

`/tmp/ror-eval-test/manual_test/custom_add_link.v`（**人工**）：

```coq
Require Import RocqOfRust.links.RocqOfRust.
Require Import custom_add.

Instance run_add_two (a b : i32) : Run.Trait add_two [] [] [φ a; φ b] i32.
Proof.
  constructor.
  run_symbolic.
Defined.
Global Transparent run_add_two.
```

实测：**自动通过**（`run_symbolic` 一行 tactic 解决；对**非递归、仅 prim arith 的 fn**全自动）。

`/tmp/ror-eval-test/manual_test/custom_add_simulate.v`（**人工**）：

```coq
Require Import RocqOfRust.simulate.RocqOfRust.
Require Import custom_add_link.

Definition add_two_simulated (a b : i32) : i32 := a +i b.

Lemma add_two_eq (a b : i32) :
  {{ SimulateM.eval_f (run_add_two a b) []%stack 🌲
     (Output.Success (add_two_simulated a b), []%stack) }}.
Proof.
  repeat (eapply Run.Call || apply Run.Pure).
Qed.

Definition a3 : i32 := {| Integer.value := 3 |}.
Definition b4 : i32 := {| Integer.value := 4 |}.
Transparent Z.add Z.sub Z.mul Z.modulo Z.pow.

Eval vm_compute in (add_two_simulated a3 b4).             (* → {| Integer.value := 7 |} *)
Eval vm_compute in (add_two_simulated a3 b4).(Integer.value). (* → 7 *)
```

实测：lemma 一行 tactic 关闭，`vm_compute` 拿到 `7`。

**关键观察**：值 `7` 是 `add_two_simulated`（**手写**）vm_compute 出的，**不是** `SimulateM.eval_f (run_add_two a3 b4) Nil` vm_compute 出的。后者会 SIGSEGV（axiom 在 native_compute 下 crash），或 cbn 卡在 `SimulateM.Call ... fun ...` 结构（如 §2.3）。

**结论档 2**：可达——但靠**手写 simulate 函数 + lemma 证明**。lemma 保证"手写函数 == ror 产物的语义"，然后手写函数 Compute。

#### 5.2.3 递归函数（fact）— 工作量上升

```coq
Instance run_fact (n : i32) : Run.Trait fact [] [] [φ n] i32.
Proof.
  constructor.
  run_symbolic.  (* leaves open goal *)
```

`run_symbolic` 自动展开所有 prim arith / match / if-else，但**留下递归 call 的 `Run.Trait fact [] [] [Integer.IsLink.(φ) value_inter]` 作为未关闭 goal**——递归函数需要手工 induction / fixpoint trick。

实测留下 1 个 goal：

```
1 goal
  n : i32
  value, value0, value_inter : Integer.t IntegerKind.I32
  ============================
  Run.Trait fact [] [] [Integer.IsLink.(φ) value_inter]
    (Integer.t IntegerKind.I32)
```

需要：(a) 改用 `Fixpoint run_fact` 而非 `Instance`，加 well-founded recursion；(b) 或对 `n.(Integer.value)` 用 `Z.lt_wf` strong induction。两种都**显著超出"automation 一行"工作量**。

### §5.3 档 3（与 Rust 一致）

`/private/tmp/translation-feasibility-2026-05-11/custom_add` 跑 `cargo run --bin custom_add_main`：

```
add_two(3,4) = 7
add_two(10,-7) = 3
fact(5) = 120
fact(6) = 720
```

ror simulated（基于 §5.2.2）：`(add_two_simulated a3 b4).(Integer.value) = 7` ✅ **match**。

但 **档 3 命中是 conditional on lemma 已证**：
- `add_two_eq` lemma 已证 → `add_two(3,4) = 7` 在 ror 翻译下成立
- `fact` 因 `run_fact` 未关闭 → 档 3 未达到

| entry | 输入 | Rust | ror simulated | lemma 证明 | 档 3 一致 |
|---|---|---|---|---|---|
| `add_two` | (3, 4) | 7 | 7 | ✅ `repeat (eapply Run.Call | apply Run.Pure)` | ✅ |
| `add_two` | (10, -7) | 3 | (theoretically 3) | ✅ 一般化的 `(a b : i32)` lemma 涵盖 | ✅ |
| `fact` | 5 | 120 | (未做出) | ❌ 留 1 open goal | ❌ |
| `basic_hello.hello` | (无返回) | "ran"（println） | — | — | n/a |

**档 3 实测：1/4 通过**（add_two 涵盖两组输入，但只有 1 个 lemma；fact / hello 不计入）。

---

## §6 可行性总结

### §6.1 短期可行性（P15 / P16 commit 周期）

| 档 | 可达 | 工作量 | 阻塞点 |
|---|---|---|---|
| **档 1（typecheck）** | ✅ **可达** | **低**（装 ror-test switch + 编 9 个核心 .v ≈ 15 min 一次性 + < 1s / entry）| 无 |
| **档 2（自动 Compute）** | ❌ **架构上不支持** | n/a | ror 产物是 deep embedding；`vm_compute / native_compute` 在 raw M 上得 syntactic 节点，crash on axiom-laden proof tree |
| **档 2（半人工 Compute）** | ✅ **简单 entry 可达**（非递归 + 纯 arith） | **高**（per-entry：手写 `run_<fn>` instance + 手写 `<fn>_simulated` Definition + 手写 lemma；`run_symbolic` 对非递归 fn 自动，但 `<fn>_simulated` 必须人工写 + lemma 必须人工证）| 递归 fn 需手工 induction；外部 std method（`wrapping_add` 等）需要完整 runtime build + 上游已有 `run_*` for 该 method |
| **档 3（一致性）** | ✅ **lemma 关闭后可达** | 与档 2 同 | 同档 2 |

**短期 yes/no**：

- 档 1：**yes**（一次性 setup 即可纳入 runner）
- 档 2/3：**no**（per-entry 工作量太大；不是单一 setup 就能 scale 到 corpus 的；本质是给用户提供一个 *证明工作台*，不是自动测试）

### §6.2 与 hax-lean 一致性测试的本质区别

上层调研 §6 显示 hax-lean 档 2 / 档 3 自动可达。两者差异：

| 维度 | hax-lean | ror |
|---|---|---|
| **产物的 monad** | `RustM`（**shallow embedding**，`do (a +? b)` 是 lean 函数）| `M = LowM.t (Value.t + Exception.t)`（**deep embedding**，`alloc / read / call_closure` 是 inductive constructor） |
| **直接 `#eval` / `Compute`** | ✅ lean `#eval add_two 3 4` 直接 `RustM.ok 7` | ❌ Compute 得 syntactic LowM tree |
| **要不要人工证明** | 不要——`partial_fixpoint` / `do` notation 都是 lean 自有的 recursive eval | 要——每个 entry 需 `Run.Trait` derivation + simulated form + lemma |
| **递归函数** | ✅ `partial_fixpoint` 自动 | ❌ 需手工 well-founded recursion |
| **stdlib method** | hax prelude 已实现 `checked_add / wrapping_add` 等（部分 sorry，命中时不可信但 typecheck 仍过）| ror runtime 已有 `Wrap.add / Wrap.sub` 等 closure，但走 `Primitive.GetAssociatedFunction` 路径要每个 method 一个 `Run.Trait` 实例 |
| **per-entry 工作量** | 0（直接 `#eval`）| 高（5–50 行手写证明 + simulated def，递归更多）|

**结论**：ror 与 hax-lean 在档 2 / 档 3 不是"同样 corpus、不同跑法"——ror 的档 2/3 是**用户提供的 Rocq 证明工作**，hax-lean 的档 2/3 是**lean 自动 eval**。两者的"一致性测试"语义并不对等。

### §6.3 ror 上游对档 2/3 的设计立场

实测可见 ror 上游对档 2/3 的设计立场是 **"产物是 proof obligation，不是 runnable"**：

- 产物头部带 `Global Instance Instance_IsFunction_<fn> : ... Admitted.` —— ror 自己**明确把"翻译完成 ≠ 可运行"标 admitted**
- 配套的 `simulate/<fn>.v` 在上游仓库里**只有 add_one 一个**——是手工写的 demo，证明可行性，不是普适
- 上游 README 头号示例说的"prove equivalent" 用词强调**equivalence proof**，不是 `Compute`

这是 ror 的**设计哲学**：把 Rust 程序翻译为 Rocq 中的 "proof relevant rough sketch"，把"语义"留作用户证明义务。

---

## §7 实施建议

### §7.1 短期（P15 / P16）：只做档 1，不动档 2/3

最优 ROI 路径：

1. 把 §3 的 ror-test switch setup 写成 `tools/rocq-of-rust/SETUP-eval.md`（不重写 README）
2. **新增** `tools/rocq-of-rust-typecheck/tool.toml`（独立 tool，wrap 现有 rocq-of-rust + 在产物上跑 `coqc`）。oracle: `coqc exit 0`
3. corpus 不需要标注 —— 直接复用现有 entry 的 rocq-of-rust SUCCESS 子集（111 个）跑档 1
4. 不动 runner 框架（每个 tool 独立 invoke）
5. **不**尝试档 2 / 档 3 ——因为：(a) per-entry 工作量太大；(b) 测试结果不会比"hax-lean 一致性测试"更有信息量；(c) ror 上游的设计立场就是"语义证明留给用户"，自动化"运行"违背工具意图

实施估计：runner 配置 + tool.toml 写法 + 一次 setup → **半天–一天**。

### §7.2 中长期：档 2/3 不推荐

理由：

- **per-entry 工作量与 corpus 规模冲突**：现 corpus ~150 entry，按"5–50 行手写证明"估，全 cover 是数周–数月的 proof engineering；这超出了"特性覆盖广度筛选" testsuite 的核心任务
- **测试结果可信度不增**：档 1 已经可以排除"ror 瞎写假装翻译"——这是 ror 自动化能给出的最高保证。档 2/3 在 ror 下是"用户写的 Rocq 证明"，不再是"工具产物的属性"
- **不与 hax-lean 档 2/3 对等**：把 hax-lean 自动可达的档 2/3 与 ror 需手工证明的档 2/3 并列在报告里反而误导（看起来"两个工具都做了档 2/3"，但本质工作量差几个数量级）

如果未来要做 ror 的档 2/3，建议作为 **demo-level case study**（手工选 3-5 个示范 entry，写完整证明，作为 ror 应用场景的"证明可行性 walking-tour"），**不作为 testsuite 的自动测项**。

### §7.3 cc-reports 更新建议

`deep-reports/cc-reports/rocq-of-rust.md` 当前只写档 0（前端接受）。可补一节：

```
## 产物可运行性（2026-05-11 调研）

详见 `docs/research/ror-runnable-deep-dive-2026-05-11.md`。要点：
- 档 1（typecheck）：在隔离 ror-test opam switch（Rocq 9.0.0 + smpl + hammer + coqutil）下可达。
  9 个核心 runtime 文件秒级编译；4/4 实测产物（custom_add / basic_hello / int_wrapping / int_checked）
  独立 typecheck 通过。runner 当前不测此档；未来可作为 `tools/rocq-of-rust-typecheck/`
  独立 tool 加入
- 档 2 / 档 3（evaluate / 与 Rust 一致）：ror 设计上不自动支持。产物是 deep embedding
  `M = LowM.t (Value.t + Exception.t)`，所有 op 是 inductive constructor，`Compute` 不
  reduce 到 native 值。上游提供 `SimulateM.eval_f`，但要求用户手写 `Run.Trait` derivation
  + 纯 Rocq simulated 形式 + Lemma 证明。per-entry 工作量 5–50+ 行，递归 fn 需手工
  induction。**本 testsuite 不计划覆盖**
```

---

## §8 短期不可行点（档 2/3 specific）

阻塞档 2/3 自动化的硬约束（按重要性排）：

1. **deep embedding 的 Compute 语义缺失**：ror 产物用 `LowM.t` inductive ad-hoc constructor 表达 control flow（alloc / read / call_closure / call_primitive / let / loop / if-then-else），这些 constructor **不绑定 Coq-level fixpoint reduction**。这是工具**设计选择**，不是 bug。要让 `Compute` 解释这些 constructor，必须自写 evaluator —— 但**自写 evaluator 等于重写 SimulateM.eval**，且不能绕过 `Run.t` 是 propositional 的事实
2. **`Run.Trait` 需 per-entry 推导**：即便 `run_symbolic` 自动化大部分情况，递归函数 + 调外部 method 的情况都需手工 induction / 上游已写好的 method linker。这与 corpus 的多样性硬冲突
3. **simulated form 需手写**：要档 2/档 3 测有意义，必须有一个**与产物语义等价、可 Compute 的纯 Rocq 函数**。这个函数**ror 不自动生成**——必须用户写。对 corpus 规模这是不 scalable 的
4. **stdlib method 的 linker 依赖**：`int_wrapping` 用 `u8::wrapping_add`，要求 ror runtime 的 `core/num/links/*.v` 提供 `Run.Trait` for `wrapping_add` —— 这部分**已经在 runtime 里**（按 §4.2 的 1253/1509 比例覆盖了 core），但**走 `run_symbolic` 自动 lookup 这条路径是否 work** 还要实测；上层调研 §3.2.1 的 `int_wrapping` 翻译产物用 `M.get_associated_function`，需要在 link 阶段 resolve

**决策建议**：

- **不投入** ror 档 2/3 自动化——工作量与本 testsuite "特性覆盖广度筛选"任务严重不匹配
- **投入** ror 档 1 自动化（半天–一天）——补全 hax-lean 已有的"档 1 实测"在 ror 工具上的对等覆盖
- 已装的 ror-test switch（1.6 GB）**保留**（用作未来 ror 档 1 自动化的依赖）；若决定不做档 1 自动化也保留作为档 1 实测 reproducibility 的支持环境（`opam switch remove ror-test` 一行删除可逆）

