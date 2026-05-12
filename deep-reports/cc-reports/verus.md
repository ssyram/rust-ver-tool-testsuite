# verus — 特性支持评估报告（v6 baseline）

## 元数据

- **数据源**：`runs/run-1778560393-59119/`（2026-05-12 v6 final，合并 verus rerun `runs/run-1778561896-25488/` + R7 5-tool rerun；首次主跑因 `/tmp` 自动清理 verus-root 而 161/161 FAILED，P29 治源迁 `~/.local/share/ts-tools/verus/` 后 rerun 合并入 v6 final，详 `docs/fixes/v6-verus-env-fix-2026-05-12.md`）
- **工具配置**：`tools/verus/`
- **工具版本**：`Verus 0.2026.05.03.8b81855`，profile release，platform macos_aarch64，Rust toolchain `1.95.0-aarch64-apple-darwin`
- **本工具实测**：n=161 / SUCCESS=66 / FAILED=95 / UNKNOWN=0，通过率 **41.0%**
- **时长分布**：avg 323 ms / median 278 ms / p90 338 ms / p95 1390 ms / max 1563 ms（亚秒级；timeout 120 s 远未触达——verus binary 内置 vstd，不走 cargo 重编 vstd）
- **宪法 baseline**：`principles.md` v8（P27 修宪后 / P31 法律传导后；UNKNOWN 严格语义 + 本地性 / 最大善意 / 不公信问题）
- **时效声明**：本快照锚定上述 run id + verus 0.2026.05.03 release binary（内置 vstd v0.2026.05.03）+ corpus，不构成长期承诺。verus 上游对 vstd 持续扩充 spec 覆盖、对 `--no-verify` 路径下的内部 panic site 会修复——本快照随之失效。

## pipeline + 前端边界

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

**前端 / 后端切割**：理想上应落在 **VIR/AIR 构造** 与 **Z3 SMT 求解** 之间。但 verus 的 `--no-verify` flag **同时切断 AIR 构造与 Z3 调用**（all-or-nothing），无 `--no-z3` / `--rlimit 0` 独立切线。所以本工具能拿到的最深前端表示是 **VIR**，而非 AIR/SMT-LIB。AIR ≈ SMT-LIB 的前置形式，按精神 AIR 算后端，所以 VIR 即为前端边界。`--log vir` 把 VIR 写到 `.verus-log/crate.vir`（每个 fn 在 VIR 里以 `(Function :name ... :body ...)` 存在），便于函数级对应性校验。

**工具自身 vs 项目维护边界**：测试命令 `verus --no-verify --log vir --crate-type=lib src/lib.rs` 直接调上游 binary，本项目**未**包装额外 wrapper 脚本——只在 runner 内做 entry_mode = "lib" 的 harness 注入（`harness.rs.tera` + 文件改名 `src/lib.rs` → `src/__ts_inner.rs`）。harness 注入属"我们这边"的输入流水线，其余 verus 行为属"工具本身"。`tool.toml` 不走 `cargo-verus`——cargo-verus 0.2026.05.03 与 crates.io vstd 任何版本均不兼容，会 panic 于 `rust_verify/src/erase.rs:405`（`Option::unwrap()` on `None`）。

## SUCCESS 信号 + 形式严格性

**判定式**：

```
SUCCESS ⟺ verus exit 0（即 Verus 前端完成 type/lifetime/mode check + VIR 构造）
```

**主信号通路**：exit code；`--log vir` 落产物到 `.verus-log/crate.vir`，oracle 可独立验证 VIR 是否真构造完成。

**wrapper 补抓通路**：本工具无项目侧 wrapper；只在 `harness.rs.tera` 强制 `mod __ts_inner;` 写在 `verus! { }` 块内、不带 `#[verifier::external]`，保证 inner 的真实代码受 verus 前端逐项检查、不会被透传给 stock rustc。这条不变量是反作弊核心（详 `tools/verus/README.md` §"反作弊"）。

