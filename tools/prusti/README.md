# prusti

Prusti：基于 Viper 中间语言的 Rust 自动验证工具，默认对每个函数做隐式 panic 检查。

## 简介

Prusti 是 ETH Zürich ViperProject 团队出品的 Rust 验证工具，以 Viper 为验证后端，
通过 prusti-rustc 编译器插件对 Rust 代码进行自动前后条件验证。
默认模式下无需手写 spec，每个函数都隐式经过 panic 可达性检查。
GitHub: https://github.com/viperproject/prusti-dev

## 安装

上游：<https://github.com/viperproject/prusti-dev>

本测试基线：v0.2.2（commit `a0681ee`，2023-08-22），匹配 nightly toolchain `nightly-2023-08-15` 与 JDK 17。

按上游文档自行安装（macOS arm64 上需要 Rosetta + x86_64 toolchain + x86_64 JDK，参考上游 release notes 与项目 wiki）。装好后把 `prusti-rustc` / `cargo-prusti` 路径、对应 x86_64 nightly toolchain 的 sysroot 目录、x86_64 JDK 的 `JAVA_HOME` 分别填到 `.env` 的 `TS_PRUSTI_RUSTC` / `TS_CARGO_PRUSTI` / `TS_PRUSTI_RUST_TOOLCHAIN_DIR` / `TS_PRUSTI_JAVA_HOME`。本项目不提供安装脚本或步骤教程，避免在工具版本变迁后误导。

## 本测试集中的“前端接受”定义

本测试集筛选的是 Rust **前端特性覆盖广度**，而非验证条件的 SAT/UNSAT。
对 Prusti，这条边界落在 **MIR → Viper encoding** 与 **Silicon SMT 求解** 之间：

- **前端**（本工具检测的范围）：rustc 解析、borrow checker、prusti spec collect、
  `Encoder::process_encoding_queue` 把每个 fn 编码成 Viper VIR、最终 lower 成 `.vpr`
  文本文件。
- **后端**（本工具不检测）：Silicon 把 Viper program 喂给 Z3 求解、判 SAT/UNSAT、
  反馈 verification error。

只要 encoder 真正跑过对应函数、产生合法的 Viper program 写出 `.vpr`，
就视为“前端接受”。Silicon 是否能验得动、能否得出真值，与覆盖度结论无关。

## 框架配置（前端-only 模式）

`tool.toml` 通过三个 env var 实现 **encoder 跑 / Silicon 不跑** 的精确切分
（语义对应 commit `a0681ee` 的 `prusti-utils/src/config.rs` 与
`prusti-server/src/process_verification.rs`）：

| flag | 作用 |
|------|------|
| `PRUSTI_NO_VERIFY=false` | 进入 `verify(env, def_spec)` 路径；触发 `Encoder::process_encoding_queue`。`CollectPrustiSpecVisitor` 默认收集所有 fn item，**无需** entry 加 `use prusti_contracts::*` 或 `#[ensures]` |
| `PRUSTI_DUMP_VIPER_PROGRAM=true` | encoder 完成后把 Viper program 写到 `target/verify/log/viper_program/<crate_module>--<fn>-Both.vpr` |
| `PRUSTI_PRINT_HASH=true` | `process_verification_request` 在 dump 之后、`new_viper_verifier()` 之前直接 `return Success`。**Silicon/Z3 永不启动**（无 JVM verifier 实例化、无 SMT 进程） |

其余 env 不变：
- `JAVA_HOME`：x86_64 Adoptium Temurin JDK 17（Prusti 仍需 JVM 加载 Viper jar 与
  `ast_factory` 把 VIR 序列化为 `.vpr`，但不会启动 verifier）
- `PATH` / `CARGO` / `RUSTC` / `RUSTUP_TOOLCHAIN` / `RUST_SYSROOT`：x86_64 toolchain
  绑定（macOS arm64 上必须用 Rosetta 整体跑 `cargo-prusti`）
- `entry_mode = "bin"`：harness 写到 `src/bin/__ts_harness.rs`

### 检测条件

- **前端接受**：exit code `0` 且 `target/verify/log/viper_program/*.vpr` 至少一个文件存在
- **前端拒绝**：exit code `≠ 0`，且 stderr 含下列 marker 之一：
  - `[Prusti: unsupported feature] ...`（graceful 拒绝路径，如 `async fn`、未支持类型）
  - `[Prusti: internal error] ...`（encoder 抛出 internal error，如 raw pointer deref）
  - `thread 'rustc' panicked at prusti-interface/src/environment/mir_storage.rs`
    （部分 unsupported case 走 ICE 路径，如 closure 表达式）

**前端接受路径**由 [`prusti-strict-wrapper.sh`](prusti-strict-wrapper.sh) 实施 exit 0 + `.vpr ≥ 1` 双 check（2026-05-08 起；0 .vpr 即 FAILED，详见 [`docs/fixes/oracle-leak-audit-2026-05-08.md`](../../docs/fixes/oracle-leak-audit-2026-05-08.md) §3.6）。**前端拒绝路径**上述 marker 在 prusti / cargo-prusti 实现上必伴 `exit ≠ 0`（marker 的 emit 点都在 fatal handler 里）——wrapper 不需要单独 grep marker，exit ≠ 0 已等价覆盖。Marker 在 README 列出仅为说明拒绝信号面貌，不是独立 oracle 条件。

