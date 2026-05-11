# 细化设计：Rust 验证工具特性覆盖测试框架

架构层在 [`architecture.md`](architecture.md)。本文给出 schema 完整定义、runner 函数级前后置、运行时伪码、所有已集成工具的完整配置实例。**约束**：本文不改变架构层确定的模块划分、接口规约和核心流程；如发现需要变更须回到架构阶段。

## 一、Schema

> **形式定义保证**（[`principles.md`](principles.md) §四原则 A 的形式定义）：
> `hirusttest.toml` 是项目自有 schema，不在 cargo / rustc 识别的文件名集合——加不加它，对 example 上任意 `cargo` 子命令的输出**字节级一致**。这是 examples 端"自身完善独立"的实现保证。
> `tools/<name>/{tool.toml, harness.rs.tera, *.sh}` 是**集成者描述工具自身行为**的桥接信号，**不是工具自身的修改**——工具本身（cargo-kani / cargo-prusti 等）作为黑盒，绝不为框架适配。

### 双轨 example schema（按 [`principles.md`](principles.md) §四 A 双轨 schema）

按 example 复杂度选轨。两轨都满足"cargo / rustc 不读 → 行为字节级一致"的形式定义。

#### 单文件轨（默认，简单单特性测试）

`<example_dir>/hirusttest.toml`：

```toml
entries = ["fn_name_1", "fn_name_2"]   # 必填，零参 pub fn 名列表
target_path = "subdir"                  # 可选，默认 "."；多 crate 样例时被测 crate 相对路径
```

每个 entry 的 fn 必须满足：

- 在 `target_path` 指向的 crate 的 `src/lib.rs` 中作为 `pub fn` 出现
- 零参，返回类型任意
- 函数体是 plain Rust，**禁止** `#[<tool>::...]` / `#[cfg(<tool>)]` / `extern crate <tool>;` / 工具相关 dep

##### `[runnable.<entry_fn>]` 扩展段（专为档 3 一致性测试预留，可选）

为支持翻译类工具的"运行一致性"测试（按 [`hax-lean-consistency-design-2026-05-11.md`](hax-lean-consistency-design-2026-05-11.md) §2 schema 设计），单文件轨 `hirusttest.toml` 可追加一组 `[runnable.<entry_fn>]` 表。**该段为可选**——缺省时 entry 仅参与档 0 / 档 1 等不需要值比对的测试。

```toml
entries = ["add_two", "fact"]

[runnable.add_two]
inputs   = [[3, 4], [10, -7], [0, 0]]   # 必填，每组实参顺序与 fn 形参一致
expected = [7, 3, 0]                      # 必填，与 inputs 同长度

[runnable.fact]
inputs   = [[5], [6]]
expected = [120, 720]
```

**ID 与真 fn 名**：v1 直接令 ID = 真带参 fn 名（即 `entries = ["add_two"]` 中的 `add_two` 就是 `pub fn add_two(a: i32, b: i32) -> i32` 的 fn 名）。这意味着所有 `entry_mode = "bin"` 且 harness 默认调 zero-arg 形态的工具（cargo-check / kani / miri / ...）在 runnable entry 上的 harness 编译会失败 = FAILED——这是已知代价，对应 [hax-lean-consistency-design](hax-lean-consistency-design-2026-05-11.md) §3.6 的对称表现（non-runnable 在 hax-lean-eval 上 FAILED 是有意为之的诚实表态；runnable 在 zero-arg 工具上 FAILED 同理）。

字段语义（v1 仅以下 2 个必填字段被读取；其余预留字段 serde 默认忽略，将由后续版本逐步实施）：

