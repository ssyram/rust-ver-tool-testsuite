# hax-lean readme Review

## §1 问题意识

hax-lean README 是矩阵中 silent path 检测论证最详细的一份——精准 grep 模式实测验证 + 漏报盲点诚实声明 + 上游 PR #1672 升级路径 + lean prelude 防误命中说明。审查重点：(1) 实测论证扎实性；(2) ⚠️ 实测验证 vs ✅ 形式可证 措辞是否准确；(3) 漏报盲点完整。

## §2 审查方法

参照源：

- [`docs/design/tool-integration.md`](../../../design/tool-integration.md) §三 / §四 / §五 / §六；
- [`tools/hax-lean/tool.toml`](../../../../tools/hax-lean/tool.toml) 内联 oracle 实施；
- [`deep-reports/cc-reports/hax-lean.md`](../../../../deep-reports/cc-reports/hax-lean.md)；
- [`docs/design/hax-lean-consistency-design-2026-05-11.md`](../../../design/hax-lean-consistency-design-2026-05-11.md)。

## §3 审查现象

#1 (严重度: 低) — README L21-26 sentinel body 说明 + 宪法精神引用清晰

**现象**：[`tools/hax-lean/README.md`](../../../../tools/hax-lean/README.md) L21-26：

> cargo hax 的 exit 码语义是：**只要任何阶段发了 `FromEngine::Diagnostic` 消息就 exit 1**。但 hax engine 在某些 case 下把不能完整翻译的项替换为 sentinel body（`sorry` / `Inhabited.default`）继续 print 出 declaration——这种情况 exit 1 但产物里函数 declaration 仍在。
> 按宪法 §六-2 不允许 partial...

**违反**：未违反——明确诠释 hax 设计 quirk + 宪法精神。

**推理链**：诚实声明的优秀范例。

**决策性**：非决策点。

**建议**：无须改。

#2 (严重度: 中) — README L38-40 形式严格性 ⚠️ 实测验证 ≠ ✅ 形式可证——措辞准确

**现象**：[`tools/hax-lean/README.md`](../../../../tools/hax-lean/README.md) L38-40：

> **形式严格性 — 0 误报（不冤枉能力）**：⚠️ 实测验证 0 误报，但**不可形式证明**。
> **形式严格性 — 0 漏报（不高估能力）**：⚠️ 实测验证 0 漏报，但**不可形式证明**。

**违反**：未违反——[`tool-integration.md`](../../../design/tool-integration.md) §四-4.3 防漏报机制的意义辨析。grep-based oracle 不能形式证明，README 准确用 ⚠️。

**推理链**：与 kani README L46-47 "形式可证" 形成对比——kani 也是 grep-based wrapper，应同样用 ⚠️ 实测验证。hax-lean 措辞准确。

**决策性**：非决策点。

**建议**：无须改——这是体例正面范例。

#3 (严重度: 低) — README L42-46 漏报盲点诚实声明完整

**现象**：[`tools/hax-lean/README.md`](../../../../tools/hax-lean/README.md) L42-46：

> **漏报盲点**：
> - hax engine 完全 skip item（item 既不写 sorry 也不发 Diagnostic，**不出现在产物里**）—— oracle 抓不到，需要对应性脚本暴露（实测在 examples corpus 0 现象）
> - 上游 PR #1672 合并后 lean.rs 的 sorry path 应消失，届时 grep 自然失效（无害）

**违反**：未违反——按 §四-4.4 诚实声明。两条盲点都准确锚定到 upstream 源码与未来路径。

**推理链**：hax-coq / hax-fstar 已通过 entry_fn 存在性 grep 封堵 "skip item" 盲点；hax-lean 没加这个封堵，README 诚实说"实测在 examples corpus 0 现象"——这是诚实声明 + 实测兜底。

**决策性**：非决策点。

**建议**：可考虑——加 entry_fn 存在性 grep 封堵该盲点（与 hax-coq / hax-fstar 对齐），让 hax 3 backend 漏报盲点状态一致。

#4 (严重度: 低) — README L46 "hax-lean prelude（`hax/proof-libs/lean/`）不复制到 `proofs/lean/extraction/`——grep 只看 extraction 目录不会误命中 prelude"

**现象**：[`tools/hax-lean/README.md`](../../../../tools/hax-lean/README.md) L46：

> 注：hax-lean prelude（`hax/proof-libs/lean/`）不复制到 `proofs/lean/extraction/`——grep 只看 extraction 目录不会误命中 prelude。

**违反**：未违反——清晰说明 grep 路径限制（仅 extraction 目录），防 prelude 中合法 `sorry`（如 `def sorry := axiom`）被误命中。

**决策性**：非决策点。

**建议**：无须改。

#5 (严重度: 低) — README L74 "hax 内部调用 `cargo metadata`；work dir 必须在 `runs/` 下" 提示 P14 fix

**现象**：[`tools/hax-lean/README.md`](../../../../tools/hax-lean/README.md) L74：

> hax 内部调用 `cargo metadata`；work dir 必须在 `runs/` 下（已在主 `Cargo.toml` 的 `exclude` 中排除），否则会触发 CargoMetadata panic

**违反**：未违反——已知 hax 工具坑（cargo metadata 寻 ancestor workspace）的诚实声明。

**推理链**：与 runner copy 隔离机制有关。runner 把副本 copy 到 `runs/<id>/work/<exec_id>/`——该目录必须不在 cargo workspace 内。

**决策性**：非决策点。

**建议**：无须改。

#6 (严重度: 低) — README L51-55 安装段 commit hash + nightly toolchain pin 完整

**现象**：[`tools/hax-lean/README.md`](../../../../tools/hax-lean/README.md) L52：

> 本测试基线：commit `30949eb87058895c24f963df90dd30ef11b0dc1a`（搭配 nightly toolchain `nightly-2025-11-08`）。三个 hax backend（coq / fstar / lean）共享同一 `hax-engine` OCaml binary——装一次即可。

**违反**：未违反——按 [`tool-integration.md`](../../../design/tool-integration.md) §一 锁定 commit hash + nightly toolchain pin。

**决策性**：非决策点。

**建议**：无须改。

#7 (严重度: 低) — 关联 sub-tests 与 hax-coq / hax-fstar 共享

**现象**：[`tools/hax-lean/README.md`](../../../../tools/hax-lean/README.md) L78 与 hax-coq / hax-fstar 关联 sub-tests 文字一致。

**违反**：未违反——3 个 hax backend 共享 limit 类目。

**决策性**：非决策点。

**建议**：无须改。

## §4 决策点 vs 非决策点

- 决策点：1（是否加 entry_fn 存在性 grep 与 hax-coq / hax-fstar 对齐）
- 非决策点：6

## §5 审查结论

hax-lean README 是 silent path 检测论证的优秀范例：

- ⚠️ 实测验证措辞准确（与 kani README L46 "形式可证"形成对比，hax-lean 措辞更严谨）；
- 漏报盲点诚实声明（"hax engine 完全 skip item 0 现象" + "上游 PR #1672 升级路径"）；
- prelude 防误命中 + cargo metadata 坑提示等细节完整。

唯一可考虑：是否加 entry_fn 存在性 grep 与 hax-coq / hax-fstar 对齐——这是 silent skip-item 漏报盲点的 belt-and-braces 封堵。当前 README 实测说"corpus 0 现象"——属可接受声明。

整体属于"高诚实度、高论证质量"的范例。
