# Runner design-impl 冲突修复（P18-D2，2026-05-11）

> 修复 Counter-5（[`audit-2026-05-11/counter-challenge/audit-5-counter.md`](audit-2026-05-11/counter-challenge/audit-5-counter.md)）中验证的 3 处「design 明文 ↔ runner 实现」缺口。

## §1 问题起源

Counter-5 用 disprove-first 重读 `runner/src/*.rs` 与 `docs/design/*.md`，确认 audit-5 中三处所谓「design / impl 冲突」全部成立——design 都有明文支撑，实现没兑现：

| 编号 | design 明文 | 实现实际行为 |
|---:|---|---|
| #6 | `detailed-design.md` §三 line 177「target_path 已 canonicalize；若不在 root 子目录下，返回 Err」+ `architecture.md` §四 line 142「target_path 已 canonicalize 且属于 root 子目录」 | discover 阶段无 `starts_with` 检查；越界 target_path 推迟到 exec stage `strip_prefix` 报错，归 UNKNOWN |
| #11 | `detailed-design.md` §七 错误处理表「Schema 解析失败（Cargo.toml / hirusttest.toml / tool.toml） → runner 启动时 panic（discover 阶段错——配置 bug，不该静默）」 | `extra_cargo_deps` 中每条 mini-TOML 的语法错误推迟到 exec stage `patch_cargo_deps` 才暴露，归 per-task UNKNOWN |
| #7 | `detailed-design.md` §一 line 20「`entries = ["fn_name_1", ...]` 必填，零参 pub fn 名列表」+ line 78「`[runnable.<entry>]` 段不创建新 entry——它扩展已存在 entry」 | `entries = []` 解析成功，example 注册但 0 task，silent |

精神共通：三者都属「配置 bug 应在 discover 阶段早 Err」，但实现都让错误沉默渗透到 exec 或根本无错误信号。

## §2 修复方案 + diff

全部改动集中在 `runner/src/discover.rs`。`exec.rs` / `main.rs` 不动；`patch_cargo_deps` 在 exec.rs 中仍保留（仍负责实际 splice 到 Cargo.toml），只是被前置一道 discover 阶段验证拦截了坏 input。

### 2.1 #6 target_path 越界 → discover Err

`find_examples` 中，对每个 example 先 `canonicalize` 其 `dir`（example.root），再 `canonicalize` `target_path`，二者用 `starts_with` 比对：

```rust
let root_canon = dir.canonicalize().with_context(|| {
    format!("canonicalizing example root {}", dir.display())
})?;

let target_rel = ts.target_path.as_deref().unwrap_or(".");
let target_dir = dir.join(target_rel).canonicalize().with_context(|| { ... })?;

if !target_dir.starts_with(&root_canon) {
    return Err(anyhow!(
        "{}: target_path '{}' resolves to {} which is not under example root {}",
        config_path.display(), target_rel, target_dir.display(), root_canon.display(),
    ));
}
```

**为何先 canonicalize root**：原代码中 `dir` 来自 `WalkDir`，可能含未解析的 symlink；`target_dir` 已 canonicalize（绝对路径 + symlink 解析）。两者不在同一坐标系下 `starts_with` 会假阴。

### 2.2 #11 `extra_cargo_deps` 坏 TOML → discover Err

`find_tools` 中，对每条 `parsed.extra_cargo_deps` 立即 `parse::<toml_edit::DocumentMut>()` 验证：

```rust
for dep_line in &parsed.extra_cargo_deps {
    let parsed_dep: toml_edit::DocumentMut = dep_line
        .parse()
        .with_context(|| format!(
            "{}: extra_cargo_deps entry `{}` is not valid TOML",
            tool_toml_path.display(), dep_line,
        ))?;
    if parsed_dep.as_table().iter().next().is_none() {
        return Err(anyhow!(
            "{}: extra_cargo_deps entry `{}` declares no dependency key",
            tool_toml_path.display(), dep_line,
        ));
    }
}
```

