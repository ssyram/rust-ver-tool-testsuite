# 特性覆盖度报告 — 19 工具 × 140 entries（2026-05-07）

旧报告（8 工具版）保留在 `feature-coverage-2026-05-07.md`。本报告覆盖完整 19 工具矩阵，并把 `*-limit` 类目展开成具体 Rust 特性。

run 标识：`run-1778148197-53283`
原始数据：`runs/run-1778148197-53283/`（results.json + 每 task raw stderr/stdout）

## 一、元数据

| 字段 | 值 |
|---|---|
| 起止 | 2026-05-07T10:03:17Z – 10:22:24Z (UTC) |
| 总耗时 | 19m 7s wall |
| Host | ssyramdeMacBook-Air.local / macos / aarch64 / Apple M5 / 24576 MB / 10 cores |
| 并发 | 10 |
| 工具数 | 19 |
| Entry 数 | 140 |
| 总 task 数 | 2660 |

## 二、整体数字

| 状态 | 数 | 占比 |
|---|---:|---:|
| SUCCESS | 1826 | 68% |
| FAILED  | 834  | 31% |
| UNKNOWN | 0    | 0%  |
| TIMEOUT | 0    | 0%  |

无 runner-internal 故障，无 task 超时。

## 三、工具维度

按 SUCCESS rate 排序（n=140 entries each）。时长字段仅作环境上下文记录，不作工具评分使用。

| tool | S | F | rate | avg_ms | p50 | p90 | max |
|---|---:|---:|---:|---:|---:|---:|---:|
| charon-poly  | 133 | 7  | 95% | 2200 | 407 | 7632 | 30165 |
| charon-mono  | 133 | 7  | 95% | 1627 | 397 | 5244 | 22933 |
| hax-lean     | 126 | 14 | 90% | 3072 | 1727 | 6677 | 27925 |
| rocq-of-rust | 122 | 18 | 87% | 127  | 95  | 291  | 517 |
| verifast     | 116 | 24 | 82% | 251  | 207 | 480  | 781 |
| hax-fstar    | 111 | 29 | 79% | 2553 | 1447 | 5705 | 20191 |
| soteria      | 100 | 40 | 71% | 1641 | 864 | 3481 | 24451 |
| creusot      |  96 | 44 | 68% | 37289 | 36952 | 47555 | 57392 |
| prusti       |  94 | 46 | 67% | 3139 | 959 | 9184 | 42193 |
| miri         |  94 | 46 | 67% | 2180 | 578 | 5703 | 19718 |
| kani         |  95 | 45 | 67% | 2933 | 992 | 6167 | 53932 |
| hax-coq      |  94 | 46 | 67% | 3088 | 1786 | 5780 | 31070 |
| cargo-check  |  95 | 45 | 67% | 1341 | 219 | 4159 | 18476 |
| aeneas-lean  |  85 | 55 | 60% | 3411 | 1751 | 6683 | 34948 |
| aeneas-fstar |  85 | 55 | 60% | 3161 | 1772 | 6559 | 25291 |
| aeneas-coq   |  85 | 55 | 60% | 3431 | 1760 | 6319 | 31933 |
| kmir         |  63 | 77 | 45% | 5614 | 4612 | 10300 | 81402 |
| aeneas-hol4  |  52 | 88 | 37% | 3331 | 1678 | 7142 | 34973 |
| verus        |  47 | 93 | 33% | 540  | 520 | 850  | 1602 |

## 四、基础特性维度（非工具自声明限制）

