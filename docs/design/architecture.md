# 架构设计：Rust 验证工具特性覆盖测试框架

> **前置约束**：本文必须 100% 满足 [`principles.md`](principles.md) 的宪法级精神。如本文与原则冲突，改本文不改原则。
>
> **本文位置**：design 层的"lib 入口"——按 `/workflow` §2.2 规范书写架构设计；索引下游模块文档。函数级细化与 schema 见 [`detailed-design.md`](detailed-design.md)。

---

## 一、设计推导：原则如何落到结构

[`principles.md`](principles.md) 的三大原则（A 双方不可侵入 / B 测必要条件非语义对错 / C 异质性归配置）落到架构层面后产生三件具体设计选择：

1. **原则 A 投影（运行原子 = 单 entry）**：一次操作对应唯一 (tool, entry) 三元组——一次隔离副本、一次 harness 渲染、一次工具进程、一份 raw output。代价是 cargo 编译缓存不能跨 entry 共享，但换来每个 entry 独立可观测、互不污染。
2. **原则 A ∩ B 投影（能力靠观测）**：tool 配置里禁止 `supported_kinds = [...]` 这类字段——所有 (tool, entry) 组合都跑，能力靠观测，不靠声明。
3. **原则 C 投影**：runner 代码不为某工具开后门——所有差异通过 `tool.toml + tera 模板`声明。

这三条投影共同约束模块切分。

### A 位阶澄清的具体落地

原则 A 的天真读法（样例对所有工具完全无感）在 Creusot 这种工具面前破——cargo-creusot 把 RUSTC 替换成 creusot-rustc 喂给 cargo，要求样例 lib 自身 `use creusot_std::prelude::*;` 且 `Cargo.toml` 字面列出 `creusot-std` dep。

A 的位阶澄清把承诺从天真版降级为：**"原始磁盘字面零修改 + 隔离副本上声明式工具特定填充"**。具体落到两个声明式机制：

- `extra_cargo_deps` 字段：runner 用 toml AST 把 dep 行 inject 到副本 `Cargo.toml` 的 `[dependencies]` 表
- `entry_mode = "lib"` 字段：harness 取代副本 `src/lib.rs` 当顶级 lib，原 lib 改名 `__ts_inner.rs` 内嵌为 mod

两机制都是 tool.toml 字段，runner 读字段做事——不破原则 C。

### 双轨 example schema：法律层面允许外部综合项目

按 [`principles.md`](principles.md) §四 A 形式定义下的"双轨 schema"，example 配置分两轨：

```
单文件轨（简单单特性测试，默认）：
  examples/<feature>/<dir>/
  ├── Cargo.toml
  ├── src/lib.rs
  └── hirusttest.toml          ← 信号文件

目录轨（外部综合项目，仅适用 vendor/ 下的真实 codebase）：
  vendor/<crate>/   或   examples/<feature>/<dir>/
  ├── Cargo.toml                  （原项目自带）
  ├── src/                        （原项目自带）
  └── .hirusttest/
      ├── config.toml             ← 主配置（必有）
      └── <可选辅助文件>          ← per-entry verifier stub / proof annotation snippet 等
```

两轨都满足 A 的形式定义——cargo / rustc 都不读 `hirusttest.toml` 与 `.hirusttest/`。

**派生原因**：

外部综合项目（如 vendor/x509-parser、vendor/openssl）作为 entry 来源有几个本质特征单文件轨无法承载：

1. **多 entry 独立配置**：同一个 vendor crate 可能选取多个独立 entry，每个 entry 涉及不同 verifier 配置 / 不同 spec 文件
2. **辅助文件**：某些 verifier（如 creusot 的 `creusot_contracts`）需要 per-entry 的 spec stub；这些 stub **不能放进** vendor crate 的 src/（违反原则 A 形式定义），但**可以放进** `.hirusttest/`（cargo 不读，对 vendor crate 行为不构成侵入）
3. **预留扩展空间**：外部综合项目接入往往需要 per-entry 的 harness 模板覆写、per-tool 的命令调整等；目录轨给这些扩展留位

