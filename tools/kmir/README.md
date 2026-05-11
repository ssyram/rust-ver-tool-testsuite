# KMIR

K Framework 上的 Rust MIR 形式化操作语义执行引擎，通过 Stable MIR JSON 对 Rust 程序做可执行语义解释。

## 简介

KMIR 由 Runtime Verification 出品，在 K Framework 上定义 Rust MIR 的形式化操作语义，并通过 `kmir run` 执行 Rust 程序。工具链由 K Framework（kompile/LLVM backend）+ Python 包（kmir、kframework）+ stable-mir-json（Rust → Stable MIR JSON 的 rustc 驱动）三层组成，安装链复杂，各组件版本须精确锁定。

GitHub: <https://github.com/runtimeverification/mir-semantics>

## 本测试集中的"前端接受"定义

本测试集筛选 Rust **前端特性覆盖广度**——把每个工具测到"工具自带后端验证器/求解器之前"为止。

KMIR 的子命令分两类：
- **`kmir run`**：在 K Framework 的 MIR 操作语义里**解释执行**程序——这是 KMIR 的"前端 = 全过程"（与 MIRI 同构：都是 MIR-level 解释器，只是 KMIR 跑在 K Framework 上）
- **`kmir prove` / `kmir prove-rs`**：reachability logic 证明，调 K 的 Haskell backend → SMT—— 这才是"后端证明"

> **为什么 `kmir run` 不能去掉？** `kmir run` 之前的 SMIR JSON 生成阶段（cargo build w/ stable-mir-json RUSTC）只是 rustc + plugin 生成标准 Stable MIR JSON，与 cargo-check 几乎重合，不体现 KMIR 任何特有能力。KMIR 的特有能力 = K Framework MIR semantics 是否能解释执行该程序。所以前端边界就是 `kmir run` 完成。

- **判定**（精确实测语义，**带 K-stuck 检测**）：
  - **exit 0 + stdout 含 `#EndProgram ~> .K`** = K interpreter 跑到终止状态 → SUCCESS
  - **exit 0 但 stdout 无 `#EndProgram`**（K cell 残留 `#execTerminator(InlineAsm)` / `#mkAggregate(closure)` 等）= K-stuck 假阳性 → tool.toml 包装翻为 exit 2，标 FAILED
  - **exit ≠ 0** = cargo + stable-mir-json 编译失败（最常见）/ Python kmir CLI 异常 → FAILED

  > **历史问题**：旧版 tool.toml 仅看 `kmir run` exit code，但 K interpreter 卡 stuck 时 CLI 仍 exit 0。实证扫到 102 个原始 SUCCESS 中 **52 个是 K-stuck 假阳性**（如 `charon-limit/inline-asm` 在 miri 上 FAILED unsupported asm，旧 oracle 在 kmir 上 SUCCESS——同一构造结论相反）。新 oracle 通过 grep `#EndProgram ~> .K` 终结模式翻转此类假阳性，真实"工具完成 K 语义"率约 32%（46/146）。

- **产物**：`target/debug/linked.smir.json`（前端 SMIR）；`kmir run` stdout 输出 K configuration（`<kmir>...</kmir>`），运行时表示，不持久化
- **FAILED 来源（实测分布）**：当前 FAILED 集合主要来自上游 **cargo + stable-mir-json** 编译失败（`json.JSONDecodeError: Expecting value: line 1 column 1` 链）—— 触发于含 Arc / BTreeMap / HashMap / 线程 / Mutex / 第三方 crate（rsa / sha2 / x509-parser）的样例。这与"K rule 缺失"的旧 README 描述**不同**：实际是上游 build 阶段就退出，K 语义未介入
- **慢**：每次 `kmir run` 做增量 kompile（10–30s），是工具链中最慢的环节——这是 K Framework 解释执行本身的成本，不是越界做了证明

### SUCCESS 信号（严格反映前端特性支持范围）

为了严格反映前端特性支持范围（不允许 partial），**SUCCESS = `kmir run` exit 0 且 stdout 含 `#EndProgram ~> .K`**（K interpreter 完整跑到终止）。任何 K-stuck → FAILED。

- **partial 暴露机制（双轨）**：
  1. cargo + stable-mir-json 编译失败 → exit ≠ 0
  2. K interpreter 卡 stuck（K cell 残留 `#execTerminator(InlineAsm)` / `#mkAggregate(closure)` 等 unsupported 项）→ kmir CLI 仍 exit 0，但 oracle 通过 grep `#EndProgram ~> .K` 终结模式翻为 exit 2
