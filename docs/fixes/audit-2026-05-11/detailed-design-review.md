# 细化设计审查 — `docs/design/detailed-design.md`

> 综合审计 `audit-2026-05-11` / 第 1 组（宪法 + 设计原则文档层）
> 审查日期：2026-05-11
> 审查范围：`docs/design/detailed-design.md`

---

## §1 问题意识

### 这份文档要审查什么？

`docs/design/detailed-design.md`（580 行）是函数级细化层，按其自身 line 3 定位：「架构层在 architecture.md。本文给出 schema 完整定义、runner 函数级前后置、运行时伪码、所有已集成工具的完整配置实例。**约束**：本文不改变架构层确定的模块划分、接口规约和核心流程；如发现需要变更须回到架构阶段」。

**章节**：§一 Schema（双轨 + tool.toml + harness.rs.tera）/ §二 ID 派生 + 文件名 sanitize / §三 Runner 函数级规约 / §四 运行时机制 / §五 已集成工具配置示例 / §六 输出字段 / §七 错误处理策略。

### 为什么要审查？

1. **细化层是 design ↔ 实施的最后桥梁**——必须与 runner/src/*.rs 的实际行为对齐
2. **P11-P16 实施的具体机制**（wrapper.sh / `${TS_*}` 替换 / TS_ENTRY_FN 注入 / kani 5-marker / ror gate 6）是否吸纳到细化层？
3. **§五 工具配置示例**是否反映当前 tool.toml 实际状态？
4. **runner/src/discover.rs / exec.rs / main.rs / host.rs / report.rs** 的所有功能是否都被细化层规约覆盖？

### 用什么方法 / 角度审查？

- 对照 runner/src/*.rs 每个函数 → 找细化层规约的覆盖
- §一 Schema → 找与实际 hirusttest.toml / tool.toml 字段集的对齐
- §四 运行时机制 → 找实际 exec.rs / main.rs 流程的对齐
- §五 工具配置示例 → 找与实际 tools/*/tool.toml 的对齐
- §六 输出字段 → 找与实际 results.json / report.md 的对齐

---

## §2 审查方法

### 参照源

- `docs/design/detailed-design.md` 全文
- `runner/src/{discover,exec,main,host,report}.rs` 全文
- `tools/*/tool.toml`、`tools/*/<name>-wrapper.sh` 实际配置
- `docs/design/{principles,architecture,tool-integration}.md` 上游对照

### 恶意角度

- 假设细化层与 runner 实际不符（漏覆盖、过覆盖、规约错误）
- 假设细化层引入了上游不支持的新假设
- 假设 §五 工具示例是写作时刻的快照，实施已进化但示例没追
- 假设 §六 输出字段示例与 results.json 实际产物不一致

### 诚实底线

每条问题给出文件路径 + 行号 + 引文 + 推理链。

---

## §3 审查现象

### #1（严重度: 高）—— §五 工具配置示例与实际 tool.toml 全面脱节

**现象**：

#### Kani

`detailed-design.md` line 363-367 示例：

```toml
command      = ["cargo", "kani", "--only-codegen", "--bin", "__ts_harness"]
timeout_secs = 120
```

实际 `tools/kani/tool.toml` line 27-29：

```toml
command         = ["${TS_PROJECT_ROOT}/tools/kani/kani-strict-wrapper.sh"]
timeout_secs    = 120
version_command = ["cargo", "kani", "--version"]
```

差异：
- 示例直调 `cargo kani --only-codegen --bin __ts_harness`
- 实际通过 `kani-strict-wrapper.sh` 包装（按 P14 引入，反 5-marker 漏报）
- 示例无 version_command；实际有
- 示例无 `${TS_*}` 替换；实际有

#### Charon-poly

`detailed-design.md` line 396-400 示例：

```toml
command      = ["/tmp/ts-tools-install/charon/bin/charon", "cargo", "--abort-on-error", "--", "--lib", "--target", "aarch64-apple-darwin"]
timeout_secs = 600
```

实际 `tools/charon-poly/tool.toml`：

```toml
command      = ["${TS_CHARON_BIN}", "cargo", "--abort-on-error", "--print-llbc", "--", "--lib", "--target", "aarch64-apple-darwin"]
timeout_secs = 600
version_command = ["${TS_CHARON_BIN}", "version"]
```