### 形式严格性

- **0 误报（不冤枉能力）**：✅ 实测 + 源码层论证。cargo-prusti exit 0 ⇔ encoder 完整跑过且无 unsupported feature 报告；wrapper .vpr 检查为下游辅助
- **0 漏报（不高估能力）**：✅ 实测 + wrapper 双通路。Prusti 任何 unsupported feature → `[Prusti: ...]` marker + exit ≠ 0；任何 internal error / closure ICE → exit ≠ 0；即使未来 toolchain drift 让 encoder fast-path silent skip lower → wrapper 检 .vpr 数量为 0 → FAILED
- **漏报盲点**：无（NEW config + wrapper .vpr 检查双重保险下，encoder silent fast-path 已被 wrapper 闭环；剩余风险仅限于 encoder 内部 silent skip 单个 fn item 但仍写出非空 .vpr 的极端情形——理论窗口，实测 0 现象）

## 与旧 `PRUSTI_NO_VERIFY=true` 配置的对比

| 维度 | 旧（NO_VERIFY=true） | 新（DUMP+PRINT_HASH） |
|------|---------------------|----------------------|
| `verify(env, def_spec)` | 跳过 | 进入 |
| `Encoder::process_encoding_queue` | 不跑 | 跑 |
| `.vpr` 文件 | 不产生 | 写到 `target/verify/log/viper_program/` |
| Silicon / Z3 进程 | 不启动 | 不启动 |
| 简单 fn 单 entry 时间（实测 `pub fn add(...)`） | ≈ 1.1 s | ≈ 2.3 s（多出来的是 encoder + JVM bootstrap，**没有** SMT 求解） |
| “前端边界” 信号 | 全部 exit 0（rustc + macro 展开通过即视为接受，**未测 encoder**） | exit 0 仅当 encoder 真接受；`closures-unsupported` / `async fn` / raw pointer 等均 exit 非 0 |
| 旧配置定性 | 退化为 “rustc + prusti-contracts proc-macro pass”，与 `cargo-check` 重合 | 真实测到 MIR → Viper 这条 Prusti 独有的前端边界 |

预期通过率定性变化：旧配置下 Prusti 与 cargo-check 在通用 Rust feature
样例上几乎无差异，无法暴露 Prusti 的前端能力边界；新配置下 `prusti-limit/`
里的 entry 应稳定 FAILED，其它 entry 中触及 Prusti 不支持特性
（async、复杂闭包、原始指针解引用、生命周期高阶用法等）的也会被识别出来。

## 已知限制

- macOS arm64 上必须 `arch -x86_64` 整体以 Rosetta 跑；5 个 env 缺一会导致 toolchain 不匹配
- rustup 的 arm64 proxy 会无视 `RUSTUP_TOOLCHAIN` 选错 arch，必须 `CARGO=...` 显式绕开
- Prusti 锁定在 `nightly-2023-08-15`，新 nightly ABI 可能不兼容 prusti-driver
- entry 含 closure 表达式时 prusti 触发 ICE 而非 graceful 错误
  （`prusti-interface/src/environment/mir_storage.rs:85` panic）；
  exit 101 + 该 panic stack 是其稳定指纹
- entry 仅触及通用 Rust 特性（无 spec、无 closures、无 unsafe）时，
  encoder 通常接受，与“Prusti 是否能正确验证此函数”无关——**这是预期行为**

## 已知限制 / 平台兼容

**当前测试运行环境**：macOS aarch64（Apple Silicon），通过 Rosetta 跑 x86_64 工具链。

**平台特定配置**：

- `tool.toml` 中 `RUSTUP_TOOLCHAIN=nightly-2023-08-15-x86_64-apple-darwin`（toolchain triple 含平台）
- `prusti-strict-wrapper.sh` 顶层 `arch -x86_64 "$CARGO_PRUSTI"`（macOS-specific Rosetta 调用，强制 x86_64 模式启动子进程）
- `version_command` 同样含 `arch -x86_64` + `nightly-2023-08-15-x86_64-apple-darwin` 串
- `.env` 中 `TS_PRUSTI_JAVA_HOME` 指向 x86_64 JDK（Prusti 仅提供 x86_64 二进制包；上游 Apple Silicon 原生分发缺位）
- 用户可通过修改 `tool.toml` / wrapper 适配其他平台：
  - Linux x86_64：去掉 `arch -x86_64`，改 toolchain 为 `nightly-2023-08-15-x86_64-unknown-linux-gnu`，`TS_PRUSTI_JAVA_HOME` 指向 Linux JDK
  - macOS x86_64：去掉 `arch -x86_64`（本机 x86_64 直跑），其他不变

未在 Linux / Windows / macOS arm64 原生 prusti 上测试。

## 关联 sub-tests

`examples/prusti-limit/` 是本工具自声明的前端限制集——这些 entry 触发 Prusti
encoder 真正拒绝的特性（closures、raw pointer deref、loan-crosses-loop 等），
预期在新配置下 FAILED；旧 `NO_VERIFY=true` 配置下它们大多 PASS（因为 rustc
本身能编过），那是 false positive。
