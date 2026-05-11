# 综合审计 audit-2026-05-11 — 第 5 组：runner 实现层审查

> 范围：`runner/src/{main,discover,exec,host,report}.rs` + `runner/Cargo.toml`
>
> 方法学：HoarePrompt Audit（按函数 / 代码块给前置 / 后置 / 不变式条件，逐一审查代码实现是否满足）+ `/principle-derivation-v2` 问题意识展开。参照源为 `docs/design/{principles,architecture,detailed-design,tool-integration}.md`，实施源为 `runner/src/*.rs`。审查不修改任何代码，仅记录现象。

---

## §1 问题意识

runner 是项目宪法（`principles.md`）认证的核心模块 1——其行为正确性直接决定整个 testsuite 数据的可信度。本次审查从三层意识展开：

1. **设计与实现一致性**：`architecture.md` §四 / `detailed-design.md` §三 给出 discover / exec / report 的函数级前后置规约，runner 实现是否完整兑现这些规约？尤其是关于「不变式」的承诺（同 run 内 exec_id 唯一、子进程独立 process group、隔离副本完整、原始磁盘字面零修改等）是否真的成立？
2. **错误传播链路完整性**：runner 是 raw 数据「真实性诚实责任」的最终担保人——子进程的 exit code / stdout / stderr / 超时 / 信号杀必须完整、不失真地落到 `results.json`。任何一处吞错都会让下游报告对工具能力做出错误归因。
3. **边界条件与恶意输入抗性**：runner 接受用户配置的 `hirusttest.toml` / `tool.toml` / `harness.rs.tera` 三种文件输入；命令行参数支持 glob 过滤。审查需问：当输入畸形 / 缺字段 / 含特殊字符 / 路径 race 时，runner 是优雅 FAILED / UNKNOWN，还是 panic / 静默吞错 / 误标信号？

第三层意识是恶意角度的核心——本次审查刻意找代码层 bug、设计与实现不一致、边界条件漏洞、信号传播错误。

---

## §2 审查方法

- **参照源**：`docs/design/principles.md`、`docs/design/architecture.md` §四 模块功能规约 + §五 模块间接口规约 + §六 并发规约、`docs/design/detailed-design.md` §三 Runner 函数级规约 + §四 运行时机制 + §七 错误处理策略、`docs/design/tool-integration.md`。
- **实施源**：`runner/src/main.rs`（291 行）、`runner/src/discover.rs`（397 行）、`runner/src/exec.rs`（347 行）、`runner/src/host.rs`（127 行）、`runner/src/report.rs`（277 行）、`runner/Cargo.toml`（19 行）。
- **审查粒度**：对外可见函数逐一给 precondition / postcondition / invariant，比对实现；内部 helper 抽查；跨文件「信号传播链」逐段验证。
- **恶意角度清单**：故意找代码层 bug、设计与实现的偏差、边界条件漏洞（空输入 / 长输入 / 特殊字符）、panic-on-bad-input、并发 race、错误吞噬、不变式破坏。

---

## §3 审查现象

按文件 + 严重度从高到低排列。

### main.rs

---

#### #1（中）— `examples` 与 `tools` CLI 参数在 `report` 子命令下仍被 canonicalize，规约不一致

**位置**：`runner/src/main.rs:72-79`

**前置条件**：`args` 已 parse；若 `cmd == Some(Cmd::Report{..})` 则 line 60 已早 return。

**后置条件**：进入主跑路径时 examples_dir / tools_dir 已 canonicalize。

**实际行为**：line 56-79：
```
let args = Args::parse();
if let Some(Cmd::Report { run_dir }) = args.cmd {
    return regenerate_report(&run_dir);
}
...
let examples_dir = args.examples.canonicalize() ...
let tools_dir = args.tools.canonicalize() ...
```
报告子命令路径已早 return，主跑路径才 canonicalize——逻辑正确。但 `Args` 的 `global = true` 标识使得 `--examples` / `--tools` 在 `report` 子命令下**也能被指定**，且会被默认值 `examples` / `tools` 覆盖；用户在调用 `runner report <run-dir>` 时若该路径不存在（例如外部消费者只想从 results.json 重生成），不影响——因为 report 路径不 canonicalize 它们。这是无 bug 但有用户体验隐忧的设计——`--examples` / `--tools` 作 `global` 但对 `report` 子命令完全无意义。

**违反**：未违反 design；属于 CLI 设计取舍。

**推理链**：clap 配置上 `global = true` 包括 subcommand 子作用域；report 路径下这两个标志被 silently ignore，文档没说明，但功能正确。

**决策性**：非决策点（CLI 美学）。

**建议**：可在 `Args` 中把 `examples` / `tools` 移出 `global`，让 `runner report` 不暴露这两个标志；不影响功能。

---

#### #2（高）— `regenerate_report` 不校验 `results.json` 中 status 值在合法枚举内

**位置**：`runner/src/main.rs:278-290`、`runner/src/report.rs:36-57`

**前置条件**：`run_dir/results.json` 存在；其结构遵循 `ResultsFile` schema。

**后置条件**：成功反序列化后调用 `report::write_report_md` 生成 `report.md`。

**实际行为**：line 285-286：
```
let parsed: report::ResultsFile =
    serde_json::from_str(&text).with_context(|| format!("parsing {}", json_path.display()))?;
```
`TaskResult.status` 是 `String`——serde 不会校验它必须 ∈ `{SUCCESS, FAILED, UNKNOWN}`。若 `results.json` 被人为篡改（或外部工具 emit）让某条 status = `"PASS"` / `"OK"`，`report::write_report_md` 不会报错——会把它当 FAILED 处理（不匹配任何 SUCCESS / UNKNOWN / timed_out 分支，fallback 到通用 `format!("FAILED ({})", code)`），但实际它本来不是 FAILED——这是静默信号失真。

**违反**：design `detailed-design.md` §三 `write_results_json` 后置条件「status ∈ { "SUCCESS", "FAILED", "UNKNOWN" }」是输入承诺；`regenerate_report` 应作为 invariant 检查器在反序列化后断言之，目前没做。

**推理链**：runner 自己 emit 的 `results.json` 不会有这种情况，但 `runner report <run-dir>` 设计上**接受任何符合 schema 的 results.json**（包括第三方消费再重 emit 的）——按 detailed-design §四「`runner report <run-dir>` 子命令」开放给第三方用。第三方写出 status = "PASS" 类，runner 不报错，结果就被错分类——report.md 内容失真。

**决策性**：决策点——「严格校验」vs「宽松接受」。建议倾向严格（与 design 后置条件一致）。

**建议**：在 `regenerate_report` 反序列化后 fold 校验：每条 `r.status ∈ {SUCCESS, FAILED, UNKNOWN}`，否则 Err 出。或在 `report::write_report_md` 入口加同样的 assert。

---

#### #3（中）— `run_id` 仅含 `unix_secs + pid`，并发 multi-runner 同秒内 PID 冲突可重叠（实际罕见）

**位置**：`runner/src/main.rs:93-97`

**前置条件**：SystemTime now 与 process::id 可用。

**后置条件**：`run_id` 在文件系统层面唯一，不与其他 run 冲突。

**实际行为**：
```
let run_id = format!(
    "run-{}-{}",
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
    std::process::id()
);
```
同秒内不同 PID 不冲突。但若同 PID 在某些容器化场景（PID 1 复用）下并发，仍可冲突。`std::fs::create_dir_all(&run_dir)` (line 99) 不会 Err 若目录已存在——会复用，与已有 run 的 raw / results.json 互覆盖。

**违反**：未违反 design 但破坏「同次 run 内 exec_id 唯一」的间接前提（不同 run 共享 run_dir 时，task 之间会跨写 raw 文件、合并出错误 results.json）。

