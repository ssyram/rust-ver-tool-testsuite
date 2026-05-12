# Aeneas → HOL4

Aeneas 是 Rust → 多 PA 翻译器，前端用 charon 出 LLBC 中间表示；本工具是 HOL4 backend。

## 简介

Aeneas (`AeneasVerif/aeneas`) 是 Rust → F\*/Coq/Lean/HOL4 翻译器，OCaml 写。前端用 charon (LLBC)，后端按 backend flag 切换。本配置 `aeneas -backend hol4` 产出 `.sml` 文件（Standard ML HOL4 Theory Script）。

主仓库：https://github.com/AeneasVerif/aeneas

## 本测试集中的"前端接受"定义

本测试集筛选 Rust **前端特性覆盖广度**——把每个工具测到"工具自带后端验证器/求解器之前"为止。

Aeneas 是**纯翻译工具**，pipeline 终点是 `.sml`（HOL4 Theory Script）文件落盘——下游 HOL4 加载不在测试范围。pipeline 与 aeneas-lean 同构（`charon → LLBC → aeneas -backend hol4 → <Mod>Script.sml`），4 个 backend 共享同一份 charon + aeneas，差异在最后的 `Extract.ml` printer 分支。

- **前端**（本工具检测的范围）：写出 `hol4-out/<CamelCaseMod>Script.sml`
- **后端**（本工具不检测）：用户自己拿 `.sml` 给 polyml / Holmake 加载

### HOL4 backend 的硬天花板

矩阵上 aeneas-hol4 ~37% 显著低于其它 3 个 backend (~60%)，**单一根因**在 `Extract.ml + ExtractBase.ml` 的 backend 分支：

- `ExtractBase.ml:1412 type_decl_kind_to_qualif` 在 HOL4 上 trait decl 始终返回 `None`
- `Extract.ml:3166 extract_trait_decl` 调 `Option.get` 不防御
- **LLBC 里只要含 ≥1 个 trait declaration（`FnOnce` / `From` / `Iterator`...）就抛 `Invalid_argument "option is None"` 截断产物**
- 与 entry 是否真"用"该 trait 无关，与下游 prover 无关

矩阵 60/60/60/37 的差距 ≈ "矩阵上含 trait declaration 的 entry 比例"。这是 aeneas-hol4 upstream 的硬天花板，作为工具事实陈述。

判定（精确实测语义）：

- **exit 0** = aeneas 全程跑通，产物 `hol4-out/<CamelCaseMod>Script.sml` 写出且完整 → SUCCESS
- **exit 1 + `Generated the partial file`** = aeneas hol4 backend 在 extract 阶段遇 unsupported 项（最常见即上述 trait_decl 触发）但仍写出产物 + 报错 → FAILED（按宪法 §六-2 不允许 partial：工具自陈"没全干完"必须被尊重）
- **exit 2** = OCaml panic（绝大部分是 `trait_decl_kind_to_qualif None` 触发的 `Invalid_argument "option is None"`），无产物 → FAILED

aeneas-hol4 的 FAILED 集合中 `trait_decl` panic 占比最高，是该 backend 通过率显著低于其它 3 个的单一原因。

### SUCCESS 信号（严格反映前端特性支持范围）

为了严格反映前端特性支持范围（不允许 partial），**SUCCESS = aeneas exit 0**。任何 partial → FAILED。

**partial 暴露机制**：aeneas `Errors.error_list` 单一信号；hol4 backend 还有 `trait_decl_kind_to_qualif` 触发的 OCaml panic（exit 2）—— 共同指向 hol4 backend 的硬天花板。

**形式严格性 — 0 误报（不冤枉能力）**：✅ 形式可证。aeneas exit 0 ⇔ `Errors.error_list` 空 + 无 OCaml panic

**形式严格性 — 0 漏报（不高估能力）**：✅ 实测 + wrapper 双通路封堵。主通路 `craise` / `Invalid_argument` panic → exit ≠ 0；**v6 cc-route audit (2026-05-12) 发现 Warn 通道 partial 自陈**（不走 craise，exit 仍 0），wrapper 加 grep gate 拦截。

**漏报盲点**（2026-05-12 v6 修订）：

- aeneas Warn 通道 partial（已 wrapper 封堵）：mutually-recursive trait / associated type / builtin model 缺字段 / core trait method silent drop 四类，wrapper grep 4-pattern OR → FAILED
- 上游若新增 Warn 通道 partial 自陈 → 需扩展 wrapper grep pattern list

## 安装

上游：<https://github.com/AeneasVerif/aeneas>（前端 charon 上游：<https://github.com/AeneasVerif/charon>）。

本测试基线：aeneas commit `a14083a6` + 自家 charon v0.1.184（commit `ed22146b`）。aeneas binary 自带 4 个 backend，4 个工具 entry 共享同一 binary 与 charon。

按上游文档自行安装；装好后把 `aeneas` 可执行文件路径填到 `.env` 的 `TS_AENEAS_BIN`，把 aeneas 自家 charon 可执行文件路径填到 `TS_CHARON_BIN`。HOL4 backend 自身不需要额外组件——`aeneas -backend hol4` 只生成 `.sml` Theory Script，不调用 HOL4（`polyml` / `Holmake`）。本项目不提供安装脚本或步骤教程，避免在工具版本变迁后误导。

## 本框架配置

参见 `tool.toml` + `aeneas-hol4-wrapper.sh`。关键：

- 两段 pipeline：`charon cargo --preset=aeneas` → `<crate>.llbc` → `aeneas -backend hol4 <file>.llbc`
- shell wrapper 把两段命令包成单一 tool 入口
- `entry_mode = "lib"`
- aeneas 自家 charon 路径 `/tmp/aeneas-src/charon/bin/charon`，独立于框架原 charon
- 输出目录：`hol4-out/`，文件类型 `.sml`（命名规范：`<CamelCaseCrateName>Script.sml`）

## 已知限制 / 坑

- macOS 上安装时需用 `gmake`（见 aeneas-lean 安装步骤）
- HOL4 backend 在 Aeneas 中标记为成熟，但 `.sml` 加载到 HOL4 需要额外 `primitivesLib`
- charon-pin 路径独立于框架原 charon，切勿混用
- 仅测 extraction（`aeneas` 退出码）；HOL4 Theory 加载与验证不在本 testsuite 范围内

## 关联 sub-tests

`examples/aeneas-limit/` 是 Aeneas（不分 backend）自声明的限制集——故意触发已知"不支持"特性（nested borrow / closure-if-capture / float types / bool bitwise ops / mutually-recursive traits 等），期望本 backend 在这些 entry 上 FAILED。
