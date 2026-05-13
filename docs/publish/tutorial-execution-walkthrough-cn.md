# Tutorial: runner 执行流水线 — 形状 + 法律 + 实测

> 这份文档**和 `tutorial.md` 互补**：tutorial.md 教"怎么用 runner 跑实验 + 读结果"，本文档教"runner 跑的时候每一阶段在生成什么形状的东西、这个形状保证了 `principles.md` 哪一条法律、怎么用 `prepare` / `--keep-work-dir` 亲眼看一遍"。
>
> 适合谁读：审稿人 / 复现者 / 想给项目加新工具的人 / 想验证 runner 没作弊的人。
>
> 阅读路径：从 §1 鸟瞰开始，再按 §2-§10 逐阶段读，最后 §11 给一张总览表 + §12 给一份"反作弊自检 6 条"。

---

## §0 准备：两扇透明性的窗

runner 默认会清理 work_dir，所以你看不到中间状态。两个新 flag 是这份文档的主要工具：

```sh
# 截到 spawn 之前 — 看 "工具看到的输入" 是什么形状
runner prepare <tool> <entry-id>

# 跑完全程但不清理 — 看 "工具看完之后留下了什么"
runner --keep-work-dir [--tool ...] [--entry ...]
```

`<entry-id>` 用 `<feature>/<dir>/<entry-fn>` 全形式。每个 prepare 输出最后一行打的 `cd <path>` 是直接进 work_dir 看实物的指针。

> **法律层根据**：宪法 §五 runner "结果记录严谨全面"——每次 run 必须记录裸数据自描述。`--keep-work-dir` 把这个"裸数据"扩到工具看见的 intermediate state，进一步把 runner 推向 reproducer-friendly。`prepare` 子命令是它的"截断"版，把 runner 拆成"可独立审计的步骤"。

---

## §1 鸟瞰：runner 流水线的 9 步

```
                ┌─────────────┐
   user input ─→│  1. discover│  scan examples/ + tools/ for signal files
                └─────┬───────┘
                      │  examples: Vec<Example> / tools: Vec<Tool>
                      ↓
                ┌─────────────┐
                │  2. filter  │  --tool / --entry → cartesian product
                └─────┬───────┘
                      │  tasks: Vec<(Tool, Example, entry)>
                      ↓
        ╔═════════════╧═════════════╗   ← parallel via rayon, per task:
        ║                           ║
   ┌────╨──────┐               ┌────╨──────┐
   │ 3. cp     │  example/ →   │ 3. cp     │
   │ + isolate │  runs/.../work│ + isolate │
   └────┬──────┘               └───────────┘
        │
        ↓
   ┌────────────┐
   │ 4. patch   │  inject [dependencies] from tool.toml.extra_cargo_deps
   │ Cargo.toml │   (现仅 Creusot 一例)
   └────┬───────┘
        ↓
   ┌────────────┐
   │ 5. render  │  Tera template (target_crate_name / entry_fn / entry_args)
   │ harness    │  → bin/__ts_harness.rs (Bin)  OR  lib.rs replace (Lib)
   └────┬───────┘                                ┐ ─────────────── ←─── prepare 终点
        ↓                                        │
   ┌────────────┐                                │
   │ 6. build   │  strip TS_* + inject TS_ENTRY_FN +
   │ spawn env  │  TS_TARGET_CRATE + [env]
   └────┬───────┘
        ↓
   ┌────────────┐
   │ 7. spawn   │  Command::new(...).process_group(0).spawn()
   │ + capture  │  → stdout/stderr threads + wait_timeout
   └────┬───────┘
        ↓
   ┌────────────┐
   │ 8. persist │  raw/<tool>/<slug>.{stdout,stderr,exit}
   │ + cleanup  │  remove work_dir   ← --keep-work-dir 把这一步关掉
   └────┬───────┘
        ↓
   ┌────────────┐
   │ 9. classify│  external_fault re-bucket → write results.json + report.md
   │ + report   │
   └────────────┘
```

下面分阶段铺开。每节四块：**做什么 / 形状 / 法律 / 自己验**。

---

## §2 阶段 1 — discover：scan signal files

### 做什么

walk `examples/` 找出每个目录下的 signal 文件，walk `tools/` 找出每个目录下的 `tool.toml + harness.rs.tera`。

example signal 文件两种形态（detailed-design §一）：

| signal | 含义 |
|---|---|
| `<dir>/hirusttest.toml` | single-file 轨 |
| `<dir>/.hirusttest/config.toml` | directory 轨（vendor/ 复合项目用） |

两个 signal 同时出现 → discover 报 `ambiguous schema at <dir>`，硬错。

### 形状