**fail 方式**：返回 `anyhow::Err`，由 `find_tools` 向上 `?` 到 `main`，整 runner 以 non-zero exit。
design §七 字面写「panic」；但 anyhow::Err 与 panic 在用户视角等价（runner 启动失败 + 错误信息明显 + 非零 exit），且与 `find_tools` 现有错误风格（command empty 时也是 Err）一致——避免在同一函数里两种 fail 风格混杂。错误信息含 tool.toml 路径 + 原始 dep 字符串 + toml_edit 的精确解析定位（行列号 + caret），比 `panic!("{}", e)` 更易诊断。

**额外护栏**：除 parse 失败外，还检查「parse 成功但 0 个 top-level key」（空字符串 / 仅注释等）——`patch_cargo_deps` 会静默 no-op，同样属配置 bug。

### 2.3 #7 `entries = []` → discover Err

`find_examples` 中，HirusttestToml deserialize 后立即检查：

```rust
if ts.entries.is_empty() {
    return Err(anyhow!(
        "{}: `entries` must be non-empty (an example must register at least one pub fn name)",
        config_path.display()
    ));
}
```

**与 `[runnable.<fn>]` 的关系**：按 detailed-design.md §一 line 78 明示，`[runnable.<entry>]` 段「不创建新 entry——它扩展已存在 entry（必须先出现在 `entries = [...]` 列表）」。所以 `entries = []` + `[runnable.foo]` 是非法配置——`foo` 没在 `entries` 注册，runnable 段无所附着。这条 design 规则把判定简化为「`entries.is_empty()` 即非法」，不需要再看 runnable 段。

## §3 实测验证

每条修复都构造一个反例（应 Err）+ 整 corpus 跨 entry 跑现有正例（应不破）。

### 3.1 #6 反例

构造 `examples-bad-target/feat1/ex1/hirusttest.toml`：

```toml
entries = ["escape_fn"]
target_path = "../../../escape-target"
```

其中 `escape-target/` 位于 examples 目录之外但物理存在（确保 canonicalize 成功）。

```
$ ./target/release/runner --examples /tmp/p18-d2-test/examples-bad-target --tool cargo-check
Error: /private/tmp/p18-d2-test/examples-bad-target/feat1/ex1/hirusttest.toml: target_path '../../../escape-target' resolves to /private/tmp/p18-d2-test/escape-target which is not under example root /private/tmp/p18-d2-test/examples-bad-target/feat1/ex1
```

discover 阶段直接 Err，exit 非零，无 run_dir 生成。**修复前**：runner 跑完后报 1 UNKNOWN，run_dir 落盘。

### 3.2 #11 反例

构造 `tools-bad-deps/cargo-check-bad/tool.toml`：

```toml
command = ["cargo", "check"]
extra_cargo_deps = ["not valid toml syntax !!!"]
```

```
$ ./target/release/runner --examples ... --tools /tmp/p18-d2-test/tools-bad-deps
Error: /private/tmp/p18-d2-test/tools-bad-deps/cargo-check-bad/tool.toml: extra_cargo_deps entry `not valid toml syntax !!!` is not valid TOML

Caused by:
    TOML parse error at line 1, column 5
      |
    1 | not valid toml syntax !!!
      |     ^
    expected `.`, `=`
```

discover 阶段 Err。**修复前**：所有 (tool, example, entry) tasks 全部到 exec stage `patch_cargo_deps` 时 UNKNOWN。

### 3.3 #7 反例

构造 `examples-empty-entries/feat1/ex1/hirusttest.toml`：

```toml
entries = []
```

```
$ ./target/release/runner --examples /tmp/p18-d2-test/examples-empty-entries --tool cargo-check
Error: /private/tmp/p18-d2-test/examples-empty-entries/feat1/ex1/hirusttest.toml: `entries` must be non-empty (an example must register at least one pub fn name)
```

discover 阶段 Err。**修复前**：runner exit 0、`No tasks match the given filters` 警告（误导——其实是 0-entries）。

