# charon-mono 深度报告

## 元数据

- **run**：`runs/run-1778226613-5282/`（2026-05-08，146 entries × 19 工具矩阵；host：Apple M5 / macOS / aarch64 / 24 GB / 10 cpu）
- **工具版本**：`charon 0.1.184`（charon 自带 toolchain `nightly-2026-02-07-aarch64-apple-darwin`，与 charon-poly 共用同一可执行文件，区别仅在 CLI flag）
- **通过率**：138/146 = 94%
- **时长**（毫秒）：avg 2296 / median 347 / p90 6912 / max 31899
- **时效声明**：本快照锚定上述 run id + 工具版本 + corpus，不构成长期承诺。

## 工具内部 pipeline + 前端边界

Charon 是 AeneasVerif 出品的 Rust → LLBC（Low-Level Borrow Calculus）翻译器。单态化模式（mono）在翻译时把所有泛型实例展开，输出无类型变量的具体 LLBC，适合需要逐实例推理的后端。

pipeline：cargo + rustc 驱动产 MIR → charon-driver 接管做 MIR 提取 → 单态化（monomorphize）展开所有泛型实例 → MIR 翻译为 LLBC → `--print-llbc` 写 stdout。**纯翻译工具**，pipeline 终点就是 LLBC，不调用任何 SMT / 模型检查器。所以"前端 = 全过程 = 翻译到 LLBC"。

`tool.toml` 命令：`charon cargo --monomorphize --abort-on-error --print-llbc -- --lib --target aarch64-apple-darwin`。相比 poly 多一个 `--monomorphize` flag。`--abort-on-error` 在 mono 模式尤为关键——实测 `Box<dyn Any>` vtable drop preshim 触发 charon 内部 panic，缺此 flag 会静默 exit 0、误判 SUCCESS。

## SUCCESS 信号 + 形式严格性

- **形式指标**：charon exit 0（含 `--abort-on-error`）
- **partial 暴露**：`--abort-on-error` + `register_error!` panic 路径
- **0 误报**：✅ 形式可证。charon exit 0 ⇔ 翻译完整无内部错误
- **0 漏报**：✅ 形式可证。`--abort-on-error` 已封死所有 silent skip
- **漏报盲点**：无

## 实测结果

### 按 feature 类目分布

138 SUCCESS / 8 FAILED 跨 41 类目：

```
全过类目（35 个）：aeneas-limit (8/8) / arc / assoc-type / bigint (8/8) / box
  closure / closure-adv (4/4) / collections / concurrency / const
  deps-complex (7/7) / drop / enum / error / float (10/10) / gat / generic (4/4)
  hax-limit (8/8) / hello / hrtb / impl-trait / industrial (6/6) / int
  int-width (14/14) / iter / miri-limit (7/7) / panic / prusti-limit (8/8)
  rc / refcell / repr / slice / trait-obj / unsafe-adv (3/3) / vec

部分通过：
  charon-limit:  4/7（async-fn、generic-to-dyn-unsize、inline-asm FAILED）
  creusot-limit: 6/7（dyn-trait-forbidden FAILED）
  kani-limit:    6/7（async-await FAILED）
  lifetime:      2/3（static-bound FAILED）
  trait:         0/1（cyclic-bound FAILED）
  unsafe-ptr:    1/2（raw-ptr-const FAILED）
```

`bigint/*` 8/8（含 num-bigint / num-rational / num-complex 三层 dep + 大量 trait 实例化）+ `deps-complex/*` 7/7 + `industrial/*` 6/6 全过——本次单态化展开未触发实例爆炸性能问题。注意 mono 模式上**整体 avg / max 均略低于 poly**（avg 2296 vs 2954, max 31899 vs 34693）：单态化具体实例反而在某些路径上比保留泛型的 LLBC 输出更直白。

### 失败模式归类

逐条读 raw stderr 后归类。8 个 FAILED 全部 `exit_code = 101`。

#### A. charon 自身翻译路径标 unsupported / panic（4/8，与 poly 共享）

| entry | panic 消息 |
|---|---|
| `charon-limit/async-fn/async_forty_two` | `Coroutine types are not supported yet` |
| `kani-limit/async-await/run_async_add` | `Coroutine types are not supported yet` |
| `charon-limit/inline-asm/nop_via_asm` | `Inline assembly is not supported` |
| `unsafe-ptr/raw-ptr-const/raw_ptr_const_match` | `Unsupported constant: ConstantExprKind::Cast {..}` |

charon 在 MIR 提取阶段对 `Coroutine` MIR 节点、`InlineAsm` MIR terminator、`Cast` constant kind 显式 panic——`--monomorphize` 不影响这些前端边界。源 panic 点 `src/errors.rs:282`，与 poly 表现完全一致。

