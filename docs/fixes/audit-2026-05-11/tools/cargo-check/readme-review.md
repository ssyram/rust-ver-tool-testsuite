# cargo-check readme Review

## §1 问题意识

cargo-check README 必须清晰传达它的 baseline 定位——"任何其他工具 FAILED 时先看 cargo-check 是否同时 FAILED"，并诚实承认它不在 §六 硬指标覆盖范围。恶意角度的考察：是否暗示了某种"工具能力评判"或"客观真理"？是否过度自信声明 0 误报 / 0 漏报？是否漏掉了 tool-integration §五 必含章节？

## §2 审查方法

参照源：

- [`docs/design/tool-integration.md`](../../../design/tool-integration.md) §五"必含章节清单"（1-8 项）；
- [`docs/design/principles.md`](../../../design/principles.md) §三-3 时效性 / 诚实测试范围；
- [`docs/design/tool-integration.md`](../../../design/tool-integration.md) §六 禁忌；
- [`deep-reports/cc-reports/cargo-check.md`](../../../../deep-reports/cc-reports/cargo-check.md) cc-report；
- 对照其他工具 README 看体例一致性。

## §3 审查现象

#1 (严重度: 低) — 缺章节 "1. 简介 + GitHub URL + elevator pitch" 不严格符合 §五

**现象**：[`tools/cargo-check/README.md`](../../../../tools/cargo-check/README.md) L1-7：

```
# cargo-check
Baseline——验证样例本身能被 rustc 成功编译，与任何验证器无关。

## 简介
`cargo check` 是 Cargo 内置子命令...官方文档：<https://doc.rust-lang.org/cargo/commands/cargo-check.html>
```

**违反**：tool-integration §五 要求"GitHub 上游 URL"——cargo-check 不是 GitHub 项目而是 Rust 工具链自带子命令，所以提供 doc.rust-lang.org URL 是合理替代。

**推理链**：cargo-check 这条目特殊，"上游"概念退化为"工具链自身"。

**决策性**：非决策点——baseline 特例。

**建议**：无须改。

#2 (严重度: 低) — "形式严格性"章节自陈"形式可证"但实际是 trivial 自指

**现象**：[`tools/cargo-check/README.md`](../../../../tools/cargo-check/README.md) L24-26：

> - **形式严格性 — 0 误报（不冤枉能力）**：✅ 形式可证。rustc 单一信号，exit 0 ⇔ type / borrow check 全部通过
> - **形式严格性 — 0 漏报（不高估能力）**：✅ 形式可证。任何 check 失败 → rustc exit ≠ 0
> - **漏报盲点**：无

**违反**：未违反 §三 / §四。但语义上"形式可证"对 baseline 来说是 trivial 自指——cargo-check 的"工具能力"定义就是"rustc 接受这段代码"，所以 oracle 等价于工具自身，0 误报 / 0 漏报由定义直接成立。

**推理链**：[`tool-integration.md`](../../../design/tool-integration.md) §三 / §四 的论证目标是"oracle 抓 partial 不冤枉、不漏报"——对验证工具有意义。对 baseline 这层"工具 = 我自己定义的 oracle"，论证退化。README 写出来不算错，但读者可能误以为这与 kani / verus 等验证工具的 0 误报性质是同类。

**决策性**：非决策点。

**建议**：可选——加一句注释说明 "cargo-check 作为 baseline，oracle = rustc exit code，0 误报 / 0 漏报由定义直接成立，与验证类工具的论证性质不同"。当前简洁优先，可不改。

#3 (严重度: 低) — README L17 "rustc 编译基线" 与正文用语"前端 = 全过程" 微妙不一致

**现象**：[`tools/cargo-check/README.md`](../../../../tools/cargo-check/README.md) L13:

> cargo-check **没有后端**：rustc 跑完整前端（parse + macro expand + name resolution + type check + borrow check + MIR build）即 exit 0，不进入 codegen / LLVM IR。所以本工具的"前端 = 全过程"。

