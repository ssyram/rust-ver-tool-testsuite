# 审计报告 — 报告层（cc-reports + internal-roundup + test-reports）

## §1 问题意识

报告层（cc-reports / internal-roundup / test-reports）核心责任是 `principles.md` §三-3-原则3 + `tool-integration.md` §七：真实性诚实、数据锚定完整、非评判性、时效性。恶意角度审：(i) 报告间数字/锚定/引用是否互相矛盾；(ii) §7.1 "约束对象明示"是否被误读为"克制评判→报告无认知"；(iii) 过时数据是否标 deprecated；(iv) 失败归类是否引用 raw；(v) 怂措辞是否在正文反复。

## §2 审查方法

26 份文档：(1) 按 run id 分组检锚定一致性；(2) 抽样 5 份 cc-report（kani / verifast / rocq-of-rust / hax-coq / aeneas-coq）精读 0误报/0漏报自陈与 oracle 对齐；(3) 比 internal-roundup §3 vs cc-report 通过率；(4) 看 v3 系统报告与 P15-impl + P16 对齐；(5) 跨文档引用准确性。

## §3 审查现象

### 3.1 高严重度（H）— cc-reports 之间互相矛盾的事实陈述

**H-1：rocq-of-rust v4 通过率 vs rocq-of-rust-typecheck 档 0 baseline 互打**

`deep-reports/cc-reports/rocq-of-rust.md:9`：「v4 共享 146 子集：109/146 = 74.7%」
`deep-reports/cc-reports/rocq-of-rust-typecheck.md:10`：「档 0 同 corpus 110/146 = 75.3%」

→ 同 corpus 同档 0，两份报告差 1 个 entry。根因：rocq-of-rust-typecheck run id `run-1778473345-64581` 用早期 single-attempt baseline，而 rocq-of-rust v4 用 `run-1778480785-20112` 的 N=7 wrapper。typecheck 报告没在第 10 行旁注"该 baseline 用早期 single-attempt wrapper"或"与 cc-reports/rocq-of-rust.md v4 不一致"。

**H-2：internal-roundup §7.3 "kani-limit 7/7" vs cc-reports/kani.md "6/7" 同份文件内部矛盾**

`internal-roundup-2026-05-08.md:436`（§7.3）：「kani-limit 7 条在 kani 上 7/7（kani --only-codegen 切割点早于 kani 自陈的 unsupported）」
但同份第 39 行（暴论 4）+ 第 103 行（§3-C 表）+ cc-reports/kani.md:53-64 都说 kani-limit 6/7。§7.3 是 P13 改造前的老结论未更新。

**H-3：cc-reports/rocq-of-rust.md 内部 N=7 vs "3 次 AND-reduce" 自相矛盾**

第 35 / 48 / 109-110 / 114 行：N=7、catch rate 99.84% = 1−0.4^7
第 129 行：「124 SUCCESS 上 N-attempt（3 次）AND-reduce ... entry_fn 都正确生成 Definition **三次**」
第 139 行：「6 道门 + N-attempt wrapper oracle（N=7 ... + **3 次产物 AND-reduce**）」
第 143 行：「N=7 ... 把 P(漏 fn)^3 推到几乎 0」← "^3" 与第 114 行 "0.4^7" 直接打脸

`docs/fixes/ror-gate6-fix-2026-05-11.md:56,78,80` 明确 N=7。所以正确数字 7，cc-report 中 "3 次" 是写作错误。

### 3.2 高严重度（H）— internal-roundup 落后于 P16 数据

**H-4：未反映 v4（rocq-of-rust corpus 扩展至 161 + N=7 wrapper）**

- 头部第 3 行只提"triple-run"，未提 v4 / quad-run
- 第 10 行 corpus 仍 "146 entries × 19 = 2774"，P16 已升 161
- 第 158/170 行 rocq-of-rust 76.0%（111/146）—— v4 已是 109/146=74.7%
- 第 384 行时长表 rocq-of-rust 76 ms avg —— v4 是 477 ms（N=7 overhead）
- 第 425 行 7.2-bis 表 rocq-of-rust「-10 entries」—— v4 应是 -12（+thread_local 类 2 条）

**H-5：v3 测试报告（最新一份）也落后于 P15-impl + P16**

`docs/test-reports/feature-coverage-2026-05-11-strict-oracle-v3.md`:
- 第 97 / 134 / 269 行 rocq-of-rust 仍 111/146 = 76%
- 第 50 行只列 strict-oracle-v2，未列 v4 N=7 wrapper

`docs/fixes/ror-gate6-fix-2026-05-11.md:161-163` 已说"下一份系统报告统一纳入"，但 v4 系统报告尚未生成，读者拿 v3 不会知道它已被超越。

**H-6：v3 测试报告 `runnable/*` 完全没出现**

P16 加 15 个 `runnable/*` entry，corpus 161。v3 报告所跑的 run `1778466265-63960` 等都是 146 entry 口径，**口径上正确**——但同 commit 已经存在 161 entry 的 P16 corpus，报告没声明"corpus 现状 vs 报告 corpus 的差异"。

### 3.3 高严重度（H）— cc-report 与 oracle 实施 / 同 commit 工具不对齐

**H-7：cc-reports/kani.md 「0 个 hard-reject」表述含混**

