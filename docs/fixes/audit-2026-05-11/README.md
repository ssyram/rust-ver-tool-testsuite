# 综合审计 `audit-2026-05-11` — 总览 + 跨层级决策汇总

## §0 元数据

- **审计 ID**：`audit-2026-05-11`
- **审计起止**：2026-05-11
- **方法论**：`/principle-derivation-v2`（问题意识 → 审查方法 → 审查现象 → 决策点 vs 非决策点 → 结论）
- **审查角度**：恶意视角找违反 + 诚实有源（每条问题给文件路径 + 行号 + 引用）
- **基线 commit**：P16 `1cd223c`（git log 已记 P11 → P16 演化）
- **报告文件总数**：49 份 review + 本 README

## §1 问题意识（本审计为何）

项目从 P11 治理定调起经 P12-P16 多轮快速迭代（oracle 漏报封堵两轮 / ror 档 1 自动化新工具 / hax-lean 一致性 corpus / runnable schema 扩展）。**6 轮 commit 在 11 天内累积**，工程化质量风险积累：

- 宪法 / 设计原则 → 实施可能逐层偏移
- 各份文档（49 份 review 之外的源文档）之间可能互相矛盾
- 数据 / 数字 / 锚定可能滞后于实际 commit
- audit 推荐已被 4 次实测 falsify，反映"design → 实施"间存在系统性 gap
- 多次"P15 反向暴露 P12 漏报"暗示决策可能未充分论证

用户原话："感觉事情有点超出我原来的考虑了" → 要求"完整宪法 + 法律审查 / 各层级 / 用恶意角度找违反"。

## §2 文件索引

### §2.1 顶层审查（9 份）

| 层级 | 文件 | 审查范围 |
|---|---|---|
| 宪法 | [`principles-review.md`](principles-review.md) | `docs/design/principles.md` |
| 架构 | [`architecture-review.md`](architecture-review.md) | `docs/design/architecture.md` |
| 工具集成原则 | [`tool-integration-review.md`](tool-integration-review.md) | `docs/design/tool-integration.md` |
| 函数级细化 | [`detailed-design-review.md`](detailed-design-review.md) | `docs/design/detailed-design.md` |
| examples | [`examples-review.md`](examples-review.md) | 36 类目 / 161 entry |
| 报告 | [`reports-review.md`](reports-review.md) | 20 cc-reports + internal-roundup + 6 test-reports |
| 调研 | [`docs-research-review.md`](docs-research-review.md) | 3 份 research 文档 |
| 修复笔记 | [`docs-fixes-review.md`](docs-fixes-review.md) | 8 份 fixes 文档（非本目录下） |
| runner 实现 | [`runner-implementation-review.md`](runner-implementation-review.md) | runner/src/{main,discover,exec,host,report}.rs |

### §2.2 工具审查（40 份）

每个工具下 `config-review.md`（tool.toml + wrapper + harness）+ `readme-review.md`：

- **翻译类（11 工具）**：[`aeneas-coq`](tools/aeneas-coq/) / [`aeneas-fstar`](tools/aeneas-fstar/) / [`aeneas-hol4`](tools/aeneas-hol4/) / [`aeneas-lean`](tools/aeneas-lean/) / [`charon-mono`](tools/charon-mono/) / [`charon-poly`](tools/charon-poly/) / [`hax-coq`](tools/hax-coq/) / [`hax-fstar`](tools/hax-fstar/) / [`hax-lean`](tools/hax-lean/) / [`rocq-of-rust`](tools/rocq-of-rust/) / [`rocq-of-rust-typecheck`](tools/rocq-of-rust-typecheck/)
- **求解类（6 工具）**：[`kani`](tools/kani/) / [`verus`](tools/verus/) / [`creusot`](tools/creusot/) / [`prusti`](tools/prusti/) / [`verifast`](tools/verifast/) / [`soteria`](tools/soteria/)
- **执行/baseline（3 工具）**：[`miri`](tools/miri/) / [`kmir`](tools/kmir/) / [`cargo-check`](tools/cargo-check/)

## §3 问题统计