- **形式严格性 — 0 误报（不冤枉能力）**：✅ 形式可证。`#EndProgram ~> .K` 是 K Framework 解释器的稳定终止 signature——K cell 化简到此 ⇔ 解释执行完整完成
- **形式严格性 — 0 漏报（不高估能力）**：✅ 形式可证。K-stuck（K cell 卡在 unsupported terminator）grep 已封死 silent path；cargo + stable-mir-json 编译失败也直接 exit ≠ 0
- **漏报盲点**：无

## 安装

上游：<https://github.com/runtimeverification/mir-semantics>（同时依赖 K Framework 与 stable-mir-json）。

本测试基线：mir-semantics commit `84bea09` + stable-mir-json commit `62a239d7`，配 K Framework v7.1.282。

按上游文档自行安装（K Framework / Python `kmir` / stable-mir-json / `kdist build` 等多组件链路自行处理）。装好后把 `openjdk` 的 `bin/` 路径填到 `.env` 的 `TS_KMIR_JAVA_BIN_DIR`；`kmir` 本体由 PATH 解析。本项目不提供安装脚本或步骤教程，避免在工具版本变迁后误导。

## 本框架配置

参见 `tool.toml`。关键参数：

- **command**：`env PATH=... kmir run --bin __ts_harness`。通过 `env` 前缀注入含 `/opt/homebrew/opt/openjdk/bin` 的 PATH，使 K Framework 在运行时找到 Java；kmir binary 位于 `/opt/homebrew/bin/kmir`。
- **entry_mode = "bin"**（默认）：runner 写入 harness 到 `src/bin/__ts_harness.rs`。`kmir run` 通过 `cargo build`（RUSTC=stable-mir-json）编译 crate 生成 Stable MIR JSON 后进入 K 语义执行，start symbol 默认为 `main`。
- **--bin __ts_harness**：在 84bea09 版本中此参数只产生 warning，不报错；实际 crate root 由 runner 设置的 `current_dir` 决定。
- **timeout_secs = 180**：kmir 每次运行都做增量 kompile（修改 definition.kore + 重链接），简单程序约 10-30 秒，预留较长超时。
- **exit code 对齐**：kmir run 成功 exit 0（SUCCESS）；cargo/stable-mir-json 失败或 K 语义执行异常 exit 非 0（FAILED）。

## 已知限制 / 坑

- **llvm-kompile-clang 补丁随 brew upgrade 消失**：每次升级 kframework 后需重新打补丁（v7.1.286+ 修复后可免）。
- **stable-mir-json 版本须与 mir-semantics 84bea09 严格匹配**：新版 stable-mir-json（如 commit 885ab4a）引入 `DynType`，84bea09 的 K definitions 不识别，会导致解析失败。不能直接用最新版。
- **kmir run 输出 K state 而非 stdout**：成功时输出整个 K configuration（`<kmir>...</kmir>`），不是普通程序输出。testsuite 只看 exit code，不受影响。
- **per-program kompile 开销**：每次 `kmir run` 对 SMIR 做增量 kompile，约 10-30 秒，是工具链中最慢的环节。
- **nightly-2024-11-29 专用**：stable-mir-json 必须用此版本构建，与项目主 stable toolchain 共存（rustup 可同时管理多个 toolchain）。

## 已知限制 / 平台兼容

**当前测试运行环境**：macOS aarch64（Apple Silicon），通过 brew 安装 openjdk + kframework。

**平台特定配置**：

- `.env` 中 `TS_KMIR_JAVA_BIN_DIR` 默认 `/opt/homebrew/opt/openjdk/bin`（macOS arm64 brew 路径前缀；Intel Mac 是 `/usr/local/opt/openjdk/bin`）
- `tool.toml` 通过 `env PATH=${TS_KMIR_JAVA_BIN_DIR}:$PATH` 注入该路径，使 K Framework 在运行时找到 Java；kmir binary 由系统 PATH 解析（基线 `/opt/homebrew/bin/kmir`）
- 用户可通过修改 `.env` 适配其他平台：
  - Linux：`TS_KMIR_JAVA_BIN_DIR=/usr/lib/jvm/<distro-openjdk>/bin`（按发行版）
  - macOS x86_64：`TS_KMIR_JAVA_BIN_DIR=/usr/local/opt/openjdk/bin`（Intel brew prefix）

未在 Linux / Windows / macOS x86_64 上测试。

## 关联 sub-tests

本工具未派生限制集 agent（集成路径复杂，未派限制集），无 `examples/kmir-limit/`。

K 语义覆盖的 Rust 特性范围有限，依赖复杂 std 特性或第三方 crate 的样例预期 FAILED。
