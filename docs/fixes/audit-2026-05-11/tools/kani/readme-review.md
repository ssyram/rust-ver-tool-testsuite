# kani readme Review

## §1 问题意识

kani README 要清晰传达：(1) 前端 / 后端切线（MIR → GotoC vs CBMC SAT/SMT）；(2) `--only-codegen` 的反作弊设计（防 kani 用 stub silent path 假 SUCCESS）；(3) 5-marker wrapper 的论据；(4) `caller_location` / `foreign function` 排除的诚实声明。

恶意角度考察：
1. 是否给 kani 能力下绝对结论？
2. 是否做工具间排序（如"kani 比 verus 好"）？
3. 形式严格性自陈是否与 wrapper 实施一致？
4. 漏报盲点是否完整？

## §2 审查方法

参照源：

- [`docs/design/tool-integration.md`](../../../design/tool-integration.md) §五 必含章节、§六 禁忌；
- [`tools/kani/kani-strict-wrapper.sh`](../../../../tools/kani/kani-strict-wrapper.sh) 对照实施；
- [`deep-reports/cc-reports/kani.md`](../../../../deep-reports/cc-reports/kani.md) cc-report；
- [`docs/fixes/oracle-leak-audit-2-2026-05-11.md`](../../oracle-leak-audit-2-2026-05-11.md) §3.1。

## §3 审查现象

#1 (严重度: 低) — README L20-23 "矩阵中的角色" 与"公平性"段提"让 kani 跑完整 CBMC 求解会失公"——略带评判语气

**现象**：[`tools/kani/README.md`](../../../../tools/kani/README.md) L23：

> **公平性**：让 kani 跑完整 CBMC 求解会失公——其他工具（cargo-check / charon / hax / creusot 等）都在前端层停下，kani 却因求解器在集合/并发/递归枚举等场景下超时而频繁 FAILED，制造假阴性

**违反**：[`tool-integration.md`](../../../design/tool-integration.md) §六禁忌——"跨工具能力排序（如'X 比 Y 更好'）—— 框架不评比，只测 entry 级二值信号"。这里讨论"公平"是说明本方法学的对称性，不是评比，但措辞有公平 vs 不公平的对立意味。

**推理链**：这段实际是论证为什么 kani 用 `--only-codegen`——方法学选择，按 [`principles.md`](../../../design/principles.md) §六-1 前端切割原则。措辞"失公"指方法学不对称，不是工具能力评比。

**决策性**：非决策点。

**建议**：可选改"失公"为"方法学不对称"——更精确。但当前措辞读者大致能理解。

#2 (严重度: 中) — README L31-37 5-marker 表格清晰，但与 wrapper 双向论证证据不完整

**现象**：[`tools/kani/README.md`](../../../../tools/kani/README.md) L31-37 列出 5 marker 表格，L41 提"`caller_location` / `foreign function` 由 std panic 路径 / std alloc 路径在几乎所有非 trivial entry 上触发（实测 60-63/144 SUCCESS 含此 warning）"——只提反向（如果加入会假阳性）。

**违反**：[`tool-integration.md`](../../../design/tool-integration.md) §四-4.2 "双向实测：已知 silent path → grep 命中（防漏报有效）；合法代码 / 合法注释 / prelude / 用户合法字面 → grep 不命中（不引入误报）"。kani README 缺"合法 SUCCESS → grep 不命中"的明示双向证据。

**推理链**：wrapper L65-72 列出 reverse-false-positive analysis 的 4 个合法 SUCCESS entry——但 README 只在 L46 一句话写"hello/basic-hello / bigint-arith / industrial/rsa-pkcs8 / industrial/sha256-digest 实测均 SUCCESS"。这条声明扎实但与 wrapper 的详细论证（L65-72）对应不够直白。

**决策性**：决策点——是否在 README 显式展开 4 个合法 SUCCESS 的 grep 不命中证据。

**建议**：可补 1 表格——4 个合法 SUCCESS entry × 5 marker，每格说明 grep 不命中。当前已经满足 §四-4.2，但展开更扎实。

#3 (严重度: 中) — README L46-48 "0 误报 / 0 漏报：形式可证" 论断与 wrapper grep heuristics 性质有张力

**现象**：[`tools/kani/README.md`](../../../../tools/kani/README.md) L46-48：

> - **形式严格性 — 0 误报（不冤枉能力）**：✅ 形式可证。任意合法 SUCCESS（不触发 5 markers）在 wrapper 下保持 SUCCESS（hello/basic-hello / bigint-arith / industrial/rsa-pkcs8 / industrial/sha256-digest 实测均 SUCCESS）
> - **形式严格性 — 0 漏报（不高估能力）**：✅ 实测 + 源码层封堵。5 markers 是 kani 自陈"我没把这条干完"的明确字面，与"工具接受"互斥。

**违反**：[`tool-integration.md`](../../../design/tool-integration.md) §四-4.3 "防漏报机制的意义辨析"——理论意义有限：grep-based 机制**无法形式证明 0 漏报**。kani README 说"形式可证"但实际是实测验证（hello/basic-hello 等 4 个 entry），不是真形式证明。

