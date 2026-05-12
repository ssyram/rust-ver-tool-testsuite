# creusot — 特性支持评估报告（v6 final post-P35 baseline）

## 元数据

- **数据源**：`runs/run-1778560393-59119/`（2026-05-12T04:33:13Z → 04:55:27Z UTC，161 entries × 20 工具，host：Apple M5 / macOS aarch64 / 24 GB / 10 cpu，parallelism = 10）
- **工具配置**：`tools/creusot/`
- **工具版本**：`cargo-creusot 0.11.0` + `Rust toolchain nightly-2026-02-27` + `Why3 1.8.2+git` + `why3find 1.3.0+dev` + `alt-ergo 2.6.2` + `z3 4.15.4` + `cvc4 1.8` + `cvc5 1.3.1`
- **本工具实测**：n=161 / SUCCESS=121 / FAILED=40 / UNKNOWN=0，通过率 **75.2 %**
- **时长分布**：avg 29654 ms / median 29997 ms / p90 37887 ms / max 44258 ms
- **宪法 baseline**：`principles.md` v8（P27 修宪后 / P31 法律传导后 / P35 累积），架构 `bug detect 归 SUCCESS`（§四 B 派生）+ `当前 crate 焦点`（§六）已应用
- **时效声明**：本快照锚定上述 run id + 工具版本 + corpus，不构成长期承诺。免责见 [`README.md`](../../README.md) 顶部。

## pipeline + 前端边界

cargo-creusot 通过替换 `RUSTC` 让 cargo 编译流程的每个 crate target 经过 `creusot-rustc`：

```
cargo build → creusot-rustc（rustc + creusot translation passes）
            → 每个 fn 的 MIR 翻译为 .coma（Why3 IR），写盘 verif/<crate>_rlib/
            → cargo-creusot 默认无 subcommand 时即退出
              （显式 cargo creusot prove 才进入 why3find → why3 → SMT solver）
```

本测试 `tool.toml` 仅传 binary 路径无 subcommand——天然停在 **Coma 翻译完成 / Why3 求解开始** 之间，等价于"前端 dry-run"。

边界：

- **前端**（本工具检测）：MIR → coma 翻译
- **后端**（本工具不检测）：why3find → why3 → Alt-Ergo / CVC5 / Z3 求解

工具自身已含完整 wrapper / 调度逻辑——cargo-creusot 是上游官方 binary，本项目**没有维护任何 wrapper 脚本**。runner 直接调用该 binary。

`tool.toml` 关键参数：

- `entry_mode = "lib"`——cargo-creusot 对每个 crate target 跑 creusot-rustc，硬要求顶级 lib 自身含 `use creusot_std::prelude::*;`；runner 把原 `src/lib.rs` 改名 `src/__ts_inner.rs`，新 `src/lib.rs` 是含两个硬性 import 的 harness lib
- `extra_cargo_deps = ['creusot-std = "0.11.0"']`——cargo-creusot 在 cargo 解析阶段就检查 manifest 必须列出 `creusot-std`，缺失直接拒
- `timeout_secs = 900`——给翻译留足时间；runner 用 `kill(-pgid)` 杀整个进程组防止孙子进程持有 fd

## SUCCESS 信号 + 形式严格性

按宪法 §六 双通路 partial 暴露：

- **主信号通路**：`cargo-creusot` exit 0（默认无 subcommand 即只翻译到 `.coma`，不调用 why3）
- **wrapper 补抓通路**：无（本项目对 creusot 不维护 wrapper，直接信任 binary 的 exit code）

形式严格性 0 误报 / 0 漏报状态（按 P27 后宪法严格语义）：

- **0 误报（不冤枉能力）**：实测层强信号——所有 40 个 FAILED 均为 exit=101 显式 reject，无"工具完成却被判错"的观察。形式严格性级别：**实测可信**，非源码层证明
- **0 漏报（不高估能力）**：creusot 通过 `crash_and_error / span_err / span_fatal / dcx().span_err` 把 unsupported 升级为 rustc error → exit 101，路径在 creusot 源码层显式存在；本项目实测未发现 silent SUCCESS 但产物缺失的反例
- **漏报盲点（诚实声明）**：
  - **盲点 1**：未独立校验 `verif/<crate>_rlib/` 目录与 `.coma` 文件实际存在 / 非空 / 含入口 fn 翻译——理论上若 creusot 内部 silent skip 某 fn 而 cargo-creusot 仍 exit 0，本测量会误报 SUCCESS。该盲点是"实测可信、非源码证明"的来源
  - **盲点 2**：本工具仅停在前端翻译边界，**SUCCESS 不蕴含 SMT 求解结论**。每个 SUCCESS entry stderr 普遍含 `warning: calling external function X with no contract will yield an impossible precondition`——表示 creusot-rustc 已接受代码并打算翻译，仅在 spec 完整性上不构成 prove 结果。本测量在 `cargo creusot prove` 这条路径上无观察