```
examples/
├── runnable/
│   └── add-u32/
│       ├── Cargo.toml          # [package].name = "runnable_add_u32"
│       ├── src/lib.rs          # pub fn add_u32(a: u32, b: u32) -> u32 { a + b }
│       └── hirusttest.toml     # entries = ["add_u32"]; [runnable.add_u32] inputs = [[0,0], ...]
├── hello/
│   └── basic-hello/
│       ├── Cargo.toml          # [package].name = "basic_hello"
│       ├── src/lib.rs          # pub fn hello() { ... }
│       └── hirusttest.toml     # entries = ["hello"]
└── industrial/
    └── x509-parser/cert-parse/
        ├── Cargo.toml
        ├── src/lib.rs
        └── hirusttest.toml     # entries + [env] RUSTFLAGS=--cap-lints=warn
tools/
├── cargo-check/
│   ├── tool.toml               # command + entry_mode + timeout_secs + extra_cargo_deps
│   └── harness.rs.tera
├── creusot/
│   └── tool.toml               # entry_mode = "lib" + extra_cargo_deps = ['creusot-std = "0.11.0"']
└── ...
```

discover 把这些扫成两个 `Vec<Example>` / `Vec<Tool>`，每条带：feature/dir/root/target_path/entries/entry_args/env (Example) 或 name/command/entry_mode/extra_cargo_deps (Tool)。

### 保证了什么 + 法律锚定

| 保证 | 怎么实现 | principles 锚 |
|---|---|---|
| 信号文件 ≠ corpus 内容 | hirusttest.toml 与 src/ + Cargo.toml 完全分离；缺信号文件 → cargo bytes 一字节不变 | §四 原则 A "双方都不可侵入"——信号文件加入前后 example cargo 行为字节级一致 |
| 双轨二选一不歧义 | 两 signal 同时出现 → `Err("ambiguous schema")` | §四 原则 A 的形式定义边界条件（detailed-design.md §一 边界 3） |
| 工具完全声明驱动 | runner 不知道任何工具名；所有差异由 tool.toml 表达 | §四 原则 C "异质性归配置，框架代码同质"；§六 当前 crate 焦点 |
| 公平：所有工具看到同一组 entries | discover 是工具/example 完全正交两次扫描，无任何 per-tool exclude | §一 根本问题 1 "不公平"；§四 原则 A 的对偶 |
| 公信：entries=[] 必须 fail | discover 见 `entries: []` → `Err("entries must be non-empty")` | §一 根本问题 2 "不公信"——不能静默 0 任务 |

### 自己验

```sh
# 看 discover 找出了多少 example
runner --tool cargo-check --entry 'nonexistent/**' 2>&1 | head -3
# stderr: "No tasks match the given filters" 说明 discover + filter 都跑了
# 输出 results 的 entry_id 列表应该是 161 行
```

或更直接，看 `results.json.tools[].name` 列表（20 个）+ `results.json.results[].entry_id` 去重后是 161 个——证明 discover 没漏。

---

## §3 阶段 2 — filter + cartesian：生成任务列表

### 做什么

```python
# 伪代码
for ex in examples:
    for entry in ex.entries:
        entry_id = f"{ex.feature}/{ex.dir}/{entry}"
        if not entry_glob_match(entry_id): continue
        for tool in tools:
            if --tool 指定且 tool.name 不在其中: continue
            tasks.append((tool, ex, entry))
```

### 形状

- 全跑（无 filter）：20 tools × 161 entries = 3220 task
- `--tool cargo-check --entry 'hello/basic-hello/*'`：1 tool × 1 entry = 1 task
- entry_id 永远是稳定的 `<feature>/<dir>/<entry-fn>` 三段——report.md / results.json / cc-reports 全用这套 id

### 保证了什么 + 法律锚定

| 保证 | 怎么实现 | principles 锚 |
|---|---|---|
| 工具间公平 | 笛卡尔积，无 per-tool exclude；同一 entry 一定喂给所有筛选后的 tool | §一 不公平问题；§四 原则 A 对偶 |
| entry_id 跨产物稳定 | discover 阶段就定形，filter 不变形 | §五 runner "结果记录严谨全面" |
| filter 不静默 | 0 task 时 `eprintln!("No tasks match")` 显式返回 0 | §一 不公信问题；detailed-design §七 "配置 bug 不该静默" |

### 自己验

```sh
runner --tool cargo-check --entry 'hello/basic-hello/hello' 2>&1 | grep '^Running'
# Running 1 task(s) with parallelism = N
# 笛卡尔积 1×1=1 — 没有 silent dropping
```

---

## §4 阶段 3 — cp + isolate：把 example 拷成可改的工作副本

### 做什么

```rust
exec_id  = sanitize(tool) + "__" + sanitize(feature) + "__" + sanitize(dir) + "__" + sanitize(entry)
work_dir = runs/<run-id>/work/<exec_id>/
cp -r example/<feature>/<dir>/   →  work_dir/   (skip: target/, Cargo.lock)
```

