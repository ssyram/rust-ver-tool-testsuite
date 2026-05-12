# Tutorial — 手动跑 + 读结果

> 不是 "怎么 cargo build" 的快速指南，是**手把手带你跑 4 个真实 demo + 教你怎么读输出**。所有命令本文都实跑过，输出原样 capture。
>
> 看完这个 tutorial 你能：
>
> - 跑 runner 任意工具子集 + entry 子集
> - 读 `results.json` / `report.md` / `raw/<tool>/<entry>.{stdout,stderr,exit}` 三层
> - 解释为什么某个 entry 在某工具上 SUCCESS / FAILED — 不是看 % 数字，看 stderr 实际信号
> - 解释 wrapper / oracle 实际怎么把 subprocess exit 翻成 SUCCESS / FAILED — **§六 当前 crate 焦点反向证明在 demo 4 直接演示**

参考：宪法 `docs/design/principles.md`、tutorial 之后看 `docs/publish/publish-readiness.md` 学外部观察者标准。

---

## §0 准备

### 0.1 系统要求

- macOS 或 Linux（dev / test 都在 macOS aarch64 + cargo 1.95.0 验证）
- `cargo` 装好 (rustup 装 stable rust 即可)
- 至少一个 stable Rust toolchain（默认）

### 0.2 拿到项目

```sh
git clone <repo-url>
cd rust-ver-tool-testsuite
git submodule update --init --recursive    # 必须！vendor/ 是 industrial entries 的依赖
cp .env.example .env                        # 按本机实际装的工具改 path
cargo build -p runner --release             # 编译 runner
```

实际 output（在本机跑的）：

```
~/.cargo/bin/cargo
cargo 1.95.0 (f2d3ce0bd 2026-03-21)

target/release/runner
target/release/runner: Mach-O 64-bit executable arm64
```

如果你只想跑 cargo-check 一种工具 demo，**.env 不用改**——cargo-check 用系统 cargo，不需要 TS_* 变量。

---

## §1 Demo 1 — 最小例（cargo-check × 1 entry）

### 1.1 看 entry 长什么样

```sh
ls examples/hello/basic-hello/
# Cargo.lock  Cargo.toml  hirusttest.toml  src/
```

**entry 由 4 部分构成**：

```sh
cat examples/hello/basic-hello/hirusttest.toml
# entries = ["hello"]

cat examples/hello/basic-hello/Cargo.toml
# [package]
# name = "basic_hello"
# version = "0.1.0"
# edition = "2021"
# 
# [lib]
# path = "src/lib.rs"

cat examples/hello/basic-hello/src/lib.rs
# /// Smoke entry: zero-arg pub fn whose body exercises trivial Rust syntax.
# /// Used as the simplest possible target for end-to-end runner verification.
# pub fn hello() {
#     let _ = 1 + 1;
# }
```

**理解**：

- `hirusttest.toml` 是 hirusttest 信号文件（只有 runner 读，cargo / rustc 完全不读）— 声明"该 entry 把 `pub fn hello()` 当 entry function"
- `Cargo.toml` 是标准 cargo manifest
- `src/lib.rs` 是 entry 实际 Rust 代码
- 没有任何 verifier-specific 注解（`#[kani::proof]` / `#[ensures]` / 等都没）—— 这是宪法 §四 A "不可侵入" 的体现

### 1.2 跑 runner

```sh
./target/release/runner --tool cargo-check --entry 'hello/basic-hello/*'
```

**实际 output**（本机刚跑的）：

```
Running 1 task(s) with parallelism = 10
[SUCCESS] cargo-check hello/basic-hello/hello (104ms)
---
Total: 1 succeeded / 0 failed / 0 unknown / 1 total
run_dir: <project>/runs/run-1778587286-8915
report:  <project>/runs/run-1778587286-8915/report.md
```

**理解每行**：

- `Running 1 task(s) with parallelism = 10` — runner 把 (tool, entry) 笛卡尔积 计算出 1 个 task，用 10 个 worker 并行跑
- `[SUCCESS] cargo-check hello/basic-hello/hello (104ms)` — 每个 task 跑完打一行 console log，5 字段：state / tool / entry_id / 耗时
- `Total` 行 — 总结统计
- `run_dir` — 本次 run 的所有产物存这里，名字是 `run-<unix_secs>-<pid>` 格式（unix_secs 保证按时间排序，pid 保证并发跑不冲突）

### 1.3 看 run_dir 结构

```sh
ls runs/run-1778587286-8915/
# raw/  report.md  results.json  work/
```