每个 SUCCESS 案例的 stderr 统一含 `warning: unused import: 'creusot_std::prelude::*'`——harness prelude import 多数样例未实际触发，warning 不影响 exit。

**bug detect 归 SUCCESS 派生（§四 B / architecture §一）的适用性**：creusot 在本测试 `tool.toml` 下停在 coma 翻译边界，**不进入 prove**——无"工具自陈在 entry 找到 bug"的信号路径。该派生原则在 creusot 当前配置下**无触发**（与 architecture.md §一末段表格一致：creusot 列"0 触发"）。所有 FAILED 均为"工具不能吃下代码"而非"工具吃下后判违反"。

**当前 crate 焦点（§六）应用**：runner 注入 `TS_TARGET_CRATE` / `TS_ENTRY_FN`，creusot-rustc 对 entry crate 自身的 fn / type 翻译报错（如 `Unsupported constant value` / `forbidden dyn type`）才视作 partial → FAILED。本 run 全部 40 个 FAILED 的报错位置均落在 entry crate 自身的 MIR（含 derive 展开 / format macro 展开在 entry-crate 内的产物）——无"外部依赖路径 opaque / skip / stub"造成的虚假 FAILED 判定。

## 按 feature 类目分布

全 SUCCESS 类目（27 项）：`aeneas-limit (8/8) · arc · assoc-type · box · closure · const · drop · enum · gat · generic · hax-limit (8/8) · hello · hrtb · impl-trait · int · iter · panic · prusti-limit (8/8) · rc · refcell · runnable (15/15) · slice · trait · unsafe-adv · vec`

部分通过：`bigint 7/8 · charon-limit 3/7 · closure-adv 2/4 · collections 1/2 · concurrency 1/2 · creusot-limit 3/7 · deps-complex 1/7 · float 5/10 · industrial 2/6 · int-width 13/14 · kani-limit 5/7 · lifetime 1/3 · miri-limit 5/7 · repr 1/2 · trait-obj 1/2`

全 FAILED：`error 0/1 · unsafe-ptr 0/2`

按 feature 失败计数（前 5）：`deps-complex 6 · float 5 · charon-limit 4 · creusot-limit 4 · industrial 4`

## 失败分桶（按 P31 §四.5 归因分类）

40 个 FAILED 全部 exit=101，按 stderr 字面信号归 7 桶。**所有桶均归"工具不支持"（creusot 自身的翻译域 / 内部 panic / std-coverage 边界）**——本工具无 wrapper、无 corpus 引入 lint、无环境损坏。

### 桶 1：显式 `forbidden dyn type`（7 case）

代表 entry：

- `closure-adv/boxed-dyn-fn`（`dyn Fn(i32)→i32`）
- `trait-obj/dyn-dispatch`（用户 trait `Greeter`）
- `lifetime/static-bound`（`dyn Any`）
- `concurrency/thread-mutex`（`dyn Any + Send` from `JoinHandle`）
- `miri-limit/thread-interleaving-partial`、`kani-limit/stack-unwinding/trigger_divide_with_recovery`、`charon-limit/generic-to-dyn-unsize/boxed_display_from_u32`

stderr 特征：

```
error: forbidden dyn type: dyn ... (dyn support is currently minimal, please open an issue to improve this feature)
```

**归因**：工具不支持（creusot-rustc 对 `dyn Trait` 走统一拒绝路径，不区分内置 trait / 用户 trait）。
**处理**：不修。本地性原则下 FAILED 站得住。

### 桶 2：Unsupported pointer cast / coercion（6 case）

代表 entry：

- `unsafe-ptr/raw-ptr-const`（`43 as *const ()` / `PointerWithExposedProvenance`）
- `int-width/cast-float-int` / `float/cast-int`（`IntToFloat`）
- `float/cast-widening`（`FloatToFloat`）
- `closure-adv/early-bound-lifetime`（`ClosureFnPointer(Safe), Implicit`）
- `creusot-limit/fn-ptr-reify`（`ReifyFnPointer(Safe), Implicit`）

stderr 特征：

```
error: Unsupported pointer cast: <expr> as <type> (PointerCoercion(...))
```

**归因**：工具不支持（creusot 显式拒收若干 PointerCoercion 与 int↔float / float↔float cast）。
**处理**：不修。

### 桶 3：raw pointer dereference forbidden（1 case）

代表 entry：`unsafe-ptr/raw-read`

stderr 特征：

```
error: Dereference of a raw pointer is forbidden in creusot: use creusot_std::ghost::perm::Perm<*const T> instead
```

