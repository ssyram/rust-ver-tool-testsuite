# miri — 特性支持评估报告（v6 baseline）

## 元数据

- **数据源**：`runs/run-1778560393-59119/`（2026-05-12 v6 final，合并 verus rerun + R7 5-tool rerun）
- **工具配置**：`tools/miri/`
- **工具版本**：`miri 0.1.0 (cb40c25f6a 2026-05-04)` @ rustup nightly toolchain
- **本工具实测**：n=161 / SUCCESS=157 / FAILED=4 / UNKNOWN=0，通过率 **97.5%**
- **时长分布**：avg 2614ms / median 887ms / p90 6405ms / max 21536ms（timeout 设 300s，本矩阵未触发）
- **宪法 baseline**：`principles.md` v8（P27 修宪后 / P31 法律传导后）
- **时效声明**：本快照锚定上述 run id + 工具版本 + corpus，不构成长期承诺。

## pipeline + 前端边界

MIRI（Mid-level Intermediate Representation Interpreter）由 Rust 官方维护，作为 nightly toolchain 的 component 分发。pipeline：cargo + nightly rustc 把 harness + entry lib 编译到 MIR；miri 在 MIR 层模拟执行 + 检测 UB（内存越界 / use-after-free / 未初始化读 / 数据竞争 / 不合法 raw pointer）。

MIRI **没有"前端 / 后端"分界**——它就是 MIR 解释器，工作内容 = 解释执行 + UB 检测。所以"前端 = 全过程"。运行入口由本项目 `tool.toml` 显式锁定 `cargo +nightly miri run --bin __ts_harness`——不包装任何 wrapper 脚本，runner 直接调用上游 binary，前端边界 = 上游 binary 的 exit code。

按宪法 §六-2 "不允许 partial"——SUCCESS = 解释执行完整跑完且无任何中断（无 UB / unsupported / panic），任何中断 → FAILED。

## SUCCESS 信号 + 形式严格性

- **主信号通路**：`cargo +nightly miri run --bin __ts_harness` exit 0
- **wrapper 补抓通路**：无（项目不维护 miri wrapper；上游 binary 单一通路即终通路）

形式严格性 0 误报 / 0 漏报状态（按 P27 后宪法严格语义）：

- **0 误报**：exit 0 ⇔ 解释执行完整跑完且未触发任何 UB / unsupported operation / panic（miri 的 exit 语义由上游源码定义）
- **0 漏报**：任何 UB / unsupported / panic → exit ≠ 0（miri 不存在 silent skip 模式；不像 verifier 类工具会"未触达分支 = vacuously pass"）
- **漏报盲点**：实测层面 miri 默认未显式 enable tree borrows / strict provenance / 全部 data race detection——本测试 SUCCESS 仅蕴含"默认 flag 下完整解释执行"；启用 `-Zmiri-strict-provenance` / `-Zmiri-tree-borrows` 后部分 SUCCESS 可能变 FAILED。这是 default-flag 测量姿势的自然边界，不是 oracle 缺陷。

注：UB 检测在某些视角下被理解为"工具有效输出"，但按宪法 §六-2 "不允许 partial / 完整跑完"精神，本测试一律 FAILED（解释执行被 UB 中断 = 没完整跑完）。

## 失败分桶（按 P31 §四.5 归因分类）

4 个 FAILED 全部归"工具能力边界 / corpus 故意触发"两类——无"我们导致"项。

### 桶 1：miri 自陈 unsupported operation（3 case）

| entry | error 摘要 |
|---|---|
| `charon-limit/inline-asm/nop_via_asm` | `unsupported operation: inline assembly is not supported`（指向 `src/lib.rs` 的 `core::arch::asm!("")`）|
| `kani-limit/extern-ffi/trigger_call_libc_abs` | `unsupported operation: can't call foreign function 'abs' on OS 'macos'`（无 libc shim）|
| `miri-limit/networking-unsupported/tcp_connect_attempt` | `unsupported operation: 'socket' not available when isolation is enabled`（默认 isolation 拒绝网络）|

stderr 共同模式：cargo 完成 `Compiling` + `Finished`，进入 `cargo-miri runner` 解释执行后 miri 在 entry 自身 `src/lib.rs` 抛 `error: unsupported operation: ...`，配 backtrace 指向具体调用点，exit 1。三条都是 miri README 明示的"已知不支持"形态（inline-assembly / unshimmed FFI / network 隔离）。

**归因**：工具自陈能力边界。
**处理**：**不修**。本地性原则下 FAILED 站得住；这些 entry 落在 `*-limit/` 目录就是为了故意触发工具边界，corpus 设计即如此。

### 桶 2：miri 真实检出 UB（1 case）

| entry | error 摘要 |
|---|---|
| `kani-limit/uninit-memory/read_uninit_byte` | `Undefined Behavior: reading memory at alloc201[0x0..0x1], but memory is uninitialized` 指向 `unsafe { x.assume_init() }` |

代码层 `MaybeUninit::<u8>::uninit().assume_init()` 在解释执行中读取未初始化栈字节，miri 真实检出 UB 中断执行。

**归因**：工具能力边界——本测试不是 UB 检测有效性测量，是覆盖度筛选；按 §六-2 "完整跑完"精神，UB 中断 = 没完整跑完 = FAILED 与其他中断同列。
**处理**：**不修**。FAILED 站得住；entry 本就在 `kani-limit/uninit-memory/` 故意触发 UB，corpus 设计即如此。

## 漏报盲点（诚实声明）

- **已封堵**：miri 单一通路（exit code）已捕获所有中断形态（UB / unsupported / panic / rustc compile fail）。miri 上游不提供 silent skip 路径。
- **测量姿势内的盲点**：默认 flag 未开启 strict provenance / tree borrows / 全部并发竞争检测——本 SUCCESS 集合在更严 flag 下可能收缩。这是测量边界，非 oracle 漏洞。
- **int-to-ptr cast 边角**：`unsafe-ptr/raw-ptr-const/raw_ptr_const_match` 含 `43 as *const ()`，miri 默认 permissive provenance 模式下输出 `warning: integer-to-pointer cast` 但 exit 0 → SUCCESS（warning 不是 partial / 不是中断）。这是 miri 上游对该模式的默认行为，记录在此供读者参考；不构成"miri 漏报 UB"——miri 在该模式下明确声明不检测此类。

## v5.1 → v6 ΔS 解释

v5.1 (146 entries) miri: 142 SUCCESS / 4 FAILED = 97.3%
v6 (161 entries) miri: 157 SUCCESS / 4 FAILED = 97.5%

ΔS = +15（来自 corpus 从 146 扩到 161，新增 15 entries 全部 SUCCESS）。FAILED 集合**完全不变**——仍是同 4 个 entry（inline-asm / extern-ffi-abs / uninit-memory / networking）。新增 15 entries 中没有再触发任何 miri 边界。

## 修订建议清单（仅"我们导致"失败）

**无需修订，所有 FAILED 均为工具能力边界 / corpus 故意触发。**

| # | 归类 | 涉及 case | 处理 |
|---|---|---|---|
| — | 桶 1（miri 自陈 unsupported）| 3 | 不修。corpus `*-limit/` 设计即触发此 |
| — | 桶 2（miri 检出 UB）| 1 | 不修。corpus `kani-limit/uninit-memory/` 设计即触发 UB |

——本工具不存在"我们 wrapper bug" / "我们 corpus 引入的 lint" / "环境损坏"任一 (b) (a) 类问题。runner 不维护 miri wrapper，pipeline 直通上游 binary，归因链最短。