**discover 双轨发现责任**（在 runner 模块切分中由 discover 模块实现，详见 §四）。

### 原则 B 在结果分类上的扩展

SUCCESS / FAILED 的二分初看够用：exit 0 = SUCCESS，非 0 = FAILED。

但 P27 修宪（2026-05-12 引入"不公信"根本问题 + 三原则栈本地性 / 社区惯例 / 最大善意）后，分类扩出 **UNKNOWN** 投影两条根本问题：

- **"不公平"侧投影**：runner-internal 错误（cp / 渲染 / spawn 失败）的失败如果归 FAILED，下游会读成"该工具不能处理该 entry"——但这是 runner 的 bug 不是工具表态。归 UNKNOWN 切干净
- **"不公信"侧投影 + UNKNOWN 严格语义**（principles.md §六）：UNKNOWN 仅两类——
  - (a) **全局工具链崩溃**（用户重装可修，如 verus 缺 `verus-root`、prusti `viper_tools` 丢失）
  - (b) **我们这边可识别问题暂未修**（如我们 harness 模板 bug → `runnable_harness_arg_mismatch`、我们 corpus 引入的 vendored crate lint → `vendor_lint_strictness`、我们环境损坏 → `environment_corruption`）。每类必附**明确归因 + 会修计划**
- **不归 UNKNOWN 一律 FAILED**：工具自身能力边界（如官方 wrapper 失败 / 工具自选 toolchain 不支持新 feature / 单文件 pipeline 不读 Cargo.toml / 官方 wrapper 不传 `--edition`）——按本地性原则 FAILED 站得住，工具开发者不能驳回。这条由 `runner/src/report.rs::classify_external_fault` 实施：P27 起 oracle 仅保留 (b) 子类 3 条规则，原 v5.1 含的 dependency_resolution / toolchain_edition_mismatch / edition_pipeline_propagation 三条已被剃（详 commit `28b4a03`）

三类处于不同语义维度（SUCCESS / FAILED 描述子进程结果，UNKNOWN 描述"我们这边问题或全局环境损坏"）——不破 B 在工具评判层的简洁。

#### §六 当前 crate 焦点的 oracle 投影（P36 + P37 落地）

宪法 §六 "当前 crate 焦点"（宽度切割）入宪后，oracle 实施层按工具输出格式分两条路径落地（详 `tool-integration.md` §四.6）：

- **路径 A（输出带 source path 直接过滤）**：aeneas 4 backend wrapper 的 charon-stage gate（P36）。charon stderr 的 `is not supported` / `^error:` signal 后跟 `--> path` 行——按路径前缀 `/rustc/` / `/cargo/registry/` / `/vendor/` 区分；全 external 则 suppress
- **路径 B（输出无 span，反向证明）**：kani-strict-wrapper.sh 的 5-markers gate（P37）。kani 5-markers 输出只汇总 count 不带 span——用 `os.walk('src/')` + 正则 grep entry crate src 是否含触发关键字（`asm!` / `simd_*` / `catch_unwind` / `::mask(` / `c"` 等）；含则 entry 自用 → FAILED，不含则必来自 deps → SUCCESS
- **路径 C（单文件 pipeline 不读 deps，自然满足）**：verus / verifast / soteria / ror—— oracle 不需要额外 filter

P37 实测影响：kani 通过率 93.8% → **98.8%**（10 FAILED 中 8 case 是 markers 来自 deps，2 case entry 自用保持 FAILED）。

#### bug detect 归 SUCCESS（§四 B 派生）

宪法 §四 B "测必要条件 / 非语义对错"——只问"工具能不能吃下这段代码并产出预期形状的输出"，不问产出对错。

派生：当工具完整跑完前端 + 求解，且**自陈"我在 entry 代码里找到了 bug / UB / violation / counterexample"**时——这是工具最有价值的输出之一。按 §四 B 归 **SUCCESS**。FAILED 只用于"工具不能吃下这段代码"（不支持 / 自身 crash / 翻译失败 / 拒绝 partial）。

适用工具（v6 corpus 中真跑完整语义检查的）：

