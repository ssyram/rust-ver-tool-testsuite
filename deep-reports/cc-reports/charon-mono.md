# charon-mono — 特性支持评估报告（v6 final post-P35 baseline）

## 元数据

- **数据源**：`runs/run-1778560393-59119/`（2026-05-12T04:33:13Z，v6 final post-P35，合并 verus rerun + R7 5-tool rerun）
- **工具配置**：`tools/charon-mono/`（`tool.toml` + `harness.rs.tera`，无项目维护 wrapper）
- **工具版本**：`charon 0.1.184`（commit `ed22146b`），自带 toolchain `nightly-2026-02-07-aarch64-apple-darwin`；与 `charon-poly` 共用同一可执行文件，仅 CLI flag `--monomorphize` 区别
- **本工具实测**：n=161 / SUCCESS=153 / FAILED=8 / UNKNOWN=0，通过率 **95.0%**
- **时长分布**：avg 1842ms / median 314ms / p90 5382ms / max 22578ms
- **host**：Apple M5 / macOS 25.4.0 / aarch64 / 24 GB / 10 cpu / parallelism=10
- **宪法 baseline**：`principles.md` v8（P27 修宪后 / P31 法律传导后 / P35 §六"当前 crate 焦点（宽度切割）"沉淀后）
- **时效声明**：本快照锚定上述 run id + 工具版本 + corpus，不构成长期承诺。

## pipeline + 前端边界

Charon 是 AeneasVerif 出品的 Rust → LLBC（Low-Level Borrow Calculus）翻译器。**单态化模式**（mono）在翻译时把所有泛型实例展开，输出无类型变量的具体 LLBC，适合需要逐实例推理的后端。

pipeline：cargo + rustc 驱动产 MIR → charon-driver 接管做 MIR 提取 → **monomorphize 展开所有泛型实例** → 翻译为 LLBC → `--print-llbc` 写 stdout。**纯翻译工具**，pipeline 终点就是 LLBC，不调用任何 SMT / 模型检查器。所以"前端 = 全过程 = 翻译到 LLBC"。

`tool.toml` 命令：

```
charon cargo --monomorphize --abort-on-error --print-llbc -- --lib --target aarch64-apple-darwin
```

相比 charon-poly 仅多 `--monomorphize` flag。`--abort-on-error` 在 mono 模式尤为关键——实测 `Box<dyn T>` 的 `vtable_drop_preshim` 单态化触发 charon 内部 panic，缺此 flag 会静默 exit 0、framework 误判 SUCCESS。`-- --lib --target aarch64-apple-darwin` 绕 macOS arm64 的 rlib 路径假设。

**项目维护的 wrapper**：无。直接调用上游 `charon` 二进制，不存在我方 shell 脚本。归因边界因此清晰——任何 driver panic 都是工具锅，按 `tool-integration.md` §四.5 判定为 FAILED。

## SUCCESS 信号 + 形式严格性

按宪法 §六 + `tool-integration.md` §三 / §四：

- **主信号通路**：charon exit 0（含 `--abort-on-error`）
- **wrapper 补抓通路**：无（无项目 wrapper）

**0 误报（不冤枉能力）**：✅ 形式可证（单一通路）。
论证：charon `--abort-on-error` 路径下，exit ≠ 0 必经 `register_error!`（`src/errors.rs:282`）或显式 panic（如 `translate_trait_objects.rs:1707`）。源码层穷尽：所有 `register_error!` 调用点对应 unsupported / unimplemented 路径，无任何"假错"分支。exit 0 ⇔ 翻译完整无内部错误。

**0 漏报（不高估能力）**：✅ 形式可证（单一通路 — 见 `tool-integration.md` §四.1）。
论证：`charon-driver/driver.rs:143` 设 `error_ctx.continue_on_failure = false`，配合 `--abort-on-error` 让 charon 内部任意 unsupported 项触发 panic + exit 1。`register_error!` 在第一次错误就 panic，是唯一 unsupported 入口；panic 后非 0 exit 是唯一退出决定，单一通路覆盖所有 silent path。