| 字段 | 类型 | 必填 | v1 是否读 | 含义 |
|---|---|---|---|---|
| `inputs` | Array of Arrays | 是 | 是 | 每内 Array 是一组实参；`inputs.len() = 测试组数` |
| `expected` | Array | 是 | 是 | 与 inputs 同长；每元素是该组对应的预期返回值 |
| `input_types` | Array of String | 否 | 否 | 参数类型注解（如 `["i32", "i32"]`）。v1 由 fn 签名推断 |
| `return_type` | String | 否 | 否 | 返回 Rust 类型；默认 `"i32"`；用于 Lean unwrap 策略选择 |
| `rust_main_override` | String | 否 | 否 | 自定义 Rust 侧 main 模板 |
| `lean_eval_override` | String | 否 | 否 | 自定义 Lean 侧 `#eval` 表达式模板 |
| `compare_mode` | String | 否 | 否 | `"exact"`（默认）/ `"epsilon"`（v1 不支持） |
| `compare_epsilon` | Number | 否 | 否 | epsilon 模式容差 |

**类型支持矩阵**（v1）：

- ✅ 参数 / 返回类型 ∈ { `i8 / i16 / i32 / i64 / i128 / u8 / u16 / u32 / u64 / u128 / bool` }
- ⛔ `tuple / array / struct / enum` 作为参数 / 返回类型 v1 不支持（normalize 复杂度高，留 v2）
- ⛔ 浮点（`f32 / f64`）v1 不支持（Rust `Debug` 与 Lean `#eval` 输出格式不一致，需 epsilon 模式才能比对）

**纯内部化判定准则**（runnable entry 函数体必须满足）：

- ✅ 基础算术 `+ - * /`、比较 `== != < <= > >=`、布尔操作 `&& || !`
- ✅ `if-else` 控制流、`match` 表达式、自递归 / 互递归
- ✅ 自定义 `struct` / `enum` + `impl` 方法（参数 / 返回仍限 v1 类型矩阵）
- ⛔ `i*::checked_* / overflowing_*`（hax-lean prelude 未实现 → 翻译产物含 sorry）
- ⛔ `Vec / VecDeque / HashMap / HashSet / String / Box / Rc / Arc`
- ⛔ `panic! / unwrap() / expect() / assert!`（panic 让 Rust 侧 exit ≠ 0，无法字面比对）
- ⛔ `println! / std::io::*`（IO 不可比）
- ⛔ `thread::spawn / async / unsafe / raw pointer`

**ID 语义不变**：`[runnable.<entry>]` 段不创建新 entry——它扩展已存在 entry（必须先出现在 `entries = [...]` 列表）。该 entry 的 ID 仍为 `<feature>/<dir>/<entry>`。这条由消费 runnable 的工具（如未来的 `tools/hax-lean-eval/`）在 wrapper 内自筛——runner 不感知 runnable 标记。

**Schema 向后兼容性**：`runner/src/discover.rs` 的 `HirusttestToml` 结构未设 `deny_unknown_fields`，多出的 `[runnable.*]` 表会被 serde 忽略，不破现有 142 个 hirusttest.toml。

#### 目录轨（仅外部综合项目，如 vendor/x509-parser、vendor/openssl）

`<example_dir>/.hirusttest/config.toml`：

```toml
# 核心字段，与单文件轨同
entries = ["fn_name_1", "fn_name_2"]
target_path = "src"                     # 可选，默认 "."

# 可选扩展字段（按需开启，runner 渐进支持）：

# [entry_overrides.<entry_fn>]          # per-entry 覆盖（未来实施）
# harness_template = "alt-harness.rs.tera"  # 该 entry 用 .hirusttest/ 内的备选模板
# tool_overrides = { kani = "stub-kani.rs" } # per-tool 的 entry 覆盖

# [tools.<tool_name>]                   # per-tool 的此 example 配置（未来实施）
# extra_command_args = ["--extra-flag"]
```

`.hirusttest/` 目录内可包含的辅助文件（v1 仅占位，渐进实施）：

- `<entry-fn>-spec.rs`：per-entry 的 verifier spec stub（如 creusot_contracts spec、prusti `#[ensures]` block 等）。runner 在隔离副本上把这类 stub 注入 src/——与原项目 src/ 不冲突
- `<tool-name>-harness.rs.tera`：per-tool 的 harness 模板覆写（覆盖 `tools/<tool>/harness.rs.tera` 的默认）
- `README.md`：本 example 的人类可读说明（不被 runner 读）

**目录轨的硬性边界**（防止双轨退化）：