**推理链**：实际 macOS / linux 单机 dev 场景下不会触发；但容器化 + run-1778126061-1 类 PID 在 docker / pod 内多见。`std::fs::create_dir(&run_dir)` 替代 `create_dir_all` 可在已存在时 Err 出，提示用户。

**决策性**：决策点——是否要把容器化场景纳入考虑。按 `principles.md` §七「性能不算问题（除非升级为功能问题）」的精神，单机 dev 用例不算问题。

**建议**：低优先级。考虑加 nanos 后缀或 UUID，或用 `create_dir`（已存在则 Err）替代 `create_dir_all`。

---

#### #4（低）— `run_started_unix_secs` 与 `iso8601_utc` 在 SystemTime 异常时不一致

**位置**：`runner/src/main.rs:153-157`、`runner/src/host.rs:82-102`

**前置条件**：SystemTime now 可用。

**后置条件**：`run_started_at` 与 `run_started_unix_secs` 反映同一时刻。

**实际行为**：line 154-157 与 host.rs:83-86 都用 `duration_since(UNIX_EPOCH).map(...).unwrap_or(0)`——即 `SystemTime` 早于 UNIX_EPOCH 时双方均回退 0。但 main.rs 用 `unwrap_or(0)` 而 iso8601_utc 也 `unwrap_or(0)`——两者一致。`run_finished_*` 同理。一致性 OK。

**违反**：无。

**推理链**：边角已经处理。

**决策性**：非决策点。

**建议**：无。

---

#### #5（中）— rayon 并行块中错误捕获只 fmt 用 `{:#}` 串接 anyhow chain，未确认 chain 包含底层 IO error 信息

**位置**：`runner/src/main.rs:218-235`

**前置条件**：`exec::execute` 返回 `Err(anyhow::Error)`。

**后置条件**：错误信息保留 anyhow context chain，使下游能定位失败步骤。

**实际行为**：
```
Err(e) => {
    let err_str = format!("{:#}", e);
    ...
    error: Some(err_str),
```
`{:#}` 格式化 anyhow chain 为「context: source: source」一行。OK——chain 完整。

**违反**：无。

**推理链**：anyhow 标准用法。

**决策性**：非决策点。

**建议**：无。

---

### discover.rs

---

#### #6（高）— `target_dir` 不在 `example.root` 子目录的情况未被验证，被 exec 阶段（exec.rs:67-75）后置兜底但 discover 提前 Err 更早一阶段

**位置**：`runner/src/discover.rs:174-181`

**前置条件**：`ts.target_path.as_deref()` 给出 hirusttest.toml 的相对路径或 `"."`。

**后置条件**：（按 `detailed-design.md` §三 `find_examples` 后置条件）「target_path 已 canonicalize；若不在 root 子目录下，返回 Err」。

**实际行为**：line 174-181：
```
let target_rel = ts.target_path.as_deref().unwrap_or(".");
let target_dir = dir.join(target_rel).canonicalize().with_context(|| {...})?;
```
只 canonicalize，**未验证 `target_dir.starts_with(&dir)`**。若用户写 `target_path = "../somewhere"` 或 `target_path = "/etc"`（虽然这是配置 bug，但 discover 不报错），canonicalize 成功后 target_dir 指向 examples/ 之外的目录——后续 `exec.rs:67-75` 的 `strip_prefix(&example.root)` 会 Err 出。但那时是在第一次 task 执行时——而 discover 在配置 bug 立即报错是 design 的预期。

**违反**：违反 `detailed-design.md` §三 `find_examples` 后置条件「若 target_path 不在 root 子目录下，返回 Err」——目前实现仅在 exec 阶段 implicit 失败，且错误归类为 UNKNOWN（runner-internal）而非 discover 错误。

**推理链**：design 期望 schema 解析失败属于 `principles.md` §七错误处理策略表 §1 — "Schema 解析失败 → runner 启动时 panic"。当前实现把它降级为 per-task UNKNOWN，矛盾。

**决策性**：决策点——是否在 discover 阶段早失败。建议 yes（与 design 一致）。

**建议**：在 line 181 后加 `target_dir.starts_with(&dir).then_some(()).ok_or_else(|| anyhow!("target_path '{}' escapes example root {}", target_rel, dir.display()))?;`。

---

#### #7（中）— `HirusttestToml` deserialize 在 entries 字段缺失时直接 Err，与 design「Schema 解析失败 → runner 启动时 panic（discover 阶段错）」一致；但若 toml 含 `entries = []` 空数组，不会 Err，example 会被注册但跑出零 task

**位置**：`runner/src/discover.rs:51-55, 171-172, 220-225`

**前置条件**：hirusttest.toml 存在且 toml 合法。

**后置条件**：返回的 Example 含至少一个 entry（implicit assumption——否则 main 的 task 迭代会跳过它）。

**实际行为**：line 51-55：
```
#[derive(Deserialize)]
struct HirusttestToml {
    entries: Vec<String>,
    target_path: Option<String>,
}
```
`entries: Vec<String>` 允许空 Vec——`entries = []` 解析成功，Example 注册但 line 119-125 在 main 的 task 构造时永远不 push。设置错位的样例会被默默忽略——但 detailed-design §一 schema「`entries = ["fn_name_1"]   # 必填，零参 pub fn 名列表`」要求非空。

**违反**：弱违反——design 的「必填」语义未强制；实际行为是「列表可为空但 Vec 必存在」。

**推理链**：用户配置 bug（写错 entries 名导致空匹配 / 列表为空）下，跑出 0 tasks 是静默的——没有 warning。

**决策性**：决策点——空 entries 是 Err 还是 warn。建议 warn（不阻断整 run）。

**建议**：在 line 221（push 之前）若 `ts.entries.is_empty()` 则 `eprintln!("[warn] {} has empty entries; skipping", config_path.display())`，或返回 Err。

---

#### #8（中）— `expand_env` 对 `$VAR`（无花括号）一律不展开，但行为文档化不充分；且不处理 `\$` 转义

**位置**：`runner/src/discover.rs:289-335`

**前置条件**：`s` 是 tool.toml 的某个 command / version_command argv 元素。

**后置条件**：`${VAR}` 展开为 env value（缺失 → 空串），`$VAR`（裸 $）保留字面。

**实际行为**：line 296-334：
```
while let Some(c) = chars.next() {
    if c != '$' { ... continue; }
    if chars.peek() != Some(&'{') { out.push('$'); continue; }
    ...
}
```
`$FOO` 字面保留——OK。但若用户写 `"${PATH"`（漏 `}`），代码 line 322 `if closed && valid` 走 else 分支保留 `${PATH`——OK 但 silent；用户期望可能是「这是个 broken var ref，请报错」。design (detailed-design §五 工具配置示例) 多处明示 `${VAR}` 用法，但未规定无效引用如何处理。

**违反**：未违反 design 显式条款，但行为不够文档化。

**推理链**：`tool.toml` 失误写 `${TS_CHARON_BIN`（漏 `}`）会让 command argv 第一个元素含字面 `${TS_CHARON_BIN`，spawn 时报「No such file or directory」——错误归类为 UNKNOWN，错误信息含原 argv，user 能定位。OK 但慢一步。

**决策性**：决策点——是否在 expand_env 阶段早报「broken var ref」。

**建议**：低优先级。可加 warn：`if !closed { eprintln!("[warn] unterminated ${{ in {}", s); }`。

---

#### #9（高）— `expand_env` 对 `${VAR}` 缺失环境变量时静默展为空字符串——用户配置错误（漏 `source .env`）会让 command argv 含空元素，后续 `Command::new("")` 报 ENOENT，错误信息不直观

**位置**：`runner/src/discover.rs:322-323`

**前置条件**：用户 tool.toml 含 `${TS_XYZ}`，但用户未 `source .env`，runner 进程不见此环境变量。

**后置条件**：`Command::new("")` 或类似空 argv 元素被 spawn，给出 IO 错误。

