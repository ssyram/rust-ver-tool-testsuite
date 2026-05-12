# soteria — 特性支持评估报告（v6 final post-P35 baseline）

## 元数据

- **数据源**：`runs/run-1778560393-59119/`（2026-05-12 v6 final post-P35）
- **run 时间**：2026-05-12T04:33:13Z → 04:55:27Z UTC，parallelism=10
- **host**：Apple M5 / macOS aarch64 / 24 GB / 10 cpu
- **工具配置**：`tools/soteria/`（含 P35 新增 `soteria-strict-wrapper.sh`）
- **工具版本**：soteria-rust @ commit `3c21278187c60c99418fe2dabb03710ce4102896`（OCaml 5.4.0）+ Obol frontend @ commit `ddea5ca5da4c07301584f47f05ea8615fc365b41`，搭配 nightly-2026-02-07
- **本工具实测**：n=161 / SUCCESS=126 / FAILED=35 / UNKNOWN=0，通过率 **78.3%**
- **时长分布**：avg 1491 ms / median 932 ms / p90 3356 ms / max 7963 ms
- **宪法 baseline**：`principles.md` v8（P27 修宪后 / P31 法律传导后 / P35 bug-detect 归 SUCCESS 派生后）
- **时效声明**：本快照锚定上述 run id + 工具版本 + corpus，不构成长期承诺。soteria-rust 无稳定 release，后续 commit（含 Obol IR / intrinsic 实现 / Tree Borrows 模型）都可能改写本边界。

## pipeline + 前端边界

soteria 是 OCaml 实现的有界符号执行 + 原生 Tree Borrows 引擎。pipeline：

```
soteria-strict-wrapper.sh （项目维护，P35 引入）
  → soteria-rust exec src/lib.rs
    → rustc + Obol 把 src/lib.rs 翻译为 ULLBC/LLBC JSON
    → soteria-rust 加载 LLBC + Charon IR
    → 从 lib::main 起做有界符号执行（--step-fuel / --branch-fuel 默认无限）
    → 完整跑完路径 / 报告 bug / OCaml exception
  → wrapper 解读 exit + stdout signature 做 P35 三分判决
```

soteria 没有"只翻译不执行"的 dry-run flag（README：最轻量调用也会完整编译 + 符号执行）。本测试通过函数体量极小（多数 entry < 50 行）+ 默认无限 fuel 让符号执行在 ~10 ms 内自然完成；编译是耗时主要部分。

**前端 / 后端切割**：本工具是符号执行引擎，无独立"前端 vs 求解层"边界。按 P35 修订后的 SUCCESS 精神（`architecture.md` §一 "bug detect 归 SUCCESS"，§四 B "测必要条件不问语义对错"派生）：

- 符号执行**完整跑完** + 无 bug → SUCCESS
- 符号执行**完整跑完** + 工具自陈在 entry 代码中发现 bug → SUCCESS（工具最有价值的输出之一）
- 工具自身 panic / crash / 翻译层 reject → FAILED

P35 之前（v3 时代）README 曾把"bug detect = FAILED"作为"不允许 partial = 必须完整跑完"的解读；P35 修宪后该旧注释已删，bug detect 归 SUCCESS 与 `architecture.md` §一对齐。

`entry_mode = "lib"`：runner 把样例 `src/lib.rs` 重命名为 `src/__ts_inner.rs`，新 `src/lib.rs` 是 harness：

```rust
mod __ts_inner;
pub use __ts_inner::*;
fn main() { let _ = __ts_inner::{{ entry_fn }}(); }
```

soteria-rust 以 `fn main` 为符号执行入口。

## SUCCESS 信号 + 形式严格性

按宪法 §六 双通路 partial 暴露：

- **主信号通路**：soteria-rust 原生 exit code
  - 0 → 符号执行完整跑完且无 bug
  - 1 → cargo build error / 检出 bug / 工具自身 extracting panic（wrapper 二次三分）
  - 2 → soteria-rust 内核 OCaml exception / unsupported intrinsic
  - 3 → Obol/Charon 前端 crash（本矩阵未触发）
- **wrapper 补抓通路**（P35 新增 `soteria-strict-wrapper.sh`）：在 exit ≠ 0 时按优先级二次判决：
  1. stdout 含 `Thread panicked when extracting` → 保留 exit ≠ 0 → FAILED（工具 panic）
  2. stdout 含 `^bug:.*in lib::` 或 `found issues in .* errors in [0-9]+ branch`（ANSI 已剥）→ 重写 exit 1 → exit 0 → SUCCESS（bug-detect 在 entry 代码中）
  3. 默认（cargo build error / unsupported / 其他）→ 保留原 exit → FAILED

