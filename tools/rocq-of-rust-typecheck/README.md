# rocq-of-rust-typecheck

ror 翻译产物在 Rocq 中的 typecheck（档 1）自动化——`rocq-of-rust` 档 0 的严格上层包裹。

## 简介

`rocq-of-rust` 是 Rust → Rocq 翻译器（formal-land 出品）。`tools/rocq-of-rust` 测档 0（前端 = 翻译落盘 + 6 道 grep 门）；本工具在此基础上加 Stage 2，**真在 Rocq 9 里 `coqc` 编译产物**，把档 1 接入 testsuite。

档位定义（按 `docs/research/translation-correctness-feasibility-2026-05-11.md`）：

| 档 | 含义 | 本工具 |
|---|---|---|
| 档 0 | 工具自陈"前端接受"（`.v` 产物落盘 + 无显式 failure marker）| `tools/rocq-of-rust` |
| **档 1** | **产物在 Rocq 里 typecheck 通过** | **本工具** |
| 档 2 | 产物 entry_fn `Compute` 出值 | n/a（ror 设计上不支持）|
| 档 3 | evaluate 结果 == `cargo run` 结果 | n/a（ror 设计上不支持）|

档 2/3 不实施的理由：ror 产物是 deep embedding `M = LowM.t (Value.t + Exception.t)`，每个 op 是 inductive constructor，无 native compute；上游设计上把"语义"留作用户 proof obligation（详见 `docs/research/ror-runnable-deep-dive-2026-05-11.md` §6.2）。

## 本测试集中的"前端接受"定义

本工具是 ror 的**档 1 收紧**版：在档 0 的 6 道门（翻译落盘 + 无 silent fallback）之上加 typecheck。

- **前端**（本工具检测的范围）：rocq-of-rust translate → `.v` 产物 → `coqc -R <runtime> RocqOfRust` → `.vo`
- **后端**（本工具不检测）：用户自己拿 `.vo` 写 `Run.Trait` instance + `simulate/<fn>.v`，配合 `SimulateM.eval_f` 走档 2/3 ——纯人工 proof engineering，不在本工具范围

### Pipeline 图

```
src/lib.rs
    │
    ▼  Stage 1: rocq-of-rust translate
    │  (DYLD_LIBRARY_PATH = nightly sysroot lib, PATH 前置 nightly bin)
    │
rocq_translation/<abs-path>/lib.v
    │
    ▼  Stage 2: coqc -R <runtime> RocqOfRust -impredicative-set <product>.v
    │  (opam switch = ror-test; Rocq 9.0.0)
    │
<product>.vo  + stderr 检查
```

Stage 1 与 `tools/rocq-of-rust` 完全等价（同 sysroot / 同 binary / 同 6 道 grep）。Stage 2 是本工具新增。

### 判定与 SUCCESS 信号

**SUCCESS 信号（严格反映档 1 边界）**：满足 9 道门。

档 0 已有的 6 道门（gate 1-6）：

1. rocq-of-rust translate exit code = 0
2. 至少一个 `.v` 产物存在
3. 无 0-byte `.v`
4. 至少一个 `.v` > 200 字节
5. 产物不含显式 failure marker grep：`\(\* (Error |Unexpected |Please report!|thir failed to compile|Unimplemented )`
6. 产物中至少一个 `.v` 含 `^[[:space:]]*Definition[[:space:]]+<TS_ENTRY_FN>[[:space:]]`

档 1 新增 3 道门（gate 7-9）：

7. **coqc exit code = 0**
8. **coqc 产出 `.vo`**（exit 0 + 无产物的理论 belt-and-braces）
9. **coqc stderr 无 `^Error` 行**（exit 0 + 含 Error 的理论 belt-and-braces）

任一门失败 → FAILED。

**Runtime bootstrap 策略（自动幂等）**：每次调用 wrapper 先 stat 9 个核心 runtime `.vo` 是否在 `$ROR_RUNTIME_PATH` 下：

```
M.vo  RocqOfRust.vo  RecordUpdate.vo
lib/lib.vo  lib/simulate/lib.vo
links/M.vo  links/RocqOfRust.vo
simulate/M.vo  simulate/RocqOfRust.vo
```

——任一缺失则按依赖顺序 `coqc` 跑一次 build（首次 < 5 秒）；全在则直接走 Stage 2（< 1 秒/entry）。bootstrap 是 self-contained，**不要求用户手工 setup**；只要求用户按"安装"段一次性建好 `ror-test` switch。

### partial 暴露机制

档 1 的 partial 信号通路（按硬指标 §六-2 不允许 partial）：

1. **rocq-of-rust 端的 silent partial**（同档 0）：translate exit 0 + 产物含 explicit failure marker / 缺 entry_fn `Definition` → gate 5/6 抓
2. **coqc 端的 partial**：coqc exit ≠ 0（Rocq 9 typecheck 失败的官方信号）→ gate 7 抓
3. **coqc exit 0 但无 .vo**（理论上不应发生，但 coqc 早退路径需 defence in depth）→ gate 8 抓
4. **coqc exit 0 但 stderr 有 Error**（理论上同上）→ gate 9 抓

### 形式严格性

**0 误报（不冤枉能力）**：✅ **基本可形式证明**。gate 7（coqc exit 0）即 Rocq 9 typecheck 完整通过——Rocq typecheck 是确定性算法，无随机性、无 silent partial 路径。gate 8/9 仅是 coqc exit 0 的 belt-and-braces 复核，不会冤枉合法 SUCCESS。gate 1-6 已由 `tools/rocq-of-rust` 论证（参 `docs/fixes/oracle-leak-audit-2026-05-08.md` §3.2）。

