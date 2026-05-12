# Counter-Challenge：audit-5（runner-implementation-review）验证

> 任务：用 disprove-first 方法验证 audit-5 中的 5 个高严重度问题 + 3 处显著 design / 实施冲突。
>
> 方法：对每条 audit challenge，**默认 audit 错**——独立重读 `runner/src/*.rs` 实际代码、独立重读 `docs/design/*.md` 的相关条款，再判定 audit 推论是否成立。必要时跑实测验证。
>
> 输出判定四档：**driver 成立 / 部分成立 / 推论错 / audit 推论与设计无关**。

---

## 一、判定结论速览

| 编号 | 严重度 | audit 主张 | 判定 |
|---:|:---:|---|:---:|
| #9 | 高 | `expand_env` 缺失环境变量静默展空 | **成立** |
| #6 | 高 | `target_dir` 不在 root 时 discover 未早 Err | **成立**（design 明示） |
| #35 | 高 | `TS_*` envvar 契约 design 层文档化不足 | **部分成立**（已散落于 `.env.example` 前言；design `detailed-design.md` 未集中） |
| #16 | 高 | reader thread 无 timeout 兜底（孙子 setpgid 逃 group） | **部分成立 / 推论非工程必要** |
| #40 | 高 | sanitize 碰撞理论存在 | **部分成立 / 理论碰撞 + design 已自认** |
| #6（冲突） | — | target_path 不在 root 时未 discover Err | **成立**（detailed-design §三 + architecture §四 双重明示） |
| #11（冲突） | 中 | `extra_cargo_deps` toml 解析错被推迟到 exec 阶段 | **成立**（design §七 错误处理表明示） |
| #7（冲突） | 中 | `entries = []` 解析成功但 0-task | **成立**（design §一 明示"必填，零参 pub fn 名列表"） |

**整体**：audit-5 在 8 个被检条目中，5 条完整成立、2 条部分成立、1 条部分成立。**无一条 audit 推论错**。这是目前所有审计 batch 里命中率最高的一组——所有 challenge 都有 design 明文或 invariant 依据支撑，没有"过度要求"的 challenge。

---

## 二、逐条 disprove-first 验证

### #9（高）— `expand_env` 缺失环境变量静默展空 ✅ **成立**

**audit 主张**：用户漏 `source .env` → command argv 含空元素 → spawn 报 ENOENT，错误信息不直观。

**独立证据 1（代码层）**：`runner/src/discover.rs:294-335`（`expand_env` 函数）：

```rust
let valid = !name.is_empty()
    && name
        .bytes()
        .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit() || b == b'_');
if closed && valid {
    out.push_str(&std::env::var(&name).unwrap_or_default());   // ← line 323
} else { ... }
```

`std::env::var(&name).unwrap_or_default()`：若 env 缺失，返回空串。**无任何 `eprintln!` / `tracing::warn!` 输出**。`grep -n "warn" ${TS_PROJECT_ROOT}/runner/src/discover.rs` 整文件 0 命中（除注释外），确认 expand_env 内 0 警告。

**独立证据 2（实测）**：构造测试场景，unset `TS_CHARON_BIN` 后跑 charon-poly：

```
$ unset TS_CHARON_BIN
$ runner --tools tools --tool charon-poly ...
[UNKNOWN] charon-poly test/empty/test_fn : spawning
  ["", "cargo", "--abort-on-error", "--print-llbc", "--", "--lib", ...]
  in /tmp/.../work/charon-poly__test__empty__test_fn/:
  No such file or directory (os error 2)
```

**实测确认**：argv 首元素为空串 `""`，`Command::new("")` 触发 `os error 2`。用户从 `[UNKNOWN] charon-poly ...` 看到的是 spawn 失败 + 完整 argv dump，**理论上可推断**根因是 `TS_CHARON_BIN` 漏 source；但**实践上**新 contributor 不会立即想到 expand_env 这一层。

**design 层证据**：`.env.example` line 7-15 提到"runner 在 spawn 子进程前会 strip 掉所有 TS_ 开头的 envvar"，但**没说"如果缺失则展为空串"**。`docs/design/detailed-design.md` §五 列出多个工具用 `${VAR}` 形式，但同样**未规定缺失变量的展开语义**。这是 design 层规约缺失。

**判定**：**audit 完整成立**。

- 行为侧：缺失变量静默展空，0 警告，确认存在
- 影响侧：spawn 失败信息可用，但用户体验不佳
- design 侧：行为未文档化