| 特性 | tasks | S | F | rate |
|---|---:|---:|---:|---:|
| hello | 19 | 19 | 0 | 100% |
| int | 38 | 38 | 0 | 100% |
| vec | 19 | 19 | 0 | 100% |
| panic | 38 | 37 | 1 | 97% |
| int-width（i8..i128/u8..u128） | 266 | 254 | 12 | 95% |
| generic | 76 | 72 | 4 | 94% |
| drop | 19 | 18 | 1 | 94% |
| const | 19 | 18 | 1 | 94% |
| slice | 19 | 17 | 2 | 89% |
| rc | 19 | 17 | 2 | 89% |
| impl-trait | 19 | 17 | 2 | 89% |
| arc | 19 | 17 | 2 | 89% |
| refcell | 19 | 16 | 3 | 84% |
| enum | 38 | 32 | 6 | 84% |
| box | 38 | 32 | 6 | 84% |
| unsafe-adv（MaybeUninit / transmute / ptr-write） | 57 | 45 | 12 | 78% |
| closure | 38 | 30 | 8 | 78% |
| assoc-type | 19 | 15 | 4 | 78% |
| closure-adv（FnOnce / 返回闭包 / boxed dyn Fn） | 76 | 57 | 19 | 75% |
| repr | 38 | 28 | 10 | 73% |
| iter | 19 | 14 | 5 | 73% |
| hrtb | 19 | 14 | 5 | 73% |
| concurrency（Arc<Mutex>、Channel） | 38 | 27 | 11 | 71% |
| collections（HashMap / BTreeMap） | 38 | 27 | 11 | 71% |
| float（f32/f64 算术 / 方法） | 190 | 134 | 56 | 70% |
| trait-obj | 38 | 26 | 12 | 68% |
| error（`?` / Result chain） | 19 | 13 | 6 | 68% |
| bigint（num-bigint / num-rational / num-integer / num-traits / num-complex） | 152 | 100 | 52 | 65% |
| lifetime | 57 | 35 | 22 | 61% |
| gat | 19 | 11 | 8 | 57% |
| unsafe-ptr（裸指针读 / 裸指针 const match） | 38 | 20 | 18 | 52% |
| deps-complex（chrono / serde / itertools / error-chain / 多 dep 组合） | 133 | 65 | 68 | 48% |
| trait | 19 | 9 | 10 | 47% |

## 五、跨工具特性边界详解

把 `*-limit` 类目展开成 24 个具体 Rust 特性主题。每个主题给：(a) 工具自声明的限制内容（来自 entry 的 doc comment）；(b) 跨工具实测分布（哪些工具 SUCCESS / FAILED）。"FAILED" 不评对错——本测试只看"工具是否接受这段代码"。

### 主题 1: `async fn` / coroutine 状态机

**触发样例 1.1**：`charon-limit/async-fn/async_forty_two`

> charon 自声明：`async fn` 被 rustc 编为 coroutines（state-machine types），charon 在 `translate_types.rs` 报 `"Coroutine types are not supported yet"`，在 `translate_bodies.rs` 报 `AggregateKind::Coroutine(..) raise_error!"Coroutines are not supported"`。

| 状态 | 工具 |
|---|---|
| SUCCESS (7) | cargo-check, hax-lean, kani, kmir, miri, prusti, verus |
| FAILED (12) | aeneas-coq, aeneas-fstar, aeneas-hol4, aeneas-lean, charon-mono, charon-poly, creusot, hax-coq, hax-fstar, rocq-of-rust, soteria, verifast |

**触发样例 1.2**：`kani-limit/async-await/run_async_add`

> kani 自声明：concurrent features (await expressions) 不在 kani scope；kani 编译时把 await 当顺序执行，可能 unsound（feature support 文档）。

| 状态 | 工具 |
|---|---|
| SUCCESS (1) | hax-lean |
| FAILED (18) | 其余全部 |

aeneas 4 backend、charon 双 mode、creusot、hax-coq/fstar、rocq-of-rust、soteria、verifast、verus 都拒收 async；cargo-check 在 1.1 通过 / 在 1.2 拒收（edition 差异）。

### 主题 2: 内联汇编 `asm!`

**触发样例 2.1**：`charon-limit/inline-asm/nop_via_asm`

> charon 自声明：`TerminatorKind::InlineAsm => raise_error!("Inline assembly is not supported")`，每个 asm! 调用 hard error。

| 状态 | 工具 |
|---|---|
| SUCCESS (11) | aeneas-coq, aeneas-fstar, aeneas-hol4, aeneas-lean, cargo-check, hax-lean, kani, kmir, prusti, rocq-of-rust, verifast |
| FAILED (8) | charon-mono, charon-poly, creusot, hax-coq, hax-fstar, miri, soteria, verus |

**触发样例 2.2**：`creusot-limit/inline-asm-basic/nop_via_asm`

> creusot 自声明：MIR `InlineAsm` terminator 在 creusot terminator translation 里标 `unreachable!`，含 `asm!` 函数会 panic 编译插件。

| 状态 | 工具 |
|---|---|
| SUCCESS (14) | aeneas × 4, charon × 2, creusot, hax × 3, rocq-of-rust, soteria, verifast, verus |
| FAILED (5) | cargo-check, kani, kmir, miri, prusti |

**触发样例 2.3**：`kani-limit/inline-assembly/add_via_asm`

> kani 自声明：不支持 inline 与 global asm（feature support 文档；issues #2, #316）。

| 状态 | 工具 |
|---|---|
| SUCCESS (11) | aeneas × 4, charon × 2, hax × 3, rocq-of-rust, verifast |
| FAILED (8) | cargo-check, creusot, kani, kmir, miri, prusti, soteria, verus |

