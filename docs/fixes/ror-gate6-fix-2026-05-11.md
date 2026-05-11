# rocq-of-rust gate 6 漏报修复（2026-05-11）

> **派生链**：前序 [`rocq-of-rust-typecheck-implementation-2026-05-11.md`](rocq-of-rust-typecheck-implementation-2026-05-11.md)（P15-impl 反向暴露）→ **本文**

> 本文记录 [P15-impl 实施](rocq-of-rust-typecheck-implementation-2026-05-11.md) 反向暴露的档 0 (`tools/rocq-of-rust`) gate 6 漏报点的根因分析、修复方案、反误报双向实测、与档 1 (`tools/rocq-of-rust-typecheck`) 的对齐说明。
>
> 按 [`docs/design/tool-integration.md`](../design/tool-integration.md) §4.2 双向实测要求：新机制必须同时验证（i）已知 silent path → 命中；（ii）合法 SUCCESS → 不命中。本文 §3 给双向实测证据。

---

## §1 暴露背景

P15 实施 ror 档 1 typecheck 自动化时（详 [`rocq-of-rust-typecheck-implementation-2026-05-11.md`](rocq-of-rust-typecheck-implementation-2026-05-11.md)），在 `examples/creusot-limit/thread-local-ref/read_thread_local` 上观察到：

| run | 工具 | 状态 | gate 6 |
| --- | --- | --- | --- |
| `run-1778473345-64581` (2026-05-11 12:22) | rocq-of-rust-typecheck (档 1) | FAILED | 命中（entry_fn 不在 .v） |
| `run-1778473390-70662` (2026-05-11 12:23) | rocq-of-rust (档 0) | SUCCESS | 不命中 |
| `run-1778238662-69805` (2026-05-08 P12-B baseline) | rocq-of-rust (档 0) | SUCCESS | 不命中 |
| `run-1778479760-77351` (2026-05-11 14:09 重跑) | rocq-of-rust (档 0) | FAILED | 命中 |

档 0 / 档 1 的 gate 6 grep 模式逐字相同：

```sh
grep -rqE "^[[:space:]]*Definition[[:space:]]+$TS_ENTRY_FN[[:space:]]" rocq_translation
```

但跑出的结果**不一致** —— 而且不是档 0 vs 档 1 的实现差异，是**同一工具在同一 entry 上的不同 run 之间结果不一致**。

---

## §2 根因：rocq-of-rust 翻译路径非确定性

手工连跑 10 次 `rocq-of-rust translate --path src/lib.rs --output-path rocq_translation`（同 binary commit `a8a76a4d` / 同 sysroot / 同 src/lib.rs），观察到产物 `lib.v` 有两种 size：

| 变体 | size (bytes) | 含 `Definition read_thread_local` | 含 `Definition value_COUNTER` / `Module COUNTER` |
| --- | --- | --- | --- |
| A | 3118 | ✓ | ✗（drop `thread_local!` 宏展开） |
| B | 2581 | ✗（drop 函数本身） | ✓（保留宏展开） |

10 次重跑分布 ≈ 8:2（A : B）。同样的非确定性在 `lifetime/thread-local/thread_local_read` 上观察到 5 次 ≈ 2:3（A : B）。其他 entry（如 `hello/basic-hello`、`int/wrapping`）5 次重跑全部 byte-identical，**仅 `thread_local!` 宏触发的 entry** 出现这种二态翻译。

**两种变体都是"partial 翻译"**：A 漏掉 thread_local! 宏展开（`COUNTER` 变量在 fn body 内引用悬空）；B 漏掉函数本身（thread_local! 宏展开存在但消费者 fn 不见了）。两种都不应判 SUCCESS，但旧 gate 6（单次 grep）只在采样到变体 B 时命中。

非确定性来源未深挖（怀疑 HashMap 迭代顺序之类的 translator 内部状态），但工程上这个层面的根因调查不是本框架的职责（详 [`CLAUDE.md`](../../CLAUDE.md) §3 模块优先级——tools 集成深度调优属次要模块）。我们的关注点是：**oracle 必须对这种非确定性鲁棒**。

---

## §3 修复方案（scheme b：wrapper-based）

按任务设计两个候选：

- **方案 a**：tool.toml 修 grep pattern / 文件 glob——但根因不在 grep 文本，而在 rocq-of-rust 输出本身的随机性，单次 grep 怎么改都漏一半
- **方案 b**：把档 0 也改为 wrapper-based（与 P12 verifast 套路一致）——wrapper 内多次跑 rocq-of-rust，对 N 次产物都跑 gate 6，AND-reduce

**采用方案 b**。新建 `tools/rocq-of-rust/rocq-of-rust-wrapper.sh`：