| 层级 | 文件数 | 总问题 | 高严重度 | 中 | 低 | 决策点 |
|---|---:|---:|---:|---:|---:|---:|
| 宪法+设计原则 (Audit-1) | 4 | 47 | 13 | 19 | 14 | 10 |
| 20 工具 (Audit-2) | 40 | ~130 | ~15 | ~50 | ~65 | ~30 |
| examples (Audit-3) | 1 | 40+ | 3 | ~15 | ~20 | 14 |
| reports+research+fixes (Audit-4) | 3 | 35 | 15 | 13 | 7 | 25 |
| runner 实现 (Audit-5) | 1 | 40 | 5 | ~20 | ~15 | 14 |
| **总计** | **49** | **~290+** | **~51** | **~117** | **~120** | **~93** |

## §4 跨层级关键发现（按优先级）

### §4.1 跨文档同步漂移（最高优先）

**F-1 / "19 工具"硬编码漂移到 20 工具未追**

- **现象**：`principles.md` + `architecture.md` + `detailed-design.md` 多处写"19 工具"，但 P15-impl 引入 `rocq-of-rust-typecheck` 已是第 20 个 tool。
- **来源**：principles-review / architecture-review / detailed-design-review 都点名
- **影响**：3 份宪法层 / 架构层文档同步漂移；读者从 design 看到"19 工具"以为 corpus 是 19 工具，实际 ls tools/ 显示 20 个
- **决策性**：决策点（修宪议案 A 候选）

**F-2 / rocq-of-rust 道门数自相矛盾**

- **现象**：principles.md L95 写 6 道门，tool-integration.md L30 写 5 道门，wrapper 实际 6 道门
- **来源**：principles-review 跨文档 §4
- **影响**：design 主线说不清 ror 的实际 oracle 形态
- **决策性**：非决策点（局部 fix，统一为 6）

**F-3 / P11-P16 实施物在 design 主线大段缺失**

- **现象**：`${TS_*}` 替换机制 / wrapper.sh 模式（charon/aeneas/verifast/prusti/kani/ror/ror-typecheck 都用）/ TS_ENTRY_FN 子进程注入 / runnable corpus `[runnable.*]` 段 / ror-typecheck 第 20 工具 — 这些已实施但 principles / architecture / tool-integration 三份主线 design 都未明示
- **来源**：principles-review #X1 + architecture-review #1 + detailed-design-review #1
- **影响**：design ↔ 实施严重脱节，新人读 design 无法理解 P11-P16 的核心模式
- **决策性**：决策点（修宪议案 A）

### §4.2 数据严重过时

**F-4 / internal-roundup 落后于 P16 数据**

- **现象**：corpus 仍 146 (P16 已 161) / rocq-of-rust 仍 76% (v4 已 74.7%) / 时长 76 ms avg (v4 已 477 ms N=7 overhead) / 7.2-bis 表 ror -10 entries (v4 已 -12)
- **来源**：reports-review H-4
- **影响**：项目主要总览文档（internal-roundup）已不反映现状
- **决策性**：决策点（需重写 v4 系统报告 + 同步 internal-roundup）

**F-5 / v3 测试报告（最新）也落后于 P15-impl + P16**

- **现象**：v3 报告写 ror 111/146 = 76%，但 v4 已 109/146 = 74.7%
- **来源**：reports-review H-5
- **影响**：读者拿 v3 不知它已被超越；ror-gate6-fix 说"下一份系统报告统一纳入"但 v4 报告未生成
- **决策性**：决策点（产出 v4 系统报告）

### §4.3 数据内部自相矛盾

**F-6 / ror cc-report N=7 vs "3 次 AND-reduce" 自打脸**

- **现象**：同一份 `deep-reports/cc-reports/rocq-of-rust.md` 第 35/48/109/114 行写 7 次 + 0.4^7，第 129/139/143 行写 3 次 + ^3。正确数字按 `ror-gate6-fix-2026-05-11.md:56` 是 N=7
- **来源**：reports-review H-3
- **影响**：单点严重写作错误，读者信息不一致
- **决策性**：非决策点（直接改 cc-report 文本）

**F-7 / internal-roundup §7.3 自身与 §3 互打**

