# 架构审查 — `docs/design/architecture.md`

> 综合审计 `audit-2026-05-11` / 第 1 组（宪法 + 设计原则文档层）
> 审查日期：2026-05-11
> 审查范围：`docs/design/architecture.md`

---

## §1 问题意识

### 这份文档要审查什么？

`docs/design/architecture.md`（299 行）是 design 层的"lib 入口"——按其自身 line 5 定位「按 /workflow §2.2 规范书写架构设计；索引下游模块文档」。它的作用是把 principles.md 三大派生原则落到模块切分、接口规约、并发规约、关键设计决策记录。

### 为什么要审查？

1. **架构层是宪法与代码之间的桥梁**：宪法层抽象、代码层具体，架构层必须既能从宪法**派生顺**（每条架构决策能溯源到宪法某条），又能让代码**实施**（接口/模块切分清晰）。如果架构层与宪法派生不顺，下游 detailed-design + runner 实现都被污染。
2. **P11-P16 的实施变化**（引入 wrapper.sh、`${TS_*}` 替换、ror-typecheck 第 20 工具、runnable corpus、TS_ENTRY_FN 注入子进程）—— 架构层是否吸纳？
3. **架构层是否引入了 principles.md 没有的新假设**？这是潜在违反 CLAUDE.md "下游不破上游"的红线。

### 用什么方法 / 角度审查？

