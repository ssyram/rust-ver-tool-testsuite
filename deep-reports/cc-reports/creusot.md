# creusot 深度报告

## 元数据

- **run**: `run-1778226613-5282`（2026-05-08T07:50:13Z → 08:16:08Z UTC，146 entries × 19 工具，host：Apple M5 / macOS aarch64 / 24 GB / 10 cpu，parallelism = 10）
- **工具版本**：`cargo-creusot 0.11.0`，Rust toolchain `nightly-2026-02-27`，Why3 `1.8.2+git`，why3find `1.3.0+dev`，alt-ergo `2.6.2`，z3 `4.15.4`，cvc4 `1.8`，cvc5 `1.3.1`
- **通过率**：106/146 = **72%**（FAILED 40 个，TIMEOUT 0，UNKNOWN 0）
- **时长（ms）**：avg 40047 / median 39972 / p90 52222 / max 64641
- **时效声明**：本快照锚定上述 run id + 工具版本 + corpus，不构成长期承诺。

## 工具内部 pipeline + 前端边界

cargo-creusot 通过把 `RUSTC` 替换为 `creusot-rustc` 跑整条 cargo 流水线：

```
cargo build → creusot-rustc（rustc + creusot translation passes）
            → 每个 fn 的 MIR 翻译为 .coma（Why3 IR）写盘到 verif/<crate>_rlib/
            → cargo-creusot 默认无 subcommand 时即退出
            （显式 cargo creusot prove 才进入 why3find → why3 → SMT solver）
```

本测试 `tool.toml` 仅传 binary 路径无 subcommand——天然停在 **Coma 翻译完成 / Why3 求解开始** 之间，等价于"前端 dry-run"。

cargo-creusot 在 manifest 解析阶段硬要求 `[dependencies]` 列出 `creusot-std`；creusot-rustc 在 crate 顶级 module 硬要求 `use creusot_std::prelude::*;`。`entry_mode = "lib"` harness 把例子原 `src/lib.rs` 改名 `src/__ts_inner.rs` 内嵌为 mod，新 `src/lib.rs` 包含两个硬性 import 满足上述 gate；`extra_cargo_deps = ['creusot-std = "0.11.0"']` 注入 dep 行。

## SUCCESS 信号 + 形式严格性

- **形式指标**：`cargo-creusot` exit 0
- **0 误报**：✅ 形式可证。exit 0 ⇔ 翻译完整无 rustc error
- **0 漏报**：✅ 形式可证。creusot 用 `crash_and_error / span_err / span_fatal / dcx().span_err` 把所有 unsupported 升级为 rustc error → exit 101，无 silent path
- **漏报盲点**：无

每个 SUCCESS 案例的 stderr 统一含 `warning: unused import: \`creusot_std::prelude::*\``——harness 的 prelude import 多数样例不实际触发，此 warning 不影响 exit。

## 实测结果

### 按 feature 类目分布

全 SUCCESS 类目（24 项）：`aeneas-limit / arc / assoc-type / box / closure / closure-adv (50%) / const / drop / enum / gat / generic / hax-limit / hello / hrtb / impl-trait / int / iter / panic / prusti-limit / rc / refcell / slice / trait / unsafe-adv / vec`。

部分通过：`bigint 7/8 · charon-limit 3/7 · closure-adv 2/4 · collections 1/2 · concurrency 1/2 · creusot-limit 3/7 · deps-complex 1/7 · float 5/10 · industrial 2/6 · int-width 13/14 · kani-limit 5/7 · lifetime 1/3 · miri-limit 5/7 · repr 1/2 · trait-obj 1/2`。

全 FAILED：`error 0/1`、`unsafe-ptr 0/2`。

按 feature 失败计数（前 5）：`deps-complex 6 · float 5 · industrial 4 · creusot-limit 4 · charon-limit 4`。

### 失败模式归类（基于 raw stderr 实读）

40 个 FAILED 全部 exit=101（cargo build 阶段），按 stderr 字面信号归 6 类：

**A. 显式 forbidden dyn type（7）**——`error: forbidden dyn type: dyn ...`：`closure-adv/boxed-dyn-fn`（`dyn Fn(i32)→i32`）、`trait-obj/dyn-dispatch`（用户 trait `Greeter`）、`lifetime/static-bound`（`dyn Any`）、`concurrency/thread-mutex`（`dyn Any + Send` from `JoinHandle`）、`miri-limit/thread-interleaving-partial`、`kani-limit/stack-unwinding/divide_with_recovery`（panic-recovery 路径上的 trait object）、`charon-limit/generic-to-dyn-unsize`。creusot-rustc 不区分内置 trait 或用户 trait，对 `dyn Trait` 走同一拒绝路径。