差异：
- 硬编码 `/tmp/ts-tools-install/charon/bin/charon` vs `${TS_CHARON_BIN}` 环境变量替换
- 多 `--print-llbc` flag
- 无 version_command vs 有

#### Prusti

`detailed-design.md` line 425-441 示例（含 hardcoded env list with `${PRUSTI_*}` placeholders）

实际 `tools/prusti/tool.toml`：使用 `${TS_PRUSTI_*}` 系列 + `${TS_PROJECT_ROOT}/tools/prusti/prusti-strict-wrapper.sh` wrapper（按 P12 引入）

**违反 / 嫌疑**：

- §五 line 335 写「19 个工具的完整 `tool.toml` / `harness.rs.tera` 配置 + 安装步骤详见各工具自己的 `tools/<name>/README.md`。下面列每个工具的关键 quirk」——本意是列 quirk
- 但实际列出的 6 个示例（cargo-check / Kani / MIRI / Charon / Prusti / Creusot）的"示例 tool.toml"都已**与实际 tool.toml 不同步**
- 这违反 CLAUDE.md "下游不破上游 / 与实施对齐"原则——细化层提供的"示例配置"应是实施事实的反映

**推理链**：

- 细化层目的之一是"让读者通过文档复现实施"
- 当前示例脱离实施，读者按示例操作会得到与实际不同的工具行为
- §五 应或全部更新示例 / 或明示"以下示例是 v1 历史快照，权威配置见各工具 tools/<name>/tool.toml"

**决策性**：☐ 非决策点（同步实施 or 历史快照标注）

**建议**：把 §五 line 335 改为：

```
19/20 个工具的完整 `tool.toml` / `harness.rs.tera` 配置 + 安装步骤详见各工具自己的 `tools/<name>/README.md`——本文不重复完整配置（避免示例与实施漂移）。下面仅列每个工具的关键 quirk（schema 怎么用、为什么这么用）。完整 schema 见 §一；以下 tool.toml 片段为示意（实际配置含 `${TS_*}` 环境变量替换、wrapper.sh 等机制，参见 §四 单次执行单元）。
```

并把 6 个示例配置的 `command` 改为示意性 placeholder（如 `["${TS_KANI_WRAPPER}"]`），同时引用 tool.toml 的实际位置。

---

### #2（严重度: 高）—— §五 缺 14 个工具的 quirk 描述

**现象**：

`detailed-design.md` § 五"已集成工具配置示例" line 333-475 包含 6 个工具的 quirk：cargo-check / Kani / MIRI / Charon (poly + mono) / Prusti / Creusot。

实际 tools 数量：20 个（含 rocq-of-rust-typecheck）。

缺漏 14 个：Verus / VeriFast / Soteria / kmir / hax-lean / hax-fstar / hax-coq / aeneas-lean / aeneas-coq / aeneas-fstar / aeneas-hol4 / rocq-of-rust / rocq-of-rust-typecheck

**违反 / 嫌疑**：

- §五 line 335 明文「下面列**每个工具**的关键 quirk」——但实际只列 6 个
- 缺漏的 14 个里有不少含独特 quirk：
  - Verus：`entry_mode = "lib"` + harness 内 `mod __ts_inner` 必须写在 `verus! {}` 块内（反作弊典型，宪法 §六-4 直接引用）
  - VeriFast：`-skip_specless_fns` flag 同样属"前端切割"宪法 §六-1 落点
  - rocq-of-rust：6 道门 oracle + N 次重试 wrapper（principles.md line 95 引用）
  - aeneas 4 backend：CHARON_BIN + AENEAS_BIN 环境变量 + wrapper.sh 模式
  - hax 3 backend：cargo hax + 产物 grep oracle

- §五 缺漏这些 quirk 让读者无法溯源关键工具的实施依据

**推理链**：

- §五 line 335 承诺"每个工具的关键 quirk"
- 但只兑现了 6/20
- 严格按 CLAUDE.md 文档优先原则，承诺与兑现的差距应消除

**决策性**：☐ 非决策点（要么补全 quirk，要么修改 line 335 措辞）

**建议**：line 335 改：