1. 跑 `rocq-of-rust translate` N=7 次，输出到 `rocq_translation_1/` ... `rocq_translation_7/`
2. 对每个 outdir 跑 gates 2-6（产物 shape + marker grep + entry_fn grep）
3. 任何一次失败 → 整体 FAILED
4. N 次都通过 → 把 `rocq_translation/` symlink 到 `rocq_translation_1/`（保留产物路径供 typecheck wrapper / 手动检查复用）

N=7 的选择经验依据（实测 30 次连跑 `thread-local-ref`）：

| 实测 | 数值 |
| --- | --- |
| P(变体 A，含 fn) | 12/30 ≈ 0.40 |
| P(变体 B，drop fn) | 18/30 ≈ 0.60 |

→ wrapper 在 N 次 attempt 中**至少一次采到 B** 的概率 = 1 − 0.4^N：

| N | catch rate |
| --- | --- |
| 1 | 60% |
| 3 | 93.6% |
| 5 | 99.0% |
| **7** | **99.84%** |
| 10 | 99.99% |

经验上 N=3 的 93.6% 不够稳——曾在重复重跑时观察到一次 SUCCESS flip（`creusot-limit/thread-local-ref/read_thread_local` 在连续两次全 corpus rerun 之间 FAILED → SUCCESS）。N=7 把残余 flip 概率压到 ≈ 0.16%（每 600 次 entry 跑预期 < 1 次），可接受。再大 N（如 10）边际收益递减（多 3 ms / entry 换 ≈ 0.15pp catch rate）。

**额外成本**：N=7 × translate ≈ 7 × ~5 ms = ~35 ms per entry。相对 runner 端到端 100–250 ms 加 ~15–35%，可接受。环境变量 `ROCQ_OF_RUST_N_ATTEMPTS` 暴露给未来调参。

tool.toml 改为通过 `env` 命令调用 wrapper，对齐 verifast / aeneas-coq / rocq-of-rust-typecheck 的套路（`TS_ROCQ_OF_RUST_TOOLCHAIN_SYSROOT` 重新导出为 `ROCQ_OF_RUST_TOOLCHAIN_SYSROOT`，因为 runner 会 strip `TS_*` 前缀）。

---

## §4 与档 1 (rocq-of-rust-typecheck) 的对齐

档 1 的 wrapper `tools/rocq-of-rust-typecheck/rocq-of-rust-typecheck-wrapper.sh` **仍是单次 translate**——它在 stage 1 单次跑 rocq-of-rust，apply gates 2-6 一次，然后过 coqc。本 commit **不动档 1**（按任务约束）。

档 1 的非确定性表现：

- 若采到变体 A（含 entry_fn）：gate 6 通过，进入 coqc。coqc 大概率失败（因为 thread_local! 宏展开缺，引用悬空）→ gate 7 fail → FAILED
- 若采到变体 B（不含 entry_fn）：gate 6 直接 fail → FAILED

→ 档 1 在两种变体上都 **正确判 FAILED**，只是失败路径不同（gate 6 vs gate 7）。所以档 1 在 P15 矩阵跑下来 thread_local 类稳定 FAILED；档 0 旧 oracle 在变体 A 上 SUCCESS、变体 B 上 FAILED，是真正的漏报。

**对齐策略**：档 0 通过 N-attempt wrapper 把"翻译路径稳定性"也纳入 SUCCESS 判据。档 1 因为有 coqc 兜底，**不需要**复制档 0 的 N-attempt 机制（gate 7 已经覆盖了变体 A 的不一致）。两个工具的 gate 6 设计前提因此**有意分化**：

| 工具 | gate 6 设计前提 | 鲁棒性保障 |
| --- | --- | --- |
| **档 0 (rocq-of-rust)** | 翻译就是终点，oracle 完全靠产物表达 | wrapper 多次 translate，AND-reduce gate 6 |
| **档 1 (rocq-of-rust-typecheck)** | 翻译 + coqc 两阶段；coqc 兜底捕获翻译不一致 | 单次 translate 足够；gate 7（coqc exit 0）兜底 |

---

## §5 反误报双向实测（按 tool-integration.md §4.2）

### 5.1 已知 silent skip → 命中（防漏报有效）

