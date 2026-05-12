# miri — 特性支持评估报告（v6 final post-P35 baseline）

## 元数据

- **数据源**：`runs/run-1778560393-59119/`（2026-05-12 v6 final，合并 verus rerun + R7 5-tool rerun，含 P35 miri-strict-wrapper 启用后的 miri 全量）
- **工具配置**：`tools/miri/`（含 `miri-strict-wrapper.sh`，P35 引入）
- **工具版本**：`miri 0.1.0 (cb40c25f6a 2026-05-04)` @ rustup nightly toolchain
- **本工具实测**：n=161 / SUCCESS=158 / FAILED=3 / UNKNOWN=0，通过率 **98.1%**
- **时长分布**：avg 2610ms / median 899ms / p90 6405ms / max 21536ms（timeout 设 300s，本矩阵未触发）
- **宪法 baseline**：`principles.md` v8（P27 修宪后 / P31 法律传导后 / P35 派生条 architecture §一 "bug detect 归 SUCCESS"已落地）
- **时效声明**：本快照锚定上述 run id + 工具版本 + corpus + wrapper 版本，不构成长期承诺。

## pipeline + 前端边界

MIRI（Mid-level Intermediate Representation Interpreter）由 Rust 官方维护，作为 nightly toolchain 的 component 分发。pipeline：cargo + nightly rustc 把 harness + entry lib 编译到 MIR；miri 在 MIR 层模拟执行 + 检测 UB（内存越界 / use-after-free / 未初始化读 / 数据竞争 / 不合法 raw pointer）。

MIRI 本身**没有"前端 / 后端"分界**——它就是 MIR 解释器，工作内容 = 解释执行 + UB 检测。在本项目 pipeline 中，runner 通过 `tools/miri/miri-strict-wrapper.sh` 调用上游 binary `cargo +nightly miri run --bin __ts_harness`。wrapper 仅做 **exit code 重映射**——不改 invocation、不传额外 flag、不读 stdin 之外的状态。前端边界 = 上游 binary 的 exit code + stderr 文本（wrapper grep 这两者）。

P35 之前（v5.1 / v6 early）：runner 直通上游 binary，任何 exit ≠ 0 一律 FAILED——把"工具无法 ingest"与"工具完成解释 + 检出真实 UB"混为一谈。

P35 之后（本快照）：按 architecture.md §一"bug detect 归 SUCCESS"（§四 B 派生），wrapper 区分两类 exit ≠ 0：

1. **MIRI 自身能力边界**（inline-asm / 未 shim 的 foreign fn / isolation 下的网络）——stderr 含 `unsupported operation` / `can't call foreign function` / `not available when isolation is enabled` → 保留原 exit code → FAILED
2. **MIRI 完成解释 + 在 entry 代码检出 UB**——stderr 含 `Undefined Behavior` 且**不含** unsupported 标记 → 重写为 exit 0 → SUCCESS

precedence：unsupported 优先级高于 UB（单次运行可同时含两 marker，但若 unsupported 命中，说明 miri 没真跑完——FAILED 是正确归类）。

## SUCCESS 信号 + 形式严格性

按宪法 §六 双通路 partial 暴露：

- **主信号通路**：`cargo +nightly miri run --bin __ts_harness` exit 0（解释执行完整完成 + 无 UB / 无 unsupported / 无 panic）
- **wrapper 补抓通路**（P35 新增）：exit ≠ 0 + stderr 含 `Undefined Behavior` + 不含 unsupported 标记 → 翻 SUCCESS（bug-detect = SUCCESS，按 architecture §一）

形式严格性 0 误报 / 0 漏报状态（按 P27 后宪法严格语义）：

- **0 误报候选**：
  - 主通路 exit 0 ⇔ miri 完整解释 + 无任何中断（miri 的 exit 语义由上游源码定义）
  - wrapper 通路要求严格双 grep（必含 `Undefined Behavior` + 必无 `unsupported operation` / 三 marker 之一）——P35 wrapper 实现已审 + 已验证（uninit-memory 用例确认 wrapper 注入了 `[miri-oracle]` 行）
- **0 漏报候选**：任何 UB / unsupported / panic / rustc compile fail → 上游 exit ≠ 0；wrapper 对"非 UB 的 exit ≠ 0"不翻转，对 UB 才翻 SUCCESS——两类都被捕到正确桶
- **漏报盲点**：
  1. 默认 flag 未显式启用 `-Zmiri-strict-provenance` / `-Zmiri-tree-borrows` / 全量 data race detection——本 SUCCESS 仅蕴含"默认 flag 下完整解释执行 / 默认 flag 下检出的 UB"。启用更严 flag 后部分 SUCCESS 可能变 FAILED。这是测量姿势的自然边界，不是 oracle 缺陷。
  2. 若 miri 上游某天改写 stderr 文本（把 `Undefined Behavior` 改名 / 国际化），wrapper grep 会失配——属"上游 stderr 接口变更"风险。当前版本 `miri 0.1.0 (cb40c25f6a 2026-05-04)` 实测稳定。
  3. wrapper 不区分"UB 在 entry 自身代码"vs"UB 在 vendored crate / std 实现"——本测试 corpus 暂无后者实例，但若将来引入，wrapper 仍会翻 SUCCESS。这是 architecture §一 当前定义的内在 scope（"工具检出 UB 即工具有效输出"），不是 wrapper bug。

