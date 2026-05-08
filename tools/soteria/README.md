# Soteria

第一个原生支持 Tree Borrows 的 Rust 符号执行引擎，对别名模型 bug 的检测比 Kani 更精确。

## 简介

Soteria 由 soteria-tools 组出品，使用 OCaml 实现，通过 Obol 前端（Rust → ULLBC）和 Charon 中间表示进行符号执行。内核原生实现 Tree Borrows 别名模型，能够检测标准借用检查器无法发现的 unsafe 别名违规。当前无稳定 release，锁定到 commit `3c21278187c60c99418fe2dabb03710ce4102896`。

GitHub: <https://github.com/soteria-tools/soteria>

## 本测试集中的"前端接受"定义

本测试集筛选 Rust **前端特性覆盖广度**——把每个工具测到"工具自带后端验证器/求解器之前"为止。

Soteria 是符号执行引擎；它的 pipeline：obol (Rust → ULLBC/LLBC JSON) → soteria-rust 加载 LLBC → 符号执行 + Tree Borrows + provenance → Z3 SMT 调用。

按宪法 §六-2 不允许 partial（"SUCCESS = 工具完整完成它的工作单元，不允许任何 partial / silent skip"）："工具完成它自己的工作" = soteria-rust **完整跑完符号执行且无中断**。具体：

- **算 SUCCESS**：exit 0 = 符号执行完整跑完且无 bug
- **算 FAILED**：任何中断
  - exit 1 = 检测到 bug（dangling pointer / aliasing 违规等）—— 符号执行被 bug 中断 = 没完整跑完
  - exit 2 = soteria-rust 内部 crash（symex 阶段触发未实现 case）
  - exit 3 = Charon/Obol 前端 crash（如 `charon-limit/inline-asm` / `async-fn` / unsized coercion）

> 注：bug detect 在某些视角下可理解为"工具有效输出"，但按宪法精神（不允许 partial = 必须完整跑完）一律 FAILED。oracle（runner 仅按 exit code 判定）与宪法一致。

- **产物**：`<crate>/src/lib.rs.llbc.json`（13–30 KB LLBC JSON）+ `lib.rs.llbc.json.crate`（pretty-printed LLBC，含完整函数路径 + 类型签名）+ 可选 `cg.dot` callgraph DOT
- **函数级对应性**：LLBC `.crate` 文件**逐字符给出函数路径 + 类型签名**——4 个翻译类工具中函数级映射最强

### SUCCESS 信号（严格反映前端特性支持范围）

为了严格反映前端特性支持范围（不允许 partial / 不接受中断），**SUCCESS = soteria exit 0**（符号执行完成且无 bug）。任何中断（含 bug detect）→ FAILED。

- **partial 暴露机制**：
  - exit 1 = 检测到 bug
  - exit 2 = soteria-rust 内部 crash
  - exit 3 = obol/charon 前端 crash
- **形式严格性 — 0 误报（不冤枉能力）**：✅ 形式可证。soteria exit 0 ⇔ 符号执行完成且无 bug
- **形式严格性 — 0 漏报（不高估能力）**：✅ 形式可证。exit 1/2/3 完整覆盖 bug detect / symex crash / 前端 crash 三类 partial
- **漏报盲点**：无

注：按工具自身语义 bug detect 是有效输出——但按"完整完成"精神，符号执行被 bug 中断 = 没完整跑完 → FAILED。

## 安装

上游：<https://github.com/soteria-tools/soteria-rust>（Obol 前端：<https://github.com/soteria-tools/obol>）。

本测试基线：soteria commit `3c21278187c60c99418fe2dabb03710ce4102896` + Obol commit `ddea5ca5da4c07301584f47f05ea8615fc365b41`（OCaml 5.4 + Z3，搭配 nightly-2026-02-07）。

按上游文档自行安装；装好后把含 `soteria-rust` 与 `obol` / `obol-driver` 的 `bin/` 目录路径填到 `.env` 的 `TS_SOTERIA_BIN_DIR`。本项目不提供安装脚本或步骤教程，避免在工具版本变迁后误导。

## 本框架配置

参见 `tool.toml`。关键参数：

- **command**：`sh -c` 包装，内联 `eval $(opam env --switch=soteria-install)` 激活 opam switch（使 `soteria-rust` 进入 PATH），追加 `/tmp/obol/bin` 后调用 `soteria-rust exec --rustc=--edition=2021 src/lib.rs`。
- **entry_mode = "lib"**：runner 将 `src/lib.rs` 重命名为 `src/__ts_inner.rs`，写入 harness 作为新的 `src/lib.rs`。harness 含 `mod __ts_inner;` + `fn main() { let _ = __ts_inner::entry_fn(); }`，soteria-rust 以 `fn main` 为符号执行入口。
- **--rustc=--edition=2021**：单文件模式默认 Rust 2015，不传此 flag 时现代 Rust 代码会因 edition 差异产生编译错误。
- **exit code 对齐**：0 = 无 bug（SUCCESS）；1 = 发现 bug（FAILED）；2 = soteria-rust 内部崩溃（FAILED）；3 = Charon 前端崩溃（FAILED）。框架默认 `exit 0 = SUCCESS` 完全对齐，无需特殊处理。
- **无 fuel 限制**：`--step-fuel` / `--branch-fuel` 使用默认值（无限）；testsuite 目标函数体量极小，符号执行在 10ms 内完成。

## 已知限制 / 坑

- **HashMap / collections 在 aarch64 产生 false-positive**：`std::collections::HashMap` 等在 aarch64 上使用 `stdarch` SIMD intrinsics，soteria-rust 将其报告为 dangling pointer 违规（即使代码完全正确）。受影响类目：`collections/hashmap` 等相关 entry，预期 FAILED。
- **单文件模式不支持外部 crate 依赖**：`soteria-rust exec <file.rs>` 只能解析 `std`/`core`/`alloc`，任何第三方 crate 依赖均无法编译。
- **Obol binary 路径临时**：`/tmp/obol/bin` 在重启后消失，需将 `obol` 和 `obol-driver` 复制到持久路径并更新 `tool.toml` 中的 PATH。
- **无稳定 release**：必须锁定 commit hash，重装时需 `git checkout` 精确 commit。
- **无 dry-run 选项**：最轻量调用也会完整编译（obol 翻译）+ 符号执行，无法仅做前端检查。

## 关联 sub-tests

本工具未派生限制集 agent，无 `examples/soteria-limit/`。

预期 FAILED 类目：`collections/hashmap` 相关 entry（aarch64 stdarch intrinsics false-positive）；依赖外部 crate 的样例。
