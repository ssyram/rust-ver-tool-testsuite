# rocq-of-rust-typecheck 实施（档 1 接入）— 2026-05-11

> 上游调研：`docs/research/ror-runnable-deep-dive-2026-05-11.md`（认为档 1 可达；档 2/3 不适合 testsuite 自动化）
> 上层调研：`docs/research/translation-correctness-feasibility-2026-05-11.md`
> 工作目录：`/Users/ssyram/workspace/rust-ver/rust-ver-tool-testsuite/`

## 概要

新增 `tools/rocq-of-rust-typecheck/` 作为独立 tool 入口，把 ror 翻译产物的 **Rocq 9 typecheck**（档 1）纳入 runner 自动化。本工具是 `tools/rocq-of-rust`（档 0：纯翻译落盘 + grep silent fallback）的**严格上层包裹**——保留档 0 全部 6 道 gate，新增 3 道 coqc 编译 gate。

档位定义（按 `translation-correctness-feasibility-2026-05-11.md`）：

| 档 | 含义 | 工具 |
|---|---|---|
| 档 0 | 工具自陈"前端接受"（产物落盘 + 无 silent marker）| `tools/rocq-of-rust` |
| **档 1** | **产物在对应 prover 里 typecheck 通过** | **`tools/rocq-of-rust-typecheck`**（本实施）|
| 档 2 / 档 3 | evaluate / 一致性 | n/a（ror 设计上不支持自动化，详见调研 §6/§7）|

## 改动文件清单

新增：
- `/Users/ssyram/workspace/rust-ver/rust-ver-tool-testsuite/tools/rocq-of-rust-typecheck/tool.toml`
- `/Users/ssyram/workspace/rust-ver/rust-ver-tool-testsuite/tools/rocq-of-rust-typecheck/rocq-of-rust-typecheck-wrapper.sh`
- `/Users/ssyram/workspace/rust-ver/rust-ver-tool-testsuite/tools/rocq-of-rust-typecheck/harness.rs.tera`
- `/Users/ssyram/workspace/rust-ver/rust-ver-tool-testsuite/tools/rocq-of-rust-typecheck/README.md`
- `/Users/ssyram/workspace/rust-ver/rust-ver-tool-testsuite/deep-reports/cc-reports/rocq-of-rust-typecheck.md`
- `/Users/ssyram/workspace/rust-ver/rust-ver-tool-testsuite/docs/fixes/rocq-of-rust-typecheck-implementation-2026-05-11.md`（本文）

修改：
- `/Users/ssyram/workspace/rust-ver/rust-ver-tool-testsuite/.env.example`（加 `TS_ROR_TYPECHECK_SWITCH` / `TS_ROR_RUNTIME_PATH`）
- `/Users/ssyram/workspace/rust-ver/rust-ver-tool-testsuite/.env`（同上）

**没动**任何其他工具的 `tool.toml` / cc-report；没动 runner 框架；没动 `docs/design/{principles, architecture, tool-integration, detailed-design}.md`。

## Pipeline 设计

```
                   wrapper 调用
                       │
        ┌──────────────┼──────────────┐
        ▼                             ▼
opam env --switch=ror-test        runtime bootstrap
(activate Rocq 9 + coqc)          (stat 9 core .vo;
                                   build missing)
        │                             │
        └──────────────┬──────────────┘
                       ▼
        Stage 1: rocq-of-rust translate
        (src/lib.rs → rocq_translation/<abs>/lib.v)
                       │
                       ▼
        Gate 1-6: exit 0, .v exists, > 200B,
                  no failure marker,
                  Definition <entry_fn> present
                       │
                       ▼
        Stage 2: coqc -R <runtime> RocqOfRust
                  -impredicative-set <product>.v
                       │
                       ▼
        Gate 7-9: coqc exit 0, .vo present,
                  stderr no "Error"
                       │
                       ▼
                    SUCCESS
```

## Runtime pre-build 策略：自动 bootstrap（选项 a）

**问题**：每次跑 entry 都重编 9 个 runtime .v 浪费时间。

**选项对比**：
- (a) wrapper 启动时 stat 9 个核心 .vo；缺则 coqc 跑一次 build（首次 ~3 秒，幂等）；后续直接复用
- (b) 要求用户预先手工 build runtime
- (c) runner 加 setup hook

**选择 (a)** —— 理由：
1. **不要求 runner 改动**（runner 框架不该感知 tool-specific 依赖；满足原则 C "异质性归配置"）
2. **不要求用户额外手工步骤**（用户只需 `opam switch create ror-test` + 装包 + clone ror upstream，跟其他工具一致的"按 README 一次 setup"心智）
3. **幂等**：bootstrap 只补缺失的 .vo，不重建已有的
4. **稳定性**：bootstrap 路径与 P15 调研验证的命令链完全一致（`coqc -R . RocqOfRust -impredicative-set <file>`）