- **派生顺序追查**：每条架构决策（line 268-289 决策表）→ 找其在 principles.md 中的依据
- **模块切分覆盖完整性**：runner/examples/tools/reports 四类是否清晰、不重叠、不漏
- **接口规约准确性**：discover → exec → report 接口是否与 runner/src/*.rs 实际行为一致
- **新假设识别**：grep 架构中"宪法 / principles"出现的位置，反向检查"无引用"的段是否引入了新假设

---

## §2 审查方法

### 参照源

- `docs/design/architecture.md` 全文
- `docs/design/principles.md`（上游对照）
- `docs/design/detailed-design.md`（下游对照）
- `runner/src/{discover,exec,report,main,host}.rs`（实施对照）
- `tools/*/tool.toml`、`tools/*/wrapper*.sh`（外部 artifact 对照）

### 恶意角度具体实施

- 假设架构有"按宪法不该写但写了"的内容（典型：引入了宪法没说的字段、暗藏方法学假设）
- 假设模块切分有重叠 / 缝隙（discover vs exec、report vs main 边界）
- 假设接口规约"前后置条件"漏了实施已做的事（典型：env 注入、stub 注入路径）
- 假设决策表（line 268-289）有 dangling refs（某条引自不存在的原则）

### 诚实底线

每条问题给出**文件路径 + 行号 + 引文 + 推理链**。

---

## §3 审查现象

### #1（严重度: 高）—— 模块切分图覆盖不全，缺 wrapper.sh / 辅助文件层

**现象**：

`architecture.md` line 99-110 模块切分：

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

**违反 / 嫌疑**：

- `tools/<name>/` 实际除了 tool.toml + harness.rs.tera 外，还可能含 `<tool>-wrapper.sh`（9 个工具用 wrapper）—— 架构图没列
- runner/src 实际还有 `host.rs` 模块（采集 host info / capture_version）—— 架构图没列
- examples 的目录轨（按 architecture.md 自己 line 30-50 描述）会有 `.hirusttest/<entry>-spec.rs` 等辅助文件—— 架构图没列

**推理链**：

- architecture.md 自身 line 32-50 写了双轨 example schema 含目录轨的辅助文件
- 但 line 99-110 模块切分图仅说"Cargo lib crate + hirusttest.toml — 由用户写"——单文件轨形态
- 类似地 line 99 说"tools/<name>/ {tool.toml + harness.rs.tera}"——没 wrapper.sh
- 架构图层的不全覆盖会让读者误以为"wrapper.sh 不在架构内"——但 wrapper.sh 是工具集成的常态产物，应在架构层声明

**决策性**：☐ 非决策点（架构图补全）

**建议**：line 99-110 改为：

```
runner/                          Rust binary
├── discover                     双轨 example schema 发现 + tool 发现
├── exec                         单次 (tool, example, entry) 执行
├── report                       results.json + report.md 生成
├── host                         host info / version_command 捕获
└── main                         CLI、调度、并发池、汇总打印

examples/<feature>/<dir>/        单文件轨：Cargo lib crate + hirusttest.toml
                                 目录轨：Cargo lib crate + .hirusttest/{config.toml, ...}
tools/<name>/                    tool.toml + harness.rs.tera + 可选 <name>-wrapper.sh
runs/run-<ts>-<pid>/             每次 run 输出根
```

---

### #2（严重度: 高）—— 决策记录表（line 268-289）未涵盖 P11-P16 引入物

**现象**：

`architecture.md` line 268-289 「关键设计决策记录」表共 19 行决策。grep "wrapper.sh"、"\${VAR}"、"TS_ENTRY_FN"、"runnable" 在该表中均 **0 hits**。

**违反 / 嫌疑**：

- 架构层 line 264「每条决策溯源到一条原则」——所以这张表是溯源中心
- 但 P11-P16 引入了多条"运行时机制"，都未进表：
  1. `${TS_*}` 环境变量替换层（discover.rs `expand_env()`）—— 派生自原则 C 异质性归配置 + 反对在代码层硬编码工具路径
  2. `TS_PROJECT_ROOT` / `TS_ENTRY_FN` / `TS_TARGET_CRATE` 三个内部环境变量注入子进程（main.rs:67 / exec.rs:178）—— 派生自工具集成需要 wrapper / spec 文件路径
  3. 子进程启动前 env_remove 所有 `TS_*`（exec.rs:165-171）—— 防止 runner-internal 变量误被工具识别（如 prusti 把任意 PRUSTI_* 视为配置）
  4. wrapper.sh 模式作为"集成者描述工具自身行为"的延伸（principles.md line 188 提到但 architecture 决策表没记）
- 这些决策缺记录会让未来读者无法判断"它们的派生依据"

**推理链**：

- 架构层的决策表是"宪法 → 架构"的桥梁
- 每条新引入的运行时机制如果不进表，未来无法溯源
- 当前 working tree 实施已包含上述机制（runner/src/discover.rs / exec.rs / main.rs 实证），但架构层没说

**决策性**：☑ **决策点** —— 需要用户审议：

- 这些 P11-P16 引入物是否要进入架构层决策表？（推荐：是）
- 如要，其上游派生依据是什么（原则 C / 工具集成的副作用控制）需用户确认

**建议**：决策表加 4 行：

| 决策 | 推自 | 状态 |
|---|---|---|
| tool.toml `${VAR}` 环境变量替换（discover.rs::expand_env） | C + 反硬编码（避免 tool.toml 写绝对路径） | 锁定 |
| TS_PROJECT_ROOT / TS_ENTRY_FN / TS_TARGET_CRATE 注入子进程 env | wrapper.sh 与 oracle 需要这些上下文做 entry-fn 自筛 / 路径定位 | 锁定 |
| 子进程 spawn 前 env_remove 所有 TS_* （exec.rs:165-171） | 防止 runner-internal 配置变量被工具误识别（如 prusti） | 锁定 |
| tools/<name>/<name>-wrapper.sh 作为 oracle 强化层 | "集成者描述工具自身行为" 的延伸（principles A-4） | 锁定 |

---

### #3（严重度: 高）—— 接口规约"exec → external_subprocess"未声明 env 注入契约

**现象**：

`architecture.md` line 225-235 接口规约：

```
接口：runner → external_subprocess

输入数据：tool.command argv、副本目录 cwd、stdio piped（stdout/stderr 到 runner 的 reader threads）
输出数据：subprocess 的 exit_code、stdout 字节流、stderr 字节流、wall-time
约束：subprocess 起在独立 process group（process_group(0)）
```

**违反 / 嫌疑**：

- 实际 `runner/src/exec.rs` line 165-179：spawn 前会做两件事：
  1. env_remove 所有以 `TS_` 开头的环境变量
  2. env_set `TS_ENTRY_FN`、`TS_TARGET_CRATE` 两个内部变量
- 这是 wrapper.sh / oracle 实际依赖的契约（如 `rocq-of-rust-wrapper.sh` line 72-76 显式说「`TS_ENTRY_FN ... — name of entry fn under test`」）
- 接口规约完全没说，读者无法从架构层知道 wrapper.sh 能拿到什么环境变量

**推理链**：

- 这是"runner 提供 / wrapper 消费"的接口契约
- 接口规约是 architecture.md §五 的核心产物，必须完备
- 当前规约缺漏会让未来 wrapper 作者不知道有什么变量可用

**决策性**：☑ **决策点** —— 需要用户审议：

- 这两个内部变量（TS_ENTRY_FN、TS_TARGET_CRATE）是否作为正式接口承诺？（一旦写入架构，将来不能轻易删）
- 是否还要加 `TS_PROJECT_ROOT`（main.rs:67 注入，但 exec.rs:165-171 不 strip——其实 TS_PROJECT_ROOT 也是 TS_ 前缀，会被 strip 掉。需复查实施细节）

实测：

```
let ts_vars: Vec<String> = std::env::vars()
    .map(|(k, _)| k)
    .filter(|k| k.starts_with("TS_"))
    .collect();
for k in &ts_vars {
    command_builder.env_remove(k);
}
// 然后再 set TS_ENTRY_FN / TS_TARGET_CRATE
```

即 `TS_PROJECT_ROOT` 也被 strip——但子进程的 `${TS_PROJECT_ROOT}` 在 tool.toml 已经被 discover.rs `expand_env()` 提前替换为字面值（in argv），所以子进程不需要这个 env。但是 wrapper.sh 内部可能要访问 `TS_PROJECT_ROOT`——按 tool.toml 内的 `command = ["${TS_PROJECT_ROOT}/tools/prusti/prusti-strict-wrapper.sh"]` 行为，argv[0] 已是字面路径，wrapper.sh 启动后是否还需要 `TS_PROJECT_ROOT` 由 wrapper.sh 自身决定。这条契约**架构层须明示**。

**建议**：接口规约补充：

```
环境变量契约：
  - runner spawn child 前会 env_remove 所有 TS_* 变量
  - 然后 env_set TS_ENTRY_FN = <entry fn name>、TS_TARGET_CRATE = <crate ident>
  - 其他变量（如 PATH / HOME / shell 等）从 runner 进程继承
  - tool.toml argv 中的 ${VAR} 在 discover 阶段已被 runner expand_env 替换为字面值；
    子进程拿到的 argv 是替换后的版本，但子进程的 env 不含 TS_*
  - wrapper.sh / oracle 脚本依赖 TS_ENTRY_FN / TS_TARGET_CRATE 做 entry-fn 自筛、
    产物校验
```

---

### #4（严重度: 中）—— "通用性论证"列举工具数与实际不符 + 反例域过严

**现象**：

`architecture.md` line 84：「实测覆盖：cargo-check / Kani / MIRI / Charon (poly + mono) / Prusti / Creusot / Verus / VeriFast / Soteria / kmir / hax (lean/fstar/coq) / aeneas (lean/coq/fstar/hol4) / rocq-of-rust 全在内（19 工具）。」

`architecture.md` line 90：「要求 `Cargo.toml` 的 `[features]` / `[patch]` / profile 配置而非仅 `[dependencies]` 行（当前 schema 不覆盖；未来可同范式扩展，例如加 `extra_cargo_features` 字段）」

**违反 / 嫌疑**：

- 19 vs 20：与 principles 同一漂移（见 principles-review.md #1）
- "未来可同范式扩展 extra_cargo_features 字段"——这是 principles.md 不存在的新假设。principles.md 没说"可扩展 schema 字段"；按 CLAUDE.md feedback_no_premature_extensibility 项目长期记忆，"不做超出当下需求的可扩展性设计"是用户偏好——架构层提"未来可扩展" mild 违反这一项目精神

**推理链**：

- 数字漂移：与 principles 同源，等待用户决策（议案 #2）
- "未来可扩展"措辞：项目内存反复强调"通过配置而非代码实现工具兼容；禁止预留代码层抽象"——line 90 的"未来可加 extra_cargo_features"是预留代码层抽象
- 但 line 90 是**论证而非承诺**——说"反例域不覆盖某类工具"再说"未来可同范式扩展"是合理的，没破"不预设可扩展性"——这是边缘案例

**决策性**：☐ 非决策点（措辞小修）

**建议**：line 90 改为「要求 `Cargo.toml` 的 `[features]` / `[patch]` / profile 配置而非仅 `[dependencies]` 行（当前 schema 不覆盖；如有这类工具需求出现，可同范式扩展）」——去掉"未来可"的预承诺。

---

### #5（严重度: 中）—— 模块功能规约 discover 未描述 `.no-hirusttest` 屏蔽机制

**现象**：

`architecture.md` line 122-149 模块 discover 规约：

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
```

**违反 / 嫌疑**：

- 实际 `runner/src/discover.rs` line 71 + line 104-119 实现了 `.no-hirusttest` 屏蔽机制：「if e.path().join(SKIP_MARKER).exists() { return false; }」——一旦目录下有 `.no-hirusttest` 文件，跳过该目录和整个子树
- 架构层规约完全没说这一机制——但 detailed-design.md line 163 提到「`递归扫描 + .no-hirusttest 屏蔽`」
- 架构层应描述这一关键发现行为

**推理链**：

- `.no-hirusttest` 是发现阶段的关键控制点——尤其用于 vendor/ 子项目（含工作区内嵌不该被当成 example 的目录）
- 架构层 discover 规约的"前置/后置条件"必须周延，应明示这一屏蔽

**决策性**：☐ 非决策点（架构小补）

**建议**：line 127 后加：

```
屏蔽机制：
  - 若 <example_dir> 含 .no-hirusttest 文件，则该 dir 及其整个子树被跳过
    （用于 vendor/ 子项目含工作区内嵌不该被当作 example 的目录）
```

---

### #6（严重度: 中）—— 双轨 schema 引入"渐进实施"暗示 vs 宪法的"决策点"性质

**现象**：

`architecture.md` line 42-50 描述目录轨：

```
目录轨（外部综合项目，仅适用 vendor/ 下的真实 codebase）：
  vendor/<crate>/   或   examples/<feature>/<dir>/
  ├── Cargo.toml                  （原项目自带）
  ├── src/                        （原项目自带）
  └── .hirusttest/
      ├── config.toml             ← 主配置（必有）
      └── <可选辅助文件>          ← per-entry verifier stub / proof annotation snippet 等
```

`architecture.md` line 56-58：「外部综合项目（如 vendor/x509-parser、vendor/openssl）...」

但 detailed-design.md line 282-288 描述：

```
3c. 若 example.schema_kind == Directory（目录轨）：
        // 渐进实施——v1 仅识别字段，stub 注入由后续版本支持
        从 .hirusttest/config.toml 读 entry_overrides / tools 节
        若有 per-entry verifier stub（<entry-fn>-spec.rs）则注入到 work_dir/<target_path>/src/
```

**违反 / 嫌疑**：

- 架构层 line 42-50 描述目录轨完整能力（含辅助文件支持）
- 细化层说"v1 仅识别字段，stub 注入由后续版本支持"——即架构层定义了能力但细化层只实施了部分
- 现状 runner/src/discover.rs 仅识别 schema_kind 与 hirusttest_dir 但**不做 stub 注入**——detailed-design 与 architecture 之间有"承诺 vs 实施"的鸿沟

**推理链**：

- 这违反 CLAUDE.md "下游不破上游"：architecture 描述了 stub 注入能力但 detailed-design 与 runner 都没实施
- 严格按 CLAUDE.md "改下游不改上游"原则，应是 architecture 不写未实施能力——直到 detailed-design 准备实施时才写
- 当前状态：架构层"超前预设"了一个未实施能力——这违反项目反复强调的 feedback_no_premature_extensibility

**决策性**：☐ 非决策点（措辞下调）

**建议**：line 47 后加：「`<可选辅助文件>` 的注入到副本 src/ 的能力由 runner exec 阶段实施；**当前状态：v1 仅识别 schema_kind 与辅助文件目录路径，stub 注入由后续版本实施**（详见 detailed-design §四 单次执行单元 3c）」——明示当前实施状态，避免"架构层超前承诺"。

---

### #7（严重度: 中）—— "原则 A 投影"vs"原则 A 位阶澄清"在架构层重复 + 派生不顺

**现象**：

- `architecture.md` line 11-15 §一设计推导：「1. **原则 A 投影（运行原子 = 单 entry）**...」
- `architecture.md` line 19-29 §一-"A 位阶澄清的具体落地"：再次描述 A 位阶澄清下的两个声明式机制
- `architecture.md` line 270-279 决策表：「样例为独立 cargo lib crate，零工具依赖 | A | 锁定」「渲染目标默认 src/bin/__ts_harness.rs ... `entry_mode = "lib"` ... | A 位阶澄清 + cargo auto-discover | 锁定」

**违反 / 嫌疑**：

- "原则 A"与"A 位阶澄清"在架构层多处反复——这本身不是问题
- 问题在于：principles.md §四 line 142 已经做了 A 位阶澄清，architecture.md 又重做一遍 line 19-29 ——存在文字重复
- 风险：如果未来宪法 line 142 改而架构 line 19-29 没改，会产生"宪法 vs 架构"暗中漂移

**推理链**：

- 架构层应"引用宪法 + 落地具体"而非"复述宪法 + 落地具体"
- 当前 line 19-29 复述了宪法的位阶澄清，不是引用——这是架构与宪法的耦合点

**决策性**：☐ 非决策点（措辞小修）

**建议**：line 19 改为「按 `principles.md` §四 line 142-144 的'A 位阶澄清'，A 的承诺降级为'原始磁盘字面零修改 + 隔离副本上声明式工具特定填充'。具体落到两个声明式机制：...」——明示引用关系而非复述。

---

### #8（严重度: 中）—— "原则 B 在结果分类上的扩展"段引入 UNKNOWN 的派生不在 principles

**现象**：

`architecture.md` line 62-64：

```
### 原则 B 在结果分类上的扩展

SUCCESS / FAILED 的二分初看够用：exit 0 = SUCCESS，非 0 = FAILED。但 runner 自己 IO 失败、cp 失败、模板渲染失败的时候若都标 FAILED，下游会误读为"该工具不能处理该 entry"——这是 runner 自己的 bug，不是工具的表态。所以分类扩出 **UNKNOWN**：runner-internal 错误归 UNKNOWN，subprocess 跑完且非零归 FAILED，subprocess exit 0 归 SUCCESS。三类处于不同语义维度（SUCCESS / FAILED 描述子进程结果，UNKNOWN 描述 runner 状态）——不破 B 在工具评判层的简洁。
```

`principles.md` 全文 grep "UNKNOWN" → **0 hits**。

**违反 / 嫌疑**：

- 架构层引入了三分（SUCCESS / FAILED / UNKNOWN），但宪法层从未提及 UNKNOWN
- 架构层 line 64 自己论证"不破 B"是合理的——UNKNOWN 描述 runner 状态而非工具评判——但这是**架构层自己引入的派生**，未先在宪法层论证过
- 严格按 CLAUDE.md "宪法是绝对底线"，**所有非细节决策应能溯源到宪法某条**。三分类是一个有意义的决策，应在宪法 §四 派生原则 B 节中预先说明

**推理链**：

- principles.md §四 原则 B（line 195-197）："本项目是**筛选**不是**认证**——只问'工具能不能吃下这段代码并产出预期形状的输出'，不问'产出语义是否正确'。分类标签只描述产出形状，不带对错评分。"
- 宪法没说"分类是几类"——只说"分类不带对错评分"
- 架构层 line 64 推导出三分类，论证合理，但属架构层创新——不是从宪法直接派生

**决策性**：☑ **决策点** —— 需要用户审议：

- 三分类（SUCCESS / FAILED / UNKNOWN）是否要写进宪法？（推荐：写，因为这是 results.json 的**对外契约**——读者拿数据时遇到的第一件事就是"为什么是三分类"）
- 还是保留在架构层即可（架构层够清楚）？

**建议**：在宪法 §四 原则 B 后加一段「**B 的运行时投影：分类三分**——为区分'runner 自身错误'（不归工具）与'工具结果'，结果分类三分：SUCCESS（subprocess exit 0）/ FAILED（subprocess 跑完且非零）/ UNKNOWN（runner-internal 错误）。UNKNOWN 描述 runner 状态而非工具评判，不破 B 简洁性。」

---

### #9（严重度: 低）—— 通用性论证 line 84 工具枚举未列 rocq-of-rust-typecheck

**现象**：

`architecture.md` line 84：「**实测覆盖**：cargo-check / Kani / MIRI / Charon (poly + mono) / Prusti / Creusot / Verus / VeriFast / Soteria / kmir / hax (lean/fstar/coq) / aeneas (lean/coq/fstar/hol4) / rocq-of-rust 全在内（19 工具）。」

**违反 / 嫌疑**：

- 与 principles 同源漂移
- 应改"19"为"20"或动态枚举，并补 rocq-of-rust-typecheck

**决策性**：☐ 非决策点（与 principles-review #1 / #X2 同步处理）

---

### #10（严重度: 低）—— "通用性论证"提到 MIRAI / ESBMC-Rust 未实测但说"在内"

**现象**：

`architecture.md` line 84：「MIRAI / ESBMC-Rust 在 token 形态分析上落入此集，未实测。」

**违反 / 嫌疑**：

- "落入此集，未实测"是正确表达——但这是"理论覆盖"声明
- 项目宪法 §三-3-原则 1（line 76-83）"承认时效性"+ "锚定具体版本" 反复强调"实测才有意义"——架构层做的"理论覆盖论证"是合理的（论证范式有效性，不是"实测断言"），但措辞有歧义

**推理链**：

- 这是论证"通用性"的合理做法——必须有"理论覆盖"陈述
- 但读者可能误读"在内"为"已集成"

**决策性**：☐ 非决策点（措辞小修）

**建议**：line 84 改「MIRAI / ESBMC-Rust 在 token 形态分析上理论可覆盖，未实测——本项目未集成」——明示"理论"vs"已集成"区分。

---

### #11（严重度: 低）—— 并发规约"线程安全性结论"过于绝对

**现象**：

`architecture.md` line 256-260：

```
线程安全性结论：
  - 每个 (tool, example, entry) 任务彼此独立，rayon par_iter 安全
  - 默认并发度 = num_cpus；--parallel N 可覆盖
```

**违反 / 嫌疑**：

- "每个任务彼此独立"——其实不完全。cargo registry / cargo target cache 是跨任务共享（line 247-248 说"cargo 自身用 flock 保证并发安全，runner 不参与"）
- 严格说，任务的**文件 / spawn / raw output 层**互不影响，但任务的**cargo 行为 / 网络 / 磁盘 IO 层**有共享资源（虽由 cargo flock 保证）
- 架构层简单结论"完全独立"有过度简化嫌疑

**推理链**：

- principles.md §七 line 282 提到"`Cargo.lock` 副本带来的 cargo re-resolve 延迟、cargo registry flock 在并发首次 fetch 时序列化"——这是宪法层明示的并发性能限制
- 架构层 line 256 结论"完全独立"与宪法层 line 282 不冲突（因为宪法说的是"性能不算问题"），但读者可能误以为"绝对独立"

**决策性**：☐ 非决策点（措辞小修）

**建议**：line 257 改为「每个 (tool, example, entry) 任务在 raw output / work_dir / spawn 层彼此独立；cargo registry / target cache 共享由 cargo 自身 flock 处理（性能层面可见串行化但不影响正确性，按 principles.md §七 性能不算问题）」

---

## §4 决策点 vs 非决策点汇总

### 决策点（需用户审 / 拍板）

| # | 摘要 |
|---|---|
| #2 | 决策表加 4 行 P11-P16 引入物（wrapper.sh / `${VAR}` / TS_* / env strip） |
| #3 | 接口规约 runner → external_subprocess 加 env 注入契约（TS_ENTRY_FN / TS_TARGET_CRATE 是否作为正式接口承诺） |
| #8 | 三分类（SUCCESS/FAILED/UNKNOWN）是否提升到宪法层 |

### 非决策点（局部 fix）

| # | 摘要 |
|---|---|
| #1 | 模块切分图补 host.rs / wrapper.sh / 目录轨辅助文件 |
| #4 | "未来可扩展 extra_cargo_features"措辞下调 |
| #5 | discover 规约补 .no-hirusttest 屏蔽 |
| #6 | 目录轨"渐进实施"明示当前实施状态 |
| #7 | A 位阶澄清复述改引用 |
| #9 | 19 工具枚举与 principles 同步 |
| #10 | MIRAI / ESBMC-Rust "在内"措辞改"理论可覆盖" |
| #11 | 并发"完全独立"措辞精细化 |

---

## §5 审查结论

### 总体判断

`architecture.md` 主体派生顺畅、与 principles 大体对齐——三大原则的运行时投影、双轨 schema、模块切分、接口规约都能从宪法溯源。但存在三类问题：

1. **实施漂移**（高严重度）：决策表（line 268-289）未涵盖 P11-P16 引入的 wrapper.sh / `${TS_*}` 替换 / TS_* 注入 / env strip 四类机制；接口规约未声明 env 契约——这与 principles-review §3 #2 同源，是 design 主线对 P11-P16 实施的整体缺失。
2. **新假设引入**（中严重度）：原则 B 三分类（UNKNOWN）由架构层自己推出，宪法层从未承诺；目录轨"辅助文件可工具读"在架构层描述完整但实施层"v1 仅识别"——架构层超前预设。
3. **小漂移**（低严重度）：19 工具枚举、并发结论过度绝对、模块切分图缺 host.rs / wrapper.sh、复述宪法 vs 引用宪法的措辞偏好。

### 严重度分布

- 高严重度：3 条（#1 + #2 + #3）
- 中严重度：5 条（#4 + #5 + #6 + #7 + #8）
- 低严重度：3 条（#9 + #10 + #11）

### 关键风险

最严重风险是 **#2 + #3**：架构层是宪法 → 实施的桥梁，但当前桥梁对 P11-P16 实施的覆盖明显不完整。如果新读者按"宪法 → 架构 → 细化 → 实现"链阅读，会在架构层完全错过 wrapper.sh 机制、`${TS_*}` 机制、TS_ENTRY_FN 内部环境变量——这些都是新工具集成时必须知道的契约。

### 与宪法的派生关系

整体派生顺畅。**没有发现架构层违反宪法的硬性条款**——但有两处"架构层独立创新未先在宪法预设"：
- 三分类（UNKNOWN 引入）—— 见 #8
- 目录轨辅助文件可被工具读取 —— 见 principles-review.md #5（与本文 #6 互相参照）

这两条决策合理但应回到宪法层确认。
