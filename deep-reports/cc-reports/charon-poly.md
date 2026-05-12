# charon-poly — 特性支持评估报告（v6 baseline）

## 元数据

- **数据源**：`runs/run-1778560393-59119/`（2026-05-12 v6 final，合并 verus rerun + R7 5-tool rerun）
- **工具配置**：`tools/charon-poly/`
- **工具版本**：`charon 0.1.184`（charon 自带 toolchain `nightly-2026-02-07-aarch64-apple-darwin`）
- **本工具实测**：n=161 / SUCCESS=154 / FAILED=7 / UNKNOWN=0，通过率 **95.7%**
- **时长分布**：avg 2285ms / median 314ms / p90 6276ms / max 28637ms（`industrial/x509-parser/cert-parse/x509_subject_extensions`，SUCCESS）
- **宪法 baseline**：`principles.md` v8（P27 修宪后 / P31 法律传导后）
- **时效声明**：本快照锚定上述 run id + 工具版本 + corpus，不构成长期承诺。

## pipeline + 前端边界

Charon 是 AeneasVerif 出品的 Rust → LLBC（Low-Level Borrow Calculus）翻译器。多态模式（poly）保留泛型参数，输出带类型变量的 LLBC。

pipeline：cargo + rustc 驱动产 MIR → charon-driver 接管做 MIR 提取 → MIR 翻译为 LLBC → `--print-llbc` 写 stdout。**纯翻译工具**，pipeline 终点就是 LLBC，不调用任何 SMT / 模型检查器。所以本工具"前端 = 全过程 = 翻译到 LLBC"。

`tool.toml` 命令：`charon cargo --abort-on-error --print-llbc -- --lib --target aarch64-apple-darwin`。这是 charon 官方自带的 cargo 驱动，**本项目未提供任何 wrapper 脚本**——entry → cargo → charon-driver 全是工具自身链路，框架仅看 exit code。

- `--abort-on-error`：charon 默认遇内部 panic 仍 exit 0（设计 quirk），加此 flag 后 panic → exit ≠ 0
- `-- --lib --target aarch64-apple-darwin`：绕 macOS arm64 rlib 路径假设，让 cargo 只编 lib target（harness 文件被 `--lib` 跳过编译）

## SUCCESS 信号 + 形式严格性

按宪法 §六 双通路 partial 暴露：

- **主信号通路**：charon exit 0（含 `--abort-on-error`）
- **wrapper 补抓通路**：无（项目未维护 wrapper；仅依赖工具自身 `--abort-on-error` + `register_error!` panic 路径）

形式严格性 0 误报 / 0 漏报状态：

- **0 误报（不冤枉能力）**：源码层证明可达——charon exit 0 ⇔ `error_ctx.continue_on_failure = false` 且 `register_error!` 未触发 panic ⇔ 翻译完整无内部错误（`charon-driver/driver.rs:143` + `src/errors.rs:282`）
- **0 漏报（不高估能力）**：源码层证明可达——`--abort-on-error` 强制把所有 unsupported 标记 panic 化；`register_error!` 是 charon 唯一的"我没全干完"出口，已被 panic 路径封死
- **漏报盲点**：基于源码静态审视，无已知盲点。但需注意：本断言锚定 v0.1.184 源码；上游若新增不走 `register_error!` 的 silent skip 路径（如直接 return / log warn 后继续），需重审本声明

## 失败分桶（按 P31 §四.5 归因分类）

7 个 FAILED 全部 `exit_code = 101`（rustc 进程 panic 标志），归 3 桶。

### 桶 1：charon MIR 提取阶段标 unsupported / panic（4 case）

代表 entry：

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

**归因**：工具不支持。charon 在 MIR 提取阶段对 `Coroutine` MIR 节点（async fn desugar 产物）、`InlineAsm` MIR terminator、`Cast` constant kind 显式 panic——这是 charon 已自陈的翻译边界（统一抛出点 `src/errors.rs:282`）。

**处理**：不修。本地性原则下 FAILED 站得住，工具开发者不能驳回。

