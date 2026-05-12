# 审计报告 — fixes 层（docs/fixes/*）

## §1 问题意识

`docs/fixes/*`（除 audit-2026-05-11/ 子目录外的 8 份文档）是项目对 bug / oracle / 工具集成阶段性 fix 的"复现 / 根因 / 参考实现 / 修正方案"沉淀（按 workflow §5.1）。恶意角度：(i) 派生链是否清晰（audit → implementation → commit）；(ii) audit 推荐被实测 falsify 是否在 fixes 文档间一致记录；(iii) 是否有 fixes 已被后续推翻但没标注 outdated；(iv) "Known issues" 段是否每条都有跟进 commit；(v) 5 份 implementation log 的反误报双向实测格式 / 论证套路是否一致。

## §2 审查方法

8 份 fixes 文档按时间序：
- `extra-cargo-deps-and-entry-mode.md`（2026-05-06 / 早期）
- `oracle-leak-audit-2026-05-08.md` + `oracle-leak-rules-implementation-2026-05-08.md`（P12）
- `oracle-leak-audit-2-2026-05-11.md` + `oracle-leak-rules-implementation-2-2026-05-11.md`（P13）
- `rocq-of-rust-typecheck-implementation-2026-05-11.md`（P15-impl）
- `ror-gate6-fix-2026-05-11.md`（P15-impl 反向暴露）
- `hax-lean-eval-corpus-baseline.md`（P16-impl-A）

按"派生链 / falsify 记录 / outdated 标注 / 跟进 / 论证一致性"5 维度审。

## §3 审查现象

### 3.1 高严重度（H）

**H-1：extra-cargo-deps-and-entry-mode.md 与后续 P12/P13/P15 链接缺失**

`extra-cargo-deps-and-entry-mode.md` 是早期 fix（2026-05-06，未标 P 编号）。后续 oracle 漏报 fixes（P12+P13）以及 ror typecheck（P15）/ ror gate6 / hax-lean-eval 都没在文末加"派生关系"段。

extra-cargo-deps 引入 `entry_mode = "lib"` 是 P15 rocq-of-rust-typecheck 的 wrapper 设计基础（典型用法 `mod __ts_inner;` 顶层接管），但读者从 extra-cargo-deps 文档无法看到后续模块的引用。

**H-2：audit-1 → implementation-1 → audit-2 → implementation-2 派生链清晰但缺索引**

派生链实际上是：

```
oracle-leak-audit-2026-05-08.md (P12 audit)
  ↓
oracle-leak-rules-implementation-2026-05-08.md (P12-A impl)
  ↓ (commit P12-B 重跑)
oracle-leak-audit-2-2026-05-11.md (P13 audit)
  ↓
oracle-leak-rules-implementation-2-2026-05-11.md (P13-A impl)
  ↓ (commit P13-B 重跑)
rocq-of-rust-typecheck-implementation-2026-05-11.md (P15-impl)
  ↓ (反向暴露)
ror-gate6-fix-2026-05-11.md (P15-impl 反向修正 P12)
  ↓
hax-lean-eval-corpus-baseline.md (P16-impl-A，待 P16-impl-B)
```

链清晰，但 fixes 文档无索引：每份文件都没在头部声明"我属于 P{N}，前序是 P{N-1}，后续 P{N+1}"。读者拿到任一份都难看到全局位置。

**H-3：audit 推荐被实测 falsify 案例（4 次）在 fixes 文档间记录不完全一致**

按 `docs/test-reports/feature-coverage-2026-05-11-strict-oracle-v3.md` §六总结：

| 轮次 | 工具 | falsify | implementation 采用 |
|---|---|---|---|
| P12 | verifast | N≤40 阈值 | verbose user-file grep |
| P13 | hax-fstar | plain `let` 漏 mutual-rec `and` | `(let\s+(rec\s+)?\|and\s+)` |
| P13 | hax-coq | `(Definition\|Equations\|Fixpoint)` 漏 `Lemma` | 6-keyword 并集 |
| P15 | rocq-of-rust | 单次 wrapper 漏非确定性 silent drop | N=7 AND-reduce |

各份 fixes 文档记录这些 falsify 案例的方式不一致：

- `oracle-leak-rules-implementation-2026-05-08.md` §2.1 verifast：实施日志中标 "audit 推荐的 N≤40 阈值在 spec-bearing 最小例上 N=39 落入 spec-less 区间会误报；改用 verbose user-file grep 双向均通过"——清晰
- `oracle-leak-rules-implementation-2-2026-05-11.md` §2.2 / §2.3：hax-fstar / hax-coq falsify 也清晰记录
- `ror-gate6-fix-2026-05-11.md` §1：实际是"P15-impl 反向暴露 P12 verifast 类型问题（单次 oracle 漏非确定性）"，本质是 P15 重新审视 P12 + audit-1 §3.2 的判断。但本文 §1 / §2 没说"audit-1 §3.2 当时'实测 0 现象'被本 commit 推翻"——P15 实际推翻的就是 audit-1 + P12-A 的"已封堵"自信。