**形式严格性 0 误报状态**：✅ 实测 + 源码层论证。verus exit 0 ⇔ VIR 构造完成且 verus 前端无错误——任何 rejection（lifetime / type / mode / vstd link / `assume_specification` 缺失 / `verus_builtin not imported` 等）通过 `dcx().emit` 触发 exit ≠ 0。

**形式严格性 0 漏报状态**：✅ 实测 + 源码层论证，但存在一类已知的盲点：

- **derived auto-spec 缺失继续**（D3.6 / 2026-05-12 README 补完）：部分 SUCCESS entry（如 `aeneas-limit/float-types/make_measurement`）的 stderr 含 verus 警告 `"autoderive Clone impl does not take the form Verus expects; continuing, but without adding a specification for the derived Clone impl"`。"continuing" 表示 VIR 构造完成、verus 前端不 reject；缺的是 `#[derive(Clone)]` 的 auto-spec 生成——属 verus → SMT 求解的中间步骤（spec gen 不属前端）。按宪法 §六 前端测量原则保持 SUCCESS。**这是诚实声明的边界**：测的是前端接受、不是后端 spec 完备。

## 失败分桶（按 P31 §四.5 归因分类）

### 桶 A：vstd 没给 std lib API 挂 spec（29 case）

代表 entry：`float/{cmp, nan-prop, special-vals, transcendental, bits, cast-int, cast-widening, round}`、`int-width/{wrapping-i64, saturating-u16, shift-overflow, arith-i128, arith-i8, bit-ops-u32, overflowing-i32}`、`unsafe-adv/{transmute, maybe-uninit}`、`collections/hashmap`、`error/result-question`、`panic/explicit`、`rc/clone-drop`、`iter/chain-collect`、`impl-trait/return-iter`、`hax-limit/unsafe-block`、`closure-adv/fn-once`、`charon-limit/box-branch-init`、`kani-limit/float-overapprox`、`miri-limit/{networking-unsupported, soundness-not-guaranteed}`。

stderr 特征：

```
error: `<core::...>` is not supported (note: you may be able to add a Verus specification to this function with `assume_specification`) (note: the vstd library provides some specification for the Rust std library, but it is currently limited)
help: pub assume_specification ...
```

具体被拒 API 类别：数值 `wrapping_*` / `saturating_*` / `overflowing_*` / `rotate_*` / `unchecked_*`；浮点常量与函数 `f64::{NAN, INFINITY, NEG_INFINITY, from_bits, is_nan}` / `f64::{floor, cos}`；内存原语 `transmute` / `mem::drop` / `MaybeUninit::write`；容器与迭代器 `Option::copied` / `iter::Map` / `Filter` / `str::Split`；字符串 `String::from` / `fmt::from_str`；panic `panicking::panic_fmt` + `fmt::Arguments`；其他 `hint::black_box` / `slice::get_unchecked` / `io::Error`。

**归因**：工具不支持（vstd spec 边界——verus 自陈"the vstd library provides some specification for the Rust std library, but it is currently limited"）
**处理**：不修。FAILED 站得住，工具开发者不能驳回。

### 桶 B：verus 语言子集尚未支持的 Rust 构造（19 case）

代表 entry：`aeneas-limit/{bool-bitwise-op, fnmut-closure-unit-return}`、`assoc-type/iter-style`、`charon-limit/{arc-slice-unsize, inline-asm}`、`closure-adv/{boxed-dyn-fn, early-bound-lifetime}`、`creusot-limit/fn-ptr-reify`、`drop/custom-drop`、`gat/lending-iter`、`generic/pair-struct`、`hax-limit/mut-arg-pattern`、`miri-limit/thread-interleaving-partial`、`refcell/borrow`、`repr/union`、`trait/cyclic-bound`、`trait-obj/conditional-method`、`unsafe-adv/ptr-write`、`unsafe-ptr/raw-read`。

stderr 特征：

```
error: The verifier does not yet support the following Rust feature: <X>
```