**推理链**：5-marker grep 是启发式：只能抓**已知** silent path。若 kani 上游引入第 6 类 hard-unsupported marker，grep 不抓——是漏报。所以严格按 §四-4.3，"形式可证" 应改为 "实测验证 + 实测 0 误报 0 漏报 + 漏报盲点声明在 L48"。

L46 写"形式可证"有过度自信之嫌。但 L47"实测 + 源码层封堵" 是更准确表述——同一段两种措辞混用。

**决策性**：决策点——是否在 0 误报 / 0 漏报段一致用"实测验证 / 不可形式证明"。

**建议**：把 L46-47 改为：
- 0 误报：⚠️ 实测验证 0 误报（hello/basic-hello / bigint-arith / industrial/rsa-pkcs8 / industrial/sha256-digest），但**不可形式证明**——5-marker grep 是启发式，理论上未来 kani 输出可能引入新 marker 让 grep 误命中
- 0 漏报：⚠️ 实测 + 源码层封堵已知 silent path。5-marker 是 kani 自陈"我没把这条干完"的明确字面；理论上 kani 上游引入新 silent path 而 grep 滞后

这与 hax-lean / rocq-of-rust README 体例一致（它们用 ⚠️ 标注实测验证）。

#4 (严重度: 中) — README L66-69 已知限制 / 坑 用"工具前端不支持"等绝对结论

**现象**：[`tools/kani/README.md`](../../../../tools/kani/README.md) L68:

> `--only-codegen` 下报 FAILED 表示 kani 在 codegen / 类型建模阶段就无法处理该特性，是强信号（工具前端不支持）。

**违反**：[`tool-integration.md`](../../../design/tool-integration.md) §六禁忌——"对工具能力下绝对结论（如'工具 X 不支持特性 Y'）"。这条措辞接近绝对结论。

**推理链**：按宪法精神应改为"FAILED 表示在本测试方法学下 kani codegen 阶段对该 entry reject"。

**决策性**：非决策点——README 描述工具自身限制时允许在"已知限制 / 坑" 段使用工具自身视角，但更严格按 §六禁忌应避免"工具前端不支持"。

**建议**：可选——改为"FAILED 表示 kani 在 codegen / 类型建模阶段 reject 该 entry，是 kani 自身前端边界的强信号"。

#5 (严重度: 低) — README L52-56 安装段未 pin kani 具体版本

**现象**：[`tools/kani/README.md`](../../../../tools/kani/README.md) L52-56：

> 本测试基线：跟随 `kani-verifier` 当前可用版本（runner 直接调用 PATH 中的 `cargo kani`）。

**违反**：[`tool-integration.md`](../../../design/tool-integration.md) §一 "锁定 commit / 版本号 / brew tap / nightly toolchain pin"。kani 版本未 pin。

**推理链**：kani-verifier 在 brew / cargo install 上有具体版本号（如 0.55.0）。当前 README 让用户用"当前可用版本"，浮动。同 miri 时效性——results.json metadata 段捕获。

**决策性**：非决策点——次要模块时效性约定。

**建议**：可选——补"具体 kani 版本由 `results.json` metadata 段 `cargo kani --version` 捕获"。

#6 (严重度: 低) — README L48 漏报盲点说明完整

**现象**：[`tools/kani/README.md`](../../../../tools/kani/README.md) L48：

> - **漏报盲点**：`caller_location` 与 `foreign function` 在 kani 上仍 codegen 为 stub 但 oracle 不抓（避高频假阳性）；kani 未来新增 unsupported MIR 节点类别（hax-engine / kani-compiler 演进可能引入新 stub 路径，需要扩展 5 markers list）

**违反**：未违反——按 §四-4.4 诚实声明已知漏报盲点。

**推理链**：声明已到位。

**决策性**：非决策点。

**建议**：无须改。

#7 (严重度: 低) — cc-report 与 README 对照一致

**现象**：[`deep-reports/cc-reports/kani.md`](../../../../deep-reports/cc-reports/kani.md)（须读才能确认；这里仅记录需要交叉验证）。

**违反**：待确认。

**决策性**：非决策点。

**建议**：人工 spot-check。

## §4 决策点 vs 非决策点

- 决策点：3（双向证据展开 / 0 误报 0 漏报措辞 / 0 误报"形式可证"vs"实测验证"）
- 非决策点：4

## §5 审查结论

kani README 内容总体完整，按 §五 8 章节都在。最值得修正：

1. **L46-47 措辞**："形式可证" 应改为 "⚠️ 实测验证（不可形式证明）" ——与 hax-lean / rocq-of-rust 体例一致，避免对 grep-based 启发式过度自信；
2. **L68 措辞**："工具前端不支持" 改为 "kani codegen 阶段 reject"——避免 §六禁忌绝对结论；
3. **L46 双向证据**：合法 SUCCESS × 5 marker 表格展开——使 §四-4.2 双向论证更直观。

整体 oracle 设计经 P12 / audit-2 收紧后稳健，README 主体准确。