1. **禁止简单单特性测试升级到目录轨**——`examples/closure/fn-fnmut/` 这类必须用单文件
2. **外部综合项目必须用目录轨**——即便目前只有一个 entry，也用目录轨预留扩展空间（未来加 stub / 覆写时不需要文件结构迁移）
3. **同一目录禁止同时存在 `hirusttest.toml` 与 `.hirusttest/`**——runner discover 阶段报错
4. `.hirusttest/` 内的 `<entry>-spec.rs` 等辅助文件**可被工具读取**（这是受控的工具适配，发生在 framework 中介层调度时）；但 example 自身的 `src/` 与 `Cargo.toml` 仍需满足形式定义"cargo 行为字节级一致"——`.hirusttest/` 整目录加不加都不影响 cargo

### `tools/<name>/tool.toml`

```toml
command          = ["cargo", "kani", "--bin", "__ts_harness"]   # 必填，argv 数组
timeout_secs     = 300                                           # 可选，默认 300
extra_cargo_deps = ['creusot-std = "0.11.0"']                    # 可选，默认 []
entry_mode       = "lib"                                         # 可选，默认 "bin"，取值 "bin" | "lib"
version_command  = ["cargo", "kani", "--version"]                # 可选，默认 []
```

字段详解：

- **command**：`String` 数组，runner 原样作 argv 传给进程启动 API。第一个元素是可执行路径或 PATH 中可解析的名字；其余为参数。需注入环境变量时用 `env KEY=VAL …` 前缀（参见 §五 Prusti）。
- **timeout_secs**：`u64`，超时后 runner 用 `kill(-pgid, SIGKILL)` 杀整个 process group，任务标 FAILED + `timed_out: true`。
- **extra_cargo_deps**：每条是一行 TOML 风格的 dependency 声明（如 `crate-name = "x.y.z"`）。runner 用 toml AST 把它 inject 到工作副本 `Cargo.toml` 的 `[dependencies]` 表（同名 key 覆盖）。仅作用于隔离副本，原磁盘样例不变。
- **entry_mode**：`"bin"`（默认）让 harness 写到 `src/bin/__ts_harness.rs`，原 `src/lib.rs` 仍是顶级 lib target；`"lib"` 让 runner 把原 `src/lib.rs` 改名为 `src/__ts_inner.rs`，再把 harness 写到新 `src/lib.rs` 当顶级 lib（harness 模板需用 `mod __ts_inner; pub use __ts_inner::*;` 把原 lib 内容拉回）。用于 Creusot 等强制顶级 lib 自身 import 工具 prelude 的工具。
- **version_command**：`String` 数组，runner 在每次 run 启动时执行一次以采集工具版本字符串，存入 `results.json` 的 `tools[].version` 字段。空数组 / 缺省 / 命令失败均 → `version: null`。优先取 stdout，否则取 stderr；trim 后超过 500 字节截断。版本输出在 stderr（如 prusti）时，用 `["sh", "-c", "... 2>&1"]` 合流。

### `tools/<name>/harness.rs.tera`

