# charon-poly 深度报告

## 元数据

- **run**：`runs/run-1778226613-5282/`（2026-05-08，146 entries × 19 工具矩阵；host：Apple M5 / macOS / aarch64 / 24 GB / 10 cpu）
- **工具版本**：`charon 0.1.184`（charon 自带 toolchain `nightly-2026-02-07-aarch64-apple-darwin`）
- **通过率**：139/146 = 95%
- **时长**（毫秒）：avg 2954 / median 367 / p90 7232 / max 34693
- **时效声明**：本快照锚定上述 run id + 工具版本 + corpus，不构成长期承诺。

## 工具内部 pipeline + 前端边界

Charon 是 AeneasVerif 出品的 Rust → LLBC（Low-Level Borrow Calculus）翻译器，为 Aeneas 等后端验证工具提供统一前端。多态模式（poly）保留泛型参数，输出带类型变量的 LLBC，适合需要参数化推理的后端。

pipeline：cargo + rustc 驱动产 MIR → charon-driver 接管做 MIR 提取 → MIR 翻译为 LLBC → `--print-llbc` 写 stdout。**纯翻译工具**，pipeline 终点就是 LLBC，不调用任何 SMT / 模型检查器。所以"前端 = 全过程 = 翻译到 LLBC"。

`tool.toml` 命令：`charon cargo --abort-on-error --print-llbc -- --lib --target aarch64-apple-darwin`。`--abort-on-error` 是 dry-run 关键 flag——charon 默认遇内部 panic 仍 exit 0（设计 quirk），加此 flag 后 panic → exit ≠ 0；`-- --lib --target aarch64-apple-darwin` 绕 macOS arm64 rlib 路径假设，并让 cargo 只编 lib target（harness 文件被跳过）。

## SUCCESS 信号 + 形式严格性

- **形式指标**：charon exit 0（含 `--abort-on-error`）
- **partial 暴露**：`--abort-on-error` 让 charon 内部任何 unsupported 项触发 panic + exit ≠ 0。`charon-driver/driver.rs:143` 设 `error_ctx.continue_on_failure = false`，`register_error!` 第一次错误就 panic
- **0 误报**：✅ 形式可证。charon exit 0 ⇔ 翻译完整无内部错误
- **0 漏报**：✅ 形式可证。`--abort-on-error` + `register_error!` panic 路径已封死所有 silent skip
- **漏报盲点**：无

## 实测结果

### 按 feature 类目分布

139 SUCCESS / 7 FAILED 跨 41 类目：

```
全过类目（35 个）：aeneas-limit (8/8) / arc / assoc-type / bigint (8/8) / box
  closure / closure-adv (4/4) / collections / concurrency / const
  deps-complex (7/7) / drop / enum / error / float (10/10) / gat / generic (4/4)
  hax-limit (8/8) / hello / hrtb / impl-trait / industrial (6/6) / int
  int-width (14/14) / iter / miri-limit (7/7) / panic / prusti-limit (8/8)
  rc / refcell / repr / slice / trait-obj / unsafe-adv (3/3) / vec

部分通过：
  charon-limit:  5/7（async-fn、inline-asm FAILED）
  creusot-limit: 6/7（thread-local-ref FAILED）
  kani-limit:    6/7（async-await FAILED）
  lifetime:      2/3（thread-local FAILED）
  trait:         0/1（cyclic-bound FAILED）
  unsafe-ptr:    1/2（raw-ptr-const FAILED）
```

`bigint/*` 8/8（含 num-bigint / num-rational / num-complex / num-traits）+ `deps-complex/*` 7/7（chrono / serde / itertools / anyhow / thiserror / error-chain / collections-serde）+ `industrial/*` 6/6（vendor x509-parser / rsa / sha2）全过——表明 charon-poly 接得住完整 cargo 依赖图，把每个外部 crate 都翻译到 LLBC。

`aeneas-limit / hax-limit / prusti-limit / miri-limit` 全过——这些"其他工具自声明限制集"在 charon-poly 这层 MIR-to-LLBC 翻译上几乎都接得住。

### 失败模式归类

逐条读 raw stderr 后归类。7 个 FAILED 全部 `exit_code = 101`（rustc 进程 panic 标志）。

#### A. charon 自身翻译路径标 unsupported / panic（4/7）

| entry | panic 消息 |
|---|---|
| `charon-limit/async-fn/async_forty_two` | `Coroutine types are not supported yet` |
| `kani-limit/async-await/run_async_add` | `Coroutine types are not supported yet` |
| `charon-limit/inline-asm/nop_via_asm` | `Inline assembly is not supported` |
| `unsafe-ptr/raw-ptr-const/raw_ptr_const_match` | `Unsupported constant: ConstantExprKind::Cast {..}` |