**对 audit 的批评 / 补充**：audit 建议「emit warning」是温和方案，但 audit 自己也提到「更激进可让 `find_tools` 直接 Err 出」——这两种方案各有取舍：
- warn 方案：好处是 `TS_*` 某些变量在某些工具下确实可有可无（如 `TS_AENEAS_BIN` 只对 aeneas-* 工具有意义），强 Err 出会让"我只想跑 cargo-check"的 user 不便
- err 方案：稳健但严格

audit 没强烈倾向哪一种——基本表态合理。

---

### #6（高）— `target_dir` 不在 `example.root` 子目录未早 Err ✅ **成立**

**audit 主张**：design `detailed-design.md` §三 明示「若 target_path 不在 root 子目录下，返回 Err」；实现仅在 exec 阶段 implicit 失败、归 UNKNOWN。

**独立证据 1（design 层）**：

`docs/design/detailed-design.md` line 177：
> `- target_path 已 canonicalize；若不在 root 子目录下，返回 Err`

`docs/design/architecture.md` line 141-143：
> `- 每个返回的 Example 含 (feature, dir, root, target_path, crate_name, entries, schema_kind)，`
> `  target_path 已 canonicalize 且属于 root 子目录`

**两处 design 明示**——架构层与细化层同时承诺 discover 阶段早 Err。这不是 audit 过度解读。

`docs/design/principles.md` §七（间接溯源 detailed-design.md §七 line 574 错误处理表）：
> `Schema 解析失败（Cargo.toml / hirusttest.toml / tool.toml） | runner 启动时 panic（discover 阶段错——配置 bug，不该静默）`

虽然「target_path 越界」不是 严格 schema 解析失败（是 schema 合法但 semantic 不允许），但其精神归类（配置 bug，不该静默）一致。

**独立证据 2（代码层）**：`runner/src/discover.rs:174-181`：

```rust
let target_rel = ts.target_path.as_deref().unwrap_or(".");
let target_dir = dir.join(target_rel).canonicalize().with_context(|| {
    format!(
        "canonicalizing target_path '{}' under {}",
        target_rel,
        dir.display()
    )
})?;
```

**无 `starts_with(&dir)` 检查**——`canonicalize` 成功后直接 push 到 `Example`。

**独立证据 3（实测）**：构造 `target_path = "../../escape-target"`，目标 dir 在 example.root 外但确实存在：

```
$ runner --examples /tmp/.../escape-example --tool cargo-check
[UNKNOWN] cargo-check test/escape/test_fn : example target /tmp/.../escape-target is not under example root /tmp/.../test/escape: prefix not found
Total: 0 succeeded / 0 failed / 1 unknown / 1 total
```

**实测确认**：
1. discover 阶段不报错 — example 被注册
2. exec 阶段 `strip_prefix` 失败 — 单 task UNKNOWN
3. 错误归类为 runner-internal（UNKNOWN），不是 schema bug（应当是 discover-time fatal error）

这正是 design 不要求的行为。如果该 example 是用户配置的少数 examples 之一（如 corpus 中有 1 个错配 + 100 个正确），用户跑完后看到 1 UNKNOWN 才发现错配——design 期望「discover 阶段就 panic」让用户立刻发现。

**判定**：**audit 完整成立**。design 双重明示，实现没兑现，是真冲突。

**audit 建议合理**：在 line 181 之后加 `target_dir.starts_with(&dir).then_some(()).ok_or_else(|| anyhow!(...))?;` 即可修复。

---

### #35（高）— `TS_*` envvar 契约 design 层文档化不足 ⚠️ **部分成立**

**audit 主张**：runner strip `TS_*` 后只 inject `TS_ENTRY_FN` / `TS_TARGET_CRATE`——`detailed-design.md` §四 子进程 spawn 段未明示，只在 `.env.example` 前言、各工具 `tool.toml`、wrapper.sh 中分散提到。

**独立证据 1**：`docs/design/detailed-design.md` §四 line 259-322 整 §四（运行时机制）：

grep `TS_` 在 detailed-design.md：

```
$ grep -n "^TS_\|TS_\\*\|TS_ENTRY_FN\|TS_TARGET_CRATE" detailed-design.md
```

整文件 0 命中。**design `detailed-design.md` 完全没提 `TS_*` 命名约定**。