**触发样例 2.4**：`miri-limit/inline-asm/add_via_inline_asm`

> miri 自声明：仅支持极少 AVX-512 intrinsics；任意 asm 块若指令无 interpreter shim 即 `unsupported operation: inline assembly is not supported`。

| 状态 | 工具 |
|---|---|
| SUCCESS (11) | aeneas × 4, charon × 2, hax × 3, rocq-of-rust, verifast |
| FAILED (8) | cargo-check, creusot, kani, kmir, miri, prusti, soteria, verus |

**注意**：同样的 `asm!` 块在 4 个 entry 上的工具支持率分布各不相同——同一 Rust 特性在不同上下文（被三方测试触发模式不同）下结果可分化。

### 主题 3: FFI 调用外部 C 函数

**触发样例 3.1**：`kani-limit/extern-ffi/call_libc_abs`

> kani 自声明：FFI 调用没有 Rust body 给 kani 分析，被当作 uninterpreted/任意值，依赖 FFI 函数后置条件的证明 unsound（undefined-behaviour 文档）。

| 状态 | 工具 |
|---|---|
| SUCCESS (10) | aeneas × 4, charon × 2, hax-fstar, hax-lean, rocq-of-rust, verifast |
| FAILED (9) | cargo-check, creusot, hax-coq, kani, kmir, miri, prusti, soteria, verus |

**触发样例 3.2**：`miri-limit/ffi-unshimmed-extern/call_unshimmed_foreign_fn`

> miri 自声明：跨平台 interpreter，无 shim 的 foreign function 调用即 `unsupported operation: can't call foreign function: bind`。

| 状态 | 工具 |
|---|---|
| SUCCESS (12) | aeneas × 4, charon × 2, creusot, hax-fstar, hax-lean, rocq-of-rust, verifast, verus |
| FAILED (7) | cargo-check, hax-coq, kani, kmir, miri, prusti, soteria |

### 主题 4: 浮点

**触发样例 4.1**：`aeneas-limit/float-types/make_measurement`

> aeneas 自声明：f32/f64 不支持。函数体含浮点字面量或运算时报 `[Error] Improperly typed constant value`（字面量）或 `[Error] unsupported floats`（算术）。

| 状态 | 工具 |
|---|---|
| SUCCESS (9) | charon × 2, creusot, hax × 3, rocq-of-rust, soteria, verus |
| FAILED (10) | aeneas × 4, cargo-check, kani, kmir, miri, prusti, verifast |

**触发样例 4.2**：`kani-limit/float-overapprox/check_sin_cos_identity`

> kani 自声明：transcendental 浮点运算（sinf32/cosf32/sqrt/log/pow 等）over-approximation，导致 spurious 验证失败（intrinsics 文档）。

| 状态 | 工具 |
|---|---|
| SUCCESS (7) | charon × 2, hax × 3, rocq-of-rust, verifast |
| FAILED (12) | aeneas × 4, cargo-check, creusot, kani, kmir, miri, prusti, soteria, verus |

更基础的 `float` 特性（含 `int-to-float` / `float-arith` / `f32/f64-methods` 等 10 个 entries × 19 工具 = 190 task）整体 70% SUCCESS——参见第四节。

### 主题 5: `&mut` 引用的复杂用法

hax 自声明它的 supported subset 把多类 `&mut` 用法排除。本节聚集 hax + prusti + aeneas 的相关边界。

**5.1 `&mut T` 作为返回类型 (`hax-limit/ret-mut-ref`)**

> hax error HAX0003 (UnallowedMutRef)；README "Supported Subset" 写明 mutable references on return types or when aliasing are forbidden（issue #420）。

S(11): aeneas × 3 (lean/coq/fstar), charon × 2, creusot, hax-lean, rocq-of-rust, soteria, verifast, verus
F(8): aeneas-hol4, cargo-check, hax-coq, hax-fstar, kani, kmir, miri, prusti

**5.2 `&mut T` 别名 (`hax-limit/mut-ref-alias`)**

> hax error HAX0003：禁止 `let y = x;` 当 `x: &mut u8`（issue #420）。

S(12): aeneas × 4, charon × 2, creusot, hax-lean, rocq-of-rust, soteria, verifast, verus
F(7): cargo-check, hax-coq, hax-fstar, kani, kmir, miri, prusti

**5.3 `&mut T` 出现在 associated type (`hax-limit/mut-in-assoc-type`)**

> hax error HAX0003 (DirectAndMut) 在 associated type 定义点（issue #1674）。