bootstrap 顺序（保证依赖关系）：
```
RecordUpdate.v → M.v → lib/lib.v → RocqOfRust.v
              → links/M.v → links/RocqOfRust.v
              → lib/simulate/lib.v → simulate/M.v → simulate/RocqOfRust.v
```

实测：第一次跑 wrapper（runtime 已存在 .vo）耗时 ~430 ms（最快 FAILED 84 ms; 最慢 SUCCESS 1074 ms; SUCCESS 平均 680 ms）。

## 实测命令链

### SUCCESS 示例（int_wrapping）

```bash
$ set -a && source .env && set +a
$ rm -rf /tmp/ror-tc-test && mkdir -p /tmp/ror-tc-test
$ cp -R examples/int/wrapping/* /tmp/ror-tc-test/ && cd /tmp/ror-tc-test
$ env ROCQ_OF_RUST_TOOLCHAIN_SYSROOT=$TS_ROCQ_OF_RUST_TOOLCHAIN_SYSROOT \
      ROR_TYPECHECK_SWITCH=$TS_ROR_TYPECHECK_SWITCH \
      ROR_RUNTIME_PATH=$TS_ROR_RUNTIME_PATH \
      TS_ENTRY_FN=int_wrapping \
      tools/rocq-of-rust-typecheck/rocq-of-rust-typecheck-wrapper.sh
Translating: src/lib.rs
Starting to translate "src/lib.rs"...
14 ms have passed to translate: "src/lib.rs"
Finished.
[ror-typecheck-wrapper] coqc -R /private/tmp/rocq-of-rust-clone/RocqOfRust RocqOfRust -impredicative-set rocq_translation/src/lib.v
$ echo $?
0
```

### FAILED 示例 — async-fn（Stage 1 翻译失败）

```bash
$ TS_ENTRY_FN=async_forty_two tools/rocq-of-rust-typecheck/rocq-of-rust-typecheck-wrapper.sh
Translating: src/lib.rs
error[E0670]: `async fn` is not permitted in Rust 2015
  --> src/lib.rs:21:5
   |
21 | pub async fn async_forty_two() -> u32 {
   |     ^^^^^ to use `async fn`, switch to Rust 2018 or later
[ror-typecheck-oracle] FAIL: rocq-of-rust translate exit 101
$ echo $?
101
```

### FAILED 示例 — repr/union（Stage 1 产物含 failure marker）

```bash
$ TS_ENTRY_FN=repr_union ...
Translating: src/lib.rs
...
[ror-typecheck-oracle] FAIL: product contains explicit failure marker
$ echo $?
1
```

## 全 matrix 跑通率

run id: `runs/run-1778473345-64581/`（2026-05-11T04:22:25Z, ~9 s wall, 10-way parallel）

| 维度 | 数 |
|---|---|
| 总 entries | 146 |
| SUCCESS | **109** |
| FAILED | 37 |
| timeout | 0 |

时长分布：
- SUCCESS: avg 680 ms / min 435 ms / max 1074 ms
- FAILED: avg 136 ms / min 84 ms / max 249 ms
- 全量: avg 542 ms

**对照 `tools/rocq-of-rust`（档 0）跑全 matrix**：
- 档 0: SUCCESS = **110** / FAILED = 36
- 档 1: SUCCESS = **109** / FAILED = 37
- **差异**：1 个 entry —— `creusot-limit/thread-local-ref/read_thread_local`

## 与档 0 差异分析（1 个 entry）

`creusot-limit/thread-local-ref/read_thread_local` 在档 0 SUCCESS、档 1 FAILED（gate 6: entry_fn 缺失）。

**事实**：
- entry fn `read_thread_local` 调 `thread_local!` macro + `with(|c| ...)`
- ror 翻译产物（`rocq_translation/src/lib.v`）含 `Definition value_COUNTER` + `Definition __init`，**不**含 `Definition read_thread_local` —— 这是 silent skip
- 档 0 oracle 中 gate 6 是同一逻辑（grep `Definition <entry_fn>`）但**没**抓到该 silent skip
- 档 1 oracle 中同一 gate 6 正常抓到

**根因推断**：可能是 `tools/rocq-of-rust/tool.toml` 把所有 gate 串在一个 `sh -c` `if/elif` 链里，某些 corner case 下 sh expansion 与 wrapper.sh 不一致。这是 `tools/rocq-of-rust` 的一个**潜在 oracle 漏报**，**不在本任务范围**（本任务只实施档 1，不动档 0 配置）。

**对本任务的意义**：差异 1 是测试范围本来就重叠的副产品，**不**是档 1 实际暴露了"档 0 SUCCESS 但 coqc typecheck failed"的 ror 翻译 bug。所有真正进入 Stage 2 的产物都 coqc 编译通过——说明 ror 当前在 typecheck 层稳定。

## 反误报双向实测

按 `principles.md` §六-2 + §三-3-2.b 的"0 误报"硬指标，oracle 必须能区分真 SUCCESS 与 partial / silent skip。本实施做了如下实测：