```
本文 §五 仅列**少数典型 quirk**作为方法学示例（cargo-check / Kani 反作弊 / MIRI 解释执行 / Charon 翻译类 / Prusti env 注入 / Creusot entry_mode lib），其余工具的完整 quirk 见各工具 tools/<name>/README.md。
```

或：补完 14 个工具的 quirk 描述（工作量大，不推荐）。

---

### #3（严重度: 高）—— `${VAR}` 环境变量替换机制未在 §一 / §三 / §四 描述

**现象**：

`runner/src/discover.rs` line 294-335 实现了 `expand_env()` 函数：把 tool.toml 的 `command` / `version_command` 中的 `${VAR_NAME}` 替换为运行时进程环境变量值（用户通过 `source .env` 或类似方式提供）。

但 `detailed-design.md` 全文 grep `\${VAR\|expand_env\|环境变量替换` → **0 hits**。

**违反 / 嫌疑**：

- 这是 runner 接受 tool.toml 的核心机制
- 实际所有 wrapper.sh 工具（9 个 tool.toml）都依赖这一机制
- 细化层完全没说，读者按细化层 §一 Schema 写新工具时**不知能用 `${VAR}` 替换**

**推理链**：

- 细化层 §一 line 116-130 描述 tool.toml schema 的每个字段语义——但没说"command / version_command 字段中的 `${VAR}` 会被 runner 替换"
- 这是隐式行为，对集成者必须显式告知

**决策性**：☑ **决策点** —— 需要用户审议：

- 是否在细化层 §一 加 `${VAR}` 替换机制描述？
- 用户曾说过`feedback_no_premature_extensibility`（不预设可扩展性）——这条机制是已实施物，不是预设，应加

**建议**：§一 line 130 后加：

```
**环境变量替换**（runner 实现的隐式机制）：

`command` 与 `version_command` 字段中的 `${VAR_NAME}` 序列在 runner 加载 tool.toml 后被替换为运行时进程环境变量 `VAR_NAME` 的值（用户通过 `source .env` 或类似方式提供；变量名需匹配 `[A-Z_][A-Z0-9_]*`；缺失变量替换为空串）。

**典型应用**：

- 工具路径：`["${TS_VERUS_BIN}", "--no-verify", ...]`
- wrapper 路径：`["${TS_PROJECT_ROOT}/tools/kani/kani-strict-wrapper.sh"]`
- env 注入：`["env", "JAVA_HOME=${TS_PRUSTI_JAVA_HOME}", ...]`

**派生原则**：避免 tool.toml 写绝对硬编码路径（C 异质性归配置的延伸 —— 路径属于"当前主机环境"维度而非"工具异质性"维度）。
```

---

### #4（严重度: 高）—— TS_ENTRY_FN / TS_TARGET_CRATE 子进程注入未在 §四 单次执行单元描述

**现象**：

`runner/src/exec.rs` line 165-179 实现 env_remove all TS_* + env_set TS_ENTRY_FN + TS_TARGET_CRATE：

```rust
let ts_vars: Vec<String> = std::env::vars()
    .map(|(k, _)| k)
    .filter(|k| k.starts_with("TS_"))
    .collect();
for k in &ts_vars {
    command_builder.env_remove(k);
}
// These are set AFTER env_remove so they survive into the child env.
command_builder.env("TS_ENTRY_FN", entry);
command_builder.env("TS_TARGET_CRATE", &crate_ident);
```

但 `detailed-design.md` § 四 line 274-306 单次执行单元伪码：

```
1.  exec_id = sanitize(...)
2.  work_dir = run_dir/work/<exec_id>/
3.  copy_dir_excluding(example.root, work_dir, ...)
3b. patch_cargo_deps (...)
3c. 目录轨 stub 注入 ...
4.  cd work_dir/<target_path>/
5.  渲染 harness.rs.tera ...
6.  Command::new(...)
6a. 双线程 read_to_end
6b. child.wait_timeout
6c. join ...
7.  写 raw 三文件
8.  remove_dir_all(work_dir)
```

完全没说 env strip + TS_ENTRY_FN / TS_TARGET_CRATE 注入。

**违反 / 嫌疑**：

- 这是 runner ↔ wrapper.sh 的核心接口契约
- 实际 `rocq-of-rust-wrapper.sh` line 72-76 显式依赖 `TS_ENTRY_FN`
- 细化层完全没说会让未来集成者写新 wrapper 时不知能用什么环境变量