S(11): aeneas-coq/fstar/lean, charon × 2, creusot, hax-lean, rocq-of-rust, soteria, verifast, verus
F(8): aeneas-hol4, cargo-check, hax-coq, hax-fstar, kani, kmir, miri, prusti

**5.4 `&mut T` 在函数参数 pattern 解构 (`hax-limit/mut-arg-pattern`)**

> hax error HAX0011 (NonTrivialAndMutFnInput)：`f((x, y): &mut (T, U))` 不允许，仅 trivial pattern。

S(11): aeneas × 4, charon × 2, creusot, hax-lean, rocq-of-rust, soteria, verifast
F(8): cargo-check, hax-coq, hax-fstar, kani, kmir, miri, prusti, verus

**5.5 闭包修改外部 `&mut` 捕获 (`hax-limit/closure-mutates-outer`)**

> hax error HAX0006 (ClosureMutatesParentBindings)。

S(10): aeneas-coq/fstar/lean, charon × 2, creusot, hax-lean, rocq-of-rust, soteria, verifast
F(9): aeneas-hol4, cargo-check, hax-coq, hax-fstar, kani, kmir, miri, prusti, verus

**5.6 借用穿越 loop boundary (`prusti-limit/loan-crosses-loop-boundary`)**

> prusti user guide loop 章节："Loans that cross a loop boundary | Not supported yet"。

S(12): aeneas × 3, charon × 2, creusot, hax-fstar, hax-lean, rocq-of-rust, soteria, verifast, verus
F(7): aeneas-hol4, cargo-check, hax-coq, kani, kmir, miri, prusti

**5.7 含 reference 字段的 struct (`prusti-limit/ref-typed-struct-field`)**

> prusti: `[unsupported feature] access to reference-typed fields is not supported`。

S(12): aeneas × 3, charon × 2, creusot, hax × 3, rocq-of-rust, soteria, verus
F(7): aeneas-hol4, cargo-check, kani, kmir, miri, prusti, verifast

**5.8 match guard 引发的 shallow borrow (`prusti-limit/shallow-borrow-match-guard`)**

> prusti: `[unsupported feature] unsupported creation of shallow borrows (implicitly created when lowering matches)`。

S(10): charon × 2, creusot, hax × 3, rocq-of-rust, soteria, verifast, verus
F(9): aeneas × 4, cargo-check, kani, kmir, miri, prusti

**5.9 数组中嵌套借用 (`aeneas-limit/nested-borrow-array`)**

> aeneas: `[Error] Found a case of unsupported nested borrows`——`arr[i]` 当 `arr: [&T; N]`。

S(9): charon × 2, creusot, hax × 3, rocq-of-rust, soteria, verifast
F(10): aeneas × 4, cargo-check, kani, kmir, miri, prusti, verus

**5.10 泛型 trait 用 `&mut T` 实例化 (`aeneas-limit/trait-impl-mut-param-mismatch`)**

> aeneas: 抽出的 Lean trait 与 impl 在 return type 不一致；trait 给 `Result Unit`、impl 给 `Result Std.U8`，因为 aeneas 给 mutable reference 生成 backward (loan-giving) return value。

S(11): aeneas-coq/fstar/lean, charon × 2, creusot, hax-lean, rocq-of-rust, soteria, verifast, verus
F(8): aeneas-hol4, cargo-check, hax-coq, hax-fstar, kani, kmir, miri, prusti

### 主题 6: 闭包高阶用法

**6.1 闭包捕获 + 内含 if (`aeneas-limit/closure-if-capture`)**

> aeneas: 闭包既捕获外部变量、body 又含 if-then-else 时报 `[Error] Unimplemented`；任一因素单独都行。

S(10): charon × 2, creusot, hax × 3, rocq-of-rust, soteria, verifast, verus
F(9): aeneas × 4, cargo-check, kani, kmir, miri, prusti

**6.2 FnMut 闭包返回 `()` (`aeneas-limit/fnmut-closure-unit-return`)**

> aeneas 自声明 codegen bug：output 是 `()` 时生成 `let p1 <- call_mut p 0`（错，丢 updated closure），应该是 `let (_, p1) <- call_mut p 0`。

S(9): aeneas-coq/fstar/lean, charon × 2, creusot, rocq-of-rust, soteria, verifast
F(10): aeneas-hol4, cargo-check, hax × 3, kani, kmir, miri, prusti, verus

**6.3 闭包内解引用 Copy 引用 (`charon-limit/copy-deref-closure`)**

> charon issue #1120：闭包捕获 `&T`（`T: Copy`）后 body 解引用 `*x` 在 charon post-transformation type-check 失败。

S(18): 全部，除 aeneas-hol4
F(1): aeneas-hol4