### 形状（实测，bin-mode + cargo-check × hello/basic-hello/hello）

```sh
$ runner prepare cargo-check hello/basic-hello/hello
prepared work_dir: <runs>/prepare-1778588004-12451/work/cargo-check__hello__basic-hello__hello

$ cd <work>
$ ls -a
.  ..  Cargo.toml  hirusttest.toml  src/
$ ls src/
lib.rs    # original lib.rs, 一字节没动
$ cat src/lib.rs
pub fn hello() {
    let _ = 1 + 1;
}
```

注意：

1. **`examples/hello/basic-hello/` 原目录此刻一字节没变**——所有改写都在 work_dir 副本上。
2. **`target/` 被排除**——避免 stale 编译缓存污染工具。
3. **`Cargo.lock` 被排除**——这是法律级选择：旧 cargo（如 Prusti 钉的 2023-08-15 nightly）解析不了新版 lockfile 格式，留下 stale lock 会让"feature 测量"退化成"lockfile 兼容性测量"。

### 保证了什么 + 法律锚定

| 保证 | 怎么实现 | principles 锚 |
|---|---|---|
| 原 example/ 永不动 | runner 改的是 cp 后的 work_dir，源目录只读 | §四 原则 A "双方都不可侵入"——signal file 不动 example 的对偶 |
| 排掉 target/ | copy_dir_excluding 的 hard-coded skip 列表 | §一 不公平（stale 缓存会让先跑的工具占便宜） |
| 排掉 Cargo.lock | 同上 | §一 不公信（lockfile 兼容性 ≠ feature 覆盖） |
| 每 task 独立 work_dir | exec_id = (tool, feature, dir, entry) 四元组 sanitize | 并发安全；§五 runner "裸数据自描述" |

### 自己验

```sh
# 跑 prepare 后，比对原 example/ 与 work_dir/
diff -r examples/hello/basic-hello/src/ <work>/src/
# 唯一差异：work_dir 多了 src/bin/__ts_harness.rs（来自下一阶段）
# 唯一差异以外没有任何修改 ← 这就是法律证据

diff examples/hello/basic-hello/Cargo.toml <work>/Cargo.toml
# (无差异，bin-mode 普通工具的 Cargo.toml 不动)
```

---

## §5 阶段 4 — patch Cargo.toml：注入 extra_cargo_deps

### 做什么

如果当前 tool.toml 有 `extra_cargo_deps`（仅 Creusot 一例），用 `toml_edit` 往 work_dir 的 `Cargo.toml` 的 `[dependencies]` 表里 splice 进去。

### 形状（实测，Creusot × hello/basic-hello/hello）

```sh
$ runner prepare creusot hello/basic-hello/hello
...
Files written by the runner (intrusive layer):
  ~ <work>/Cargo.toml  ([dependencies] patched with extra_cargo_deps:
        creusot-std = "0.11.0"
    )

$ diff examples/hello/basic-hello/Cargo.toml <work>/Cargo.toml
+ [dependencies]
+ creusot-std = "0.11.0"
```

`tools/creusot/tool.toml`：

```toml
command          = ["${TS_CARGO_CREUSOT}"]
entry_mode       = "lib"
extra_cargo_deps = ['creusot-std = "0.11.0"']
```

### 保证了什么 + 法律锚定

| 保证 | 怎么实现 | principles 锚 |
|---|---|---|
| 异质需求只能从数据层进 | runner code 里没有 `if tool == "creusot"`；只读 `tool.extra_cargo_deps`，通用 splice | §四 原则 C "异质性归配置，框架代码同质" |
| 原 example/ 仍不动 | 改的是 work_dir 拷贝 | §四 原则 A |
| 注入失败可识别 | `extra_cargo_deps` entry 在 discover 阶段就被 `toml_edit.parse()` 校验，bad TOML 在 discover 阶段 panic 不在 exec 阶段静默 | §一 不公信；detailed-design §七 |
| `[dependencies]` 不存在自动创建 | `entry().or_insert_with(Table::new())` | 工程鲁棒，公信落地 |

### 自己验

```sh
runner prepare creusot hello/basic-hello/hello
cat <work>/Cargo.toml | tail -3
# [dependencies]
# creusot-std = "0.11.0"
```

非 Creusot 工具：

```sh
runner prepare cargo-check hello/basic-hello/hello
diff examples/hello/basic-hello/Cargo.toml <work>/Cargo.toml
# (无差异)
```

→ 证明 patch 阶段是 declarative + 通用的，不针对工具名歧视。

---

## §6 阶段 5 — render harness：注入测量入口

这是 runner **最关键的"侵入"步骤**——`mod __ts_inner;` 的位置直接决定 SUCCESS 信号有没有意义。