**独立证据 2**：`.env.example` line 7-15 明确：
```
# 命名约定：所有变量统一用 TS_ 前缀，避免与工具自身的环境变量命名空间冲突
# （比如 prusti 把 PRUSTI_* 全部解读为它的 config flag）。runner 在 spawn
# 子进程前会 strip 掉所有 TS_ 开头的 envvar，所以这些变量仅对 runner 自身
# 用于 tool.toml 的 ${VAR} 展开，不会污染工具 child process 的 env。
```

**`.env.example` 的确文档化了 strip 规则**。但**未提 `TS_ENTRY_FN` / `TS_TARGET_CRATE` re-inject**——这两个变量是 runner 自动注入给 wrapper.sh 用于 oracle 校验的，仅在 `runner/src/exec.rs:178-179` 代码 + 注释中有：

```rust
// These are set AFTER env_remove so they survive into the child env.
command_builder.env("TS_ENTRY_FN", entry);
command_builder.env("TS_TARGET_CRATE", &crate_ident);
```

`grep "TS_ENTRY_FN" docs/`——找到的只有：
- `docs/design/hax-lean-consistency-design-2026-05-11.md`（hax-lean 专属设计文档）
- 各 wrapper 注释（如 `tools/rocq-of-rust/rocq-of-rust-wrapper.sh`）

`detailed-design.md` 主文档完全没提 `TS_ENTRY_FN` / `TS_TARGET_CRATE`。

**判定**：**audit 部分成立**。

- 「strip TS_* 规则」**已**在 `.env.example` 文档化（audit 未提到这一点是 audit 的轻微疏忽）
- 「re-inject TS_ENTRY_FN / TS_TARGET_CRATE」**未**在主 design 文档化，仅在代码注释 + 单一 feature design（hax-lean-consistency）有提
- audit 推论「未来维护者读 design 不会立刻明白契约」**成立**——他们读 `detailed-design.md` 主文档不会看到这一节

**audit 略嫌严苛之处**：`.env.example` 在项目根目录，是用户首先接触的文件，文档化"够用"——但作为「design 层契约」（principles.md 宪法层之下、architecture 之下、detailed-design 应当覆盖）确实缺失。

**修复优先级**：低-中（不是工程问题，是 design 完整性问题）。建议 `detailed-design.md` §四「子进程 spawn」段加 1-2 段：「子进程 env 契约 — strip + re-inject」。

---

### #16（高）— reader thread 无 timeout 兜底（孙子 setpgid 逃 group） ⚠️ **部分成立 / 推论非工程必要**

**audit 主张**：若孙子进程在 fork 后又 setpgid 离开本 process group，`kill(-pgid, SIGKILL)` 不杀它，reader thread 永 block；无 timeout 兜底。

**独立证据 1（代码层）**：`runner/src/exec.rs:192-201`：

```rust
let stdout_reader = std::thread::spawn(move || -> Vec<u8> {
    let mut buf = Vec::new();
    let _ = child_stdout.read_to_end(&mut buf);
    buf
});
let stderr_reader = std::thread::spawn(move || -> Vec<u8> { ... });
```

`read_to_end` 是阻塞调用，**无 timeout / 无 select 兜底**。
`runner/src/exec.rs:223-224`：

```rust
let stdout_buf = stdout_reader.join().unwrap_or_default();
let stderr_buf = stderr_reader.join().unwrap_or_default();
```

`join()` 同样无 timeout——若 reader thread 卡住，**整个 runner thread 卡死**。

**独立证据 2（process_group 全检）**：grep `setpgid|process_group` 全 codebase：

```
runner/src/exec.rs:143:    // is started in its own process group (setpgid via process_group(0)) so
runner/src/exec.rs:183:        command_builder.process_group(0);
```

整 codebase 只有 runner 自己用 `process_group(0)`，**无任何 wrapper / tool.toml 调用 setpgid**——audit 提到的「孙子 setpgid 逃 group」是**纯理论威胁**。Rust 主流验证工具（cargo-kani、cargo-creusot、cargo-prusti、cargo-verus、charon、aeneas、hax-engine 等）都通过 cargo 启动，cargo 自身不 setpgid，被驱动的二进制 cbmc / z3 / why3 / silicon 等也不 setpgid（这些是计算密集型 solver，没有 daemonize 需求）。

**独立证据 3（design 层）**：