**理解**：

- `raw/<tool>/<entry-slug>.{stdout,stderr,exit}` — 每个 task 的 subprocess 真实 stdout/stderr/exit，保留作为 audit ground truth
- `report.md` — human-readable 矩阵报告
- `results.json` — machine-readable 完整记录（含 host info / tool versions / per-task results）
- `work/` — 跑完后空（每个 task work_dir 跑完即删，避免堆积。失败 task 也删，因为 raw/ 已 captured 必要信息）

### 1.4 看 raw outputs（the ground truth）

```sh
cat runs/run-1778587286-8915/raw/cargo-check/hello__basic-hello__hello.exit
# Some(0)
```

`Some(0)` 是 Rust `Option<i32>` 的 `Some(0)` 形式 — subprocess 正常退出 exit code = 0。

```sh
cat runs/run-1778587286-8915/raw/cargo-check/hello__basic-hello__hello.stderr
#     Checking basic_hello v0.1.0 (<work>/cargo-check__hello__basic-hello__hello)
#     Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.09s
```

cargo check 的标准输出（按 cargo 惯例，"Checking ..." / "Finished" 都打到 stderr 不是 stdout）。

```sh
cat runs/run-1778587286-8915/raw/cargo-check/hello__basic-hello__hello.stdout
# (empty)
```

stdout 空也正常——cargo check 不打 stdout，只打 stderr。

### 1.5 看 results.json（机读）

```sh
python3 -c "
import json
with open('runs/run-1778587286-8915/results.json') as f:
    d = json.load(f)
print(json.dumps(d['results'], indent=2, ensure_ascii=False))
"
```

实际 output：

```json
[
  {
    "entry_id": "hello/basic-hello/hello",
    "tool": "cargo-check",
    "status": "SUCCESS",
    "exit_code": 0,
    "duration_ms": 104,
    "raw_stdout": "raw/cargo-check/hello__basic-hello__hello.stdout",
    "raw_stderr": "raw/cargo-check/hello__basic-hello__hello.stderr"
  }
]
```

**理解每字段**：

- `entry_id` — 三段 path：`<feature>/<dir>/<entry-fn>`，唯一确定一个 entry
- `tool` — 工具名（`tools/<name>/` 目录名）
- `status` — `SUCCESS` / `FAILED` / `UNKNOWN`（v6 final 0 个 UNKNOWN）
- `exit_code` — subprocess exit（注意：是 wrapper 重写后的，不是工具原始 exit；详 §3）
- `duration_ms` — wall time，包含 work_dir cp + harness 渲染 + subprocess + 删 work_dir
- `raw_stdout` / `raw_stderr` — 相对路径，指向 raw/ 下文件

`results.json` 顶部还有 `host` / `tools[].version` / `run_started_at` 等 metadata — 详 §5 解读。

---

## §2 Demo 2 — 多 entry × 1 tool

```sh
./target/release/runner --tool cargo-check \
  --entry 'enum/**' --entry 'int/**' --entry 'panic/**'
```

实际 output：

```
Running 6 task(s) with parallelism = 10
[SUCCESS] cargo-check enum/data-variants/enum_match_data (163ms)
[SUCCESS] cargo-check int/wrapping/int_wrapping (165ms)
[SUCCESS] cargo-check panic/explicit/explicit_panic (168ms)
[SUCCESS] cargo-check enum/nested-guard/nested_match_guard (167ms)
[SUCCESS] cargo-check int/checked/int_checked (209ms)
[SUCCESS] cargo-check panic/div-zero/div_zero_path (241ms)
---
Total: 6 succeeded / 0 failed / 0 unknown / 6 total
```

**注意**：

- `--entry 'enum/**'` 是 glob pattern（`**` = 任意层级递归）
- 多个 `--entry` flag 是 OR（任一匹配即 include）
- 输出顺序不稳定（rayon 并发），但 results.json 内部按 sort key 稳定

**子集筛选 CLI flags 总览**：

```sh
runner                                       # 全矩阵（20 工具 × 161 entries = 3220 task）
runner --tool kani                           # 只跑 kani，全 corpus
runner --tool kani --tool miri               # 两工具
runner --entry 'closure-adv/**'              # 只跑 closure-adv/ 桶
runner --tool kani --entry 'industrial/*'    # 组合
runner --parallel 4                          # 限制并发度
runner report runs/run-XXXX                  # 仅从已有 results.json 重生成 report.md
```

---

## §3 Demo 3 — 真 FAILED（entry 自用工具不支持的特性）

