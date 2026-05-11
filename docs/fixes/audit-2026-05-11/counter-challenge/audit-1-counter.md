# Counter-Challenge for Audit-1 (design 4 份)

> Disprove-first 验证。我对 audit-1 的 13 条高严重度 challenge + 4 条修宪议案逐条独立重读源文件、找证据驳斥、给定判定。
>
> 重要声明：counter-challenge 比 audit 更狠地 disprove-first——意味着我把 audit 当成可疑方，不预设 audit 描述的现状准确。

---

## §1 验证范围

Audit-1 高严重度（H）challenge 清单（按文件 + 编号）：

- principles-review.md：#1（19 vs 20）/ #2（P15-P16 引入物宪法地位）+ §4-cross-doc #X1（道门数）/ #X2（19 工具跨 design 漂移）= **4 条高严重**
- architecture-review.md：#1（模块切分图）/ #2（决策表未涵盖 P11-P16）/ #3（env 注入契约缺失）= **3 条高严重**
- tool-integration-review.md：#1（5 vs 6 道门）/ #2（§五 README 必含缺反作弊）/ #3（§三 0 误报论证标准缝隙）= **3 条高严重**
- detailed-design-review.md：#1（§五 工具示例脱节）/ #2（缺 14 工具 quirk）/ #3（`${VAR}` 替换机制未描述）/ #4（TS_ENTRY_FN 注入未描述）= **4 条高严重**

合计 **14 条**高严重（注：审计总结说 13 条，但实际 audit 文档逐文件汇总 4 + 3 + 3 + 4 = 14；上下文中我按 14 条逐条验）。

修宪议案（principles-review §5 末尾）：议案 #2-A / #2-B / #2-C / #5 = **4 条**。

总计 **14 + 4 = 18** 条逐条验证。

---

## §2 验证方法

1. **独立重读**：每条 challenge 我独立 grep / 读源文件原文 + 行号，不接受 audit 摘述。
2. **disprove-first**：先假设 audit 错（误读 / 过度推论 / 凑数 / 事实错），找驳斥证据；驳不倒才承认 audit 成立。
3. **四档判定**：成立 / 部分成立 / 不成立 / 建议有问题。
4. **每条独立 evidence**：引用文件 + 行号 + 原文。

---

## §3 逐条验证

### Challenge #P-1 (高) —— "19 工具"硬编码与实施漂移

**Audit 判定**：principles.md line 71 / 308 写"19 个"，实际 `tools/` 目录 20 个（多 rocq-of-rust-typecheck）；宪法层"模块定位"段把 corpus 数硬编码到正文，应改为"若干"。

**Counter 重读**：
- `principles.md:71` 原文："**定位**：本项目目前接入的 19 个具体 Rust 验证工具（cargo-check / kani / miri / charon × 2 / creusot / hax × 3 / aeneas × 4 / prusti / verus / verifast / soteria / kmir / rocq-of-rust）的配置与实测..." — 列表数清确实 14 个标记 + × 倍数 = 1+1+1+2+1+3+4+1+1+1+1+1+1 = 19，不含 rocq-of-rust-typecheck。
- `ls tools/` 实际 20 项：`aeneas-{coq,fstar,hol4,lean}` / `cargo-check` / `charon-{mono,poly}` / `creusot` / `hax-{coq,fstar,lean}` / `kani` / `kmir` / `miri` / `prusti` / `rocq-of-rust` / `rocq-of-rust-typecheck` / `soteria` / `verifast` / `verus` = 20。
- `principles.md:308` 原文："**下游：`detailed-design.md`** — 函数级细化，schema 完整定义、运行时机制、19 工具配置示例" — 同步漂移。

**判定**：☑ **成立**

**Counter 证据链**：现象准确（行号 + 原文 + 实际目录确认）。但 audit 推论与建议有微妙偏差——audit 既建议"改若干"又同时建议"补完 rocq-of-rust-typecheck 到列表"（议案 #2-A）。这两个建议本质上冲突：前者去除硬编码、后者保留硬编码只是补漏。**Counter 倾向前者**——因为"模块定位"是宪法 §三 章节，列具体工具数本身就是把"次要模块"内容侵入宪法主体；若按"通用性 + 长期承诺"原则，宪法应只描述"次要模块 3 = 工具集成与实测，清单见 tools/"，不 enumerate。

**修正建议**：line 71 改"若干个" + 把工具枚举降级到"详细清单见 tools/ + tools/README.md"。议案 #2-A 应否决——不应让"补 ror-typecheck 这条行"再触发一次修宪，结构性把"宪法 enumerate"问题去掉。

---

### Challenge #P-2 (高) —— P15/P16 引入物未在宪法地位声明

**Audit 判定**：grep `runnable` / `rocq-of-rust-typecheck` / `TS_*` / `${VAR}` 在 principles.md 全部 0 hits；`wrapper.sh` 仅 line 188 一次性列举式提及，未做地位声明。CLAUDE.md §1.3 文档优先原则要求 P15/P16 引入物在宪法或下游先获覆盖再实施——这是反向漂移。