`detailed-design.md` line 318：
> `子进程 spawn 时设独立 process group，超时时 kill(-pgid, SIGKILL) 杀整个 group，孙子进程一并（cargo-kani → cbmc、cargo-creusot → why3 / alt-ergo / cvc 等）。这是必须的——若只 kill 直接 child，孙子持有 stdout/stderr fd，runner 的 reader thread 在 read_to_end 永不收 EOF → join 阻塞 → runner thread 卡死。`

**design 明确意识到 reader 卡死风险**，给出的解决方案是「group kill」。design 隐含了「假设：孙子不会逃 group」——这是合理工程假设。

**判定**：**audit 部分成立**——
- 「reader thread 无 timeout」是**事实**
- 「孙子 setpgid 逃 group → reader 卡死」是**理论威胁**，未在现实工具上发生
- design 隐含「孙子不逃 group」假设，**未在 design 文档明示这条假设**——这是 audit 真正的 design 层 finding

**audit 的工程判断中肯**——audit 自己在「建议」段说「保留现状；若发现某工具确实有逃 group 行为，加 recv_timeout 兜底」。这是合理的 YAGNI 表态。

**对 audit 的批评**：audit 把此条标「高」严重度——按"实际触发概率 × 工程成本"算应该是「中」（理论威胁，现实零触发，且修复需重写 reader 逻辑）。但 audit 同时表态「保留现状」——说明 audit 自己也认可此条非工程必要。严重度标签与建议表态略不一致。

---

### #40（高）— sanitize 碰撞理论存在 ⚠️ **部分成立 / design 已自认**

**audit 主张**：`exec_id = sanitize(tool)__sanitize(feature)__sanitize(dir)__sanitize(entry)`，sanitize 把 `/ \ : ` 全替 `_`；分隔符是 `__`。若 corpus 含 `feature = vec, dir = basic-pop` 和 `feature = vec_basic-pop, dir = ...`，sanitize 后碰撞。

**独立证据 1（代码层）**：`runner/src/exec.rs:339-346`：

```rust
fn sanitize(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | ' ' => '_',
            c => c,
        })
        .collect()
}
```

`exec.rs:41-47`：
```rust
let exec_id = format!(
    "{}__{}__{}__{}",
    sanitize(&tool.name),
    sanitize(&example.feature),
    sanitize(&example.dir),
    sanitize(entry),
);
```

**碰撞构造**：假设两个 example：
- E1: `feature = "vec"`, `dir = "basic_pop"`（注意原始就含 `_`），`entry = "foo"`
- E2: `feature = "vec_basic"`, `dir = "pop"`, `entry = "foo"`

E1 的 exec_id = `T__vec__basic_pop__foo`
E2 的 exec_id = `T__vec_basic__pop__foo`

**字符串不同**——因为 `__` 是「`_` 重复 2 次」，单 `_` 不会被识为分隔符。重新构造碰撞：

- E1: `feature = "vec"`, `dir = "basic__pop"`（dir 内含双下划线）, `entry = "foo"`
- E2: `feature = "vec__basic"`, `dir = "pop"`, `entry = "foo"`

E1 = `T__vec__basic__pop__foo`
E2 = `T__vec__basic__pop__foo`

**碰撞！** —— 当 dir 或 feature 自身含 `__` 时，可碰撞。

但**实际能发生吗**？feature / dir 来自 `examples/<feature>/<dir>/` 文件系统路径——文件系统名理论允许 `__`（如 `examples/foo__bar/baz/`），cargo crate 名也允许。

`grep -rn "__" examples/`（搜索 examples 内含 `__` 的目录名）：

```
$ find examples/ -type d -name "*__*" 2>/dev/null
（无命中）
```

当前 142 个 hirusttest.toml 中无任何 feature / dir 含 `__`。但 corpus 扩展时 audit 提到的风险存在。

**独立证据 2（design 层）**：`docs/design/detailed-design.md` §二 line 152：

> `由于 cargo crate 名 / Rust ident / 文件系统路径都不含这四种字符，sanitize 在实践中不发生碰撞。`

**design 自承「实践不发生碰撞」**——但这里有两层混淆：
1. cargo crate 名 / Rust ident 不含 `/ \ : ` 四种 sanitize 替换字符 — design 这一点对
2. 但 **不含 `__`** 这一点 design 没明示——文件系统 / cargo 都允许 `__`

audit 抓住了第二点：实际碰撞触发条件是「dir 或 feature 含 `__`」，与 design 论证的「不含 `/ \ : `」不同条件，因此 design 的「实践不碰撞」结论**不严格**。