### 做什么

```rust
let mut ctx = TeraContext::new();
ctx.insert("target_crate_name", &example.crate_name.replace('-', "_"));
ctx.insert("entry_fn",          entry);
ctx.insert("entry_args",        entry_args);    // 从 [runnable.<entry>].inputs[0] 渲染来
let rendered = tera.render("harness", &ctx)?;

match tool.entry_mode {
    EntryMode::Bin  => write!(work_dir/src/bin/__ts_harness.rs, rendered),
    EntryMode::Lib  => { rename(work_dir/src/lib.rs, work_dir/src/__ts_inner.rs);
                         write!(work_dir/src/lib.rs, rendered); }
}
```

### 形状 — bin 模式（cargo-check × runnable/add-u32/add_u32）

`tools/cargo-check/harness.rs.tera`：

```rust
fn main() {
    let _ = {{ target_crate_name }}::{{ entry_fn }}({{ entry_args }});
}
```

`examples/runnable/add-u32/hirusttest.toml`：

```toml
entries = ["add_u32"]
[runnable.add_u32]
inputs   = [[0, 0], [1, 2], [100, 200], [1000, 23456]]
expected = [0, 3, 300, 24456]
```

Tera 渲染后 `<work>/src/bin/__ts_harness.rs`（实测，run-id `prepare-1778588317-13180`）：

```rust
fn main() {
    let _ = runnable_add_u32::add_u32(0, 0);
}
```

读这条 evidence：

- `target_crate_name` 从 Cargo.toml `[package].name`，连字符 → 下划线（Rust ident 形式）：`runnable-add-u32` → `runnable_add_u32`
- `entry_fn` 从 hirusttest.toml `entries[]`，直接放进调用位置
- `entry_args` 从 `[runnable.add_u32].inputs[0]`，渲染成 Rust expr list `0, 0`

无 `[runnable]` 表的 entry → `entry_args = ""`，渲染出 `entry_fn()` 零参形式，与历史 146 个非 runnable entry 完全 back-compat。

### 形状 — lib 模式（verus × hello/basic-hello/hello）

`tools/verus/harness.rs.tera`：

```rust
use vstd::prelude::*;

verus! {
    mod __ts_inner;
    pub use __ts_inner::*;

    #[allow(dead_code)]
    #[verifier::external]
    fn __ts_invoke() {
        let _ = __ts_inner::{{ entry_fn }}({{ entry_args }});
    }
}
```

prepare 后（实测，run-id `prepare-1778588014-12475`）：

```sh
$ ls <work>/src/
__ts_inner.rs    # ← runner 把原 lib.rs 改名到这里
lib.rs           # ← 这是新写的 Verus harness

$ cat <work>/src/__ts_inner.rs
/// Smoke entry: zero-arg pub fn whose body exercises trivial Rust syntax.
pub fn hello() {
    let _ = 1 + 1;
}

$ cat <work>/src/lib.rs
use vstd::prelude::*;

verus! {
    mod __ts_inner;          # ← 关键：mod 写在 verus!{} 内部
    pub use __ts_inner::*;
    #[allow(dead_code)]
    #[verifier::external]
    fn __ts_invoke() {
        let _ = __ts_inner::hello();
    }
}
```

### 保证了什么 + 法律锚定 — render 阶段是最密集的一节

| 保证 | 怎么实现 | principles 锚 |
|---|---|---|
| 工具差异通过数据 (entry_mode + .tera) 表达 | runner code 只 `match entry_mode { Bin / Lib }`，不针对工具名 if-else | §四 原则 C |
| `target_crate_name` / `entry_fn` 是 oracle 反查锚点 | 每个 .tera 都引用这两个变量，wrapper 用它 grep 工具产物 | §六 当前 crate 焦点（宽度切割）："runner 注入的 TS_TARGET_CRATE / TS_ENTRY_FN 锚点指向" |
| **工具真正经过自己的前端**（关键！） | lib-mode harness 把 `mod __ts_inner` **写在 `verus!{}` 内**——如果在外面，Verus 把 inner 交给 stock rustc，SUCCESS 退化成"rustc parsed it"，与 cargo-check 没区别 | §六 前端测量（深度切割）："测必须命中工具自身前端，而非 rustc 等代理前端——否则 SUCCESS 信号退化为 'rustc parses it'，丢失工具间分化" |
| runnable corpus 不掉入 E0061 | `[runnable.<entry>].inputs[0]` → entry_args 注入 Rust expr list，bin-mode harness 调用点 typecheck 通过 | §一 不公信（false-positive-audit-2026-05-11.md §4.1 — 没这层之前 134 false-positive） |
| harness 模板 bug 是 UNKNOWN | render 失败由 main.rs catch → `status = "UNKNOWN", error = ...` | §六 Oracle 责任 UNKNOWN 严格语义 (b)："我们 harness 模板 bug" |

