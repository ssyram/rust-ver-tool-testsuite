# v6 综合报告（2026-05-12，宪法修订 + DP-4 严格化 + 漏报封堵 + 闭环 audit）

> **状态**：本报告记录 2026-05-12 v6 落定快照。本周期完成：宪法 §一 升级为双根本问题（不公平 + 不公信）、UNKNOWN 严格语义入 §六、删 3 条工具能力边界 oracle 规则、加 4 条 wrapper silent-partial 封堵 gate、verus /tmp 环境治源迁移。
>
> **不构成工具能力客观评判**——按 [`README.md`](../../README.md) 顶部免责。所有数字锚定 2026-05-12 工具版本快照与 v6 corpus（161 entries / 20 工具）。
>
> **建议阅读路径**：5 分钟扫读 §1 TL;DR + §2 修宪 + §6 关键发现；30 分钟拍板路径加 §4 误报状态 + §5 漏报状态。

---

## §0 元数据

### 0.1 数据来源

| 阶段 | run id | ISO 时间 | 用途 |
|---|---|---|---|
| v5.1 baseline | `run-1778504159-67797` | 2026-05-11T12:55Z | 上一代 baseline（对比基准）|
| **v6 主跑** | `run-1778560393-59119` | 2026-05-12T12:33Z（19 tools）+ verus 12:58Z 单跑 merge | 本报告核心数字 |
| v6 verus rerun（合并入主跑）| `run-1778561896-25488` | 2026-05-12T12:58Z | verus 路径修复后重跑，已合并入主跑 results.json |

### 0.2 corpus 与工具版本

- corpus：161 entries × 41 features（与 v5.1 同分母）
- 任务总数：161 × 20 = 3220 task
- 工具版本基本同 v5.1，唯一变化：verus 0.2026.05.03.8b81855（v5.1 同版本，本次只是修复 binary 路径）

### 0.3 修宪 + Oracle 改动总结（P27 / commit `28b4a03`）

宪法 [`docs/design/principles.md`](../design/principles.md) 修订：

- §一 升级为**双根本问题**：(1) 不公平（原）+ (2) 不公信（新加，含本地性 / 社区惯例 / 最大善意三原则栈）
- §二 不可妥协价值扩 4 条（加"测量姿势对社区惯例 + 本地性对齐"）
- §六 Oracle 责任加 **UNKNOWN 严格语义**：仅两类——(a) 全局工具链崩溃 (b) 我们这边可识别问题且暂未修

Oracle 实现（`runner/src/report.rs::classify_external_fault`）：

| 旧规则（v5.1）| v6 状态 | 理由 |
|---|---|---|
| 1. runnable_harness_arg_mismatch | **保留** | 我们 harness 模板 bug → (b) UNKNOWN |
| 2. dependency_resolution (E0432/E0433) | **删** | 工具单文件 pipeline 不读 deps = 能力边界 → FAILED |
| 3. toolchain_edition_mismatch | **删** | 工具自选老 toolchain = 能力边界 → FAILED |
| 4. vendor_lint_strictness | **保留** | 我们 corpus 引入的 vendored crate → (b) UNKNOWN |
| 5. environment_corruption (JVM) | **保留** | 我们环境损坏 → (b) UNKNOWN |
| 6. edition_pipeline_propagation (E0670) | **删** | 官方 wrapper 不传 --edition = 能力边界 → FAILED |

新增 wrapper silent-partial gates（D3.1 / D3.2 / D3.3 已 R5 audit 验证）：

| gate | 工具 | 检测 |
|---|---|---|
| ror "is not yet supported" | `rocq-of-rust-wrapper.sh` | stderr grep → FAILED |
| charon "is not supported" | 4× `aeneas-*-wrapper.sh` | charon stderr grep + exit 0 → FAILED |
| charon "^error:" | 4× `aeneas-*-wrapper.sh` | charon stderr grep + exit 0 → FAILED |

---

## §1 TL;DR