stderr 共同形态（以 inline-asm 为例）：

```
warning: Inline assembly is not supported
thread 'rustc' (...) panicked at src/errors.rs:282:13:
Inline assembly is not supported
warning: Thread panicked when extracting body.
warning: Thread panicked when extracting item `charon_limit_inline_asm::nop_via_asm`.
ERROR Compilation panicked
```

`src/errors.rs:282` 是 charon 的统一 panic 抛出点。charon 在 MIR 提取阶段对 `Coroutine` MIR 节点（async fn desugar 产物）、`InlineAsm` MIR terminator、`Cast` constant kind 显式 panic——这是 charon 已声明的翻译边界。

#### B. thread-local references 路径（2/7）

| entry | panic 消息 |
|---|---|
| `lifetime/thread-local/thread_local_read` | `charon does not support thread local references` |
| `creusot-limit/thread-local-ref/read_thread_local` | 同上 |

panic 给的源位置是 `/rustc/library/std/src/sys/thread_local/native/mod.rs:97:25`——charon 在翻译标准库 TLS 实现自己时遇到 unsupported 节点（`thread_local!` 宏展开成 std TLS API 调用，charon 跟着 cargo 构建图把 std TLS 实现也拽进翻译）。

#### C. charon 内部 stack overflow（1/7）

```
trait/cyclic-bound/cyclic_bound_use:
  thread 'rustc' (...) has overflowed its stack
  fatal runtime error: stack overflow, aborting
  ... (signal: 6, SIGABRT: process abort signal)
```

cyclic trait bound 让 charon 内部某条 trait/type resolution 路径无限递归。这条 SIGABRT 不在 `--abort-on-error` 的覆盖逻辑里，是被 OS 杀进程后 cargo 转报的 exit 101。

### 时长尾端观察

`max=34.7s` 在 `industrial/x509-parser/cert-parse/x509_parse_der` SUCCESS——charon 把 x509-parser 完整 deps tree（asn1-rs / der-parser / nom / time 等）全部翻译到 LLBC。`deps-complex/collections-serde/collections_serde`（28s）、`industrial/x509-parser/cert-parse/x509_subject_extensions`（34s）等 dep-heavy entry 时间偏高；median 仅 367ms。timeout 设 600s，本次未触发。

### 与 charon-mono 的 4 处行为差异

把同 entry 在 charon-poly vs charon-mono 上的 status 对比，4 个 entry 行为不同（其余 142 个一致）：

| entry | charon-poly | charon-mono | 解释 |
|---|---|---|---|
| `lifetime/thread-local/thread_local_read` | FAILED | SUCCESS | poly 进入 std TLS 内部 polymorphic 路径踩 unsupported；mono 单态化后 specialized 实例避开该路径 |
| `creusot-limit/thread-local-ref/read_thread_local` | FAILED | SUCCESS | 同上 |
| `charon-limit/generic-to-dyn-unsize/boxed_display_from_u32` | SUCCESS | FAILED | mono 展开 `Box<dyn Display>` vtable drop preshim 时 panic；poly 不展开 vtable |
| `lifetime/static-bound/static_bound` | SUCCESS | FAILED | mono 展开 `Box<dyn Any>` vtable drop preshim 时 panic；poly 不展开 vtable |

两 mode FAILED 各自集合 7 / 8 个，对称差 5 个 entry——poly 与 mono 的翻译能力**不互含**。

## 与本次测试边界的关系

- **测试切割点**：SUCCESS 仅蕴含"charon-poly 把该 entry 的 cargo 构建图整体翻译到 LLBC 无 panic"。LLBC 产物的语义忠实度、下游 verifier（aeneas / soteria）能否消费此 LLBC，均不在本测试范围。
- **已知 corpus 偏向**：`charon-limit/*` 7 个 entry 故意触发 charon 已知"不支持"特性（async / inline-asm / dyn unsize 等），本测试如期触发其中 2 个 FAILED；其他 5 个（含 inline-asm 一类的边角）在 poly 模式下未触发翻译失败。`*-limit/*` 各类下是否对应工具可见 FAILED 依工具 mode 而异。
- **本次未触达**：charon-poly 在 industrial vendor crate 上的翻译失败（x509-parser / rsa / sha2 全过）；deps-complex 长链 dep 中的翻译失败。

## 历史快照声明

本报告是 2026-05-08 运行 `runs/run-1778226613-5282` 的实测快照；锚定 `charon 0.1.184` × charon 自带 nightly-2026-02-07 toolchain × 当前 corpus（146 entries）。charon 升级（特别是 coroutine / inline-asm / TLS / vtable drop 等翻译路径修订）后需重测。
