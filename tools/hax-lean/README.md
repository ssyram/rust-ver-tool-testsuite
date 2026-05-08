# Hax → Lean

Hax 是 Rust → 多个证明助手翻译器；本工具是 Lean backend。

## 简介

Hax (`hacspec/hax`) 是 Rust → F\*/Coq/Lean/EasyCrypt/SSProve/ProVerif 多 backend 翻译器，OCaml 写的引擎 + Rust 写的前端。本配置使用 `cargo hax into lean` 子命令将 Rust crate 翻译为 Lean 文件。

主仓库：https://github.com/hacspec/hax

## 本测试集中的"前端接受"定义

本测试集筛选 Rust **前端特性覆盖广度**——把每个工具测到"工具自带后端验证器/求解器之前"为止，不看下游求解结果。

Hax 是**纯翻译工具**，pipeline 终点是 `.lean` 文件落盘——下游 Lean type-check / 证明完全不在测试范围。

- **前端**（本工具检测的范围）：rustc → frontend exporter (THIR JSON) → engine (OCaml/Rust) → phase pipeline (reject + 改写) → Lean printer 写出 `<work_dir>/proofs/lean/extraction/<crate>.lean`
- **后端**（本工具不检测）：用户自己拿 `.lean` 给 Lean 编译器 / Mathlib / 自定义证明库做下游验证

### 判定与 sentinel body

cargo hax 的 exit 码语义是：**只要任何阶段发了 `FromEngine::Diagnostic` 消息就 exit 1**。但 hax engine 在某些 case 下把不能完整翻译的项替换为 sentinel body（`sorry` / `Inhabited.default`）继续 print 出 declaration——这种情况 exit 1 但产物里函数 declaration 仍在。

按宪法 §六-2 不允许 partial：sentinel body（term-position `sorry` / `pure sorry` 等）即工具自陈"这一项我没翻完"，按精神必须 → FAILED。所以判定为：cargo hax exit 0 + 产物 grep 不命中 term-position sorry 才是 SUCCESS；否则 FAILED。

`sorry` 在 Lean 4 是 term-level placeholder（admit "T 这个类型有项"，不是命题级 admit），是 hax engine emit 失败 fallback 时的标志——按宪法判 FAILED。

### SUCCESS 信号（严格反映前端特性支持范围）

为了严格反映前端特性支持范围（不允许 partial），**SUCCESS = cargo hax exit 0 **且** 产物（strip Lean `--` 行注释后）grep 不命中 sorry 在 term 位置**。任何 partial → FAILED。

精准 grep（实测 0 误报 0 漏报）：先 `awk '{ sub(/--.*/, ""); print }'` strip 注释，再 grep `(:=|pure|mk|,)\s*sorry\b|\bsorry\s*[,)\]]` —— 抓 sorry 在 term 位置（`:= sorry` / `pure sorry` / `mk sorry` / `, sorry` / `sorry,)` 等），不抓 binder 位置（`let sorry :` 是用户合法变量名）。

**partial 暴露机制（双轨）**：
1. cargo hax exit 1 = engine emit `FromEngine::Diagnostic`（官方信号）
2. silent path：`rust-engine/src/backends/lean.rs:1287, 2163` 的 `PatKind::Error / error_node` 路径 emit `text!("sorry")` **不发 Diagnostic**——cargo hax 仍 exit 0 但产物含 term-position sorry → 精准 grep 抓

**形式严格性 — 0 误报（不冤枉能力）**：⚠️ 实测验证 0 误报，但**不可形式证明**。grep 模式经过实测：用户合法 `let sorry : i32 := 5;` + doc comment 含 `sorry` 字面字符串都不触发 FAILED；真 partial（`(pure sorry)` / `mk sorry`）稳定触发 FAILED。但理论上未来 hax 输出可能引入新合法语法让 grep 误命中——本测试以**实测正确性**作底，不做形式证明。

**形式严格性 — 0 漏报（不高估能力）**：⚠️ 实测验证 0 漏报，但**不可形式证明**。grep 抓 hax-lean 已知 silent path（`rust-engine/src/backends/lean.rs:1287, 2163` 的 PatKind::Error / error_node 路径）；理论上 hax 上游可能引入新 silent path 而 grep 滞后

**漏报盲点**：
- hax engine 完全 skip item（item 既不写 sorry 也不发 Diagnostic，**不出现在产物里**）—— oracle 抓不到，需要对应性脚本暴露（实测在 examples corpus 0 现象）
- 上游 PR #1672 合并后 lean.rs 的 sorry path 应消失，届时 grep 自然失效（无害）

注：hax-lean prelude（`hax/proof-libs/lean/`）不复制到 `proofs/lean/extraction/`——grep 只看 extraction 目录不会误命中 prelude。

## 安装

上游：<https://github.com/hacspec/hax>

本测试基线：commit `30949eb87058895c24f963df90dd30ef11b0dc1a`（搭配 nightly toolchain `nightly-2025-11-08`）。三个 hax backend（coq / fstar / lean）共享同一 `hax-engine` OCaml binary——装一次即可。

按上游文档自行安装（OCaml + opam + node + jq + 上述 nightly toolchain；`setup.sh` 同时构建 Rust CLI 与 OCaml 引擎）。装好后把 `hax-engine` 可执行文件路径填到 `.env` 的 `TS_HAX_ENGINE_BIN`，并确保 `cargo-hax` 在 PATH 中。Lean backend 自身不需要额外组件——`cargo hax into lean` 只生成 `.lean` 文件，不调用 Lean 编译器。本项目不提供安装脚本或步骤教程，避免在工具版本变迁后误导。

## 本框架配置

参见 `tool.toml`。关键参数：

- `command` 含 `cargo +nightly-2025-11-08 hax -C --lib ; into lean`
  - `-C --lib ;` 限制只翻译 lib target，跳过 runner 注入的 `src/bin/__ts_harness.rs`
  - `;` 是 hax 自己的 cargo-args 终结符，作为 argv 数组元素直接传入，无需 shell 引用
  - `+nightly-2025-11-08` 与 hax 编译时的 ABI 必须匹配
- `env` 前缀注入 `HAX_ENGINE_BINARY=~/.opam/default/bin/hax-engine`
- `timeout_secs = 600`
- `entry_mode` 使用默认 `bin`；runner 将 harness 写为 bin target，hax 通过 `-C --lib ;` 只提取 lib，两者不干扰
- 提取输出写入 `<work_dir>/proofs/lean/extraction/<crate_name>.lean`，由 hax 自动创建目录

## 已知限制 / 坑

- Lean backend 在 hax upstream 标记为 active development（实验性），部分 Rust 构造会被跳过
- 不支持的构造（如 `dyn Trait`、返回 `&mut T`）：hax 打印 `[HAX0001]` 错误并以 exit 1 退出，runner 记录 FAILED——这是正确信号
- `HAX_ENGINE_BINARY` 必须为绝对路径，机器迁移时需更新 `tool.toml`
- hax 内部调用 `cargo metadata`；work dir 必须在 `runs/` 下（已在主 `Cargo.toml` 的 `exclude` 中排除），否则会触发 CargoMetadata panic

## 关联 sub-tests

`examples/hax-limit/` 是 Hax（不分 backend）自声明的限制集——这些 entry 故意触发 Hax 已知"不支持"特性（如返回 `&mut`、let-chains、closure mutating outer、labelled-break、unsafe-block 等），期望本 backend 在这些 entry 上 FAILED。