### 自己验

```sh
# 1. bin-mode 实测：原 lib 一字节没动
runner prepare cargo-check hello/basic-hello/hello
diff examples/hello/basic-hello/src/lib.rs <work>/src/lib.rs
# (无差异)

# 2. lib-mode 实测：原 lib 的字节存在于 __ts_inner.rs
runner prepare verus hello/basic-hello/hello
diff examples/hello/basic-hello/src/lib.rs <work>/src/__ts_inner.rs
# (无差异 — 字节级一致，只是改了文件名)

# 3. runnable corpus 实测：entry_args 真注入到了调用点
runner prepare cargo-check runnable/add-u32/add_u32
grep 'add_u32(' <work>/src/bin/__ts_harness.rs
# let _ = runnable_add_u32::add_u32(0, 0);
```

→ 这三条对比直接证明 §四 A + §六 当前 crate 焦点的实施落地。

---

## §7 阶段 6 — build spawn env：strip + inject

### 做什么

```rust
// 1. 剥掉 runner 内部 envvar
for k in std::env::vars().map(|(k,_)| k).filter(|k| k.starts_with("TS_")) {
    cmd.env_remove(k);
}
// 2. 注入 oracle 反查锚点
cmd.env("TS_ENTRY_FN",     entry);
cmd.env("TS_TARGET_CRATE", &crate_ident);
// 3. 注入 example 自己声明的运行时 env（hirusttest.toml [env]）
for (k, v) in &example.env {
    cmd.env(k, v);
}
cmd.current_dir(&target_in_workdir);
```

### 形状 — 实测三套不同形态

**Case A** — cargo-check × hello/basic-hello/hello（最干净）：

```
- strip every TS_* envvar from the child env
- inject TS_ENTRY_FN=hello
- inject TS_TARGET_CRATE=basic_hello
- cwd = <work>/
- argv (raw, ${TS_*} un-expanded) = ["cargo", "check", "--bin", "__ts_harness"]
```

**Case B** — verus × hello/basic-hello/hello（${TS_*} 路径展开）：

```
- inject TS_ENTRY_FN=hello
- inject TS_TARGET_CRATE=basic_hello
- argv (raw)      = ["${TS_VERUS_BIN}", "--no-verify", "--log", "vir", "--crate-type=lib", "src/lib.rs"]
- argv (expanded) = [".../verus", "--no-verify", ...]   # ← spawn 时用 expanded
```

`results.json` 里写的是 raw 形式（无主机绝对路径），spawn 时用 expanded——可发布性与可执行性分离。

**Case C** — cargo-check × industrial/x509-parser/cert-parse/x509_parse_der（[env] 注入）：

```
- inject TS_ENTRY_FN=x509_parse_der
- inject TS_TARGET_CRATE=x509_parser_cert_parse
- inject hirusttest [env] vars:
      RUSTFLAGS=--cap-lints=warn
- cwd = <work>/
- argv = ["cargo", "check", "--bin", "__ts_harness"]
```

读这条 evidence：

- vendor/x509-parser 上游作者写了 `#![deny(unstable_features, unused_qualifications)]`。新版 rustc 检出 8 处冗余 `crate::`，deny 升级成 error，cargo build 在工具前端看到代码之前就中断。
- 这是 vendor 代码风格选择，不是工具能力边界，也不是 corpus 内容问题。
- example 用 `[env] RUSTFLAGS=--cap-lints=warn` 自我声明运行时需求；runner 通用照做。
- 不动 example src/ 的同时解决问题——`[env]` 是 §四 原则 A 在运行时端的对偶。

### 保证了什么 + 法律锚定

| 保证 | 怎么实现 | principles 锚 |
|---|---|---|
| TS_* envvar 不泄漏给工具 | spawn 前 `env_remove` 所有 TS_* key | 防 prusti 这种把任何匹配自家前缀的 envvar 当 config 解析的工具崩；§六 Oracle 责任 "不藏 / 不冤枉" |
| oracle 锚点稳定 | TS_ENTRY_FN + TS_TARGET_CRATE 写死注入 | §六 当前 crate 焦点 — wrapper 反查工具产物里有没有 entry fn |
| `[env]` 不污染 example src | RUSTFLAGS 仅在 spawn 时注入；example src/ 与 Cargo.toml 不动 | §四 原则 A 信号文件非侵入的运行时对偶；P33 P-future v8 |
| results.json 不内嵌主机绝对路径 | argv 写 raw `${TS_*}` 形式 | §五 runner "裸数据自描述" + publish-ready 匿名化 |
| oracle 失败信号可识别 | wrapper 输出 `[<tool>-oracle] reason: ...` 到 stderr | §六 Oracle 责任 "不冤枉" + "不藏" |

