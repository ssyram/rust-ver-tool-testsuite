# charon-mono

Charon（单态化模式）：将 Rust 代码翻译为 LLBC 中间表示，泛型实例全部展开。

## 简介

Charon 是 AeneasVerif 团队出品的 Rust→LLBC（Low-Level Borrow Calculus）翻译器，
为 Aeneas 等后端验证工具提供统一前端。单态化模式（mono）在翻译时将所有泛型实例展开，
输出无类型变量的具体 LLBC，适合需要逐实例推理的后端。
GitHub: https://github.com/AeneasVerif/charon

## 本测试集中的"前端接受"定义

本测试集筛选 Rust **前端特性覆盖广度**——把每个工具测到"工具自带后端验证器/求解器之前"为止，不看下游求解结果。

Charon 是**纯翻译工具**，pipeline 终点就是 LLBC——不调用任何 SMT / 模型检查器。所以本工具"前端 = 全过程 = 翻译到 LLBC"。

- **判定**：exit 0 = SUCCESS（charon 翻译完成且无内部 panic，配 `--abort-on-error` 防 silent fail）
- **产物**：`--print-llbc` 输出到 stdout（runner 仅看 exit code，不解析内容）。如需函数级对应性校验，可换 `--dest-dir` 让 LLBC 落盘
- **覆盖度精确意义**：charon-mono 的 SUCCESS = "Rust 源码可被 charon 单态化展开为 LLBC"。下游 aeneas / soteria 是否能消费此 LLBC 与本工具无关

### SUCCESS 信号（严格反映前端特性支持范围）

为了严格反映前端特性支持范围（不允许 partial），**SUCCESS = charon exit 0**（含 `--abort-on-error` flag）。任何 partial → FAILED。

- **partial 暴露机制**：`--abort-on-error` 让 charon 内部任何 unsupported 项触发 panic + exit 1。`charon-driver/driver.rs:143` 设 `error_ctx.continue_on_failure = false`，`register_error!` 在第一次错误就 panic
- **形式严格性 — 0 误报（不冤枉能力）**：✅ 形式可证。charon exit 0 ⇔ 翻译完整无内部错误
- **形式严格性 — 0 漏报（不高估能力）**：✅ 形式可证。`--abort-on-error` + `register_error!` panic 路径已封死所有 silent skip
- **漏报盲点**：无

## 安装

上游：<https://github.com/AeneasVerif/charon>

本测试基线：v0.1.184（commit `ed22146b`），与 charon-poly 共用同一可执行文件。

按上游文档自行安装；装好后把 `charon` 可执行文件路径填到 `.env` 的 `TS_CHARON_BIN`（charon-poly 与 charon-mono 共享）。本项目不提供安装脚本或步骤教程，避免在工具版本变迁后误导。

## 本框架配置

参见 `tool.toml`。关键参数：

- **command**：`charon cargo --monomorphize --abort-on-error --print-llbc -- --lib --target aarch64-apple-darwin`
  - `--monomorphize`：相比 poly 多出此 flag，令 charon 在翻译时展开所有泛型实例
  - `--abort-on-error`：同 poly——将 charon 内部 panic 暴露为非 0 exit（实测 mono 模式下 `Box<dyn Any>` vtable drop preshim 触发 panic，缺此 flag 会静默 exit 0）
  - `--print-llbc`：输出翻译结果到 stdout
  - `-- --lib --target aarch64-apple-darwin`：绕 macOS arm64 rlib 路径假设
- **timeout**：600 秒（单态化可能因实例爆炸而更耗时，与 poly 相同上限）
- **env 注入**：无
- **entry_mode**：默认 `"bin"`；harness 存在但被 `--lib` 跳过
- **extra_cargo_deps**：无

## 已知限制 / 坑

- 单态化展开可能引发实例数量爆炸，对泛型使用密集的样例耗时显著高于 poly 模式
- `--abort-on-error` 在 mono 模式下尤为关键：实测 `Box<dyn Any>` 的 vtable drop preshim 会触发 charon 内部 panic，若缺此 flag 则静默 exit 0，框架误判 SUCCESS
- macOS arm64 同样必须加 `--lib --target aarch64-apple-darwin`
- 高阶生命周期、部分 unsafe raw pointer 等超出 charon 翻译域，exit 非 0

## 已知限制 / 平台兼容

**当前测试运行环境**：macOS aarch64（Apple Silicon）。

**平台特定配置**：

- `tool.toml` 中 `--target aarch64-apple-darwin`（同 charon-poly，因 macOS arm64 上 charon 对 bin rlib 路径有错误假设，配 `--lib` 一并绕开）
- 用户可通过修改 `tool.toml` 适配其他平台：Linux x86_64 改为 `--target x86_64-unknown-linux-gnu`、macOS x86_64 改为 `--target x86_64-apple-darwin` 等

未在 Linux / Windows / macOS x86_64 上测试。

## 关联 sub-tests

`examples/charon-limit/` 是本工具自声明的限制集——这些 entry 故意触发 charon 的"不支持"特性，期望本工具在这些 entry 上 FAILED。