**0 漏报（不高估能力）**：✅ **基本可形式证明**。coqc 对任何 typecheck 失败必 exit ≠ 0；gate 7 直接捕获。理论上 ror 翻译可能利用 axiom / `Admitted` 让产物 typecheck 通过但语义不真——本工具档 1 边界明确**只**保证 typecheck 通过，**不**保证"语义正确"或"evaluate 一致"。这与档 2/3 的覆盖范围严格区分。

**漏报盲点**：
- **`Admitted` 一通到底**：ror 翻译生成的 `Global Instance Instance_IsFunction_<fn> : ... Admitted.` 是 ror 的设计选择（让 `Instance` lookup 通过即可），typecheck 通过包含 `Admitted` 占位——这**不算漏报**，是档 1 的诚实边界："产物能在 Rocq 中编译通过"包含 `Admitted` 路径。本工具不评判 ror 设计选择，只测产物可编译性
- **ror 翻译走 axiom**：若 ror 上游引入 `Axiom <name> : ...`（理论上 typecheck 仍过），同上不算漏报
- 本工具档 1 边界**不**等同档 2/3（evaluate / 一致性），不构成对工具语义正确性的判断

## 安装

上游 ror：<https://github.com/formal-land/rocq-of-rust>

本测试基线：rocq-of-rust commit `a8a76a4d` + Rocq 9.0.0 + 隔离 opam switch `ror-test`。

**一次性 setup**（详见 `docs/research/ror-runnable-deep-dive-2026-05-11.md` §3）：

```bash
# 1. 创建隔离 opam switch（不动 default / rocq9 等）
opam switch create ror-test --empty

# 2. 装 Rocq 9 + ror runtime 依赖
opam install --switch=ror-test ocaml-base-compiler.5.2.0 dune.3.18.2 -y
opam install --switch=ror-test rocq-core.9.0.0 rocq-stdlib.9.0.0 -y
opam install --switch=ror-test rocq-smpl coq-hammer.1.3.2+9.0 coq-coqutil -y

# 3. clone ror upstream（runtime 源码位于 RocqOfRust/）
git clone https://github.com/formal-land/rocq-of-rust.git
# → RocqOfRust 目录 ≈ $TS_ROR_RUNTIME_PATH

# 4. 把 binary 路径 + sysroot + switch + runtime path 填到 .env
#    rocq-of-rust binary 在 PATH 上（cargo +nightly-2024-12-07 install --path cli/）
#    其余参 .env.example
```

总用时：~15 min（switch + Rocq 9 install）+ ~3 min（首次 wrapper 调用自动 build 9 个核心 runtime）。

**清理**：`opam switch remove ror-test`（一次性释放 ~1.6 GB）。

## 本框架配置

参见 `tool.toml` + `rocq-of-rust-typecheck-wrapper.sh`。关键参数：

- **command**：通过 `env` 注入 `ROR_TYPECHECK_SWITCH` / `ROR_RUNTIME_PATH` / `ROCQ_OF_RUST_TOOLCHAIN_SYSROOT` 后调 wrapper
- **wrapper**：activate opam switch → runtime bootstrap → Stage 1 translate → 6 道 grep 门 → Stage 2 coqc → gate 7-9
- **entry_mode**：默认 `bin`；harness 写入 `src/bin/__ts_harness.rs`，但 rocq-of-rust 只读 `src/lib.rs`，harness 不参与
- **timeout_secs**：180 s（rocq-of-rust 一般 < 5 s + coqc 一般 < 1 s + 首次 bootstrap ~3 s + slack）

**与 tools/rocq-of-rust 的关系**：本工具是档 1 的严格上层包裹。Stage 1 与 `tools/rocq-of-rust/tool.toml` 等价（同 6 道 grep）；Stage 2 是新增。因此**任一 entry 在本工具 SUCCESS ⇒ 在 `tools/rocq-of-rust` SUCCESS**（档 1 ≤ 档 0）；反过来一个 entry 可能 ror 档 0 SUCCESS 但 coqc 编不过——这种 entry **暴露 ror 翻译的 silent typecheck bugs**（产物落盘但 Rocq 内部不接受）。

## 已知限制 / 坑

- **runtime path 必须为绝对路径**：wrapper 用 `coqc -R "$ROR_RUNTIME_PATH" RocqOfRust`，相对路径会因 wrapper 在 example workdir 下 cd 而失效
- **switch 名假定 = "ror-test"**：可改但需对应改 `.env` 的 `TS_ROR_TYPECHECK_SWITCH`
- **runtime bootstrap 是 destructive on first call**：在 `$ROR_RUNTIME_PATH` 下写入 9 个 `.vo` —— 但只写入缺失的，是幂等的
- **coqc binary 来自 opam switch**：wrapper 用 `eval $(opam env --switch=...)` activate；若用户机器没装 opam，gate 失败显式报错
- **macOS-only 测试**：基线在 Apple M5 / darwin 25.4.0；Linux 应该 work 但未实测
- **不测 evaluate / 一致性**：本工具**只**测 typecheck（档 1）。ror 档 2/3 是 per-entry proof engineering，与"特性覆盖广度筛选" testsuite 任务严重不匹配——参 `docs/research/ror-runnable-deep-dive-2026-05-11.md` §6 / §7

## 关联 sub-tests

本工具未派生限制集 agent，无 `examples/rocq-of-rust-typecheck-limit/`。

跟 ror 档 0 的 corpus 重合：在 `tools/rocq-of-rust` 标 SUCCESS 的 entry 集合上跑本工具，对比通过率差——差额暴露 ror 翻译的"产物能 emit、但 Rocq 不接受"的 silent typecheck bug。
