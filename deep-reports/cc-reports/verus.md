# verus — 特性支持评估报告

## 元数据

- **数据源**：`runs/run-1778226613-5282/`（2026-05-08，146 entries × 19 工具矩阵；host: Apple M5 / macOS aarch64 / 24 GB / 10 cpus，并发 10）
- **工具版本**：`Verus 0.2026.05.03.8b81855`，profile release，platform macos_aarch64，Rust toolchain `1.95.0-aarch64-apple-darwin`
- **工具配置**：`tools/verus/`
- **通过率**：SUCCESS 51 / 146 ≈ **34.9%**（FAILED 95，TIMEOUT 0）
- **耗时分布**：avg 563 ms / median 514 ms / p90 909 ms / p95 1113 ms / max 2312 ms（亚秒级，时间上限 120 s 远未触达）
- **时效声明**：本快照锚定上述 run id + verus 0.2026.05.03 release binary（内置 vstd v0.2026.05.03）+ corpus，不构成长期承诺。verus 上游对 vstd 持续扩充 spec 覆盖，未来版本对 std lib API 的接受面会扩大、对 `--no-verify` 路径下的内部 panic site 会修复——本快照随之失效。

## 工具内部 pipeline + 前端边界

```
verus binary
  → rustc fork（verus-driver）解析 + macro 扩展
  → verus! { } 宏块识别
  → AST → VIR（Verus Intermediate Representation）
  → mode check / lifetime check / type check / vstd 链接 check
  → [--no-verify cut here]
  → simplified VIR → AIR (Verus 自家 SMT-LIB 前置形式)
  → SMT-LIB → Z3
```

本测试关心 `verus --no-verify --log vir --crate-type=lib src/lib.rs` 这条命令的"前端通过"。

**前端 / 后端切割**：理想上应落在 **VIR/AIR 构造**与 **Z3 SMT 求解**之间。但 verus 的 `--no-verify` flag **同时切断 AIR 构造与 Z3 调用**（all-or-nothing），无 `--no-z3` / `--rlimit 0` 独立切线。所以本工具能拿到的最深前端表示是 **VIR**，而非 AIR/SMT-LIB。AIR ≈ SMT-LIB 的前置形式，按精神 AIR 算后端，所以 VIR 即为前端边界。`--log vir` 把 VIR 写到 `.verus-log/crate.vir`（每个 fn 在 VIR 里以 `(Function :name ... :body ...)` 存在），便于函数级对应性校验。

`tool.toml` 直接调用 verus binary、不走 `cargo-verus`——`cargo-verus` 0.2026.05.03 与 crates.io vstd 任何版本均不兼容，会 panic 于 `rust_verify/src/erase.rs:405`（`Option::unwrap()` on `None`）。verus binary 内置 `libvstd.rlib` / `libverus_builtin.rlib`，不需要重新编译规约库——这就是亚秒级耗时的成因。

## SUCCESS 信号 + 形式严格性

**判定式**：

```
SUCCESS ⟺ verus exit 0 （即 Verus 前端完成 type/lifetime/mode check + VIR 构造）
```

**反作弊核查**（宪法 §六-4）：

`harness.rs.tera` 内容：

```rust
use vstd::prelude::*;

verus! {
    mod __ts_inner;          // ← 必须在 verus! {} 块内
    pub use __ts_inner::*;

    #[allow(dead_code)]
    #[verifier::external]
    fn __ts_invoke() {
        __ts_inner::{{ entry_fn }}();
    }
}
```

`mod __ts_inner;` 在 `verus! { }` 块内，且**不带** `#[verifier::external]`——这意味着 `__ts_inner` 内的所有 item 都被 verus 前端**逐项检查**，不会被透传给 stock rustc。`#[verifier::external]` 仅给 `__ts_invoke` 这层壳函数，告诉 verus "这个 wrapper 不要 verify"。配置已确认满足反作弊要求：本矩阵 35% SUCCESS 与同矩阵 cargo-check 的接近 100% 通过率之间的显著分化，正是 verus 前端对 Rust 子集真实接受范围的反映；如果 `mod __ts_inner` 写在块外，inner 直接被 stock rustc 透传，SUCCESS 信号会退化成"rustc parses it"，与 cargo-check 同质化、丢掉 verus 的特性子集分化信号。

**形式严格性**：
- **0 误报**：✅ 形式可证。verus exit 0 ⇔ VIR 构造完成且 verus 前端无错误（任何 rejection → `dcx().emit` → exit ≠ 0）
- **0 漏报**：✅ 形式可证。verus 任何 rejection（lifetime / type check / `assume_specification` 缺失 / `verus_builtin not imported` 等）都通过 `dcx().emit` 触发 exit ≠ 0
- **漏报盲点**：无（注：`--no-verify` 同时切 AIR + Z3，最深前端到 VIR——所以"前端边界 = VIR 构造完成"）