**Counter 重读**：
- `principles.md:188` 原文："**Tools 端绝不为框架适配** —— tools 是黑盒。`tools/<name>/` 目录下的所有文件（`tool.toml`、`harness.rs.tera`、wrapper.sh 等）是**集成者描述工具自身行为**的桥接信号——**不是工具自身的修改**。tool 本身（cargo-kani / cargo-prusti / cargo-creusot 等）作为黑盒，绝不为框架做任何代码或行为上的适配。" — wrapper.sh 这里**与 tool.toml/harness.rs.tera 并列**，并非"列举式一次性提及"——而是和正式集成产物同地位。**audit 把"在 A-4 段并列声明地位"读成"未做地位声明"，是事实层错误**。
- `principles.md:41-42`：「**工具版本**：每次 run 跑 `version_command` 采集每工具的版本字符串」 + 「**测试环境**：host info」 —— 这正是 host.rs / capture_version 实现兑现的宪法层承诺。host.rs / TS_PROJECT_ROOT / TS_ENTRY_FN 的实施在宪法 §三 line 41-42 已 implicitly 授权。
- runnable corpus：principles.md / architecture.md / tool-integration.md 全文确实 0 hits。但 detailed-design.md line 30-80 详细写 `[runnable.<fn>]` 段；line 78 显式说"runner 不感知 runnable 标记"——runnable 段是 example schema 的扩展，按 principles §四 A 形式定义（hirusttest.toml schema 扩展且 cargo 不读 → 不破形式定义），属"细化层自由扩展，无需修宪"范畴。
- `rocq-of-rust-typecheck` 在 design 主线 0 hits：确实未在宪法登记为新工具。但这条与 #P-1 同源（工具枚举漂移），不是独立问题。

**判定**：☐ **部分成立**

**Counter 证据链**：
- audit "wrapper.sh 未做地位声明" 部分 **不成立** ——line 188 已明示桥接信号地位
- audit "runnable 未在 design 主线吸纳" 部分 **现象成立但推论偏过严**——runnable 段是 example schema 扩展，宪法 A 形式定义已给授权（信号文件不被 cargo 读），无需"修宪"
- audit "ror-typecheck 未在工具枚举" 部分 **成立但与 #P-1 同源**
- audit "TS_* / `${VAR}` 替换 / TS_ENTRY_FN 注入" 部分 **现象成立但部位错**——这些是 runner 实施细节，应在 detailed-design 描述（见 #D-3 / #D-4 / #A-3），不应进宪法。CLAUDE.md 说"原则可溯源即可"，不是"宪法 enumerate 实施细节"。

**修正后判定**：现象部分成立、推论偏激进。议案 #2-B / #2-C 应否决（见 §4）。议案 #2-A 应降级为"局部 fix"（去 enumerate）。

---

### Challenge #P-X1 (高) —— rocq-of-rust 道门 5 vs 6（tool-integration 落后）

**Audit 判定**：principles.md line 95 写 "6 道门"；tool-integration.md line 30 写 "5 道门"；实际 wrapper 注释 6 gates。tool-integration 落后于宪法与实施。

**Counter 重读**：
- `principles.md:95`："多门组合（rocq-of-rust **6 道门**）" — 准确
- `tool-integration.md:30`："多门组合（如 rocq-of-rust 的 5 道门：exit 0 + 至少一个 .v + 无 0-byte + > 200B + 无 silent marker）" — 列了 5 个，缺 gate 6（entry_fn 出现校验）
- 实际 `tools/rocq-of-rust/rocq-of-rust-wrapper.sh:55-61` 注释列 6 gates，且 `tools/rocq-of-rust/tool.toml` 头注释也说 "6 gates"

**判定**：☑ **成立**

**Counter 证据链**：现象准确、tool-integration 确实落后、修复方向（改 5 → 6 + 补 gate 6 描述）正确。无反对意见。**非决策点**——直接改 tool-integration.md line 30。

---

### Challenge #P-X2 (高) —— "19 工具"跨 4 份 design 同步漂移

**Audit 判定**：principles.md line 71/308、architecture.md line 84/294、detailed-design.md line 335 全部硬编码 "19"。

**Counter 重读**：
- principles.md:71/308：确认硬编码 19（见 #P-1）
- architecture.md:84："cargo-check / Kani / MIRI / Charon (poly + mono) / Prusti / Creusot / Verus / VeriFast / Soteria / kmir / hax (lean/fstar/coq) / aeneas (lean/coq/fstar/hol4) / rocq-of-rust 全在内（19 工具）"
- architecture.md:294："19 工具配置示例"
- detailed-design.md:335："19 个工具的完整 `tool.toml` / `harness.rs.tera` 配置..."

**判定**：☑ **成立**

**Counter 证据链**：4 份 design 全部硬编码 "19"，是 P15 引入 ror-typecheck 之后未级联更新的同源漂移。修复方向（统一去 enumerate 或动态化）正确。**与 #P-1 同源**——一次性修复即可。

---

### Challenge #A-1 (高) —— 模块切分图覆盖不全（缺 wrapper.sh / host.rs / 辅助文件）

**Audit 判定**：architecture.md line 99-110 模块切分图缺 host.rs 子模块、tools/<name>/ 下的 wrapper.sh、examples 目录轨辅助文件。

