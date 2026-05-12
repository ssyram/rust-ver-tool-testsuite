# Aeneas → Coq

Aeneas 是 Rust → 多 PA 翻译器，前端用 charon 出 LLBC 中间表示；本工具是 Coq backend。

## 简介

Aeneas (`AeneasVerif/aeneas`) 是 Rust → F\*/Coq/Lean/HOL4 翻译器，OCaml 写。前端用 charon (LLBC)，后端按 backend flag 切换。本配置 `aeneas -backend coq` 产出 `.v` 文件。

主仓库：https://github.com/AeneasVerif/aeneas

## 本测试集中的"前端接受"定义

本测试集筛选 Rust **前端特性覆盖广度**——把每个工具测到"工具自带后端验证器/求解器之前"为止。

Aeneas 是**纯翻译工具**，pipeline 终点是 `.v` 文件落盘——下游 `coqc` type-check 不在测试范围。pipeline 与 aeneas-lean 同构（`charon → LLBC → aeneas -backend coq → .v`），4 个 backend 共享同一份 charon + aeneas，差异在最后的 `Extract.ml` printer 分支。

- **前端**（本工具检测的范围）：写出 `coq-out/<Mod>.v` + `Primitives.v`
- **后端**（本工具不检测）：用户自己拿 `.v` 给 `coqc` + aeneas Coq 支持库做下游验证

判定（精确实测语义）：

- **exit 0** = aeneas 全程跑通，产物 `coq-out/<Mod>.v` 写出且完整 → SUCCESS
- **exit 1 + `Generated the partial file (because of N errors)` 路径** = aeneas extract 阶段遇 unsupported 项但仍写出产物 + 报错 → FAILED（按宪法 §六-2 不允许 partial：工具自陈"没全干完"必须被尊重）
- **exit 2** = OCaml panic，无产物 → FAILED

`Primitives.v` 约 100 条 `Axiom` 是运行时库抽象（与翻译质量无关）。成功 entry 的 `<Mod>.v` 里零 `Admitted` / `Axiom`。

### SUCCESS 信号（严格反映前端特性支持范围）

为了严格反映前端特性支持范围（不允许 partial），**SUCCESS = aeneas exit 0**。任何 partial → FAILED（含 `Generated the partial file` 路径）。

**partial 暴露机制**：aeneas `Errors.error_list` 单一信号——`Main.ml:773` `if has_errors then exit 1`。

**形式严格性 — 0 误报（不冤枉能力）**：✅ 形式可证。aeneas exit 0 ⇔ `Errors.error_list` 空

**形式严格性 — 0 漏报（不高估能力）**：✅ 实测 + wrapper 双通路封堵。主通路 `craise` → exit ≠ 0；**v6 cc-route audit (2026-05-12) 发现 Warn 通道 partial 自陈**（不走 craise，exit 仍 0），wrapper 加 grep gate 拦截。

**漏报盲点**（2026-05-12 v6 修订）：

- aeneas Warn 通道 partial（已 wrapper 封堵）：mutually-recursive trait / associated type / builtin model 缺字段 / core trait method silent drop 四类，wrapper grep 4-pattern OR → FAILED
- 上游若新增 Warn 通道 partial 自陈 → 需扩展 wrapper grep pattern list

## 安装

上游：<https://github.com/AeneasVerif/aeneas>（前端 charon 上游：<https://github.com/AeneasVerif/charon>）。

本测试基线：aeneas commit `a14083a6` + 自家 charon v0.1.184（commit `ed22146b`）。aeneas binary 自带 4 个 backend，4 个工具 entry 共享同一 binary 与 charon。

按上游文档自行安装；装好后把 `aeneas` 可执行文件路径填到 `.env` 的 `TS_AENEAS_BIN`，把 aeneas 自家 charon 可执行文件路径填到 `TS_CHARON_BIN`。Coq backend 自身不需要额外组件——`aeneas -backend coq` 只生成 `.v` 文件，不调用 `coqc`。本项目不提供安装脚本或步骤教程，避免在工具版本变迁后误导。

## 本框架配置

参见 `tool.toml` + `aeneas-coq-wrapper.sh`。关键：

- 两段 pipeline：`charon cargo --preset=aeneas` → `<crate>.llbc` → `aeneas -backend coq <file>.llbc`
- shell wrapper 把两段命令包成单一 tool 入口
- `entry_mode = "lib"`
- aeneas 自家 charon 路径 `/tmp/aeneas-src/charon/bin/charon`，独立于框架原 charon
- 输出目录：`coq-out/`，文件类型 `.v`（同时复制 `Primitives.v`）

## 已知限制 / 坑

- macOS 上安装时需用 `gmake`（见 aeneas-lean 安装步骤）
- Coq backend 文档覆盖相对 Lean 少；extraction 功能本身稳定，但 Coq proof 侧需要手工补充
- charon-pin 路径独立于框架原 charon，切勿混用
- 仅测 extraction（`aeneas` 退出码）；`coqc` 类型检查不在本 testsuite 范围内

## 关联 sub-tests

`examples/aeneas-limit/` 是 Aeneas（不分 backend）自声明的限制集——故意触发已知"不支持"特性（nested borrow / closure-if-capture / float types / bool bitwise ops / mutually-recursive traits 等），期望本 backend 在这些 entry 上 FAILED。