### 桶 2：charon 不支持 thread-local references（2 case）

代表 entry：

| entry | panic 消息 |
|---|---|
| `lifetime/thread-local/thread_local_read` | `charon does not support thread local references` |
| `creusot-limit/thread-local-ref/read_thread_local` | 同上 |

panic 源位置：`/rustc/library/std/src/sys/thread_local/native/mod.rs:97:25`——charon 在翻译标准库 TLS 实现时遇到 unsupported 节点（`thread_local!` 宏展开成 std TLS API 调用，charon 沿 cargo 构建图把 std TLS 实现也拽进翻译）。

**归因**：工具不支持。错误消息为 charon 自陈的明示 reject。

**处理**：不修。FAILED 站得住。

### 桶 3：charon 内部 stack overflow（1 case）

`trait/cyclic-bound/cyclic_bound_use`：

```
thread 'rustc' (...) has overflowed its stack
fatal runtime error: stack overflow, aborting
... (signal: 6, SIGABRT: process abort signal)
```

cyclic trait bound 让 charon 内部某条 trait/type resolution 路径无限递归。SIGABRT 不在 `--abort-on-error` 覆盖逻辑里，是被 OS 杀进程后 cargo 转报 exit 101。

**归因**：工具不支持 / 工具内部 bug（charon-driver 在该构造上栈递归不终止）。属"工具能力边界"——本地性原则下 FAILED 站得住。

**处理**：不修。

## 漏报盲点（诚实声明）

- **已通过工具自身机制封堵**：所有 `register_error!` 调用点 → panic → exit ≠ 0（无需项目 wrapper 介入）
- **仍存在的盲点**：基于 v0.1.184 源码静态审视无已知盲点；上游版本变更需重审 `register_error!` / `continue_on_failure` 路径完整性

## 与 charon-mono 的 4 处行为差异（v6 实测）

把同 entry 在 charon-poly vs charon-mono 上的 status 对比，5 个 entry 行为不同（其余 156 个一致）：

| entry | poly | mono | 解释 |
|---|---|---|---|
| `lifetime/thread-local/thread_local_read` | FAILED | SUCCESS | poly 进入 std TLS 内部 polymorphic 路径踩 unsupported；mono 单态化后 specialized 实例避开该路径 |
| `creusot-limit/thread-local-ref/read_thread_local` | FAILED | SUCCESS | 同上 |
| `charon-limit/generic-to-dyn-unsize/boxed_display_from_u32` | SUCCESS | FAILED | mono 展开 `Box<dyn Display>` vtable drop preshim 时 panic；poly 不展开 vtable |
| `lifetime/static-bound/static_bound` | SUCCESS | FAILED | mono 展开 `Box<dyn Any>` vtable drop preshim 时 panic；poly 不展开 vtable |
| `creusot-limit/dyn-trait-forbidden/trigger_call_dyn_display` | SUCCESS | FAILED | 同 vtable drop preshim 类（mono 触发；poly 规避）|

两 mode FAILED 各自集合 7 / 8 个，对称差 5 个 entry——poly 与 mono 的翻译能力**不互含**。

## v5.1 → v6 ΔS 解释

v5.1：139/146 = 95.2%。v6：154/161 = 95.7%。ΔS = +15 SUCCESS 来源于 v6 corpus 扩增（+15 entries，主要为 `runnable/*` 等新增类目）；FAILED 集合精确不变（仍是同 7 个 entry，桶分布一致）。通过率本质同。

## 修订建议清单（仅"我们导致"失败）

**无需修订**。所有 7 个 FAILED 均为工具自陈的能力边界（charon MIR 提取域不支持的 Coroutine / InlineAsm / Cast constant / thread-local ref / cyclic-bound 内部递归），无任何"我们 wrapper bug / 我们 corpus 引入的 lint / 环境损坏"类失败——本工具未维护任何项目层 wrapper 脚本，调用链全在 charon 官方驱动内。

| # | 桶 | 涉及 case | 修复方案 | 优先级 |
|---|---|---|---|---|
| — | — | 0 | 无 | — |