**6.4 闭包基本不支持 (`prusti-limit/closures-unsupported`)**

> prusti: `[unsupported feature] this is unsupported, because it uses closures`（issue #169 / PR #138）。

S(13): aeneas-coq/fstar/lean, charon × 2, creusot, hax × 3, rocq-of-rust, soteria, verifast, verus
F(6): aeneas-hol4, cargo-check, kani, kmir, miri, prusti

**6.5 `#[pure]` fn 内闭包 (`prusti-limit/closure-in-pure-fn`)**

> prusti：`use of impure function "std::ops::Fn::call" in pure code is not allowed` + `unsupported constant type ... Closure(...)`。

S(13): aeneas-coq/fstar/lean, charon × 2, creusot, hax × 3, rocq-of-rust, soteria, verifast, verus
F(6): aeneas-hol4, cargo-check, kani, kmir, miri, prusti

### 主题 7: 控制流（label / let-chains / nested loop）

**7.1 标签 break / continue (`hax-limit/labelled-break`)**

> hax issue #1799："break or continue to labelled blocks or loops" / #1800：标签不被 honor。

S(8): charon × 2, creusot, hax-fstar, hax-lean, rocq-of-rust, soteria, verifast
F(11): aeneas × 4, cargo-check, hax-coq, kani, kmir, miri, prusti, verus

**7.2 Let chains `if let ... && let ...` (`hax-limit/let-chains`)**

> hax issue #2018："[Unsupported Rust] Let chains"（RFC 2497；Rust 1.88 edition 2024 stable）。

S(8): aeneas × 4, charon × 2, creusot, hax-lean
F(11): cargo-check, hax-coq, hax-fstar, kani, kmir, miri, prusti, rocq-of-rust, soteria, verifast, verus

注：cargo-check 失败因测试 toolchain 默认 edition 不开 let-chains。`feature(let_chains)` 在多数工具下未启。

**7.3 嵌套 loop 内 `return` / outer break (`aeneas-limit/return-inside-nested-loop`)**

> aeneas: `'a: loop { loop { break 'a; } }` 不支持；symbolic interpreter 遇 early-exit edge 报 `[Error] Unreachable`。

S(11): aeneas × 4, charon × 2, creusot, hax-lean, rocq-of-rust, soteria, verifast
F(8): cargo-check, hax-coq, hax-fstar, kani, kmir, miri, prusti, verus

### 主题 8: trait object / unsizing

**8.1 dyn Trait 大部分禁 (`creusot-limit/dyn-trait-forbidden`)**

> creusot：仅 dyn Debug / dyn Write "logically dyn-compatible"；其他 trait object 在 translation time 触发 `forbidden dyn type`（issue #2071）。

S(5): charon × 2, hax-fstar, rocq-of-rust, verifast
F(14): 其余全部

**8.2 `Arc<[T;N]>` → `Arc<[T]>` 智能指针 unsizing (`charon-limit/arc-slice-unsize`)**

> charon issue #855：unsizing through custom smart pointers (`CoerceUnsized`) 不支持。

S(11): cargo-check, charon × 2, hax × 3, kani, miri, prusti, rocq-of-rust, verifast
F(8): aeneas × 4, creusot, kmir, soteria, verus

**8.3 泛型 → dyn unsize (`charon-limit/generic-to-dyn-unsize`)**

> charon test suite 中泛型 `Box<T>` (`T: Trait`) → `Box<dyn Trait>` coercion 全用 `#[charon::opaque]` 标记，注释 "Opaque because we don't support unsize coercions"。

S(10): cargo-check, charon-poly, hax-fstar, kani, miri, prusti, rocq-of-rust, soteria, verifast, verus
F(9): aeneas × 4, charon-mono, creusot, hax-coq, hax-lean, kmir

注：charon-mono 失败而 charon-poly SUCCESS——本测试矩阵观察到的两 mode 行为分化点之一。

### 主题 9: panic / unwinding 策略

**9.1 stack unwinding (`kani-limit/stack-unwinding`)**

> kani 自声明：仅支持 `panic = "abort"` 策略；unwinding 策略 unsupported（feature support 文档；issue #692）。

S(5): charon × 2, hax-fstar, rocq-of-rust, verifast
F(14): 其余全部

**9.2 unbounded loop unwinding (`kani-limit/loop-unwinding`)**

> kani 自声明：non-constant 迭代次数循环需 `#[kani::unwind(n)]`；不足时 unwinding assertion failure / non-termination（loop-unwinding tutorial）。

S(10): aeneas-coq/fstar/lean, charon × 2, hax × 3, rocq-of-rust, verifast
F(9): aeneas-hol4, cargo-check, creusot, kani, kmir, miri, prusti, soteria, verus