X 清单：`bitwise AND/OR for bools (not short-circuited)` / `internal item statements`（fn body 内嵌 trait/impl/union 定义）/ `dyn with more than one trait` / `inline-asm expressions` / `unsizing operation from Arc<[T; N]> to Arc<[T]>` / `function pointer types` / `dereferencing a pointer (here implicit)` / `deref_mut for RefMut<...>` / `only variables are supported here, not general patterns`。

**归因**：工具不支持（verus 自陈"yet to support"语言子集边界）
**处理**：不修。FAILED 站得住。

### 桶 C：外部 crate 无法 resolve（21 case）

代表 entry：`bigint/*`（8 条）、`deps-complex/*`（7 条）、`industrial/{rsa/*, sha2/*, x509-parser/*}`（6 条）。

stderr 特征：

```
error[E0432]: unresolved import `num_bigint` / `serde` / `chrono` / `sha2` ...
error[E0433]: cannot find module or crate `x509_parser` in this scope
```

**归因**：工具不支持。verus binary 不通过 cargo 工作（cargo-verus 不可用——见 §pipeline），不读 `Cargo.toml`，外部 crate 无 rmeta 可链接——这是 verus 自陈的 pipeline 设计选择（standalone binary + 内置 vstd）。按本地性原则，工具的 pipeline 设计就是它的能力边界。
**处理**：不修。

### 桶 D：verus-driver 内部 panic `generic_args.rs:54` index OOB（12 case）

entry 列表：`aeneas-limit/{nested-borrow-array, return-inside-nested-loop}`、`collections/btreemap`、`concurrency/thread-mutex`、`creusot-limit/generic-for-loop`、`generic/sum-bound`、`hax-limit/{closure-mutates-outer, labelled-break}`、`kani-limit/loop-unwinding`、`miri-limit/simd-bitmask-large-vector`、`prusti-limit/for-loop-iterator`、`slice/index-iter`。

stderr 特征：

```
thread 'rustc' panicked at /rustc-dev/.../compiler/rustc_middle/src/ty/generic_args.rs:54:14:
index out of bounds: the len is 0 but the index is 0
```

**归因**：工具不支持。同一 panic site、不同触发条件，是 verus-driver 在 `--no-verify` 模式下某条共用代码路径的边界条件——上游 bug，不属于 verus 设计内的 reject 路径，但按宪法 §六"不允许 partial"延伸：工具内部异常等同未完成处理。
**处理**：不修。FAILED 站得住（本地性原则）。

### 桶 E：verus-driver 其他 panic 站点（7 case）

entry 列表：`const/const-fn`、`creusot-limit/thread-local-ref`、`float/total-order`、`kani-limit/async-await`、`lifetime/thread-local`、`miri-limit/weak-memory-incomplete`、`unsafe-ptr/raw-ptr-const`。

stderr 特征：

```
panicked at rustc_mir_build/.../rustc_mir_build_additional_files/verus.rs:176:21:
Internal Verus Error: The thir_body query is running for item ... which may require erasure, but the VerusErasureCtxt has not been initialized
```

或 `rust_to_vir_base.rs:749:14`（async fn / thread_local! / `*const T` 解引用 const 路径触发）。

**归因**：工具不支持（verus 自陈 Internal Verus Error，建议 `--no-lifetime` 临时绕开——属上游 bug）
**处理**：不修。

### 桶 F：closure 捕获 `&mut`（2 case）

entry 列表：`closure/fn-fnmut/{closure_fn, closure_fnmut}`。stderr：`error: Verus does not currently support closures capturing a mutable reference for variables of any mode`。

**归因**：工具不支持。
**处理**：不修。

### 桶 G：edition 默认（1 case）

entry：`hax-limit/let-chains`。stderr：`error: let chains are only allowed in Rust 2024 or later`。

**归因**：工具不支持。verus binary 直接调 rustc fork、不读 Cargo.toml 中的 edition 设置——这是 verus pipeline 设计选择（同桶 C 同根因，但表面 marker 不同）。
**处理**：不修。

### 桶 H：其他单点 verus 拒绝（4 case）

