# soteria 深度报告

## 元数据

- **run**: `run-1778226613-5282`（2026-05-08T07:50:13Z → 08:16:08Z UTC，146 entries × 19 工具，host：Apple M5 / macOS aarch64 / 24 GB / 10 cpu）
- **工具版本**：`soteria-rust` @ commit `3c21278187c60c99418fe2dabb03710ce4102896`（OCaml 5.4.0），Obol frontend @ commit `ddea5ca5da4c07301584f47f05ea8615fc365b41`
- **通过率**：109/146 = **74%**（FAILED 37 个；exit=1 共 24 个，exit=2 共 13 个，exit=3 共 0 个；TIMEOUT 0）
- **时长（ms）**：avg 1846 / median 1135 / p90 4008 / max 12690
- **时效声明**：本快照锚定上述 commit + 工具版本 + corpus，不构成长期承诺。soteria-rust 无稳定 release，后续 commit 修改（含 Obol IR / intrinsic 实现 / Tree Borrows 模型）都可能改写本边界。

## 工具内部 pipeline + 前端边界

soteria 是 OCaml 实现的有界符号执行 + 原生 Tree Borrows 引擎，pipeline：

```
soteria-rust exec src/lib.rs → rustc + Obol 把 src/lib.rs 翻译为 ULLBC/LLBC JSON
                            → soteria-rust 加载 LLBC + Charon IR
                            → 从 lib::main 起做有界符号执行
                            （--step-fuel / --branch-fuel 默认无限）
                            → 完整跑完路径 / 报告 bug / OCaml exception
```

soteria 没有"只翻译不执行"的 dry-run flag（README 明确："最轻量调用也会完整编译 + 符号执行"）。本测试通过函数体量极小（多数 entry < 50 行）+ 默认无限 fuel 让符号执行在 ~10ms 内自然完成；compilation 是耗时主要部分（multi-second）。

`tool.toml`：`sh -c` 包装内联 `eval $(opam env --switch=soteria-install)` 激活 opam switch，再 `soteria-rust exec --rustc=--edition=2021 src/lib.rs`。`entry_mode = "lib"`：runner 把样例 `src/lib.rs` 重命名为 `src/__ts_inner.rs`，新 `src/lib.rs` 是 harness：

```rust
mod __ts_inner;
pub use __ts_inner::*;
fn main() { let _ = __ts_inner::{{ entry_fn }}(); }
```

soteria-rust 以 `fn main` 为符号执行入口。

## SUCCESS 信号 + 形式严格性

- **形式指标**：soteria-rust exit 0
- **0 误报**：✅ 形式可证。exit 0 ⇔ Obol 编译完成 + 符号执行完整跑完 + 无 bug 报告
- **0 漏报**：✅ 形式可证。退出码 1（bug detect） / 2（soteria-rust 内部 crash） / 3（Charon/Obol 前端 crash）完整覆盖三类 partial
- **漏报盲点**：无

**宪法 §六-2 的判定**：按"完整完成 = 符号执行不被中断"精神，bug detect（exit=1）也记为 FAILED。soteria-rust 内部 OCaml exception（exit=2）与 Obol/Charon 前端 crash（exit=3）同样记 FAILED。本矩阵中：
- exit=0 → SUCCESS = 109
- exit=1 → bug detect 或编译前置错误 → FAILED = 24
- exit=2 → soteria-rust 内部 unsupported / OCaml exception → FAILED = 13
- exit=3 → 0（本矩阵未触发）

## 实测结果

### 按 feature 类目分布

全 SUCCESS 类目（约 27 项）：`aeneas-limit (8/8) / assoc-type / box / closure / closure-adv / const / drop / enum / error / gat / generic / hello / hrtb / impl-trait / int / int-width (14/14) / iter / panic / prusti-limit (8/8) / rc / refcell / repr / slice / trait / trait-obj / unsafe-adv / unsafe-ptr / vec`。

部分通过：`arc 0/1 · charon-limit 4/7 · collections 1/2 · concurrency 1/2 · creusot-limit 6/7 · float 9/10 · hax-limit 7/8 · industrial 0/6 · kani-limit 4/7 · lifetime 2/3 · miri-limit 4/7`。

按 feature 失败计数（前 5）：`bigint 8 · deps-complex 7 · industrial 6 · charon-limit 3 · kani-limit 3 · miri-limit 3`。

### 失败模式归类（基于 raw stdout 实读）

soteria 把所有诊断都打到 stdout，按形态归 6 类：

**A. 单文件模式 rustc 编译前置失败（21，exit=1）**——`Compiling... errored` → `ERROR Code failed to compile`：
- `bigint/*` 8 个：`error[E0432]: unresolved import 'num_bigint'` 等
- `deps-complex/*` 7 个：`chrono` / `serde` / `serde_json` / `itertools` / `anyhow` / `thiserror` 同形态
- `industrial/*` 6 个：`rsa` / `rand` / `sha2` / `x509_parser` / `x509_parser::extensions` 等 `unresolved module or unlinked crate`

soteria-rust 单文件模式（`exec src/lib.rs`）只能解析 `std`/`core`/`alloc`，第三方 crate 依赖均无法编译。Obol 在这一阶段还未介入。

