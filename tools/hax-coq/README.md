# Hax → Coq

Hax 是 Rust → 多个证明助手翻译器；本工具是 Coq backend。

## 简介

Hax (`hacspec/hax`) 是 Rust → F\*/Coq/Lean/EasyCrypt/SSProve/ProVerif 多 backend 翻译器，OCaml 写的引擎 + Rust 写的前端。本配置使用 `cargo hax into coq` 子命令将 Rust crate 翻译为 Coq/Rocq 文件。

主仓库：https://github.com/hacspec/hax

## 本测试集中的"前端接受"定义

本测试集筛选 Rust **前端特性覆盖广度**——把每个工具测到"工具自带后端验证器/求解器之前"为止。

Hax 是**纯翻译工具**，pipeline 终点是 `.v` 文件落盘——下游 `coqc` type-check / 证明不在测试范围。

- **前端**（本工具检测的范围）：rustc → frontend exporter → engine → phase pipeline → Coq printer 写出 `<work_dir>/proofs/coq/extraction/<Crate>.v`
- **后端**（本工具不检测）：用户自己拿 `.v` 给 `coqc` + hax Coq 支持库做下游验证

Coq backend 是 hax 中 reject phase 最多的（9 个：`Reject.Unsafe`、`RawOrMutPointer`、`Arbitrary_lhs`、`Continue ×2`、`EarlyExit`、`As_pattern`、`Dyn`、`Trait_item_default`），所以矩阵通过率 ~67%（低于 lean 89% / fstar 79%）。

> **Coq printer 的 silent fallback 路径**：上游源码 `engine/backends/coq/coq/coq_backend.ml:137` 的 `default_document_for s = "TODO: please implement the method `..."` 是纯文本输出，**不发 Diagnostic**——cargo hax 仍 exit 0 但 .v 文件里散布 `"TODO: please implement..."` 字面字符串。当前 oracle 不抓这种情况。

### SUCCESS 信号（严格反映前端特性支持范围）

为了严格反映前端特性支持范围（不允许 partial），**SUCCESS = cargo hax exit 0 **且** 产物 grep 不命中 `failure ((` / `please implement the method` **且** entry_fn 在 .v 中有定义**。任何 partial → FAILED。

**partial 暴露机制（三轨）**：
1. cargo hax exit 1 = engine emit Diagnostic
2. silent path A：`engine/backends/coq/coq/coq_backend.ml:137` 的 `default_string_for "TODO: please implement the method..."` 是**纯文本输出，不发 Diagnostic**——cargo hax 仍 exit 0 但产物含字面字符串 → grep 抓
3. **NEW (2026-05-11)** silent path B：`engine/backends/coq/coq/coq_backend.ml:588 method item'_NotImplementedYet = string "(* NotImplementedYet *)"` —— 当 hax-engine 在 AST 遇到 `NotImplementedYet` 类型的 item kind，把整个 item 渲染为单行 `(* NotImplementedYet *)` comment，**不写 `Definition`**。但该 marker 与每个 .v 的 boilerplate header 字面相同，不能直接 grep marker（会全 corpus 误报）。改抓 entry_fn 定义存在性：grep `^\s*(Definition|Fixpoint|Lemma|Equations|Theorem|Program Definition)\s+<entry_fn>\b` 在 .v 中命中 → SUCCESS；不命中 → FAILED。覆盖 Coq backend 三个 `CoqNotation` 分支（coq_backend.ml:454 lemma / :518 fixpoint / :540 definition）+ 容忍 Equations / Theorem / Program Definition 等上游可能引入的 keyword

**形式严格性 — 0 误报（不冤枉能力）**：✅ 实测 + 设计层论证 0 误报。规则 (1)(2) 同前；规则 (3) 的反误报：hax-coq 对 Rust `fn` 项必经过 coq_backend.ml:454-540 三个 CoqNotation 分支之一（`is_lemma` / `is_rec` / else），渲染为 `Definition` / `Fixpoint` / `Lemma` 之一——合法翻译的 entry_fn 必有这三个 keyword 的定义。双向实测验证 5 类 case（Definition / Fixpoint / Lemma / 嵌套 module / missing），见 [`docs/fixes/oracle-leak-rules-implementation-2-2026-05-11.md`](../../docs/fixes/oracle-leak-rules-implementation-2-2026-05-11.md) §2.3

**形式严格性 — 0 漏报（不高估能力）**：✅ 实测 + 源码层封堵。规则 (1)(2) 同前；规则 (3) 抓 silent path B 的 `(* NotImplementedYet *)` item-level skip——这是 audit-2 §3.3 识别出的剩余 silent path，本规则用 entry_fn 定义存在性间接抓（绕开 boilerplate marker 与 item-level marker 同字面的尴尬）

**漏报盲点**：
- `(* NotImplementedYet *)` 是每个 .v 文件的 boilerplate header（**注**：第二轮规则不直接 grep 这个 marker，改抓 entry_fn 定义存在性以兼容 boilerplate）
- hax engine 上游引入新 silent path（如 backend item kind 新增 `string "..."` 分支）—— 当前 coq_backend.ml:137 + :588 是已知 silent path

## 安装

上游：<https://github.com/hacspec/hax>

本测试基线：commit `30949eb87058895c24f963df90dd30ef11b0dc1a`（搭配 nightly toolchain `nightly-2025-11-08`）。三个 hax backend（coq / fstar / lean）共享同一 `hax-engine` OCaml binary。

按上游文档自行安装；装好后把 `hax-engine` 可执行文件路径填到 `.env` 的 `TS_HAX_ENGINE_BIN`。Coq backend 自身不需要额外组件——`cargo hax into coq` 只生成 `.v` 文件，不调用 `coqc`。本项目不提供安装脚本或步骤教程，避免在工具版本变迁后误导。

## 本框架配置

参见 `tool.toml`。关键参数：

- `command` 含 `cargo +nightly-2025-11-08 hax -C --lib ; into coq`
  - `-C --lib ;` 限制只翻译 lib target，跳过 runner 注入的 `src/bin/__ts_harness.rs`
  - `+nightly-2025-11-08` 与 hax 编译时的 ABI 必须匹配
- `env` 前缀注入 `HAX_ENGINE_BINARY=~/.opam/default/bin/hax-engine`
- `timeout_secs = 600`
- `entry_mode` 使用默认 `bin`
- 提取输出写入 `<work_dir>/proofs/coq/extraction/<CrateName>.v`，同时生成 `_CoqProject`

与 hax-lean 路径同构，差异只在 `command` 最后一个 token（`coq` vs `lean`）。

## 已知限制 / 坑

- Coq backend 在 hax upstream 标记为 **partial**：生成的 `.v` 文件中可能含 `(* NotImplementedYet *)` 注释占位，表示该构造暂未实现翻译
- 生成的 `_CoqProject` 中库名为 `TODO`（`-R ./ TODO`），未配合 hax Coq 支持库时不能直接 `coqc` 编译——这是 upstream 的预期行为
- 不支持的 Rust 构造（如 `dyn Trait`、`&mut` 返回等）：hax 以 exit 1 退出，runner 记录 FAILED
- `HAX_ENGINE_BINARY` 必须为绝对路径，机器迁移时需更新 `tool.toml`

## 关联 sub-tests

`examples/hax-limit/` 是 Hax（不分 backend）自声明的限制集——这些 entry 故意触发 Hax 已知"不支持"特性（如返回 `&mut`、let-chains、closure mutating outer、labelled-break、unsafe-block 等），期望本 backend 在这些 entry 上 FAILED。
