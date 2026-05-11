# miri readme Review

## §1 问题意识

miri 是矩阵中的"前端 = 全过程"解释执行类工具。README 要清晰阐述：(1) "前端 / 后端" 切割对解释执行类不适用（"前端 = 全过程"）；(2) UB / unsupported / panic 三类中断都 FAILED——这与"工具有效输出"的工具视角主张不冲突，按宪法精神所有中断都是没完整跑完；(3) silent path / 漏报盲点状态。

恶意角度考察：
1. README 是否对 miri 能力下绝对结论？
2. README 是否暗示"UB detection 表示该工具更强"等评判？
3. README 是否真按 tool-integration §五 必含 8 章节？
4. README 与 cc-report 是否互相矛盾？
5. 形式严格性自陈（0 误报 / 0 漏报形式可证）与 wrapper / 实际实施是否一致？

## §2 审查方法

参照源：

- [`docs/design/tool-integration.md`](../../../design/tool-integration.md) §五 必含章节；
- [`docs/design/tool-integration.md`](../../../design/tool-integration.md) §六 禁忌；
- [`docs/design/principles.md`](../../../design/principles.md) §六-2 不允许 partial；
- [`deep-reports/cc-reports/miri.md`](../../../../deep-reports/cc-reports/miri.md) cc-report；
- 与 [`tools/miri/tool.toml`](../../../../tools/miri/tool.toml) 对照配置一致性。

## §3 审查现象

#1 (严重度: 高) — README L33-34 注释把"UB 检测 = 工具有效输出"的视角偷渡入正文，使读者可能误解 oracle 设计

**现象**：[`tools/miri/README.md`](../../../../tools/miri/README.md) L20、L33：

> 注：UB 检测在某些视角下可理解为"工具有效输出"，但按宪法精神（不允许 partial = 必须完整跑完）一律 FAILED——解释执行被中断 = 没完整跑完。
> ...
> 注：UB 检测在某些视角下被理解为"工具有效输出"——但按"不允许 partial / 完整完成"精神，本测试一律 FAILED（解释执行没完整跑完）。

**违反**：未严格违反。但这两段重复且关键 ——按 [`principles.md`](../../../design/principles.md) §三-3 "诚实测试范围"，README 应明确告知读者本测试的口径与"工具有效输出"的视角不同。当前的写法在 oracle 段（L20）与形式严格性段（L33）两处都出现——重复说明是好的，避免读者跳读漏掉。

**推理链**：这一注释是关键边界声明。但 L33 出现在"形式严格性"段——把它从"补充注"位置移到"前端接受定义"主段会更清晰。

**决策性**：非决策点。

**建议**：保留重复，作为关键诚实边界声明。

#2 (严重度: 中) — README 完全没提及"-Zmiri-disable-isolation 默认未启用"这条方法学选择

**现象**：[`tools/miri/README.md`](../../../../tools/miri/README.md) L43-49 "本框架配置"段：

```
- `command = ["cargo", "+nightly", "miri", "run", "--bin", "__ts_harness"]`：显式指定 `+nightly` toolchain，因为 miri 只在 nightly 上可用。用 `run` 而非 `check`——miri 必须实际执行程序才能检测 UB，仅类型检查无意义。
- `timeout_secs = 300`：miri 解释执行速度远低于原生执行...
```

**违反**：[`tool-integration.md`](../../../design/tool-integration.md) §五 (6) "本框架配置 — `tool.toml` 关键参数"。README 未提 isolation default 选择——但这是关键 oracle 决策：isolation 开启时，miri 在 OS 调用上 exit ≠ 0 → FAILED；若开启 `-Zmiri-disable-isolation`，OS 调用 silent succeed，oracle 误判 SUCCESS。

**推理链**：与 config-review.md #1 关联。README 漏写关键 oracle 设计——读者不知道为什么 `examples/miri-limit/networking-unsupported` 预期 FAILED 是因为 isolation default 而非 miri 不支持 socket。

**决策性**：决策点——是否在 README 显式声明 isolation 选择。

**建议**：在 L46 "用 `run` 而非 `check`" 后补一句："不显式启用 `-Zmiri-disable-isolation`——保持 miri default isolation，让 OS 系统调用走 exit ≠ 0 → FAILED 路径，与宪法 §六-2 不允许 partial 对齐。"

#3 (严重度: 中) — README L54 提到 examples/miri-limit 的预期 FAILED 列表与实际 cc-report 是否一致未交叉验证

**现象**：[`tools/miri/README.md`](../../../../tools/miri/README.md) L60：

> 这些 entry 故意触发 MIRI 的已知"不支持"特性（如 `inline-asm`、`ffi-unshimmed-extern`、`networking-unsupported`、`simd-bitmask-large-vector`、`uninit-memory` 等），期望 MIRI 在这些 entry 上 FAILED。

**违反**：未严格违反。[`tool-integration.md`](../../../design/tool-integration.md) §八禁忌中"禁止对工具能力下绝对结论"——这里写"期望 FAILED"是 corpus 设计意图，不是工具能力评判。措辞合规。

**推理链**：列举的 entry 名都是 corpus 中已存在的 miri-limit 子目录命名，与 corpus 设计一致。

**决策性**：非决策点。