**Counter 重读**：
- `architecture.md:99-110` 原文确实列：`runner/` 下 `discover / exec / report / main` 4 子模块；`tools/<name>/` 下"tool.toml + harness.rs.tera — 由集成者写"；`examples/<feature>/<dir>/` 下"Cargo lib crate + hirusttest.toml — 由用户写"——确实没列 host.rs / wrapper.sh / `.hirusttest/` 目录辅助文件。
- 但 `architecture.md:32-50` 双轨 schema 段已列目录轨结构含 `.hirusttest/<可选辅助文件>` —— 同份 architecture 文档内有重叠覆盖。
- runner/src/ 实际目录列出 5 个：discover / exec / host / main / report —— host 实际存在。

**判定**：☐ **部分成立**

**Counter 证据链**：
- "缺 host.rs" 部分 **成立**——architecture line 99-110 应补这个子模块
- "缺 wrapper.sh" 部分 **成立**——line 99-110 应补"tools/<name>/ + 可选 wrapper.sh"，因为 9 个工具用 wrapper.sh
- "缺辅助文件" 部分 **部分成立**——architecture line 42-50 已列双轨 schema 含辅助文件；line 99-110 仅给"用户写"侧的单文件轨示意，是简化呈现。建议加注"详见 §一 双轨 schema"即可，不必双重列出。

**修正后判定**：现象部分成立、修复方向（补 host.rs + 可选 wrapper.sh + 加注引）正确。**非决策点**。

---

### Challenge #A-2 (高) —— 决策表未涵盖 P11-P16 引入物

**Audit 判定**：architecture.md line 268-289 决策表 grep `wrapper.sh / ${VAR} / TS_ENTRY_FN / runnable` 全部 0 hits。建议加 4 行决策。

**Counter 重读**：
- `architecture.md:281`："| ResultClass 三分（SUCCESS / FAILED / UNKNOWN，UNKNOWN 仅记 runner-internal 错误） | B + Occam | 锁定 |" — 决策表里有
- `architecture.md:283`："| `results.json` 顶部 metadata 段（host / 时间戳 / 各工具 version_command 输出） | 工具非静态原则 | 锁定 |" — host.rs 的根派生在这里
- 决策表 line 268-289 共 19 行——确实没列 wrapper.sh / `${VAR}` 替换 / TS_ENTRY_FN 注入 / strip TS_*

**判定**：☐ **部分成立**

**Counter 证据链**：
- audit 现象描述准确——决策表缺 P11-P16 引入物
- 但 audit 建议"加 4 行决策"有问题：
  - wrapper.sh 已经在 architecture line 99（"集成者写"）+ principles A-4 line 188（"桥接信号"）覆盖；决策表加一行也行、不加也行（非"破宪法"级缺失）
  - `${VAR}` 替换 / TS_ENTRY_FN 注入 / strip TS_* 应在 **detailed-design** 加（function-level），不在 architecture（架构层）；架构层加只是装饰
- audit 把"宪法 → 架构 → 细化"的层级误读为"任何实施细节都进架构决策表"——这违反 architecture 自身的"层级分级"精神

**修正后判定**：现象成立、修复部位错。建议把 audit 提的 4 行决策**移到 detailed-design** 而非 architecture。非决策点（部位调整）。

---

### Challenge #A-3 (高) —— 接口规约 runner → external_subprocess 未声明 env 注入契约

**Audit 判定**：architecture.md line 225-235 接口规约缺 env strip TS_* + env_set TS_ENTRY_FN / TS_TARGET_CRATE 的描述。实际 exec.rs:165-179 + 178 已实施。

**Counter 重读**：
- `architecture.md:225-235` 原文："接口：runner → external_subprocess / 输入数据：tool.command argv、副本目录 cwd、stdio piped... / 输出数据：subprocess 的 exit_code、stdout 字节流、stderr 字节流、wall-time / 约束：subprocess 起在独立 process group" — 确实没说 env 注入契约
- `exec.rs:165-171`：strip 所有 TS_* / `exec.rs:178`：注入 TS_ENTRY_FN + TS_TARGET_CRATE
- `tools/rocq-of-rust/tool.toml` 头注释明示："`TS_ENTRY_FN` is re-injected by the runner after the strip (runner/src/exec.rs:178), so the wrapper accesses it directly" — wrapper 的 README 明确依赖这一契约

**判定**：☑ **成立**

**Counter 证据链**：现象精确、修复方向（架构接口规约加 env 注入契约）合理。这是真决策点——这两个内部环境变量（TS_ENTRY_FN / TS_TARGET_CRATE）一旦作为正式契约，未来不能轻易删，是 wrapper.sh 集成者依赖的接口。**真决策点，应交用户审议**。

但 audit 的实测段（line 178-190）说"TS_PROJECT_ROOT 也是 TS_ 前缀，会被 strip 掉"—— 我独立查了 `main.rs:67`，确实 set TS_PROJECT_ROOT，然后 exec.rs:165-171 strip 所有 TS_*。**子进程实际拿不到 TS_PROJECT_ROOT**——它只在 runner expand_env 阶段被消费替换到 argv 字面值。audit 这条事实也准确。

---

### Challenge #T-1 (高) —— tool-integration "5 道门"过时

**Audit 判定**：tool-integration.md line 30 写 "5 道门"，落后于 P15 加入 gate 6。

**Counter 重读**：同 #P-X1。

**判定**：☑ **成立**

**Counter 证据链**：与 #P-X1 完全同源，audit 在两文件中分别记，是合理的（一份 audit 同一现象交叉两文件）。修复方向正确——非决策点。

