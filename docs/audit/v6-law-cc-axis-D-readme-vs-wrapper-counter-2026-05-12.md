# Axis D counter-challenge: README/tool.toml 文案 vs wrapper 实际行为

审查日期：2026-05-12
对象：`docs/audit/v6-law-c-axis-D-readme-vs-wrapper-2026-05-12.md` 的 4 候选
方法：disprove-first counter——以"现象 ≠ 缺陷 / 文案 ≠ 实现 / 局部 stale ≠ 全局缺 / 历史注释 / 流程建议"为优先反挑刺。

---

## 候选 D-1 — rocq-of-rust README "6 道门" vs wrapper 7 道门

**c 路证据复核**：README L43 字面 `6 道门`；wrapper L128-140 实施 gate 7 (`grep "is not yet supported"`)；wrapper 头注 L49-58 同样停在 6 道门叙述。**事实层不一致成立**。

**counter 反挑**：
- **R11（文案 ≠ 实现）✓** — wrapper gate 7 已在 P28 / D3.1 落地，oracle 实际行为 sound（多查一道只会增 reject 严格性，不破 0 误报论证）。README 缺更新 = 文案脱节，不是 contract violation。
- **R6（局部 stale ≠ 全局缺）部分** — wrapper gate 7 注释 L128-134 已显式引 `docs/fixes/decisions-2026-05-11.md` D3.1；同源决议有上游文档承载。
- **R12（作者反驳）** — 作者会说 gate 7 与 gate 5（产物 failure marker）语义同属 "silent partial 通道封堵"，README §"漏报盲点" 概念上覆盖，但确实没枚举 stderr 通道；下一次 README pass 补 1-2 行。
- **R1 反方**：原 c 路引用 `0 漏报论证基础` 受影响——但 README §"漏报盲点" L62 实际已写"上游引入新 silent fallback 路径不带已知 markers"——可解读为对 stderr 通道兜底声明，仅未点名 stderr 锚。

**counter 结论**：候选**部分成立 / 低严重性**——事实不一致真实存在，但属 R11 文案 + R6 局部 stale，不影响 oracle 正确性。

---

## 候选 D-2 — rocq-of-rust-typecheck README "9 道门" + "等价 6 道 grep" vs wrapper 10 道

**c 路证据复核**：wrapper L116-126 自己注释明写 `tier-1 README 第 43 / 136 行硬声明 ... v6 cc-route 漏报审查发现 tier-1 漏抄此 gate，破声明。补此 gate 恢复 tier-1 ⊆ tier-0 不变式`。**作者自陈不一致**——这是最强的"事实层成立"信号。

**counter 反挑**：
- **R11（文案 ≠ 实现）✓** — 同 D-1，gate 实施已对齐 tier-0，oracle 行为 sound。
- **R1（现象 ≠ 缺陷）反方部分失败** — wrapper 自陈"破声明"是作者已意识到的，"档 1 ⊆ 档 0" 不变式的可审性确实受影响——但补 1 道 gate 是**让 tier-1 更严**，不变式方向是向"更严"漂，不是"漏抓"。
- **R4（上游已消化）✓** — wrapper 已替 README 补齐，oracle 端无窟窿。
- **R12（作者反驳）** — 作者会说 D-2 与 D-1 同源（P28 / D3.1 落地时漏改两处 README 门数表），可一次 README pass 同批改：`9 道门` → `10 道门` / `6 道 grep` → `7 道 grep`。

**counter 结论**：候选**部分成立 / 低严重性**——事实不一致最显眼（wrapper 自陈），但仍属 R11 文案 + R6 局部 stale，oracle 实际行为已对齐宪法。

---

## 候选 D-3 — prusti README "wrapper 联合实施 stderr marker" vs wrapper 只查 exit + .vpr

**c 路证据复核**：README L56-60 列 3 个 stderr marker；L62 "两个条件由 wrapper 联合实施"；wrapper L48-87 实际只查 `arch -x86_64` exit + `.vpr` 数。**事实层不一致**。