- **现象**：§7.3 写 "kani-limit 7/7"，但同份 §3 / 暴论 4 / cc-reports/kani.md 都说 6/7。§7.3 是 P13 改造前老结论未更新
- **来源**：reports-review H-2
- **决策性**：非决策点

**F-8 / cc-reports/rocq-of-rust v4 vs rocq-of-rust-typecheck 档 0 baseline 差 1 entry 未说明**

- **现象**：cc-reports/rocq-of-rust.md:9 写 109/146 (v4 N=7)，cc-reports/rocq-of-rust-typecheck.md:10 写 110/146 (档 0 same corpus)。差 1 entry 因 typecheck 用早期 single-attempt baseline，未旁注
- **来源**：reports-review H-1
- **决策性**：非决策点（加交叉链接说明）

### §4.4 漏审 / 工具间不对称

**F-9 / hax-lean silent-skip-item 路径漏审**

- **现象**：P13-A 只给 hax-fstar / hax-coq 加 entry_fn gate，hax-lean 同型号 `lean_backend.ml NotImplementedYet` 路径未做对等审计。audit-1 §3.5 hax-lean 中风险至今未跟进
- **来源**：reports-review H-8 + docs-fixes-review H-4（互相印证）
- **影响**：可能漏报理论窗口仍在 hax-lean 上
- **决策性**：决策点（启动 P17 hax-lean entry_fn gate 审计 + 实施）

**F-10 / hax-coq / verifast / verus README 内部矛盾**

- **现象**：3 个工具 README 内部各有矛盾（hax-coq L22 写"oracle 不抓"但 L26 + tool.toml 实际抓 / verifast L106 仍说"plain Rust 预期 SUCCESS"但 vacuous pass 已抓 / verus L88 vs L23 互打）
- **来源**：Audit-2 Top 5 #2/#3/#4
- **决策性**：非决策点（直接修 README）

### §4.5 ror 反向暴露的修正脉络不完整

**F-11 / ror-gate6-fix 与 rocq-of-rust-typecheck-implementation 根因推断冲突**

- **现象**：rocq-of-rust-typecheck-implementation 推断"sh chain expansion"，但 ror-gate6-fix 实测是"翻译器非确定性"。前者推断被推翻但没回头加更正
- **来源**：docs-fixes-review H-3 + M-2
- **决策性**：决策点（rocq-of-rust-typecheck-implementation 加更正标注）

**F-12 / audit 推荐 4 次被 falsify 但记录不一致**

- **现象**：P12 verifast / P13 hax-fstar / P13 hax-coq / P15 ror 共 4 次。前 3 次清晰记录，P15 在 ror-gate6-fix 记录但未声明"P12-A '已封堵'自信被本 commit 实测推翻"
- **来源**：docs-fixes-review H-3
- **决策性**：决策点（fixes 链补脉络）

### §4.6 实施层 design-impl 冲突

**F-13 / runner 3 处显著 design-实施冲突**

- **现象**：
  - target_dir 子目录检查（design 要求 discover Err，实际仅 exec 阶段失败）
  - `extra_cargo_deps` toml 错（design 要求 panic，实际推迟到 exec）
  - `entries=[]` 解析成功但 0-task（缺早期 fail-fast）
- **来源**：runner-implementation-review #6/#7/#11
- **决策性**：决策点（3 项实施层小修）

**F-14 / industrial 三件套违反双轨 schema 边界**

- **现象**：principles §四强制"外部综合项目"用 `.hirusttest/` 目录轨，但 rsa-pkcs8 / sha256-digest / cert-parse 全用单文件轨（find 验证 0 个 `.hirusttest` 目录）
- **来源**：examples-review §3.4-#18
- **决策性**：决策点（迁移 industrial 到目录轨 / 或修宪松绑）

**F-15 / aeneas wrappers `set -euo pipefail` 与 `AENEAS_EXIT=$?` 冲突**

- **现象**：set -e 下 aeneas 非 0 退出立即终止脚本，`AENEAS_EXIT=$?` 不执行 → 诊断信号丢失（oracle 不漏，仅诊断质量降级）
- **来源**：Audit-2 Top 5 #1（aeneas-{coq,fstar,hol4,lean} 都受影响）
- **决策性**：非决策点（局部 fix shell 错误处理）