注：UB 检测在 v5.1 / v6 early 时代曾被理解为"按完整完成精神 = FAILED"。P35 修订后此立场被 architecture §一 显式推翻——UB 检出 = 工具输出最有价值形态 → SUCCESS。

## 失败分桶（按 P31 §四.5 归因分类）

3 个 FAILED 全部归"工具能力边界"——无"我们导致"项。

### 桶 1：miri 自陈 unsupported operation（3 case）

| entry | error 摘要 |
|---|---|
| `charon-limit/inline-asm/nop_via_asm` | `unsupported operation: inline assembly is not supported`（指向 `src/lib.rs` 的 `core::arch::asm!("")`）|
| `kani-limit/extern-ffi/trigger_call_libc_abs` | `unsupported operation: can't call foreign function 'abs' on OS 'macos'`（无 libc shim）|
| `miri-limit/networking-unsupported/tcp_connect_attempt` | `unsupported operation: 'socket' not available when isolation is enabled`（默认 isolation 拒绝网络）|

stderr 共同模式：cargo 完成 `Compiling` + `Finished`，进入 `cargo-miri runner` 解释执行后 miri 在 entry 自身 `src/lib.rs` 抛 `error: unsupported operation: ...`，配 backtrace 指向具体调用点，exit 1。三条都是 miri README 明示的"已知不支持"形态（inline-assembly / unshimmed FFI / network 隔离）。wrapper 在 Priority 1 grep 命中，保持 exit ≠ 0 → FAILED。

**归因**：工具自陈能力边界。
**处理**：**不修**。本地性原则下 FAILED 站得住；这些 entry 落在 `*-limit/` 目录就是为了故意触发工具边界，corpus 设计即如此。

## 漏报盲点（诚实声明）

- **已封堵**：
  - 主通路（exit code）捕获所有非 UB 中断形态（unsupported / panic / rustc compile fail）
  - wrapper 通路（P35 新增）捕获"完成解释 + UB"形态并归 SUCCESS（bug-detect 语义）
- **测量姿势内的盲点**：默认 flag 未开启 strict provenance / tree borrows / 全部并发竞争检测——本 SUCCESS 集合在更严 flag 下可能收缩。这是测量边界，非 oracle 漏洞。
- **int-to-ptr cast 边角**：`unsafe-ptr/raw-ptr-const/raw_ptr_const_match` 含 `43 as *const ()`，miri 默认 permissive provenance 模式下输出 `warning: integer-to-pointer cast` 但 exit 0 → SUCCESS（warning 不是 partial / 不是中断）。这是 miri 上游对该模式的默认行为，记录在此供读者参考；不构成"miri 漏报 UB"——miri 在该模式下明确声明不检测此类。
- **wrapper grep 文本依赖**：见上节"漏报盲点候选 2"——上游若改写 stderr marker 文本会致 wrapper 失配。属上游接口变更风险，当前版本稳定。

## v5.1 → v6 final post-P35 ΔS 解释

- v5.1 (146 entries) miri：142 SUCCESS / 4 FAILED = 97.3%
- v6 early (161 entries / pre-P35) miri：157 SUCCESS / 4 FAILED = 97.5%
- **v6 final post-P35 (161 entries) miri：158 SUCCESS / 3 FAILED = 98.1%**

ΔS（v5.1 → v6 final）= +16：

- +15 来自 corpus 从 146 扩到 161，新增 15 entries 全部 SUCCESS
- +1 来自 P35 wrapper 把 `kani-limit/uninit-memory/read_uninit_byte` 从 FAILED 翻为 SUCCESS（miri 完成解释 + 检出 UB → bug-detect SUCCESS）

FAILED 集合（3 个 unsupported operation 类）与 v5.1 时代的 unsupported 子集完全不变——P35 wrapper 仅影响 UB 检出类的归类，对 unsupported 类无任何作用。

## 修订建议清单（仅"我们导致"失败）

**无需修订，所有 FAILED 均为工具能力边界（miri 自陈 unsupported）。**

| # | 归类 | 涉及 case | 处理 |
|---|---|---|---|
| — | 桶 1（miri 自陈 unsupported）| 3 | 不修。corpus `*-limit/` 设计即触发此 |

P35 后本工具不存在"我们 wrapper bug" / "我们 corpus 引入的 lint" / "环境损坏"任一 (b) (a) 类问题。wrapper 仅做 exit code 重映射，不改 invocation，归因链短且明确。
