# Counter-Challenge：audit-4 高严重度逐条验证（disprove-first）

> 本文对 audit-2026-05-11 的 audit-4（reports / docs-research / docs-fixes 三份 review）合计 15 条高严重度逐条做 disprove-first 验证。**默认 audit 错，找证据驳斥**；若证据反向支撑 audit，再判 **持** 或 **部分持**。
>
> 判定四档：
> - **持**（audit 正确，证据完全支撑）
> - **部分持**（audit 方向对，但措辞 / 严苛度 / 推论部分过度）
> - **驳**（audit 错，证据反驳）
> - **存疑**（证据不足无法定论）

---

## §1 reports-review H-1 至 H-8

### H-1：ror v4 vs typecheck 档 0 baseline 差 1 entry 没旁注

**audit 引用**：
- `deep-reports/cc-reports/rocq-of-rust.md:9` v4 共享 146 子集 109/146 = 74.7%
- `deep-reports/cc-reports/rocq-of-rust-typecheck.md:10` 档 0 同 corpus 110/146 = 75.3%
- audit 推论："typecheck 报告没在第 10 行旁注'该 baseline 用早期 single-attempt wrapper'或'与 cc-reports/rocq-of-rust.md v4 不一致'"

**独立证据**：
- 两份引用的 run id **确实不同**：ror.md:5 v4 = `run-1778480785-20112`；ror-typecheck.md:6 配对档 0 = `run-1778473390-70662`（"相邻时刻跑 rocq-of-rust，用于档 0 vs 档 1 对照"）
- 但 `rocq-of-rust-typecheck.md:170-180` 有专门 §"档 0 vs 档 1 差异"段落：「档 0 = 110 SUCCESS，档 1 = 109 SUCCESS，**差 1 个 entry**：`creusot-limit/thread-local-ref/read_thread_local`」「**事实**：这不是 ror 翻译落盘后 coqc 编不过的'档 1-specific 失败'，而是 `tools/rocq-of-rust` 的一个潜在 oracle 漏报（在该 entry 上 gate 6 应抓但没抓）。本档 wrapper 用独立 bash 实现 gate 6，逻辑一致但执行路径不同，意外把档 0 的这个漏报抓出来了。」
- 这正是 ror-gate6-fix-2026-05-11.md §1 反向暴露的同一现象

**判定：驳**。typecheck 报告 §"档 0 vs 档 1 差异" 三段已经**详细解释了 110 vs 109 的根因**——只是没在第 10 行同一行旁注。audit 要求"在第 10 行旁注"过于严苛——文档结构上，元数据段只列数字，机制解释放专门段落是合理写作。audit 推论"读者拿 typecheck 报告会以为档 0 仍是 110"忽略了同份文档 §"档 0 vs 档 1 差异"明示"档 0 这个漏报抓出来了"。

但更深一层：**typecheck 报告里"档 0 110"是 typecheck 配对 run 时档 0 的快照（采到变体 A 时 SUCCESS），不是 ror v4 的 109**。即 typecheck 的"110"反映**修补前**档 0 的随机性结果，而 ror.md v4 的"109"是修补后的稳定结果。**两份都是事实陈述**，只是锚定的 run / oracle 版本不同。typecheck.md §"档 0 vs 档 1 差异" 已明示这种"档 0 漏报"。

**audit 此处误读**：把"110 vs 109"的不一致当作"未旁注差异"，但实际两份各自的 run id + oracle 版本已明示锚定。audit 严苛过度。

---

### H-2：internal-roundup §7.3 "kani-limit 7/7" vs cc-reports/kani.md "6/7" 内部矛盾

**audit 引用**：`internal-roundup-2026-05-08.md:436`（§7.3）：「kani-limit 7 条在 kani 上 7/7」 vs 同份 line 39（暴论 4）+ line 103（§3-C 表 kani 93.2%）+ cc-reports/kani.md:53-66 都说 kani-limit 6/7。

