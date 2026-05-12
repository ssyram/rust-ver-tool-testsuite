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

为了严格反映前端特性支持范围（不允许 partial），**SUCCESS = `cargo kani --only-codegen` exit 0 且 stdout 不含 5 个 hard-unsupported MIR construct markers**。任何 partial → FAILED。

调用路径：runner 调 `kani-strict-wrapper.sh`（非直接调 cargo kani），wrapper 跑 `cargo kani --only-codegen --bin __ts_harness`，exit 0 后 grep stdout 中 `Found the following unsupported constructs:` 警告 list 的 5 个 marker（[`kani-strict-wrapper.sh`](kani-strict-wrapper.sh) 第 50-100 行）：

| Marker | 含义 |
| --- | --- |
| `TerminatorKind::InlineAsm` | inline asm MIR terminator —— kani 无 goto-cc 语义，emit stub |
| `simd_cast` | packed-SIMD cast intrinsic —— stub |
| `catch_unwind` | panic recovery —— kani unwind model 为 stub |
| `ptr_mask` | raw-pointer bit-mask intrinsic —— stub |
| `C string literal` | `c"..."` raw cstr 字面量 MIR rvalue —— Rust 2024 stable feature kani-compiler 尚未 lower |

命中后 wrapper **进一步按 P37 §六 当前 crate 焦点宽度过滤**（principles.md §六）：

- entry crate `src/` 自含 markers 的触发关键字（`asm!` / `global_asm!` / `simd_*` / `catch_unwind` / `catch_panic` / `::mask(` / `ptr::mask` / `pointer::mask` / `c"` / `cr"` / `CStr::from_bytes_with_nul`）→ entry 自陈不支持 → wrapper 重写 exit 0 为 exit 2 → 框架记 FAILED
- entry crate `src/` 全无触发关键字 → markers 必来自 deps (std / cargo registry / vendor) → 按 §六 宽度切割豁免 → wrapper 保持 exit 0 → 框架记 SUCCESS

实施在 wrapper 内嵌 Python 段 `os.walk('src/')` 加正则匹配。

**特意排除 `caller_location` 与 `foreign function`** —— 它们由 std panic 路径 / std alloc 路径在几乎所有非 trivial entry 上触发（实测 60-63/144 SUCCESS 含此 warning），抓它们会大规模假阳性。这两个 warning 是 kani 对 std 内部的标准处理，不是用户代码的 partial codegen 信号。

详见：[`docs/fixes/oracle-leak-audit-2-2026-05-11.md`](../../docs/fixes/oracle-leak-audit-2-2026-05-11.md) §3.1，落地记录 [`docs/fixes/oracle-leak-rules-implementation-2-2026-05-11.md`](../../docs/fixes/oracle-leak-rules-implementation-2-2026-05-11.md) §2.1。

- **partial 暴露机制**：kani-compiler codegen 阶段任何 hard 失败 → exit ≠ 0；soft 失败（emit stub + warning）→ wrapper 抓 5 markers + entry-src 关键字反向证明后才重写 exit 2
- **形式严格性 — 0 误报（不冤枉能力）**：✅ 实测 + wrapper 双通路区分。markers fired 且 entry src 含关键字 → FAILED（entry 自陈不支持）；markers fired 但 entry src 无关键字 → SUCCESS（markers 来自 deps，§六 豁免）
- **形式严格性 — 0 漏报（不高估能力）**：✅ 实测 + 源码层 + 关键字反向证明。5 markers 是 kani 自陈"我没把这条干完"的明确字面；entry-src 关键字反向证明锚定 partial 来源在 entry 自身代码
- **漏报盲点**：
  - **关键字反向证明的边界**：若 entry src 在注释 / 字符串内含 marker 关键字会误判 FAILED（保守 false positive，非漏报方向）；entry 通过 macro / build script 间接引入 markers 而 src 文本不含关键字 → 误判 SUCCESS（理论漏报，未实测命中）
  - `caller_location` 与 `foreign function` 在 kani 上仍 codegen 为 stub 但 oracle 不抓（避高频假阳性）；kani 未来新增 unsupported MIR 节点类别（hax-engine / kani-compiler 演进可能引入新 stub 路径，需要扩展 5 markers list）
  - **concurrency 单线程语义**（D3.4 / 2026-05-12 补完）：8 个 v5 SUCCESS entries 含 kani `"Kani currently does not support concurrency. The following constructs will be treated as sequential operations"` warning（atomic_* / thread_local / fence）。kani-compiler **真 codegen 原子操作**（atomic_block / SKIP / binop），不是 stub —— 这是 BMC 单线程语义约束（不模拟多线程交错），属求解层假设而非前端 partial。按宪法 §六-3 前端测量原则**不抓 marker**，这些 entries 保持 SUCCESS。该 warning 表征求解层简化口径，不属漏报盲点；列出以诚实声明。

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