**B. Unsupported pointer cast / coercion（6）**——`unsafe-ptr/raw-ptr-const`（`43 as *const ()` / `PointerWithExposedProvenance`）、`int-width/cast-float-int` 与 `float/cast-int` 与 `float/cast-widening`（`IntToFloat`）、`closure-adv/early-bound-lifetime`、`creusot-limit/fn-ptr-reify`（`PointerCoercion(ReifyFnPointer(Safe), Implicit)`）。

**C. raw pointer dereference forbidden（1）**——`unsafe-ptr/raw-read`：`error: Dereference of a raw pointer is forbidden in creusot: use creusot_std::ghost::perm::Perm<*const T> instead`。错误信息主动给出替代方案，说明 creusot 对 raw deref 有显式 ghost-permission 模型。

**D. Unsupported constant value/expression（10）**——`Unsupported constant value: Scalar(allocN) of type &'?N [u8; M_usize]` 形态出现在所有 `industrial/*` 4 个失败 entry（rsa / sha2 上的 byte-string 字面量 `b"hello rsa"` / `b"the quick brown fox..."`）+ `creusot-limit/dyn-trait-forbidden`（`format!()` 展开后的 `&[u8; 2]`）；`Unsupported constant expression: "name"` 出现在 `deps-complex/*` 5 个 serde-derive 案例（`bigint-serde / chrono-serde / collections-serde / error-chain / trait-serde-generic`）。Industrial 与 deps-complex 的 stderr 共同形态：cargo 编译链能跑完依赖树（含 vendor 的 `sha2 v0.11.0`），但 creusot-rustc 进入用户代码翻译 byte-string / serde-derive 展开后失败。

**E. creusot-rustc 内部 panic（4）**——`internal error: entered unreachable code`：`repr/union/repr_union`（`creusot/src/backend/ty.rs:393:14`）、`charon-limit/async-fn` 与 `kani-limit/async-await`（`creusot/src/translation/specification.rs:321:61`）、`charon-limit/inline-asm`（`creusot/src/translation/function/terminator.rs:189:34: asm!("", options())`）。union / async fn / inline asm 三类构造分别触发不同 unreachable assertion。

**F. spec 层未覆盖的 std lib 路径（8）**——`error[E0277]: trait bound \`X: creusot_std::model::DeepModel\` not satisfied`（`bigint/bigint-arith`、`float/cmp` / `float/special-vals` / `float/total-order`、`deps-complex/itertools-multi`）、`IteratorSpec` 缺失（`collections/btreemap`、`error/result-question`、`creusot-limit/generic-for-loop`、`deps-complex/itertools-multi`）、`NaN is not yet supported`（`float/cmp` / `float/special-vals` / `float/total-order`）。`creusot_std` 提供的规约 trait 未覆盖 `f64` / `BigInt` / 部分 std iterator——构成 lib spec 边界。

**G. 其他显式拒（4）**——`charon-limit/arc-slice-unsize`：`error: unsupported cast from Arc<[u32; 3]> to Arc<[u32]>`；`lifetime/thread-local` / `creusot-limit/thread-local-ref`：`thread_local!` 宏展开内 `dyn Any + Send` 路径；`miri-limit/weak-memory-incomplete`：`error: unsupported definition kind ... Static { safety: Safe, mutability: Not, nested: false }`（顶级 `static SHARED: AtomicU32 = ...`）。

每一类的 stderr 形态都是 creusot 内部某条 translation pass 上的显式 reject——`crash_and_error`/`span_err` 路径——而非"fallback to stock rustc"。

## 与本次测试边界的关系

`tool.toml` 仅传 binary 无 subcommand → SUCCESS = creusot-rustc 完成 Coma 翻译，**不蕴含 SMT 求解结论**。本矩阵在 `cargo creusot prove` 调用 Why3 / SMT solver 这条路径上没有任何观察。每个 SUCCESS entry 上 stderr 普遍含 `warning: calling external function X with no contract will yield an impossible precondition`——表示 creusot-rustc 已接受代码并打算翻译，只是某些外部函数无 contract；这条 **warning 本身不导致 FAILED**，是 creusot 接受范围内状态。

`industrial/*` 中 vendor 的 `sha2 v0.11.0` 与 `rsa` 全树能编通（含 `digest` / `crypto-common` / `block-buffer` 等链），失败点落在用户 entry 代码的 byte-string 字面量翻译——意味着 creusot 接受了 vendor 的 trait/struct 定义，是入口处的具体常量构造触发 D 类边界。

## 历史快照声明

本报告记录的所有数字、reject 形态、panic 位置锚定 run `run-1778226613-5282` + cargo-creusot 0.11.0 + nightly-2026-02-27 + creusot-std 0.11.0。creusot 升级后 `forbidden dyn` / `Unsupported pointer cast` / `Unsupported constant` / `DeepModel`/`IteratorSpec` 缺失等可能改写。