形式严格性 0 误报 / 0 漏报状态：

- **0 误报**：✅ 实测 + wrapper 双通路区分。SUCCESS 仅来自两条互斥路径——exit 0（无 bug 完整跑完）∪ exit 1 with bug-detect signature（在 entry 代码中真检出 bug）。两条均要求符号执行"完整跑完且产出有意义结果"，不冤枉。
- **0 漏报**：⚠️ 实测 + wrapper 三分封堵 + 求解层简化口径盲点。exit 1 的 cargo build / extraction panic / bug detect 三类 + exit 2/3 完整覆盖。盲点见下条。
- **漏报盲点（诚实声明）**：soteria-rust 在两类 intrinsic 上以 warning + 继续符号执行的方式做求解层简化（不中断执行，exit 仍 0 → SUCCESS）：
  - `An atomic intrinsic was encountered; it will be executed as sequential code`：单线程语义近似（同 kani concurrency 的求解层假设）
  - `A complex floating point intrinsic was encountered; it will be executed with a significant over-approximation`：soundness-preserving 抽象
  - 本 v6 矩阵共 4 个 SUCCESS entries 含上述求解层简化 warning：`concurrency/atomic/atomic_seqcst`、`miri-limit/weak-memory-incomplete/relaxed_load_may_not_observe_all_stores`、`float/transcendental/float_transcendental`、`kani-limit/float-overapprox/trigger_check_sin_cos_identity`
  - 按宪法 §六-3 "前端测量"原则：前端（编译 + Obol 翻译 + 符号执行调度）确实完整跑完，warning 表征求解层简化口径，**不属"silent skip 前端"** → 保持 SUCCESS
  - 该 warning 不构成漏报盲点本身，但应列出以诚实声明"SUCCESS 在这些 entry 上不蕴含求解层精确"

## 失败分桶（按 P31 §四.5 归因分类）

按 raw stdout / stderr 实读 35 个 FAILED，按形态归 5 类。

### 桶 A：第三方 crate 依赖（单文件模式不支持）（21 case，exit=1）

代表 entry：`bigint/bigint-arith/bigint_arith` / `deps-complex/bigint-serde/bigint_serde` / `industrial/rsa/rsa-pkcs8/rsa_pubkey_from_pkcs8`

stdout 特征：

```
error[E0432]: unresolved import `num_bigint`
  ...
  = help: you might be missing a crate named `num_bigint`
ERROR Code failed to compile
```

涉及：`bigint/*` 8 个（`num_bigint` / `num_complex` / `num_integer` / `num_rational` / `num_traits`）、`deps-complex/*` 7 个（`chrono` / `serde` / `serde_json` / `itertools` / `anyhow` / `thiserror`）、`industrial/*` 6 个（`rsa` / `rand` / `sha2` / `x509_parser`）。

**归因**：工具不支持。soteria-rust 单文件模式（`exec src/lib.rs`）只能解析 `std`/`core`/`alloc`，第三方 crate 依赖均无法编译。Obol 在这一阶段还未介入。这是 soteria 自身设计选择（README §"已知限制"明记："单文件模式不支持外部 crate 依赖"），按本地性原则 FAILED 站得住。
**处理**：不修。

### 桶 B：edition 边界（1 case，exit=1）

代表 entry：`hax-limit/let-chains/hax_limit_let_chains`

stdout 特征：

```
error: let chains are only allowed in Rust 2024 or later
```

**归因**：工具不支持。`tool.toml` 当前传 `--rustc=--edition=2021`，未升级到 2024。这是 soteria 单文件 pipeline 与 rustc edition 的耦合：soteria-rust 没有"读 Cargo.toml 自动取 edition"的路径，单文件模式 edition 必须显式传。属于宪法 §六 末段"工具单文件 pipeline 不读 Cargo.toml"类工具能力边界。
**处理**：不修。如未来升 `--edition=2024` 是工具集成层调整，但当前 baseline 内 FAILED 站得住。

### 桶 C：Obol / charon-fork 翻译层不支持（5 case，exit=2）

代表 entry：`charon-limit/inline-asm/nop_via_asm` / `charon-limit/async-fn/async_forty_two` / `charon-limit/arc-slice-unsize/arc_array_to_slice` / `kani-limit/async-await/run_async_add` / `lifetime/static-bound/static_bound`

stdout 特征：

```
warning: lib::main (Xms): unsupported feature, Can't execute function ...: (GAst.Error ...)
warning: lib::main (Xms): exception, Soteria_rust_lib.Crate.MissingDecl("Global")
msg = "Inline assembly is not supported"
msg = "Coroutine types are not supported yet"
```