---

### Challenge #T-2 (高) —— §五 README 必含章节清单缺反作弊声明

**Audit 判定**：tool-integration.md line 120-130 §五 8 个必含章节中无"反作弊证据"，而 principles.md §六-4 line 261-265 反作弊推论是宪法硬指标——方法学必须把宪法硬指标全覆盖到 README 必含章节中。

**Counter 重读**：
- `tool-integration.md:120-130` 8 章节：简介 / 前端接受定义 + pipeline 阶段图 + 我方切割点 / SUCCESS 信号 / 形式严格性 / 安装 / 本框架配置 / 已知限制 / 关联 sub-tests
- 章节 4 "形式严格性 —— 按 §三 / §四 声明 0 误报 / 0 漏报状态" —— **这里就是"反作弊"的承载点**！0 误报 ≈ oracle 不冤枉工具能力；反作弊 = "oracle 不退化为 cargo-check 等价" = 0 误报论证的延伸。
- 章节 2 "本测试集中的'前端接受'定义 + pipeline 阶段图 + **我方切割点**" —— "切割点" 描述了 oracle 切割哪段；切割点正确 ⇒ 不退化为 cargo-check 等价。

**判定**：☐ **部分成立**

**Counter 证据链**：
- audit "反作弊声明无专章节" **形式上成立**——确实没有名为"反作弊证据"的独立章节
- audit "宪法 §六-4 硬指标方法学覆盖必须独立章节" **推论过强**——§六-4 是硬指标，但工具方法学已经通过章节 2（切割点）+ 章节 4（形式严格性 / 0 误报）双重承载，是 implicit 覆盖。verus 反作弊典型例就是 章节 4 / 章节 7（已知限制）的内容（实际 tools/verus/README.md 在哪段都可声明）
- audit 建议"扩展章节 2 加反作弊证据"合理；建议"加新章节 9 反作弊证据"过度结构化

**修正后判定**：现象 80% 成立——扩展章节 2 加一句"反作弊证据：dry-run flag 确证真喂工具前端"是合理的小补。无需独立章节。**真决策点（中等优先级）**——影响 README 模板。

---

### Challenge #T-3 (高) —— §三 0 误报论证"实测验证"标准模糊

**Audit 判定**：tool-integration.md line 36-58 §三 0 误报论证列了 4 种论证形式，其中"实测验证"未明示"多少样本算足够"。P14 / P15 实测打击经验未吸纳。

**Counter 重读**：
- `tool-integration.md:45`："**实测验证**：在足够多样的合法 SUCCESS 样例上验证 oracle 不误报（不构成形式证明，但提供足够经验证据时可接受）" — 确实模糊"足够多样"
- `tool-integration.md:57`："每个 ⚠️ 通过实测（而非源码层穷尽）论证 0 误报的工具必须在 README 列出双向实测证据（按 §四-4.2 反误报检查）" — 这是 implicit 标准：双向实测
- 实际 P14 / P15 修复记录在 `docs/fixes/oracle-leak-rules-implementation-{1,2}-2026-05-{08,11}.md` 留底

**判定**：☐ **部分成立 + 建议有问题**

**Counter 证据链**：
- audit "实测验证标准模糊"**部分成立**——line 45 "足够多样"确实没量化
- audit 建议"corpus ≥ 20 + 覆盖 std / no-std / async / unsafe + ..."**有问题**：硬码数字是反 CLAUDE.md `feedback_no_premature_extensibility` 精神。`audit 的修复方向把方法学约束逐条硬编码，反而打破了 audit 自己强调的"通用性"和"诚实性边界"`——不同工具的合理 corpus 大小完全取决于工具特性，没有一刀切数字。
- 当前文字"足够多样"是有意为之的工程判断空间——这是宪法 §三-3-2.c "下限诚实"的延伸（承认主观判断空间）。强行量化反而把方法学定型化。

**修正后判定**：audit 现象部分成立、建议有问题（不应硬编码数字）。可改为"line 45 加注 see 实施记录 docs/fixes/oracle-leak-*-implementation-*.md"，提供经验参考而非硬约束。**议案 #T-3 应否决修宪建议、降级为 line 45 加引用 fix 记录**。

---

### Challenge #D-1 (高) —— §五 工具配置示例与 tool.toml 全面脱节

**Audit 判定**：detailed-design.md §五 6 个工具示例（cargo-check / Kani / MIRI / Charon-poly/mono / Prusti / Creusot）的 `command` 字段全部与实际 tool.toml 脱节。

**Counter 重读**：
- `detailed-design.md:365`：示例 `command = ["cargo", "kani", "--only-codegen", "--bin", "__ts_harness"]`
- 实际 `tools/kani/tool.toml:27`：`command = ["${TS_PROJECT_ROOT}/tools/kani/kani-strict-wrapper.sh"]` + `version_command = ["cargo", "kani", "--version"]`
- Kani 示例缺：wrapper.sh / `${TS_*}` 替换 / version_command
- charon-poly 示例 (line 398)：`["/tmp/ts-tools-install/charon/bin/charon", ...]` —— 硬编码 /tmp 路径；实际 tool.toml 用 `${TS_CHARON_BIN}` 替换、多 `--print-llbc`
- Prusti 示例 (line 425-440)：用 `${PRUSTI_*}` 硬编码 env list；实际用 `${TS_PRUSTI_*}` + wrapper.sh