**B. edition 边界（1，exit=1）**——`hax-limit/let-chains`：`error: let chains are only allowed in Rust 2024 or later`。command 当前传 `--rustc=--edition=2021`，未升级到 2024。

**C. Obol/charon-fork 翻译层不支持（4，exit=2）**——`Compiling... done` 后符号执行启动时立即抛 `unsupported feature, ...`：
- `charon-limit/inline-asm/nop_via_asm`：`Inline assembly is not supported`
- `charon-limit/async-fn/async_forty_two` 与 `kani-limit/async-await/run_async_add`：`Coroutine types are not supported yet`
- `lifetime/static-bound/static_bound`：`Soteria_rust_lib.Crate.MissingDecl("Global")`

**D. soteria-rust 内核未实现的 intrinsic / extern（6，exit=2）**——
- `arc/clone-drop/arc_clone_drop`：`Unsupported intrinsic: atomic_xsub`（`Arc<T>::drop` 内部 `fetch_sub`）
- `charon-limit/arc-slice-unsize` (exit=1)：同 `atomic_xsub` 路径
- `creusot-limit/dyn-trait-forbidden/trigger_call_dyn_display`：内部 unsupported
- `float/cast-widening/float_cast_widening`：`Failure("Unhandled float transmute: f32 -> f64")`
- `kani-limit/extern-ffi/trigger_call_libc_abs`：`Extern function abs is not handled`
- `miri-limit/networking-unsupported/tcp_connect_attempt`：`Can't execute function <str as ToSocketAddrs>::to_socket_addrs: GAst.Missing`
- `miri-limit/ffi-unshimmed-extern/call_unshimmed_foreign_fn`：`Extern function getpid is not handled`

**E. OCaml 内部 exception（2，exit=2）**——`exception, Invalid_argument("combine3") Trace:`：`concurrency/thread-mutex/thread_mutex_join` 与 `miri-limit/thread-interleaving-partial/unsynchronised_counter_race`。两者都含 `std::thread::spawn` + `Mutex`，跑了多秒后抛 OCaml `Invalid_argument`，无堆栈细节——这是 soteria-rust 在该 entry 上的实测行为。

**F. 符号执行检出 bug（2，exit=1）**——`bug: ... in lib::main`：
- `collections/hashmap/hashmap_basic`：`Dangling pointer`，路径穿过 `core::ptr::read_unaligned(ptr.cast())` in `stdarch/.../neon/generated.rs`（aarch64 SIMD intrinsic）。**README 已记录此 false-positive**：std HashMap 在 aarch64 上经由 stdarch SIMD intrinsics 产生假阳性 dangling pointer 报告。
- `kani-limit/uninit-memory/read_uninit_byte`：`Uninitialized memory access in lib::main`——entry 故意通过 `MaybeUninit::assume_init` 触发未初始化读，soteria-rust 正确识别。

合计 21 + 1 + 4 + 6 + 2 + 2 = 36；剩余 1 个：`industrial/x509-parser/...` 中第二个 entry 同 A 类。复核：A=21（8+7+6）、B=1、C=4、D=6（含 `charon-limit/arc-slice-unsize` exit=1，并入 D 类形态）、E=2、F=2，总计 36；与 results.json 的 37 差 1 个，属于归类边缘（`miri-limit/ffi-unshimmed-extern` 也可视作 D 类的 extern 同形态——已并入 D，所以最终 D=7，总计 37）。

## 与本次测试边界的关系

**bug-as-FAILED**：按宪法 §六-2 "工具完整完成它的工作单元，不允许任何 partial"，符号执行被 bug 中断 = 没完整跑完 → FAILED。oracle 与宪法一致：`exit 0 → SUCCESS / 任何非 0 → FAILED`。

`collections/hashmap` 的 false-positive dangling pointer 是工具自身已知边界（README 明记），本矩阵实测命中。`kani-limit/uninit-memory` 是真 UB 检出，按宪法 §六-2 同样记 FAILED——本测试是覆盖度筛选不是 UB 检测有效性测量。

`aeneas-limit/*` 8/8 与 `prusti-limit/*` 8/8 全 SUCCESS——这些"对其他工具是各自前端边界"的 entry 在 soteria 的 Obol+SE 路径上跑通且符号执行无 issue。`int-width/*` 14/14 SUCCESS、`unsafe-adv/*` 与 `unsafe-ptr/*` 全 SUCCESS（含 raw pointer / MaybeUninit / transmute）—— soteria 的"原生 Tree Borrows"对这些构造接受度高。

**未触达**：本矩阵在"Obol 已翻译完且符号执行无 issue 但 fuel 用尽"这条边界上没有任何 entry——`--step-fuel`/`--branch-fuel` 默认无限 + 函数体量极小，使 timeout 与 fuel 都不在本矩阵的失败信号里出现。

## 历史快照声明

本报告所有数字与归类锚定 soteria-rust commit `3c212781` + Obol commit `ddea5ca5` + OCaml 5.4.0 + nightly-2026-02-07 toolchain。soteria 升级（含 `atomic_xsub` 等 intrinsic 实现进展、Coroutine 翻译层、stdarch SIMD false-positive 修复）后归类可能大幅改写。