**推理链**：

- §四 line 296 后应明示"spawn 前 env_remove TS_*" + "env_set TS_ENTRY_FN / TS_TARGET_CRATE"
- 这与 architecture-review.md #3 同源（架构层接口规约也缺）

**决策性**：☑ **决策点**（与 architecture #3 同源） —— 需要用户审议这两个内部 env 是否作为正式契约

**建议**：§四 单次执行单元 step 6 之前加 step 5b：

```
5b. 准备子进程环境变量：
       - 从 runner 进程环境中 strip 所有 TS_* 变量（防止 runner-internal 配置
         漏入工具进程；典型 prusti 把任意 PRUSTI_* 视作配置 flag）
       - 注入 TS_ENTRY_FN = <entry fn name>
       - 注入 TS_TARGET_CRATE = <crate ident，等于 example.crate_name 把 - 替换为 _>
       - 其他 env (PATH / HOME / shell 等) 从 runner 进程继承
```

并在 §一 tool.toml 描述后加"wrapper.sh / oracle 脚本可访问 TS_ENTRY_FN / TS_TARGET_CRATE 两个 runner 注入的环境变量"。

---

### #5（严重度: 中）—— host 模块完全未在 §三 函数级规约中描述

**现象**：

`runner/src/host.rs` 实现：
- `collect()` —— 采集 hostname / os / arch / kernel / cpu / memory / num_cpus
- `iso8601_utc(t: SystemTime)` —— ISO 8601 时间格式化
- `capture_version(version_command: &[String])` —— 执行 version_command 取 stdout/stderr 截断 500 字节

但 `detailed-design.md` § 三 line 156-258 函数级规约只列：
- `discover::find_examples`
- `discover::find_tools`
- `exec::execute`
- `exec::patch_cargo_deps`
- `report::write_results_json` / `write_report_md`

完全没列 host 模块的 3 个函数。

**违反 / 嫌疑**：

- 细化层 §三 line 154 说「按 /workflow §3.1 书写。函数级细化见 detailed-design.md §三」
- 但 §三 漏了 host.rs 整个模块

**推理链**：

- host.rs 提供的元数据是 results.json 的核心组成（meta.host / meta.tools.*.version）
- principles.md §三 line 33-46 强调"工具版本 / 测试环境 / 测试样例特征"——这正是 host.rs 实现
- 细化层缺失这部分让读者无法溯源元数据采集行为

**决策性**：☐ 非决策点（补 host 函数级规约）

**建议**：§三 加：

```
### `host::collect() -> HostInfo`

后置条件：
  - 返回 HostInfo 含 hostname / os / arch / kernel / cpu_brand / total_mem_mb / num_cpus
  - 字段缺失时为 None（仅 hostname / kernel / cpu_brand / total_mem_mb / num_cpus 可缺）
  - os / arch 始终非空（从 std::env::consts 取，永不失败）

副作用：仅读系统信息

### `host::capture_version(version_command: &[String]) -> Option<String>`

前置条件：
  - version_command 可能为空数组

后置条件：
  - version_command 为空 → None
  - 执行失败 → None
  - 优先取 stdout；stdout 空则取 stderr
  - trim 后超过 500 字节截断并附加 "..."；trim 后为空 → None
  - 否则返回 Some(版本字符串)

副作用：spawn 一个短暂子进程
```

---

### #6（严重度: 中）—— report.md 输出示例与实际 report.rs 行为脱节

**现象**：

`detailed-design.md` § 六 line 545-563 report.md 示例：

```
## vec
| entry              | cargo-check | kani    | miri    | ... |
|--------------------|-------------|---------|---------|-----|
| basic-push-pop/push_pop_seq | SUCCESS | SUCCESS | SUCCESS | ... |

## concurrency
...

---
Total: 260 succeeded / 41 failed / 0 unknown / 301 total
```

但实际 `runner/src/report.rs` line 112-271 生成的 report.md 包含：

1. **Run metadata 段**（line 116-136）：started / finished / duration / host / memory cores
2. **Tool versions 表**（line 137-149）
3. **Per-tool summary** 段（line 155-190）：n / S / F / U / TO / rate / avg / p50 / p90 / max 按 SUCCESS rate 排序
4. **Per-feature summary** 段（line 193-220）：tasks / S / F / rate 按 SUCCESS rate 排序
5. **Per-feature × per-tool 矩阵**（line 222-263）
6. **Total 汇总**（line 265-271）