| 工具 | bug detect 信号 | wrapper 翻转 |
|---|---|---|
| MIRI | stderr `Undefined Behavior` 且无 `unsupported operation` | exit ≠ 0 → SUCCESS |
| soteria | stdout `^bug:` 或 `found issues.*errors in [0-9]+ branch` 且无 `Thread panicked when extracting` | exit 1 → SUCCESS |
| verifast | 当前 corpus 无 `//@ req/ens` 注解 → verifast 不真做 verification → 0 触发 | — |
| kani / verus / creusot / prusti / ror | 项目设计跑前端层（`--only-codegen` / `--no-verify` / 等），不进入求解 → 0 触发 | — |

这条派生与 §六 UNKNOWN 严格语义、§四 A 不可侵入、§六 当前 crate 焦点都不冲突，是 §四 B 在 v6 corpus 上的具体投影。soteria README 历史注释"按完整完成精神 bug detect = FAILED"（v3 时代立场）与 §四 B 实际冲突，已删除。

**对称性论证（防 reviewer 误读）**：bug detect = SUCCESS 给 MIRI / soteria 不是"额外 reward"或"对 verifier-style 工具不公平"。原因：

- **MIRI 是 abstract interpreter** —— 它的工作 nature 就是把 Rust 当解释执行的代码完整跑一遍，**没有"前端 vs 求解层"切点可切**。给一段代码 → 跑完无 issue ∨ 跑完 + 检测 UB 都是它按设计意图工作。**bug-detect 不是它的"额外能力"，是它的有效输出形态之一**。
- **soteria 是 symbolic execution** —— 同理，符号执行整段走完是其本性。
- **verifier-style 工具（kani / verus / prusti / creusot）**按宪法 §六 前端测量 deliberate 切到前端层（`--only-codegen` / `--no-verify` / `PRUSTI_NO_VERIFY=false+PRINT_HASH` / 等）—— **它们本来就不在 bug-detect 路径**，不是被歧视，是项目主动让它们停在更前面，避免 BMC/SMT 求解时间和资源消耗。
- **verifast** 跑完整 verification 但需要用户 spec 注解；本 corpus 0 个 entry 有 `//@` 注解，verifast 不进入真 verification 路径——所以也不进入 bug-detect 路径。

每个工具按它能给出的有效输出形态评 SUCCESS，**对称**。这条对称性在 principles.md §六 "工具输出形态对称性" 也有声明。

---

## 二、通用性论证

**意识 → 论证**：本方法在哪些工具上适用？

工具 T 在嵌入式验证里"被工具吃下样例"这件事，需要在 token 形态上表达"verify 这一段代码"。逐工具看 token 形态：Kani 是 `#[kani::proof] fn p() { ... }`；MIRI 是 `fn main()`；Verus 是 `verus! { fn p() requires/ensures }` 宏块；Prusti / Creusot 是 caller 上的 `#[ensures]` attr；Charon 是任意 fn 入口翻译到 LLBC；MIRAI 是 `mirai_annotations::*` 调用；ESBMC-Rust 是 harness fn + assertion；cargo-check 默认编译无显式 token。

这些 token 都能由两件东西联合表达：**(1) 一个新增的 `.rs` 文件**（runner 渲染到 `src/bin/__ts_harness.rs` 或 `src/lib.rs`），**(2) 工作副本 `Cargo.toml` 的 `[dependencies]` 表上的声明式 append**（仅 Creusot 类工具需要 (2)）。

→ 形式覆盖域：

```
{ tool : token T 可由 (单个新增 .rs 文件 + 工作副本 [dependencies] 声明式 inject) 表达
       ∧ tool 通过 cargo 子命令运行
       ∧ tool 不要求修改样例源码（src/） }
```

**实测覆盖**：cargo-check / Kani / MIRI / Charon (poly + mono) / Prusti / Creusot / Verus / VeriFast / Soteria / kmir / hax (lean/fstar/coq) / aeneas (lean/coq/fstar/hol4) / rocq-of-rust 全在内（19 工具）。MIRAI / ESBMC-Rust 在 token 形态分析上落入此集，未实测。

**反例域**——本方法不覆盖的工具：