**关于外部依赖 opaque 的合法性（P35 §六 当前 crate 焦点）**：测量对象是 entry crate（`TS_TARGET_CRATE` 锚指）。charon 把 `std` / `core` / 第三方依赖里的 type / impl / fn 翻为 opaque / placeholder / stub 是 **合法翻译选择**，不算 silent partial——只要 entry crate 自己的 fn / type / trait 在 LLBC 产物里完整出现即可。本节 0 漏报论证不与此原则冲突：charon 对外部依赖的"opaque 化"并不走 `register_error!` 通路（不算 unsupported），属于正常翻译策略；只有当 entry crate 自己的 item 被 silently 跳过才需触发 partial 判定。实测 153 个 SUCCESS 中，所有 entry fn 在 charon 视角下都得到了具体翻译，未观察到 entry-level silent skip。

**漏报盲点**：无已知。本次实测 8 个 FAILED 全部 `exit_code=101`，全部 stderr 含明确 charon 自陈 unsupported / panic 消息，无 silent skip 候选。

## 失败分桶（按 P31 §四.5 归因分类）

所有 8 个 FAILED 均 `exit_code=101`。逐条读 `runs/run-1778560393-59119/raw/charon-mono/*.stderr` 后归类。

### 桶 1：charon MIR 提取阶段显式自陈 unsupported（4 case）

| entry | 自陈消息 |
|---|---|
| `charon-limit/async-fn/async_forty_two` | `Coroutine types are not supported yet`（`src/errors.rs:282`，针对 entry 自身 `async fn`）|
| `kani-limit/async-await/run_async_add` | `Coroutine types are not supported yet`（同上）|
| `charon-limit/inline-asm/nop_via_asm` | `Inline assembly is not supported`（`src/errors.rs:282`，针对 entry 体内 `InlineAsm` MIR terminator）|
| `unsafe-ptr/raw-ptr-const/raw_ptr_const_match` | `Unsupported constant: ``ConstantExprKind::Cast {..}``` |

stderr 共性片段：

```
thread 'rustc' (...) panicked at src/errors.rs:282:13:
<具体自陈消息>
warning: Thread panicked when extracting item `<entry crate>::<entry fn>`.
ERROR Compilation panicked
```

**归因**：工具不支持。charon 在 MIR 提取阶段对 `Coroutine`、`InlineAsm` terminator、`ConstantExprKind::Cast` 显式 `register_error!` 自陈未实现，且 panic 位置都在 entry crate 自己的 item 上（非外部依赖）。`--monomorphize` 不影响这些前端边界——与 charon-poly 表现一致（共享 FAILED）。

**处理**：不修。本地性原则下 FAILED 站得住，工具开发者不能驳回（`charon-limit/*` 三例正是工具自声明的限制集）。

### 桶 2：单态化路径独有的 vtable drop preshim panic（3 case）

| entry | 触发场景 |
|---|---|
| `charon-limit/generic-to-dyn-unsize/boxed_display_from_u32` | `Could not determine method index for drop in vtable` 触发于 `core::fmt::Display::{vtable_drop_preshim}`；entry 把 `T: Display + 'static` unsize 到 `Box<dyn Display>` |
| `lifetime/static-bound/static_bound` | 同消息，触发于 `core::any::Any::{vtable_drop_preshim}`；entry 走 `Box::new(x)` 后 `b1.is::<i32>()` 触及 `Box<dyn Any>` vtable |
| `creusot-limit/dyn-trait-forbidden/trigger_call_dyn_display` | 同消息，触发于 `core::fmt::Display::{vtable_drop_preshim}`；entry 显式构造 `Box<dyn Display>` 调用 |

panic 源指向 `src/bin/charon-driver/translate/translate_trait_objects.rs:1707:13`，stderr 自陈"the error occurred when translating `core::<...>::{vtable_drop_preshim}`, which is (transitively) used at the following location(s)"，并指回 entry crate 的 `src/lib.rs` 行号。

**归因辨析（P35 §六）**：panic 位置虽然落在 `core::fmt::Display` / `core::any::Any` 这些外部依赖 item 上，但触发条件是 entry crate 自身代码 transitively 用到了 dyn trait drop 路径——charon 此时**没有**把这些 vtable 翻为 opaque 选择放过（那样合法），而是显式 panic 中止整次翻译。换言之，工具没有按"外部依赖 opaque"策略处理，而是按"必须展开但展开不了"中止，导致 entry crate 自己的产物也没生成。属工具自身能力边界，与 P35 §六 不矛盾。