**判定**：☑ **成立**

**Counter 证据链**：示例脱节是 P11-P16 后实施超前于 detailed-design 的明确证据。修复方向（要么同步示例、要么明示"历史快照"）合理。audit 建议把示例改为示意性 placeholder 引用 tool.toml 实际位置——更稳健，不需要双源维护。**非决策点（局部 fix）**。

---

### Challenge #D-2 (高) —— §五 缺 14 个工具的 quirk 描述

**Audit 判定**：detailed-design.md §五 line 333-475 仅列 6 个工具 quirk（cargo-check / Kani / MIRI / Charon × 2 / Prusti / Creusot），缺 14 个（Verus / VeriFast / Soteria / kmir / hax × 3 / aeneas × 4 / rocq-of-rust × 2）。

**Counter 重读**：
- `detailed-design.md:335`："19 个工具的完整 `tool.toml` / `harness.rs.tera` 配置 + 安装步骤详见各工具自己的 `tools/<name>/README.md`。下面列每个工具的关键 quirk（schema 怎么用、为什么这么用）"
- §五 接下来逐节列出 cargo-check / Kani / MIRI / Charon (poly/mono) / Prusti / Creusot —— 共 6 节
- 缺 14 个：Verus / VeriFast / Soteria / kmir / hax × 3 / aeneas × 4 / rocq-of-rust × 2

**判定**：☐ **部分成立 + 建议有问题**

**Counter 证据链**：
- audit 现象**成立**——line 335 字面承诺"每个工具"，但只兑现 6/20
- audit 建议"补完 14 个 quirk"**有问题**——这是次要模块工作（按 principles §三-3 "次要地位"+ CLAUDE.md "次要模块不抢核心模块优先级"），detailed-design 不应承担这部分。每工具 quirk 应在 `tools/<name>/README.md` 写（按 tool-integration §五 README 必含章节）。
- audit 备选建议"line 335 改为'仅典型 quirk'"**合理**——明示 §五 是方法学示例展示而非穷举，是更轻量级、更符合宪法 §三-3 精神的修法

**修正后判定**：现象成立、修复方向（line 335 改措辞）合理；建议补完 14 工具 quirk 应否决（侵入次要模块工作量）。**非决策点（措辞调整）**。

---

### Challenge #D-3 (高) —— `${VAR}` 环境变量替换机制未在细化层描述

**Audit 判定**：detailed-design.md §一 / §三 / §四 完全没说 `${VAR}` 替换；实际 runner/src/discover.rs:294-335 的 `expand_env()` 函数支撑了所有 wrapper.sh 工具（9 个 tool.toml）。

**Counter 重读**：
- `runner/src/discover.rs:294`：`fn expand_env(s: &str) -> String { ... }`
- `runner/src/discover.rs:377`：`let command: Vec<String> = parsed.command.into_iter().map(|s| expand_env(&s)).collect();`
- `runner/src/discover.rs:381`：`.map(|s| expand_env(&s))` 应用到 version_command
- `detailed-design.md` 全文 grep `expand_env / \${VAR / 环境变量替换`：0 hits
- `tools/rocq-of-rust/tool.toml` 实际用 `["env", "ROCQ_OF_RUST_TOOLCHAIN_SYSROOT=${TS_ROCQ_OF_RUST_TOOLCHAIN_SYSROOT}", "${TS_PROJECT_ROOT}/tools/rocq-of-rust/rocq-of-rust-wrapper.sh"]` —— 直接依赖 `${VAR}` 替换

**判定**：☑ **成立**

**Counter 证据链**：现象精确——这是 runner 实施的核心机制（9 个 wrapper.sh 工具都依赖），但细化层完全无描述。修复方向（§一 加 `${VAR}` 替换机制描述）正确。**真决策点（应该把它写进 detailed-design §一）**。

但 audit 议案 #2-C "明示 `${TS_*}` 环境变量替换层符合原则 C" 主张升宪——**反对**：这是 runner 实施细节，应在 detailed-design 写、archietcture 决策表加一行（"`${VAR}` 替换层 → C + 反硬编码"）就够，不必到宪法 §四 加新派生原则。

---

### Challenge #D-4 (高) —— TS_ENTRY_FN / TS_TARGET_CRATE 子进程注入未在细化层 §四 描述

**Audit 判定**：detailed-design.md §四 单次执行单元 line 274-306 没有任何 env strip / TS_ENTRY_FN inject 描述；实际 runner/src/exec.rs:165-179 实施。

**Counter 重读**：
- `detailed-design.md:274-306` 单次执行单元 8 步骤：exec_id / work_dir / cp / patch / cd / 渲染 / spawn / 写 raw / cleanup —— 确实无 env strip / TS_* inject 步骤
- `exec.rs:163-179` 实施 env strip + TS_ENTRY_FN/TS_TARGET_CRATE 注入
- `tools/rocq-of-rust/rocq-of-rust-wrapper.sh:72`：`# TS_ENTRY_FN — name of entry fn under test` (wrapper 显式依赖)
- `tools/rocq-of-rust/tool.toml` 头注释明示：「`TS_ENTRY_FN` is re-injected by the runner after the strip」

**判定**：☑ **成立**