细化层示例完全没说 1-4 这四段，只有 5 + 6 与示例对应。

**违反 / 嫌疑**：

- 细化层提供的 report.md 示例**严重落后**于实际 report.rs 行为
- P5/P6 实施了元数据 + summary 表，但细化层没追

**推理链**：

- 细化层 §六 line 544 写「按 feature 分组的工具 × entry 矩阵」——这是错的，实际还有元数据 + 双重 summary
- 读者按细化层规约写测试或工具消费 report.md 时会漏读 summary 段

**决策性**：☐ 非决策点（同步示例）

**建议**：§六 report.md 示例改：

```
按 feature 分组的工具 × entry 矩阵，前置元数据段 + 双重 summary 段：

# Run run-1778071324-9549

## Run metadata

- **started**:  `2026-05-07T03:54:21Z`
- **finished**: `2026-05-07T04:02:11Z`
- **duration**: 470 s wall
- **host**:     `host` (macos / aarch64 25.4.0, Apple M5)
- **memory / cores**: 24576 MB / 10 cpus (parallelism = 10)

### Tool versions

| tool | version |
|---|---|
| kani | cargo-kani 0.67.0 |
| ... | ... |

## Per-tool summary

Sorted by SUCCESS rate. Duration columns are ms (avg / median / p90 / max). Time fields are environmental — not a tool-quality score.

| tool | n | S | F | U | TO | rate | avg | p50 | p90 | max |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| ... |

## Per-feature summary

...

## Per-feature × per-tool (entry × tool matrix follows below)

## vec

| entry              | cargo-check | kani    | ... |
| ... |

---
Total: 260 succeeded / 41 failed / 0 unknown / 301 total
```

---

### #7（严重度: 中）—— `[runnable.<fn>]` 段在 §一 描述但未在 §四 单次执行单元体现

**现象**：

`detailed-design.md` § 一 line 30-80 详细描述了 `[runnable.<entry_fn>]` 扩展段，含字段语义、类型支持矩阵、纯内部化判定准则。

但 § 四 单次执行单元 line 274-306 完全没说 runner 对 runnable entry 的特殊处理。

line 78 注："这条由消费 runnable 的工具（如未来的 `tools/hax-lean-eval/`）在 wrapper 内自筛——runner 不感知 runnable 标记"。

**违反 / 嫌疑**：

- "runner 不感知 runnable" 是合理的设计选择（按 principles A-4 wrapper 是集成者描述工具行为）
- 但 § 一 用了 50 行描述 `[runnable.*]` 段——读者会期待 § 四 也对应描述
- 实际 § 四 没有任何对应行——这是细化层的不对称

**推理链**：

- 当前细化层 line 78 显式说"runner 不感知"——这是有意为之的设计选择
- 但 § 一 不应让读者期待 § 四 的对应支持
- 建议在 § 一 line 31 加注："本段定义的 schema 仅 wrapper / oracle 消费，runner 自身不感知（见 §四 注）"

**决策性**：☐ 非决策点（指引性补注）

**建议**：§一 line 31 改：

```
##### `[runnable.<entry_fn>]` 扩展段（专为档 3 一致性测试预留，可选）

**位阶**：runner 自身不感知此段；由消费 runnable 的工具（如未来的 `tools/hax-lean-eval/`）在 wrapper / oracle 内自筛（A-4 集成者描述工具行为的延伸）。
```

---

### #8（严重度: 中）—— § 四 "异常" 节与 § 七 "错误处理策略" 表 部分重复 + 部分不一致

**现象**：

`detailed-design.md` § 四 line 320-322 「异常」：

```
子进程被信号杀（SIGSEGV / SIGABRT 等）→ 非零 exit → FAILED。runner 自身 IO / 渲染 / spawn 失败 → 该任务标 UNKNOWN（不归工具责任），整 run 继续，错误信息记入 `results.json` 的 `error` 字段。cleanup work_dir 失败 → 仅 log warning，不归类——cleanup 失败不影响已捕获 raw outputs；残留目录在整次 run 结束时随 `runs/<ts>/` 整体可删。
```

§ 七 line 571-580 错误处理策略表：

