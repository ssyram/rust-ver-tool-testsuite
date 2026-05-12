# creusot

Creusot：基于 Why3 的 Rust 演绎验证工具，通过 cargo-creusot 驱动整个 cargo 项目验证。

## 简介

Creusot 是 creusot-rs 团队出品的 Rust 演绎验证工具，以 Why3 为后端（支持 Alt-Ergo、CVC5 等 SMT solver）。
它通过 `cargo-creusot` 替换 `RUSTC` 环境变量，让 cargo 编译流程的每个 crate target 都经过 creusot-rustc 处理，
从而提取 WhyML 证明义务并交给 Why3 验证。
GitHub: https://github.com/creusot-rs/creusot

## 本测试集中的"前端接受"定义

本测试集筛选 Rust **前端特性覆盖广度**——把每个工具测到"工具自带后端验证器/求解器之前"为止，不看下游求解结果。

对 Creusot，这条边界落在 **MIR → coma 翻译** 与 **why3 SMT 求解** 之间：

- **前端**（本工具检测的范围）：cargo-creusot 调度 creusot-rustc 把每个 fn 的 MIR 翻译为 coma 文件，写到 `verif/<crate>_rlib/<module>/<fn>.coma`
- **后端**（本工具不检测）：`cargo creusot prove` 子命令调用 why3find → why3 → Alt-Ergo / CVC5 / Z3 求解证明义务

`cargo-creusot` **默认无 subcommand 时即只翻译，不进入 prove**——所以现行 tool.toml 已经天然停在前端边界。

- **判定**：exit 0 = SUCCESS（翻译完成）；exit ≠ 0 = FAILED（cargo 链接失败 / creusot-rustc 翻译失败 / Why3 module 序列化失败）
- **可补强检测**：检查 `verif/*_rlib/` 存在 + 至少一个非 0 字节 `.coma` 文件 + 入口 fn 的 `let rec` 在 coma 里能 grep 到（runner 当前仅看 exit code，已足够）
- **真实失败常见来源**：creusot-std 不模型化某些 std 类型（如 `std::str::Split` / 部分 iterator）导致 rustc-creusot 在 type check 阶段拒收——这是 creusot 自身的 std-coverage 边界

### SUCCESS 信号（严格反映前端特性支持范围）

为了严格反映前端特性支持范围（不允许 partial），**SUCCESS = `cargo-creusot` exit 0**（默认无 subcommand 即只翻译到 `.coma`，不调用 why3）。任何 partial → FAILED。

- **partial 暴露机制**：creusot 用 `crash_and_error / span_err / span_fatal / dcx().span_err` 把任何 unsupported 升级为 rustc error → exit 101
- **形式严格性 — 0 误报（不冤枉能力）**：✅ 实测 + 源码层论证。cargo-creusot exit 0 ⇔ 翻译完整无 rustc error
- **形式严格性 — 0 漏报（不高估能力）**：✅ 实测 + 源码层论证。creusot 用 `crash_and_error / span_err / span_fatal` 把所有 unsupported 升级为 rustc error，无 silent path
- **漏报盲点**：无

## 安装

上游：<https://github.com/creusot-rs/creusot>

本测试基线：`cargo-creusot` 0.11.0（搭配 OCaml 5.3 + Why3 + Alt-Ergo / CVC5 等 SMT solver）。

按上游文档自行安装（OCaml / opam / Why3 / SMT solver / `cargo-creusot` 自行处理）。装好后把 `cargo-creusot` 可执行文件路径填到 `.env` 的 `TS_CARGO_CREUSOT`。本项目不提供安装脚本或步骤教程，避免在工具版本变迁后误导。

## 本框架配置

参见 `tool.toml`。关键参数：

- **command**：`/Users/<user>/.cargo/bin/cargo-creusot`（直接调用 binary，无额外 flag；cargo-creusot 自行驱动验证流程）
- **timeout**：900 秒（Why3 + SMT solver 验证耗时，且会启动孙子进程 why3 / alt-ergo / cvc；runner 用 `kill(-pgid)` 杀整个进程组）
- **env 注入**：无（cargo-creusot 自身定位 creusot-rustc）
- **entry_mode**：`"lib"`——cargo-creusot 对整个 cargo project 每个 crate target 跑 creusot-rustc，要求顶级 lib 自身含 `use creusot_std::prelude::*;`；runner 将原 `src/lib.rs` 改名为 `src/__ts_inner.rs`，harness（含 `extern crate creusot_std; use creusot_std::prelude::*;`）写为新 `src/lib.rs` 顶级 lib
- **extra_cargo_deps**：`['creusot-std = "0.11.0"']`——cargo-creusot 入口处硬检查 manifest 中必须列出 `creusot-std`；runner 在隔离副本的 `Cargo.toml` 的 `[dependencies]` 表中注入此行

## 已知限制 / 坑

- cargo-creusot 在 cargo 解析阶段就检查 `creusot-std` dep，缺失则直接拒、不进入编译，因此 `extra_cargo_deps` 字段是硬性需求
- creusot-rustc 对每个 crate target 的顶级 module 检查 `creusot_std` import，仅在 harness（bin target）中 `use` 不够——这是引入 `entry_mode = "lib"` 的根因
- Why3 验证阶段会启动多个孙子进程（why3、alt-ergo、cvc 等），runner 必须用 `kill(-pgid, SIGKILL)` 杀整个 process group，否则孙子进程持有 fd 导致 runner reader thread 阻塞
- 演绎验证对规格（spec）的依赖度高，无 spec 的普通 Rust 样例验证覆盖面有限
- 部分 unsafe / raw pointer / 复杂生命周期超出 creusot-rustc 翻译域

## 关联 sub-tests

`examples/creusot-limit/` 是本工具自声明的限制集——这些 entry 故意触发 Creusot 的"不支持"特性，期望本工具在这些 entry 上 FAILED。