**判定**：**audit 部分成立**——
- 碰撞理论存在：确凿（构造例子可触发）
- design 已声明「实践不发生」：是事实，但 design 论证逻辑不严密
- 当前 142 个 example 无 `__` 名：是事实
- 未来 corpus 扩展风险：存在

**audit 建议合理**：「换无歧义 separator」（如 `\x1f` ASCII unit separator）或「discover 阶段断言唯一性」——后者更稳健，前者改 separator 影响文件系统可读性（运维 / 调试 raw 文件路径时麻烦）。中等优先级。

**对 audit 的判断**：严重度标「高」略激进——按当前 corpus 是「中」级别，仅未来扩展时升「高」。但 audit 列「决策点」让用户拍板——表态合理。

---

### #11（冲突，中）— `extra_cargo_deps` toml 解析推迟到 exec ✅ **成立**

**audit 主张**：design `detailed-design.md` §七 错误表「Schema 解析失败 → runner 启动时 panic（discover 阶段错）」；实现把 toml 解析推迟到 exec 阶段。

**独立证据 1（design）**：`detailed-design.md` line 574：
> `| Schema 解析失败（Cargo.toml / hirusttest.toml / tool.toml） | runner 启动时 panic（discover 阶段错——配置 bug，不该静默） |`

`extra_cargo_deps` 是 `tool.toml` 内的一个字段（line 119）：
> `extra_cargo_deps = ['creusot-std = "0.11.0"']`

字段值是「一行 TOML 风格的 dependency 声明」——这本身就是 schema 的一部分。

**独立证据 2（代码）**：`runner/src/discover.rs:277-282`：
```rust
#[derive(Deserialize)]
struct ToolToml {
    command: Vec<String>,
    #[serde(default = "default_timeout")]
    timeout_secs: u64,
    #[serde(default)]
    extra_cargo_deps: Vec<String>,    // ← 只解析为 Vec<String>
    ...
}
```

`find_tools`（line 384-389）直接 push `extra_cargo_deps: parsed.extra_cargo_deps`——**未对每条 string 做 `toml_edit::DocumentMut` 解析**。

`runner/src/exec.rs:324-327`（patch_cargo_deps 内）：
```rust
for line in dep_lines {
    let mini: toml_edit::DocumentMut = line
        .parse()
        .with_context(|| format!("parsing extra_cargo_deps entry `{}` as TOML", line))?;
    ...
}
```

**toml 解析在 exec 阶段**——失败时通过 `?` 回传到 main，归 UNKNOWN。

**影响场景**：用户 `tools/creusot/tool.toml` 写 `extra_cargo_deps = ["not valid toml ;;;"]`：
1. discover 阶段：成功（serde 只验 Vec<String> 类型，不验内容）
2. runner 跑全部 144 tasks
3. **每条 creusot tasks 都 UNKNOWN**，错误信息一致

vs design 期望：runner 启动时直接 panic，user 立刻发现 typo。

**判定**：**audit 完整成立**。

**修复优先级**：中。修复方式简单——在 `find_tools` 加一行 toml dry-parse。

---

### #7（冲突，中）— `entries = []` 解析成功但 0-task ✅ **成立**

**audit 主张**：`HirusttestToml.entries: Vec<String>` 接受空 Vec；example 被注册但跑 0 task；design 「必填，零参 pub fn 名列表」未强制。

**独立证据 1（design）**：`detailed-design.md` line 20：
> `entries = ["fn_name_1", "fn_name_2"]   # 必填，零参 pub fn 名列表`

「必填」一词——semantic 上应当解读为「非空列表」（仅声明字段存在但内容为空，违背"必填"精神）。但 design 没明示「必须非空」。

**独立证据 2（代码）**：`runner/src/discover.rs:51-55`：
```rust
#[derive(Deserialize)]
struct HirusttestToml {
    entries: Vec<String>,          // ← Vec 可空
    target_path: Option<String>,
}
```

serde 默认接受 `entries = []`。

**独立证据 3（实测）**：构造 `hirusttest.toml` 含 `entries = []`：

```
$ runner --examples /tmp/.../examples --tool cargo-check ...
No tasks match the given filters (--tool / --entry).
```

**实测确认**：
1. discover 解析成功（无 Err）
2. example 注册，但 `for entry in &ex.entries` (main.rs:120) 空循环
3. tasks 列表空 → main.rs:138 命中 `if tasks.is_empty() { eprintln!("No tasks match the given filters") }`
4. **错误信息误导**——实际不是 filter 不匹配，而是 entries 字段为空