**实际行为**：
```
if closed && valid {
    out.push_str(&std::env::var(&name).unwrap_or_default());
}
```
变量缺失 → 展为空串。例如 `command = ["${TS_CHARON_BIN}", "cargo", ...]` 未 source .env 时变成 `["", "cargo", ...]`，line 152 `Command::new(&tool.command[0])` 即 `Command::new("")`——spawn 报「No such file or directory」，error 归类 UNKNOWN。错误信息含原 argv `["{:?}"]` 即 `["", "cargo", ...]`，用户能猜出原因，但需要看 raw 数据。

**违反**：design 没明示这个细节，但「真实性诚实责任」精神要求错误信息直观。

**推理链**：「expand_env 静默 + spawn 报 ENOENT」让用户看 UNKNOWN error 时不知道根因是「漏 source .env」还是「.env 内路径填错」。两者 mitigation 不同。

**决策性**：决策点——是否在 expand_env 阶段就 Err。

**建议**：高优先级——若变量缺失，emit warning（如 `eprintln!("[warn] env var ${{{}}} unset; expanding to empty in tool.toml argv", name)`）。更激进可让 `find_tools` 直接 Err 出——但会与「.env 是 user-side concern」精神冲突。最低限度加 warning。

---

#### #10（低）— `expand_env` 变量名匹配规则缺 ASCII 小写字母

**位置**：`runner/src/discover.rs:318-321`

**前置条件**：`${name}` 中 name 是 env var 名。

**后置条件**：合法 env var 名展开，否则保留字面。

**实际行为**：
```
let valid = !name.is_empty()
    && name
        .bytes()
        .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit() || b == b'_');
```
按 doc comment（line 290-293）「Variable names must match `[A-Z_][A-Z0-9_]*`」一致——但 design `detailed-design.md` §五没规定大小写约束；POSIX env var 允许小写（如 `PATH` 一定是大写，但 user 可自定义 `home_dir`）。当前实现拒绝 `${home_dir}` 类，会保留字面 `${home_dir}` — user 可能预期展开。

**违反**：未违反 design（design 未明示）；实现强 enforce 比 doc 更严格——已与 doc comment 一致。

**推理链**：design 没约束；implementation 按 doc。统一性 OK。

**决策性**：非决策点。

**建议**：无（doc 与 impl 已一致）。

---

#### #11（中）— `find_tools` 不验证 tool.toml 中 `extra_cargo_deps` 字符串确实为合法 toml dep line

**位置**：`runner/src/discover.rs:277-278, 384-389`

**前置条件**：tool.toml 解析成功。

**后置条件**：tool.extra_cargo_deps 已可用于 patch_cargo_deps。

**实际行为**：`extra_cargo_deps: Vec<String>` 在 discover 阶段不做 toml 解析；只在 `exec::patch_cargo_deps` line 325-327 才解析：
```
let mini: toml_edit::DocumentMut = line
    .parse()
    .with_context(|| format!("parsing extra_cargo_deps entry `{}` as TOML", line))?;
```
此处失败归类 UNKNOWN。design `detailed-design.md` §七 错误表说「Schema 解析失败 → runner 启动时 panic」——但此「解析失败」发生时机被推迟到 exec 阶段，矛盾。

**违反**：违反 design §七 错误处理策略表中 schema 类错误的「discover 阶段」时机。

**推理链**：用户 tool.toml 写 `extra_cargo_deps = ["not valid toml ;;;"]`，runner 启动 OK，跑 144 tasks 时每条都 UNKNOWN。

**决策性**：决策点——schema 完整性应在 discover 还是 exec 阶段验证。建议 discover。

**建议**：在 `find_tools` line 388 push 之前 fold 遍历 `parsed.extra_cargo_deps`，对每条 `line.parse::<toml_edit::DocumentMut>()` 试解析，失败立即 Err。

---

#### #12（中）— `discover::find_examples` 的 `filter_entry`：当某中间目录无 `.no-hirusttest`、但属于 `.hirusttest/` 子树（嵌套 case）时被错跳，可能漏 example

**位置**：`runner/src/discover.rs:104-120`

**前置条件**：`examples_dir` 下任意目录可能含 `.no-hirusttest` 标记或 `.hirusttest/` 子目录。

**后置条件**：`.no-hirusttest` 整子树跳过；`.hirusttest/` 自身不下钻。

**实际行为**：
```
let walker = WalkDir::new(examples_dir).into_iter().filter_entry(|e| {
    if !e.file_type().is_dir() { return true; }
    if e.path().join(SKIP_MARKER).exists() { return false; }
    if e.file_name() == DIR_TRACK_DIRNAME { return false; }
    true
});
```
对每个目录：若它名为 `.hirusttest`——不下钻其内容。OK，符合 design。但**filter_entry 也对 files 调用**——line 105 `if !e.file_type().is_dir() { return true; }` 让 files 通过过滤器。后续主循环 line 123 `if !entry.file_type().is_dir() { continue; }` 又跳 files——OK，两重过滤等价。无 bug。

**违反**：无。

**推理链**：逻辑正确，只是冗余。

**决策性**：非决策点。

**建议**：无。

---

#### #13（中）— `find_examples` 对 `dir == examples_dir` 自己（walker root）的处理在 ambiguity check 之前发生，但若 examples_dir 自己有 `.hirusttest/` 或 `hirusttest.toml`，会 silently skip 而不报错

**位置**：`runner/src/discover.rs:129-131`

**前置条件**：`examples_dir` 是有效目录路径。

**后置条件**：examples_dir 自己不会被注册为 example。

**实际行为**：
```
if dir == examples_dir { continue; }
```
直接跳过——若用户在 `examples_dir` 自身（如 `examples/`）放了 hirusttest.toml，连警告都没有。这是边角的配置失误，但 silently skip 让 user 不知道为啥某文件被忽略。

**违反**：未违反 design。

**推理链**：edge case，影响小。

**决策性**：非决策点。

**建议**：低优先级。可加 warn：`if dir == examples_dir && (single_file.exists() || dir_track_root_exists) { eprintln!("[warn] {} is the examples root; hirusttest.toml here is ignored", dir.display()); }`。

---

#### #14（高）— `find_examples` 对 `dir_track_config_exists` 与 `single_file_exists` 同时为 true 时报 ambiguity—— **但**「`.hirusttest/` 目录存在但 `.hirusttest/config.toml` 不存在」+「`hirusttest.toml` 也存在」会触发 line 144 的 `dir_track_root_exists` 条件，报 ambiguity；而 design `detailed-design.md` §一「3. 同一目录禁止同时存在 `hirusttest.toml` 与 `.hirusttest/`」要求是 `.hirusttest/` **目录** 存在就报错——实现行为与 design 一致

**位置**：`runner/src/discover.rs:144-151`

**前置条件**：候选 `<example_dir>` 已通过 filter。

**后置条件**：双轨同时出现 → Err。

**实际行为**：
```
if single_file_exists && dir_track_root_exists {
    return Err(anyhow!("ambiguous schema at {}: both `{}` and `{}/` exist; ..."));
}
```
`dir_track_root_exists = dir_track_root.exists()` (line 139)——只要目录 `.hirusttest/` 存在（不管里面有没有 `config.toml`），就触发 ambiguity。与 design 行为一致。

**违反**：无。

**推理链**：design 表述「同一目录禁止同时存在 `hirusttest.toml` 与 `.hirusttest/`」精确匹配代码。

**决策性**：非决策点。

**建议**：无。

---

#### #15（低）— `find_examples` 中 line 195 `dir.strip_prefix(examples_dir).unwrap()` 在 walker 已保证 dir 在 examples_dir 之下时安全，但 `unwrap()` 在 panic-on-bad-input 风险评估上属警示点

**位置**：`runner/src/discover.rs:195`