**建议**：无须改。

#4 (严重度: 低) — README L29-32 "形式严格性 — 0 误报 / 0 漏报：形式可证" 论证略简

**现象**：[`tools/miri/README.md`](../../../../tools/miri/README.md) L29-31：

> - **形式严格性 — 0 误报（不冤枉能力）**：✅ 形式可证。miri exit 0 ⇔ 解释执行完整跑完且无 UB / unsupported operation
> - **形式严格性 — 0 漏报（不高估能力）**：✅ 形式可证。任何 UB / unsupported / panic 触发 → miri exit ≠ 0

**违反**：[`tool-integration.md`](../../../design/tool-integration.md) §三 / §四 要求"论证形式不限定——任何能提供足够证据的方式都可"。当前 README 给的是结论但论证浅——"miri exit 0 ⇔ 完整跑完" 缺少对 miri 内部源码路径的引用（aeneas 给了 `Main.ml:773`，charon 给了 `--abort-on-error`，kani 给了 wrapper 行号——miri 给的是直觉性陈述）。

**推理链**：miri 源码层论证应是"`throw_unsupported_format!` / `throw_ub_format!` 是 miri 内部唯一的 reject 通路，所有抛出最终经 `process::exit(N)` 暴露"——这条论证可在 README 加 5-10 行。

**决策性**：决策点——是否补充源码引用。

**建议**：可选——补 1 行 "miri 内部所有 UB / unsupported 通过 `InterpError` 单一通路通过 `process::exit(1)` 暴露；不存在 silent emit-stub 或 silent skip 路径"。这条声明使形式严格性论证更扎实。

#5 (严重度: 低) — README L38 锁定"跟随当前 nightly toolchain 中 miri component 的可用版本"——浮动版本

**现象**：[`tools/miri/README.md`](../../../../tools/miri/README.md) L39：

> 本测试基线：跟随当前 `nightly` toolchain 中 `miri` component 的可用版本。

**违反**：[`tool-integration.md`](../../../design/tool-integration.md) §一 "锁定的 commit hash / 版本号 / brew tap / nightly toolchain pin"——miri 不 pin 到具体 nightly 日期。

**推理链**：与 config-review.md #5 关联。miri component 跟随 nightly toolchain，版本浮动是预期。但 README 应补一句"读者引用某 entry 的实测结果时，请查 `results.json` metadata 段的 miri 版本字符串"。

**决策性**：非决策点——按 baseline 模块时效性约定。

**建议**：可选——补 "具体 miri 版本由 `results.json` metadata 段 `cargo +nightly miri --version` 捕获"。

#6 (严重度: 低) — README L52-57 "已知限制 / 坑" 提到 miri 不支持 SIMD / FFI / inline-asm，但未说明"不支持"的原因（缺 shim / 概念上无法解释）

**现象**：[`tools/miri/README.md`](../../../../tools/miri/README.md) L52-55：

> - miri 不支持内联汇编（`asm!`）、部分 FFI 调用（无 shim 的 extern fn）、SIMD 部分指令...
> - 网络 / 系统调用等 OS 接口 miri 默认拒绝执行，会报 UB 或不支持错误。

**违反**：[`tool-integration.md`](../../../design/tool-integration.md) §六 禁忌——"对工具能力下绝对结论"。这里写"miri 不支持 X" 是绝对结论的表述。

**推理链**：按宪法精神，应改为"在本测试方法学下，miri 在 X 类 entry 上 FAILED"——但作为"已知限制 / 坑" 段，描述工具自身限制是 README 的合法用途（按 tool-integration §五 第 7 条 "已知限制 / 坑"）。

**决策性**：非决策点——README 的"已知限制 / 坑" 段允许描述工具自身限制，前提是诚实锚定版本。

**建议**：可选——把 "miri 不支持 inline asm" 改为 "miri 当前版本对 inline asm 无 shim，触发 UB / unsupported"。

#7 (严重度: 低) — cc-report 与 README 完全一致

**现象**：[`deep-reports/cc-reports/miri.md`](../../../../deep-reports/cc-reports/miri.md) 与 [`tools/miri/README.md`](../../../../tools/miri/README.md) 对照检查：

- pipeline 描述一致；
- SUCCESS 信号一致（exit 0）；
- 形式严格性一致（0 误报 / 0 漏报形式可证）；
- 漏报盲点一致（无）。

**违反**：未违反。

**决策性**：非决策点。

**建议**：无须改。这是好的对齐范例。

## §4 决策点 vs 非决策点

- 决策点：2（isolation 选择是否在 README 声明，形式严格性源码论证是否补充）
- 非决策点：5

## §5 审查结论

miri README 内容诚实、结构齐全（按 tool-integration §五 必含 8 章节都在），与 cc-report 高度对齐。

最值得补强：

1. **关键缺失**：README 未提"不启用 `-Zmiri-disable-isolation`" 这条方法学选择背后的考量——读者不能从 README 推断为什么 OS 调用类 entry 预期 FAILED；
2. **次要补强**：形式严格性段可补 1 行 miri 源码层论证（InterpError 单一通路）。

无 critical 矛盾或越界结论。整体属于"健壮稳定的低维护工具集成"，与 miri 自身工具链稳定性匹配。