**归因**：工具不支持（错误信息主动给出替代方案——creusot 对 raw deref 设计了 ghost-permission 模型，要求用户改写）。
**处理**：不修。

### 桶 4：Unsupported constant value / expression（10 case）

代表 entry：

- `industrial/rsa/rsa-pkcs8/rsa_pubkey_from_pkcs8`、`rsa_pkcs1v15_encrypt`（`Scalar(alloc4) of type &[u8; 9_usize]`）
- `industrial/sha2/sha256-digest/sha256_digest_one_shot`、`sha256_digest_incremental`（`Scalar(alloc1) of type &[u8; 43_usize]`）
- `creusot-limit/dyn-trait-forbidden`（`format!()` 展开后的 `&[u8; 2]`）
- `deps-complex/error-chain`（`Scalar(alloc11) of type &[u8; 33_usize]`，`::core::write` 内的 byte-string 字面量）
- `deps-complex/trait-serde-generic`（`Scalar(alloc7) of type &[u8; 25_usize]`）
- `deps-complex/bigint-serde` / `chrono-serde` / `collections-serde`（E0277 `str: Sized` 由 serde-derive 展开触发，配 `Unsupported constant expression: "label"` 等）

stderr 特征：

```
error: Unsupported constant value: Scalar(alloc{N}) of type &'?N [u8; M_usize]
error: Unsupported constant expression: "{string}"
```

**归因**：工具不支持。byte-string 字面量 / static-promoted str slice 落在 creusot 当前的常量翻译域之外；serde-derive / `::core::write` / `format!()` 展开生成此类构造。注意按 §六 当前 crate 焦点判据：报错位置在 entry-crate 内（derive 展开是 entry-crate 自身的 token），不属于"外部依赖 stub"豁免。cargo 依赖树（含 vendor 的 `sha2 v0.11.0`、`rsa`、`num-bigint`、`serde`）能编通，creusot 接受 vendor 的 trait/struct 定义；失败点落在用户 entry 代码或其 derive 展开。
**处理**：不修。

### 桶 5：creusot-rustc 内部 panic（4 case）

代表 entry：

- `repr/union/repr_union`（`creusot/src/backend/ty.rs:393:14`）
- `charon-limit/async-fn/async_forty_two` 与 `kani-limit/async-await/run_async_add`（`creusot/src/translation/specification.rs:321:61`）
- `charon-limit/inline-asm/nop_via_asm`（`creusot/src/translation/function/terminator.rs:189:34: asm!("", options())`）

stderr 特征：

```
internal error: entered unreachable code(: ...)
thread 'rustc' panicked at <crate path>:<line>:<col>:
```

**归因**：工具不支持。union / async fn / inline asm 三类构造分别触发 creusot 内部不同 `unreachable!` assertion。按 P31 §四.5：本项目未维护 wrapper、报错来自上游官方 binary 自身——"官方 driver crash"归"工具的锅"，FAILED 站得住，本测试集不替工具修。
**处理**：不修（属上游缺陷，应在上游 issue tracker 反馈）。

### 桶 6：spec 层未覆盖的 std lib 路径（8 case）

代表 entry：

- `bigint/bigint-arith`（`num_bigint::BigInt: creusot_std::model::DeepModel` 未满足）
- `float/cmp` / `float/special-vals` / `float/total-order`（`f64: DeepModel` / `NaN is not yet supported`）
- `collections/btreemap`（`std::collections::btree_map::Iter: IteratorSpec` 未满足）
- `error/result-question`（`std::str::Split<'_, char>: IteratorSpec` 未满足）
- `creusot-limit/generic-for-loop`（`I: IteratorSpec` 泛型边界）
- `deps-complex/itertools-multi`（`Chunks<'_, Copied<Iter<'_, i32>>>: IteratorSpec`）

stderr 特征：

```
error[E0277]: the trait bound `X: creusot_std::prelude::IteratorSpec` is not satisfied
error[E0277]: the trait bound `X: creusot_std::model::DeepModel` is not satisfied
error: NaN is not yet supported
```

**归因**：工具不支持（lib spec 边界）。`creusot_std` 提供的规约 trait 未覆盖 `f64` / `BigInt` / 部分 std iterator——这是 creusot 自陈的 std-coverage 边界（README §"真实失败常见来源"已声明）。错误在 entry-crate 的 trait-bound 检查中触发——不属"外部 stub"豁免。
**处理**：不修。

### 桶 7：其他显式拒（4 case）

代表 entry：

- `charon-limit/arc-slice-unsize/arc_array_to_slice`（`error: unsupported cast from Arc<[u32; 3]> to Arc<[u32]>`）
- `lifetime/thread-local/thread_local_read` 与 `creusot-limit/thread-local-ref/read_thread_local`（`unsupported definition kind ... Static { safety: Safe, mutability: Not, nested: false }`，`thread_local!` 宏展开内部 Static）
- `miri-limit/weak-memory-incomplete/relaxed_load_may_not_observe_all_stores`（顶级 `static SHARED: AtomicU32` 触发同样的 `unsupported definition kind Static`）