### 主题 10: uninit / 未定义行为 / soundness

**10.1 读未初始化内存 (`kani-limit/uninit-memory`)**

> kani 自声明：仅在 `-Z uninit-checks` 实验性开启时检测；标准 verification run 不开（feature support 文档；issue #3300）。

S(12): aeneas × 4, charon × 2, creusot, hax-fstar, hax-lean, rocq-of-rust, verifast, verus
F(7): cargo-check, hax-coq, kani, kmir, miri, prusti, soteria

**10.2 safe wrapper 隐藏 UB (`miri-limit/soundness-not-guaranteed`)**

> miri 自声明：fundamentally cannot ensure soundness——只检查特定执行，不覆盖所有 caller；unsafe 包成 safe API 的 UB 未必触发。

S(10): aeneas × 4, charon × 2, hax-fstar, hax-lean, rocq-of-rust, verifast
F(9): cargo-check, creusot, hax-coq, kani, kmir, miri, prusti, soteria, verus

### 主题 11: const-generic / drop glue

**11.1 含 const-generic 的函数 (`prusti-limit/const-generics`)**

> prusti issue #1195：const-generic 参数（`const N: usize`）让 `get_body_with_borrowck_facts` 在 prusti 内 panic，body 无法分析。

S(12): aeneas-coq/fstar/lean, charon × 2, creusot, hax × 3, rocq-of-rust, soteria, verus
F(7): aeneas-hol4, cargo-check, kani, kmir, miri, prusti, verifast

**11.2 `--precise-drops` + const-generic + Box (`charon-limit/precise-drops-const-generic`)**

> charon: 用 `--precise-drops` 时 const-generic array 字段 + Box 字段 struct 触发 rustc ICE on drop-glue retrieval（translate_drops.rs known issue）。

S(16): 大多数
F(3): aeneas-hol4, kmir, verifast

### 主题 12: 互递归 / 函数指针强转

**12.1 Mutually recursive functions (`creusot-limit/mutual-recursion`)**

> creusot：termination check 不处理 mutually recursive call graphs；翻译触发 internal error 或 Why3 unbound symbol（issue #910 / #1232）。

S(11): aeneas × 4, charon × 2, hax × 3, rocq-of-rust, verifast
F(8): cargo-check, creusot, kani, kmir, miri, prusti, soteria, verus

**12.2 Mutually-recursive traits + associated type (`aeneas-limit/mutually-recursive-traits`)**

> aeneas: trait associated types 通常被提升为 trait 定义参数，但 mutually-recursive traits 与 GAT 上失败。

S(12): aeneas-coq/fstar/lean, charon × 2, creusot, hax × 3, rocq-of-rust, soteria, verifast
F(7): aeneas-hol4, cargo-check, kani, kmir, miri, prusti, verus

**12.3 函数指针 reify (`creusot-limit/fn-ptr-reify`)**

> creusot：把命名函数 coerce 成 bare function pointer 触发 MIR `PointerCoercion(ReifyFnPointer)` cast，creusot 报 `Unsupported cast: ... PointerCoercion(ReifyFnPointer, Implicit)`（issue #1728）。

S(8): charon × 2, hax × 3, rocq-of-rust, soteria, verifast
F(11): aeneas × 4, cargo-check, creusot, kani, kmir, miri, prusti, verus

### 主题 13: thread / 并发 / 内存模型

**13.1 thread-local 引用 (`creusot-limit/thread-local-ref`)**

> creusot：MIR `Rvalue::ThreadLocalRef` 在 statement translator 标 unsupported，crash with `MIR code used an unsupported Rvalue`。

S(4): charon-mono, rocq-of-rust, soteria, verifast
F(15): 其余全部（含 charon-poly！）

注：charon 双 mode 在此 entry 行为不同——charon-mono SUCCESS，charon-poly FAILED。

**13.2 thread interleaving (`miri-limit/thread-interleaving-partial`)**

> miri：每次执行只探索一种 interleaving；`-Zmiri-many-seeds` 可缓解但不完整。

S(5): charon × 2, hax-fstar, rocq-of-rust, verifast
F(14): 其余全部

**13.3 weak memory (`miri-limit/weak-memory-incomplete`)**

> miri：weak memory emulation 不完整；存在 miri 永不产生的合法行为；推荐 loom。

S(12): aeneas × 4, charon × 2, hax × 3, rocq-of-rust, soteria, verifast
F(7): cargo-check, creusot, kani, kmir, miri, prusti, verus

**13.4 Networking 系统调用 (`miri-limit/networking-unsupported`)**