**前置条件**：dir 是 WalkDir 从 examples_dir 出发遍历的——必然以 examples_dir 为前缀。

**后置条件**：`rel` 是 dir 相对 examples_dir 的路径。

**实际行为**：`let rel = dir.strip_prefix(examples_dir).unwrap();`——unwrap 在该不变式下安全。但若未来重构改 walker 调用，可能破坏。

**违反**：无。

**推理链**：unwrap 在已建立不变式下安全；属于「正当 unwrap」类。

**决策性**：非决策点。

**建议**：低优先级。可换为 `expect("dir is under examples_dir by WalkDir construction")` 显式假设。

---

### exec.rs

---

#### #16（高）— `exec::execute` 在 timeout 路径下，process group 杀掉后**仍 join stdout/stderr reader threads**，但若孙子进程死前已 close fd，reader 立即返回；若 fd 状态意外（不应该）reader 永远 block——reader 没 timeout 保护

**位置**：`runner/src/exec.rs:204-224`

**前置条件**：超时分支已 `libc::kill(-child_pid, SIGKILL)`，child 已 reap。

**后置条件**：reader threads join 返回。

**实际行为**：line 211-221（kill）+ line 223-224（join）：
```
Some(s) => (s, false),
None => {
    #[cfg(unix)]
    unsafe { libc::kill(-child_pid as libc::pid_t, libc::SIGKILL); }
    ...
    let s = child.wait()...?;
    (s, true)
}
};
let stdout_buf = stdout_reader.join().unwrap_or_default();
let stderr_buf = stderr_reader.join().unwrap_or_default();
```
注释（line 145-147）说「设独立 process group → kill -pgid 杀全 group，孙子进程一并；防止孙子 hold 住 stdout/stderr fd 让 reader 永 block」。但若**孙子进程 fork 后又 setpgid 离开本 group**（rare），kill -pgid 不杀它，reader 永 block。当前没有 reader 端的 timeout 兜底。

**违反**：未违反 design——design `detailed-design.md` §四「超时」一段明示这套机制，没要求 reader timeout。但 invariant「runner thread 不卡死」依赖「孙子进程不会逃离 process group」这一假设。

**推理链**：cargo-kani / cargo-prusti / cargo-creusot 内部不会 setpgid——它们 follow cargo 标准 spawn 流。但如果未来集成的工具调用 daemonize 库 / nohup 类 → 风险。

**决策性**：决策点——是否要 reader 端 timeout 兜底。当前 design 显式说「按性能不算问题」精神，graceful shutdown 不属功能必需。

**建议**：保留现状；若发现某工具确实有逃 group 行为，加 `recv_timeout` 兜底。

---

#### #17（高）— `copy_dir_excluding` 不处理「目标已存在但非目录」的 race：line 273 `create_dir_all(dst)` 若 dst 是个已存在的文件会 Err

**位置**：`runner/src/exec.rs:272-298`

**前置条件**：dst 在调用前已 `remove_dir_all`（line 52）；若调用方未清理，dst 可能存在。

**后置条件**：dst 是个完整副本目录。

**实际行为**：line 51-54：
```
if work_dir.exists() {
    std::fs::remove_dir_all(&work_dir)
        .with_context(|| format!("clearing {}", work_dir.display()))?;
}
```
若 work_dir 是 symlink 或 file（非目录），`remove_dir_all` Err out——任务标 UNKNOWN。可接受。

**违反**：无。

**推理链**：边角情况由 anyhow 错误 propagate 到 UNKNOWN。

**决策性**：非决策点。

**建议**：无。

---

#### #18（中）— `copy_dir_excluding` 不处理硬链接 / 设备文件——非 regular file / dir / symlink 会被 `std::fs::copy` 当 regular file 处理

**位置**：`runner/src/exec.rs:280-296`

**前置条件**：src 下任意 entry 是 dir / file / symlink。

**后置条件**：dst 下复制对应类型。

**实际行为**：line 283-296：
```
if ty.is_dir() {
    copy_dir_excluding(&from, &to, excludes)?;
} else if ty.is_symlink() {
    let target = std::fs::read_link(&from)?;
    #[cfg(unix)] std::os::unix::fs::symlink(target, &to)?;
    ...
} else {
    std::fs::copy(&from, &to)?;
}
```
若 entry 是 FIFO / socket / device，`std::fs::copy` 会试图读它——FIFO 可能 block 永远。examples/ 下不会有这种文件，但若 vendor/ submodule 含错误的 special file，runner 卡死。

**违反**：未违反 design（design 未考虑）。

**推理链**：现实风险极低——cargo crate 不含 special file。

**决策性**：非决策点。

**建议**：低优先级。可加 `if !ty.is_file() { eprintln!("[warn] skipping non-regular {}", from.display()); continue; }` 兜底。

---

#### #19（中）— `exec::execute` line 178-179 注入 `TS_ENTRY_FN` / `TS_TARGET_CRATE` 在 `env_remove("TS_*")` 之后，**但不在 strip 列表内**——OK；但如果 `parsed.command` 含 `TS_ENTRY_FN` 自定义重导出（如某 tool.toml 写 `command = ["env", "TS_ENTRY_FN=overwrite", ...]`），spawn 时 env_remove 不影响 argv，最终 child 看到的是 argv-set 值（覆盖 runner 注入的），属设计内行为

**位置**：`runner/src/exec.rs:165-179`

**前置条件**：runner env 含若干 TS_* 变量。

**后置条件**：spawn 时 child env 不含 TS_*，除了 runner 显式注入的 TS_ENTRY_FN / TS_TARGET_CRATE。

**实际行为**：line 165-171 收集所有 `TS_*` 变量名 + line 169 全部 env_remove；line 178-179 注入两个 runner-managed 的 TS_*。注释（line 158-164）明示意图：strip user-side TS_*，再 inject runner-side 两个——逻辑正确。但 `tool.toml.command` 含 `env TS_FOO=bar` 前缀的话，`env` 是另一个进程（spawn 出 sh），spawn 时 runner 设置的 TS_ENTRY_FN 会被 `env` 程序读到并 forward 到下层 — OK。

**违反**：无。

**推理链**：env 层级清晰；与 rocq-of-rust-wrapper.sh 等 wrapper 读 TS_ENTRY_FN 的契约一致。

**决策性**：非决策点。

**建议**：无。

---

#### #20（高）— `exec::execute` line 99-100 `target_crate_name` 用 `crate_name.replace('-', "_")`，但**未做 Rust 关键字 / 非法 ident 检查**。若 example crate name 含 `1abc` 或 `match` 等，渲染出的 harness 无法编译——任务标 FAILED，归因到工具能力，错误归类

**位置**：`runner/src/exec.rs:97-100`

**前置条件**：example.crate_name 来自 `Cargo.toml` 的 `[package].name`，由 cargo 自身校验。

**后置条件**：crate_ident 是合法 Rust ident。

**实际行为**：
```
let crate_ident = example.crate_name.replace('-', "_");
```
cargo 强制 crate name 满足 `[a-zA-Z0-9_-]+` 且首字符非数字——`replace('-', "_")` 后是合法 Rust ident（cargo 的 normalize 规则）。OK。

**违反**：无。

**推理链**：cargo 已守门。

**决策性**：非决策点。

**建议**：无。

---

#### #21（高）— `exec::execute` 的 `target_in_workdir` 计算用 `example.target_path.strip_prefix(&example.root)`——若 example.target_path 与 example.root 都 canonicalize 但 symlink 解析不一致（如 example.root 是 symlink, target_path 是 absolute non-symlink），strip_prefix 会 Err

**位置**：`runner/src/exec.rs:66-76`

**前置条件**：example 来自 discover；example.root 与 example.target_path 都已 canonicalize（line 175 + 通过 main.rs:74 间接保证）。

**后置条件**：target_in_workdir 是 work_dir 下的子目录。