## 实测结果

### 按 feature 类目分布

下列 feature 类目下 verus **全部 entry 通过**（即 verus 前端完成 type/lifetime/mode check + VIR 构造、exit 0）：

```
arc / box / enum / hello / hrtb(1/1) / int / vec
```

**部分通过**（数字为 S/总）：`aeneas-limit` 3/8、`charon-limit` 4/7、`closure-adv` 1/4、`concurrency` 1/2、`creusot-limit` 3/7、`float` 1/10、`generic` 2/4、`hax-limit` 3/8、`int-width` 7/14、`kani-limit` 3/7、`lifetime` 1/3、`miri-limit` 2/7、`panic` 1/2、`prusti-limit` 7/8、`repr` 1/2、`trait-obj` 1/2。

**全 FAILED**：`assoc-type`、`bigint`(0/8)、`closure`、`collections`、`const`、`deps-complex`(0/7)、`drop`、`error`、`gat`、`impl-trait`、`industrial`(0/6)、`iter`、`rc`、`refcell`、`slice`、`trait`、`unsafe-adv`、`unsafe-ptr`。

### 失败模式归类（基于 raw stderr）

95 条 FAILED 按 raw stderr 字面信号分桶：

| 桶 | 数量 | 字面信号 / 含义 |
|---|---:|---|
| **A. vstd 没给 std lib API 挂 spec**（verus 自声明的 vstd 边界） | 29 | `error: \`<core::...>\` is not supported (note: you may be able to add a Verus specification to this function with \`assume_specification\`) (note: the vstd library provides some specification for the Rust std library, but it is currently limited)`，stderr 总附 `help: pub assume_specification ...` 给出可补的形态 |
| **B. verus 语言子集还不支持的 Rust 构造**（verus 自声明的语言边界） | 19 | `error: The verifier does not yet support the following Rust feature: <X>`，X 见下 |
| **C. 输入流水线层：外部 crate 无法 resolve** | 19 | `error[E0432]: unresolved import \`num_bigint\`` / `serde` / `chrono` / `sha2` 等。verus binary 不读 `Cargo.toml`，外部 crate 无 rmeta 可链接——这不是 verus 自身能力边界 |
| **D. verus 内部 panic（index out of bounds）** | 12 | `thread 'rustc' panicked at /rustc-dev/.../ty/generic_args.rs:54:14: index out of bounds: the len is 0 but the index is 0` |
| **E. verus-driver MIR 构造 panic** | 7 | `panicked at rustc_mir_build/src/../../rustc_mir_build_additional_files/verus.rs:176:21` 或 `rust_verify/src/rust_to_vir_base.rs:749:14`（async fn / thread_local! / `*const T` 解引用 const 路径触发） |
| **F. closure 捕获 `&mut`** | 2 | `error: Verus does not currently support closures capturing a mutable reference for variables of any mode`：`closure/fn-fnmut/{closure_fn,closure_fnmut}` |
| **G. edition / nightly feature 默认（输入流水线层）** | 1 | `hax-limit/let-chains`：`error: let chains are only allowed in Rust 2024 or later` |
| **H. industrial vendor crate edition 2024 拦截** | 6 | `industrial/{x509-parser/*, rsa/*, sha2/*}`：vendor crate Cargo.toml 写 `edition = "2024"` 或依赖含 edition 2024 的子 crate（`base64ct` 等），但 verus 直接调 `verus src/lib.rs`，连 cargo 都没启动；本桶实际通过 C 桶（unresolved import）形态触发——industrial 模块在 verus 上 0/6 |

数量加总（一个 entry 计入主导桶）：A29 + B19 + C19 + D12 + E7 + F2 + G1 + 其他（含 H、I 等剩余）= 95（其中 I 类杂项 ≈ 6：`creusot-limit/dyn-trait-forbidden` 的 `Unsupported constant type`、`kani-limit/stack-unwinding` 的 `Verus does not recognize this trait bound`、`lifetime/static-bound` 的 `trait core::any::Any not declared to Verus`、`aeneas-limit/mutually-recursive-traits` 的 `cyclic self-reference` 等单点拒绝）

### A 桶（vstd 边界）的 std API 类别分布

raw stderr 显示的具体被拒 API：