> miri：跨平台 interpreter 无 networking shim；TCP connect 触发 unsupported operation。

S(11): aeneas-coq/fstar/lean, charon × 2, creusot, hax × 3, rocq-of-rust, verifast
F(8): aeneas-hol4, cargo-check, kani, kmir, miri, prusti, soteria, verus

### 主题 14: SIMD / AVX-512

**14.1 大 SIMD vector bitmask (`miri-limit/simd-bitmask-large-vector`)**

> miri：`only supports very few AVX512 intrinsics`；64-bit+ vector bitmask 触发 `unimplemented intrinsic: avx512...`。

S(8): aeneas-coq/fstar/lean, charon × 2, hax-lean, rocq-of-rust, verifast
F(11): aeneas-hol4, cargo-check, creusot, hax-coq, hax-fstar, kani, kmir, miri, prusti, soteria, verus

### 主题 15: For loop / iterator

**15.1 泛型 iterator 上 for loop (`creusot-limit/generic-for-loop`)**

> creusot：`for` 需要 iterator type 实现 `creusot_std::Iterator`（带 `produces` spec 的 creusot 自家 trait），plain `std::iter::Iterator` bound 不带 spec 触发 `the trait bound &mut I: creusot_contracts::IntoIterator is not satisfied`（issue #1285）。

S(10): aeneas-coq/fstar/lean, charon × 2, hax × 3, rocq-of-rust, verifast
F(9): aeneas-hol4, cargo-check, creusot, kani, kmir, miri, prusti, soteria, verus

**15.2 普通 iterator 的 for loop (`prusti-limit/for-loop-iterator`)**

> prusti：`for x in iter` 糖到 `Iterator::next` 涉及 magic wands（borrow expiry），prusti 不能编码。`for_iter.rs` / `simple_iterator.rs` 标 ignore-test，"magic wands in loop invariants" not yet supported。

S(12): aeneas-coq/fstar/lean, charon × 2, creusot, hax × 3, rocq-of-rust, soteria, verifast
F(7): aeneas-hol4, cargo-check, kani, kmir, miri, prusti, verus

### 主题 16: Box 初始化分支控制流

**16.1 Box::new 含 early return (`charon-limit/box-branch-init`)**

> charon `transform/resugar/reconstruct_boxes.rs`："Could not reconstruct Box initialization; branching during Box initialization is not supported." rustc 把 `Box::new(x)` 降为两步序列；分支让两步分散到不同 BB。

S(13): cargo-check, charon × 2, creusot, hax × 3, kani, miri, prusti, rocq-of-rust, soteria, verifast
F(6): aeneas × 4, kmir, verus

### 主题 17: `vec![...]` 标准宏

**17.1 Standard `vec![]` (`creusot-limit/vec-macro-std`)**

> creusot：标准 `vec![]` 用 compiler "magic"（特殊 `box_free` / `RawVec` intrinsic 路径）直接初始化堆，creusot 翻译会 crash。需用 creusot-std 替换 macro。

S(14): aeneas × 4, charon × 2, creusot, hax × 3, rocq-of-rust, soteria, verifast, verus
F(5): cargo-check, kani, kmir, miri, prusti

### 主题 18: bool 上的 bitwise 运算

**18.1 bool 的 `& | ^` (`aeneas-limit/bool-bitwise-op`)**

> aeneas：bitwise binop 只识别整数类型；`bool` 操作数（`a & b` 当 `a, b: bool`）报 `[Error] Invalid inputs for binop`。

S(9): charon × 2, creusot, hax × 3, rocq-of-rust, soteria, verifast
F(10): aeneas × 4, cargo-check, kani, kmir, miri, prusti, verus

### 主题 19: unsafe 块整体禁

**19.1 任何 `unsafe { }` (`hax-limit/unsafe-block`)**

> hax error HAX0000 (UnsafeBlock)：unconditional reject。manual quick_start 写明 hax 不支持 unsafe code。

S(12): aeneas × 4, charon × 2, creusot, hax-fstar, hax-lean, rocq-of-rust, soteria, verifast
F(7): cargo-check, hax-coq, kani, kmir, miri, prusti, verus

注：hax-coq 失败而 hax-fstar / hax-lean SUCCESS——三 backend 行为分化。

### 主题 20: prusti 高阶 spec

**20.1 Spec entailments `f |= |args| [...]` (`prusti-limit/spec-entailment-unsupported`)**

> prusti user guide：`f |= |args| [requires(...), ensures(...)]` 整个 feature "NOT YET SUPPORTED"，无法给泛型 Fn 参数附 pre/post。