**实际行为**：line 67-75：
```
let target_rel = example
    .target_path
    .strip_prefix(&example.root)
    .with_context(|| {...})?;
```
若 examples_dir 是 symlink，`canonicalize` 会解析它；`dir`（walker 给的）也已是解析后路径——一致 OK。但**`example.root` 字段在 line 218 `root: dir.to_path_buf()`** 中 `dir` 是 WalkDir 路径——是否 canonicalize？

查阅：WalkDir 默认不 canonicalize；但 `examples_dir` (main.rs:74) 已 canonicalize，所以 dir 是「canonical examples_dir + 相对路径」——若 examples 内含中间 symlink 指向另一处，dir 不解析它。target_dir（line 175）是 `dir.join(target_rel).canonicalize()`——解析了。两者基底相同（canonical examples_dir）但 target_dir 多解析了中间 symlink——`strip_prefix` 可能 Err。

**违反**：design `architecture.md` §五 接口规约「runner 保证副本目录是完全隔离的当前 (tool, entry) 专属目录」——若 strip_prefix Err，task 标 UNKNOWN，但 root cause 是 path 不一致，归因不清。

**推理链**：现实场景下 examples/ 下不含 symlink，所以不触发；但 vendor/ submodule 用 symlink 接入是合法做法。

**决策性**：决策点——是否一并 canonicalize `dir` 在 line 218。建议 yes。

**建议**：在 line 218 改为 `root: dir.canonicalize().unwrap_or_else(|_| dir.to_path_buf())`，或在 discover 顶部明示「dir 已 canonicalize」（实际未必）。

---

#### #22（高）— `exec::execute` 在 `EntryMode::Lib` 路径下，若原 `src/lib.rs` 包含 `mod __ts_inner;` 或符号冲突的内容，rename + harness 注入会编译失败，任务 FAILED 归因到工具能力错误

**位置**：`runner/src/exec.rs:116-139`

**前置条件**：example 含 `src/lib.rs`，tool.entry_mode == Lib。

**后置条件**：rename 后 harness 占据 `src/lib.rs`，原文件路径变 `src/__ts_inner.rs`。

**实际行为**：line 116-139：rename `src/lib.rs → src/__ts_inner.rs` + 写新 `src/lib.rs`。**未检查原文件是否已经有 `__ts_inner` 模块名占用**。若 example 本就含 `mod __ts_inner;`（极罕见），harness 模板 `mod __ts_inner; pub use __ts_inner::*;` 与原内容冲突——编译错。

**违反**：未违反 design——design `architecture.md` §一「位阶澄清」明示「entry_mode = lib」是声明式工具特定填充，承诺「原始磁盘字面零修改」+「隔离副本上的填充」。`__ts_inner` 是约定名，design 已默认它不冲突。

**推理链**：现实样例不会用 `__ts_inner` 这个 ident（项目自有命名）；冲突风险接近零。

**决策性**：非决策点。

**建议**：无。

---

#### #23（中）— `exec::execute` line 240-241 写 raw stdout/stderr 文件时，若 raw_dir 由于 race 不存在（line 229 `create_dir_all`），后续 `std::fs::write` 会 Err；但 race 在 par_iter 中可能多线程并发 create

**位置**：`runner/src/exec.rs:228-247`

**前置条件**：raw_dir 路径 `runs/<run_id>/raw/<tool>/`。

**后置条件**：三个 raw 文件已写。

**实际行为**：line 229-230 `create_dir_all(&raw_dir)`——线程安全（`create_dir_all` 内部对已存在 dir 不 Err）。write 之后 OK。

**违反**：无。

**推理链**：`create_dir_all` 是 idempotent。

**决策性**：非决策点。

**建议**：无。

---

#### #24（中）— `exec::execute` line 247 `std::fs::write(run_dir.join(&raw_exit_rel), exit_marker)` 把 timed_out 时写 `__ts_timeout (after N s)\n`，与 design `detailed-design.md` §四运行时机制「exit 内容: timed_out ? "__ts_timeout (after N s)" : "{:?}".format(exit.code())」一致

**位置**：`runner/src/exec.rs:242-247`

**前置条件**：subprocess 已完成或被 SIGKILL。

**后置条件**：raw exit 文件含状态标记。

**实际行为**：
```
let exit_marker = if timed_out {
    format!("__ts_timeout (after {} s)\n", tool.timeout_secs)
} else {
    format!("{:?}\n", exit_status.code())
};
```
与 design 一致。

**违反**：无。

**推理链**：直接落地 design 文本。

**决策性**：非决策点。

**建议**：无。

---

#### #25（中）— `exec::execute` 失败路径（spawn / patch_cargo_deps / render harness 任一失败）**不写 raw 三文件**——`raw_stdout` / `raw_stderr` 在 main.rs:212-213 `Some(r.raw_stdout_rel)` 仍取自 ExecResult，但若 execute 返回 Err，main.rs:227-235 走 UNKNOWN 路径，raw_stdout / raw_stderr 设 None——OK，与 design `detailed-design.md` §六 输出字段「UNKNOWN 任务 raw_stdout / raw_stderr null」一致

**位置**：`runner/src/exec.rs` 整体 vs `runner/src/main.rs:226-236`

**前置条件**：execute 返回 Err 或 Ok。

**后置条件**：TaskResult 字段对应状态。

**实际行为**：与 design 一致。

**违反**：无。

**推理链**：错误传播链清晰。

**决策性**：非决策点。

**建议**：无。

---

#### #26（高）— `exec::execute` 的 `extra_cargo_deps` patch 流程不验证「副本 Cargo.toml 中 `[dependencies]` 是 inline-table 形态」——`patch_cargo_deps` line 317 `as_table_mut().ok_or_else(...)` 处理，但**design 未明示「inline table 视作 Err」的语义**

**位置**：`runner/src/exec.rs:314-322`

**前置条件**：副本 Cargo.toml 存在且可解析。

**后置条件**：deps 部分含新增 deps。

**实际行为**：
```
let deps = deps_item.as_table_mut().ok_or_else(|| {
    anyhow!("[dependencies] in {} is not a table — refusing to patch", cargo_path.display())
})?;
```
若用户 example 的 Cargo.toml 写 `dependencies = { foo = "0.1" }`（inline table），`as_table_mut` 返回 None（inline-table 不是 standard table），patch Err 出——task 标 UNKNOWN。

**违反**：未违反 design——design `detailed-design.md` §三 `patch_cargo_deps` 后置条件明示「若 [dependencies] 非 table 形态，返回 Err」。一致。

**推理链**：现实样例都用 `[dependencies]` standard form。

**决策性**：非决策点。

**建议**：无。

---

### host.rs

---

#### #27（中）— `host::capture_version` 不施加 timeout——若工具 `version_command` 挂起，runner 启动阶段卡死

**位置**：`runner/src/host.rs:107-126`

**前置条件**：tool.version_command 非空。

**后置条件**：返回 Some(version) 或 None。

**实际行为**：line 111-114：
```
let out = Command::new(&version_command[0])
    .args(&version_command[1..])
    .output()
    .ok()?;
```
`output()` 是阻塞调用，无 timeout。若某工具 `--version` 实现 broken（连服务器查 license / 等 stdin / hang）→ runner 永远阻塞在启动阶段。

**违反**：未违反 design——但 invariant「runner 总是能在 host info 收集后进入执行循环」依赖此假设。

**推理链**：现实工具 `--version` 不会 hang，但不保证。`cmd_oneline` (line 40-47) 同问题——`uname` / `hostname` / `sysctl` 不会 hang，但若用户系统状态异常，理论可能。

**决策性**：决策点——是否给 capture_version 一个 hard timeout（如 5 秒）。

**建议**：中优先级。引入 wait-timeout，给 capture_version 一个 5-10s 限制；失败回退 None。

---