**Counter 证据链**：现象精确、修复方向（§四 加 step 5b "env strip + TS_ENTRY_FN/TS_TARGET_CRATE 注入"）正确。**真决策点（与 #A-3 同源——架构层接口规约缺 env 契约 + 细化层步骤缺 env 注入步）**。

但 audit 议案 #2-C "升宪" **反对**——这是 runner ↔ wrapper.sh 的接口契约，属 architecture 接口规约层 + detailed-design 步骤层。宪法 §四 已经给"工具非静态原则" + "host info / version_command 采集"等 implicit 授权（principles.md:41-42）。无需升宪。

---

## §4 修宪议案审议

Audit-1 §5 末尾提出 4 条修宪议案：

### 议案 #2-A：吸纳 ror-typecheck 为正式集成工具（更新 §三-3 工具枚举 + §六-1 表）

**议案原文**（audit-1 principles §5）：在宪法的 §三-3 工具枚举更新 + §六-1 表加 rocq-of-rust-typecheck 行 + §四 A 形式定义后注明"hirusttest.toml 内 `[runnable.*]` 等子表扩展不破形式定义"。

**是否真有必要修宪？**：❌ **不必要修宪——前提不成立**

**论证**：
1. 宪法 §三-3 line 71 硬 enumerate "19 工具"是宪法书写质量缺陷，不是 ror-typecheck 缺席这件事。**应改的方向是把硬编码数字去掉**（按 #P-1 修正后判定），而不是"每加一个工具就修宪补一条枚举"——这只会让宪法变成实施现状追踪表。
2. 按 CLAUDE.md "principles.md 是宪法、未经允许不可修订"——但本项目同时说"次要模块（tools 集成）随实施动态变化、不构成长期承诺"。这两条结合：**宪法不应 enumerate 工具，让工具列表动态反映在 tools/ 目录即可**。
3. §六-1 表是"前端切割落地表"，列各工具的具体 flag——这条表本来就该随实施动态更新，但**不构成"修宪"**（按 audit-1 自己 #P-1 末尾的诚实承认："纯计数的小修属于反映实施现状，不构成实质宪法修订"）。

**判定**：☑ **议案前提不成立** —— 应降级为"去除 enumerate + tools/ 列表替代"的局部 fix，不构成修宪。

---

### 议案 #2-B：明示 hirusttest.toml 子表扩展（如 `[runnable.*]`）符合原则 A 形式定义

**议案原文**：在宪法 §四 原则 A 形式定义后注明 hirusttest.toml 内 `[runnable.*]` 等子表扩展不破形式定义。

**是否真有必要修宪？**：❌ **不必要——议案前提部分错**

**论证**：
1. principles.md:146-157 A 形式定义已经精确：「**信号文件必须对 cargo / rustc / 任何 verifier 工具完全不可见**——加不加它，对 `cargo build` / `cargo check` / `cargo run` / `cargo test` 等任意 cargo 子命令的输出**字节级一致**」 —— 这条形式定义是关于"signal file 在 cargo 子命令上是否被读取"，**与 schema 内部结构无关**。`[runnable.*]` 段是 hirusttest.toml 内部增加 toml 表，cargo 仍然不读 hirusttest.toml，行为字节级一致。
2. 形式定义已经把"作用域"绑定在"对 cargo 子命令的影响"上，**不需要给 schema 扩展逐一签发免疫书**——任何 hirusttest.toml schema 扩展只要满足"cargo 不读"自动满足形式定义。
3. detailed-design.md:78 已明示 "ID 语义不变 / 这条由消费 runnable 的工具在 wrapper 内自筛——runner 不感知 runnable 标记" —— 已说清 runnable 段不波及 cargo 行为。
4. 议案 "在宪法明示 schema 扩展不破形式定义"是把"形式定义的演绎结论"重写为"宪法的具体授权" —— 反而把形式定义的通用性降级。**宪法应保持原则简洁，演绎在下游做**。

**判定**：☑ **议案前提不成立** —— hirusttest.toml schema 扩展无需修宪；line 173 + 形式定义已涵盖。

---

### 议案 #2-C：明示 `${TS_*}` 环境变量替换层符合原则 C

**议案原文**：在宪法 §四 原则 C 段后明示 `${TS_*}` 替换机制符合 C 异质性归配置。

**是否真有必要修宪？**：❌ **不必要——属层级误置**

**论证**：
1. principles.md:198-200 原则 C 原文："工具间差异以声明数据形式（`tool.toml + tera 模板`）存在，不作为框架代码的 if 分支。任何 `if tool == "kani"` 都被禁止" —— C 的核心是"工具差异不进 runner 代码 if 分支"；`${TS_*}` 替换是"路径属当前主机环境维度"的衍生处理，**runner 代码用统一 expand_env 函数实施**——没有 `if tool == "kani" then path = X` 之类——它本身就符合 C。
2. 这是 detailed-design 应描述的运行时机制（见 #D-3），架构层决策表加一行就够（"`${VAR}` 替换层 → C + 反硬编码"）——**不需要宪法 §四 加新派生**。
3. 把每一个细化机制"明示符合宪法某条"会让宪法变成实施目录索引——破坏宪法的高度抽象性。

**判定**：☑ **议案前提不成立** —— `${TS_*}` 替换层应在 detailed-design 描述、architecture 决策表加一行；宪法 §四 不变。