第 162 行：「10 个 FAILED 中 ... **0 个在 hard-reject 层面**」
但同份 §"0 漏报"第 45-46 行明承认还有 60+ entry 含 `caller_location` warning 未封堵。"0 hard-reject" 实际上是 corpus 没触发 kani 的 hard-reject 路径——这是 corpus 事实而非工具事实。表述应改为"本 corpus 上 0 个 hard-reject 触发"。

**H-8：cc-reports/hax-lean.md 未应用 P13 entry_fn gate（漏审）**

第 5 行 run = `run-1778226613-5282`，第 8 行 110/146=75.3%。oracle 仅 v1 旧 sentinel sorry grep（第 32-36 行）。**P13-A 同 commit hax 30949eb 在 hax-fstar/hax-coq 上加 entry_fn gate 各抓 2 条** —— 但 hax-lean 端 `lean_backend.ml` 同型号 `NotImplementedYet` silent-skip-item 路径**没做对等审计**。internal-roundup 第 168 行只说 P13-A 给"hax-fstar / hax-coq 各加 gate"，完全不提 hax-lean——这是漏审。

### 3.4 中严重度（M）

**M-1：internal-roundup §"19 工具版本表" kmir 缺 nightly pin**

第 20 行 kmir 行只列 `mir-semantics 84bea09 + stable-mir-json 62a239d7 + K 7.1.282`，无 nightly toolchain。cc-reports/kmir.md:107 + v3 测试报告:49 都列 `nightly-2024-11-29`。读者从 internal-roundup 读不到。

**M-2：cc-reports/charon-poly.md §"与 charon-mono 的差异" 漏列第 5 条**

第 104-114 行只列 4 条 entry 差异。但同份第 115 行 + cc-reports/charon-mono.md:100-101 + internal-roundup §3-E 第 145-153 行表都明确"对称差 5 个 entry"——遗漏 `creusot-limit/dyn-trait-forbidden/trigger_call_dyn_display`。

**M-3：aeneas-fstar avg 1ms 差**

cc-reports/aeneas-fstar.md:9 写 avg 3984；internal-roundup §6 第 388 行写 3985。浮点四舍五入差，可忽略。

**M-4：怂措辞 vs §7.1**

实测 cc-reports 都按 §7.3"必含时效声明"加"不构成长期承诺"。但符合规范——未发现正文反复"不评判 / 不构成"。**M-4 整体合规**。

**M-5："audit §3.2" 引用模糊**

cc-reports/rocq-of-rust.md:58 + hax-fstar.md:61 都用 "audit §3.2" 没标 audit-1 还是 audit-2。audit-1 §3.2 是 rocq-of-rust；audit-2 §3.2 是 hax-fstar。建议明示 "audit-1 §3.2" / "audit-2 §3.2"。

**M-6：v3 测试报告 §十一附录引用缺新 implementation log**

v3 报告第 379 行只引用 P12 的 implementation log。P15-impl / P16 引入的 `rocq-of-rust-typecheck-implementation-2026-05-11.md` + `ror-gate6-fix-2026-05-11.md` + `hax-lean-eval-corpus-baseline.md` 都没列入。

### 3.5 低严重度（L）

**L-1：历史报告无"已被 vN 取代"标注**

`feature-coverage-2026-05-07.md` 最早一份没"我已被取代"反向标注。`feature-coverage-2026-05-07-fixed-harness.md:9-13` 与 `feature-coverage-2026-05-08-19tools-strict-oracle.md:42-49` 做了相邻对比，但同样没自标"已被 vN 取代"。

**L-2：cc-reports/rocq-of-rust.md "第三案例" 编号问题**

第 146 行："审计启发实施暴露漏报第三案例（前两：P12 verifast / P13 hax-fstar 漏 mutual-rec）"。但 P13 hax-fstar 是"audit pattern 被 falsify"而非"漏报案例"。若按 audit pattern falsify 算，rocq-of-rust 是第 4 次（P12 verifast / P13 hax-fstar / P13 hax-coq / P15 rocq-of-rust）。

## §4 决策点 vs 非决策点

| 项 | 类型 | 理由 |
|---|---|---|
| H-1, H-2, H-3, H-4, H-5, H-7, H-8 | 决策点 | 内部矛盾、数据严重过时、漏审，需要修订 |
| M-1, M-2, M-5, M-6 | 决策点 | 锚定 / 数据 / 引用准确性 |
| H-6, M-3, M-4, L-2 | 非决策点 | 表述差异 / 口径正确 / 整体合规 |
| L-1 | 决策点（低）| 推荐加但不强求 |

## §5 结论

**cc-reports（20 份）**：质量较高，锚定完整、归类基于 raw。最严重 3 处：H-1+H-3（ror v4 与 typecheck 数字不一致 + N 自打脸）、H-7（kani "0 hard-reject" 措辞误导）、H-8（hax-lean 漏审）。

**internal-roundup**：严重落后于 P16 数据。corpus / rocq-of-rust 数字 / 时长表都需要 v4 系统报告更新。§7.3 自身与 §3 矛盾必须改。

**test-reports（6 份）**：历史进化追溯清晰；v3 已被 P15-impl + P16 部分超越，建议出 v4 系统报告。

**高严重度问题：8（H-1 至 H-8）/ 中：6 / 低：2**。