```
| Schema 解析失败（Cargo.toml / hirusttest.toml / tool.toml） | runner 启动时 panic（discover 阶段错——配置 bug，不该静默） |
| 单 task 内 IO / cp / 渲染 / spawn 失败 | 任务标 UNKNOWN，整 run 继续 |
| 子进程退出非零 | 任务标 FAILED + exit_code |
| 子进程被信号杀（SIGSEGV 等） | 任务标 FAILED + exit_code = None |
| 子进程超时 | runner 发 kill(-pgid, SIGKILL)，任务标 FAILED + timed_out = true |
| cleanup work_dir 失败 | 仅 log warning，不影响任务结果 |
```

**违反 / 嫌疑**：

- § 四 line 320-322 "异常"与 § 七 line 571-580 表内容相近但措辞不同
- § 七 表多了"Schema 解析失败 → panic"和"超时 → SIGKILL"两条
- § 四 没说 schema 解析失败的处理
- 同一行为在两处描述是冗余 + 风险（一处改另一处不改）

**推理链**：

- 细化层应一处定义、另一处引用
- 当前 § 四 line 320 + § 七 line 571 是双源——维护风险

**决策性**：☐ 非决策点（合并 / 一处引用）

**建议**：§ 四 line 320 简化为：「异常处理详见 § 七 错误处理策略。」

---

### #9（严重度: 低）—— `discover::find_examples` 规约缺 `.no-hirusttest` 屏蔽完整描述

**现象**：

`detailed-design.md` line 162-168：

```
双轨发现（按 architecture §四 discover 模块功能规约）：
  对每个候选 <example_dir>（递归扫描 + .no-hirusttest 屏蔽），按以下顺序判定：
    1. .hirusttest/config.toml 存在 → 走目录轨
       schema_kind = Directory
       辅助文件目录 = <example_dir>/.hirusttest/
    2. 否则 hirusttest.toml 存在 → 走单文件轨
       schema_kind = SingleFile
       无辅助文件目录
    3. 否则 <example_dir> 不是 example，跳过
    4. .hirusttest/ 与 hirusttest.toml 同时存在 → 报 Err（二义性）
```

实际 `runner/src/discover.rs` line 71 + 104-119：`.no-hirusttest` 屏蔽是"目录及其整个子树跳过"。但 detailed-design 仅一行附带提，未说"子树跳过"特性。

**违反 / 嫌疑**：

- 细化层应明示这一关键行为
- 这与 architecture-review.md #5 同源

**决策性**：☐ 非决策点（小补）

**建议**：line 163 改为：

```
双轨发现（按 architecture §四 discover 模块功能规约）：
  对每个候选 <example_dir>，按以下顺序判定：
    - 屏蔽：若 <example_dir> 含 .no-hirusttest 文件，则该 dir 及其整个子树被跳过（typical use case: vendor/ 子项目含工作区内嵌不该被当作 example 的目录）
    1. .hirusttest/config.toml 存在 → 走目录轨
    ...
```

---

### #10（严重度: 低）—— `target_path` 描述与实际 schema_kind = Directory 行为有歧义

**现象**：

`detailed-design.md` line 88：

```toml
# 核心字段，与单文件轨同
entries = ["fn_name_1", "fn_name_2"]
target_path = "src"                     # 可选，默认 "."
```

但 line 21（单文件轨）：「target_path = "subdir"                  # 可选，默认 "."」

**违反 / 嫌疑**：

- 单文件轨示例 target_path = "subdir"（普通子目录）
- 目录轨示例 target_path = "src"——但 vendor/ 综合项目通常以 vendor/<crate>/ 为 example dir，其 Cargo.toml 在 example dir 根而非 src/
- "src" 作为 target_path 在目录轨示例中**有歧义**——若 vendor/x509-parser/Cargo.toml 在根，target_path 应是 "."；若 target_path="src" 暗示 Cargo.toml 在 vendor/x509-parser/src/，这极不寻常

**推理链**：

- target_path 是相对 example dir 的子目录，指向被测 crate
- 单文件轨 / 目录轨语义应一致
- 目录轨示例的"src"读起来易引发歧义

**决策性**：☐ 非决策点（措辞小修）

**建议**：line 88 改 target_path = "."（默认） 或 target_path = "rsa"（vendor/rsa 这种实际项目子目录）—— 避免"src"歧义。