#### #28（低）— `host::iso8601_utc` 使用 `libc::gmtime_r` 是 thread-safe，但 cast `secs as libc::time_t` 在 32-bit time_t 平台（旧 mac）上溢出（2038 Y2K38）

**位置**：`runner/src/host.rs:82-102`

**前置条件**：t 是 SystemTime now。

**后置条件**：返回 ISO 8601 字符串。

**实际行为**：line 87 `let secs_t = secs as libc::time_t;`——`libc::time_t` 在 macOS arm64 / linux x86_64 是 i64，不溢出。32-bit 平台溢出，但本项目 only targets 64-bit。OK。

**违反**：无。

**推理链**：64-bit 环境 OK。

**决策性**：非决策点。

**建议**：无。

---

### report.rs

---

#### #29（高）— `report::write_report_md` line 177-178 计算 p50 / p90 时未防护 n=0——但调用前 line 166-168 已 `if n == 0 { continue; }`，OK——**但 line 178 `(n * 9) / 10` 当 n=1 时为 0，line 178 取 `durs[0]`——OK，但 p90 数值意义在 n=1 时不严谨**

**位置**：`runner/src/report.rs:174-179`

**前置条件**：n > 0。

**后置条件**：avg / p50 / p90 / max 数值合理。

**实际行为**：
```
let mut durs: Vec<u128> = rs.iter().map(|r| r.duration_ms).collect();
durs.sort_unstable();
let avg = durs.iter().copied().sum::<u128>() / (n as u128);
let p50 = durs[n / 2];
let p90 = durs[(n * 9) / 10];
let max = *durs.last().unwrap_or(&0);
```
n=1: p50 = durs[0], p90 = durs[0]——都是同一数，数学上正确（单点分布的所有分位都相等）。n=10: p50 = durs[5]（理想 5），p90 = durs[9]（最大值）——是「最近邻」分位算法，不是线性插值；与 numpy default 不同但 percentile 算法选取是接受的。

**违反**：无——design 没规定哪种分位算法。

**推理链**：现实场景下 per-tool n 通常 >= 100，差异微小。

**决策性**：非决策点。

**建议**：低优先级。在 report.md header 注明分位算法或 disclaimer。

---

#### #30（中）— `report::write_report_md` line 197-199 把 entry_id 按 `/` 分组取 `feature = first segment`——若 entry_id 因 bug 是空串或不含 `/`，feature 变 ""——会出现空 feature 行；与 design 「entry_id 形如 `<feature>/<dir>/<entry-fn>`」要求一致，但**未在入口处验证**

**位置**：`runner/src/report.rs:96-108, 197-200`

**前置条件**：每条 entry_id 形如 `<feature>/<dir>/<entry-fn>`。

**后置条件**：feature × entry × tool 矩阵正确分组。

**实际行为**：line 98 `let mut parts = r.entry_id.splitn(2, '/');` + line 99-100 `parts.next().unwrap_or("")`——若 entry_id 不含 `/`，feature = entry_id（整个），local = ""；line 197-199 同。是 silent fallback，不报错。design `detailed-design.md` §三 write_report_md 前置条件「每条 results[i].entry_id 形如 `<feature>/<dir>/<entry-fn>`」是输入承诺，但实现是宽容的。

**违反**：弱违反——design 是「输入承诺」，实现按宽容处理；不是必修。

**推理链**：runner emit 的 results.json 不会有这种情况；但 `runner report <run-dir>` 接受外部 results.json 时风险。

**决策性**：决策点——同 #2，建议严格校验。

**建议**：与 #2 合并修。

---

#### #31（中）— `report::write_report_md` 在多文件累积写 markdown 时使用 `String::push_str`，n=10000 tasks 时字符串内存 ~MB——不算性能问题，但 `std::fs::write` 一次性写大字符串时若 disk 已满会 Err，task 已完成但 report 没写——data 可恢复（results.json 优先写于 line 259）

**位置**：`runner/src/report.rs:91-275`

**前置条件**：results 非空。

**后置条件**：report.md 写入。

**实际行为**：先写 results.json 再写 report.md（main.rs:259-260）；即便 report.md 写失败，results.json 已落地，可用 `runner report <run-dir>` 回放。良好的失败 ordering。

**违反**：无。

**推理链**：data loss 风险已被 ordering 缓解。

**决策性**：非决策点。

**建议**：无。

---

#### #32（低）— `report::write_report_md` line 245-256 的 cell formatting：`Some(r) if r.timed_out` 在 status == "FAILED" 时已经独立于 r.timed_out 判断；若 status == "UNKNOWN" 且 r.timed_out == true（不可能但 schema 上允许），cell 字串显示 "UNKNOWN" 而非 "FAILED (timeout)"——与 design 「timed_out = true ⟹ status = FAILED」精神一致

**位置**：`runner/src/report.rs:244-257`

**前置条件**：TaskResult.timed_out 与 .status 一致（timed_out → FAILED）。

**后置条件**：cell 显示对应文本。

**实际行为**：
```
let cell = match by_tool.get(*t) {
    Some(r) if r.status == "SUCCESS" => "SUCCESS".to_string(),
    Some(r) if r.status == "UNKNOWN" => "UNKNOWN".to_string(),
    Some(r) if r.timed_out => "FAILED (timeout)".to_string(),
    Some(r) => { ... format!("FAILED ({})", code) }
    None => "-".to_string(),
};
```
按 design invariant，UNKNOWN ∩ timed_out 不出现。OK。

**违反**：无。

**推理链**：design invariant 隐式保护。

**决策性**：非决策点。

**建议**：无。

---

### Cargo.toml

---

#### #33（低）— 依赖版本宽松（`clap = "4"`、`serde = "1"` 等），无 lock 文件 (`Cargo.lock` 被 .gitignore 排除)

**位置**：`runner/Cargo.toml:6-18`、`.gitignore`

**前置条件**：开发者首次 build runner。

**后置条件**：cargo resolve 依赖到具体版本，写本地 Cargo.lock（未 commit）。

**实际行为**：版本 spec 用 caret 默认，无 `Cargo.lock` 提交——每个开发者 / CI run 可能拿到不同 patch 版本。但 runner 是 binary（不是 lib），习惯做法是 commit Cargo.lock。

**违反**：未违反 design——但 design `tool-integration.md` §一「锁定 commit hash / 版本号」精神也适用于本项目自身依赖。runner 行为可重现性受影响。

**推理链**：现实下 clap 4 / serde 1 / toml 0.8 等 patch 升级不破 API；但若某 patch 引入 regression，runner 行为变。

**决策性**：决策点——是否提交 Cargo.lock。Rust 官方推荐 binary crate 提交 lock；本项目 .gitignore 排除。

**建议**：中优先级。考虑 commit `runner/Cargo.lock`（保留根 `/Cargo.lock` 排除，仅 runner 单独）；与 design「数据自描述、可重现」精神一致。

---

#### #34（低）— `wait-timeout = "0.2"`、`libc = "0.2"` 都是已知稳定 minor，version 选取合理

**位置**：`runner/Cargo.toml:15-17`

**实际行为**：OK。

**违反**：无。

**决策性**：非决策点。

**建议**：无。

---

### 跨文件审查

---

#### #35（高）— `TS_*` envvar 命名约定在 runner 与 wrapper 之间的契约：runner 在 spawn 前 strip 所有 `TS_*`，再重新 inject `TS_ENTRY_FN` / `TS_TARGET_CRATE`——但 design `detailed-design.md` 没有专门一节明示这套契约；只在 `tools/<name>/{tool.toml,wrapper.sh}` 内的注释片段中分散提到

**位置**：跨 `runner/src/exec.rs:158-179` + `tools/rocq-of-rust/rocq-of-rust-wrapper.sh:60-65` + `tools/aeneas-{coq,lean,fstar,hol4}/tool.toml` + `.env.example` 前言