**应明示**：ror-gate6-fix-2026-05-11.md §2 / §3 应该加一段"audit-1 §3.2 + implementation-1 §2.3 当时的判定"被本 commit 实测进一步细化的脉络。

**H-4：oracle-leak-audit-2026-05-08.md "Known issues" 段后续跟进状态混乱**

audit-1 §1 TL;DR #2-5 的工具风险评估：

- `verifast`：高风险 → P12-A 已封堵
- `rocq-of-rust`：中风险 → P12-A 已封堵 → 但 P15 又发现非确定性漏报 → P15-impl 反向修补
- `hax 三个 backend`：实测 0 现象 → P13 重审 → P13-A 落地（**只针对 hax-fstar / hax-coq**，hax-lean 没动）
- `prusti`：理论窗口 → P12-A 落地

整体逻辑成立。但 **hax-lean 端的 silent-skip-item 路径 P13 没单独评估也没单独 falsify**——audit-1 §3.5 hax-lean 中风险（"实测 0 触发，理论窗口存在"）至今未被任何后续 fixes 文档跟进。Known issue 状态"无明确跟进 commit"。

### 3.2 中严重度（M）

**M-1：oracle-leak-audit-2-2026-05-11.md §3.4 prusti 剩余 entry-level 窗口"实测 0 现象，可选增强"未在后续跟进**

audit-2 §3.4 第 204-225 行：

> prusti 仍有 entry-level 漏报理论窗口：P12 的 .vpr 存在性 check 只校验 ≥ 1 个 .vpr，**不校验 entry_fn 是否在 .vpr 内**。README 已自陈"encoder 内部 silent skip 单个 fn item（不影响 .vpr 总数 ≥ 1）"为剩余盲点。**实测 0 现象**，可选增强。

→ P13 / P14 / P15 / P16 都没跟进这条。本应作为"P{N+1} 实施时检"的项，但 fixes 链没把它当作"待办"传递。

**M-2：rocq-of-rust-typecheck-implementation-2026-05-11.md §"与档 0 差异分析" 推断而非确证**

第 161-173 行（§"与档 0 差异分析"）：

> 根因推断：可能是 tools/rocq-of-rust/tool.toml 把所有 gate 串在一个 sh -c if/elif 链里，某些 corner case 下 sh expansion 与 wrapper.sh 不一致。**这是 tools/rocq-of-rust 的一个潜在 oracle 漏报**，不在本任务范围

而 `ror-gate6-fix-2026-05-11.md` §2 实际给出根因是**非确定性翻译路径**而非"sh 链 expansion"。所以 rocq-of-rust-typecheck-implementation §"差异分析"的"根因推断"被后续 ror-gate6-fix 推翻——但 rocq-of-rust-typecheck-implementation 本身没回头加"根因推断已被 ror-gate6-fix 修正"标注。

**M-3：5 份 implementation log 反误报双向实测格式略有不一致**

- `oracle-leak-rules-implementation-2026-05-08.md` §2.1（verifast）：清晰 §"防漏报论证" + §"反误报论证" + §"端到端 runner 验证" + §"信心等级" + §"阈值 / 路径校准"五段
- `oracle-leak-rules-implementation-2-2026-05-11.md` §2.1（kani）：清晰 §"防漏报论证" + §"反误报论证" + §"信心等级" + §"待主进程实测验证项" 四段
- `rocq-of-rust-typecheck-implementation-2026-05-11.md` §"反误报双向实测"：分 §"正向（已知会 typecheck 通过）" / §"反向（已知会 fail）" / §"gate 7-9 反向（构造档 1-specific 失败）"——格式与前面两份略不同
- `ror-gate6-fix-2026-05-11.md` §5（反误报双向实测）：清晰 §5.1 / §5.2 / §5.3 信心等级
- `hax-lean-eval-corpus-baseline.md` §2（P16-impl-B 反误报双向实测起点）：作为 baseline 文档，目前只列"已知 SUCCESS 一致性"基准，等 P16-impl-B 时再做双向

5 份格式都做了双向实测论证，**信息完整**，但章节标题与 §结构略有不同。可统一化但非强制。

**M-4：hax-lean-eval-corpus-baseline.md §5 "本机当前实测，未达预期" 是 known issue 但未指向后续跟进 commit**

第 81-87 行：

> P16-impl-A 跑 runner --tool hax-lean --entry 'runnable/*' 实测：**0 SUCCESS / 15 FAILED**（2026-05-11 本机）。
> 根因：本机 TS_HAX_ENGINE_BIN=${OPAM_DEFAULT_PREFIX}/bin/hax-engine 当前不可达——rust-engine/src/ocaml_engine.rs:130 内部 spawn 找不到子组件 ...
> 这是已知的 hax 工具链本地环境配置问题，不属本任务（P16-impl-A：corpus）范围。P16-impl-B 实施前需先恢复 hax-engine 子组件路径——这条记入 hax-lean-eval tool README 的安装步骤。

这是"P16-impl-B 已知 blocker"。但本 baseline 文档没说"如该 blocker 在 P16-impl-B 仍未解，应如何处理"。建议在 §6 后续工作段加"如 hax-engine 不可恢复，需评估替换方案 / 暂缓 P16-impl-B"。