### 自己验

```sh
# Case C 是最 informative 的——验 [env] 注入
runner prepare cargo-check industrial/x509-parser/cert-parse/x509_parse_der 2>&1 | grep -A1 'hirusttest \[env\]'
#   - inject hirusttest [env] vars:
#         RUSTFLAGS=--cap-lints=warn

# 验 example 仍干净
diff examples/industrial/x509-parser/cert-parse/Cargo.toml <work>/Cargo.toml
# (无差异)
diff examples/industrial/x509-parser/cert-parse/src/lib.rs <work>/src/lib.rs
# (无差异)
```

---

## §8 阶段 7 — spawn + capture：subprocess 隔离 + 不挂死

### 做什么

```rust
cmd.process_group(0);                                       // 自己一个 pgid
cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
let mut child = cmd.spawn()?;

// reader 线程读 pipe，防 buffer 溢出阻塞
let stdout_t = thread::spawn(move || read_to_end(stdout));
let stderr_t = thread::spawn(move || read_to_end(stderr));

match child.wait_timeout(timeout)? {
    Some(s) => (s, false),
    None => {
        kill(-pgid, SIGKILL);   // 整组干掉，cbmc / sub-cargo 一起死
        (child.wait()?, true)
    }
}
```

### 形状

每 task 都生成三个 raw 文件：

```
runs/<run-id>/raw/<tool>/
├── <feature>__<dir>__<entry>.stdout    # 完整捕获
├── <feature>__<dir>__<entry>.stderr    # 完整捕获 + wrapper diagnostic
└── <feature>__<dir>__<entry>.exit      # "Some(0)\n" / "Some(1)\n" / "__ts_timeout (after 300 s)\n"
```

文件路径在 results.json 里以 `raw_stdout` / `raw_stderr` 字段引用为 run_dir-relative。

### 保证了什么 + 法律锚定

| 保证 | 怎么实现 | principles 锚 |
|---|---|---|
| timeout 不挂死 runner | `process_group(0)` + SIGKILL pgid → cbmc / sub-cargo grandchild 一起死，pipe fd 不被锁住 | §五 runner "结果记录严谨全面"——挂死会跳过结果记录 |
| 完整 stdout/stderr 入文件 | reader 线程在 pipe buffer 满之前一直读 | §五 runner 裸数据自描述；§六 Oracle 责任 "不藏" |
| timeout 与正常 exit-fail 分开 | results.json `timed_out: bool` 独立字段 | §六 时空锚定——timeout 通常是工具版本/硬件相关，与 feature 缺失语义不同 |

### 自己验

跑一个会 timeout 的——例如把超时调小到 1 秒 + 跑 kani × concurrency entry，应该看到 `<entry>.exit` 写的是 `__ts_timeout (after 1 s)`，但 `<entry>.stderr` 仍有内容（kani 至少 flush 出来的部分）。

最直接看证据的是任何 raw 文件：

```sh
cat runs/<latest>/raw/cargo-check/hello__basic-hello__hello.stderr
#     Checking basic_hello v0.1.0 (...)
#     Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.09s
```

→ 完整 captured，不是摘要。

---

## §9 阶段 8 — cleanup OR --keep-work-dir：透明性的开关

### 做什么

```rust
if !keep_work_dir {
    fs::remove_dir_all(&work_dir).ok();    // best-effort
} else {
    eprintln!("[keep-work-dir] preserved: {}", work_dir.display());
}
```

### 形状 — 用 `--keep-work-dir` 看 spawn 之后的状态

```sh
$ runner --keep-work-dir --tool cargo-check --entry 'hello/basic-hello/hello'
Running 1 task(s) with parallelism = 10
[keep-work-dir] preserved: <runs>/run-xxx/work/cargo-check__hello__basic-hello__hello
[SUCCESS] cargo-check hello/basic-hello/hello (89ms)

$ find <runs>/run-xxx/work/.../ -maxdepth 3 -type f
.../Cargo.toml         # 不变
.../Cargo.lock         # ← cargo 跑完后自己生成（runner cp 时排掉的）
.../hirusttest.toml    # 不变
.../src/lib.rs         # 不变
.../src/bin/__ts_harness.rs    # runner render 阶段写的
.../target/debug/...           # cargo check 生成的编译产物
.../target/.rustc_info.json
```

可以直接 diff 看工具改了什么：

- runner 写了 `src/bin/__ts_harness.rs`
- cargo 自己又写了 `Cargo.lock` + 整个 `target/`
- example src/ 完全干净
- 这就是 SUCCESS 的"足迹"——审稿人可以自己验

对 lib-mode 工具：

```sh
$ runner --keep-work-dir --tool verus --entry 'hello/basic-hello/hello'
$ ls <work>/.verus-log/
vir-*.log                  # Verus 自己写的 VIR dump
```