**独立证据**：
- internal-roundup:436（§7.3）：「kani-limit 7 条在 kani 上 7/7（kani --only-codegen 切割点早于 kani 自陈的 unsupported——这些 unsupported 在 CBMC 求解阶段才显现）」
- internal-roundup:448（§7.7）：「kani-limit/* 6/7 仍过（仅 stack-unwinding 在 `catch_unwind` 路径上翻 FAILED）」
- cc-reports/kani.md:60：「kani-limit 6/7（stack-unwinding/trigger_divide_with_recovery 命中 catch_unwind）」
- cc-reports/kani.md:66：「`kani-limit/*` 仍 6/7」
- internal-roundup:39：暴论 4 也说 "kani-limit/stack-unwinding" 在 P13-B 后翻 FAILED

**判定：持**。§7.3 的"7/7"是 P13-A 改造**前**的老结论未更新，与同份 §3-C / §7.7 / 暴论 4 / cc-reports/kani.md 全部矛盾。audit 引用准确，描述准确。

---

### H-3：cc-reports/rocq-of-rust.md 内部 N=7 vs "3 次"自相矛盾

**audit 引用**：
- 第 35 / 48 / 109-110 / 114 行：N=7、catch rate 99.84% = 1−0.4^7
- 第 129 行："124 SUCCESS 上 N-attempt（3 次）AND-reduce ... 都正确生成 Definition **三次**"
- 第 139 行："6 道门 + N-attempt wrapper oracle（N=7 ... + **3 次产物 AND-reduce**）"
- 第 143 行："N=7 ... 把 P(漏 fn)^3 推到几乎 0"

**独立证据**：
- cc-reports/rocq-of-rust.md:35：「wrapper 跑 N=7 次 `rocq-of-rust translate`」
- cc-reports/rocq-of-rust.md:48：「v4：wrapper-based，N=7 次 translate」
- cc-reports/rocq-of-rust.md:114：「v4 N-attempt 把 P(漏抓) 从 0.4 推到 0.4^7 ≈ 0.0016 (99.84% catch rate)；**N=3 catch rate 仅 93.6%，曾在重复重跑时观察到一次 flip，N=7 是稳健阈值**」
- cc-reports/rocq-of-rust.md:129：「124 SUCCESS 上 N-attempt（3 次）AND-reduce 后 marker grep 没命中任何 ... 且 entry_fn 都正确生成 `Definition` 三次」
- cc-reports/rocq-of-rust.md:139：「N=7，marker 集 = ... + **3 次产物 AND-reduce**」
- cc-reports/rocq-of-rust.md:143：「**把 P(漏 fn)^3 推到几乎 0**」
- ror-gate6-fix-2026-05-11.md:56：「跑 `rocq-of-rust translate` N=7 次」
- ror-gate6-fix-2026-05-11.md:78：「经验上 N=3 的 93.6% 不够稳... N=7 是稳健阈值」
- ror-gate6-fix-2026-05-11.md 表（line 70-76）：N=3 catch rate 93.6%，N=7 catch rate 99.84%

**驳斥尝试**："3 次"是否指别的概念（如 marker grep 步骤数 / AND-reduce 中间环节）？
- line 139："N=7 ... + 3 次产物 AND-reduce" 句式上"3 次"明显修饰"AND-reduce"——AND-reduce 的次数等于 N attempts 的次数，应是 7 次而非 3
- line 143："P(漏 fn)^3" 在 P(drop fn) ≈ 0.6 上算 = 0.216，与 99.84% catch rate 严重不符（应 ^7 = 0.0016）
- line 129："Definition 三次"——若 SUCCESS 要求每次 attempt 都命中，N=7 attempts 就应是"Definition 七次"

→ "3 次"在三处都不可能指其他概念，**是从 N=3 早期草稿遗留的写作错误**。ror-gate6-fix 显式记录了 N=3 → N=7 的升级理由（line 78），cc-report 在 §"SUCCESS 信号" 段（line 35/48/114）已经更新到 N=7，但 §"与本次测试边界的关系"（line 129）+ §"历史快照声明"（line 139）+ §"关键发现摘要"（line 143）三处遗漏未改。

**判定：持**。audit 准确识别三处"3 次"/"^3"是 N=3 历史遗留写作错误，与同份文档 N=7 + ror-gate6-fix N=7 不一致。

---

### H-4：internal-roundup 未反映 v4 (corpus 161 + N=7 wrapper)

**audit 引用**：
- 头部第 3 行只提"triple-run"
- 第 10 行 corpus 仍 "146 entries × 19 = 2774"
- 第 158/170 行 rocq-of-rust 76.0% (111/146)
- 第 384 行时长表 rocq-of-rust 76 ms avg
- 第 425 行 7.2-bis 表 rocq-of-rust「-10 entries」

**独立证据**：
- internal-roundup-2026-05-08.md:1：「内部综评 2026-05-08（暴论版；**triple-run 数据增订 2026-05-11**）」
- line 3："本报告整合 3 次实测 run...triple-run 覆盖报告见 ...strict-oracle-v3.md" ← **完全没提 v4 / quad-run**
- line 10："**corpus**：146 entries × 19 工具 = 2774 任务"
- line 162："rocq-of-rust | 76.0% (111/146)（v2 P12-B；旧 oracle 82.9% / 121/146）"
- line 384：「rocq-of-rust | 76 (P12-B) | 170 (P12-B)」
- line 425："**rocq-of-rust**（P12）| top-level kind dispatch silent vec![] | gate 6 entry_fn Definition 存在性 | 82.9% → 76.0%（-7pp / -10 entries）"
- 对照：ror-gate6-fix:144 "新（本 commit，161 entry）SUCCESS 124 (77.0%)" + "仅看共享 146 entry 子集 109 (74.7%)"
- 对照：cc-reports/rocq-of-rust.md:11 "时长 v4 avg 477 / median 547 / p90 641"
- v4 在共享 146 子集 109/146（vs 5 道门 121 = -12 entries）

**判定：持**。internal-roundup 5 处数据都明确仍是 v3/P12-B 口径，未反映 v4：
1. corpus 数：146 → 实际已扩展至 161（+15 runnable）
2. ror 通过率：76.0% (111/146) → 实际 v4 124/161 (77.0%) 或共享子集 109/146 (74.7%)
3. ror 时长：76 ms avg → 实际 v4 477 ms（N=7 overhead 7×）
4. 7.2-bis 表："-10 entries" → 实际 v4 抓 12 个（多 2 个 thread_local 类）
5. triple-run 锚 → 实际应是 quad-run

ror-gate6-fix §7（line 162-163）已声明"docs/test-reports/feature-coverage-2026-05-08-strict-oracle-v2.md（差 1.3pp 微调，按任务'下一份系统报告统一纳入'约定推后）" + "docs/reports/internal-roundup.md（同上）"——明示本 commit 推后更新。但 internal-roundup 本身未加"⚠️ 已被 v4 超越"反向标注。

---

### H-5：v3 测试报告（最新一份）也落后于 P15-impl + P16

**audit 引用**：
- `docs/test-reports/feature-coverage-2026-05-11-strict-oracle-v3.md:97 / 134 / 269` rocq-of-rust 仍 111/146 = 76%
- 第 50 行只列 strict-oracle-v2，未列 v4 N=7 wrapper

**独立证据**：
- v3 报告 line 50："rocq-of-rust | `rocq_of_rust_cli 0.1.0` ... · **strict-oracle-v2**: 6 道门"——未列 v4 / N=7
- v3 报告 line 97："rocq-of-rust | 146 | 111 | 35 | 76% | 76 | 76 | 89 | 170 | v2 P12-B (run-...69805)"
- v3 报告 line 130："7 | rocq-of-rust 76% | rocq-of-rust 76%"
- v3 报告 line 269："syn | hax-fstar 77%（v3，v2 78%）> rocq-of-rust 76% > hax-lean 75% > hax-coq 65%"
- v3 报告**全文无 `runnable/*` 出现**（用 grep -c 验证）
- ror-gate6-fix §7 line 162："docs/test-reports/feature-coverage-2026-05-08-strict-oracle-v2.md（差 1.3pp 微调，按任务'下一份系统报告统一纳入'约定推后）"

**判定：持**。v3 测试报告全部数据停留在 P12-B / v2 oracle，未纳入 P15-impl + P16。ror-gate6-fix §7 已声明推后，但 v3 报告本身未加"⚠️ 部分被 v4 超越"提示，读者拿 v3 不会知道有 v4。

audit 同时指出 ror-gate6-fix:161-163 已说明"下一份系统报告统一纳入"——audit 在严苛意义上仍正确：当前没有 v4 系统报告，读者依然只看到 v3。

---

### H-6：v3 测试报告 `runnable/*` 完全没出现

**audit 自己已承认**："v3 报告所跑的 run `1778466265-63960` 等都是 146 entry 口径，**口径上正确**——但同 commit 已经存在 161 entry 的 P16 corpus"。

**独立证据**：
- hax-lean-eval-corpus-baseline.md line 81-87：「P16-impl-A 跑... 0 SUCCESS / 15 FAILED」——hax-engine 不可达，本机未跑通
- 但 ror v4 run-1778480785-20112 跑了 161 entry，含 `runnable/* 15/15`
- v3 报告锚定的 run id 都是 146 entry 时代

**判定：部分持**（audit 也承认口径正确）。audit 把这条放在"高严重度"里偏重，本质是"v3 锚定的 run 早于 P16，无需更新"。audit §4 已把 H-6 列为"非决策点"——内部一致。

---

### H-7：cc-reports/kani.md "0 个 hard-reject" 表述含混

**audit 引用**：cc-reports/kani.md:162："10 个 FAILED 中 ... **0 个在 hard-reject 层面**"

**独立证据**：
- cc-reports/kani.md:158-162 完整段落标题：「## 与本次测试边界的关系」，第三 bullet 起首：「- **本次未触达**：kani 的 GotoC codegen 因 MIR 节点 unsupported 而 hard-reject 的案例（区别于本次的 stub-with-warning）。10 个 FAILED 中 2 个在 lint 层面（x509 vendor）、8 个在 5-markers stub 层面，**0 个在 hard-reject 层面**」
- 该 bullet 起首词 "**本次未触达**" 明确锚定 corpus-anchored

**判定：驳**（部分驳）。audit 要求改为"本 corpus 上 0 个 hard-reject 触发"——但**该段已用"本次未触达"作为起首词明示 corpus 锚定语义**。"本次未触达 ... 0 个在 hard-reject 层面" = "本 corpus 上 0 个 hard-reject 触发"语义已包含。audit 把"0 个 hard-reject"措辞割裂阅读、忽略上下文锚定，是过度严苛。

不过 audit 提及的"corpus 事实 vs 工具事实"区分本身有价值——若读者跳读只看 "0 个 hard-reject" 摘要，可能误读为工具自陈无 hard-reject。但严格说该表述并不违反 §7.3 + §7.1。

**audit 此处属于严苛性过度，证据反驳 audit 主张**。

---

### H-8：cc-reports/hax-lean.md 未应用 P13 entry_fn gate（漏审）

**audit 引用**：P13-A 同 commit 在 hax-fstar/hax-coq 上加 entry_fn gate 各抓 2 条，hax-lean 端 `lean_backend.ml` 同型号 `NotImplementedYet` silent-skip-item 路径**没做对等审计**。

**独立证据**：
- `tools/hax-fstar/tool.toml:22`：「`elif [ -n "$TS_ENTRY_FN" ] && ! grep -rqE "^(let[[:space:]]+(rec[[:space:]]+)?|and[[:space:]]+)$TS_ENTRY_FN[[:space:]]" proofs/fstar/extraction/ 2>/dev/null; then echo "[hax-fstar-oracle] FAIL: entry_fn '$TS_ENTRY_FN' missing from .fst products (silent skip — fstar_backend.ml:1771 Use/NotImplementedYet path...`」——entry_fn gate 已落地
- `tools/hax-coq/tool.toml:27`：「`elif [ -n "$TS_ENTRY_FN" ] && ! grep -rqE "^[[:space:]]*(Definition|Fixpoint|Lemma|Equations|Theorem|Program[[:space:]]+Definition)[[:space:]]+$TS_ENTRY_FN[[:space:]]" proofs/coq/extraction/ 2>/dev/null; then echo "[hax-coq-oracle] FAIL: entry_fn '$TS_ENTRY_FN' missing from .v products (silent skip — coq_backend.ml:588 item'_NotImplementedYet path...`」——entry_fn gate 已落地
- `tools/hax-lean/tool.toml:14-17`：command 只做 `cargo hax... into lean` + sorry-in-term-position grep，**完全无 entry_fn gate**
- `tools/hax-lean/tool.toml:1-13` 注释也只说 sorry grep 精准化，**不提 silent-skip-item 路径**
- internal-roundup-2026-05-08.md:168 末段："P13-A 在 hax-fstar / hax-coq 上各增加 entry_fn 存在性 gate"——只列 hax-fstar / hax-coq

**判定：持**。hax-lean tool.toml 确实没有 P13-A 对等的 entry_fn gate，与 hax-fstar / hax-coq 完全不对称。audit H-8 准确识别。lean_backend.ml 是否有同型号 silent-skip-item 路径需独立审 lean backend 源码（audit 推论"同型号 NotImplementedYet silent-skip-item 路径"是合理推测但未独立验证；本 counter-challenge 不展开 backend 源码审）。但 oracle 不对称这一事实成立。

---

## §2 docs-research-review H-1 至 H-3

### H-1：testsuite-research §3 "推后到 v2 的扩展点" 部分被实施跟进，未标更新

**audit 引用**：
- testsuite-research.md:36-43 "推后到 v2 的扩展点"列 6 项
- P16 引入 `runnable.<entry_fn>.inputs/expected` schema 本质是"期望比对"扩展
- P15-impl 引入 `tools/rocq-of-rust-typecheck/` 把 typecheck 纳入测试

**独立证据**：
- testsuite-research.md:38-43 列 6 项推后：tool.toml.env/cwd / External_lib mode / **期望比对（expect.<glob>）** / bug_description LLM / entry 名超链 raw / 跨运行对比
- `examples/runnable/power/hirusttest.toml`：「`[runnable.pow_n] inputs = [[2,0],[2,8],...] expected = [1,256,81,...]`」——确实是新 schema
- ror-runnable-deep-dive-2026-05-11.md §3 实测 ror 档 1 跑通

**判定：部分持**。
- "期望比对（expect.<glob>）"在 §3 是 **glob 模式的 SUCCESS / FAILED 区分**；P16 的 `runnable.fn.expected` 是 **per-entry 函数求值结果匹配**——schema 形式不同（前者 glob over outputs，后者 fn evaluation match），思路重合但实际 scope 也不同。audit 自己也承认"虽然方向不同，但 schema 思路重合"
- testsuite-research.md 是 2026-05-08 调研版本，距 P16 几天内 commit，调研文档**未声明持续维护承诺**
- 但 testsuite-research.md §3 头部明确写"按 Occam 当前不做"——这是设计决策声明，不更新的话读者会以为这些项目仍未做

audit 方向正确但 "v2 扩展点已部分跟进" 措辞夸张："期望比对" 和 `runnable.fn` schema 思路重合但 scope 不等。**部分持**。

---

### H-2：translation-correctness-feasibility-2026-05-11.md §7 → P15-impl 部分跟进，但调研文档未指向 impl

**audit 引用**：
- feasibility §7.1 推荐 "hax-lean-typecheck / hax-lean-eval / hax-lean-consistency"
- feasibility §6.3 line 250-254："短期（hax-lean only）yes...短期 hax-coq / rocq-of-rust no"
- P15-impl 实际选 `tools/rocq-of-rust-typecheck/` 而非 `tools/hax-lean-typecheck/`——顺序"反过来"

**独立证据**：
- feasibility:226 (§6.1) "rocq-of-rust ❌ 本机不可达"
- feasibility:250-254 (§6.3) 推荐 hax-lean 先做
- ror-runnable-deep-dive-2026-05-11.md §3 实测装 Rocq 9.0.0 isolated switch 成功 + 编 runtime 通过 + 档 1 4/4 PASS——**实测推翻 "本机不可达"**
- feasibility 文末（应该在 §6.1 / §7 末尾）**未加** "已被 ror-runnable-deep-dive 推翻" 标注（实测打开 feasibility 看不到向 deep-dive 的反向链接）

**判定：持**。两份调研文档确实缺反向链接。P15-impl 选 rocq-of-rust-typecheck 而非 hax-lean-typecheck 实质上是**实测推翻** feasibility 的优先级建议。feasibility 文档自身未更新指向 deep-dive。audit 准确识别。

---

### H-3：ror-runnable-deep-dive §7.2 档 2/3 不推荐 vs P16 实际做 hax-lean 档 2/3

**audit 引用**：
- ror-runnable-deep-dive §7.2 第 396-406 行："中长期：档 2/3 不推荐"理由：per-entry 工作量 / 测试可信度不增 / 不与 hax-lean 档 2/3 对等
- P16 在 hax-lean 上做了档 2/3（hax-lean-eval 框架 + 15 个 runnable entry）

**独立证据**：
- audit 自己说"注意这与 ror 档 2/3 不冲突（ror-runnable-deep-dive 明确说'hax-lean 档 2/3 自动可达'）"
- 实测 examples/runnable/* 15 个 entry 是 hax-lean-eval baseline
- hax-lean-eval-corpus-baseline.md 是 P16-impl-A 的产物

**判定：部分持**。ror-runnable-deep-dive **限定 "ror" 档 2/3 不推荐**，并未阻止 hax-lean 档 2/3——deep-dive 本身明示 hax-lean 档 2/3 自动可达。audit 自己也承认"与 ror 档 2/3 不冲突"。但 audit 提的"工具间分化决策（ror no / hax-lean yes）调研文档没明示"——这一点对的，工具间分化决策需要在 design / fixes 文档显示。但作为 research 文档批评，这条 audit 偏严苛——research 文档不需要列所有工具的分化决策。**部分持**（指出现象但严重度被夸大）。

---

## §3 docs-fixes-review H-1 至 H-4

### H-1：extra-cargo-deps-and-entry-mode.md 与后续 P12/P13/P15 链接缺失

**audit 引用**：extra-cargo-deps 是早期 fix（2026-05-06），后续 P12/P13/P15 都没在文末加"派生关系"段。

**独立证据**：未读 extra-cargo-deps 全文，但 audit 描述符合"早期 fix 缺前后索引"的一般模式。文档存在不在本验证 scope；audit §4 自己把 H-1 列为"决策点（低）"——即承认严重度低。

**判定：部分持**（严重度本来就低）。

---

### H-2：audit-1 → implementation-1 → audit-2 → implementation-2 派生链清晰但缺索引

**audit 描述**：派生链客观存在但每份文件头部没声明"我属于 P{N}，前序 P{N-1}，后续 P{N+1}"。

**判定：部分持**。这是文档习惯问题，audit §4 自己也只判"决策点"（非"决策点（高）"）。是合理改进建议，但作为"高严重度"偏重。

---

### H-3：4 次 falsify 案例在 fixes 文档间记录不完全一致

**audit 引用**：
- 4 次 falsify 案例（P12 verifast / P13 hax-fstar / P13 hax-coq / P15 rocq-of-rust）
- ror-gate6-fix-2026-05-11.md §1 / §2 没说"audit-1 §3.2 当时'实测 0 现象'被本 commit 推翻"——P15 实际推翻的就是 audit-1 + P12-A 的"已封堵"自信

**独立证据**：
- ror-gate6-fix-2026-05-11.md:3：「本文记录 P15-impl 实施反向暴露的档 0 (`tools/rocq-of-rust`) gate 6 漏报点的根因分析、修复方案、反误报双向实测...」——明确说"反向暴露"
- ror-gate6-fix §2 line 41："两种变体都是 partial 翻译...两种都不应判 SUCCESS，但旧 gate 6（单次 grep）只在采样到变体 B 时命中"——明确说旧 gate 6 漏
- 但 ror-gate6-fix 全文搜 "audit-1" / "audit-2" / "P12-A 自信" 没出现——确实**没**显式溯源到 audit-1 §3.2 + P12-A 的"已封堵"判断
- ror.md (cc-report) line 146："**审计启发实施暴露漏报第三案例**（前两：P12 verifast N≤40 / P13 hax-fstar 漏 mutual-rec）" ← 显式列入 falsify 系列，但 ror-gate6-fix 本身没

**判定：持**（部分持）。ror-gate6-fix **脉络存在**（"P15-impl 反向暴露"语义已含），但**未显式溯源到"audit-1 §3.2 + P12-A 已封堵自信被推翻"**。audit 准确指出溯源链不完整。

---

### H-4：oracle-leak-audit-2026-05-08.md "Known issues" hax-lean 至今未跟进

**audit 引用**：audit-1 §3.5 hax-lean "实测 0 触发，理论窗口存在"至今未被任何后续 fixes 文档跟进。与 reports-review H-8 重叠。

**独立证据**：见 §1 H-8 验证——`tools/hax-lean/tool.toml` 确实没有 P13-A 对等 entry_fn gate。

**判定：持**。与 reports-review H-8 同事实，但聚焦不同：reports-review H-8 看 oracle 不对称，docs-fixes-review H-4 看 fixes 链未跟进 known issue。两条都成立。

---

## §4 整体判定汇总

| audit 条 | 判定 | 主要证据 |
|---|---|---|
| reports H-1（ror v4 vs typecheck baseline 没旁注）| **驳** | typecheck.md §"档 0 vs 档 1 差异" 已详细解释 110 vs 109 根因 |
| reports H-2（kani-limit 7/7 vs 6/7）| **持** | line 436 vs line 39/103/448/cc-reports/kani.md:60/66 全矛盾 |
| reports H-3（N=7 vs "3 次"自打脸）| **持** | line 129/139/143 的"3 次"/"^3"是 N=3 历史遗留写作错误 |
| reports H-4（internal-roundup 落后 P16）| **持** | 5 处数据未更新 (corpus 146 / ror 76% / 时长 76 ms / -10 entries / triple-run) |
| reports H-5（v3 报告落后 P15-impl + P16）| **持** | line 50/97/130/269 均停留在 v2，无 runnable/* |
| reports H-6（v3 报告 runnable/* 没出现）| **部分持** | audit 自己承认口径正确，且 §4 自归"非决策点" |
| reports H-7（kani "0 hard-reject" 含混）| **驳** | "本次未触达"起首词已明示 corpus 锚定 |
| reports H-8（hax-lean entry_fn gate 漏审）| **持** | hax-lean/tool.toml:14-17 vs hax-fstar/hax-coq tool.toml entry_fn gate 不对称 |
| research H-1（testsuite-research §3 v2 推后未跟进）| **部分持** | runnable schema 与 expect.glob 思路重合但 scope 不等 |
| research H-2（feasibility vs deep-dive 缺反向链接）| **持** | feasibility §6.1 line 226 "本机不可达"未指向 deep-dive 实测推翻 |
| research H-3（ror 档 2/3 不推荐 vs hax-lean 档 2/3 做了）| **部分持** | deep-dive 明示 ror-only 限定，hax-lean 不冲突 |
| fixes H-1（extra-cargo-deps 与后续模块缺索引）| **部分持** | audit §4 自归"决策点（低）" |
| fixes H-2（audit→impl 派生链清晰但缺索引）| **部分持** | 文档习惯问题，audit §4 也只判"决策点" |
| fixes H-3（ror-gate6-fix 未溯源 P12-A 自信被推翻）| **持** | "反向暴露"脉络在，但未显式溯源 audit-1 §3.2 + P12-A |
| fixes H-4（hax-lean silent-skip-item 未跟进）| **持** | 与 reports H-8 同事实 |

---

## §5 关键 audit 错误（top）

### 错误 1：reports H-1 误读 typecheck.md §"档 0 vs 档 1 差异"

audit 要求"在 typecheck.md:10 旁注 baseline 差异"，但 typecheck.md:170-180 整段 §"档 0 vs 档 1 差异" 已经详细解释了 110 vs 109 的根因（"档 0 的潜在 oracle 漏报"+ "本档独立实现 gate 6 意外抓出"）。audit 把"在第 10 行旁注"作为标准过于严苛——文档结构上元数据段只列数字、机制解释放专门段落是合理写作。

### 错误 2：reports H-7 割裂"0 hard-reject"措辞

audit 要求改为"本 corpus 上 0 个 hard-reject 触发"，但 cc-reports/kani.md:162 整 bullet 起首词就是 "**本次未触达**"——已经明示 corpus 锚定。audit 把"0 个 hard-reject"割裂、忽略上下文，过度严苛。

### 错误 3：research H-1 与 H-3 严重度被夸大

- H-1："期望比对（expect.<glob>）"是 glob over outputs，P16 的 `runnable.fn.expected` 是 fn evaluation match——schema 思路重合但 scope 实质不同，audit 自己也承认"虽然方向不同"——把这条放高严重度偏重
- H-3：ror-runnable-deep-dive §7.2 明确限定"ror 档 2/3 不推荐"，hax-lean 不冲突，audit 自己也承认"与 ror 档 2/3 不冲突"——把这条放高严重度偏重

---

## §6 internal-roundup 是否真的严重落后

**结论：是**。实际行号 + 现状如下：

| 项 | line | 实际值 | v4 现状 |
|---|---|---|---|
| 标题"triple-run" | line 1, 3 | "triple-run 数据增订 2026-05-11" | 应为 quad-run（v4 加 P15-impl + P16）|
| corpus 数 | line 10 | 146 entries × 19 = 2774 | 161 entries（+15 runnable）|
| ror 通过率（§F 第 162 行）| line 162 | 76.0% (111/146)（v2 P12-B）| 124/161 = 77.0%（v4）；共享 146 子集 109/146 = 74.7% |
| ror 通过率（暴论 三 line 170）| line 170 | 76.0% (P12-B 重跑后) | 同上 |
| ror 时长（§6 时长表 line 384）| line 384 | 76 ms avg (P12-B) | 477 ms avg（N=7 overhead 7×）|
| 7.2-bis 表 ror 行 | line 425 | 82.9% → 76.0%（-10 entries）| 应更新为 82.9% → 74.7%（共享子集 -12 entries，多 2 个 thread_local）|
| §7.3 kani-limit | line 436 | "kani-limit 7 条在 kani 上 7/7" | 实际 6/7（与 §3-C / §7.7 / cc-reports/kani.md 全矛盾）|

ror-gate6-fix §7 line 162-163 已明示"按任务'下一份系统报告统一纳入'约定推后"——但读者拿到 internal-roundup 本身看不到反向标注。

**audit H-4 准确，且发现了 §7.3 与 §3 互打的内部矛盾（H-2）—— internal-roundup 不仅落后于 P16，还存在 P13 改造前的老结论残留。**

---

## §7 audit-4 整体质量评级

| 维度 | 评价 |
|---|---|
| 引用准确性 | 高。15 条引用行号 / 数字基本准确（reports H-1 是误读结构、H-7 是割裂上下文，其余引用准确）|
| 证据扎实度 | 中-高。reports H-2 / H-3 / H-4 / H-5 / H-8 + fixes H-3 / H-4 都有清晰证据；research H-1 / H-3 严重度判断偏重 |
| 自我一致性 | 中。audit §4 表格里把 H-6 / M-3 / M-4 / L-2 自归"非决策点"——内部一致；但 reports H-1 / H-7 在 §3 写"高严重度"却被本验证驳，severity 标定欠校准 |
| disprove-first | 弱。audit 习惯性把"未旁注"/"未交叉链接"判高严重度，但部分案例文档已用同份不同段落 / 同 bullet 起首词锚定 ——audit 未做"是否上下文 / 跨段落已包含语义"的反向检查 |

**评级：高质量但严苛度过度（≈ B+）**。

audit-4 的强项是 reports H-2 / H-3 / H-4 / H-5 / H-8（5 条客观数据问题，全部成立）+ fixes H-3 / H-4（脉络 + 跟进缺失，成立）。这 7 条是本 audit 的真实价值。

弱项是 reports H-1 / H-7 的过度严苛（2 条被驳）+ research H-1 / H-3 的严重度夸大（2 条降为部分持）。

最终分布：**持 8 / 部分持 5 / 驳 2 / 存疑 0**（15 条）。