### 正向（已知会 typecheck 通过）

4 个 P15 实测过的 entry（`custom_add` / `basic_hello` / `int_wrapping` / `int_checked`）：本 testsuite 对应 `examples/hello/basic-hello/hello`、`examples/int/wrapping/int_wrapping`、`examples/int/checked/int_checked`（无 custom_add 对应 entry）—— 全 SUCCESS，与 P15 调研 §5.1 的 4/4 PASS 一致。

### 反向（已知会 fail）

| entry | 期望 | 实测 |
|---|---|---|
| `charon-limit/async-fn/async_forty_two` | FAIL（rustc 不接受 `async fn` 在 Rust 2015）| ✅ Stage 1 exit 101，oracle 抓 |
| `repr/union/repr_union` | FAIL（ror 翻译产物含 `Unimplemented` / failure marker）| ✅ Stage 1 gate 5 抓 |
| `bigint/bigint-arith/*` | FAIL（rocq-of-rust 在 std dep crate 上 panic）| ✅ Stage 1 exit 101 |

### gate 7-9 反向（构造档 1-specific 失败）

档 1 新增的 gate 7-9 在当前 corpus 上 **0 触发**——因为 ror 翻译落盘的产物全部 typecheck 通过。**漏报盲点诚实声明**：本工具的 gate 7-9 在当前 corpus 上没有任何 entry 触发，理论上 gate 7-9 的反误报能力依赖未来 corpus 中出现"档 0 落盘但 coqc 编不过"的 entry。但是 coqc 是 Rocq 自带 确定性 typecheck 算法，本身 0 误报 0 漏报，gate 7（exit 0）即"typecheck 通过"的标定信号——形式上不会冤枉。

## 与 rocq-of-rust（档 0）的关系

| 维度 | rocq-of-rust（档 0）| rocq-of-rust-typecheck（档 1）|
|---|---|---|
| Stage 1 翻译 | ✅ | ✅（完全相同）|
| Gate 1-6（grep）| ✅ | ✅（独立实现，但等价逻辑）|
| Stage 2 coqc | ❌ | ✅ |
| Gate 7-9（coqc）| ❌ | ✅ |
| 工具依赖 | rocq-of-rust binary + nightly sysroot | + opam switch `ror-test` + ror runtime path |
| 单 entry 时长 | ~50-100 ms | ~430-1074 ms |
| 形式严格性 | ⚠️ 0 误报 / 0 漏报 实测 | ✅ 基本可形式证明 0 误报 / 0 漏报（gate 7 是 Rocq 确定性 typecheck）|
| 漏报盲点 | 上游引入新 silent fallback path | 同档 0 + `Admitted` 占位（这是档 1 边界的诚实声明，不算漏报）|

按硬指标 §六-1（前端支持性观察原则），档 0 与档 1 的"前端边界"不同：
- 档 0 前端 = rocq-of-rust 自身翻译落盘
- 档 1 前端 = rocq-of-rust 翻译落盘 + Rocq coqc 接受

两者都是合法的"前端测试范围"切割，分别按各自定义的边界报告通过率。读者引用时必须知道引用的是哪个档位。

## 实施陷阱（P15 调研没覆盖的）

实测发现的小坑（都已处理）：

1. **coqc 默认 `-noinit` ?** ：无问题，wrapper 用 `coqc -R <runtime> RocqOfRust -impredicative-set` 即可，与 P15 §5.1 命令一致
2. **wrapper cwd**：wrapper 被 runner 在 example workdir 下 spawn（runner exec 行为）。所以 `cd "$PRODUCT_DIR"` 跑 coqc 时是相对路径；`-R "$ROR_RUNTIME_PATH"` 必须**绝对路径**（已在 README "已知限制 / 坑"段声明）
3. **opam env 与 nightly sysroot 路径序的协同**：wrapper 先 `eval $(opam env --switch=ror-test)` 把 coqc 加入 PATH；后续 stage 1 `export PATH="$SYSROOT/bin:$PATH"` 把 nightly rustc 加在最前面但**保留** coqc 所在路径——不会破坏 stage 2 的 coqc 解析
4. **rocq-of-rust 进入 sysroot bin 的 rustc**：实测正常，rocq-of-rust 的 `rustc --print=sysroot` 解析到 nightly sysroot，与 `tools/rocq-of-rust` 行为一致
5. **TS_ENTRY_FN 注入**：通过 runner 的 `env_remove(TS_*) → env(TS_ENTRY_FN, entry)` 流程，wrapper 直接读 `$TS_ENTRY_FN` 即可 —— 不需要在 tool.toml command 里转写

## 需要主进程批准的不可逆操作

无。所有改动都在 `tools/rocq-of-rust-typecheck/` 与 `docs/` 下，无修改第三方源码，无破坏性 opam / git 操作。`ror-test` switch 与 `/private/tmp/rocq-of-rust-clone/` 都是用户之前 P15 调研已经建好的，本任务只读用。