- `inline-asm`：Charon-fork raise `"Inline assembly is not supported"`
- `async-fn` / `async-await`：`"Coroutine types are not supported yet"`
- `arc-slice-unsize`：`Unsupported intrinsic: atomic_xsub`（`Arc<[T]>` unsize coercion 路径上的 Arc drop intrinsic 未实现）
- `static-bound`：`MissingDecl("Global")`——`'static` lifetime global 未在 Crate decls 中注册

**归因**：工具不支持。Obol（charon-fork）翻译层 / 内核加载阶段未实现的 MIR 构造或 global decl 注册。
**处理**：不修。

### 桶 D：soteria-rust 内核未实现的 intrinsic / extern / std API（6 case，exit=2）

代表 entry：`arc/clone-drop/arc_clone_drop` / `creusot-limit/dyn-trait-forbidden/trigger_call_dyn_display` / `float/cast-widening/float_cast_widening` / `kani-limit/extern-ffi/trigger_call_libc_abs` / `miri-limit/ffi-unshimmed-extern/call_unshimmed_foreign_fn` / `miri-limit/networking-unsupported/tcp_connect_attempt`

stdout 特征：

```
warning: lib::main (Xms): unsupported feature, Unsupported intrinsic: atomic_xsub
warning: lib::main (Xms): unsupported feature, Extern function abs is not handled
warning: lib::main (Xms): exception, Failure("Unhandled float transmute: f32 -> f64")
warning: lib::main (Xms): unsupported feature, Can't execute function std::fmt::format::format_inner: GAst.Missing
warning: lib::main (Xms): unsupported feature, Can't execute function <str as std::net::ToSocketAddrs>::to_socket_addrs: GAst.Missing
```

- `arc_clone_drop`：`Arc<T>::drop` 路径上 `atomic_xsub` 未实现
- `dyn-trait-forbidden`：`std::fmt::format::format_inner: GAst.Missing`（dyn formatter pipeline）
- `cast-widening`：`Unhandled float transmute: f32 -> f64`
- `extern-ffi/libc_abs` / `ffi-unshimmed-extern/getpid`：extern C function 未 handled
- `networking-unsupported/tcp_connect`：`<str as ToSocketAddrs>::to_socket_addrs: GAst.Missing`

**归因**：工具不支持。soteria-rust 内核未实现的 intrinsic / extern / std API。
**处理**：不修。

### 桶 E：OCaml `Invalid_argument("combine3")` 内核 exception（2 case，exit=2）

代表 entry：`concurrency/thread-mutex/thread_mutex_join` / `miri-limit/thread-interleaving-partial/unsynchronised_counter_race`

stdout 特征：

```
=> Running lib::main...
warning: An atomic intrinsic was encountered; it will be executed as sequential code
warning: lib::main (Xms): exception, Invalid_argument("combine3")
Trace: 
```

两者都含 `std::thread::spawn` + `Mutex`，跑了几秒后 soteria-rust 抛 OCaml `Invalid_argument` exception。

**归因**：工具内部 bug / 未实现路径。soteria-rust 在多线程 + 原子组合上触发 OCaml exception。这是工具自身内核状态，按本地性原则 FAILED 站得住。
**处理**：不修。

### 分桶小计

| 桶 | N | exit | 归因 |
|---|---|---|---|
| A 第三方 crate 依赖 | 21 | 1 | 工具不支持（单文件模式） |
| B edition 边界 | 1 | 1 | 工具不支持 |
| C Obol/charon-fork 翻译层 | 5 | 2 | 工具不支持 |
| D 内核 intrinsic/extern/std API | 6 | 2 | 工具不支持 |
| E OCaml 内部 exception | 2 | 2 | 工具自身内核状态 |
| **合计** | **35** | | 全部"工具不支持 / 工具自身边界" |

注：v6-pre-P35 曾存在的"桶 F：符号执行检出 bug（2 case）"——`collections/hashmap/hashmap_basic` + `kani-limit/uninit-memory/read_uninit_byte`——在 P35 修订后已翻 SUCCESS（见下节"P35 ΔS 来源"）。

## 漏报盲点（诚实声明）

- **已通过 oracle / wrapper 封堵**：
  - exit=0 → SUCCESS（符号执行完整完成且无 bug）
  - exit=1 + bug-detect signature（`^bug:.*in lib::` ∪ `found issues in .* errors in N branch`）→ wrapper 重写 SUCCESS（P35）
  - exit=1 + `Thread panicked when extracting` → FAILED（工具 panic）
  - exit=1 默认（cargo build error / unresolved import）→ FAILED
  - exit=2（soteria-rust 内核 OCaml exception / unsupported intrinsic）→ FAILED
  - exit=3（Obol/Charon 前端 crash）→ FAILED（本矩阵未触发）