### §4.7 平台 / 环境硬编码

**F-16 / 多工具 macOS arm64 硬编码**

- **现象**：charon-{mono,poly} `--target aarch64-apple-darwin` / prusti `arch -x86_64` / rocq-of-rust `DYLD_LIBRARY_PATH` / kmir `/opt/homebrew/opt/openjdk/bin`
- **来源**：Audit-2 Top 5 #5 + 多份 config-review
- **影响**：Linux 用户无法跑这些工具
- **决策性**：决策点（要不要支持 Linux / Windows）

## §5 修宪议案（4 条 + 派生 1 条）

按 Audit-1 提出 + 后续审计补充：

| 议案 | 来源 | 内容 |
|---|---|---|
| **A** | principles-review #2 | 吸纳 ror-typecheck 第 20 工具 + runnable corpus + `${TS_*}` 替换机制 + wrapper.sh 模式为正式宪法条款 |
| **B** | principles-review #5 | 原则 A（examples 不被工具改变）形式定义明示作用域为"原始磁盘 example 目录的 cargo 行为" |
| **C** | architecture-review #8 | SUCCESS / FAILED / UNKNOWN 三分类是否提升到宪法层（当前隐含散落多处） |
| **D** | architecture-review #3 + detailed-design-review #4 | TS_ENTRY_FN / TS_TARGET_CRATE 作为正式 runner ↔ wrapper 接口契约（与 D 议案合并） |
| **E** | examples-review §3.4-#18 (派生) | industrial 三件套违反双轨边界 — 是松绑宪法 §四 还是迁移现有 3 个目录到目录轨 |

## §6 决策路径建议

按优先级分组。**用户决定每组的处理顺序与深度**：

### §6.1 短期修复（非决策点 / 文字层）

不需要用户拍板，agent 可批量修：

- F-6 ror cc-report N=7 / 3 次自打脸
- F-7 internal-roundup §7.3 老结论
- F-10 hax-coq / verifast / verus README 内部矛盾
- F-15 aeneas wrappers `set -e` 修
- 各份 Audit 中其他低严重度文字 fix（约 80+ 条）

### §6.2 报告同步（需新 commit）

- F-4 / F-5 重写 v4 系统报告 + 同步 internal-roundup
- F-8 cc-reports 之间加交叉链接

### §6.3 实施层 fix（runner / wrapper）

- F-13 runner 3 处 design-impl 冲突
- F-9 hax-lean silent-skip-item 路径审计 + 落地 P17
- F-11 / F-12 fixes 链补脉络

### §6.4 修宪议案审议

A / B / C / D / E 5 条 — 需要用户逐一拍板。建议先审 A（吸纳 P11-P16 实施物）和 E（industrial 双轨违反），其他可后续。

### §6.5 长期项

- F-16 多平台支持（macOS arm64 硬编码）— 影响项目可扩展性
- runnable corpus 与现有工具的"机械稀释"问题（P16 commit 已记 known issue）

## §7 整体评价

项目工程化质量**良好**（runner 代码 90% 兑现 design / cc-report 调性合规 / 反误报双向实测格式总体一致 / 派生原则 C 异质性归配置完整兑现）。

**主要问题在"实施快于文档"**：
- 5 轮 audit 推荐被实测 falsify 暗示"design → 实施 → 反向更新 design"链需更系统化
- P11-P16 在 11 天内迭代过快，3 份主线 design 文档未及时追实施

**最严重风险**：F-1 / F-3 / F-4 / F-5 / F-9（5 条），需要本轮综合审计后立即处理。

**实测无 panic 风险点**（runner 16 处 unwrap/expect 全在已建立不变式下安全 / 用户输入路径都走 anyhow Err 传播）。

## §8 下一步

待用户审 §6 决策路径 + §5 修宪议案后启动 P17（或后续）实施。

---

**附录**：审计 agent 完成时间
- Audit-5 (runner) ~7 min
- Audit-3 (examples) ~10 min
- Audit-4 (reports/research/fixes) ~10 min
- Audit-1 (design 4 份) ~15 min
- Audit-2 (20 工具 × 2) ~30 min

5 agent 并行后台跑，总 wall ~30 min。