我们故意挑一个 entry 知道它会 FAILED：`charon-limit/inline-asm/nop_via_asm` —— entry 自己写了 `core::arch::asm!()`，但 kani 不支持 inline assembly。

```sh
./target/release/runner --tool kani --entry 'charon-limit/inline-asm/*'
```

实际 output：

```
Running 1 task(s) with parallelism = 10
[FAILED ] kani charon-limit/inline-asm/nop_via_asm (754ms, exit=2)
---
Total: 0 succeeded / 1 failed / 0 unknown / 1 total
```

### 3.1 看 entry 自己长什么样

```sh
grep -n "asm!" examples/charon-limit/inline-asm/src/lib.rs
# 1://! Charon limitation: inline assembly blocks (`asm!`) are not supported.
# 23:            core::arch::asm!("nop", options(nomem, nostack, preserves_flags));
# 31:            core::arch::asm!("");
```

entry **自己写了** `core::arch::asm!` —— 这是宪法 §六 当前 crate 焦点意义下的"entry 自用工具不支持的特性"。

### 3.2 kani 跑完输出（subprocess + wrapper 双层）

raw stdout（kani 原始输出）：

```
Kani Rust Verifier 0.67.0 (cargo plugin)
   Compiling charon_limit_inline_asm v0.1.0 (<work>/...)
warning: Found the following unsupported constructs:
             - TerminatorKind::InlineAsm (1)
         
         Verification will fail if one or more of these constructs is reachable.
         See https://model-checking.github.io/kani/rust-feature-support.html for more details.

    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.30s
```

注意：kani 实际 **exit 0**（因为 `--only-codegen` 跑完就退）—— 5-markers 是 warning 不是 error。

**wrapper 看到 warning 后 grep + entry-src 关键字反向证明，把 exit 0 重写成 exit 2**：

```
[kani-oracle] FAIL: codegen with hard-unsupported markers; entry crate src/
[kani-oracle]       self-uses triggering keyword(s) for: ['TerminatorKind::InlineAsm']
[kani-oracle]       Per project §六-2 反作弊, entry-self partial → FAILED.
```

这两行就是 wrapper（`tools/kani/kani-strict-wrapper.sh` 内嵌 Python）的 diagnostic。

### 3.3 为什么"5-markers warning" 翻 FAILED — 不仅看 marker 本身