→ 看到 `.verus-log/vir-*.log` 存在 = Verus front-end 确实跑了，不是 cargo-check 的伪 SUCCESS（§六 前端测量）。

### 保证了什么 + 法律锚定

| 保证 | 怎么实现 | principles 锚 |
|---|---|---|
| 默认可复现：每次跑都从干净起点 | cleanup is default | §五 runner "结果记录严谨全面" |
| 透明：审稿人能验"SUCCESS 真有意义" | `--keep-work-dir` opt-in，VIR dump / .lean / .v 产物可见 | §六 前端测量；§八 c+cc disprove-first 协议（审稿人能反挑刺有抓手） |
| 反作弊：runner 没有藏中间状态的能力 | cleanup 完全可关，paper artifact 流程也能要求开 | §一 不公信问题；ACM Artifact Functional badge |

### 自己验

见 §12 反作弊清单。

---

## §10 阶段 9 — classify + report：external_fault 归桶 + 写 results.json

### 做什么

```rust
let stderr_text = fs::read_to_string(&stderr_path)?;
let stdout_text = fs::read_to_string(&stdout_path)?;
if let Some(tag) = report::classify_external_fault(&stderr_text, &stdout_text, exit_code) {
    status = "UNKNOWN";
    error  = format!("external_fault: {}", tag);
}
write results.json + report.md
```

### 形状

```json
{
  "meta": {
    "run_id":           "run-1778587286-8915",
    "run_started_at":   "2026-05-12T10:21:26Z",
    "host":             { "os": "...", "cpu_brand": "...", "num_cpus": 10, ... },
    "parallelism":      10,
    "tools":            [{"name": "cargo-check", "command": ["cargo", "check", "--bin", "__ts_harness"], "version": "...", ...}, ...]
  },
  "results": [
    {
      "entry_id":     "hello/basic-hello/hello",
      "tool":         "cargo-check",
      "status":       "SUCCESS",
      "exit_code":    0,
      "duration_ms":  89,
      "timed_out":    false,
      "raw_stdout":   "raw/cargo-check/hello__basic-hello__hello.stdout",
      "raw_stderr":   "raw/cargo-check/hello__basic-hello__hello.stderr",
      "error":        null
    },
    ...
  ]
}
```

每条结果是**形状描述**而非评判——SUCCESS / FAILED / UNKNOWN / TIMEOUT 都是工具输出形态的分类标签。

### 保证了什么 + 法律锚定 — UNKNOWN 严格语义最关键

| 保证 | 怎么实现 | principles 锚 |
|---|---|---|
| UNKNOWN 只用于"runner / pipeline-upstream 故障" | external_fault catalogue 只匹配五类已知 upstream 模式（runnable mismatch / 单文件 pipeline 缺 cargo deps / 旧 cargo edition 拒绝 / vendor lint 严格 / runner 自身 spawn 失败） | §六 Oracle 责任 UNKNOWN 严格语义 (b) |
| 工具能力边界 → FAILED，不 UNKNOWN | classify_external_fault 没有匹配项 → 落回 FAILED | §六 UNKNOWN 严格语义底线："工具自身能力边界...一律 FAILED——按本地性原则 FAILED 站得住" |
| 不评判 / 只描述形状 | status 是分类标签，不是 score；report.md 不出工具排序 | §四 原则 B 测必要条件，非语义对错；§六 时空锚定（不构成对工具能力的长期承诺） |
| 时空锚定 | results.json 顶部含 ISO 8601 时间戳 + 工具 version_command 输出 | §六 时空锚定 |

### 自己验

```sh
# 看 v6 final results.json 的 status 分布
jq -r '.results[] | .status' deep-reports/cc-reports/.../results.json | sort | uniq -c
# 大致：
#    2202 SUCCESS
#    1008 FAILED
#      10 UNKNOWN
# UNKNOWN 占比 < 0.5%——证明 catalogue 真的"严格"，不当兜底
```

---

## §11 总览表 — 9 阶段 × 4 列