**前置条件**：runner 与 wrapper 之间通过 envvar 通信。

**后置条件**：wrapper 看到 runner 注入的 TS_ENTRY_FN 和 user-side 重导出的非 TS_ 变量（如 CHARON_BIN / VERIFAST_BIN / HAX_ENGINE_BINARY）。

**实际行为**：实现正确——`tool.toml` 用 `env CHARON_BIN=${TS_CHARON_BIN}` 在 user-side 重导出，spawn 前 runner strip TS_*，再 inject TS_ENTRY_FN / TS_TARGET_CRATE——wrapper 看到 CHARON_BIN + TS_ENTRY_FN，正确。但 design 没正式声明这套契约，依赖各 tool.toml / wrapper.sh 内的注释。

**违反**：design 文档化不足——`detailed-design.md` §四「子进程 spawn」段未明示 TS_* strip 规则；只在 `.env.example` 前言提了一句。

**推理链**：未来加新工具 / 新维护者读 design 不会立刻明白 TS_* 契约。

**决策性**：决策点——是否在 design 中加一节「子进程 env 契约」。建议 yes。

**建议**：高优先级（design 层修补）。在 `detailed-design.md` §四 加一段「子进程 env 契约：runner strip `TS_*` 后只 inject TS_ENTRY_FN / TS_TARGET_CRATE，wrapper 通过 `tool.toml` 的 `env KEY=${TS_*}` 重导出 user-side 配置」。

---

#### #36（中）— 错误信号从 wrapper → exec.rs → report.rs 的传播链：wrapper exit 非零 → child exit_status 非零 → ExecResult.status = Failed → TaskResult.status = "FAILED" + exit_code = Some(code) → report.md 显示 "FAILED (code)"。链条完整。但 **wrapper 内部 stderr 输出**（如 hax-lean oracle FAIL 行）仅落到 raw stderr 文件，**不出现在 report.md cell 内**——cell 只显示 exit code

**位置**：跨 `tools/hax-lean/tool.toml:16` (wrapper) + `runner/src/exec.rs:240-241, 256-260` + `runner/src/report.rs:244-256`

**前置条件**：wrapper 输出 `[hax-lean-oracle] FAIL: ...` 到 stderr 并 exit 1。

**后置条件**：报告显示「该 entry 在该工具上 FAILED」。

**实际行为**：raw stderr 文件已保留 oracle FAIL 信息——下游 cc-report 写作者可读 raw 数据归因。report.md 本身不显示 oracle 文本，符合 design「report.md 是 raw 数据的总览，详细需读 raw」。

**违反**：无。

**推理链**：链条完整；user 需要看 raw 文件理解 FAIL 细分。

**决策性**：非决策点。

**建议**：无（已在 cc-report 写作流程中由 reporter 手动归类）。

---

#### #37（中）— 派生原则 C「异质性归配置，框架代码同质」——审查 runner 代码无 `if tool == "kani"` / `match tool.name { ... }` 类硬编码；所有差异通过 `tool.toml + harness.rs.tera` + `extra_cargo_deps` + `entry_mode` 字段表达。✅

**位置**：全 `runner/src/*.rs`

**前置条件**：N/A。

**后置条件**：runner 代码对所有工具同质。

**实际行为**：grep 全部 .rs 文件，无任何工具名硬编码。`EntryMode::{Bin, Lib}` 是类型层抽象，不绑定具体工具。✅

**违反**：无。

**推理链**：design 派生原则 C 兑现。

**决策性**：非决策点。

**建议**：无。

---

#### #38（中）— design `detailed-design.md` §一「`[runnable.<entry_fn>]` 段是否真的被 serde 默认忽略」：审查 `discover.rs:51-55` 的 `HirusttestToml` struct——只有 `entries` 与 `target_path` 两字段，**未设 `#[serde(deny_unknown_fields)]`**。多余的 `[runnable.*]` 表被 serde 默认忽略。✅

**位置**：`runner/src/discover.rs:51-55`

**前置条件**：hirusttest.toml 可能含 `[runnable.<fn>]` 扩展段。

**后置条件**：解析成功，extension 段被忽略。

**实际行为**：默认 serde 允许 unknown fields；测试方法（不写 `deny_unknown_fields`）正确实现了 design 「向后兼容」承诺。✅

**违反**：无。

**推理链**：design `detailed-design.md` §一「Schema 向后兼容性」明示「未设 deny_unknown_fields → 多出的 `[runnable.*]` 表会被 serde 忽略」与代码一致。

**决策性**：非决策点。

**建议**：在 doc-comment 中显式说明（line 49-50 已说明，OK）。

---

#### #39（中）— `host::cpu_brand` / `total_mem_mb` 在非 macOS / 非 linux 上返回 None——report.md cell 显示「? / 0 MB」，与 design「失败的字段 None」精神一致

**位置**：`runner/src/host.rs:49-78`

**前置条件**：runtime OS。

**后置条件**：受支持 OS 给出值；其他返回 None。

**实际行为**：windows / freebsd 类返回 None；report.md 显示 0 MB / "?cpu"——稍丑但 honest。

**违反**：无。

**推理链**：design 没要求 host info 跨平台完整。

**决策性**：非决策点。

**建议**：无。

---

#### #40（高）— 「test 运行原子 = 单 entry」不变式：`exec::execute` 对 `(tool, example, entry)` 唯一三元组操作；同 example 多 entry 在 main.rs:119-136 展开为多个 task。OK。但 **同次 run 内 exec_id 唯一性依赖 `sanitize(tool)__sanitize(feature)__sanitize(dir)__sanitize(entry)` 在 corpus 内不碰撞**——sanitize 把 `/ \ : ` 全替 `_`，若两个不同 entry path 替换后碰撞（如 `vec/basic-pop` 和 `vec_basic-pop`），exec_id 冲突，work_dir 互覆盖

**位置**：`runner/src/exec.rs:41-48, 339-346`

**前置条件**：tool / feature / dir / entry 字符集互不引入碰撞。

**后置条件**：exec_id 在同次 run 内唯一。

**实际行为**：`vec/basic-pop` → `vec_basic-pop`、`vec_basic-pop` → `vec_basic-pop`——冲突！若 corpus 含两个 example feature 名 `vec`（dir = `basic-pop`）和 feature `vec_basic-pop`（dir = 任意），sanitize 后 ID 段拼接可能碰撞。

**违反**：design `detailed-design.md` §二「由于 cargo crate 名 / Rust ident / 文件系统路径都不含这四种字符，sanitize 在实践中不发生碰撞」——但这是「实践不发生」，不是「形式不发生」。design 不证；若现实 corpus 引入特定 feature/dir 名，可能碰撞。

**推理链**：当前 142 个 hirusttest.toml 中无 feature 名带下划线匹配此模式；但未来 corpus 扩展时风险存在。

**决策性**：决策点——是否换无歧义 separator（如 `\x1f` ASCII unit separator，文件系统允许）。

**建议**：中优先级。把 exec_id / slug 的分隔符 `__` 换成 sanitize 时一定不出现的 token；或在 discover 阶段断言「所有 (feature, dir, entry) 三元组 sanitize 后唯一」。

---

## §4 决策点 vs 非决策点汇总