P37 后 wrapper 加了 §六 反向证明：marker fired 后**再 grep entry crate src/** 是否含 markers 触发关键字。这里 entry 自己写了 `asm!` → entry-self → FAILED 站得住。

---

## §4 Demo 4 — §六 当前 crate 焦点反向证明实际演示

挑一个 entry **自己没用**任何 5-markers 关键字，但 deps（std）内部用了 → marker 仍 fire 但 wrapper 应翻 SUCCESS。

```sh
./target/release/runner --tool kani --entry 'concurrency/thread-mutex/*'
```

实际 output：

```
Running 1 task(s) with parallelism = 10
[SUCCESS] kani concurrency/thread-mutex/thread_mutex_join (2208ms)
---
Total: 1 succeeded / 0 failed / 0 unknown / 1 total
```

### 4.1 entry 自身代码

```rust
// examples/concurrency/thread-mutex/src/lib.rs
pub fn thread_mutex_join() {
    use std::sync::{Arc, Mutex};
    use std::thread;
    let counter = Arc::new(Mutex::new(0i32));
    let mut hs = vec![];
    for _ in 0..2 {
        let c = Arc::clone(&counter);
        hs.push(thread::spawn(move || {
            let mut g = c.lock().unwrap();
            *g += 1;
        }));
    }
    for h in hs { h.join().unwrap(); }
    let _ = *counter.lock().unwrap();
}
```

entry 用了 `thread::spawn` / `Mutex` / `Arc::clone` —— **但 entry 自己没写** `asm!` / `catch_unwind` / `simd_*` / `::mask` / `c"..."` 任何一个 5-markers 关键字。

```sh
grep -E 'asm!|catch_unwind|simd_|::mask|c"' examples/concurrency/thread-mutex/src/lib.rs
# (0 hits)
```

### 4.2 kani 仍然 fire 5-markers warning（因为 std 内部用了）

raw stdout 包含 kani 5-markers warning：

```
warning: Found the following unsupported constructs:
             - C string literal (1)
             - caller_location (1)
             - catch_unwind (3)
             - foreign function (36)
             - ptr_mask (1)
```

`catch_unwind (3)` 等 fire——但**这些是 std 内部 `thread::spawn` 实现的事**，跟 entry 用户代码无关。

### 4.3 wrapper 的反向证明逻辑

wrapper（[kani-strict-wrapper.sh](../../tools/kani/kani-strict-wrapper.sh)）做的事：

```python
# 伪代码
markers_fired = grep_kani_5_markers(kani_stdout)
if not markers_fired:
    exit 0  # SUCCESS

entry_src = read_all_rs_files("src/")
entry_self_uses = []
for marker in markers_fired:
    triggers = {  # marker → 触发关键字正则
        'TerminatorKind::InlineAsm': r'\b(asm|global_asm)\s*!',
        'simd_cast': r'\b(simd_cast|simd_eq|...)\b|core::simd',
        'catch_unwind': r'\bcatch_unwind\b',
        'ptr_mask': r'::mask\s*\(|\bptr::mask\b',
        'C string literal': r'\bc"|CStr::from_bytes_with_nul',
    }[marker]
    if re.search(triggers, entry_src):
        entry_self_uses.append(marker)

if entry_self_uses:
    exit 2  # FAILED — entry 自用
else:
    exit 0  # SUCCESS — markers must come from deps
```

实际 wrapper diagnostic：

```
[kani-oracle] kani 5-markers fired ['catch_unwind', 'ptr_mask', 'C string literal'] but entry crate src/ has no triggering
[kani-oracle]       keyword; markers must originate from deps (std/registry/vendor).
[kani-oracle]       Per principles.md §六 当前 crate 焦点 (宽度切割), external-dep
[kani-oracle]       partial does not count — suppressing FAILED → SUCCESS.
```

**对比 Demo 3 + Demo 4**：

| Entry | entry src 含 marker 关键字 | wrapper 决策 |
| --- | --- | --- |
| `charon-limit/inline-asm/nop_via_asm` | `asm!` ✓ | FAILED (entry-self) |
| `concurrency/thread-mutex/thread_mutex_join` | 0 hits | SUCCESS (markers in deps, §六 豁免) |

这就是宪法 §六 "当前 crate 焦点（宽度切割）"在 oracle 实施层的具体体现——**测的是 entry crate 自己**，不是 deps。

---

## §5 读 results.json + report.md 完整解读

### 5.1 results.json schema

```json
{
  "run_id": "run-1778587286-8915",
  "run_started_at": "2026-05-12T12:01:26Z",
  "run_finished_at": "2026-05-12T12:01:28Z",
  "run_started_unix_secs": 1778587286,
  "run_finished_unix_secs": 1778587288,
  "host": {
    "hostname": null,       // P41 anonymized — 不收 hostname (privacy)
    "os": "macos",
    "arch": "aarch64",
    "kernel": "25.4.0",
    "cpu_brand": "Apple M5",
    "total_mem_mb": 24576,
    "num_cpus": 10
  },
  "parallelism": 10,
  "tools": [ ... 20 tool metadata ... ],
  "results": [ ... per-task records ... ]
}
```

**`tools[]` 字段**（P41 后用 raw `${TS_*}` form 不含私路径）：

```json
{
  "name": "aeneas-coq",
  "command": [
    "env",
    "CHARON_BIN=${TS_CHARON_BIN}",
    "AENEAS_BIN=${TS_AENEAS_BIN}",
    "${TS_PROJECT_ROOT}/tools/aeneas-coq/aeneas-coq-wrapper.sh"
  ],
  "timeout_secs": 600,
  "entry_mode": "lib",
  "extra_cargo_deps": [],
  "version": "aeneas a14083a6"
}
```

`${TS_*}` 是 placeholder，runner spawn 时 expand 成实际路径。published artifact 看到的就是这种 form。

### 5.2 results[] 字段

每个 task 一条 record（前面 demo 已展示）。SUCCESS 任务有 stdout/stderr 文件路径，FAILED 同样；UNKNOWN 任务 `raw_stdout` / `raw_stderr` 可能是 null（如 spawn 失败），并多一个 `error` 字段说明原因。

### 5.3 report.md 主要分段

- **Run metadata**：时间戳 + host info（hostname 隐去）+ 内存 cpu
- **Tool versions**：20 工具的 `version_command` 输出第一行——用于 reproducibility 锚定
- **Per-tool summary**：每工具 S/F/U 计数 + 时长分布（avg / median / p90 / max ms）
- **Per-feature summary**：按 feature 桶聚合
- **Per-feature × per-tool matrix**：按 feature 列出每 entry 在每工具上的 status（最有用的人读 view）

---

## §6 v6 final 主报告（已有快照）

不每次都重跑全矩阵——已经有 v6 final 快照在：

```sh
ls runs/run-1778560393-59119/
# raw/  report.md  results.json
```

主表（按家族分组、含 Wilson CI + Measurement boundary + Wrapper status 列）在：

```sh
less docs/test-reports/feature-coverage-2026-05-12-v6.md
```

**§3 主表的关键解读**（per family，**不**按 pass rate 比较）：

- **3.0 cargo-check (100%)** — baseline 角色，**不参与与其他工具横向能力比较**
- **3.1 Frontend translators** — charon (95%) / aeneas × 4 (40-61%) / hax × 3 (70-81%) / ror × 2 (76-77%)
- **3.2 Verifiers (deductive)** — kani (98.8%) / creusot (75%) / prusti (44%) / verus (41%)
- **3.3 verifast** (8.1%) — **caveat**：corpus 0 `//@` 注解，verifast 不真做 verification，仅 vacuous-pass detection
- **3.4 Abstract interpreter / symex** — miri (98%) / soteria (78%) / kmir (38%)

**关键解读规则**：

1. **不要直接比通过率**——跨家族工具不同 `Measurement boundary`，比通过率等于比苹果和橘子
2. **同家族内可比 with McNemar**：aeneas × 3 同（identical SUCCESS sets, p=1.0000）/ hax F* 显著高于 Coq (p=0.0000) / 等
3. **kani 98.8% 不暗示 kani 优于其他**：kani 跑前端层（`--only-codegen`），verifier-family 工具都切前端；测的是"前端能 codegen 的代码量"不是"工具实用价值"
4. **MIRI/soteria bug detect = SUCCESS 是工具按设计意图工作**，不是占便宜（detail：宪法 §六 + architecture §一 P35 派生）

---

## §7 cc-reports（per-tool 深度分析）

每工具有一份独立 cc-report：

```sh
ls deep-reports/cc-reports/
# aeneas-coq.md  aeneas-fstar.md  ...  verus.md  (20 个)
```

每份 cc-report 包含：

- 元数据 + 工具版本 + 通过率
- pipeline + 前端边界（每工具不同！）
- SUCCESS 信号 + 形式严格性（0 误报 / 0 漏报论证 + 漏报盲点）
- 失败分桶（按 P31 §四.5 归因 6 类）
- 修订建议清单（如有"我们导致"项）
- ΔS vs v5.1 解释

**读 cc-report 的最大价值**：理解某工具具体在哪些 entry FAILED + 是 entry 自用工具不支持的特性 vs deps 用 vs 工具自身 bug。

例：

```sh
# 看 kani cc-report
less deep-reports/cc-reports/kani.md
# 失败分桶 - 仅 1 桶（2 case 工具能力边界）:
#   - charon-limit/inline-asm/nop_via_asm (entry 自写 core::arch::asm!)
#   - kani-limit/stack-unwinding/trigger_divide_with_recovery (entry 自写 catch_unwind)
```

---

## §8 加新 entry 的最小流程

1. 创建目录 `examples/<feature>/<dir>/`
2. 写 `Cargo.toml`：单 lib crate，标准 cargo manifest，**不**加任何 verifier dep
3. 写 `src/lib.rs`：**plain Rust 子集**，零 verifier 注解，至少一个 `pub fn`
4. 写 `hirusttest.toml`：`entries = ["<fn_name>"]`
5. 测试：`./target/release/runner --tool cargo-check --entry '<feature>/<dir>/*'` 应 SUCCESS

宪法约束（§四 A）：

- `hirusttest.toml` 加入前后，`cargo build`/`check`/`run` 行为**字节级一致**（hirusttest.toml 是项目自有 schema，cargo 不读）
- entry crate src/ 不含 `#[<tool>::...]` / `#[ensures]` / `extern crate <tool>` 等任何 verifier-specific token
- entry crate 自包含（不跨 entry 共享 helper）

例：见 `examples/hello/basic-hello/` 的 4 个文件结构。

---

## §9 加新工具的最小流程

1. 创建 `tools/<name>/`
2. 写 `tool.toml` 声明 command + timeout + entry_mode + version_command
3. 写 `harness.rs.tera`：Tera 模板，3 个变量可用：
   - `{{ target_crate_name }}` — entry crate name (sanitized)
   - `{{ entry_fn }}` — entry fn name
   - `{{ entry_args }}` — entry call args (runnable corpus 用，default empty)
4. 若需要复杂 oracle gate：写 `<name>-strict-wrapper.sh`，tool.toml 指向它
5. 写 `tools/<name>/README.md`：pipeline / 前端边界 / SUCCESS 信号 / 漏报盲点（4 必含）
6. **不要**给 entry crate 注入 verifier-specific 注解（违反 §四 A）

参考实例：

- 最简：`tools/cargo-check/` — 无 wrapper，纯 cargo 调用
- 单工具自带 binary + 简单 oracle：`tools/charon-mono/` / `tools/verus/`
- 复杂 wrapper + 多 gate：`tools/kani/kani-strict-wrapper.sh` (P37 §六 反向证明)
- 两阶段 pipeline：`tools/aeneas-lean/aeneas-lean-wrapper.sh` (charon + aeneas backend)

详 `docs/design/tool-integration.md` + `docs/design/detailed-design.md` §五 各工具配置示例。

---

## §10 跑全矩阵（生产环境）

如果你装齐了 20 工具：

```sh
./target/release/runner    # 跑 161 × 20 = 3220 tasks
```

实测在 Apple M5 / 10 cpu / parallelism=10 下：

- 全矩阵 wall ≈ 25-35 分钟
- 主要瓶颈：creusot（cargo + nightly + Why3 IR 翻译，~40s/entry）+ kani (CBMC 求解前置 ~10s/entry)
- 单 entry / 单工具 demo（demos 1-4）都 < 1 分钟

如果只装了部分工具：

```sh
./target/release/runner --tool cargo-check --tool miri --tool kani
```

---

## §10.5 透明性 — 看 runner 实际侵入式做了什么

到这里你看的是 raw stdout/stderr 和最终 verdict。但 runner 在 spawn 工具之前会**修改一份 example 的隔离拷贝**——这是 principles.md §四 原则 A "signal file 非侵入"的对偶：信号文件不动 example，但**运行时拷贝是会被改的**（rendered harness 注入 / Cargo.toml 打补丁 / lib-mode 把 `src/lib.rs` 重命名）。默认这些 work_dir 在工具跑完后会 cleanup 掉。两个 flag 可以让你看见这一切：

- `runner prepare <tool> <entry>` — 只做 cp + 改写步骤，**不 spawn 工具**。看完整的"工具看到什么"。
- `runner --keep-work-dir ...` — 正常跑，但 spawn 后**不清理 work_dir**。看工具在改写 + 它自己跑完之后的所有产物（包括 cargo target/ 缓存）。

### 10.5.1 prepare：bin-mode（cargo-check / kani / miri / verifast / soteria / ...）

`cargo-check × hello/basic-hello` 是最干净的 bin-mode 例子。entry_mode = "bin" 意味着 example 的 `src/lib.rs` 不动，runner 在 `src/bin/` 下新加一个 `__ts_harness.rs` 作为调用点。

```sh
./target/release/runner prepare cargo-check hello/basic-hello/hello
```

输出（实际跑了，run-id `prepare-1778588004-12451`）：

```text
prepared work_dir: <runs>/prepare-1778588004-12451/work/cargo-check__hello__basic-hello__hello

tool         : cargo-check
entry_id     : hello/basic-hello/hello
entry_mode   : Bin
target_path  : .

Files written by the runner (intrusive layer):
  + <work>/src/bin/__ts_harness.rs   (rendered harness — new file)

At spawn time (skipped by `prepare`), the runner would also:
  - strip every TS_* envvar from the child env
  - inject TS_ENTRY_FN=hello
  - inject TS_TARGET_CRATE=basic_hello
  - cwd = <work>/
  - argv (raw, ${TS_*} un-expanded) = ["cargo", "check", "--bin", "__ts_harness"]
  - argv (expanded) = ["cargo", "check", "--bin", "__ts_harness"]
```

进 work_dir 看实际写下来的东西：

```sh
$ cd <work>
$ ls src/
bin  lib.rs                  # 原 lib.rs 完全没动

$ cat src/lib.rs              # === 原例 ===
/// Smoke entry: zero-arg pub fn whose body exercises trivial Rust syntax.
/// Used as the simplest possible target for end-to-end runner verification.
pub fn hello() {
    let _ = 1 + 1;
}

$ cat src/bin/__ts_harness.rs # === runner 新加 ===
// Template variables (see runner/src/exec.rs Tera context):
//   basic_hello — Rust ident form of cargo [package].name
//   hello       — entry function name from hirusttest.toml
//                 …
fn main() {
    let _ = basic_hello::hello();
}
```

读起来：**runner 唯一的"侵入"就是给 example crate 加了一个 binary target**。原 lib 一个字节没改。cargo-check 跑的是 "请检查这个 binary 能编译" — 这个 binary 唯一干的事是调一次 entry function。SUCCESS = "entry's Rust 被 rustc 接受了"。

### 10.5.2 prepare：lib-mode（Verus / Creusot / Aeneas 系列 / Soteria）

lib-mode 工具不是去编译一个 binary，而是要求 example 的 lib 本身就是 tool dialect。run 时 runner 把原 `src/lib.rs` 改名 `src/__ts_inner.rs`，把 rendered harness 写成新的 `src/lib.rs`。

```sh
./target/release/runner prepare verus hello/basic-hello/hello
```

输出（run-id `prepare-1778588014-12475`）：

```text
entry_mode   : Lib

Files written by the runner (intrusive layer):
  + <work>/src/lib.rs                (rendered harness — replaces original lib)
  + <work>/src/__ts_inner.rs         (original lib.rs renamed here, re-exported by harness)
```

```sh
$ ls <work>/src/
__ts_inner.rs  lib.rs

$ cat <work>/src/__ts_inner.rs    # === 原 lib.rs 完整保留，改了名 ===
/// Smoke entry: zero-arg pub fn whose body exercises trivial Rust syntax.
pub fn hello() {
    let _ = 1 + 1;
}

$ cat <work>/src/lib.rs           # === runner 渲染的 Verus harness ===
use vstd::prelude::*;

verus! {
    mod __ts_inner;
    pub use __ts_inner::*;

    #[allow(dead_code)]
    #[verifier::external]
    fn __ts_invoke() {
        let _ = __ts_inner::hello();
    }
}
```

注意 `mod __ts_inner` 写在 `verus! {}` **里面**——这是 principles.md §六 "确实经过工具" 的硬保证：如果 `mod` 在 `verus!` 外面，Verus 把 `__ts_inner` 交给 stock rustc，SUCCESS 就退化成"rustc parsed it"，跟 cargo-check 没区别。

### 10.5.3 prepare：Cargo.toml 注入额外依赖（Creusot）

少数工具要求 example 依赖 tool-specific support crate（Creusot 要 `creusot-std`）。Runner 通过 `extra_cargo_deps` 在 tool.toml 声明，prepare 时往 working-copy Cargo.toml 的 `[dependencies]` 里 splice 进去——**原 example/ 里的 Cargo.toml 不动**。

```sh
./target/release/runner prepare creusot hello/basic-hello/hello
```

输出：

```text
Files written by the runner (intrusive layer):
  + <work>/src/lib.rs                (rendered harness — replaces original lib)
  + <work>/src/__ts_inner.rs         (original lib.rs renamed here, re-exported by harness)
  ~ <work>/Cargo.toml                ([dependencies] patched with extra_cargo_deps:
        creusot-std = "0.11.0"
    )
```

```sh
$ diff examples/hello/basic-hello/Cargo.toml <work>/Cargo.toml
+ [dependencies]
+ creusot-std = "0.11.0"
```

读起来：原例没有 `[dependencies]`，runner 在 working copy 上加了一节。这是 principles.md §四 原则 C "异质需求沉到数据层"的实例——不在 runner 代码里给 Creusot 写 special case，而是 tool.toml 声明 `extra_cargo_deps`，runner 通用地照做。

### 10.5.4 --keep-work-dir：保留 spawn 之后的状态

`prepare` 截断到 spawn 之前。如果想看工具**跑完**之后留下了什么（编译产物、生成的 `.lean` / `.v` / `.vir.log`、wrapper 写的标记文件……），用 `--keep-work-dir`：

```sh
./target/release/runner --keep-work-dir --tool cargo-check --entry 'hello/basic-hello/hello'
```

stderr 会多打一行：

```text
[keep-work-dir] preserved: <runs>/run-xxx/work/cargo-check__hello__basic-hello__hello
```

然后 work_dir 里会有 cargo 自己产生的 `target/`、`Cargo.lock`（runner 之前 cp 时排了，cargo 重生成的）、rendered harness、未动的 src/lib.rs。验证用了这两个 flag 的人可以**自己**确认"runner SUCCESS 信号 == cargo 真的 check 过了"，不必信 runner 的报告。

### 10.5.5 这两个 flag 不是 paper artifact 流程的一部分

`prepare` 和 `--keep-work-dir` 是给**审计 / 学习 / debug 的人**的，不是生产流程。`examples/`、`tools/`、`runner/src/` + 一次正常的 `./target/release/runner` 跑完全自足。这两个 flag 只在你想"自己肉眼验证 runner 没作弊"的时候用。

### 10.5.6 进阶：按阶段读 runner 流水线

本节给的是"用法 demo"。如果你想看 **runner 9 阶段每一步生成什么形状 + 保证哪条 `principles.md` 法律 + 怎么用 prepare/--keep-work-dir 自己验**，读 [`tutorial-execution-walkthrough.md`](tutorial-execution-walkthrough.md)——按 discover → filter → cp → patch Cargo.toml → render harness → spawn env → spawn+capture → cleanup → classify+report 9 阶段铺开，每阶段四列（做什么 / 形状 / 法律 / 自己验），末尾给 §12 反作弊 6 条自检清单。

---

## §11 故障 & FAQ

### 11.1 runner 报 "TS_X must be set"

`.env` 没 source 或工具路径不对：

```sh
set -a && source .env && set +a
./target/release/runner --tool <name> ...
```

### 11.2 某工具全 FAILED stderr 含 "did not find a valid <tool>root"

工具 binary 在 `/tmp/` 被 macOS 周期清理（每周清 > 3 天未访问）。把工具迁到 `~/.local/share/ts-tools/<name>/` —— 见 `docs/fixes/v6-verus-env-fix-2026-05-12.md` 案例。

### 11.3 results.json 缺某些字段

results.json schema 在 v5.1 → v6 / P41 之间变化过（如 P41 加入 raw `${TS_*}` form / hostname=None）。旧 run 与新 schema 字段不同——不影响读取（用 jq / python dict 取需要的字段即可）。

### 11.4 wrapper diagnostic 在哪里

每个工具的 wrapper 输出 `[<tool>-oracle] ...` 行到 stderr。raw/<tool>/<entry>.stderr 末尾常含此 diagnostic（详 Demo 3 / 4）。

---

## §12 进阶 — 自己写 oracle 分析脚本

results.json 是 machine-readable，几行 Python 就能写自定义分析：

```python
import json
with open("runs/run-1778560393-59119/results.json") as f:
    d = json.load(f)

# 找所有 kani SUCCESS 但 stderr 含 unsupported 警告的 entry
import os
for r in d["results"]:
    if r["tool"] == "kani" and r["status"] == "SUCCESS":
        stderr_path = os.path.join("runs/run-1778560393-59119", r["raw_stderr"])
        with open(stderr_path) as fp:
            text = fp.read()
        if "Found the following unsupported constructs" in text:
            print(f"{r['entry_id']}: kani SUCCESS but markers fired (P37 §六 deps-only)")
```

**这种自定义 grep 是 cc-report agent 的工作方式**——你也能直接复用（详 `.claude/skills/tool-cc-report-rewrite.md` skill）。

---

## §13 下一步

- 读 `docs/design/principles.md` 学宪法精神（§六 双切割 + 对称性）
- 读 `docs/publish/publish-readiness.md` 学外部观察者标准 + ACM Artifact Badges
- 读 `docs/test-reports/feature-coverage-2026-05-12-v6.md` 学完整 quantitative analysis（含 Wilson CI / McNemar / ToV）
- 加新 entry 或新工具——按 §8 / §9 流程
- 投稿 publishing：按 `docs/publish/publish-readiness.md §3.10` 走 anonymization workflow

---

## Appendix — 本 tutorial 跑过的实际 demo run-ids

| Demo | run id | 命令 |
| --- | --- | --- |
| 1 | run-1778587286-8915 | `runner --tool cargo-check --entry 'hello/basic-hello/*'` |
| 2 | run-1778587336-9313 | `runner --tool cargo-check --entry 'enum/**' --entry 'int/**' --entry 'panic/**'` |
| 3 | run-1778587348-9788 | `runner --tool kani --entry 'charon-limit/inline-asm/*'` |
| 4 | run-1778587363-10125 | `runner --tool kani --entry 'concurrency/thread-mutex/*'` |
| 10.5.1 (prepare bin) | prepare-1778588004-12451 | `runner prepare cargo-check hello/basic-hello/hello` |
| 10.5.2 (prepare lib) | prepare-1778588014-12475 | `runner prepare verus hello/basic-hello/hello` |
| 10.5.3 (prepare + cargo patch) | prepare-1778588029-12592 | `runner prepare creusot hello/basic-hello/hello` |

实际 output 已嵌入 §1-§4 + §10.5 各小节。
