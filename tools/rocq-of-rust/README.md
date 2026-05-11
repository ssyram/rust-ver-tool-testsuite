# rocq-of-rust

Rust 源码到 Rocq（原 Coq）的自动翻译工具，生成可进一步做形式化证明的 `.v` 文件。

## 简介

rocq-of-rust 由 Formal Land 出品，通过 `rustc_interface` 直接读取 `.rs` 文件并翻译为 Rocq 定义。不是 cargo subcommand，是独立 binary。使用 nightly-2024-12-07 toolchain（通过 `rustc_private` API），需锁定到该 toolchain 构建。翻译结果为 Rocq monadic IR，可在 Rocq 中进一步做等价性证明或规格验证。

GitHub: <https://github.com/formal-land/rocq-of-rust>

## 本测试集中的"前端接受"定义

本测试集筛选 Rust **前端特性覆盖广度**——把每个工具测到"工具自带后端验证器/求解器之前"为止，不看下游求解结果。

rocq-of-rust 是**纯翻译工具**，没有内置的 Coq type-check / 证明阶段——pipeline 终点就是 `.v` 文件写盘。所以本工具"前端 = 全过程 = 翻译到 .v"。

- **判定**：exit 0 + 产物存在 + 通过扩展 grep marker 检测 = SUCCESS
- **产物**：`rocq_translation/<absolute-source-path>.v`（每个 fn 翻译为 `Definition <name>` + `Global Instance Instance_IsFunction_<name> : M.IsFunction.C "..." <name>. Admitted.` typeclass 注册）
- **覆盖度精确意义**：SUCCESS = "rocq-of-rust 把全部源码 item lower 到产物，且无可见落空标记"。下游 `coqc` 是否能 type-check 该 .v 文件**不在本测试范围**——这依赖 RocqOfRust runtime library 提供 std/外部 crate 的 binding

### Silent fallback 检测（当前 oracle）

rocq-of-rust 设计上对未支持的构造**仍 exit 0**，把失败信号塞进 `.v` 文件里的特定标记。当前 tool.toml 在 sh -c 尾部实施 6 道门 oracle（详见 §"SUCCESS 信号"）。门 5 是 corpus-tested 的 5 类显式 failure marker grep：

```sh
! grep -rqE '\(\* (Error |Unexpected |Please report!|thir failed to compile|Unimplemented )' rocq_translation
```

门 6（2026-05-08 新增）是 `Definition <entry_fn>` 存在性 grep。

已知 silent fallback 路径（深度调研，参考 [`deep-reports/cc-reports/rocq-of-rust.md`](../../deep-reports/cc-reports/rocq-of-rust.md)）：

- THIR 编译 panic → `(* thir failed to compile *) tt`（被门 5 抓）
- TyKind 落空 → `RocqType::var("type ... not yet handled")`（实测 0 现象，门 5 + 门 6 间接覆盖）
- `extern crate` / `use` / `macro_rules!` 静默丢（vec![] 直接返回）—— 合理 skip；但若 entry 名误指向这些 item，门 6 抓
- `TopLevelItem::Error` 系列（GlobalAsm / Union / TraitAlias）→ `(* Error <Variant> *)`（被门 5 抓）
- `ConstKind::Infer/Bound/Placeholder` → 裸名 `InferConst` / `BoundConst` / `PlaceholderConst`（const 单独 case，本 corpus 实测 0 现象）
- `lib/src/core.rs:157` 的 HashMap.next() 单文件丢盘 → 多 mod 项目仅第一个文件入 .v（这种 fn 缺失会被门 6 抓）

### SUCCESS 信号（严格反映前端特性支持范围）

为了严格反映前端特性支持范围（不允许 partial），rocq-of-rust 的 SUCCESS 必须满足 **6 道门**（2026-05-08 起；现行 `tool.toml` 实施）：

1. exit code = 0
2. 至少一个 `.v` 产物存在
3. 无 0-byte `.v`
4. 至少一个 `.v` > 200 字节
5. 产物不含显式 failure marker grep：`\(\* (Error |Unexpected |Please report!|thir failed to compile|Unimplemented )`
6. **NEW**：产物中至少一个 `.v` 文件含 `^[[:space:]]*Definition[[:space:]]+<TS_ENTRY_FN>[[:space:]]` —— entry 函数名必须真出现在 Rocq 产物里（非 silent skipped）

任何门未满足 → FAILED。

第 6 道门由 runner 注入的 `TS_ENTRY_FN` 环境变量驱动（`runner/src/exec.rs:178`），封堵 audit ([`docs/fixes/oracle-leak-audit-2026-05-08.md`](../../docs/fixes/oracle-leak-audit-2026-05-08.md) §3.2) 提到的"完全 skip item 类"silent漏报路径——典型场景：entry 名实际是 `use` / `extern crate` 别名或者其他非 fn item，rocq-of-rust 在 `top_level.rs:349-390` 走 `vec![]`，产物中没有该 fn 的 `Definition`。`^[[:space:]]*` 锚点允许嵌套模块内的 fn 也命中。

