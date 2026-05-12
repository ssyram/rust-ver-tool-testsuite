# v6 误报候选审查（c 路独立审查）

时间：2026-05-12
审查对象：`runs/run-1778560393-59119/`（v6，3220 task / 1001 FAILED）
对比基线：`runs/run-1778504159-67797/`（v5.1）

## 1. 总览

- **候选数：0（强候选 0 / 弱候选 0 / 自审排除 N 类）**
- 涉及工具：审查覆盖 v6 vs v5.1 FAILED 增量大的全部工具：verifast (+24), verus (+22), rocq-of-rust (+24), rocq-of-rust-typecheck (+23), soteria (+22), prusti (+7), aeneas-* (+1~2), 以及 hax-lean/hax-coq/kani/miri
- 涉及 entry：审查覆盖 v6 新增 corpus 全部 21 个（bigint/* × 8, deps-complex/* × 7, industrial/* × 6）+ hax-limit/let-chains + kani-limit/async-await + 其余非典 entry
- **结论：未发现可升级为 SUCCESS / UNKNOWN 的误报。v6 1001 FAILED 全部站得住宪法 §六 UNKNOWN 严格语义。**

## 2. 自审排除的类别（按宪法 §六分类）

### 类别 A：工具单文件 pipeline 不读 Cargo deps（NOT 误报）

**涉及工具**：rocq-of-rust, rocq-of-rust-typecheck, verifast, verus, soteria（共 21 × 5 = 105 个 FAILED）
**涉及 entry**：bigint/* (8), deps-complex/* (7), industrial/* (6)
**stderr 共性**：`error[E0432]: unresolved import \`num_bigint\``（或 `sha2`/`rsa`/`x509_parser` 等）

**反向论证**（工具开发者会不会驳回 FAILED？）：
- rocq-of-rust README 明确"ingests a single .rs file via rustc_interface; it doesn't read Cargo.toml"（见 `tools/rocq-of-rust/tool.toml` header），开发者会承认 FAILED
- verifast / soteria / verus 同属单文件 pipeline 类
- 工具能力边界，宪法本地性原则下 FAILED 站得住

**结论**：NOT 误报。

### 类别 B：工具自带 cargo / toolchain 不识 edition 2024（NOT 误报）

**涉及工具**：prusti（共 7 个 v6 新增 FAILED）
**涉及 entry**：hax-limit/let-chains, industrial/rsa/* (2), industrial/sha2/* (2), industrial/x509-parser/* (2)
**stderr 共性**：
```
failed to parse the `edition` key
... this version of Cargo is older than the `2024` edition,
and only supports `2015`, `2018`, and `2021` editions.
```

**反向论证**：
- prusti pin 的 toolchain 是 `nightly-2024-09-23`（自带老 cargo）
- prusti 开发者读到这个 FAILED 会说"是的，prusti 当前 toolchain 不支持 edition 2024 / let-chains"，不会驳回
- 部分 industrial entries（rsa, x509-parser）依赖 `base64ct v1.8.3` / `time-macros v0.2.27`，这些 crates 自身用了 edition 2024 → prusti cargo 解析 dep manifest 失败
- 工具自带 wrapper（cargo-prusti）的能力边界

**结论**：NOT 误报。

### 类别 C：工具官方驱动 crash / 自爆 panic（NOT 误报）

**涉及**：verus rustc-internal panic、charon-driver "Coroutine types are not supported" panic、hax prettyplease panic、creusot `internal error: entered unreachable code`、prusti `[Prusti: internal error]`

**stderr 样本**：
- `runs/run-1778560393-59119/raw/charon-mono/charon-limit__async-fn__async_forty_two.stderr:11` — `thread 'rustc' panicked at src/errors.rs:282:13: Coroutine types are not supported yet`
- `runs/run-1778560393-59119/raw/verus/generic__sum-bound__generic_sum_bound.stderr:2` — `thread 'rustc' panicked at ty/generic_args.rs:54:14: index out of bounds`
- `runs/run-1778560393-59119/raw/prusti/deps-complex__bigint-serde__bigint_serde.stderr:128` — `error: [Prusti: internal error] Prusti encountered an unexpected internal error`

**反向论证**：
- 这些是工具**官方**驱动 / 后端的崩溃，不是我们 wrapper 的脚本 bug
- 宪法明确"工具官方 wrapper crash → FAILED"
- 开发者会承认"是的，我们 backend 在这种构造上 panic 了"

**结论**：NOT 误报。

### 类别 D：我们的 wrapper 检测到 silent partial（设计意图，NOT 误报）

**涉及工具**：aeneas-{coq,fstar,hol4,lean}, hax-{coq,fstar,lean}, rocq-of-rust, rocq-of-rust-typecheck, kani, kmir, prusti（共 555 处 oracle FAIL）

**stderr 共性**：wrapper 自报 `[<tool>-oracle] FAIL: <理由>`，理由包括：
- "charon exited 0 but emitted partial-signal stderr ('is not supported' or '^error:')"
- "entry_fn missing from .v products (silent skip)"
- "silent partial — sorry in term position"
- "codegen completed with hard-unsupported MIR constructs"
- "K interpreter stuck"
- "cargo-prusti exited 0 but produced 0 .vpr files"

**反向论证**：
- 这些是 wrapper 按宪法 §六-2「反作弊 / 无 partial」设计意图正确触发——工具 exit 0 但实际 silent 跳过了 entry / 丢失了支持
- 不是 wrapper 脚本内部 bug；wrapper 自身 IO / env 解析无失败
- 升 UNKNOWN 会违反 §六 严格语义（这些不是"我们这边可识别问题且暂未修"——而是工具能力边界被严格识别）

**结论**：NOT 误报。

### 类别 E：Vendor lint UNKNOWN 已被 oracle 正确接住（已分类）

**情况**：industrial/x509-parser/cert-parse/* 在 aeneas-hol4, hax-{coq,fstar,lean}, kani 上是 UNKNOWN（10 处），对应 vendor_lint_strictness 规则。
**结论**：oracle 已正确分类为 UNKNOWN，非 FAILED 误判。

## 3. 主动搜索过的"我方 wrapper 内部 bug"信号（均未命中）

按候选 1（我们 wrapper bash 内部 bug）grep 全部 raw 目录：
- `: unbound variable` / `TS_.*: parameter not set` / `command not found` — **0 命中**
- `wrapper.sh: line` / bash 语法报错 — **0 命中**
- `permission denied` / `No such file or directory` — **0 命中**

按候选 2（全局工具链崩溃）grep：
- `JVM` / `OutOfMemory` / `StackOverflow` / `java: command not found` / `cargo: command not found` / `viper_tools` / `Z3 not found` — **0 命中**
- `core dumped` / `Segmentation fault` / `killed` — **0 命中**

## 4. 总结

- **强候选数：0**
- **弱候选数：0**
- **自审排除：A/B/C/D/E 五类全部站得住宪法 §六**

v6 的 1001 FAILED 全部为工具能力边界或工具官方组件 crash 或 wrapper 设计意图触发的 silent-partial 截断；无一是我方 wrapper 内部 bug 或全局工具链崩溃。报告稳定性不需要因为误报而调整。

P27（DP-4 严格化 + D3 wrapper gate）的影响域被独立审查证实：v5.1 → v6 增量 FAILED 全部由 corpus 扩展（bigint / deps-complex / industrial 共 21 entries）+ 已有 oracle 规则识别工具能力边界产生，未在 v6 引入新的误报源。