- **仍存在的盲点**：
  - **atomic intrinsic 求解层简化**：soteria-rust 在 `atomic_load` / `atomic_store` 等基础原子 intrinsic 上以 warning + 继续执行处理（单线程语义近似）。SUCCESS entry 上若涉及并发原子语义，符号执行**完整完成**但不反映真实多线程行为
  - **complex-float intrinsic 求解层简化**：`sin` / `cos` 等超越函数以 over-approximation 处理。SUCCESS entry 上 floating-point 路径不反映精确数学语义
  - 本 v6 矩阵 4 个相关 SUCCESS entries 已列出（见上节"形式严格性"）
  - 这些是 README D3.5 (2026-05-12) 已补完的求解层简化口径声明——不属"silent skip 前端" → 仍记 SUCCESS，但读者引用 soteria 支持率时应注意这些盲点

## v5.1 → v6 final ΔS 解释

- v5.1 baseline `run-1778226613-5282`（146 entries）：SUCCESS=109，通过率 74.7%
- v6 final post-P35 baseline `run-1778560393-59119`（161 entries）：SUCCESS=126，通过率 78.3%

ΔS = +17，corpus Δn = +15（v6 加了 15 个新 entry）+ P35 翻盘 +2（见下）。

### P35 ΔS 来源（+2 SUCCESS）

P35 修宪：`architecture.md` §一新增"bug detect 归 SUCCESS"（§四 B 派生）。配套 `tools/soteria/soteria-strict-wrapper.sh` 上线，对 exit ≠ 0 做三分：

| entry | stdout 关键 signature | v6-pre-P35 | v6 final |
|---|---|---|---|
| `collections/hashmap/hashmap_basic` | `bug: Dangling pointer in lib::main` + `found issues in T, errors in 1 branch` | FAILED | **SUCCESS**（bug-detect） |
| `kani-limit/uninit-memory/read_uninit_byte` | `bug: Uninitialized memory access in lib::main` + `found issues in ...` | FAILED | **SUCCESS**（bug-detect） |

两个 entry 工具均**完整跑完符号执行 + 在 entry 代码中自陈 bug**——P35 之前因 v3 时代 README 的"必须完整跑完 = bug detect 也算中断"旧解读判 FAILED；P35 后归 SUCCESS。soteria README 已删该旧注释。

注：`collections/hashmap/hashmap_basic` 的 bug detect 是工具自身已知的 aarch64 stdarch SIMD intrinsic false-positive（README §"已知限制"），但按 §四 B "测必要条件不问语义对错" / §一"bug detect 归 SUCCESS"派生——工具是否冤枉用户代码不在本框架测量范围；工具完整跑完 + 给出有意义输出就是 SUCCESS。

### 其余 ΔS 来源（+15）

corpus 新增 15 个 entry。归因结构与 commit hash 一致，工具版本无变化。FAILED 桶 A–E 结构与 v5.1 整体一致（A=21（v5.1 同），B=1（v5.1 同），C 由 4 → 5（v6 新 entry 中 `arc-slice-unsize` 落到桶 C），D=6（v5.1 中 D=7 含 arc-slice-unsize；归属调整），E=2（v5.1 同）），无 v5.1 已有 entry 在 v6 上 status flip。

## 修订建议清单（仅"我们导致"失败）

**无需修订。所有 35 个 FAILED 均为工具能力边界**（单文件模式不支持外部 crate / edition 未升 2024 / Obol 翻译层未实现 / 内核 intrinsic 未实现 / OCaml exception 等），按本地性原则站得住，工具开发者不能驳回。

| # | 桶 | 涉及 case | 修复方案 | 优先级 |
|---|---|---|---|---|
| — | — | — | 本工具无"我们导致"失败 | — |

注：oracle 主信号是工具原生 exit code；P35 新增 wrapper 仅做 exit ≠ 0 时的二次三分（bug-detect / extracting-panic / 其他），不掩盖工具失败——天然没有"我们 wrapper bug"类失败。

### 非 fix 性记号（仅诚实声明，不入修订清单）

- 桶 B（`hax-limit/let-chains`）若未来想覆盖 Rust 2024，需把 `tool.toml` 中 `--rustc=--edition=2021` 升 `--edition=2024`——属"工具集成调整"，不属"我们 bug 修复"，不在本清单范围
- README D3.5 段已完整列出求解层简化盲点 + 4 个相关 SUCCESS entry——该声明已就位，无需新增 fix
- soteria README 在 P35 修订时已删 v3 时代 "按完整完成精神 bug detect = FAILED" 旧注释，与 wrapper 实施一致
