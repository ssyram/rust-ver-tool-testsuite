# rocq-of-rust readme Review

## §1 问题意识

rocq-of-rust README 重点：(1) 工具设计上对 unsupported case **仍 exit 0**——oracle 完全靠产物 grep + N-attempt；(2) 6 道门 + N-attempt（默认 7）AND-reduce 设计；(3) 非确定性翻译路径漏报封堵（2026-05-11 P15-impl）；(4) ⚠️ 实测验证 0 误报 / 0 漏报（不可形式证明）；(5) 漏报盲点详细列出。

恶意角度考察：是否对 rocq-of-rust 能力下绝对结论？6 道门 + N-attempt 论据扎实？

## §2 审查方法

参照源：

- [`docs/design/tool-integration.md`](../../../design/tool-integration.md) §五 / §六；
- [`tools/rocq-of-rust/tool.toml`](../../../../tools/rocq-of-rust/tool.toml)；
- [`tools/rocq-of-rust/rocq-of-rust-wrapper.sh`](../../../../tools/rocq-of-rust/rocq-of-rust-wrapper.sh)；
- [`deep-reports/cc-reports/rocq-of-rust.md`](../../../../deep-reports/cc-reports/rocq-of-rust.md)；
- [`docs/fixes/ror-gate6-fix-2026-05-11.md`](../../ror-gate6-fix-2026-05-11.md)。

## §3 审查现象

#1 (严重度: 低) — README 8 章节齐全

**现象**：[`tools/rocq-of-rust/README.md`](../../../../tools/rocq-of-rust/README.md) 完整。

**违反**：未违反。

**决策性**：非决策点。

**建议**：无须改。

#2 (严重度: 中) — README L22-39 "Silent fallback 检测" 段—— 矩阵中最详尽的 silent path 调研

**现象**：[`tools/rocq-of-rust/README.md`](../../../../tools/rocq-of-rust/README.md) L22-39 列出 7 类 silent fallback 路径（THIR panic / TyKind 落空 / extern crate skip / TopLevelItem::Error / ConstKind / lib/src/core.rs:157 单文件丢盘 / 非确定性翻译路径），每类附实证状态。

**违反**：未违反——按 [`tool-integration.md`](../../../design/tool-integration.md) §三-源码层穷尽。**这是矩阵中最详尽的 silent path 调研**。

**推理链**：与 hax-coq / hax-fstar 的 silent path 调研同模式，但 rocq-of-rust 列出更多类（7 类 vs hax-fstar 1 类）——因为 rocq-of-rust 设计上对所有 unsupported case exit 0，silent path 多于上游用 Diagnostic 的 hax。

**决策性**：非决策点——优秀范例。

**建议**：无须改。

#3 (严重度: 中) — README L40-67 SUCCESS 信号详细列出 6 道门 + N-attempt——结构清晰

**现象**：[`tools/rocq-of-rust/README.md`](../../../../tools/rocq-of-rust/README.md) L42-50 列 6 道门：

```
1. exit code = 0
2. 至少一个 .v 产物存在
3. 无 0-byte .v
4. 至少一个 .v > 200 字节
5. 产物不含显式 failure marker grep：...
6. 产物中至少一个 .v 文件含 `^[[:space:]]*Definition[[:space:]]+<TS_ENTRY_FN>[[:space:]]`
```

**违反**：未违反——优秀文档结构。

**决策性**：非决策点。

**建议**：无须改。

#4 (严重度: 中) — README L60-66 形式严格性 ⚠️ 实测验证 0 误报 / 0 漏报 + 漏报盲点诚实

**现象**：[`tools/rocq-of-rust/README.md`](../../../../tools/rocq-of-rust/README.md) L60-66：

> **形式严格性 — 0 误报**：⚠️ 实测验证 0 误报，但**不可形式证明**。oracle 用保守的 marker 集——只抓 rocq-of-rust 自己 emit 的 explicit failure comment 块...
> **形式严格性 — 0 漏报**：⚠️ 实测验证 0 漏报，但**不可形式证明**。rocq-of-rust **设计上不用 exit code 表达 partial**（永远 exit 0，对所有 unsupported 用 rustc warning），所以 oracle 只能靠产物字面 grep + 产物 shape 检测...

**违反**：未违反——按 [`tool-integration.md`](../../../design/tool-integration.md) §四-4.3 防漏报机制的意义辨析。⚠️ 标注准确——grep-based 不能形式证明。

**推理链**：与 hax-lean ⚠️ 同模式（hax-coq / hax-fstar 是 ✅ 因为源码层穷尽 + 实测）。rocq-of-rust 是 ⚠️ 因为依赖 grep + N-attempt（启发式），实测验证为底。**措辞精确**。

**决策性**：非决策点。

**建议**：无须改。

#5 (严重度: 低) — README L64-67 漏报盲点诚实声明完整

**现象**：[`tools/rocq-of-rust/README.md`](../../../../tools/rocq-of-rust/README.md) L64-67：

> - 上游引入新 silent fallback 路径不带已知 markers（实测在 examples corpus 0 现象）
> - 完全 skip item 类（`use` / `extern crate` / `macro_rules!` 在 `top_level.rs:349-390` 直接 `vec![]`）：这些是 rustc 编译时已被处理的 import / macro，**不需要在产物里有 declaration**，所以是合理 skip，**不算漏报**——但若 entry_fn 名误指向这些 item（错误 corpus 设置），门 6 会捕获
> - 新非确定性翻译路径中，N 次 attempt 都恰好采到含 entry_fn 的变体——可通过把 N 增大缓解（`ROCQ_OF_RUST_N_ATTEMPTS` env 已暴露；当前默认 N=7 在 P(drop)=0.4 时 catch rate 99.84%）

**违反**：未违反——按 §四-4.4 诚实声明。

**决策性**：非决策点。

**建议**：无须改。

#6 (严重度: 低) — README L69-75 安装段 commit hash + nightly toolchain 完整

**现象**：[`tools/rocq-of-rust/README.md`](../../../../tools/rocq-of-rust/README.md) L73：

> 本测试基线：commit `a8a76a4d`（cli/，搭配 nightly toolchain `nightly-2024-12-07`）。

**违反**：未违反——按 [`tool-integration.md`](../../../design/tool-integration.md) §一 commit hash 锁定。

**决策性**：非决策点。

**建议**：无须改。

#7 (严重度: 低) — README L91-95 已知限制 / 坑 详细

**现象**：[`tools/rocq-of-rust/README.md`](../../../../tools/rocq-of-rust/README.md) L91-95 列 5 条：静默吞错 / 单文件输入 / toolchain 锁定 / 输出路径结构 / 翻译质量。

**违反**：未违反。

**决策性**：非决策点。

**建议**：无须改。

#8 (严重度: 低) — README L97-101 关联 sub-tests "本工具未派生限制集"

**现象**：[`tools/rocq-of-rust/README.md`](../../../../tools/rocq-of-rust/README.md) L97-101。

**违反**：未违反。

**决策性**：非决策点。

**建议**：无须改。

## §4 决策点 vs 非决策点

- 决策点：0
- 非决策点：8

## §5 审查结论

rocq-of-rust README **优秀**：

- L22-39 Silent fallback 检测段是矩阵中最详尽的 silent path 调研；
- ⚠️ 实测验证措辞准确（与 hax-lean 同体例）；
- 漏报盲点诚实声明 + 数值校准（N=7 → 99.84% catch rate）扎实；
- 与 rocq-of-rust-typecheck（档 1）的关系明确文档化。

无 critical 问题；整体属于"两轮 oracle audit + 反向暴露后的稳态配置"，论证质量极高。