[Tera](https://keats.github.io/tera/) 模板。可用变量恰好两个：

| 变量 | 类型 | 含义 |
|---|---|---|
| `target_crate_name` | `String` | 来自样例 `Cargo.toml` 的 `[package].name` |
| `entry_fn` | `String` | 当前 task 的 entry 名 |

变量集**固定**——不允许 tool config 扩展（破坏原则 C 的"标准词汇"约束）。

## 二、ID 派生 + 文件名 sanitize

ID 格式：`<feature>/<dir>/<entry-fn>`，三段分别取自 `examples/` 第一层目录名、第二层目录名、`hirusttest.toml` 中 `entries` 注册项。无独立 `id` 字段——path 是事实，metadata 不重复声明。

`sanitize` 把 `'/'`、`'\\'`、`':'`、`' '` 替换为 `'_'`，其他字符不变。用于：

- exec_id = `sanitize(tool)__sanitize(feature)__sanitize(dir)__sanitize(entry)` —— 工作目录唯一名
- slug = `sanitize(feature)__sanitize(dir)__sanitize(entry)` —— raw output 文件名

由于 cargo crate 名 / Rust ident / 文件系统路径都不含这四种字符，sanitize 在实践中不发生碰撞。

## 三、Runner 函数级规约

### `discover::find_examples(examples_dir: &Path) -> Result<Vec<Example>>`

```
前置条件：
  - examples_dir 是合法目录路径

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

后置条件：
  - 返回 Vec<Example>，每条含 (feature, dir, root, target_path, crate_name, entries, schema_kind)
  - schema_kind ∈ { SingleFile, Directory }，对应双轨之一
  - 目录轨 example 的 .hirusttest/ 路径可由 runner 在 exec 阶段访问（用于 stub 注入等）
  - target_path 已 canonicalize；若不在 root 子目录下，返回 Err
  - crate_name 取自 target_path/Cargo.toml 的 [package].name，且非空（否则 Err）
  - 结果按 (feature, dir) 排序

副作用：仅读 hirusttest.toml 或 .hirusttest/config.toml + Cargo.toml
```

### `discover::find_tools(tools_dir: &Path) -> Result<Vec<Tool>>`

```
前置条件：
  - tools_dir 是合法目录路径

后置条件：
  - 返回 Vec<Tool>，每条含 (name, command, timeout_secs, harness_template, extra_cargo_deps, entry_mode)
  - command 非空（否则 Err）
  - entry_mode ∈ { Bin, Lib }；缺省字段走 #[serde(default)]
  - 结果按 name 排序

副作用：仅读 tool.toml + harness.rs.tera
```

### `exec::execute(tool: &Tool, example: &Example, entry: &str, run_dir: &Path) -> Result<ExecResult>`

```
前置条件：
  - tool / example 来自 discover，entry ∈ example.entries
  - run_dir 已 canonicalize、存在、可写

后置条件（成功路径，含 subprocess FAILED 与 timeout）：
  - run_dir/raw/<tool>/<slug>.{stdout, stderr, exit} 三文件已写：
      stdout、stderr 为子进程原始输出
      exit 文件内容为 "Some(N)\n" 或 "__ts_timeout (after N s)\n"
  - 返回 ExecResult { status, exit_code, duration_ms, timed_out, raw_stdout_rel, raw_stderr_rel }
      status = Success ⟺ subprocess exit 0 ∧ ¬timed_out
      timed_out = true ⟹ status = Failed

后置条件（失败路径——runner-internal 错）：
  - 返回 Err(anyhow::Error)，不保证 raw 三文件已写
  - work_dir 可能残留（cleanup best-effort）
  - 由调用方 main 转为 TaskResult { status: "UNKNOWN", error: Some(e) }

不变式：
  - exec_id 唯一（参见 §二），work_dir 与其他任务路径互不重叠
  - 子进程在自己的 process group 中

副作用：cp 隔离副本、patch Cargo.toml（按 extra_cargo_deps）、写 harness 文件、起子进程、写 raw 三文件、删 work_dir
```

### `exec::patch_cargo_deps(cargo_path: &Path, dep_lines: &[String]) -> Result<()>`

```
前置条件：
  - cargo_path 指向合法 TOML 文件

后置条件：
  - cargo_path 文件中 [dependencies] 表含 dep_lines 中每条声明（同名 key 覆盖）
  - 若原本无 [dependencies]，新建之
  - 若 [dependencies] 非 table 形态（例如 inline table 罕见写法），返回 Err
  - 文件其他段保持原样

实现：通过 toml_edit::DocumentMut AST 编辑，避免字符串拼接的边角错误（注释、子表、CRLF、bom 等）

副作用：原地写 cargo_path
```

### `report::write_results_json(run_dir, run_id, results) -> Result<()>` / `write_report_md(...)`

```
前置条件：
  - run_dir 存在可写
  - 每条 results[i].entry_id 形如 "<feature>/<dir>/<entry-fn>"
  - status ∈ { "SUCCESS", "FAILED", "UNKNOWN" }

后置条件：
  - run_dir/results.json 含 { run_id, results: [...] } 的 pretty JSON
  - run_dir/report.md 含按 feature 分组的工具 × entry 矩阵；末尾汇总 succeeded / failed / unknown / total
  - 输出确定性：相同输入产生相同字节序列（BTreeMap 排序）

副作用：写两个文件
```

## 四、运行时机制

### 总体调度

```
1. 读 CLI 参数（--examples / --tools / --runs / --parallel）
2. find_examples + find_tools
3. 计算 tasks = { (tool, example, entry) | tool ∈ tools, example ∈ examples, entry ∈ example.entries }
4. 创建 run_dir = runs/run-<unix_secs>-<pid>/，canonicalize
5. rayon ThreadPoolBuilder::num_threads(parallel) 起 work-stealing pool
6. tasks.par_iter().map(|t| match exec::execute(...) { Ok → TaskResult Success/Failed; Err → TaskResult UNKNOWN }).collect()
7. write_results_json + write_report_md
8. 控制台汇总：succeeded / failed / unknown / total + run_dir + report.md path
```

### 单次执行单元

对一对 `(tool, example, entry)`：

```
1.  exec_id = sanitize(...)
2.  work_dir = run_dir/work/<exec_id>/   （若已存在则先 remove_dir_all）
3.  copy_dir_excluding(example.root, work_dir, &["target", "Cargo.lock"])
3b. 若 tool.extra_cargo_deps 非空：
        patch_cargo_deps(work_dir/<target_path>/Cargo.toml, deps)
3c. 若 example.schema_kind == Directory（目录轨）：
        // 渐进实施——v1 仅识别字段，stub 注入由后续版本支持
        从 .hirusttest/config.toml 读 entry_overrides / tools 节
        若有 per-entry verifier stub（<entry-fn>-spec.rs）则注入到 work_dir/<target_path>/src/
        若有 per-tool harness 模板覆写（<tool-name>-harness.rs.tera）则覆盖步骤 5 的默认模板
4.  cd work_dir/<target_path>/
5.  渲染 harness.rs.tera 用 (target_crate_name, entry_fn)
5a. 按 tool.entry_mode 落盘：
        Bin:  写到 src/bin/__ts_harness.rs（原 lib 不动）
        Lib:  rename src/lib.rs → src/__ts_inner.rs；写 harness 到 src/lib.rs
6.  Command::new(tool.command[0]).args(...).cwd(work_dir/<target_path>)
       .stdout(Piped).stderr(Piped)
       .process_group(0)         ← Unix 独立 process group
       .spawn()
6a. 双线程 read_to_end(child_stdout) / read_to_end(child_stderr)
6b. child.wait_timeout(Duration::from_secs(tool.timeout_secs))
       Some(status) → timed_out = false
       None         → kill(-pgid, SIGKILL); child.wait(); timed_out = true
6c. join 双 reader thread → stdout_buf / stderr_buf
7.  写 run_dir/raw/<tool>/<slug>.{stdout, stderr, exit}
        exit 内容: timed_out ? "__ts_timeout (after N s)" : "{:?}".format(exit.code())
8.  remove_dir_all(work_dir) — best-effort，失败仅 log warning（参见 §异常）
```

### 隔离机制

cp 而非 git worktree：worktree 拿 HEAD 要求每次新增样例都 commit；cp 拿当前文件，开发友好。两者契约等价（隔离副本），通用性论证不依赖具体复制机制。`copy_dir_excluding` 跳过 `target/`（避免 build 缓存污染）和 `Cargo.lock`（避免旧 cargo 不识别 v4 锁文件，且本项目是 feature 覆盖筛选不需要跨 run 锁定 dep）。

### 并发

rayon `par_iter().collect()`，默认并发度 = `num_cpus`，CLI `--parallel N` 可覆盖。每 task 独立 work_dir → 完全无序可并发。cargo registry 的 flock 由 cargo 自身处理。

### 超时

`tool.timeout_secs` 默认 300，超时直接 SIGKILL（v1 不做 SIGTERM 软退让——按"性能不算问题"精神，graceful shutdown 不属功能必需）。子进程 spawn 时设独立 process group，超时时 `kill(-pgid, SIGKILL)` 杀整个 group，孙子进程一并（cargo-kani → cbmc、cargo-creusot → why3 / alt-ergo / cvc 等）。这是必须的——若只 kill 直接 child，孙子持有 stdout/stderr fd，runner 的 reader thread 在 read_to_end 永不收 EOF → join 阻塞 → runner thread 卡死。超时分类为 FAILED，raw exit 文件写 `__ts_timeout (after N s)`。超时是 per-(tool, entry) 的——一个 entry 触发不牵连其他。

### 异常

子进程被信号杀（SIGSEGV / SIGABRT 等）→ 非零 exit → FAILED。runner 自身 IO / 渲染 / spawn 失败 → 该任务标 UNKNOWN（不归工具责任），整 run 继续，错误信息记入 `results.json` 的 `error` 字段。cleanup work_dir 失败 → 仅 log warning，不归类——cleanup 失败不影响已捕获 raw outputs；残留目录在整次 run 结束时随 `runs/<ts>/` 整体可删。

### `runner report <run-dir>` 子命令

从已有 `results.json` 重生成 `report.md`，不重跑任何工具。用于：

- 调整 report.md layout / 头部 metadata schema 后回填老 run
- 第三方工具消费 `results.json` 后再 emit 自定义报告

实现：`main.rs` 用 clap subcommand `report`，读 `<run-dir>/results.json` 反序列化为 `ResultsFile { meta, results[] }`，调 `report::write_report_md`。`results.json` 是 single source of truth，runner 仅生成。

## 五、已集成工具配置示例

19 个工具的完整 `tool.toml` / `harness.rs.tera` 配置 + 安装步骤详见各工具自己的 `tools/<name>/README.md`。下面列每个工具的关键 quirk（schema 怎么用、为什么这么用）。完整 schema 见 §一。

### cargo-check（baseline）

```toml
# tools/cargo-check/tool.toml
command = ["cargo", "check", "--bin", "__ts_harness"]
```

```rust
// tools/cargo-check/harness.rs.tera
fn main() {
    {{ target_crate_name }}::{{ entry_fn }}();
}
```

### Kani

`--only-codegen` 让 kani 跑 MIR → GotoC codegen + 类型 / 模型检查，**不 invoke cbmc 求解**。本项目是前端支持性测试（见 architecture "前端支持性观察原则"），让 kani 跑完整 SAT 求解会让对比失公（其他工具都在前端层停下）。timeout 砍到 120s，仅作 codegen 内部失控的兜底。

类似处理（同前端原则）：

- `tools/prusti/tool.toml`：env `PRUSTI_NO_VERIFY=false` + `PRUSTI_DUMP_VIPER_PROGRAM=true` + `PRUSTI_PRINT_HASH=true`（让 encoder 真跑、dump Viper program 后提前返回，不调 Silicon SMT）
- `tools/verus/tool.toml`：argv `--no-verify`（跳过 Z3 求解）
- `tools/verifast/tool.toml`：`-skip_specless_fns`（无 spec fn 全跳过）
- `tools/charon-{poly,mono}/tool.toml`：`--abort-on-error`（翻译报错暴露）
- `tools/creusot/tool.toml`：仅 `cargo creusot`（不带 `prove`，默认仅翻译到 Coma）

```toml
# tools/kani/tool.toml
command      = ["cargo", "kani", "--only-codegen", "--bin", "__ts_harness"]
timeout_secs = 120
```

```rust
// tools/kani/harness.rs.tera
#[kani::proof]
fn ts_proof() {
    {{ target_crate_name }}::{{ entry_fn }}();
}
fn main() {}
```

### MIRI

```toml
# tools/miri/tool.toml
command = ["cargo", "+nightly", "miri", "run", "--bin", "__ts_harness"]
```

```rust
// tools/miri/harness.rs.tera
fn main() {
    {{ target_crate_name }}::{{ entry_fn }}();
}
```

### Charon (poly / mono)

macOS arm64 上需 `--lib --target aarch64-apple-darwin` 绕 charon 的 bin rlib 路径假设。`--abort-on-error` 必加——charon 默认遇内部 panic 仍 exit 0，会把"无法翻译"误报为 SUCCESS（实测 `Box<dyn Any>` vtable drop preshim 在 mono 模式触发 panic）。

```toml
# tools/charon-poly/tool.toml
command      = ["/tmp/ts-tools-install/charon/bin/charon", "cargo", "--abort-on-error", "--", "--lib", "--target", "aarch64-apple-darwin"]
timeout_secs = 600
```

```toml
# tools/charon-mono/tool.toml （加 --monomorphize）
command      = ["/tmp/ts-tools-install/charon/bin/charon", "cargo", "--monomorphize", "--abort-on-error", "--", "--lib", "--target", "aarch64-apple-darwin"]
timeout_secs = 600
```

```rust
// tools/charon-{poly,mono}/harness.rs.tera —— harness 文件存在但被 --lib 跳过编译
fn main() {
    {{ target_crate_name }}::{{ entry_fn }}();
}
```

### Prusti

macOS arm64 上经 Rosetta 跑 x86_64 prusti-driver，多个环境变量缺一不可（详见 [`fixes/`](../fixes/) 与 [`tools/prusti/README.md`](../../tools/prusti/README.md) 的 Prusti 安装步骤）。

**关键三 env**（实现 encoder 真跑 + Silicon 不调）：
- `PRUSTI_NO_VERIFY=false` — 进入 `verify(env, def_spec)` 路径，触发 `Encoder::process_encoding_queue`
- `PRUSTI_DUMP_VIPER_PROGRAM=true` — encoder 完成后 dump Viper program 到 `target/verify/log/viper_program/`
- `PRUSTI_PRINT_HASH=true` — `process_verification_request` 在 dump 之后、`new_viper_verifier()` 之前直接 `return Success`，**Silicon/Z3 永不启动**

```toml
# tools/prusti/tool.toml （示意；完整内容含 Rosetta 5 toolchain env 见实际 tool.toml）
command = [
  "env",
  "JAVA_HOME=${PRUSTI_JAVA_HOME}",
  "PATH=${PRUSTI_RUST_TOOLCHAIN_DIR}/bin:${PRUSTI_JAVA_HOME}/bin:/usr/bin:/bin",
  "CARGO=${PRUSTI_RUST_TOOLCHAIN_DIR}/bin/cargo",
  "RUSTC=${PRUSTI_RUSTC}",
  "RUSTUP_TOOLCHAIN=nightly-2023-08-15-x86_64-apple-darwin",
  "RUST_SYSROOT=${PRUSTI_RUST_TOOLCHAIN_DIR}",
  "PRUSTI_QUIET=false",
  "PRUSTI_NO_VERIFY=false",
  "PRUSTI_DUMP_VIPER_PROGRAM=true",
  "PRUSTI_PRINT_HASH=true",
  "arch", "-x86_64",
  "${CARGO_PRUSTI}"
]
timeout_secs = 900
```

```rust
// tools/prusti/harness.rs.tera —— prusti 默认对每个 fn 跑隐式 panic 检查，无需特殊标注
fn main() {
    {{ target_crate_name }}::{{ entry_fn }}();
}
```

### Creusot

cargo-creusot 把整个 cargo project 喂给 creusot-rustc，要求**样例 lib 自己** `use creusot_std`。光在 harness（bin target）`use` 不够。用 `entry_mode = "lib"`：harness 取代副本 lib，原 lib 内嵌为 `mod __ts_inner`。

```toml
# tools/creusot/tool.toml
command          = ["/Users/<user>/.cargo/bin/cargo-creusot"]
timeout_secs     = 900
extra_cargo_deps = ['creusot-std = "0.11.0"']
entry_mode       = "lib"
```

```rust
// tools/creusot/harness.rs.tera —— 此文件取代 src/lib.rs
extern crate creusot_std;
use creusot_std::prelude::*;

mod __ts_inner;
pub use __ts_inner::*;

#[allow(dead_code)]
fn __ts_invoke() {
    __ts_inner::{{ entry_fn }}();
}
```

## 六、输出字段

### `runs/run-<id>/results.json`

```json
{
  "run_id": "run-1778126061-57374",
  "run_started_at": "2026-05-07T03:54:21Z",
  "run_finished_at": "2026-05-07T04:02:11Z",
  "run_started_unix_secs": 1778126061,
  "run_finished_unix_secs": 1778126531,
  "host": {
    "hostname": "ssyramdeMacBook-Air.local",
    "os": "macos",
    "arch": "aarch64",
    "kernel": "25.4.0",
    "cpu_brand": "Apple M5",
    "total_mem_mb": 24576,
    "num_cpus": 10
  },
  "parallelism": 10,
  "tools": [
    {
      "name": "kani",
      "command": ["cargo", "kani", "--bin", "__ts_harness"],
      "timeout_secs": 600,
      "entry_mode": "bin",
      "extra_cargo_deps": [],
      "version": "cargo-kani 0.67.0"
    }
  ],
  "results": [
    {
      "entry_id": "vec/basic-push-pop/push_pop_seq",
      "tool": "kani",
      "status": "SUCCESS",
      "exit_code": 0,
      "duration_ms": 1636,
      "raw_stdout": "raw/kani/vec__basic-push-pop__push_pop_seq.stdout",
      "raw_stderr": "raw/kani/vec__basic-push-pop__push_pop_seq.stderr"
    },
    {
      "entry_id": "concurrency/thread-mutex/thread_mutex_join",
      "tool": "kani",
      "status": "FAILED",
      "exit_code": null,
      "duration_ms": 600045,
      "timed_out": true,
      "raw_stdout": "raw/kani/concurrency__thread-mutex__thread_mutex_join.stdout",
      "raw_stderr": "raw/kani/concurrency__thread-mutex__thread_mutex_join.stderr"
    },
    {
      "entry_id": "<...>/<...>",
      "tool": "<some-tool>",
      "status": "UNKNOWN",
      "exit_code": null,
      "duration_ms": 0,
      "raw_stdout": null,
      "raw_stderr": null,
      "error": "patching ...: parsing Cargo.toml as TOML: ..."
    }
  ]
}
```

`timed_out` 字段省略时为 false（serde `skip_serializing_if = "is_false"`）。`error` 字段仅 UNKNOWN 任务存在。

### `runs/run-<id>/report.md`

按 feature 分组的工具 × entry 矩阵：

```
# Run run-1778071324-9549

## vec
| entry              | cargo-check | kani    | miri    | ... |
|--------------------|-------------|---------|---------|-----|
| basic-push-pop/push_pop_seq | SUCCESS | SUCCESS | SUCCESS | ... |

## concurrency
| entry                       | kani               | miri    | prusti       | ... |
|-----------------------------|---------------------|---------|--------------|-----|
| thread-mutex/thread_mutex_join | FAILED (timeout) | SUCCESS | FAILED (101) | ... |

---
Total: 260 succeeded / 41 failed / 0 unknown / 301 total
```

### `runs/run-<id>/raw/<tool>/<slug>.{stdout, stderr, exit}`

- `stdout`、`stderr`：subprocess 原始字节流
- `exit`：单行文本，`Some(N)\n` / `None\n` / `__ts_timeout (after N s)\n`

## 七、错误处理策略

| 错误类型 | 处理 |
|---|---|
| Schema 解析失败（Cargo.toml / hirusttest.toml / tool.toml） | runner 启动时 panic（discover 阶段错——配置 bug，不该静默） |
| 单 task 内 IO / cp / 渲染 / spawn 失败 | 任务标 UNKNOWN，整 run 继续 |
| 子进程退出非零 | 任务标 FAILED + exit_code |
| 子进程被信号杀（SIGSEGV 等） | 任务标 FAILED + exit_code = None |
| 子进程超时 | runner 发 `kill(-pgid, SIGKILL)`，任务标 FAILED + timed_out = true |
| cleanup work_dir 失败 | 仅 log warning，不影响任务结果 |