### 3.4 正例：现有 corpus 跨 entry 跑 cargo-check

```
$ ./target/release/runner --tool cargo-check --parallel 10
...
Total: 146 succeeded / 15 failed / 0 unknown / 161 total
```

15 个 FAILED 全部分布于 `runnable/*` 二级目录——按 detailed-design.md §一 line 46 明示：

> 这意味着所有 entry_mode = "bin" 且 harness 默认调 zero-arg 形态的工具（cargo-check / kani / miri / ...）在 runnable entry 上的 harness 编译会失败 = FAILED——这是已知代价。

与本修复**无关**。**0 unknown** 证实修复未引入新的 runner-internal 失败模式；142 个原 hirusttest.toml entries 全部仍然正常 discover + exec。

## §4 是否引入新风险

### 4.1 用户调试友好度

**反例**：用户写错 `target_path` 现在拿到 anyhow Err，整 runner 立刻失败。
**vs 修复前**：runner 完整跑，错配例子归 UNKNOWN，其他 examples 仍跑完。

后者看似「更鲁棒」，但实际上：
- 错配 example 已 0% 出有效信号
- 用户从 `results.json` 看到 `[UNKNOWN]` 时根本看不出是 target_path 越界（错误信息是 exec stage 的 `strip_prefix: prefix not found`，含义不直观）
- 一旦混在 142 个 entries 里，错配很容易被忽视

discover-time fail 把「错配」从 UNKNOWN 一类的"杂音"提升为"runner 启动门"——用户无法忽略它。这与 design §七「配置 bug，不该静默」精神一致，**降低**而非增加调试难度。

**唯一可能的回归**：若某个用户的 corpus 中本来就有「故意"故障" example」（比如开发中半 commit 的 example），修复前会在每次 run 出 UNKNOWN 不影响其他 entries；修复后整 runner 拒启动。

我评估这是**可接受的代价**：
- design 已明示「entries must 必填」「target_path must 在 root 子目录」——故意半 commit 本身就违反 schema 契约
- 项目有 `.no-hirusttest` 屏蔽标记机制（discover.rs:71）；半 commit 的 example 应当放 `.no-hirusttest` 屏蔽，而不是依赖 runner 容忍坏配置

### 4.2 与 D3 减法修宪的关系

D3 主进程刚改了 `principles.md` L168-172（删除"强制边界"条款 1+2）。这两条删除的条款描述的是 example 不能含 `#[cfg(<tool>)]` 等 fly-rule，与本修复涉及的 schema 验证完全独立。本修复涉及的 design 段（`detailed-design.md` §一/§三/§七、`architecture.md` §四）均未修订，依然是修复的 design 依据。**无冲突**。

### 4.3 与 D1 / D5 并行 agent 工作的隔离

本修复**只动 `runner/src/discover.rs`**：
- 不动 `runner/src/exec.rs` / `main.rs` / `report.rs` / `host.rs`
- 不动 `tools/` 任何文件
- 不动 `examples/` 任何文件
- 不动 `principles.md` / `architecture.md` / `tool-integration.md`

与 D1（工具集成）、D5（tools/）的并行工作无文件级冲突。

### 4.4 测试用例残留

`/tmp/p18-d2-test/` 是临时测试目录，不入 git，整目录可删；runner `runs/` 目录中本修复期间无新 run_dir 残留（反例都在 discover 阶段失败，没到生成 run_dir 这一步；正例 run-1778491585-91359 是常规 cargo-check run，与本修复无关）。

## §5 构建 + 全 corpus 验证

```
$ cargo build --release
   Compiling runner v0.1.0
    Finished `release` profile [optimized] target(s) in 2.10s
```

rustc 通过、无 warning。

`runner --tool cargo-check --parallel 10` 跨 161 entries：146 SUCCESS / 15 FAILED（全 runnable/*，design 明示） / 0 UNKNOWN。修复不破现有功能。
