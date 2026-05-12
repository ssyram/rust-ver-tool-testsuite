# Aeneas → F*

Aeneas 是 Rust → 多 PA 翻译器，前端用 charon 出 LLBC 中间表示；本工具是 F\* backend。

## 简介

Aeneas (`AeneasVerif/aeneas`) 是 Rust → F\*/Coq/Lean/HOL4 翻译器，OCaml 写。前端用 charon (LLBC)，后端按 backend flag 切换。本配置 `aeneas -backend fstar` 产出 `.fst` 文件。

主仓库：https://github.com/AeneasVerif/aeneas

## 本测试集中的"前端接受"定义

本测试集筛选 Rust **前端特性覆盖广度**——把每个工具测到"工具自带后端验证器/求解器之前"为止。

Aeneas 是**纯翻译工具**，pipeline 终点是 `.fst` 文件落盘——下游 F* type-check 不在测试范围。pipeline 与 aeneas-lean 同构（`charon → LLBC → aeneas -backend fstar → .fst`），4 个 backend 共享同一份 charon + aeneas，差异在最后的 `Extract.ml` printer 分支。

- **前端**（本工具检测的范围）：写出 `fstar-out/<Mod>.fst` + `Primitives.fst`
- **后端**（本工具不检测）：用户自己拿 `.fst` 给 F* / Karamel 做下游验证

判定（精确实测语义）：

- **exit 0** = aeneas 全程跑通，产物 `fstar-out/<Mod>.fst` 写出且完整 → SUCCESS
- **exit 1 + `Generated the partial file (because of N errors)` 路径** = aeneas extract 阶段遇 unsupported 项但仍写出产物 + 报错 → FAILED（按宪法 §六-2 不允许 partial：工具自陈"没全干完"必须被尊重）
- **exit 2** = OCaml panic，无产物 → FAILED

### SUCCESS 信号（严格反映前端特性支持范围）

为了严格反映前端特性支持范围（不允许 partial），**SUCCESS = aeneas exit 0**。任何 partial → FAILED（含 `Generated the partial file` 路径）。

**partial 暴露机制**：aeneas `Errors.error_list` 单一信号——`Main.ml:773` `if has_errors then exit 1`。

**形式严格性 — 0 误报（不冤枉能力）**：✅ 实测 + 源码层论证。aeneas exit 0 ⇔ `Errors.error_list` 空

**形式严格性 — 0 漏报（不高估能力）**：✅ 实测 + wrapper 双通路封堵。主通路 `craise` → exit ≠ 0；**v6 cc-route audit (2026-05-12) 发现 Warn 通道 partial 自陈**（不走 craise，exit 仍 0），wrapper 加 grep gate 拦截。

**漏报盲点**（2026-05-12 v6 修订）：

- aeneas Warn 通道 partial（已 wrapper 封堵）：mutually-recursive trait / associated type / builtin model 缺字段 / core trait method silent drop 四类，wrapper grep 4-pattern OR → FAILED
- 上游若新增 Warn 通道 partial 自陈 → 需扩展 wrapper grep pattern list

## 安装

上游：<https://github.com/AeneasVerif/aeneas>（前端 charon 上游：<https://github.com/AeneasVerif/charon>）。

本测试基线：aeneas commit `a14083a6` + 自家 charon v0.1.184（commit `ed22146b`）。aeneas binary 自带 4 个 backend，4 个工具 entry 共享同一 binary 与 charon。

按上游文档自行安装；装好后把 `aeneas` 可执行文件路径填到 `.env` 的 `TS_AENEAS_BIN`，把 aeneas 自家 charon 可执行文件路径填到 `TS_CHARON_BIN`。F\* backend 自身不需要额外组件——`aeneas -backend fstar` 只生成 `.fst` 文件，不调用 `fstar.exe`。本项目不提供安装脚本或步骤教程，避免在工具版本变迁后误导。

## 本框架配置

参见 `tool.toml` + `aeneas-fstar-wrapper.sh`。关键：

- 两段 pipeline：`charon cargo --preset=aeneas` → `<crate>.llbc` → `aeneas -backend fstar <file>.llbc`
- shell wrapper 把两段命令包成单一 tool 入口
- `entry_mode = "lib"`
- aeneas 自家 charon 路径 `/tmp/aeneas-src/charon/bin/charon`，独立于框架原 charon
- 输出目录：`fstar-out/`，文件类型 `.fst`（同时复制 `Primitives.fst`）

## 已知限制 / 坑

- macOS 上安装时需用 `gmake`（见 aeneas-lean 安装步骤）
- F\* backend 在 Aeneas 中文档覆盖相对 Lean 少，但核心 extraction 功能同样稳定
- charon-pin 路径独立于框架原 charon，切勿混用

## 关联 sub-tests

`examples/aeneas-limit/` 是 Aeneas（不分 backend）自声明的限制集——故意触发已知"不支持"特性（nested borrow / closure-if-capture / float types / bool bitwise ops / mutually-recursive traits 等），期望本 backend 在这些 entry 上 FAILED。
