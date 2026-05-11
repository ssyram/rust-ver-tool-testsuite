# creusot readme Review

## §1 问题意识

creusot README 重点：(1) MIR → coma 翻译与 why3 SMT 求解切线；(2) extra_cargo_deps + entry_mode = "lib" 双机制（宪法 §四-A 位阶澄清落地）；(3) `cargo-creusot` 默认无 subcommand 即只翻译的天然切线；(4) creusot-std 标准库模型化覆盖边界。

## §2 审查方法

参照源：

- [`docs/design/tool-integration.md`](../../../design/tool-integration.md) §五 / §六；
- [`tools/creusot/tool.toml`](../../../../tools/creusot/tool.toml)；
- [`tools/creusot/harness.rs.tera`](../../../../tools/creusot/harness.rs.tera)；
- [`deep-reports/cc-reports/creusot.md`](../../../../deep-reports/cc-reports/creusot.md)。

## §3 审查现象

#1 (严重度: 低) — README 8 章节齐全

**现象**：[`tools/creusot/README.md`](../../../../tools/creusot/README.md) 完整。

**违反**：未违反。

**决策性**：非决策点。

**建议**：无须改。

#2 (严重度: 中) — README L24 "**可补强检测**"提到 "检查 `verif/*_rlib/` 存在 + 至少一个非 0 字节 `.coma` 文件 + 入口 fn 的 `let rec` 在 coma 里能 grep 到（runner 当前仅看 exit code，已足够）"

**现象**：[`tools/creusot/README.md`](../../../../tools/creusot/README.md) L24：

> **可补强检测**：检查 `verif/*_rlib/` 存在 + 至少一个非 0 字节 `.coma` 文件 + 入口 fn 的 `let rec` 在 coma 里能 grep 到（runner 当前仅看 exit code，已足够）

**违反**：未违反——诚实声明 oracle 当前覆盖范围 + 可补强方向。与 rocq-of-rust / hax-coq 的 entry_fn 存在性 grep 同思路。

**推理链**：creusot 形式严格性 ✅ 形式可证（crash_and_error / span_err / span_fatal 单一通路），所以补强检测是 belt-and-braces 而非必要。

**决策性**：决策点——是否实施补强。

**建议**：可选——加 wrapper 实施 entry_fn `let rec` grep 作为 belt-and-braces。当前 oracle 已足够。

#3 (严重度: 低) — README L25 "**真实失败常见来源**：creusot-std 不模型化某些 std 类型（如 `std::str::Split` / 部分 iterator）导致 rustc-creusot 在 type check 阶段拒收——这是 creusot 自身的 std-coverage 边界"

**现象**：[`tools/creusot/README.md`](../../../../tools/creusot/README.md) L25：

> **真实失败常见来源**：creusot-std 不模型化某些 std 类型（如 `std::str::Split` / 部分 iterator）导致 rustc-creusot 在 type check 阶段拒收——这是 creusot 自身的 std-coverage 边界

**违反**：[`tool-integration.md`](../../../design/tool-integration.md) §六禁忌——"creusot-std 不模型化某些 std 类型" 是描述 creusot 上游限制。"这是 creusot 自身的 std-coverage 边界" 是合理边界声明，不算绝对评判——锚定 creusot 0.11.0 std-coverage 状态。

**推理链**：合规——是工具自身限制陈述。

**决策性**：非决策点。

**建议**：无须改。

#4 (严重度: 低) — README L29-34 形式严格性 ✅ 形式可证扎实

**现象**：[`tools/creusot/README.md`](../../../../tools/creusot/README.md) L29-34：

> - **partial 暴露机制**：creusot 用 `crash_and_error / span_err / span_fatal / dcx().span_err` 把任何 unsupported 升级为 rustc error → exit 101
> - **0 误报**：✅ 形式可证。
> - **0 漏报**：✅ 形式可证。creusot 用 `crash_and_error / span_err / span_fatal` 把所有 unsupported 升级为 rustc error，无 silent path
> - **漏报盲点**：无

**违反**：未违反——按 [`tool-integration.md`](../../../design/tool-integration.md) §四-4.1 形式证明。

**决策性**：非决策点。

**建议**：无须改。

#5 (严重度: 低) — README L40-42 安装段 cargo-creusot 0.11.0 + Why3 + Alt-Ergo / CVC5 完整

**现象**：[`tools/creusot/README.md`](../../../../tools/creusot/README.md) L40-42：

> 本测试基线：`cargo-creusot` 0.11.0（搭配 OCaml 5.3 + Why3 + Alt-Ergo / CVC5 等 SMT solver）。

**违反**：未违反——按 [`tool-integration.md`](../../../design/tool-integration.md) §一 版本锁定。

**决策性**：非决策点。

**建议**：无须改。

#6 (严重度: 低) — README L52 "entry_mode = "lib" .. runner 将原 src/lib.rs 改名为 src/__ts_inner.rs..." 完整解释机制

**现象**：[`tools/creusot/README.md`](../../../../tools/creusot/README.md) L52-53：

> - **entry_mode**：`"lib"`——cargo-creusot 对整个 cargo project 每个 crate target 跑 creusot-rustc，要求顶级 lib 自身含 `use creusot_std::prelude::*;`；runner 将原 `src/lib.rs` 改名为 `src/__ts_inner.rs`，harness（含 `extern crate creusot_std; use creusot_std::prelude::*;`）写为新 `src/lib.rs` 顶级 lib
> - **extra_cargo_deps**：`['creusot-std = "0.11.0"']`——cargo-creusot 入口处硬检查 manifest 中必须列出 `creusot-std`；runner 在隔离副本的 `Cargo.toml` 的 `[dependencies]` 表中注入此行

**违反**：未违反——清晰解释宪法 §四-A 位阶澄清落地的两机制。

**决策性**：非决策点——优秀范例。

**建议**：无须改。

#7 (严重度: 低) — README L55-60 已知限制 / 坑 cargo-creusot 解析阶段 + Why3 进程组管理 + creusot-std 标准库限制

**现象**：[`tools/creusot/README.md`](../../../../tools/creusot/README.md) L55-60。

**违反**：未违反。

**决策性**：非决策点。

**建议**：无须改。

## §4 决策点 vs 非决策点

- 决策点：1（是否实施可补强检测）
- 非决策点：6

## §5 审查结论

creusot README 内容详尽，是宪法 §四-A 位阶澄清的关键文档：

- L52-53 详细解释 extra_cargo_deps + entry_mode = "lib" 两机制；
- 形式严格性 ✅ 形式可证扎实（crash_and_error / span_err / span_fatal 单一通路）；
- 真实失败来源诚实归因（creusot-std 不模型化某些 std 类型）。

无 critical 问题；整体属于"宪法 A 位阶澄清接入典范"。
