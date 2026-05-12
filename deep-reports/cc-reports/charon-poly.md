# charon-poly — 特性支持评估报告（v6 baseline）

## 元数据

- **数据源**：`runs/run-1778560393-59119/`（2026-05-12 v6 final，合并 verus rerun + R7 5-tool rerun）
- **工具配置**：`tools/charon-poly/`
- **工具版本**：`charon 0.1.184`（charon 自带 toolchain `nightly-2026-02-07-aarch64-apple-darwin`，与 `charon-mono` 共用同一可执行文件，仅 CLI flag 区别——poly 不传 `--monomorphize`）
- **本工具实测**：n=161 / SUCCESS=154 / FAILED=7 / UNKNOWN=0，通过率 **95.7%**
- **时长分布**：avg 2285ms / median 314ms / p90 6276ms / max 28637ms（`industrial/x509-parser/cert-parse/x509_subject_extensions`，SUCCESS）
- **host**：Apple M5 / macOS 25.4.0 / aarch64 / 24 GB / 10 cpu
- **宪法 baseline**：`principles.md` v8（P27 修宪后 / P31 法律传导后 / P33-P35 §六 当前 crate 焦点 + Oracle 责任二分）
- **时效声明**：本快照锚定上述 run id + 工具版本 + corpus，不构成长期承诺。

## pipeline + 前端边界

Charon 是 AeneasVerif 出品的 Rust → LLBC（Low-Level Borrow Calculus）翻译器。**多态模式**（poly）保留泛型参数，输出带类型变量的 LLBC，适合需要参数化推理的后端（如 Aeneas / Soteria）。

pipeline：cargo + rustc 驱动产 MIR → charon-driver 接管做 MIR 提取 → MIR 翻译为 LLBC（**保留泛型参数，不单态化**）→ `--print-llbc` 写 stdout。**纯翻译工具**，pipeline 终点就是 LLBC，不调用任何 SMT / 模型检查器。所以本工具"前端 = 全过程 = 翻译到 LLBC"，不存在求解层切割。

`tool.toml` 命令：

```
charon cargo --abort-on-error --print-llbc -- --lib --target aarch64-apple-darwin
```

- `--abort-on-error`：charon 默认遇内部 panic 仍 exit 0（设计 quirk），加此 flag 后 `register_error!` 触发 panic 路径让 exit ≠ 0
- `-- --lib --target aarch64-apple-darwin`：绕 macOS arm64 rlib 路径假设，让 cargo 只编 lib target（harness 文件被 `--lib` 跳过编译，不参与此工具翻译）

**项目维护的 wrapper**：无。直接调用上游 `charon` 二进制，不存在我方 shell 脚本介入。归因边界因此清晰——按 [`tool-integration.md`](../../docs/design/tool-integration.md) §四.5，任何 charon-driver 抛错都是工具锅，FAILED 站得住。

## SUCCESS 信号 + 形式严格性

按宪法 §六 + tool-integration.md §三 / §四：

- **主信号通路**：charon exit 0（含 `--abort-on-error`）
- **wrapper 补抓通路**：无（项目未维护 wrapper；完全依赖工具自身 `--abort-on-error` + `register_error!` panic 路径）

**0 误报（不冤枉能力）**：源码层穷尽可证（锚定 v0.1.184）。
论证：charon `--abort-on-error` 路径下，exit ≠ 0 必经 `register_error!` 调用点或显式 panic（`src/errors.rs:282` 统一抛出点）。`register_error!` 所有调用点对应 unsupported / unimplemented 路径，无"假错"分支。exit 0 ⇔ 翻译完整无内部错误（`charon-driver/driver.rs:143` 设 `error_ctx.continue_on_failure = false`）。

**0 漏报（不高估能力）**：源码层穷尽可证（锚定 v0.1.184，单一通路 — 见 tool-integration.md §四.1）。
论证：`register_error!` 是 charon 唯一"unsupported / 我没全干完"入口，配合 `--abort-on-error` 让第一次 `register_error!` 即 panic + exit 1，是唯一退出决定，单一通路覆盖所有 silent path。

**漏报盲点**（按 §四.4 诚实声明）：

- **当前实测无已知盲点**：本次 161 entries 实测中所有 FAILED 全部 `exit_code=101` + stderr 含明确 charon 自陈消息；未观察到 SUCCESS 但产物缺失 / silent skip 的案例
- **潜在风险（版本绑定）**：本断言锚定 v0.1.184 源码静态审视。上游若新增不走 `register_error!` 的 silent skip 路径（如直接 return / log warn 后继续 / 新 "soft warning" 通道），需重审本声明
- **当前 crate 焦点的副作用**（§六 / P33 / P35）：charon 沿 cargo 构建图把外部依赖（std / core / 第三方）也拽进翻译。`register_error!` 在 std / 第三方代码上触发也算 FAILED——本测试集对此**接受**：charon 多态模式下 std 翻译失败客观影响了 entry crate 翻译的可完成性（poly 不展开实例，std API 调用以泛型形式留在 entry LLBC 中需 std 段成功翻译才完整），不归类为"外部依赖路径下的 opaque / skip / stub"豁免

## 失败分桶（按 P31 §四.5 归因分类）

7 个 FAILED 全部 `exit_code=101`（rustc 进程 panic 标志），归 3 桶。

### 桶 1：charon MIR 提取阶段显式自陈 unsupported / panic（4 case）

| entry | 自陈消息（`src/errors.rs:282` 统一抛出点）|
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

**归因**：工具不支持。charon 在 MIR 提取阶段对 `Coroutine` MIR 节点（async fn desugar 产物）、`InlineAsm` MIR terminator、`Cast` constant kind 显式 panic——charon 已自陈的翻译边界。`charon-limit/*` 二例正是工具自声明的限制集。