**partial 暴露机制**：rocq-of-rust **设计上不用 exit code 表达 partial**（几乎永远 exit 0，对所有 unsupported 用 rustc warning，不影响 exit）。所以 oracle 完全靠产物 grep + 产物 shape 检测——这是工具自身设计决定的"前端测试范围"切割方式。

**形式严格性 — 0 误报（不冤枉能力）**：⚠️ 实测验证 0 误报，但**不可形式证明**。oracle 用保守的 marker 集——只抓 rocq-of-rust 自己 emit 的 explicit failure comment 块（`(* (Error |Unexpected |Please report!|thir failed to compile|Unimplemented )`），用户合法代码极难误命中。早期试探性 16-marker 已主动收缩到这 5 类显式 failure marker 以避免误报。门 6 的 grep `Definition <name>` 是 rocq-of-rust 翻译产物的固定形式（每个 fn item 必生成 `Definition <name> ...`），合法代码不会缺。

**形式严格性 — 0 漏报（不高估能力）**：⚠️ 实测验证 0 漏报，但**不可形式证明**。rocq-of-rust **设计上不用 exit code 表达 partial**（永远 exit 0，对所有 unsupported 用 rustc warning），所以 oracle 只能靠产物字面 grep + 产物 shape 检测；理论上 rocq-of-rust 上游可能引入新 fallback 路径不带这些 marker。门 6 把 entry-level silent skip 类闭环。

**漏报盲点**：
- 上游引入新 silent fallback 路径不带已知 markers（实测在 examples corpus 0 现象）
- 完全 skip item 类（`use` / `extern crate` / `macro_rules!` 在 `top_level.rs:349-390` 直接 `vec![]`）：这些是 rustc 编译时已被处理的 import / macro，**不需要在产物里有 declaration**，所以是合理 skip，**不算漏报**——但若 entry_fn 名误指向这些 item（错误 corpus 设置），门 6 会捕获

## 安装

上游：<https://github.com/formal-land/rocq-of-rust>

本测试基线：commit `a8a76a4d`（cli/，搭配 nightly toolchain `nightly-2024-12-07`）。

按上游文档自行安装（rustup 装好 nightly toolchain 后从 cli/ `cargo install` 自行处理）。`rocq-of-rust` 本体由 PATH 解析；本工具不直接配置 binary 路径，但需要把对应 nightly toolchain 的 sysroot 目录填到 `.env` 的 `TS_ROCQ_OF_RUST_TOOLCHAIN_SYSROOT`（runtime 注入 `DYLD_LIBRARY_PATH` 与 `PATH`，使 rocq-of-rust 找到自身 `librustc_driver-*.dylib` 与 nightly `rustc`）。本项目不提供安装脚本或步骤教程，避免在工具版本变迁后误导。

## 本框架配置

参见 `tool.toml`。关键参数：

- **command**：`sh -c` 包装，内联设置运行时环境变量后调用 `rocq-of-rust translate --path src/lib.rs --output-path rocq_translation`，最后 `! grep -rqE '(Unexpected|Please report!)' rocq_translation` 检测静默错误。
- **entry_mode**：未设置（默认 `bin`）；harness 写入 `src/bin/__ts_harness.rs`，但 rocq-of-rust 只读 `src/lib.rs`，harness 不参与翻译。
- **DYLD_LIBRARY_PATH**：指向 nightly-2024-12-07 sysroot 的 `lib/`，使 `librustc_driver-*.dylib` 在 macOS 上可被动态链接器找到。
- **PATH 注入**：将 nightly sysroot 的 `bin/` 置于 PATH 最前，确保 rocq-of-rust 内部调用 `rustc --print=sysroot` 时返回 nightly sysroot（而非 stable）。
- **静默吞错升级**：rocq-of-rust 对不支持的构造 exit 0 但在 `.v` 文件中嵌入 `(* Unexpected ... *)` 或 `(* Please report! *)` 注释块。框架通过在 `sh -c` 末尾追加 `! grep -rqE '...' rocq_translation` 将这类静默失败升级为非 0 exit，映射为 FAILED。

## 已知限制 / 坑

- **静默吞错**：工具无 `--abort-on-error` 等价旗标，遇不支持的构造不报 exit 非 0，必须依赖 grep 检测 `.v` 文件中的占位注释来识别翻译失败。
- **单文件输入**：rocq-of-rust 通过 `rustc_interface` 直接处理 `.rs` 文件，不读 Cargo.toml，不支持跨 crate 依赖。
- **toolchain 锁定**：必须使用 nightly-2024-12-07 构建 binary 并在运行时注入对应 sysroot；换 toolchain 需重新 `cargo install`。
- **输出路径结构**：`--output-path` 指定目录后，输出路径为 `<output-path>/<绝对输入路径>.v`，需提前 `mkdir -p`。
- **翻译质量**：Rust 语言特性覆盖不完整，复杂 trait 实现、`unsafe` 指针操作、宏展开后的代码等易触发占位注释。

## 关联 sub-tests

本工具未派生限制集 agent，无 `examples/rocq-of-rust-limit/`。

翻译成功（且 `.v` 文件无占位注释）的样例预期 SUCCESS；触发静默错误的样例预期 FAILED（grep 将 exit 0 翻转为非 0）。