| 阶段 | 生成 / 改动 | 法律支撑 | prepare 看得到 | --keep-work-dir 看得到 |
|---|---|---|---|---|
| 1. discover | Vec<Example> + Vec<Tool> | §四 A 信号文件非侵入；§四 C 框架代码同质；§一 不公平/不公信 | ✗（内存） | ✗（内存） |
| 2. filter | Vec<(Tool, Example, entry)> | §一 不公平（笛卡尔积无 per-tool exclude） | ✗（内存） | ✗（内存） |
| 3. cp + isolate | work_dir/（去除 target/, Cargo.lock） | §四 A；§一 不公信（lockfile drift） | ✓ | ✓ |
| 4. patch Cargo.toml | work_dir/.../Cargo.toml | §四 C 数据驱动 | ✓ | ✓ |
| 5. render harness | src/bin/__ts_harness.rs / 或 src/lib.rs ↔ __ts_inner.rs | §四 A 对偶；§六 前端测量（`mod` 在 `verus!{}` 内部）；§六 当前 crate 焦点（entry 锚点） | ✓ | ✓ |
| 6. build spawn env | TS_* strip + TS_ENTRY_FN + TS_TARGET_CRATE + [env] | §四 A 运行时对偶；§六 Oracle 责任 "不藏 / 不冤枉" | ✓（打印 effective env + argv） | ✓（同 prepare） |
| 7. spawn + capture | raw/<tool>/<slug>.{stdout,stderr,exit} | §五 runner 裸数据；§六 Oracle 责任 "不藏" | ✗（不跑） | ✓ |
| 8. cleanup | work_dir 删 / 保留 | §五 可复现；§一 不公信（透明性） | ✗ | ✓（这就是 flag 的目的） |
| 9. classify + report | results.json + report.md | §六 UNKNOWN 严格语义；§四 B 测必要条件 / 非语义对错；§六 时空锚定 | ✗ | 跑了才有 |

---

## §12 反作弊自检清单（用 prepare + --keep-work-dir 实测验 6 条）

每条都能用前面阶段介绍的 flag 自己跑一遍，不必信 runner 的报告。

### 12.1 原 example/ 一字节没动

```sh
git status examples/      # 跑完应该是 clean
```

→ 验 §四 原则 A。如果有 modified 文件，runner 违反信号文件非侵入。

### 12.2 example 的 cargo bytes 在 work_dir 与原例字节级一致（bin-mode）

```sh
runner prepare cargo-check hello/basic-hello/hello
diff -r examples/hello/basic-hello/src/  <work>/src/   # 唯一差异：work 多了 src/bin/__ts_harness.rs
diff   examples/hello/basic-hello/Cargo.toml  <work>/Cargo.toml   # 无差异
```

### 12.3 lib-mode 的原 lib 内容完整保留在 __ts_inner.rs

```sh
runner prepare verus hello/basic-hello/hello
diff examples/hello/basic-hello/src/lib.rs  <work>/src/__ts_inner.rs
# (无差异 — 字节级一致，只改了文件名)
```

### 12.4 lib-mode `mod` 在 `verus!{}` 内部（SUCCESS 信号没退化）

```sh
runner prepare verus hello/basic-hello/hello
grep -A3 'verus!' <work>/src/lib.rs | head -5
# verus! {
#     mod __ts_inner;       ← 这里在 verus! {} 内
#     pub use __ts_inner::*;
```

→ 验 §六 前端测量。如果 `mod` 跑外面去了，Verus 把 inner 交给 stock rustc，SUCCESS == "rustc parsed it"，与 cargo-check 没区别。

### 12.5 oracle 锚点真注入了

```sh
runner prepare cargo-check runnable/add-u32/add_u32
# 输出末段应有：
#   - inject TS_ENTRY_FN=add_u32
#   - inject TS_TARGET_CRATE=runnable_add_u32
```

→ wrapper 用这两个变量 grep 工具产物（aeneas-* / rocq-of-rust 等），是 §六 当前 crate 焦点的实施。

### 12.6 工具 spawn 完真留了"它自己的足迹"（lib-mode 实测）

```sh
runner --keep-work-dir --tool verus --entry 'hello/basic-hello/hello'
ls <work>/.verus-log/
# vir-*.log
```

→ 验 §六 前端测量。SUCCESS 后没有 `.verus-log` = Verus 没真跑前端 = SUCCESS 信号无意义。

---

## §13 结语

这份 tutorial 把 runner 9 阶段每一步说清三件事：

- **形状**：每一步在文件系统 / 内存 / subprocess env 里**生成什么** — 用 prepare + --keep-work-dir 看得见
- **法律**：每一步**保证哪条** `principles.md` 条款 — 一一对锚
- **自己验**：每一条法律保证给一条最小命令做 evidence

外部观察者**不需要信 runner 的报告**——他们用 prepare 看 spawn 之前的输入形态，用 --keep-work-dir 看 spawn 之后的产物形态，**自己**做 §四 A + §六 前端测量的反挑刺（§八 c+cc disprove-first）。

宪法做的是**禁止式法律**（"不可侵入" / "不评判" / "前端命中"），runner 做的是**形状化生成**（具体写哪些文件 / 用哪些 env / cwd 在哪）；prepare + --keep-work-dir 是把两者拉到同一观察平面的窗口。

> **配套阅读**：宏观使用流程见 `tutorial.md`；publish 层 audit 标准见 `publish-readiness.md`；具体函数级实现见 `docs/design/detailed-design.md`；宪法本身见 `docs/design/principles.md`。