---

### 议案 #5：A 形式定义加"作用于原始磁盘 example 目录的 cargo 行为"限定语

**议案原文**（audit-1 principles #5）：line 146 改为「一个 example 的**原始磁盘目录**是"自身完善独立"的当且仅当——在该目录下加入框架信号文件 `hirusttest.toml` 之前与之后，**该原始磁盘目录上的任意 cargo 子命令输出**完全 100% 一致」

**是否真有必要修宪？**：❌ **不必要——议案误读了原文**

**论证**：
1. 重读 principles.md:142-157 完整段：
   - line 142 已明示 "位阶澄清：A 保护的是**原始磁盘的样例源码**——`src/`、entry 函数体、属性标注一律不为工具改动；隔离副本上的 manifest 注入（`extra_cargo_deps`）与 entry 入口替换（`entry_mode = "lib"`）属于'中介层声明式填充'"
   - line 150 形式定义已说 "对 `cargo build` / `cargo check` / `cargo run` / `cargo test` 等任意 cargo 子命令的输出**字节级一致**" —— **cargo 子命令默认操作在当前目录（即原始磁盘）**
   - line 173 已说 "`.hirusttest/` 仅可包含项目自有 schema 的辅助文件...这些辅助文件可被工具读取（**这是受控的工具适配，发生在 framework 中介层调度时**），但 example 自身的 src/ 与 Cargo.toml 仍需满足形式定义'cargo 行为字节级一致'"
2. line 142 + 150 + 173 已构成完整的位阶澄清——audit 议案说"形式定义没明示作用域"是把 line 142 的位阶澄清和 line 173 的 src/ 边界**割裂**来读，等于无视位阶澄清这一节存在。
3. 实际隔离副本 stub 注入（line 173 描述的"`<entry>-spec.rs` 注入到副本 src/"）是中介层调度的副本操作；line 142 早已声明这种操作是"位阶澄清下允许的"——形式定义只约束"原始磁盘目录上 cargo 子命令的行为字节级一致"已经通过 line 173 末尾"但 example 自身的 src/ 与 Cargo.toml 仍需满足形式定义"显式落地。

**判定**：☑ **议案前提不成立** —— audit 误读 line 142 + 173 的现有约束，把"位阶澄清没说"误推为"形式定义没说"。当前文字已自洽，无需修订。

---

## §5 总结

### 14 条高严重 challenge 最终分布

| 编号 | audit 摘要 | counter 判定 |
|---|---|---|
| #P-1 | 19 工具硬编码 | ☑ **成立**（建议方向：去 enumerate，不是补 ror-typecheck） |
| #P-2 | P15/P16 引入物宪法地位 | ☐ **部分成立**（wrapper.sh 已在 line 188 声明地位；runnable 段宪法 A 形式定义已涵盖；TS_* 应在 detailed-design 描述） |
| #P-X1 | 5 vs 6 道门 (tool-integration 落后) | ☑ **成立** |
| #P-X2 | 19 工具跨 4 份 design 漂移 | ☑ **成立**（与 #P-1 同源） |
| #A-1 | 模块切分图缺 host.rs/wrapper.sh/辅助文件 | ☐ **部分成立**（缺 host.rs / wrapper.sh 成立；缺辅助文件部分 §一已覆盖） |
| #A-2 | 决策表未涵盖 P11-P16 引入物 | ☐ **部分成立 + 建议部位错**（应在 detailed-design 加，不在 architecture 决策表全加） |
| #A-3 | 接口规约缺 env 注入契约 | ☑ **成立** |
| #T-1 | tool-integration 5 道门过时 | ☑ **成立**（与 #P-X1 同源） |
| #T-2 | §五 README 必含缺反作弊声明 | ☐ **部分成立**（章节 2 切割点 + 章节 4 形式严格性已 implicit 覆盖；加一句明示反作弊证据合理） |
| #T-3 | §三 0 误报论证"实测验证"标准模糊 | ☐ **部分成立 + 建议有问题**（不应硬编码 corpus 数字） |
| #D-1 | §五 工具示例与 tool.toml 全面脱节 | ☑ **成立** |
| #D-2 | §五 缺 14 工具 quirk | ☐ **部分成立 + 建议有问题**（应改 line 335 措辞为"仅典型示例"，不应补 14 工具到 detailed-design） |
| #D-3 | `${VAR}` 替换机制未描述 | ☑ **成立** |
| #D-4 | TS_ENTRY_FN/TS_TARGET_CRATE 注入未描述 | ☑ **成立**（与 #A-3 同源） |

**分布统计**：
- **成立**：8 条（#P-1, #P-X1, #P-X2, #A-3, #T-1, #D-1, #D-3, #D-4）
- **部分成立**：5 条（#P-2, #A-1, #A-2, #T-2, #D-2）
- **建议有问题**：2 条（#T-3, #D-2 同时归类——D-2 既部分成立又建议有问题）
- **不成立**：0 条

### 4 修宪议案审议

