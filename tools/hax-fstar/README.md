# Hax → F*

Hax 是 Rust → 多个证明助手翻译器；本工具是 F\* backend。

## 简介

Hax (`hacspec/hax`) 是 Rust → F\*/Coq/Lean/EasyCrypt/SSProve/ProVerif 多 backend 翻译器，OCaml 写的引擎 + Rust 写的前端。本配置使用 `cargo hax into fstar` 子命令将 Rust crate 翻译为 F\* 文件。

主仓库:https://github.com/hacspec/hax

## 本测试集中的"前端接受"定义

本测试集筛选 Rust **前端特性覆盖广度**——把每个工具测到"工具自带后端验证器/求解器之前"为止。

Hax 是**纯翻译工具**，pipeline 终点是 `.fst` 文件落盘——下游 F* type-check / 证明不在测试范围。

- **前端**（本工具检测的范围）：rustc → frontend exporter → engine → phase pipeline → F* printer 写出 `<work_dir>/proofs/fstar/extraction/<Crate>.fst`
- **后端**（本工具不检测）：用户自己拿 `.fst` 给 F* / Karamel 做下游验证

F* backend 在 hax 里有 6 个 reject phase（介于 lean 4 与 coq 9 之间）：`RawOrMutPointer` / `Reject_impl_type_method` / `Arbitrary_lhs` / `Question_mark` / `As_pattern` / `Trait_item_default`。printer 自身也有多处 `Error.unimplemented` 抛异常路径，叠加后通过率 ~79%（lean 89% / coq 67% 中间）。

判定与 hax-lean 同源但 hax-fstar 后端较成熟，**几乎不走 silent path**——unsupported 都让 cargo hax exit 1。

### SUCCESS 信号（严格反映前端特性支持范围）

为了严格反映前端特性支持范围（不允许 partial），**SUCCESS = cargo hax exit 0 **且** 产物 grep 不命中 `Rust_primitives.Hax.failure`**。任何 partial → FAILED。

**partial 暴露机制**：
1. cargo hax exit 1 = engine emit Diagnostic（实测 hax-fstar 在 unsupported entry 上稳定 exit 1，是主要信号）
2. 产物 grep `Rust_primitives.Hax.failure` —— 防御性检测，抓 `hax_failure_expr` 渲染为 fstar literal `Rust_primitives.Hax.failure "..."` 的潜在 silent path

**形式严格性 — 0 误报（不冤枉能力）**：⚠️ 实测验证 0 误报，但**不可形式证明**。grep 模式 `Rust_primitives.Hax.failure` 是 hax 内部完整 fstar literal 路径，用户合法代码极难写出该字面字符串——但不能形式排除。实测：用户 doc comment + 局部变量 `let failure: i32 = 5;` 都不触发 FAILED

**形式严格性 — 0 漏报（不高估能力）**：⚠️ 实测验证 0 漏报，但**不可形式证明**。hax-fstar 后端较成熟，几乎不走 silent path——unsupported 都让 cargo hax exit 1（实测 hax-fstar 在 unsupported entry 上稳定 exit 1）。grep `Rust_primitives.Hax.failure` 是防御性检测

**漏报盲点**：
- hax engine 完全 skip item（实测 0 现象）—— 同 hax-lean
- 上游引入新 silent path 的可能（实测无）

## 安装

上游：<https://github.com/hacspec/hax>

本测试基线：commit `30949eb87058895c24f963df90dd30ef11b0dc1a`（搭配 nightly toolchain `nightly-2025-11-08`）。三个 hax backend（coq / fstar / lean）共享同一 `hax-engine` OCaml binary。

按上游文档自行安装；装好后把 `hax-engine` 可执行文件路径填到 `.env` 的 `TS_HAX_ENGINE_BIN`。F\* backend 自身不需要额外组件——`cargo hax into fstar` 只生成 `.fst` 文件，不调用 `fstar.exe`。本项目不提供安装脚本或步骤教程，避免在工具版本变迁后误导。

## 本框架配置

参见 `tool.toml`。关键参数：

- `command` 含 `cargo hax -C --lib ; into fstar`
  - 与 hax-lean / hax-coq 一致，显式固定 `+nightly-2025-11-08`，避免上游 `rust-toolchain.toml` 变动导致跨 backend / 跨 run 不可比
  - `-C --lib ;` 限制只翻译 lib target，跳过 runner 注入的 `src/bin/__ts_harness.rs`
- `env` 前缀注入 `HAX_ENGINE_BINARY=~/.opam/default/bin/hax-engine`
- `timeout_secs = 300`（F\* 提取比 Lean/Coq 快，timeout 较短）
- `entry_mode` 使用默认 `bin`
- 提取输出写入 `<work_dir>/proofs/fstar/extraction/<CrateName>.fst`

与 hax-lean 路径同构，差异只在 `command` 最后一个 token（`fstar` vs `lean`）。

## 已知限制 / 坑

- 不支持的 Rust 构造（如 `dyn Trait`、`&mut` 返回等）：hax 以 exit 1 退出，runner 记录 FAILED
- `HAX_ENGINE_BINARY` 必须为绝对路径，机器迁移时需更新 `tool.toml`
- F\* backend 是 hax 最成熟的后端，整体支持度高于 Lean 和 Coq backend

## 关联 sub-tests

`examples/hax-limit/` 是 Hax（不分 backend）自声明的限制集——这些 entry 故意触发 Hax 已知"不支持"特性（如返回 `&mut`、let-chains、closure mutating outer、labelled-break、unsafe-block 等），期望本 backend 在这些 entry 上 FAILED。