**counter 反挑**：
- **R2（design contract）✓** — 3 个 Prusti marker（`[Prusti: unsupported feature]` / `[Prusti: internal error]` / `panicked at mir_storage.rs`）**必伴 exit ≠ 0**（Prusti 设计不变量）。wrapper 单看 exit code = oracle 等价覆盖 stderr marker 集合——sound by construction。
- **R11（文案 ≠ 实现）✓✓** — 这是典型 README 措辞歧义："联合实施"可解读为(A) wrapper 内有 stderr grep 代码 / (B) oracle 信号集合上等价覆盖。实际是 (B)。
- **R12（作者反驳）** — 作者会说 README §形式严格性 L66-68 已明写 "wrapper 检 .vpr 数量为 0 → FAILED"，没承诺 wrapper 内有 marker grep。"联合实施"是表述简化，意指 marker 与 exit 在 oracle 层面合一。
- **R1 反方**：实测 oracle 行为 = 字面正确（exit ≠ 0 拒所有 3 类 marker case + .vpr 兜底 silent skip）。

**counter 结论**：候选**不成立 / 文案级**——属 R2 design contract + R11 措辞简化，oracle 正确性 0 影响。下次 README pass 可改"oracle 上等价联合实施"即可。

---

## 候选 D-4 — verifast tool.toml L7-13 "N ≤ 40 threshold" stale 注释

**c 路证据复核**：`tool.toml` L7-13 `"0 errors found (N statements verified)" with N ≤ 40`；wrapper L15-23 + L25-40 + L105-114 明确说 N 不可分辨真假 SUCCESS，改用 `src/lib.rs(` 锚 grep；README L48-52 与 wrapper 一致。**仅 tool.toml 头注 stale**。

**counter 反挑**：
- **R5（历史 / 废弃）✓✓** — tool.toml 头注是当年 design pass 的描述性记录，wrapper 内的设计自陈才是事实源；wrapper L25 明写 `Rule (more robust than the audit's ≤ 40 N-threshold)`——升级路径有据。
- **R6（局部 stale ≠ 全局缺）✓✓** — README + wrapper 双处都对齐 src/lib.rs 锚，仅 tool.toml 头注遗留；下游消费者（runner）不读 tool.toml comment，只读 `command` / `timeout_secs`。
- **R11（文案 ≠ 实现）✓** — 注释字面 vs 实施差异不影响 oracle。
- **R12（作者反驳）** — 作者会说 "tool.toml 头注是审计当时的实施快照，wrapper 升级后我应该 sed 一刀但没顺手；下次 README pass 顺便清"——纯流程 nit。

**counter 结论**：候选**不成立 / 极低严重性**——R5 + R6 + R11 三命中，纯历史注释，下次顺手删。

---

## 总结

| 候选 | 成立 / 不成立 | 严重性 | 主反挑 |
|---|---|---|---|
| D-1 ror | 部分成立 | 低 | R11 文案 / oracle 已对齐 |
| D-2 ror-typecheck | 部分成立（作者自陈） | 低 | R11 文案 / oracle 已对齐 / 漂向更严 |
| D-3 prusti | 不成立 | 文案级 | R2 marker 必伴 exit ≠ 0 / R11 措辞简化 |
| D-4 verifast | 不成立 | 极低 | R5 历史注释 / R6 局部 stale / R11 |

**根本判定**：4 候选**全部**实际 oracle 行为已对齐宪法 §六，问题面只在文档描述滞后于实施。无一个候选影响 SUCCESS / FAILED 判定、0 误报 / 0 漏报论证、或 partial 暴露通路。属下次 README pass 顺手收拾的 process nit，不是 blocking。

**建议 fix 操作**（3-5 行 diff 即可，非 blocking）：

```diff
# tools/rocq-of-rust/README.md
- L43: 6 道门
+ L43: 7 道门（含 stderr "is not yet supported" silent partial gate）
- L47-50 后插一条 gate 7
+ §漏报盲点 L64 后补一行 stderr 通道说明

# tools/rocq-of-rust-typecheck/README.md
- L43 + L136: 同 6 道 grep
+ L43 + L136: 同 7 道 grep
- L47: 满足 9 道门
+ L47: 满足 10 道门
- §SUCCESS 信号 gate 6 后插 gate 7（stderr 通道）

# tools/prusti/README.md
- L62: 两个条件由 wrapper 联合实施
+ L62: 两个条件 oracle 上等价联合实施（marker 必伴 exit ≠ 0，wrapper 通过 exit + .vpr 数双道门已覆盖）

# tools/verifast/tool.toml
- L7-13: 删除 "N ≤ 40 threshold" 注释段，简化为指引 "详见 wrapper 头部 §Rule"
```

修复优先级：D-1 + D-2 同批改（同源 P28 / D3.1 落地遗漏，5 行 sed 解决）；D-3 + D-4 下次 README pass 顺手清。