- 要求修改 entry 函数自身（Rust 主流生态无此类）
- 不通过 cargo 运行（如直接读单 `.rs` 的独立工具——Rust 主流验证工具普遍以 cargo 为入口）
- 要求 `Cargo.toml` 的 `[features]` / `[patch]` / profile 配置而非仅 `[dependencies]` 行（当前 schema 不覆盖；未来可同范式扩展，例如加 `extra_cargo_features` 字段）
- RustBelt 一类（Coq / Iris 形式化）：人类把代码 transcribe 到 λ-Rust 演算后在 Coq 写证明，无 cargo 入口

**技术风险与防御**：若某工具只分析 post-DCE 后的活码，harness 调 entry 但调用被消除，entry body 未被分析。Rust 主流验证器都在 pre-optimization MIR/HIR 层分析，DCE 后置，稳健。如未来需要，harness 内对 entry 调用结果加 `std::hint::black_box(...)` 抑制 DCE——属 `tool.toml + harness.rs.tera` 调整，不波及框架内核。

---

## 三、模块切分

```
runner/                  Rust binary
├── discover             读 examples/ 与 tools/ 两个目录树，产出 Vec<Example>、Vec<Tool>
├── exec                 单次 (tool, example, entry) 执行：cp → patch → render → spawn → capture → cleanup
├── report               收集 Vec<TaskResult>，写 results.json（机读）+ report.md（人读）
└── main                 CLI、调度、并发池、汇总打印

examples/<feature>/<dir>/    Cargo lib crate + hirusttest.toml — 由用户写
tools/<name>/                tool.toml + harness.rs.tera   — 由集成者写
runs/run-<ts>-<pid>/         每次 run 输出根 — 由 runner 生成
```

**边界**：runner 的代码与 examples / tools 解耦——examples 与 tools 改动均不要求改 runner 代码。这是原则 C 的可观测兑现。

**新增工具的硬指标**：见 [`principles.md`](principles.md) §六。每个新增工具必须在该工具 README 中明确声明 (a) pipeline 阶段图、(b) 前端 / 后端边界落点、(c) `tool.toml` 用什么 flag 实现该切割、(d) SUCCESS 信号 + 形式严格性 + 漏报盲点。

---

## 四、模块功能规约（半形式化）

按 `/workflow` §3.1 书写。函数级细化见 [`detailed-design.md`](detailed-design.md) §三。

```
模块：discover

功能描述：扫描 examples/ 与 tools/（以及作为 example 来源的外部综合项目目录如 vendor/）
         产出 Vec<Example> 与 Vec<Tool>。

双轨 example 发现规则（按 principles §四 A 双轨 schema）：
  对每个候选 <example_dir>，按以下顺序判定：
    - 若 .hirusttest/ 与 .hirusttest/config.toml 都存在 → 走目录轨
    - 否则若 hirusttest.toml 存在 → 走单文件轨
    - 否则 <example_dir> 不是 example
    - 双方同时存在 → 报错 (Err)，不接受这种二义性

前置条件：
  - examples_dir 与 tools_dir 是合法目录路径
  - 任一 <example_dir> 至多含 hirusttest.toml 与 .hirusttest/config.toml 之一
  - 任一 <example_dir>（无论哪轨）的 target_path 指向的目录有合法 Cargo.toml
  - tools/<name>/ 下凡含 tool.toml 的目录均同时含 harness.rs.tera

后置条件：
  - 每个返回的 Example 含 (feature, dir, root, target_path, crate_name, entries, schema_kind)，
    target_path 已 canonicalize 且属于 root 子目录
    schema_kind ∈ { SingleFile, Directory }，记录该 example 来自哪一轨
  - 每个返回的 Tool 含 (name, command, timeout_secs, harness_template, extra_cargo_deps, entry_mode)，
    command 非空、entry_mode 是 Bin 或 Lib
  - 二者按 (feature, dir) / name 排序

副作用：仅文件读取，不写盘
```