| 编号 | 严重度 | 决策点？ | 摘要 |
|---:|:---:|:---:|---|
| #1 | 中 | 否 | CLI 子命令 global 标志冗余 |
| #2 | 高 | **是** | regenerate_report 不校验 status 枚举 |
| #3 | 中 | **是** | run_id 同秒 PID 冲突 |
| #4 | 低 | 否 | iso8601 与 unix_secs 一致性 OK |
| #5 | 中 | 否 | rayon 错误 fmt OK |
| #6 | 高 | **是** | target_dir 不在 root 子目录未验证 |
| #7 | 中 | **是** | entries=[] 静默注册 |
| #8 | 低 | **是** | broken var ref 静默 |
| #9 | 高 | **是** | env var 缺失静默展空 |
| #10 | 低 | 否 | expand_env 大小写约束与 doc 一致 |
| #11 | 中 | **是** | extra_cargo_deps schema 校验时机推迟 |
| #12 | 中 | 否 | filter_entry 冗余但正确 |
| #13 | 中 | 否 | examples_dir 自身 silently skip |
| #14 | 高 | 否 | 双轨 ambiguity 检测正确 |
| #15 | 低 | 否 | strip_prefix unwrap 安全 |
| #16 | 高 | **是** | reader thread 无 timeout 兜底 |
| #17 | 中 | 否 | dst 非目录 race 已处理 |
| #18 | 中 | 否 | special file 处理 |
| #19 | 中 | 否 | TS_* env 层级正确 |
| #20 | 高 | 否 | crate_ident 合法性靠 cargo 守门 |
| #21 | 高 | **是** | symlink + canonicalize 不一致 |
| #22 | 高 | 否 | `__ts_inner` 冲突理论存在但实践零 |
| #23 | 中 | 否 | raw_dir race 已处理 |
| #24 | 中 | 否 | exit_marker 格式正确 |
| #25 | 中 | 否 | UNKNOWN 路径 raw=None 一致 |
| #26 | 高 | 否 | inline-table Cargo.toml 已处理 |
| #27 | 中 | **是** | version_command 无 timeout |
| #28 | 低 | 否 | iso8601 32-bit time_t 溢出非本项目目标 |
| #29 | 高 | 否 | p50 / p90 算法选取合理 |
| #30 | 中 | **是** | entry_id 格式入口验证缺 |
| #31 | 中 | 否 | write_report_md 与 write_results_json ordering 安全 |
| #32 | 低 | 否 | UNKNOWN ∩ timed_out 不出现 |
| #33 | 低 | **是** | Cargo.lock 未 commit |
| #34 | 低 | 否 | wait-timeout / libc 版本选取合理 |
| #35 | 高 | **是** | TS_* envvar 契约 design 文档化不足 |
| #36 | 中 | 否 | wrapper stderr 传播链完整 |
| #37 | 中 | 否 | 派生原则 C 已兑现 |
| #38 | 中 | 否 | `[runnable.*]` 段被 serde 忽略 ✅ |
| #39 | 中 | 否 | 跨平台 host info 失败 fallback OK |
| #40 | 高 | **是** | sanitize 碰撞理论存在 |

**决策点（需用户拍板）**：#2、#3、#6、#7、#8、#9、#11、#16、#21、#27、#30、#33、#35、#40。

**非决策点（实现细节，可直接修）**：#1、#4、#5、#10、#12-15、#17-20、#22-26、#28、#29、#31、#32、#34、#36-39。

---

## §5 审查结论

### 5.1 高严重度 top 5 问题

按"实际影响 × 触发概率"排序：

1. **#9（高）— `expand_env` 环境变量缺失静默展空**（`runner/src/discover.rs:322-323`）。用户漏 `source .env` → command argv 含空元素 → spawn 报 ENOENT → UNKNOWN 错误信息不直观。这是用户体验上的真坑——新 contributor 跑 runner 第一次会卡 5 分钟在「为啥是 UNKNOWN」。建议加 warning。
2. **#6（高）— `target_dir` 不在 `example.root` 子目录的情况未被 discover 早 Err**（`runner/src/discover.rs:174-181`）。design `detailed-design.md` §三 明示「若不在 root 子目录下，返回 Err」，实现仅在 exec 阶段隐式失败、错误归类为 UNKNOWN（runner-internal）而非 discover schema 错。
3. **#35（高）— `TS_*` envvar 契约 design 层文档化不足**（跨 `runner/src/exec.rs:158-179` 与 19 个 wrapper / tool.toml）。strip + re-inject 规则正确实现，但 design `detailed-design.md` §四「子进程 spawn」段未明示——未来加新工具 / 新维护者读 design 不会立刻明白契约。
4. **#16（高）— reader thread 无 timeout 兜底**（`runner/src/exec.rs:223-224`）。若孙子进程在 SIGKILL 时已 setpgid 离开 group，reader 永 block → runner thread 卡死。现实工具不会这么做，但 invariant 风险存在。
5. **#40（高）— sanitize 碰撞**（`runner/src/exec.rs:339-346`）。同 run 内 exec_id 唯一性依赖「实践不碰撞」假设，理论上可碰撞。当前 142 个 example 不触发，未来 corpus 扩展时风险。

### 5.2 design / 实施明显冲突

发现 3 处显著冲突：

- **#6**：design `detailed-design.md` §三 `find_examples` 后置条件「target_path 已 canonicalize；若不在 root 子目录下，返回 Err」——实现未做 root containment check。
- **#11**：design `detailed-design.md` §七 错误处理策略表「Schema 解析失败（Cargo.toml / hirusttest.toml / tool.toml）→ runner 启动时 panic（discover 阶段错——配置 bug，不该静默）」——`extra_cargo_deps` 内 toml 行的语法错误推迟到 exec 阶段被发现并归 UNKNOWN。
- **#7**：design `detailed-design.md` §一「entries = ["fn_name_1"]   # 必填，零参 pub fn 名列表」——`entries = []` 解析成功但 example silently 0-task；建议明示空 entries 报错或 warn。

均建议在下一轮迭代修复（决策点性质——见 §4 表）。

### 5.3 panic-on-bad-input 风险点

`grep -n "unwrap\|expect"` 全扫共 16 处 `unwrap()` / `expect()`：

- **安全 unwrap**（在已建立的不变式下不会失败）：discover.rs:195（strip_prefix——WalkDir 保证）、main.rs:95（duration_since UNIX_EPOCH——SystemTime now 不在 EPOCH 前）、main.rs:146（available_parallelism——已 unwrap_or(1)）、host.rs:86（已 unwrap_or(0)）、main.rs:157, 247（已 unwrap_or(0)）。
- **expect with msg**：exec.rs:190-191（`expect("piped stdout")` / `expect("piped stderr")`）——`Command` 设 `Stdio::piped()` 后 `take()` 必返 Some，安全。
- **unwrap_or / unwrap_or_default**：均给了 fallback，不 panic。

**无 panic-on-bad-input 风险**——所有用户输入路径（hirusttest.toml / tool.toml / CLI 参数）都走 anyhow Err 传播。✅

### 5.4 整体评价

- **runner 代码与 design 一致性**：90% 兑现 design 的前后置条件。3 处显著冲突（#6 / #7 / #11）属于 design 严格 vs 实现宽容的取向偏差，建议向严格收敛。
- **错误信号传播链**：完整——subprocess exit code / timeout / 信号杀 / stderr / stdout 全部完整落地到 raw 文件 + results.json。wrapper 内的 oracle FAIL 信号通过 exit code + stderr 完整传播。
- **派生原则 C「异质性归配置」**：✅ 完整兑现——runner 代码无任何工具名硬编码。
- **panic-on-bad-input 抗性**：✅ 通过——所有用户输入路径走 anyhow，无 panic。
- **并发安全**：依赖 design 给出的「每 task 独立 work_dir + per-task raw 三文件路径」契约——文件系统层 race 不存在。process group + reader thread + timeout 三机制正确组合，但 reader 端 timeout 是软肋（#16）。
- **可重现性**：runner 自身的 Cargo.lock 未提交（#33）——与 design「数据自描述、可重现」精神弱冲突；建议改。

**整体认定**：runner 实现层质量良好。3 处显著 design / 实施冲突（#6 / #7 / #11）需在下一轮 fixes 中处理；其余高严重度问题（#9 / #16 / #21 / #27 / #35 / #40）属于「现实未触发但风险存在」类，按优先级分批修。