**处理**：不修。本地性原则下 FAILED 站得住，工具开发者不能驳回。

### 桶 2：charon 显式拒绝 thread-local references（2 case）

| entry | 自陈消息 |
|---|---|
| `lifetime/thread-local/thread_local_read` | `charon does not support thread local references` |
| `creusot-limit/thread-local-ref/read_thread_local` | 同上 |

entry 触发：entry crate 内 `thread_local!` 宏展开为 std TLS API 调用；poly 模式不单态化 TLS 内部实现，charon 沿调用图翻译到 std TLS 内部某节点踩 unsupported 标记并 panic。panic stack 末端位于 `/rustc/library/std/src/sys/thread_local/native/mod.rs:97:25`，但 charon 自陈消息明确——是 charon 的能力边界，不是 std 内部 ICE。

**归因**：工具不支持。charon 自陈明示 reject。

**§六 当前 crate 焦点辨析**：尽管 panic 调用栈末端在 std 路径，但 charon 自陈消息（`charon does not support thread local references`）证明这是 charon 的明示能力边界，而非"外部依赖 opaque / skip / stub"。entry crate 的 `thread_local!` 调用是 entry 自身代码——此处 FAILED 归类站得住，不享有 §六 外部依赖豁免。（对照：charon-mono 在这两例 SUCCESS，因为单态化展开 specialized 实例避开了泛型 TLS 路径——poly / mono 翻译能力不互含。）

**处理**：不修。FAILED 站得住。

### 桶 3：charon 内部 stack overflow（1 case）

`trait/cyclic-bound/cyclic_bound_use`：

```
thread 'rustc' (...) has overflowed its stack
fatal runtime error: stack overflow, aborting
... (signal: 6, SIGABRT: process abort signal)
```

cyclic trait bound 让 charon 内部某条 trait / type resolution 路径无限递归。SIGABRT 不在 `--abort-on-error` 覆盖逻辑里，是被 OS 杀进程后 cargo 转报 exit 101。

**归因**：工具不支持 / 工具内部 bug（charon-driver 在该构造上栈递归不终止）。属"工具能力边界"——本地性原则下 FAILED 站得住。charon-mono 同 entry 同样 FAILED，证明与单态化无关。

**处理**：不修。

## 漏报盲点（诚实声明）

- **已通过工具自身机制封堵**：所有 `register_error!` 调用点 → panic → exit ≠ 0（无需项目 wrapper 介入；单一通路）
- **仍存在的已知盲点**：无（基于 v0.1.184 源码静态审视 + 本次 161 entries 实测；未观察到 SUCCESS 候选有 silent skip 嫌疑）
- **潜在风险（版本绑定）**：上游版本变更需重审 `register_error!` / `continue_on_failure` 路径完整性；新增 soft warning / 非 panic 路径会突破单一通路保证

## 与 charon-mono 的行为差异（v6 实测）

把同 entry 在 charon-poly vs charon-mono 上的 status 对比，5 个 entry 行为不同（其余 156 个一致）。两 mode FAILED 集合各自 7 / 8 个，对称差 5 个 entry——poly 与 mono 的翻译能力**不互含**。

| entry | poly | mono | 解释 |
|---|---|---|---|
| `lifetime/thread-local/thread_local_read` | FAILED | SUCCESS | poly 进入 std TLS 内部 polymorphic 路径踩 unsupported；mono 单态化后 specialized 实例避开该路径 |
| `creusot-limit/thread-local-ref/read_thread_local` | FAILED | SUCCESS | 同上 |
| `charon-limit/generic-to-dyn-unsize/boxed_display_from_u32` | SUCCESS | FAILED | mono 展开 `Box<dyn Display>` vtable drop preshim 时 panic（`translate_trait_objects.rs:1707`）；poly 不展开 vtable |
| `lifetime/static-bound/static_bound` | SUCCESS | FAILED | mono 展开 `Box<dyn Any>` vtable drop preshim 时 panic；poly 不展开 vtable |
| `creusot-limit/dyn-trait-forbidden/trigger_call_dyn_display` | SUCCESS | FAILED | mono 展开 `Box<dyn Display>` vtable drop preshim 时 panic；poly 规避 |

## v5.1 → v6 ΔS 解释

v5.1：139/146 = 95.2%。v6：154/161 = 95.7%。

corpus 从 146 → 161（+15 entries，主要新增 `runnable/*` 15 entries），分母变化导致直接比率不可比；按"FAILED 数"看 v5.1 / v6 均为 7 个 FAILED，FAILED 集合**精确不变**（同 7 个 entry，桶分布一致）。charon 版本无变化（v0.1.184 同），通过率本质同；v6 增量 15 entries 全部 SUCCESS（runnable/* 等无触发 charon 边界）。

## 修订建议清单（仅"我们导致"失败）

**无需修订**。

所有 7 个 FAILED 均归属工具自陈的能力边界：

- charon MIR 提取域不支持的 `Coroutine` / `InlineAsm` / `Cast` constant kind（4 case，桶 1）
- charon 显式拒绝 thread-local references（2 case，桶 2）
- charon-driver 在 cyclic-bound 上栈递归不终止（1 case，桶 3）

无任何"我们 wrapper bug / 我们 corpus 引入的 lint / 环境损坏"类失败——本工具未维护任何项目层 wrapper 脚本，调用链全在 charon 官方驱动内；corpus 未触发 vendored crate lint；环境完好（binary 路径正确、无文件锁问题）。

按 tool-integration.md §四.5 判据，所有 FAILED 归属"工具锅"侧，FAILED 站得住，工具开发者不能驳回。

| # | 桶 | 涉及 case | 修复方案 | 优先级 |
|---|---|---|---|---|
| — | — | 0 | 无 | — |