S(9): charon × 2, creusot, hax-coq, hax-fstar, rocq-of-rust, soteria, verifast, verus
F(10): aeneas × 4, cargo-check, hax-lean, kani, kmir, miri, prusti

### 主题 21（基础特性维度补充）

**deps-complex 类**（133 task / 48% rate）：rocq-of-rust 全 0（不读 Cargo.toml，dep import 失败）；soteria 0（依赖混合解析问题）；verifast / verus 0；kani / miri / cargo-check 100%。chrono / serde / serde-generic / itertools / error-chain / 多 dep 组合。

**bigint 类**（152 task / 65% rate）：含 num-bigint / num-rational / num-integer / num-traits / num-complex 8 个 entry。rocq-of-rust 0（同上 dep import）；soteria / verus / verifast 0；其他多数 100%。

**industrial 类**（sha2 / x509-parser / rsa）：本次 run **未跑**——runner discover 用 `walkdir::max_depth(2).min_depth(2)`，industrial 是三级路径 `industrial/sha2/sha256-digest/`。后续 fix runner 后单独补跑。

## 六、若干跨工具特性分化点

矩阵里观察到的"同一 entry 在工具间分化最显著"的几例：

| Entry | 状态分化 |
|---|---|
| `kani-limit/async-await` | 仅 hax-lean SUCCESS / 18 工具 FAILED——async 是覆盖度最低构造 |
| `creusot-limit/thread-local-ref` | 仅 4 工具 SUCCESS（charon-mono, rocq-of-rust, soteria, verifast）/ 15 工具 FAILED |
| `creusot-limit/dyn-trait-forbidden` | 仅 5 工具 SUCCESS / 14 FAILED |
| `kani-limit/stack-unwinding` | 仅 5 工具 SUCCESS（charon × 2, hax-fstar, rocq-of-rust, verifast）/ 14 FAILED |
| `miri-limit/thread-interleaving-partial` | 仅 5 工具 SUCCESS / 14 FAILED |
| `charon-limit/copy-deref-closure` | 18 工具 SUCCESS / 仅 aeneas-hol4 FAILED——单点失败 |

charon 双 mode 与 hax 三 backend 内部分化点：

- `charon-limit/generic-to-dyn-unsize`：charon-poly SUCCESS, charon-mono FAILED
- `creusot-limit/thread-local-ref`：charon-mono SUCCESS, charon-poly FAILED
- `hax-limit/unsafe-block`：hax-fstar/lean SUCCESS, hax-coq FAILED

aeneas 4 backend 内部分化点：

- aeneas-hol4 在 9 个 entry 上 FAILED 而 aeneas-{coq,fstar,lean} 同时 SUCCESS：`assoc-type/iter-style`, `closure/fn-fnmut/closure_fnmut`, `closure-adv/return-impl-fn/return_impl_fn`, `slice/basic-iter/sum_slice`, `aeneas-limit/fnmut-closure-unit-return`, `aeneas-limit/mutually-recursive-traits`, `aeneas-limit/nested-borrow-array`（部分）, `aeneas-limit/trait-impl-mut-param-mismatch`, `loop-unwinding/sum_to_n`。HOL4 backend 在闭包返回类型与 trait associated type pretty-print 上更严格。

## 七、覆盖度核心观察

1. **140 entries 覆盖了 19 类工具自声明限制 + 大量基础 Rust 特性**，整体接受率 68%。
2. **"全工具都接受"的特性**：hello / int / vec 三个特性 100% S（所有 19 工具都接受）。
3. **"无工具接受"的特性**：本矩阵无单 entry 在 19 工具上全 FAILED。最低是 `kani-limit/async-await` 仅 1/19 SUCCESS。
4. **`*-limit` 类目的精确语义**：每个 limit 名下并非工具的"统一不支持点"——不同 limit entry 测的是工具自声明的不同具体特性边界，本矩阵把它们逐条展开成 24 个 Rust 特性主题（见第五节）。
5. **基础特性弱点**：`trait`、`unsafe-ptr`、`gat`、`lifetime`、`error`、`trait-obj` 在多工具上是分化点（rate 47–68%），是后续扩样例的重点。
6. **`deps-complex` 整体 48%**——多 dep 组合在 rocq-of-rust / soteria / verifast / verus 上零通过；揭示这些工具在外部 crate 集成层面的接受边界。

数据来源 `runs/run-1778148197-53283/results.json`；每 entry 的 doc comment 在 `examples/<feature>/<dir>/src/lib.rs`；每工具的 raw stderr 在 `runs/run-1778148197-53283/raw/<tool>/<entry_id>.stderr`。
