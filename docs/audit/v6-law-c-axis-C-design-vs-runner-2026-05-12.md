# Axis C: 设计文档 → runner 实现一致性 challenge

## 1. 总览

- 候选数：10（C1-C10）
- 涉及文件：
  - design：`docs/design/architecture.md`、`docs/design/detailed-design.md`、`docs/design/tool-integration.md`、`docs/design/principles.md`
  - impl：`runner/src/main.rs`、`runner/src/discover.rs`、`runner/src/exec.rs`、`runner/src/report.rs`、`runner/src/host.rs`

按"design 说 X / impl 是 Y"的具体不一致汇总。挑刺前已按 R1/R2/R5/R6/R11 自筛。落入清单的均是"design 文本与 impl 行为可指证差距"。

---

## 2. 候选清单

### C1：Tera 模板变量数量与 architecture §七"锁定"决策正面冲突

**design 文本**（detailed-design.md:134-141）：

> [Tera](https://keats.github.io/tera/) 模板。可用变量恰好两个：
> `target_crate_name` / `entry_fn`
> 变量集**固定**——不允许 tool config 扩展（破坏原则 C 的"标准词汇"约束）。

architecture.md:274 同一锁定：

> | 标准模板变量 = `target_crate_name`、`entry_fn` 两个 | C | 锁定 |

**impl 代码**（runner/src/exec.rs:108-113）：

```rust
let entry_args: &str = example
    .entry_args
    .get(entry)
    .map(String::as_str)
    .unwrap_or("");
ctx.insert("entry_args", entry_args);
```

实际可用变量为三个：`target_crate_name` / `entry_fn` / `entry_args`。

**不一致**：design 写"恰好两个 / 锁定"，impl 注入第三个。`entry_args` 来源是 `discover.rs::Example::entry_args`（line 36-46），用于 runnable corpus 的 bin-mode 调用站点参数填充。这是 P-runnable / 假阳性审计 §4.1 落地，但 detailed-design §一与 architecture §七决策表均未追加。

**反状态排除**：
- R1 ✗：有具体差距（"锁定两个" vs 实际三个），非现象推断
- R2 ✗：design 没承认这个 trade-off
- R6 ✗：检查 architecture / detailed-design 两处均未同步
- R11 ✗：决策表是结构性陈述，不是文案

**建议**：改 detailed-design §一 Tera 变量表加 `entry_args` 行 + 改 architecture §七决策表"标准模板变量 = …三个"，溯源到 runnable corpus（principles §六 Oracle 不冤枉的 corpus 端展开）。当前决策表 "C 锁定" 陈述与实现已发散。

---

### C2：`classify_external_fault` 在 detailed-design §七 完全缺席

**design 文本**（detailed-design.md:570-579 §七 错误处理）：

| 错误类型 | 处理 |
|---|---|
| 单 task 内 IO / cp / 渲染 / spawn 失败 | 任务标 UNKNOWN，整 run 继续 |
| 子进程退出非零 | 任务标 FAILED + exit_code |
| 子进程被信号杀（SIGSEGV 等） | 任务标 FAILED + exit_code = None |
| 子进程超时 | runner 发 `kill(-pgid, SIGKILL)`，任务标 FAILED + timed_out = true |

整张表无"FAILED → UNKNOWN 重分类"的描述。detailed-design.md:269 也只说 `Err → UNKNOWN`，未覆盖 `Ok(Failed) → 模式匹配 → UNKNOWN`。

**impl 代码**（runner/src/main.rs:200-223 + runner/src/report.rs:75-114）：

```rust
// main.rs:215
if let Some(tag) = report::classify_external_fault(&stderr_text, &stdout_text, r.exit_code) {
    external_fault = Some(tag);
    status_str = "UNKNOWN";
}
```

`report.rs::classify_external_fault` 实现 3 条模式匹配规则（`runnable_harness_arg_mismatch` / `vendor_lint_strictness` / `environment_corruption`），命中即将 FAILED 重分类为 UNKNOWN。

**不一致**：principles.md:111 §六 已落 UNKNOWN 严格语义（两类合法 UNKNOWN）；report.rs:62-67 doc comment 也明示"narrowed catalogue"。但 detailed-design §七 错误处理表未同步——读者按 detailed-design §七 理解会以为子进程非零一律 FAILED，与实现不符。

**反状态排除**：
- R1 ✗：detailed-design §七 表是穷举式描述错误处理路径，未列 = 缺
- R5 ✗：principles.md §六 是 2026-05-12 已落地的 v8 内容，不是历史段
- R6 ✗：检查 detailed-design 与 architecture 均未提及

**建议**：detailed-design §七 错误处理表新增一行"子进程退出非零且 stderr/stdout 命中 narrowed external-fault catalogue → 任务重分类为 UNKNOWN + error = `external_fault: <tag>`"，并把 catalogue 当前的 3 类（runnable_harness_arg_mismatch / vendor_lint_strictness / environment_corruption）以脚注或子表形式记录，溯源到 principles §六。

---

### C3：`TS_*` env 隔离 + `TS_ENTRY_FN` / `TS_TARGET_CRATE` 注入未文档化

**design 文本**：architecture.md / detailed-design.md 均未提及 `TS_*` 环境变量约定。仅 hax-lean-consistency-design-2026-05-11.md:345-349 在 wrapper 例子里使用了 `TS_ENTRY_FN` / `TS_TARGET_CRATE`，但那是单 design 文档的 sample，不是核心 design 层的契约说明。

**impl 代码**：
- runner/src/main.rs:66-70：设 `TS_PROJECT_ROOT = current_dir`
- runner/src/exec.rs:178-192：spawn 前 `env_remove` 所有 `TS_*`，再注入 `TS_ENTRY_FN` / `TS_TARGET_CRATE`
- runner/src/discover.rs:430-471：`expand_env` 实现 `${VAR}` 在 `command` / `version_command` 中的展开

**不一致**：这是已实现的 runtime contract——`tool.toml` 里 `${VAR}` 能展开 / wrapper 进程能读到 `TS_ENTRY_FN` / `TS_*` 不会泄漏到工具自身——但 detailed-design §一 `tool.toml` 字段表与 §四 单次执行单元步骤里**完全没提**。集成者读 detailed-design 无从知道这些可用。

**反状态排除**：
- R1 ✗：明确缺章节
- R6 ✗：detailed-design / architecture 两处均无；仅 hax-lean-consistency-design 例子用，那是 sample 不是 contract spec
- R11 ✗：是接口契约缺失，非措辞

**建议**：detailed-design §一 `tool.toml` 字段表加 "`${VAR}` 展开规则"小节；§四 单次执行单元在步骤 6 前新增"6_pre. spawn 前 env 处理：strip `TS_*`，注入 `TS_ENTRY_FN` / `TS_TARGET_CRATE`"。或在 architecture §五"接口：runner → external_subprocess"补 env 协议条目。

---

### C4：detailed-design §五 kani timeout 示例 (120) 与 §六 results.json 例子 (600) 自相矛盾

**design 文本**：
- detailed-design.md:363-367：
  ```toml
  # tools/kani/tool.toml
  command      = ["cargo", "kani", "--only-codegen", "--bin", "__ts_harness"]
  timeout_secs = 120
  ```
- detailed-design.md:498-507（同一份 detailed-design.md 的 results.json sample）：
  ```json
  {
    "name": "kani",
    "timeout_secs": 600,
    ...
  }
  ```

**impl 代码**：实际 tools/kani/tool.toml 由文件控制；此项是 design 文档**内部**自我不一致。

**不一致**：design 同文件内 kani 的 timeout 一处 120 一处 600，读者无从判断该参考哪个。

**反状态排除**：
- R1 ✗：是 design 自身文本内不一致
- R11 部分适用：是示例数值层面，但 detailed-design §五 在 kani 描述里特意强调"timeout 砍到 120s"是设计意图，§六 示例 600 反映另一意图——不是单纯笔误

**建议**：把 §六 results.json 示例的 `timeout_secs` 同步到 120（或调整 §五 描述）。次要决策点，文档自洽即可。

---

### C5：report.md 章节布局 design 严重欠描述（impl 是 superset）

**design 文本**（detailed-design.md:546-563）：

report.md 例子只含：feature 标题 + 工具 × entry 表 + 末尾 `Total: ... succeeded / ... failed / ... unknown / ... total`。

architecture.md:181 模块功能规约：

> run_dir/report.md 含按 feature 分组的工具 × entry 矩阵

**impl 代码**（runner/src/report.rs:167-326）：

实际 report.md 至少含 6 个章节：
1. `## Run metadata`（started/finished/duration/host/memory/cores）
2. `### Tool versions` 表
3. `## Per-tool summary`（n / S / F / U / TO / rate / avg / p50 / p90 / max）
4. `## Per-feature summary`（tasks / S / F / rate）
5. `## Per-feature × per-tool` 锚标
6. 每 feature 的 entry × tool 矩阵 + 末尾 Total

**不一致**：实现是 design 描述的严格超集。design §六 例子读者会以为 report.md 只是个矩阵 + 汇总；实际有 Run metadata / Tool versions / Per-tool summary / Per-feature summary 等聚合段。

**反状态排除**：
- R1 ✗：design 例子不是"省略"提示，文字也未注"省略其他章节"
- R2 ✗：design 没声明 report.md 可任意扩展
- R6 ✗：architecture §四 后置条件也只说"按 feature 分组的工具 × entry 矩阵"

**建议**：detailed-design §六 把 report.md 6 章节明示列出，或直接放完整最新样本片段。次要文档完善——不影响功能正确性。

---

### C6：`schema_kind` / `hirusttest_dir` 字段 design 设计有契约、impl 标 `#[allow(dead_code)]`

**design 文本**（detailed-design.md:284-288 单次执行单元步骤 3c）：

> 3c. 若 example.schema_kind == Directory（目录轨）：
>     // 渐进实施——v1 仅识别字段，stub 注入由后续版本支持
>     从 .hirusttest/config.toml 读 entry_overrides / tools 节
>     若有 per-entry verifier stub（<entry-fn>-spec.rs）则注入到 work_dir/<target_path>/src/
>     若有 per-tool harness 模板覆写（<tool-name>-harness.rs.tera）则覆盖步骤 5 的默认模板

**impl 代码**（runner/src/discover.rs:50-57 + runner/src/exec.rs 全文）：

```rust
#[allow(dead_code)]
pub schema_kind: SchemaKind,
#[allow(dead_code)]
pub hirusttest_dir: Option<PathBuf>,
```

`exec.rs` 完全未读取这两字段——3c 步骤无任何 impl 痕迹。

**不一致**：design 自己注 "v1 仅识别字段，stub 注入由后续版本支持" → 这是 design 已声明的 R6 缓冲（自标 deprecate / TODO）。但 `#[allow(dead_code)]` 是 impl 端的"用了 design 留的字段名但实际不消费"的反向状态：design 是 forward contract，impl 把 forward contract 标为 dead-code。

**反状态排除**：
- R2 部分适用：design §运行时机制 3c 自标"v1 仅识别字段，stub 注入由后续版本支持"——其实可视为 design 自带 contract 缓冲；但 contract 文本本身没说"discover 暴露字段但 exec 完全不读"
- R5 ✗：是当前 v1 行为不是历史段
- R6 ✗：design 与 impl 在"暴露但不用"这点上方向一致，但 contract 明示性弱

**建议**：detailed-design §三 `find_examples` 后置条件可补一句"v1 实现仅暴露 schema_kind / hirusttest_dir 字段供下游访问，exec 端 stub 注入由后续版本落地"。当前已基本对齐，仅明示性不足，决策点弱。

---

### C7：detailed-design §一 未声明 `entries = []` 在 discover 阶段报错

**design 文本**（detailed-design.md:21）：

```toml
entries = ["fn_name_1", "fn_name_2"]   # 必填，零参 pub fn 名列表
```

"必填"通常理解为"字段必须出现"，但 toml 中 `entries = []` 是合法的字段出现。§七 错误处理表也只说"Schema 解析失败" → panic，未明示"空 entries 是否算 schema 解析失败"。

**impl 代码**（runner/src/discover.rs:217-222）：

```rust
if ts.entries.is_empty() {
    return Err(anyhow!(
        "{}: `entries` must be non-empty (...)",
        config_path.display()
    ));
}
```

impl 主动拒绝空 entries。

**不一致**：design 文字"必填"不精确；impl 行为是"非空"。读者按 design 写 `entries = []` 会 hit hard fail，无 forward warning。

**反状态排除**：
- R11 部分适用：是文档措辞，但行为差异（拒绝 vs 接受）是实质而非措辞——空 list 会触发 hard error，对于配置者是行为问题

**建议**：detailed-design §一 把"必填"改"非空必填"，或在 §七 错误处理表加一行"hirusttest.toml `entries = []` → discover 阶段 Err"。低优先级决策点。

---

### C8：`extra_cargo_deps` discover 阶段 TOML 预校验未文档化

**design 文本**（detailed-design.md:128 + §七）：

> **extra_cargo_deps**：每条是一行 TOML 风格的 dependency 声明（如 `crate-name = "x.y.z"`）。runner 用 toml AST 把它 inject 到工作副本 `Cargo.toml` 的 `[dependencies]` 表（同名 key 覆盖）。

design 描述 inject 时机是 exec 阶段。§七 错误处理表 schema 解析失败由 discover 启动时 panic 兜底。

**impl 代码**（runner/src/discover.rs:507-528）：

```rust
for dep_line in &parsed.extra_cargo_deps {
    let parsed_dep: toml_edit::DocumentMut = dep_line.parse()...?;
    if parsed_dep.as_table().iter().next().is_none() {
        return Err(anyhow!(... "declares no dependency key" ...));
    }
}
```

discover 阶段就用 toml_edit parse 每条 dep_line + 校验非空 key。

**不一致**：impl 把校验从 exec 阶段提前到 discover 阶段（fail-fast 改进）。design 没说但也不矛盾——属补强。

**反状态排除**：
- R1 ✗：仅是"未文档化"，行为方向与 design §七 兜底原则（schema bug 启动期 panic）一致
- R2 ✓ 部分：detailed-design §七 已声明 schema 错误 discover 阶段 panic，可解释为 deprecated"未明示"

**建议**：detailed-design §一 `extra_cargo_deps` 字段描述补"在 discover 阶段做 toml_edit AST 预校验，恶劣形态早 fail"。低优先级。

---

### C9：principles.md §六 UNKNOWN 严格语义 与 detailed-design / architecture 未传导

**design 文本**：
- principles.md:108-111 §六：明示 UNKNOWN 严格语义两类合法、4 种"工具能力边界"FAILED 不入 UNKNOWN
- architecture.md:281 决策表：`UNKNOWN 仅记 runner-internal 错误`
- detailed-design.md:217 / §七：UNKNOWN = runner-internal IO/spawn 失败

**impl 代码**：runner/src/report.rs:62-114 已按 principles §六 narrowed catalogue 实现。

**不一致**：架构层（architecture / detailed-design）仍停留在"runner-internal 错误 = UNKNOWN"的旧表述，未传导 principles §六"narrowed catalogue + tool 边界一律 FAILED"的严格语义。architecture §一末段（line 64）也是旧描述。

**反状态排除**：
- R1 ✗：差距具体可指证
- R5 ✗：principles §六 是 2026-05-12 现行表述
- R6 ✗：检查 architecture / detailed-design 两处均未传导
- R11 ✗：是宪法→架构→细化的传导失败，非文案

**建议**：architecture §一第 64 行 UNKNOWN 段、§七 决策表 line 281、detailed-design §七 错误处理表全部按 principles §六 同步更新——明示"runner-internal 错误"展开成"narrowed catalogue (a) 全局工具链崩溃 (b) 我方可识别问题"。

这是宪法→下游传导漏 1 跳的明显证据。

---

### C10：`error` 字段的 schema 在 UNKNOWN 子类型上 design 未区分

**design 文本**（detailed-design.md:529-542）：

```json
{
  "entry_id": "<...>/<...>",
  "tool": "<some-tool>",
  "status": "UNKNOWN",
  ...
  "error": "patching ...: parsing Cargo.toml as TOML: ..."
}
```

`error` 字段仅 UNKNOWN 任务存在，且 design 例子只展示 runner-internal IO 错误形态（`patching ...`）。

**impl 代码**（runner/src/main.rs:256 + main.rs:277）：

UNKNOWN 子类型现有两种：
- runner-internal Err：`error: Some(err_str)` — `format!("{:#}", e)`
- FAILED→UNKNOWN 重分类：`error: Some(format!("external_fault: {}", s))`

**不一致**：design schema 示例与文字未区分这两种 UNKNOWN 子类型；下游消费者从 schema 拿不到"error 字段以 `external_fault:` 前缀表明 P27 重分类"的契约。

**反状态排除**：
- R1 ✗：error 字段 schema 描述缺一类
- R2 ✗：design schema 部分没声明 error 是自由文本可任意前缀
- R6 ✗：architecture / detailed-design 均未提

**建议**：detailed-design §六 results.json schema 注明 `error` 字段的两种合法前缀（`external_fault: <tag>` / runner-internal 错误字符串）。也是 C2 同源——P27 改动的传导缺口。

---

## 3. 总结

10 个候选中：

**强候选**（design 与 impl 实质冲突，宜立即同步）：
- **C1**：Tera 变量数量与 architecture §七 "锁定两个" 决策正面冲突，3 个事实
- **C2** / **C9** / **C10**：P27 改动（narrowed external-fault catalogue + FAILED→UNKNOWN 重分类 + error 字段前缀语义）principles.md §六 已落地，但 architecture / detailed-design 未传导。三条同源
- **C3**：`TS_*` env 隔离 / `${VAR}` 展开机制完全未在 architecture / detailed-design 文档化，但实际是 wrapper 工具必依赖的 contract

**中候选**（明示性 / 完整性补强）：
- **C5**：report.md 实现是 design 描述的严格超集，design 例子只示矩阵
- **C7**：`entries = []` 行为差异（design 写"必填"实际拒绝空）

**弱候选**（文档完善 / 自洽）：
- **C4**：detailed-design §五 / §六 kani timeout 自相矛盾 120 vs 600
- **C6**：`schema_kind` / `hirusttest_dir` 字段 design 有 forward contract，impl 标 dead-code（design 已自带 v1 缓冲，弱差距）
- **C8**：discover 阶段 `extra_cargo_deps` 预校验是 fail-fast 补强，未在 §一 字段描述里提

**P27 改动整体评估**：principles.md §六 已明确 UNKNOWN 严格语义 + 4 种工具边界 FAILED；report.rs 已删 3 条规则保留 3 条；但 **architecture / detailed-design 两份层级文档未同步**——这是宪法→架构→细化的 3 跳传导漏 2 跳的典型证据。C2 / C9 / C10 是同根的三个面。

宪法→实现的传导路径上：principles.md ✅（已落）→ architecture ✗（旧表述）→ detailed-design ✗（旧表述）→ impl ✅（已落）。建议优先修 architecture §一末段 + §七决策表与 detailed-design §七错误处理表三处，使传导链完整。