```
模块：exec

功能描述：执行一次 (tool, example, entry) 三元组：拷贝隔离副本、按 tool 配置注入 manifest 与 harness、起独立 process group 跑工具、捕获 raw output、应用 timeout、清理。

前置条件：
  - tool 与 example 来自 discover，已通过 schema 验证
  - run_dir 存在且可写
  - entry ∈ example.entries

后置条件（成功路径）：
  - run_dir/work/<exec_id>/ 已创建并最终被删除（best-effort）
  - run_dir/raw/<tool>/<slug>.{stdout, stderr, exit} 三文件已写
  - 返回 ExecResult 含 (status, exit_code, duration_ms, timed_out, raw_stdout_rel, raw_stderr_rel)
  - timed_out = true 蕴含 status = Failed，且 exit 文件内容为 "__ts_timeout (after N s)"

后置条件（失败路径）：
  - 任一步 IO 错误时返回 Err(anyhow)，由 main 转为 UNKNOWN（属"我们这边问题"——runner 自身故障，按 principles.md §六 UNKNOWN 严格语义 (b) 类，应附诊断信息）
  - 子进程 spawn 成功但 exit ≠ 0 → status = Failed（worker 再交给 report::classify_external_fault 检查是否符合 (b) UNKNOWN 子类型，否则保持 FAILED）

不变式：
  - exec_id 由 (tool.name, example.feature, example.dir, entry) 经 sanitize 唯一确定
  - 同次 run 内 exec_id 唯一（路径不冲突）
  - 子进程在自己的 process group 内（process_group(0)）

副作用：cp、写 harness、起子进程、捕获 stdout/stderr/exit、删除 work_dir
```

```
模块：report

功能描述：把 Vec<TaskResult> 落盘为 results.json（机读）+ report.md（按 feature 分组的人读矩阵）。

前置条件：
  - run_dir 存在且可写
  - results 中每个 TaskResult 含合法的 entry_id 形如 "<feature>/<dir>/<entry-fn>"

后置条件：
  - run_dir/results.json 含 (run_id, results[]) 的 JSON
  - run_dir/report.md 含按 feature 分组的工具 × entry 矩阵
  - 输出确定性：相同输入产生相同字节序列（BTreeMap 排序）

副作用：写两个文件
```

---

## 五、模块间接口规约

按 `/workflow` §3.2 书写。

```
接口：discover → exec

输入数据：&Tool, &Example, &str (entry), &Path (run_dir)
输出数据：Result<ExecResult>

协议约定：
  - 调用方（main）保证：tool / example 来自 discover，run_dir 是已 canonicalize 的目录
  - 被调用方（exec）保证：成功路径产生上述 raw 三文件 + 返回结果；
    失败路径返回 Err 不部分破坏 run_dir 内已存在文件（已写的不会回滚）
```

```
接口：exec → report

输入数据：Vec<TaskResult>（每条由 main 从 ExecResult / Err 转换而来）
输出数据：写 run_dir/results.json + run_dir/report.md

协议约定：
  - main 保证：每个 (tool, entry) 三元组转出恰一条 TaskResult
  - report 保证：输入相同时输出字节级一致；status 字段唯一取值集合是 {SUCCESS, FAILED, UNKNOWN}
```

```
接口：runner → external_subprocess

输入数据：tool.command argv、副本目录 cwd、stdio piped（stdout/stderr 到 runner 的 reader threads）
输出数据：subprocess 的 exit_code、stdout 字节流、stderr 字节流、wall-time
约束：subprocess 起在独立 process group（process_group(0)）

协议约定：
  - runner 保证：副本目录是完全隔离的当前 (tool, entry) 专属目录；不并发改它
  - subprocess 由工具决定行为；runner 不解读其内部 protocol，只看 exit code 二值化（SUCCESS = 0，FAILED = 非 0 / 信号 / 超时 SIGKILL）
  - timeout 时 runner 通过 kill(-pgid, SIGKILL) 杀整个 process group（孙子进程一并），等待 child.wait() 收尸
```

---

## 六、并发规约