如果 user 的 `entries = []` 是因「我没决定填啥就先占位」，static 配置错；如果是因「我以为空数组 = 跳过这个 example」，semantic 错。两种 case runner 都该提示。

**判定**：**audit 完整成立**。

**修复优先级**：中。修复方式：在 `find_examples` 注册 example 前检查 `ts.entries.is_empty()` → 报 Err 或至少 `eprintln!("[warn] ...")`。

---

## 三、design 是否真有"要求 discover Err"的明示？（核心问题）

**结论**：**是**。两条核心 audit 引用的"design 要求"都有明示，**不是 audit 过度解读**：

1. **#6（target_path 不在 root）**：
   - `detailed-design.md` §三 line 177「target_path 已 canonicalize；若不在 root 子目录下，返回 Err」
   - `architecture.md` §四 line 142「target_path 已 canonicalize 且属于 root 子目录」
   - 两处独立明示，无歧义

2. **#11（extra_cargo_deps schema）**：
   - `detailed-design.md` §七 line 574 错误处理表「Schema 解析失败 → runner 启动时 panic（discover 阶段错）」
   - 「Schema 解析失败」涵盖 `tool.toml` 内字段值——`extra_cargo_deps` 字段值是 TOML 子片段，属 schema 一部分

3. **#7（entries = []）**：
   - `detailed-design.md` line 20 注释「必填」——semantic 强度略弱（仅"必须有此字段"，不必明示"非空"）
   - 这一条的 design 明示**不如 #6 / #11 强**，但精神延伸合理

**总评**：audit 这三条冲突是**真冲突**——design 在前后置 / 错误处理表中明示了 discover 阶段严格性，实现没兑现。修复成本低，每条加 5-15 行代码即可。

---

## 四、audit-5 整体质量评级

### 优点

1. **disprove-first 友好**：每条 audit 都有具体 line number + design 引用，可独立验证。没有「靠感觉判定」的条目。
2. **决策点 / 非决策点分类清晰**：14 决策点 + 26 非决策点列表（§4 表），让 user 一眼看到哪些条需要拍板。
3. **panic 抗性整体审查**（§5.3）确认 16 处 unwrap / expect 均在已建立 invariant 下安全——是有价值的 invariant 检查。
4. **派生原则 C「异质性归配置」整审查**（#37）——`grep` 全 codebase 确认无工具名硬编码——正向验证 design 兑现。

### 缺陷

1. **严重度标签略宽松**：#16（孙子逃 group）、#40（sanitize 碰撞）严重度「高」——按"实际触发概率 × 工程成本"算更适合「中」。但 audit 自己在 §4 决策点表里都列出，让 user 拍板，没有强推「必须修」——表态自洽。
2. **#35 文档化主张略严苛**：未注意 `.env.example` 已文档化 strip 规则——这是审计盲区。但「`TS_ENTRY_FN` / `TS_TARGET_CRATE` re-inject 规则没文档化」一面仍成立。
3. **个别决策点表态不强**：如 #8（broken var ref 静默）严重度「低」却列决策点——可以直接给「保留现状」建议而非要 user 拍板。

### 综合

audit-5 是这轮所有 audit batch 中**质量最稳的一份**。8 个被检条目中：
- **5 条完整成立**（#9 / #6 / #11 / #7 + #6 冲突重复）
- **3 条部分成立**（#35 / #16 / #40）
- **0 条 audit 推论错**

3 处 design / 实施冲突均有 design 明文支撑，非 audit 过度解读。修复成本低（每条加 5-15 行代码 + 1-2 段 design 补遗）。

**建议**：所有 8 条均纳入下一轮 fixes，其中 #6 / #11 / #7 三冲突优先修（design 明文兑现），#35 中优先级（design 补遗），#9 / #16 / #40 按 audit 建议分级处理。

---

## 五、附：实测脚本痕迹

实测在 `/tmp/audit5-test/` 构造，已在 `/tmp/audit5-test/runs/` 留下 raw 输出，确认：

1. `entries = []` → runner 退出 `No tasks match the given filters`（错误信息误导）
2. `TS_CHARON_BIN` unset → spawn `["", "cargo", ...]` 触发 `os error 2`，UNKNOWN（无 warn）
3. `target_path = "../../escape-target"` 越界 → discover 阶段无 Err，exec 阶段 strip_prefix 失败 UNKNOWN

均与 audit 主张一致。