| entry | stderr 要点 |
|---|---|
| `creusot-limit/dyn-trait-forbidden/trigger_call_dyn_display` | Unsupported constant type |
| `aeneas-limit/mutually-recursive-traits/trigger_mutually_recursive_traits` | cyclic self-reference |
| `lifetime/static-bound/static_bound` | trait `core::any::Any` not declared to Verus |
| `kani-limit/stack-unwinding/trigger_divide_with_recovery` | does not recognize this trait bound |

**归因**：工具不支持（verus 显式 reject）
**处理**：不修。

### 失败分桶小计

| 桶 | 归因 | 数量 |
|---|---|---:|
| A vstd 边界 | 工具不支持 | 29 |
| B 语言子集未支持 | 工具不支持 | 19 |
| C 外部 crate unresolved | 工具不支持（pipeline 设计） | 21 |
| D `generic_args:54` panic | 工具不支持（上游 bug） | 12 |
| E 其他 driver panic | 工具不支持（上游 bug） | 7 |
| F closure & mut | 工具不支持 | 2 |
| G edition let-chain | 工具不支持（pipeline 设计） | 1 |
| H 其他单点 reject | 工具不支持 | 4 |
| **合计** |  | **95** |

## 漏报盲点（诚实声明）

- 已通过 harness 反作弊不变量封堵：`mod __ts_inner` 必须在 `verus! { }` 块内、不带 `#[verifier::external]`——确保 inner 代码受 verus 前端逐项检查，不被透传给 stock rustc。`harness.rs.tera` 已固化此不变量。
- 仍存在的盲点：
  - **derived auto-spec 警告 entry**（如 `aeneas-limit/float-types`）按宪法 §六 前端测量原则保持 SUCCESS——VIR 构造完成属前端通过，spec gen 不属前端。oracle 当前未升 wrapper grep gate；触发条件 = SUCCESS entry 的 stderr 含 `continuing, but without adding a specification`。修复 backlog 在 `docs/fixes/audit-v6-cc-false-negative-counter-2026-05-12.md`（D3.6 已 README 补完声明、未升 gate——属"前端测量"判定边界，非工具能力误判）
  - **求解层假设**：`--no-verify` 同时切 AIR + Z3，未触达 AIR 构造、SMT-LIB 生成、Z3 求解。最深前端到 VIR

## v5.1 → v6 ΔS 解释

v5.1 SUCCESS=66 → v6 SUCCESS=66，ΔS=0。

- v5 旧报告写 SUCCESS=51 是基于 v5 corpus 146 entry；v5.1 corpus 扩到 161、SUCCESS=66；v6 corpus 仍 161、SUCCESS=66，与 v5.1 一致。
- v6 首次主跑因 `/tmp` 自动清理 verus-root 而 161/161 FAILED——P29 治源迁 `~/.local/share/ts-tools/verus/` 后 rerun（`runs/run-1778561896-25488/`）合并入 v6 final（`runs/run-1778560393-59119/`），最终数据与预期一致。这是项目侧环境损坏，按宪法 §六 UNKNOWN (a) 类——但治源后已修，未在 final results 留下 UNKNOWN。
- UNKNOWN 数 v5.1 22 → v6 0，是 P27 修宪后 DP-4 严格化的全局效应（不允许把工具能力边界记为 UNKNOWN）——不构成 ΔS。

## 修订建议清单（仅"我们导致"失败）

| # | 桶 | 涉及 case | 修复方案 | 优先级 |
|---|---|---:|---|---|
| — | — | 0 | **无"我们导致"失败**。所有 95 FAILED 均为工具能力边界（vstd 边界 / 语言子集未支持 / pipeline 设计 / 上游 panic / 显式 reject）。本工具 wrapper-less 设计 + harness 反作弊不变量已固化、未发现项目侧 bug | — |

环境损坏类（v6 首次 verus-root 丢失）已 P29 治源、不再列入修订；详 `docs/fixes/v6-verus-env-fix-2026-05-12.md`。

derived auto-spec SUCCESS 警告是判定边界问题、非"我们导致"——按宪法 §六 前端测量原则解释为前端通过、README 已诚实声明、oracle 不升 gate。