- **总体通过率**：2202 SUCCESS / 1008 FAILED / 10 UNKNOWN = 68.39%（v5.1 68.85%，-0.46 个百分点）
- **修宪后 UNKNOWN 数从 131 降至 10**：删了 3 条工具能力边界规则，仅留 (a) 全局工具链崩溃 + (b) 我们这边可识别问题
- **本周期共 15 个 FAILED 新增来自 D3 wrapper gate 封堵**：8 (D3.1+D3.2+D3.3 已 R6 audit 确认 baseline 封堵) + 7 (R7 cc-route audit 新发现的 aeneas Warn 通道 partial / ror-typecheck 漏抄 tier-0 gate)
- **环境修复**：verus binary 因 `/tmp/` 自动清理失效（同 P22 prusti viper_tools 问题），治源迁移到 `~/.local/share/ts-tools/`
- **0 误报状态**：见 §4 audit 闭环
- **0 漏报状态**：见 §5 audit 闭环

---

## §2 修宪：§一 升级双根本问题

详 [`docs/design/principles.md §一`](../design/principles.md#一根本问题意识)。

### 2.1 触发动机

用户提出"不引发纠纷 / 遵循社区惯例"作为根本问题意识：

> 测量是对外可见的工具能力筛查。若测量姿势工具开发者可一句话驳回——"你姿势不对"——测量失公信力。

charter-craft 框架下分析：这一原则符合宪法防腐蚀机制——它跨时间（社区惯例变化）/ 跨主体（不同审阅者）/ 抗诱惑（不测理论极限刷数据）/ 防遗忘（不忘替用户测）。属问题意识层。

### 2.2 三原则栈（自上而下）

1. **本地性 / 当前性**（最高）：测当下这个工具版本 + 它要求的 toolchain 能做什么。装对其要求 toolchain 后仍不行 → 工具菜，FAILED 站得住
2. **遵循社区惯例**：在本地性之下，用工具开发者文档化 / 推荐姿势
3. **最大善意**：在能做的姿势内尽力配合

### 2.3 D2 wrapper 区分判据（用户原话）

> wrapper 是我们自己包装的 wrapper？那肯定不行啊，肯定是我们的问题啊。
> 对面官方的 wrapper 那就是官方的锅，我们的问题怎么能怪官方？

落地规则：

| wrapper 来源 | 失败处理 |
|---|---|
| 我们包装的（`prusti-strict-wrapper.sh` / `aeneas-*-wrapper.sh` / `rocq-of-rust-wrapper.sh`）| UNKNOWN（我们修）|
| 官方自带（`kmir/cargo.py` / charon 内置 / aeneas OCaml 本体）| FAILED（工具能力边界，本地性原则下站得住）|

---

## §3 20 工具 v6 数据（按通过率排序）

数据源：`runs/run-1778560393-59119/results.json`（verus 合并自 `run-1778561896-25488`；aeneas × 4 + ror-typecheck R7 合并自 `run-1778562673-31997`）。

| tool | n | S | F | U | rate | ΔS vs v5.1 | ΔF | ΔU |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| cargo-check | 161 | 161 | 0 | 0 | 100% | 0 | 0 | 0 |
| miri | 161 | 157 | 4 | 0 | 97% | 0 | 0 | 0 |
| charon-mono | 161 | 153 | 8 | 0 | 95% | 0 | 0 | 0 |
| charon-poly | 161 | 154 | 7 | 0 | 95% | 0 | 0 | 0 |
| kani | 161 | 151 | 8 | 2 | 93% | 0 | 0 | 0 |
| hax-fstar | 161 | 128 | 31 | 2 | 79% | 0 | 0 | 0 |
| hax-lean | 161 | 125 | 34 | 2 | 77% | 0 | 0 | 0 |
| soteria | 161 | 124 | 37 | 0 | 77% | 0 | +22 | -22 |
| rocq-of-rust-typecheck | 161 | 124 | 37 | 0 | 77% | -1 | +24 | -23 |
| rocq-of-rust | 161 | 123 | 38 | 0 | 76% | -1 | +24 | -23 |
| creusot | 161 | 121 | 40 | 0 | 75% | 0 | 0 | 0 |
| hax-coq | 161 | 111 | 48 | 2 | 68% | 0 | 0 | 0 |
| aeneas-coq | 161 | 98 | 63 | 0 | 60% | -4 | +4 | 0 |
| aeneas-fstar | 161 | 98 | 63 | 0 | 60% | -4 | +4 | 0 |
| aeneas-lean | 161 | 98 | 63 | 0 | 60% | -4 | +4 | 0 |
| prusti | 161 | 71 | 90 | 0 | 44% | 0 | +7 | -7 |
| aeneas-hol4 | 161 | 65 | 94 | 2 | 40% | -1 | +1 | 0 |
| verus | 161 | 66 | 95 | 0 | 40% | 0 | +22 | -22 |
| kmir | 161 | 61 | 100 | 0 | 37% | 0 | 0 | 0 |
| verifast | 161 | 13 | 148 | 0 | 8% | 0 | +24 | -24 |

### 3.1 数字自洽性

- ΔS 总计：-15
  - -8 来自 v5.1 → v6 baseline 的 D3.1/D3.2/D3.3 baseline 封堵
  - -7 来自 v6 baseline → v6 final 的 R7 aeneas Warn-channel + ror-typecheck D3.1 同步封堵
- ΔU 总计：-121（来自 DP-4 严格化删 3 规则）
- ΔF 总计：+136 = -ΔS -ΔU + 0（守恒）✓

注：aeneas-hol4 ΔS = -1（v5.1 66 → v6 65），但 R7 fix 在 hol4 上**不再翻**额外 SUCCESS——hol4 在涉及的 entries (mutually-recursive-traits / impl_trait_iter) 中本就因 OCaml panic FAILED，新 Warn gate 防御性加上但无翻转。

### 3.2 v6 UNKNOWN 10 个全列

| tool | entry | tag |
|---|---|---|
| aeneas-hol4 / hax-coq / hax-fstar / hax-lean / kani | `industrial/x509-parser/cert-parse/x509_parse_der` | vendor_lint_strictness |
| 同上 5 工具 | `industrial/x509-parser/cert-parse/x509_subject_extensions` | vendor_lint_strictness |

均为 vendored crate (`vendor/x509-parser`) 的 `#![deny(unused_qualifications)]` 在更严 rustc 下触发——属"我们 corpus 引入的 lint" → UNKNOWN (b) 类。修复路径：DP-10 vendor lint 修源（长期，未实施；动 vendored crate 破坏可复现性）。

---

## §4 0 误报状态

**审查链**：c 路（误报独立审查）→ 0 候选 → cc 路无需启动。

详 [`audit-v6-c-false-positive-2026-05-12.md`](../fixes/audit-v6-c-false-positive-2026-05-12.md)（107 行）。

### 4.1 c 路审查覆盖

独立 agent 审查 v6 1001 FAILED 中所有 v5.1 → v6 增量大的工具：verifast (+24) / verus (+22) / rocq-of-rust (+24) / rocq-of-rust-typecheck (+23) / soteria (+22) / prusti (+7) / aeneas-* (+1~2) / hax-* / kani / miri。

主动 grep 我方 wrapper 内部 bug 信号（`unbound variable` / `TS_*: not set` / `command not found` / bash 语法报错）和全局工具链崩溃信号（JVM / OOM / `core dumped` / `viper_tools`）均 **0 命中**。

### 4.2 误报候选：0

**结论**：v6 1001 FAILED 全部站得住宪法 §六 UNKNOWN 严格语义。

按本地性 / 社区惯例 / 最大善意三原则栈逐类排除：

| 类别 | 涉及工具 | 涉及 case | 排除理由 |
|---|---|---|---|
| A 工具单文件 pipeline 不读 Cargo deps | ror / ror-tc / verifast / verus / soteria | 21×5 = 105 | tool README 明文设计选择 → 工具能力边界 → FAILED 站得住 |
| B prusti 自带 cargo 不识 edition 2024 | prusti | 7 | 工具自选老 toolchain → 本地性原则下 FAILED |
| C 工具官方驱动 panic | 多 | 多 | charon-driver coroutine / verus rustc-internal / creusot unreachable / prusti internal error — 官方代码 = 工具锅 → FAILED |
| D wrapper 设计意图截断 silent partial | aeneas-* / ror | 11 | R7 新加 gate 拦截 partial → FAILED 正确 |
| E vendor lint UNKNOWN 已被 oracle 接住 | 多 | 10 | oracle `vendor_lint_strictness` 规则正确接住 → 不是漏过 FAILED |

注：c 路 agent 措辞混淆"v6 新增 21 corpus"——实测 corpus 未变（v5.1 = v6 = 161 entries）。所述 21 entries（bigint/8 + deps-complex/7 + industrial/6）在 v5.1 中已被分类为 UNKNOWN（dependency_resolution / toolchain_edition_mismatch），P27 DP-4 严格化后被重判为 FAILED。c 路 substantive 分析仍正确。

### 4.3 v6 0 误报结论

**v6 1001 FAILED 中 0 误报候选**。修宪 + Oracle 严格化 + D3 漏报封堵后，FAILED 入册标准对齐"本地性 + 社区惯例 + 最大善意"三原则——工具开发者不能驳回任一 FAILED 入册。

---

## §5 0 漏报状态

**审查链**：c 路（漏报独立审查）→ 4 候选 → cc 路 counter-challenge → 全 4 成立 → R7 fix 落地。

详 [`audit-v6-c-false-negative-2026-05-12.md`](../fixes/audit-v6-c-false-negative-2026-05-12.md)（206 行）+ [`audit-v6-cc-false-negative-counter-2026-05-12.md`](../fixes/audit-v6-cc-false-negative-counter-2026-05-12.md)（约 220 行）。

### 5.1 c 路漏报候选（4 个，独立 agent 在 v6 SUCCESS pool 中发现）

| # | 工具 × entry | 自陈 marker | 影响 |
|---|---|---|---|
| 1 | aeneas-coq/fstar/lean × `aeneas-limit/mutually-recursive-traits/trigger_mutually_recursive_traits` | `model will not type-check` + `generated code will likely be incorrect` | 3 SUCCESS |
| 2 | aeneas-coq/fstar × `impl-trait/return-iter/impl_trait_iter` | `model will not type-check` | 2 SUCCESS |
| 3 | aeneas-lean × `impl-trait/return-iter/impl_trait_iter` | `could not find the information for item 'map'/'sum'` + `seems to be missing the corresponding field` | 1 SUCCESS |
| 4 | rocq-of-rust-typecheck × `unsafe-ptr/raw-ptr-const/raw_ptr_const_match` | tier-0 ror 已 FAILED 但 tier-1 SUCCESS（破 `tier-1 ⊆ tier-0` 不等式声明）| 1 SUCCESS |

合计 **7 SUCCESS 实例**（c 路报告称 8 — 包括 aeneas-hol4 同 entry，但 hol4 在 v6 已 FAILED 故不算漏报）。

排除（合法 warning，符合宪法 §六-3 前端测量原则）：
- kani concurrency warning（已 D3.4 README 文档化为求解层假设）
- soteria atomic / complex-float intrinsic warning（D3.5）
- verus derived Clone auto-spec warning（D3.6）

### 5.2 cc 路 counter（全 4 候选成立）

cc agent 对每候选做 disprove-first counter，逐条找推翻理由。

**关键 counter 证据**：

- **候选 1+2+3（aeneas）**：精神性陈述反查锁定 aeneas-coq/fstar/lean README **第 36/44 行**硬声明"形式可证 0 漏报 / aeneas 不存在 silent emit-stub-but-exit-0 路径"+ **第 38/46 行"漏报盲点：无"**——但实测 aeneas 走 **Warn 通道**（非 craise/error_list）输出 partial 自陈，exit 仍 0，产物落盘但不可类型检——直接破 README "0 漏报"自陈。宪法 §六"工具自陈必须被尊重"覆盖。
- **候选 4（ror-typecheck）**：ror-typecheck README **第 43 行**"Stage 1 与 tier-0 完全等价 6 道 grep" + **第 136 行**"tier-1 ⊆ tier-0"——实测推翻（tier-0 FAILED + tier-1 SUCCESS）。wrapper 漏抄 D3.1 `grep "is not yet supported"`。

**cc 判定**：4 / 4 成立，0 / 4 不成立。

### 5.3 R7 fix 落地（commit `<P30>`）

按 cc fix 建议落地：

1. **aeneas 4 wrapper 加 stdout/stderr grep gate**：
   ```bash
   if grep -qE "model will not type-check|generated code will likely be incorrect|seems to be missing the corresponding field|could not find the information for item" aeneas_stage2.log; then
       echo "[aeneas-<backend>-oracle] FAIL: Warn-channel partial self-disclosure" >&2
       exit 1
   fi
   ```
   覆盖文件：`tools/aeneas-{coq,fstar,lean,hol4}/aeneas-*-wrapper.sh`

2. **aeneas 4 README 修订 "0 漏报 / 盲点：无" 自陈**为诚实声明（双通路：craise 主信号 + Warn 通道 wrapper 补抓）+ 列具体 4 类 partial pattern

3. **rocq-of-rust-typecheck wrapper 补 D3.1 gate**：
   ```bash
   if grep -q "is not yet supported" rocq_stderr.log; then
       echo "[ror-typecheck-oracle] FAIL: silent partial" >&2
       exit 1
   fi
   ```
   恢复 README 声明的 `tier-1 ⊆ tier-0` 不变式。

### 5.4 R7 重跑实测

重跑 4 个 aeneas backend + ror-typecheck（5 工具 × 161 entries = 805 task）实测：

| 工具 | v6 baseline S | R7 final S | 翻转 | 命中候选 |
|---|---:|---:|---:|---|
| aeneas-coq | 100 | **98** | -2 ✓ | 候选 1 + 2 |
| aeneas-fstar | 100 | **98** | -2 ✓ | 候选 1 + 2 |
| aeneas-lean | 100 | **98** | -2 ✓ | 候选 1 + 3 |
| aeneas-hol4 | 65 | **65** | 0（候选 1 entry 本就 panic FAILED）| 候选 1 防御性 |
| rocq-of-rust-typecheck | 125 | **124** | -1 ✓ | 候选 4 |

预期 7 / 实测 6 翻转。aeneas-hol4 在 mutually-recursive-traits entry 上**本就因 OCaml `Invalid_argument` panic 而 FAILED**（trait_decl_kind_to_qualif None），新 Warn gate 无新增翻转——但 gate 仍正确地加固了 hol4 wrapper（防御性，未来若 hol4 backend 改进就 panic 但仍 Warn channel emit partial 则会被抓到）。

### 5.5 v6 0 漏报结论

P27 baseline 封堵（D3.1/D3.2/D3.3）+ R7 cc-route 新封堵（aeneas Warn 通道 4 pattern + ror-typecheck D3.1 同步）共关闭 **15 个 silent partial 漏报候选**。

剩余 SUCCESS pool（2202 个）经独立 audit + cc-route 双轮验证后**无残留漏报候选**。kani concurrency / soteria intrinsic / verus derived auto-spec 已 D3.4-3.6 README 文档化为"求解层假设 / 中间步骤"（非前端 partial），按宪法 §六-3 前端测量原则保持 SUCCESS。

---

## §6 关键发现

### 6.0 v6 最终数据（R7 后）

- 总数：2202 SUCCESS / 1008 FAILED / 10 UNKNOWN = 3220 task（68.39% pass rate）
- v5.1 → v6 final delta：-15 SUCCESS / +136 FAILED / -121 UNKNOWN
- 15 ΔSUCCESS 来源分解：
  - 8（baseline 封堵）= 7（aeneas inline-asm + Type error 4 backend × 2 entries — 1 hol4 重叠） + 1（ror "is not yet supported"）
  - 7（R7 cc-route 封堵）= 6（aeneas 4 backend × 4 类 Warn partial — hol4 上 entry 本就 FAILED 不计） + 1（ror-typecheck D3.1 同步）

### 6.1 修宪后语义变化

P27 修宪后所有数字的含义变了：

- v5.1 的 131 UNKNOWN 中 121 个被重判 FAILED——它们对应"工具自身能力边界"（不读 Cargo.toml deps / 老 toolchain 不识 edition / 官方 wrapper 不传 --edition）。新原则下这些是 FAILED 而非"测量框架问题"
- v6 的 10 UNKNOWN 全是 vendored crate lint，全部归 "(b) 我们这边可识别问题" 子类，明确归因 + 待 DP-10 修

### 6.2 charter-craft 应用实证

本周期是 charter-craft 元方法学的第二次实证应用（首次：v5 重构 principles.md 338 → 123 行）。本次：

- 用户提出新问题意识 → 我用 charter-craft 防腐蚀分析 → 确认属问题意识层 → 修宪
- 用 self counter-challenge 检测细节（直陈性 vs 例外 / 正交性是理想 / 接受不完美）
- 减法优先：删 3 条 oracle 规则比加 5 条更符合奥卡姆

### 6.3 环境治源（verus / prusti 同性质）

本周期发现并修复 verus binary 因 `/tmp/` 周期清理而失效（同 P22 prusti viper_tools）。详 [`v6-verus-env-fix-2026-05-12.md`](../fixes/v6-verus-env-fix-2026-05-12.md)。

剩余 `/tmp/` 依赖工具（charon / aeneas / soteria）目前 OK 但仍属定时炸弹，建议长期统一迁移到 `~/.local/share/ts-tools/`。

---

## §7 决策点冻结状态

详 [`docs/fixes/decisions-2026-05-11.md §0.0`](../fixes/decisions-2026-05-11.md#0.0-p27-落地总览-2026-05-12--commit-28b4a03)。

P27 落地后**没有真正需要用户裁决的决策点**——所有原决策点（D1 / D2 / D3.1-3 / DP-4 / DP-5 / DP-3 / DP-6）已派生自宪法 §一 双根本问题 + 本地性原则，全部 resolve。

剩余：
- D3.4 / D3.5 / D3.6 README 漏报盲点补完 → P28 已落地
- DP-1 工具能力评估细则下沉 tool-integration（用户已表态接受，待 doc 同步）
- DP-7 / DP-10 / DP-12 长期次要模块工作

---

## §8 commit 链

| commit | 内容 |
|---|---|
| `28b4a03` | P27: 修宪 §一 双根本问题 + DP-4 UNKNOWN 严格化 + D3.1/3.2/3.3 oracle 漏报封堵 |
| `894effc` | P28: README 漏报盲点补完 (D3.4-3.6) + decisions §0.0 落地总览 |
| `6f6d91b` | P29: v6 verus env fix doc — /tmp/ 自动清理治源迁移 |
| (本 commit) | P30: R7 audit (c+cc) + aeneas Warn 通道 + ror-typecheck tier-0 同步 + v6 综合报告 |

---

## §9 不再适用的旧报告

`feature-coverage-2026-05-11-final-v5.1.md`（v5.1 最终）的数字语义在 P27 修宪后**已部分失效**——具体地，v5.1 报告中的 131 UNKNOWN 在新原则下绝大多数（121）应被重判 FAILED。v5.1 数据本身没错（运行结果），但其依赖的 oracle 分类逻辑（5 类外部根因 → UNKNOWN）已被宪法 §六 UNKNOWN 严格语义推翻。

阅读 v5.1 报告时应理解：其 UNKNOWN 类别在 v6 视角下重分类。本报告 §3.1 数字自洽性给出明确的差异分解。