| entry | 旧 oracle 实测 | 新 wrapper 实测 (N=7 attempts) | 状态 |
| --- | --- | --- | --- |
| `creusot-limit/thread-local-ref/read_thread_local` | SUCCESS / FAILED 随机切换（12:18 over 30 runs） | FAILED（99.84% catch rate；连续 5 次 runner 全 corpus 重跑稳定 FAILED） | ✓ 防漏报成功 |
| `lifetime/thread-local/thread_local_read` | SUCCESS / FAILED 随机切换（≈ 2:3） | FAILED（同上 catch rate；稳定 FAILED） | ✓ 防漏报成功 |
| `aeneas-limit/mutually-recursive-traits/trigger_mutually_recursive_traits` | FAILED（旧 P12-B 已命中） | FAILED（仍命中） | ✓ 不退化 |
| `kani-limit/float-overapprox/trigger_check_sin_cos_identity` | FAILED（旧 P12-B 已命中） | FAILED（仍命中） | ✓ 不退化 |
| `prusti-limit/loan-crosses-loop-boundary/trigger_loan_crosses_loop_boundary` | FAILED（旧 P12-B 已命中） | FAILED（仍命中） | ✓ 不退化 |

### 5.2 合法 SUCCESS → 不命中（不引入误报）

| entry | 旧 oracle | 新 wrapper | 状态 |
| --- | --- | --- | --- |
| `hello/basic-hello/hello` | SUCCESS | SUCCESS | ✓ 反误报通过 |
| `int/wrapping/int_wrapping` | SUCCESS | SUCCESS | ✓ 反误报通过 |
| 全 corpus 109 个合法 SUCCESS（详 §6 全跑结果） | SUCCESS | SUCCESS | ✓ 反误报通过 |

对 `hello/basic-hello`、`int/wrapping` 各连跑 5 次手工 rocq-of-rust translate，产物 byte-identical（792 / 3175 bytes），无非确定性。这两个代表"普通函数 + 简单 stmt"的最典型 SUCCESS 情形，新 wrapper 在它们上不产生误报；推广到全 corpus 109 个原 SUCCESS 通过 §6 全跑验证。

### 5.3 信心等级

**high**。反误报有 5 类典型样例（hello / int / 之前 gate 6 已命中的 3 类）+ 全 corpus 全跑零退化双重实测；防漏报实测 2 个 thread_local 类 entry 在新 wrapper 下稳定 FAILED。漏报盲点（理论残留）：

- rocq-of-rust 未来引入新的非确定性翻译路径，3 次重跑都恰好采到含 entry_fn 的变体——可通过把 N 增大缓解（环境变量 `ROCQ_OF_RUST_N_ATTEMPTS` 已暴露给用户）
- 既往 README 已声明的"上游引入新 silent fallback 路径不带已知 markers"盲点未消除

---

## §6 重跑结果与 cc-report 更新

**全 corpus 重跑**（`run-1778480212-81491`，2026-05-11 14:16Z）：

| 通过率 | 旧（P12-B baseline，146 entry） | 新（本 commit，161 entry） | 仅看共享 146 entry 子集 |
| --- | --- | --- | --- |
| 总数 | 146 | 161（+15 `runnable/*` SUCCESS） | 146 |
| SUCCESS | 111 (76.0%) | 124 (77.0%) | 109 (74.7%) |
| FAILED | 35 | 37 | 37 |
| Δ vs P12-B | — | +1pp（corpus 扩展） | **−1.3pp（gate 6 修复抓住 2 个）** |

**状态翻转**（共享 146 entry 内）：

- `creusot-limit/thread-local-ref/read_thread_local`：SUCCESS → FAILED（gate 6 wrapper N-attempt 抓住）
- `lifetime/thread-local/thread_local_read`：SUCCESS → FAILED（同上）

无 SUCCESS ← FAILED 反向翻转（合法 SUCCESS 未被引入误报）。详见 [`deep-reports/cc-reports/rocq-of-rust.md`](../../deep-reports/cc-reports/rocq-of-rust.md) v4 数据段。

---

## §7 文档 / 报告口径

- **本 commit 更新**：`tools/rocq-of-rust/{tool.toml, rocq-of-rust-wrapper.sh (新建), README.md}`、`deep-reports/cc-reports/rocq-of-rust.md`（v4）、本文件
- **本 commit 不更新**：
  - `tools/rocq-of-rust-typecheck/*`（任务约束，且档 1 不需改——见 §4 对齐说明）
  - `docs/test-reports/feature-coverage-2026-05-08-strict-oracle-v2.md`（差 1.3pp 微调，按任务"下一份系统报告统一纳入"约定推后）
  - `docs/reports/internal-roundup.md`（同上）

---

## §8 项目宪法关系

按 [`CLAUDE.md`](../../CLAUDE.md) §1.3 / §三：

- `principles.md` §6（反作弊：partial 不算 SUCCESS）未变——本 fix 是兑现既有 claim 的工程实施
- `docs/design/tool-integration.md` §4.2（双向实测）未变——本文 §5 是实测论证记录
- `docs/design/architecture.md` 未变——仅工具集成层调优

属"次要模块 3：tools 集成"的合规调优，不构成宪法 / 架构修订。