---

### #11（严重度: 低）—— `runner report <run-dir>` 子命令章节位置略偏

**现象**：

`detailed-design.md` § 四"runtime 机制"含"runner report <run-dir>" 子命令在 line 324-331。这一段实际描述的是 main.rs 的一个子命令路径，与 § 四 "总体调度 / 单次执行单元 / 隔离 / 并发 / 超时 / 异常"等并列——但 "report 子命令"是 main.rs 的辅助路径而非主调度逻辑。

**违反 / 嫌疑**：

- 章节结构略不对称——子命令应单列 § 或归到 § 三 main / report 函数级规约下
- 不是错，是次序不齐整

**决策性**：☐ 非决策点（章节小调整）

**建议**：把 "runner report <run-dir>" 子命令移到 § 三 函数级规约下，或单建 § 四之下的子小节。

---

## §4 决策点 vs 非决策点汇总

### 决策点（需用户审 / 拍板）

| # | 摘要 |
|---|---|
| #3 | `${VAR}` 环境变量替换机制是否在 §一 正式纳入（与上游 design 主线一致） |
| #4 | TS_ENTRY_FN / TS_TARGET_CRATE 注入是否作为正式接口契约（与 architecture-review #3 同源） |

### 非决策点（局部 fix）

| # | 摘要 |
|---|---|
| #1 | §五 工具示例 6 个全部与实际 tool.toml 同步 / 或加"历史快照"标注 |
| #2 | §五 14 个缺漏工具 quirk 处理（补全 or 改 line 335 措辞为"仅典型示例"） |
| #5 | 补 host.rs 函数级规约 |
| #6 | report.md 输出示例同步实际（元数据 + 双重 summary） |
| #7 | §一 `[runnable.*]` 段加"runner 不感知"位阶注 |
| #8 | §四 "异常"节与 §七 表合并去重 |
| #9 | discover 规约补 .no-hirusttest 子树屏蔽完整描述 |
| #10 | 目录轨 target_path = "src" 示例改为 "." 或更典型例 |
| #11 | "runner report <run-dir>" 子命令章节位置 |

---

## §5 审查结论

### 总体判断

`detailed-design.md` 主体规约完备、Schema 定义精细——双轨 schema、`[runnable.*]` 段、ID 派生 + sanitize、双进程组 SIGKILL 等核心机制都有规约。但存在两大类问题：

1. **实施漂移**（高严重度集中）：§五 工具配置示例（6 个 + 缺 14 个）全面与 tools/*/tool.toml 实际状态脱节；`${VAR}` 替换机制 / TS_* 子进程注入 / host.rs 模块 / report.md 输出实际行为四类 P11-P16 引入物未在细化层吸纳。这违反 CLAUDE.md "下游文档应与实施对齐"原则。
2. **结构性小问题**：discover `.no-hirusttest` 子树屏蔽完整描述缺；§四"异常"与 §七 表重复；目录轨 target_path 示例歧义；`runner report` 子命令位置不齐整。

### 严重度分布

- 高严重度：4 条（#1 + #2 + #3 + #4）
- 中严重度：4 条（#5 + #6 + #7 + #8）
- 低严重度：3 条（#9 + #10 + #11）

### 关键风险

最严重风险是 **#3 + #4**：runner 实施的两个核心机制（`${VAR}` 替换 + TS_ENTRY_FN/TS_TARGET_CRATE 注入）在细化层完全无规约。这两个机制是新工具集成者必须知道的——但当前任何新集成者按细化层 §一 Schema 写新 tool.toml / wrapper.sh 时**无法从文档得知**能用这些机制。

次严重风险是 **#1 + #2**：§五 工具配置示例与实际 tool.toml 脱节。读者按细化层示例操作会与实施不符；这违反"细化层应让读者通过文档复现实施"的根本目的。

### 与上游 design 的派生关系

整体派生顺畅。**没有发现细化层引入违反 architecture / principles 的硬性新假设**——但 §一 `[runnable.*]` 段（line 30-80）的引入虽满足 principles A 形式定义（cargo 不读 hirusttest.toml），但未在 principles / architecture / tool-integration 三份上游文档明示承认（见 principles-review.md #X3）。这条不破宪法但缺上游确认。