- **数值方法**：`core::num::<impl iN>::wrapping_div / wrapping_abs / saturating_mul / overflowing_sub / wrapping_shl / rotate_right / unchecked_add` 等（覆盖 `int-width/{wrapping-i64, saturating-u16, shift-overflow, arith-i128, arith-i8, bit-ops-u32, overflowing-i32}` 7 条）
- **浮点常量与函数**：`core::f64::NEG_INFINITY / NAN / INFINITY / from_bits / is_nan`、`std::f64::floor / cos`（覆盖 `float/{cmp, nan-prop, special-vals, transcendental, bits, cast-int, cast-widening, round}` 等多条）
- **内存原语**：`core::intrinsics::transmute`、`core::mem::drop`、`core::mem::maybe_uninit::write`
- **容器与迭代器**：`core::option::copied`、`core::iter::adapters::map::Map`、`core::iter::range::Filter::default`、`core::str::iter::Split`
- **字符串**：`alloc::string::From`（`String::from` 等）、`core::fmt::from_str`
- **panic**：`core::panicking::panic_fmt` + `core::fmt::Arguments`
- **hint 与原子**：`core::hint::black_box`、`core::slice::get_unchecked`、`std::io::error::Error`

stderr 自报 "the vstd library provides some specification for the Rust std library, but it is currently limited"——这是 verus 自己声明的边界。

### B 桶（语言子集未支持）的 X 清单

- `bitwise AND/OR for bools (i.e., the not-short-circuited version)`：`aeneas-limit/bool-bitwise-op`
- `internal item statements`：6 条，`assoc-type/iter-style`、`drop/custom-drop`、`gat/lending-iter`、`generic/pair-struct`、`repr/union/repr_union`、`trait/cyclic-bound`、`trait-obj/conditional-method`——指 fn body 内嵌 trait/impl/union 定义的 HIR stmt-Item
- `dyn with more that one trait`：`closure-adv/boxed-dyn-fn`、`miri-limit/thread-interleaving-partial`
- `inline-asm expressions`：`charon-limit/inline-asm`
- `unsizing operation from \`std::sync::Arc<[u32; 3]>\` to \`std::sync::Arc<[u32]>\``：`charon-limit/arc-slice-unsize`
- `function pointer types`：`creusot-limit/fn-ptr-reify`
- `dereferencing a pointer (here the dereference is implicit)`：`unsafe-adv/ptr-write`、`unsafe-ptr/raw-read`
- `deref_mut for std::cell::RefMut<...> not yet supported`：`refcell/borrow`
- `only variables are supported here, not general patterns`：`aeneas-limit/{fnmut-closure-unit-return}`、`closure-adv/early-bound-lifetime`、`hax-limit/mut-arg-pattern`

### D 桶（panic at `generic_args:54`）entry 列表

```
aeneas-limit/{nested-borrow-array, return-inside-nested-loop}
collections/btreemap
concurrency/thread-mutex
creusot-limit/generic-for-loop
generic/sum-bound
hax-limit/{closure-mutates-outer, labelled-break}
kani-limit/loop-unwinding
miri-limit/simd-bitmask-large-vector
prusti-limit/for-loop-iterator
slice/index-iter
```

形态全部相同：`thread 'rustc' panicked at /rustc-dev/.../compiler/rustc_middle/src/ty/generic_args.rs:54:14: index out of bounds: the len is 0 but the index is 0`——同一 panic site 触发条件不同，意味着这是 verus-driver 在 `--no-verify` 模式下某条共用代码路径的边界条件。本测试把它们记为 FAILED（verus exit 1）。

## 与本次测试边界的关系

- 测试切割点：verus exit 0 = VIR 构造完成无错误。**未触达**：AIR 构造、SMT-LIB 生成、Z3 求解——这些在 `--no-verify` 下完全不跑
- C 桶 19 条（外部 crate unresolved）严格说不是 verus 自身的"特性接受"层面边界——verus binary 不通过 cargo 工作的输入流水线导致所有需要外部 crate 的 entry 在 import resolution 阶段就被拒。`bigint/*` 8 条、`deps-complex/*` 7 条、`industrial/*` 6 条都死在这层
- A 桶 29 条 + B 桶 19 条 = 48 条**严格反映 verus 前端对 Rust 子集的真实接受边界**——A 是 vstd spec 边界、B 是 verus 语言子集语义边界，都是 verus 项目自陈的"yet to support"
- D 桶 12 条 + E 桶 7 条 = 19 条 **verus 内部 panic** 不属于 verus 设计内的 reject 路径，但本测试按 exit code 一律记 FAILED——这是宪法 §六-2"不允许 partial"的延伸：工具内部异常等同于工具未完成处理

## 历史快照声明

本报告所有数字基于 `runs/run-1778226613-5282`（2026-05-08）+ verus 0.2026.05.03.8b81855 + Rust toolchain 1.95.0。verus 上游每次 release 对 vstd spec 覆盖率提升、内部 panic site 修复、新语言构造支持，都会让 A/B/D/E 桶数字漂移——届时本快照随之失效，需要在新 run 上重测。