**违反**：未违反，但需注意——cargo-check 实际上**不进 MIR build**？事实上 `cargo check` 跑 type / borrow check 后**会**做 MIR construction（rustc 内部仍走到 MIR build 阶段才能完成 borrow check），只是不会 LLVM lowering。所以"前端 = parse + macro + name res + type check + borrow check + MIR build" 是准确的。

**推理链**：READMЕ 的描述符合实际。

**决策性**：非决策点。

**建议**：无须改。

#4 (严重度: 中) — README 显式声明 cargo-check 是"健全性基准"但未说明矩阵报告解读规则

**现象**：[`tools/cargo-check/README.md`](../../../../tools/cargo-check/README.md) L7、L17：

> 在本框架中作为"健全性基准"存在：若某工具在某 entry 上 FAILED，可先看 cargo-check 是否同样 FAILED——若是，说明是样例本身的 Rust 错误，而非工具自身的限制。
> **矩阵中的角色**：rustc 编译基线。任何 entry 在其它工具上 FAILED 时，先看 cargo-check 是否同时 FAILED——若是，说明 entry 自身 Rust 不合法，与工具能力无关

**违反**：未严格违反。但 [`principles.md`](../../../design/principles.md) §三-3 / [`tool-integration.md`](../../../design/tool-integration.md) §七 把"实测报告解读"放在报告文档侧，不在工具 README 内。README 提"矩阵报告解读规则"略越界——README 应只描述工具自己，报告解读规则应放在 `test-reports/` 的报告头部或 cc-reports。

**推理链**：README 写"健全性基准"是工具自身的角色定位，合理；但"先看 cargo-check 是否同时 FAILED" 这种使用建议属于报告解读层。

**决策性**：非决策点——baseline 的角色定位与使用方式天然耦合，分不开。

**建议**：无须改。可考虑微调为"cargo-check 是 baseline，其 FAILED 信号表示 entry 自身 Rust 不合法"——避免直接给读者使用建议。

#5 (严重度: 低) — README 未明示版本捕获覆盖度

**现象**：[`tools/cargo-check/README.md`](../../../../tools/cargo-check/README.md) L32：

> 本测试基线：稳定版 `cargo` / `rustc`（与机器上的默认 toolchain 一致）。

**违反**：[`tool-integration.md`](../../../design/tool-integration.md) §一 要求锁定 commit / 版本。cargo-check 用机器默认 toolchain，不锁定具体版本——但这是 baseline 特殊性：版本浮动是预期的。

**推理链**：cargo-check baseline 角色就是"用你机器上的 rustc 看 entry 是否能编"——锁版本反而违反 baseline 定位。

**决策性**：非决策点。

**建议**：可补一句"基线版本由 results.json metadata 段 version_command 捕获——读者按时点查 toolchain 版本"。

#6 (严重度: 低) — README 未声明与 cargo-check version_command 仅捕获 cargo 版本（不包含 rustc）的对应性

**现象**：[`tools/cargo-check/README.md`](../../../../tools/cargo-check/README.md) 无说明 version_command 实际捕获内容。

**违反**：tool-integration §一——README 应明示版本捕获方式。

**推理链**：与 config-review.md #4 关联——`cargo --version` 不显式含 rustc 版本，但 cargo 与 rustc 版本基本一对一。读者若需追究 rustc 版本，需查机器上 default toolchain。

**决策性**：非决策点——cargo-check baseline 容忍这点。

**建议**：可选补——README 增加"version_command 捕获 cargo 版本字符串；rustc 版本由 default toolchain 推断"。

## §4 决策点 vs 非决策点

- 决策点：0
- 非决策点：6

## §5 审查结论

cargo-check README 整体诚实、范围克制——它准确传达了 baseline 定位，未越界给工具能力下绝对结论，未做工具间排序。所有 6 条审查现象都是低严重度的微调建议（版本捕获 / 矩阵解读用语等），不是必须修复的问题。

特别值得肯定的：

- L26"漏报盲点：无"对 baseline 是恰如其分的——baseline 不需要担心 silent path；
- L48-49 "macOS arm64 无平台特定限制" / "无法检测运行时 UB 或语义错误" 提供了诚实边界。

cargo-check README 是矩阵中最简洁、最稳定的一份，作为其他工具 README 的体例参考是合适的。