| 议案 | 审议结果 |
|---|---|
| #2-A 吸纳 ror-typecheck 进宪法工具枚举 | ☑ **议案前提不成立**——应降级为"去 enumerate"局部 fix |
| #2-B 明示 hirusttest.toml 子表扩展符合 A 形式定义 | ☑ **议案前提不成立**——形式定义已涵盖 schema 扩展；无需修宪 |
| #2-C 明示 `${TS_*}` 替换符合原则 C | ☑ **议案前提不成立**——应在 detailed-design 描述、architecture 决策表加行 |
| #5 A 形式定义加"作用于原始磁盘 cargo 行为"限定语 | ☑ **议案前提不成立**——audit 误读 line 142 / 150 / 173 的现有约束 |

**全部 4 条修宪议案前提不成立**——audit-1 高估了"需要修宪"的严重度，但 4 条议案描述的现象本身是 design 文档可改进的方向，均可在 architecture / detailed-design 层做局部 fix。

### Audit-1 整体质量评估

**等级：B 级**

**优点**：
- 引用行号准确（除 #P-2 wrapper.sh 部分误判"无地位声明"）
- 现象描述精确（19/20 工具数、5/6 道门、env 注入契约缺失、`${VAR}` 替换缺描述、§五 示例脱节等都是事实）
- 区分决策点 / 非决策点的总结结构清晰

**主要问题**：
- **过度修宪倾向**：4 条修宪议案全部前提不成立，audit 把"宪法应抽象 → 下游派生实施"的层级误读为"任何实施细节都应宪法明示"。这是 audit 对 CLAUDE.md "宪法是绝对底线、未经允许不可修订" 的反向过度解读——觉得"必须修宪才能解决问题"，反而让宪法越来越拥肿。
- **修复方向偶尔反精神**：
  - #T-3 建议硬编码 corpus 数字 ≥ 20 + 覆盖 std/no-std/async/unsafe 等分类 —— 违反 CLAUDE.md `feedback_no_premature_extensibility` 精神
  - #D-2 建议补完 14 个工具 quirk 到 detailed-design —— 让次要模块工作量侵入核心 design 文档
  - #A-2 建议把 P11-P16 引入物全加到 architecture 决策表 —— 把架构决策表混入实施细节
- **凑数嫌疑**：principles-review §3 #5（A 形式定义作用域）实际是 audit 把 line 142 + 173 的现有位阶澄清当成不存在，挑出来当决策点；本该是 audit 自己重读漏了 line 142 + 173，反映 audit 的诚实度可改进。

**最关键 audit 错误 top 3**：

1. **#P-2 wrapper.sh 地位声明读错**：principles.md:188 把 wrapper.sh 与 tool.toml/harness.rs.tera 并列声明为"集成者描述工具自身行为的桥接信号"，**这就是地位声明**。audit 把"未独立成段"误读为"未做地位声明"。
2. **议案 #5 A 形式定义作用域**：audit 把 line 142 位阶澄清 + line 173 末尾 src/ 边界视作"没说"，挑出来当修宪议案；实际宪法已自洽。这是 audit 重读不充分的典型。
3. **议案 #2-A 修宪枚举工具**：把"宪法 enumerate 19 工具"和"P15 加 ror-typecheck 未追"混为一谈；正确的判定应该是"宪法不该 enumerate"（结构性修缺陷），而不是"每加一工具就修宪补一行"（让宪法变实施跟踪表）。

### 总体结论

Audit-1 是**事实级合格**的审查——绝大多数行号引用、现象描述准确，14 条高严重 challenge 中 8 条**完全成立**，5 条**部分成立**，0 条**事实错**。但 audit 在**推论层 / 建议层**有系统性偏差：

- 把"实施细节漂移"过度推论为"需要修宪"
- 修复建议偶尔违反项目长期记忆精神（硬编码、premature extensibility）
- 漏读 principles.md 关键段（line 142 位阶澄清 / line 173 src/ 边界 / line 188 wrapper.sh 桥接信号地位）导致挑出一些假问题

合理的下游处理：

1. 接受 8 条完全成立 challenge：直接 detailed-design / architecture / tool-integration 局部 fix（#P-1 去 enumerate、#P-X1 道门数、#A-3 + #D-4 env 注入契约、#D-1 工具示例、#D-3 `${VAR}` 替换）
2. 部分接受 5 条部分成立 challenge：按 counter 修正后的方向落实，不全按 audit 原建议
3. 否决 4 条修宪议案：全部前提不成立；前 3 条议案的现象在 detailed-design / architecture 层处理就足够，议案 #5 则误读现有原文。

P11-P16 引入物（wrapper.sh / `${TS_*}` 替换 / runnable corpus / TS_ENTRY_FN 注入 / ror-typecheck）的整体落点应该是：

- **宪法层**：保持现状（line 188 已含 wrapper.sh；line 41-42 已含 version_command / host info；A 形式定义已涵盖 schema 扩展）—— **不修宪**
- **架构层**：决策表加 1-2 行（`${VAR}` 替换 / env 注入契约）；接口规约加 env 契约段；模块切分图补 host.rs / 可选 wrapper.sh
- **细化层**：§一 加 `${VAR}` 替换机制描述；§四 加 step 5b env strip + inject TS_ENTRY_FN/TS_TARGET_CRATE；补 host.rs 函数级规约；§五 line 335 改措辞为"仅典型示例"；report.md 示例同步实际 report.rs
- **工具集成层**：tool-integration.md line 30 改"6 道门"；§五 章节 2 加"反作弊证据"小句

整体工作量在"局部 fix"范围，无需触及宪法。
