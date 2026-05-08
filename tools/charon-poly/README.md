# charon-poly

Charon（多态模式）：将 Rust 代码翻译为 LLBC 中间表示，保留泛型参数不展开。

## 简介

Charon 是 AeneasVerif 团队出品的 Rust→LLBC（Low-Level Borrow Calculus）翻译器，
为 Aeneas 等后端验证工具提供统一前端。多态模式（poly）保留泛型参数，
输出带类型变量的 LLBC，适合需要参数化推理的后端。
GitHub: https://github.com/AeneasVerif/charon

## 本测试集中的"前端接受"定义

本测试集筛选 Rust **前端特性覆盖广度**——把每个工具测到"工具自带后端验证器/求解器之前"为止，不看下游求解结果。

Charon 是**纯翻译工具**，pipeline 终点就是 LLBC——不调用任何 SMT / 模型检查器。所以本工具"前端 = 全过程 = 翻译到 LLBC"。

- **判定**：exit 0 = SUCCESS（charon 翻译完成且无内部 panic，配 `--abort-on-error` 防 silent fail）
- **产物**：`--print-llbc` 输出到 stdout（runner 仅看 exit code，不解析内容）。如需函数级对应性校验，可换 `--dest-dir` 让 LLBC 落盘
- **覆盖度精确意义**：charon-poly 的 SUCCESS = "Rust 源码可被 charon 保留泛型翻译为 LLBC"。下游 aeneas / soteria 是否能消费此 LLBC 与本工具无关

### SUCCESS 信号(严格反映前端特性支持范围）

为了严格反映前端特性支持范围（不允许 partial），**SUCCESS = charon exit 0**（含 `--abort-on-error` flag）。任何 partial → FAILED。

- **partial 暴露机制**：`--abort-on-error` 让 charon 内部任何 unsupported 项触发 panic + exit 1。`charon-driver/driver.rs:143` 设 `error_ctx.continue_on_failure = false`，`register_error!` 在第一次错误就 panic
- **形式严格性 — 0 误报（不冤枉能力）**：✅ 形式可证。charon exit 0 ⇔ 翻译完整无内部错误
- **形式严格性 — 0 漏报（不高估能力）**：✅ 形式可证。`--abort-on-error` + `register_error!` panic 路径已封死所有 silent skip
- **漏报盲点**：无

## 安装

上游：<https://github.com/AeneasVerif/charon>

本测试基线：v0.1.184（commit `ed22146b`）。

按上游文档自行安装；装好后把 `charon` 可执行文件路径填到 `.env` 的 `TS_CHARON_BIN`。本项目不提供安装脚本或步骤教程，避免在工具版本变迁后误导。

## 本框架配置

参见 `tool.toml`。关键参数：

- **command**：`charon cargo --abort-on-error --print-llbc -- --lib --target aarch64-apple-darwin`
  - `cargo` 子命令：让 charon 通过 cargo 驱动翻译
  - `--abort-on-error`：charon 默认遇内部 panic 仍 exit 0（设计 quirk），加此 flag 将 silent error 暴露为非 0 exit，否则框架会把"无法翻译"误判为 SUCCESS
  - `--print-llbc`：把翻译结果输出到 stdout（框架不解析，仅用 exit code 判断）
  - `-- --lib --target aarch64-apple-darwin`：传给 cargo 的参数；macOS arm64 上 charon 对 bin rlib 路径有错误假设，用 `--lib` 绕开，`--target` 明确指定三元组
- **timeout**：600 秒（泛型不展开，翻译代价相对低）
- **env 注入**：无（tool.toml 无 env 字段）
- **entry_mode**：默认 `"bin"`；harness 写到 `src/bin/__ts_harness.rs`，但实际被 `--lib` 跳过编译——harness 存在但不参与此工具的翻译
- **extra_cargo_deps**：无

## 已知限制 / 坑

- macOS arm64 必须加 `--lib --target aarch64-apple-darwin`，否则 charon 在定位 rlib 时路径假设错误，进程报错退出
- `--abort-on-error` 不可省：实测若缺少此 flag，某些无法翻译的特性（如部分 trait object drop glue）charon 内部 panic 后仍 exit 0，框架会误报 SUCCESS
- charon 翻译的是 MIR 语义子集，某些 unsafe / raw pointer / 高阶生命周期构造超出其翻译域，会 exit 非 0

## 关联 sub-tests

`examples/charon-limit/` 是本工具自声明的限制集——这些 entry 故意触发 charon 的"不支持"特性，期望本工具在这些 entry 上 FAILED。
