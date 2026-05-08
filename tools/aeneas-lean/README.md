# Aeneas → Lean

Aeneas 是 Rust → 多 PA 翻译器，前端用 charon 出 LLBC 中间表示；本工具是 Lean 4 backend。

## 简介

Aeneas (`AeneasVerif/aeneas`) 是 Rust → F\*/Coq/Lean/HOL4 翻译器，OCaml 写。前端用 charon (LLBC)，后端按 backend flag 切换。本配置 `aeneas -backend lean` 产出 `.lean` 文件。

主仓库：https://github.com/AeneasVerif/aeneas

## 本测试集中的"前端接受"定义

本测试集筛选 Rust **前端特性覆盖广度**——把每个工具测到"工具自带后端验证器/求解器之前"为止。

Aeneas 是**纯翻译工具**，pipeline 终点是 `.lean` 文件落盘——下游 Lean type-check / 证明不在测试范围。pipeline 含两阶段：

```
charon cargo --preset=aeneas  →  <crate>.llbc       [Stage A]
aeneas -backend lean <crate>.llbc  →  <Mod>.lean    [Stage B]
```

Stage B 里 aeneas 单进程跑完整翻译：read_llbc → translate_crate_to_pure（symbolic interp + SymbolicToPure + PureMicroPasses）→ extract_translated_crate（Extract.ml printer 写文件）。

- **前端**（本工具检测的范围）：Stage A + Stage B 全程，写出 `lean-out/<Mod>.lean`
- **后端**（本工具不检测）：用户自己拿 `.lean` 给 Lean 编译器做下游 type-check / 证明

判定（精确实测语义）：

- **exit 0** = aeneas 全程跑通，产物 `lean-out/<Mod>.lean` 写出且完整 → SUCCESS
- **exit 1 + `Generated the partial file (because of N errors)` 路径** = aeneas 在 extract 阶段遇 N 个 unsupported 项，仍写出产物（含可处理的 fn）+ 报错 → FAILED（按宪法 §六-2 不允许 partial：工具自陈"没全干完"必须被尊重，即便产物落盘也是 FAILED）
- **exit 2** = OCaml panic / 内部异常，无产物 → FAILED
- **charon stage 失败** = `<crate>.llbc` 没产出，aeneas stage 不启动 → FAILED

成功 entry 产物里**零 sorry / Admitted**——aeneas 的 backward function 模型（`fn f(x: &mut T) -> U` ⟼ `f : T → U × T`）是纯函数化，**不引入证明义务**。`Primitives.lean` 含约 100 条 `Axiom` 是运行时库抽象（`isize_min` / `array_index_usize` 等），所有 entry 共享，与翻译质量无关。

### SUCCESS 信号（严格反映前端特性支持范围）

为了严格反映前端特性支持范围（不允许 partial），**SUCCESS = aeneas exit 0**（含 charon stage exit 0）。任何 partial → FAILED（含 `Generated the partial file (because of N errors)` 路径——这是 aeneas 主动声明 "我没全干完"）。

**partial 暴露机制**：aeneas `Errors.error_list` 单一信号——每次 `craise` push 一项；结束时 `Main.ml:773` `if has_errors then exit 1`。任何 unsupported 项都 push error_list；exit 2 = OCaml panic 无产物。

**形式严格性 — 0 误报（不冤枉能力）**：✅ 形式可证。aeneas exit 0 ⇔ `Errors.error_list` 空 ⇔ 翻译完整

**形式严格性 — 0 漏报（不高估能力）**：✅ 形式可证。aeneas 用 `craise` 把所有 unsupported 项 push error_list；`Main.ml:773` `if has_errors then exit 1` 是单一信号通路。aeneas **不存在** silent emit-stub-but-exit-0 路径

**漏报盲点**：无（依赖 aeneas 上游正确实现 craise——已知所有 unsupported 路径都走 craise）

## 安装

上游：<https://github.com/AeneasVerif/aeneas>（前端 charon 上游：<https://github.com/AeneasVerif/charon>）。

本测试基线：aeneas commit `a14083a6` + 自家 charon v0.1.184（commit `ed22146b`，由 `charon-pin` 锁定）。aeneas binary 自带 4 个 backend（fstar / coq / lean / hol4），4 个工具 entry 共享同一 binary 与 charon。

按上游文档自行安装（OCaml + opam + GNU make + `charon-pin` 指定的 charon commit + `gmake build-bin-dir` 等自行处理）。装好后把 `aeneas` 可执行文件路径填到 `.env` 的 `TS_AENEAS_BIN`，把 aeneas 自家 charon 可执行文件路径填到 `TS_CHARON_BIN`（与 charon-mono/charon-poly 用的 charon 是同一上游同一版本，但二进制路径独立维护，请避免覆盖）。Lean 4 backend 自身不需要额外组件——`aeneas -backend lean` 只生成 `.lean` 文件，不调用 Lean checker。本项目不提供安装脚本或步骤教程，避免在工具版本变迁后误导。

## 本框架配置

参见 `tool.toml` + `aeneas-lean-wrapper.sh`。关键：

- 两段 pipeline：`charon cargo --preset=aeneas` → `<crate>.llbc` → `aeneas -backend lean <file>.llbc`
- shell wrapper 把两段命令包成单一 tool 入口
- `entry_mode = "lib"`
- aeneas 自家 charon 路径 `/tmp/aeneas-src/charon/bin/charon`，独立于框架原 charon
- 输出目录：`lean-out/`，文件类型 `.lean`

## 已知限制 / 坑

- macOS 上必须用 `gmake`（Homebrew），不能用系统自带 `make`
- `gmake build-bin-dir` 需要 opam PATH 显式前缀，否则找不到 `dune`
- charon-pin 与主框架 charon 版本号相同（v0.1.184）但二进制路径独立，切勿混用
- `charon-ml` 构建失败（无 dune 时）无害，只需要 `bin/charon`
- FnMut 闭包返回 `()`、trait `&mut` 实例化参数不匹配等场景下 aeneas 能产出文件但 Lean 类型检查会失败（issues #960, #961）

## 关联 sub-tests

`examples/aeneas-limit/` 是 Aeneas（不分 backend）自声明的限制集——故意触发已知"不支持"特性（nested borrow / closure-if-capture / float types / bool bitwise ops / mutually-recursive traits 等），期望本 backend 在这些 entry 上 FAILED。
