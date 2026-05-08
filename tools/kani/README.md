# Kani

有界模型检测器（bounded model checker）——基于 CBMC 的 Rust 前端验证工具。

## 简介

Kani 由 AWS 开发，将 Rust MIR 翻译为 CBMC 可消费的 GotoC IR，再由 CBMC（基于 SAT/SMT）做有界模型检测。支持对内存安全、算术溢出、断言可达性等性质进行自动验证，入口用 `#[kani::proof]` 标注。官方文档：<https://model-checking.github.io/kani/>

## 本测试集中的"前端接受"定义

本测试集筛选 Rust **前端特性覆盖广度**——把每个工具测到"工具自带后端验证器/求解器之前"为止，不看下游求解结果。

对 Kani，这条边界落在 **MIR → GotoC codegen** 与 **CBMC SAT/SMT 求解** 之间：

- **前端**（本工具检测的范围）：kani-compiler (rustc plugin) 把 MIR 翻译为 GotoC IR，最终写出 goto-binary（`*.symtab.out`）
- **后端**（本工具不检测）：CBMC 从 goto-binary 起 SAT/SMT 求解，判 reachability / assertion violation

`--only-codegen` 是 kani 上 CBMC 的标准切线（man page："Kani will only compile the crate. No verification will be performed"）。

- **判定**：exit 0 = SUCCESS（codegen 完成）；exit ≠ 0 = FAILED（kani-compiler 在 codegen 阶段失败）
- **产物**：`target/kani/<triple>/debug/deps/__ts_harness-*.symtab.out`（goto-binary）+ `*.kani-metadata.json` + `*.pretty_name_map.json`（含 1100+ 项 demangled 函数级映射，含 closure mangled 名）
- **关键不变量**：stderr 中 `Found the following unsupported constructs: caller_location / foreign function ...` 只是 warning，不影响 exit 0、不影响 codegen 完成。它对 SAT 求解阶段才有意义（哪个分支 trap），但本测试不到那一步
- **公平性**：让 kani 跑完整 CBMC 求解会失公——其他工具（cargo-check / charon / hax / creusot 等）都在前端层停下，kani 却因求解器在集合/并发/递归枚举等场景下超时而频繁 FAILED，制造假阴性

### SUCCESS 信号（严格反映前端特性支持范围）

为了严格反映前端特性支持范围（不允许 partial），**SUCCESS = `cargo kani --only-codegen` exit 0**（goto-binary 完整生成，CBMC 不调用）。任何 partial → FAILED。

- **partial 暴露机制**：kani-compiler 在 codegen 阶段任何失败 → exit ≠ 0
- **形式严格性 — 0 误报（不冤枉能力）**：✅ 形式可证。kani exit 0 ⇔ codegen 完成无错误
- **形式严格性 — 0 漏报（不高估能力）**：⚠️ 形式上**部分可证**——codegen 完成性由 kani-compiler 保证，但 stderr 中 `Found the following unsupported constructs: caller_location / foreign function ...` 是 warning（仅对 SAT 求解阶段有意义），不影响 codegen 完成 / exit 码。实测 corpus 上 SUCCESS entry 无此 warning，但理论上其他 corpus 可能触发"warning + codegen 完成"的边角情形（这种情况下 kani 实际上仍接受了源码进入 codegen，只是 SAT 阶段会警告）
- **漏报盲点**：codegen 完成 + warning 但 SAT 阶段才会触发问题的 entry（实测未观察到）

## 安装

上游：<https://github.com/model-checking/kani>（官方文档：<https://model-checking.github.io/kani/>）。

本测试基线：跟随 `kani-verifier` 当前可用版本（runner 直接调用 PATH 中的 `cargo kani`）。

按上游文档自行安装；runner 直接调用 PATH 中的 `cargo kani`，无需为本工具配置 `TS_*` 变量。本项目不提供安装脚本或步骤教程，避免在工具版本变迁后误导。

## 本框架配置

参见 `tool.toml`。关键参数：

- `--only-codegen`：让 kani 跑 MIR → GotoC codegen 及类型 / 模型检查，**不 invoke CBMC 求解**。本框架测的是"前端支持性"（工具能否理解某 Rust 特性），而非验证结果。让 kani 跑完整 SAT 求解会使对比失公——其他工具（cargo-check / charon / hax 等）都在前端层停下，kani 却因求解器在集合/并发/递归枚举等场景下超时而频繁 FAILED，制造假阴性。
- `timeout_secs = 120`：仅作 codegen 内部失控的兜底，实际 codegen 通常在数秒内完成。
- harness 形态：`#[kani::proof] fn ts_proof() { <crate>::<entry>(); }` + 空 `fn main() {}`——kani 要求入口带 `#[kani::proof]` 标注，bin target 还需要 `fn main`。

## 已知限制 / 坑

- `--only-codegen` 下报 FAILED 表示 kani 在 codegen / 类型建模阶段就无法处理该特性，是强信号（工具前端不支持）。
- macOS arm64 平台下 kani 官方提供预编译 CBMC，通常正常工作；若 `setup` 失败可检查网络或手动指定 CBMC 路径。
- 内联汇编、SIMD、部分 FFI、弱内存模型等特性在 codegen 层可能失败，见 `examples/kani-limit/`。

## 关联 sub-tests

`examples/kani-limit/` 是 Kani 自声明的限制集。这些 entry 故意触发 Kani 的已知"不支持"特性（如 `inline-assembly`、`async-await`、`float-overapprox`、`thread-interleaving-partial` 等），期望 Kani 在这些 entry 上 FAILED。