```
并发单元：exec::execute

共享资源：
  - 文件系统：每个 (tool, example, entry) 独占自己的 work_dir 与 raw/<tool>/<slug>.* 三文件，路径互不重叠
  - cargo registry / target cache：cargo 自身用 flock 保证并发安全，runner 不参与
  - stdout / stderr 终端：rayon 默认 println! 行级原子，不撕字串

锁协议：
  - runner 不持任何显式锁
  - 子进程内部锁由其自身管理（不可见）

顺序约束：
  - 同一 work_dir 内：cp → patch_cargo_deps → render harness → spawn → wait → write raw → cleanup 严格线性
  - 跨 work_dir：完全无序，rayon work-stealing 调度

线程安全性结论：
  - 每个 (tool, example, entry) 任务彼此独立，rayon par_iter 安全
  - 默认并发度 = num_cpus；--parallel N 可覆盖
```

---

## 七、关键设计决策记录

每条决策溯源到一条原则。

| 决策 | 推自 | 状态 |
|---|---|---|
| 样例为独立 cargo lib crate，零工具依赖 | A | 锁定 |
| 入口为零参 `pub fn`，无 parametrised 形态 | A + 测试入口划界 | 锁定 |
| ID 格式 `<feature>/<dir>/<entry-fn>`，无 category | C + Occam | 锁定 |
| 工具集成 = `tool.toml` + `harness.rs.tera` 两文件 | C | 锁定 |
| 标准模板变量 = `target_crate_name`、`entry_fn` 两个 | C | 锁定 |
| 渲染目标默认 `src/bin/__ts_harness.rs`（`entry_mode = "bin"`），可切换 `src/lib.rs` 取代原 lib（`entry_mode = "lib"`，原 lib 内嵌 mod） | A 位阶澄清 + cargo auto-discover | 锁定 |
| `extra_cargo_deps` 用 toml AST inject 到副本 `[dependencies]` 表 | C + 稳健性 | 锁定 |
| 子进程 spawn 独立 process group，timeout 时 `kill(-pgid, SIGKILL)` | 功能性 | 锁定 |
| cp 时跳过 `Cargo.lock`（让 fresh resolve；规避旧 cargo 不识别新版 lockfile） | 功能性 | 锁定 |
| 隔离机制：per-execution `cp -r` 隔离副本 | A | 锁定 |
| 运行原子 = 单 entry，不批跑 | A 的运行时投影 | 锁定 |
| ResultClass 三分（SUCCESS / FAILED / UNKNOWN，UNKNOWN 仅含 §一 不公信侧 (a) 全局工具链崩溃 + (b) 我们这边可识别问题暂未修两类；详 §一 末段） | B + §一 双根本投影 + Occam | 锁定（P27 严格化）|
| 不预跳过任何 (tool, entry) 组合——能力靠观测 | A ∩ B | 锁定 |
| `results.json` 顶部 metadata 段（host / 时间戳 / 各工具 version_command 输出） | 工具非静态原则 | 锁定 |
| CLI `--tool <NAME>` / `--entry <GLOB>` 子集筛选 | UX | 锁定 |
| 性能/资源开销不算问题（除非升级为功能问题） | 设计精神 + Occam | 锁定 |
| 砍掉 external_lib mode、verdict 期望比对、category clean/bug 二分 | Occam | v1 锁定 |
| Oracle 0 误报第一，0 漏报次要（grep-based 工具不可形式证明） | 诚实性宗旨（principles §三-3 诚实测试范围） | 锁定 |
| SUCCESS = 工具完整完成；不允许 partial（即便产物落盘） | 不允许 partial（principles §六-2） | 锁定 |

---

## 八、下游索引

- **函数级细化、Schema 完整定义、运行时机制、19 工具配置示例、输出字段、错误处理策略**：[`detailed-design.md`](detailed-design.md)
- **工具集成原则、README 书写规范、实测报告责任边界**（我方方法学，自我性，外来 tools 无须遵守）：[`tool-integration.md`](tool-integration.md)
- **每个工具的前端边界 / SUCCESS 信号 / 形式严格性**：`tools/<name>/README.md`
- **修正方案文档**：[`../fixes/`](../fixes/)
- **实测报告**：[`../test-reports/`](../test-reports/)
- **调研报告（问题意识来源）**：[`../research/testsuite-research.md`](../research/testsuite-research.md)