### 3.3 低严重度（L）

**L-1：5 份 implementation log 都没在文末统一加 "下一阶段 commit / 待办" 索引**

每份 implementation log 在末尾都给"等 cc-report 主进程重跑"或"等 P{N+1}"，但**没有正式 TODO 索引**。读者扫文档时需要全文搜"TODO"或"待办"。

**L-2：ror-gate6-fix-2026-05-11.md §7 "本 commit 不更新"清单有用但格式可统一**

第 159-163 行：

> 本 commit 不更新：
>   - tools/rocq-of-rust-typecheck/*（任务约束，且档 1 不需改——见 §4 对齐说明）
>   - docs/test-reports/feature-coverage-2026-05-08-strict-oracle-v2.md（差 1.3pp 微调，按任务"下一份系统报告统一纳入"约定推后）
>   - docs/reports/internal-roundup.md（同上）

格式好（明确不动什么 / 为何不动）。但其他 implementation log 没这种段，建议统一加 "本 commit 范围 vs 不范围" 段。

注：路径 `docs/reports/internal-roundup.md` 是笔误—— 实际路径是 `deep-reports/internal-roundup-2026-05-08.md`。

**L-3：oracle-leak-rules-implementation-2-2026-05-11.md §0 "与第一轮的关系" 表格 prusti 行错误**

第 22 行（§0 表）：

> | 第一轮（P12）| 第二轮（P13）|
> | 漏报机制 | A 类（spec-less skip：verifast）+ D 类（产物 check 未实施：prusti）+ B 类（单 item silent skip：rocq-of-rust）| C1（codegen 完成 + unsupported stub：kani）+ C2（backend silent-skip-item：hax-fstar/coq）|

把 prusti 的漏报机制称 "D 类（产物 check 未实施）"——但 audit-1 §3.6 实际把 prusti 归为 D 类**理论窗口**（实测 0 现象，是防御性兜底）。"未实施"措辞有歧义（README 写过文字层但 oracle 没落地）。

## §4 决策点 vs 非决策点

| 项 | 类型 | 理由 |
|---|---|---|
| H-1 | 决策点（低）| 早期 fix 与后续模块缺索引，可加可不加 |
| H-2 | 决策点 | 派生链文档化能改善读者体验；建议每份 fix 头部加"前序 / 后续"索引 |
| H-3 | 决策点 | ror-gate6-fix-2026-05-11.md §2-§3 应明示反向修正 audit-1 + P12-A |
| H-4 | 决策点 | hax-lean silent-skip-item 路径需评估补做 P14（与 reports-review.md H-8 重叠） |
| M-1 | 决策点（低）| prusti 剩余 entry-level 窗口"实测 0 现象"未跟进，但实测 0 即可不补 |
| M-2 | 决策点 | rocq-of-rust-typecheck-implementation §"根因推断"被推翻，应加更正标注 |
| M-3 | 非决策点 | 5 份 implementation log 格式略不同，信息完整，可统一可不统一 |
| M-4 | 决策点 | P16-impl-A baseline 文档应加 blocker 应对方案 |
| L-1, L-2 | 决策点（低）| 文档习惯，易改 |
| L-3 | 决策点 | ror-gate6-fix-2026-05-11.md §7 笔误（"docs/reports/internal-roundup.md"）应改 |

## §5 结论

8 份 fixes 文档完整、链路清晰、反误报双向实测格式总体一致（见 M-3）、与设计宪法（principles.md / tool-integration.md）对齐良好。

**最严重 3 处**：
1. **H-3 + M-2**：ror-gate6-fix-2026-05-11.md 反向修正 P12-A + rocq-of-rust-typecheck-implementation §"根因推断"被推翻——两份相邻 fixes 之间的修正关系应明示
2. **H-4**：hax-lean 端 silent-skip-item 路径 P13 没单独评估，audit-1 §3.5 至今未被任何后续 fixes 跟进（与 reports-review.md H-8 重叠）
3. **M-4 + H-1**：P16-impl-A baseline 文档有 known blocker（hax-engine 不可达）但没说应对方案；early fix extra-cargo-deps 与后续模块缺索引

**audit 推荐被实测 falsify 案例（共 4 次）**：
- P12 verifast（N≤40 阈值）：记录清晰
- P13 hax-fstar（plain `let` 漏 mutual-rec）：记录清晰
- P13 hax-coq（pattern 漏 Lemma）：记录清晰
- P15 rocq-of-rust（单次 wrapper 漏非确定性）：记录在 ror-gate6-fix-2026-05-11.md，但未声明这是"P12-A 的反向修正"——脉络不完整

**Known issues 跟进**：
- audit-1 §3.5 hax-lean："实测 0 触发" → 未跟进
- audit-2 §3.4 prusti entry-level："实测 0 现象，可选增强" → 未跟进
- hax-lean-eval-corpus-baseline §5 hax-engine 路径不可达 → P16-impl-B 待办

**高严重度：4（H-1 至 H-4）/ 中：4（M-1 至 M-4）/ 低：3（L-1 至 L-3）**。