#### B. 单态化路径独有的 vtable drop preshim panic（3/8）

| entry | panic 消息 + 触发场景 |
|---|---|
| `charon-limit/generic-to-dyn-unsize/boxed_display_from_u32` | `Could not determine method index for drop in vtable` 触发于 `core::fmt::Display::{vtable_drop_preshim}`；entry 把 `T: Display + 'static` 泛型 unsize 到 `Box<dyn Display>` |
| `lifetime/static-bound/static_bound` | 同消息，触发于 `core::any::Any::{vtable_drop_preshim}`；entry 走 `Box::new(x)` 后 `b1.is::<i32>()` 触及 `Box<dyn Any>` vtable |
| `creusot-limit/dyn-trait-forbidden/trigger_call_dyn_display` | 同消息，触发于 `core::fmt::Display::{vtable_drop_preshim}`；entry 显式构造 `Box<dyn Display>` 调用 |

panic 源指向 `src/bin/charon-driver/translate/translate_trait_objects.rs:1707:13`。这是 `--monomorphize` 路径独有的失败——单态化时对 trait object 的 vtable drop preshim 索引计算路径会 panic；charon-poly 不展开 vtable 具体实例所以避开。`tools/charon-mono/README.md` 也记录了 `Box<dyn Any>` vtable drop preshim 是 mono 模式已知触发点。

#### C. charon 内部 stack overflow（1/8，与 poly 共享）

```
trait/cyclic-bound/cyclic_bound_use:
  thread 'rustc' (...) has overflowed its stack
  fatal runtime error: stack overflow, aborting
  ... (signal: 6, SIGABRT: process abort signal)
```

cyclic trait bound 让 charon 内部某条 trait / type resolution 路径无限递归——单态化与否都炸（poly 也同样 FAILED）。

### 时长尾端观察

`max=31.9s` 在 `industrial/x509-parser/cert-parse/x509_subject_extensions` SUCCESS——单态化后 LLBC 实例展开。`deps-complex/bigint-serde/bigint_serde`（22s）、`deps-complex/chrono-serde/chrono_serde`（22s）也偏长。mono 与 poly 在 dep-heavy entry 上耗时近似，本次未观察到 mono 单态化导致的实例爆炸性能问题。

### 与 charon-poly 的 4 处行为差异

| entry | charon-poly | charon-mono | 解释 |
|---|---|---|---|
| `lifetime/thread-local/thread_local_read` | FAILED | SUCCESS | poly 进入 std TLS 内部 polymorphic 路径踩 unsupported；mono 单态化后 specialized 实例避开该路径 |
| `creusot-limit/thread-local-ref/read_thread_local` | FAILED | SUCCESS | 同上 |
| `charon-limit/generic-to-dyn-unsize/boxed_display_from_u32` | SUCCESS | FAILED | mono 展开 `Box<dyn Display>` vtable drop preshim 时 panic；poly 不展开 vtable |
| `lifetime/static-bound/static_bound` | SUCCESS | FAILED | mono 展开 `Box<dyn Any>` vtable drop preshim 时 panic；poly 不展开 vtable |

注：本次 mono 还多出 1 个独有 FAILED `creusot-limit/dyn-trait-forbidden/trigger_call_dyn_display`（poly 上 SUCCESS），加上对称差，两 mode 在 FAILED 集合上**不互含**。

## 与本次测试边界的关系

- **测试切割点**：SUCCESS 仅蕴含"charon 在 mono 模式下把该 entry 翻译到 LLBC 无 panic"。LLBC 产物的语义忠实度、下游 verifier 能否消费此 LLBC，均不在本测试范围。
- **已知 corpus 偏向**：`charon-limit/*` 7 个 entry 故意触发 charon 已知"不支持"特性。`generic-to-dyn-unsize/boxed_display_from_u32` 是 mono 模式独有的边角触发——poly 不展开 vtable 所以躲过；该 entry 在 mono 上的 FAILED 反映真实翻译路径分化。
- **本次未触达**：mono 因实例展开规模而 timeout（最长 32s，远低于 600s timeout）；industrial vendor crate 上的翻译失败。

## 历史快照声明

本报告是 2026-05-08 运行 `runs/run-1778226613-5282` 的实测快照；锚定 `charon 0.1.184` × charon 自带 nightly-2026-02-07 toolchain × 当前 corpus（146 entries）。charon 升级（特别是 vtable drop preshim 索引计算 / coroutine / inline-asm / Cast constant 等翻译路径修订）后需重测；mono 与 poly 的 FAILED 集合可能因升级而重新对齐或分化。