stderr 特征：

```
error: unsupported cast from Arc<[T; N]> to Arc<[T]>
error: unsupported definition kind DefId(...) Static { ... }
```

**归因**：工具不支持（Arc unsize coercion + 顶级 / thread-local Static 翻译域外）。
**处理**：不修。

## 漏报盲点（诚实声明）

- 已通过 wrapper gate 封堵：**无**——本工具无项目 wrapper。仅信任 cargo-creusot exit code
- 仍存在的盲点：
  - **盲点 1（实测可信而非源码证明）**：runner 仅检查 exit code，未独立校验 `verif/<crate>_rlib/` 与 `.coma` 文件存在 / 非空 / 入口 fn 已翻译。若 creusot 内部 silent skip 某 fn 但 cargo-creusot 仍 exit 0，本测量会误报 SUCCESS。本 run 161 entries 未观察到此类反例，但**未在源码层证明不可能**
  - **盲点 2（语义边界声明）**：SUCCESS 仅蕴含"创建 Coma 翻译完整无 rustc error"，不蕴含 SMT 求解通过。`cargo creusot prove` 路径在本测试集上无任何观察——SUCCESS 不构成"已证明该样例正确"的判定
  - **盲点 3**：每个 SUCCESS entry stderr 含 `warning: calling external function X with no contract` ——表示 creusot-rustc 接受代码但若进入 prove 阶段需补 contract。本工具未把此 warning 当作 partial 标记 / 升 FAILED——属"前端边界"语义下的设计选择，但仍是"前端通过 ≠ 端到端可证"的盲点表现

修复 backlog：

- (低优先) 加 grep gate 校验 `.coma` 产物存在且非空，关闭盲点 1
- (无需修) 盲点 2 / 3 是前端边界的固有语义，不属于"漏报"——README 与本报告均明确声明

## v5.1 → v6 ΔS 解释

- v5.1: 106 / 146 = 72.6 %
- v6:   121 / 161 = 75.2 %
- ΔS = +15 SUCCESS、ΔF = 0 FAILED、Δtotal = +15 entries

corpus 增量主要来自 v6 新增的 `runnable/` 类目（15 / 15 全 SUCCESS）——这些 entries 都是简单可执行样例，creusot-rustc 翻译完整。其余 feature 的 SUCCESS / FAILED 计数与 v5.1 同分布（40 个 FAILED 完全延续 v5.1 的 40 个，无新增 / 无消解）。

ΔS 不来自工具版本变化或翻译能力提升，纯粹是 corpus 扩张带入的简单样例。

P27-P35 派生原则的影响：

- **P31 §四.5 归因判据**：明确"无项目 wrapper"+"无 corpus lint"+"无环境损坏"→ 全 40 个 FAILED 落在"工具不支持"，无 UNKNOWN 升级候选
- **P35 §六 当前 crate 焦点**：复核所有 FAILED 报错位置在 entry crate 自身（含 derive / macro 展开在 entry-crate 内）→ 无"外部 stub"豁免可应用，FAILED 分类不变
- **bug detect 归 SUCCESS（§四 B 派生）**：creusot 本测试停在 coma 翻译边界，无 prove 阶段 → 该派生原则在 creusot 当前配置下零触发，不影响分类

## 修订建议清单（仅"我们导致"失败）

**无需修订**——所有 40 个 FAILED 均为 creusot 自身的能力边界（工具不支持类构造 / 内部 panic / std spec 未覆盖）。本工具：

- 无项目 wrapper（直接调用上游 binary）
- 无 corpus 引入的 lint / 配置错误
- 无环境损坏

按 P31 §四.5 归因判据，本测试集对 creusot 无"我们导致"的失败可修。FAILED 全部站得住，工具开发者不能驳回。

**唯一可选改进**（非"修"，属可观测性增强）：

| # | 性质 | 描述 | 优先级 |
|---|---|---|---|
| 1 | 可观测性 | 加 `.coma` 产物存在性 grep gate 关闭漏报盲点 1（实测层证据补强，不是治源） | 低 |

## 历史快照声明

本报告记录的所有数字、reject 形态、panic 位置锚定 run `run-1778560393-59119` + cargo-creusot 0.11.0 + nightly-2026-02-27 + creusot-std 0.11.0。creusot 升级后 `forbidden dyn` / `Unsupported pointer cast` / `Unsupported constant` / `DeepModel`/`IteratorSpec` 缺失 / 内部 panic 位置等均可能改写。