**处理**：不修。这是 charon mono 路径的实际能力边界，FAILED 站得住。`tools/charon-mono/README.md` 已记录此为 mono 模式已知触发点。

### 桶 3：charon 内部 stack overflow（1 case）

`trait/cyclic-bound/cyclic_bound_use`，stderr：

```
thread 'rustc' (...) has overflowed its stack
fatal runtime error: stack overflow, aborting
... (signal: 6, SIGABRT)
```

**归因**：工具不支持。cyclic trait bound 让 charon 内部某条 trait / type resolution 路径无限递归——单态化与否都炸（charon-poly 同样 FAILED）。

**处理**：不修。属 charon 自身能力边界，工具开发者可改进 resolution 路径但非我方责任。

## 漏报盲点（诚实声明）

- **已通过单一通路封堵**：`--abort-on-error` + `continue_on_failure=false` 让所有内部 unsupported 升 panic + exit ≠ 0
- **P35 §六 边界声明**：测量限于 entry crate；外部依赖（std / core / 第三方）被 opaque / stub 视为合法翻译，不触发 partial 判定。本次实测中此类合法 opaque 不构成漏报候选
- **仍存在的盲点**：无已知。本次 161 entries 实测中未观察到 charon exit 0 但 entry crate 自身 item 缺失 / 内部 skip 的案例
- **理论上的潜在风险**：若 charon 上游未来引入新的 "soft warning" 通道（不走 `register_error!`），可能漏报；当前版本无此通道

## v5.1 → v6 ΔS 解释

v5.1 数据：138/146 SUCCESS（旧 cc-report）。v6 数据：153/161 SUCCESS。

corpus 从 146 → 161（+15 entries，主要新增 `runnable/*` 15 entries），分母变化导致直接比率不可比；按"FAILED 数"看 v5.1 / v6 均为 8 个 FAILED，FAILED 集合稳定：

- 共享 5 项：`async_forty_two` / `nop_via_asm` / `raw_ptr_const_match` / `run_async_add` / `cyclic_bound_use`
- 共享 3 项 mono 独有：`boxed_display_from_u32` / `static_bound` / `trigger_call_dyn_display`

charon 版本无变化（v0.1.184 同），FAILED 集合自洽。

## 与 charon-poly 的行为差异（v6）

| entry | charon-poly | charon-mono | 解释 |
|---|---|---|---|
| `lifetime/thread-local/thread_local_read` | FAILED | SUCCESS | poly 进入 std TLS 内部 polymorphic 路径踩 unsupported；mono 单态化后 specialized 实例避开 |
| `creusot-limit/thread-local-ref/read_thread_local` | FAILED | SUCCESS | 同上 |
| `charon-limit/generic-to-dyn-unsize/boxed_display_from_u32` | SUCCESS | FAILED | mono 展开 `Box<dyn Display>` vtable drop preshim 时 panic |
| `lifetime/static-bound/static_bound` | SUCCESS | FAILED | mono 展开 `Box<dyn Any>` vtable drop preshim 时 panic |
| `creusot-limit/dyn-trait-forbidden/trigger_call_dyn_display` | SUCCESS | FAILED | mono 展开 `Box<dyn Display>` vtable drop preshim 时 panic |

对称差合计 5 项，两 mode 在 FAILED 集合上不互含。SUCCESS 总数 mono 153 vs poly 154，差 1。

## 修订建议清单（仅"我们导致"失败）

**无需修订**。所有 8 个 FAILED 均归属工具能力边界（charon 自陈 unsupported / mono 路径 vtable preshim 未实现 / 内部 stack overflow），无任何"我方 wrapper / corpus / 环境"类问题：

- 本工具无项目维护 wrapper（直接调上游 binary）
- corpus 未触发 vendored crate lint（charon 在 entry 翻译失败前就 panic，未走到 deps 翻译阶段）
- 环境完好（无 binary 丢失 / JVM crash / `/tmp` 清理）
- 按 P35 §六 重审：所有 FAILED 都是 entry crate 自身或 entry transitively 触发的 charon 内部 panic，未观察到"应判 SUCCESS 却被冤枉成 FAILED"的外部依赖 opaque 误判

按 `tool-integration.md` §四.5 判据，所有 FAILED 站得住，工具开发者不能驳回。
