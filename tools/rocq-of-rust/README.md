# rocq-of-rust

Rust 源码到 Rocq（原 Coq）的自动翻译工具，生成可进一步做形式化证明的 `.v` 文件。

## 简介

rocq-of-rust 由 Formal Land 出品，通过 `rustc_interface` 直接读取 `.rs` 文件并翻译为 Rocq 定义。不是 cargo subcommand，是独立 binary。使用 nightly-2024-12-07 toolchain（通过 `rustc_private` API），需锁定到该 toolchain 构建。翻译结果为 Rocq monadic IR，可在 Rocq 中进一步做等价性证明或规格验证。

GitHub: <https://github.com/formal-land/rocq-of-rust>

## 本测试集中的"前端接受"定义

本测试集筛选 Rust **前端特性覆盖广度**——把每个工具测到"工具自带后端验证器/求解器之前"为止，不看下游求解结果。

rocq-of-rust 是**纯翻译工具**，没有内置的 Coq type-check / 证明阶段——pipeline 终点就是 `.v` 文件写盘。所以本工具"前端 = 全过程 = 翻译到 .v"。

- **判定**：exit 0 + 产物存在 + 通过扩展 grep marker 检测 = SUCCESS
- **产物**：`rocq_translation/<absolute-source-path>.v`（每个 fn 翻译为 `Definition <name>` + `Global Instance Instance_IsFunction_<name> : M.IsFunction.C "..." <name>. Admitted.` typeclass 注册）
- **覆盖度精确意义**：SUCCESS = "rocq-of-rust 把全部源码 item lower 到产物，且无可见落空标记"。下游 `coqc` 是否能 type-check 该 .v 文件**不在本测试范围**——这依赖 RocqOfRust runtime library 提供 std/外部 crate 的 binding

### Silent fallback 检测（当前 oracle：v4 = 7 道门 + N-attempt wrapper）

rocq-of-rust 设计上对未支持的构造**仍 exit 0**，把失败信号塞进 `.v` 文件里的特定标记。当前 oracle 通过 `tools/rocq-of-rust/rocq-of-rust-wrapper.sh` 实施 7 道门 + N-attempt AND-reduce（详见 §"SUCCESS 信号"）。门 5 是 corpus-tested 的 5 类显式 failure marker grep：

```sh
! grep -rqE '\(\* (Error |Unexpected |Please report!|thir failed to compile|Unimplemented )' rocq_translation_*
```

门 6（2026-05-08 引入，2026-05-11 升级为 N-attempt）是 `Definition <entry_fn>` 存在性 grep。

已知 silent fallback 路径（深度调研，参考 [`deep-reports/cc-reports/rocq-of-rust.md`](../../deep-reports/cc-reports/rocq-of-rust.md)）：

- THIR 编译 panic → `(* thir failed to compile *) tt`（被门 5 抓）
- TyKind 落空 → `RocqType::var("type ... not yet handled")`（实测 0 现象，门 5 + 门 6 间接覆盖）
- `extern crate` / `use` / `macro_rules!` 静默丢（vec![] 直接返回）—— 合理 skip；但若 entry 名误指向这些 item，门 6 抓
- `TopLevelItem::Error` 系列（GlobalAsm / Union / TraitAlias）→ `(* Error <Variant> *)`（被门 5 抓）
- `ConstKind::Infer/Bound/Placeholder` → 裸名 `InferConst` / `BoundConst` / `PlaceholderConst`（const 单独 case，本 corpus 实测 0 现象）
- `lib/src/core.rs:157` 的 HashMap.next() 单文件丢盘 → 多 mod 项目仅第一个文件入 .v（这种 fn 缺失会被门 6 抓）
- **非确定性翻译路径** (2026-05-11 P15-impl 反向暴露)：`thread_local!` 宏触发的 entry 上同 binary 同输入翻译输出在 fn-present / fn-dropped 之间随机切换；单次 grep 漏报，N-attempt AND-reduce 抓住 —— 详 [`docs/fixes/ror-gate6-fix-2026-05-11.md`](../../docs/fixes/ror-gate6-fix-2026-05-11.md)

### SUCCESS 信号（严格反映前端特性支持范围）

为了严格反映前端特性支持范围（不允许 partial），rocq-of-rust 的 SUCCESS 必须满足 **7 道门，且 wrapper N=7 次 `rocq-of-rust translate` 都通过所有门**（2026-05-11 起；现行 `rocq-of-rust-wrapper.sh` 实施；N 经验校准至 99.84% catch rate—详 fix doc §3）：

1. exit code = 0（每次 attempt）
2. 至少一个 `.v` 产物存在（每次 attempt）
3. 无 0-byte `.v`（每次 attempt）
4. 至少一个 `.v` > 200 字节（每次 attempt）
5. 产物不含显式 failure marker grep：`\(\* (Error |Unexpected |Please report!|thir failed to compile|Unimplemented )`（每次 attempt）
6. 产物中至少一个 `.v` 文件含 `^[[:space:]]*Definition[[:space:]]+<TS_ENTRY_FN>[[:space:]]` —— entry 函数名必须真出现在 Rocq 产物里（非 silent skipped）（每次 attempt）
7. stderr 不含 `is not yet supported`（2026-05-12 P28 / D3.1 加；rocq-of-rust 对 silent `Pattern::Wild` 退化等 partial 走 stderr warning + exit 0，本门拦截；每次 attempt）

任何一次 attempt 的任何门未满足 → FAILED。

第 7 道门由 runner 注入的 `TS_ENTRY_FN` 环境变量驱动（`runner/src/exec.rs:178`），封堵 audit ([`docs/fixes/oracle-leak-audit-2026-05-08.md`](../../docs/fixes/oracle-leak-audit-2026-05-08.md) §3.2) 提到的"完全 skip item 类"silent漏报路径——典型场景：entry 名实际是 `use` / `extern crate` 别名或者其他非 fn item，rocq-of-rust 在 `top_level.rs:349-390` 走 `vec![]`，产物中没有该 fn 的 `Definition`。`^[[:space:]]*` 锚点允许嵌套模块内的 fn 也命中。

N-attempt（默认 N=7，可通过 `ROCQ_OF_RUST_N_ATTEMPTS` env 覆盖）封堵 2026-05-11 P15-impl 反向暴露的非确定性翻译路径漏报：单次 grep 在 `thread_local!` 宏触发 entry 上随机 SUCCESS / FAILED，N 次 AND-reduce 把 P(漏) 压缩到原 P(漏)^N（实测 P(漏)=0.4，N=7 → catch rate 99.84%）。

**partial 暴露机制**：rocq-of-rust **设计上不用 exit code 表达 partial**（几乎永远 exit 0，对所有 unsupported 用 rustc warning，不影响 exit）。所以 oracle 完全靠产物 grep + 产物 shape + N-attempt 稳定性检测——这是工具自身设计决定的"前端测试范围"切割方式。

**形式严格性 — 0 误报（不冤枉能力）**：⚠️ 实测验证 0 误报，但**不可形式证明**。oracle 用保守的 marker 集——只抓 rocq-of-rust 自己 emit 的 explicit failure comment 块（`(* (Error |Unexpected |Please report!|thir failed to compile|Unimplemented )`），用户合法代码极难误命中。早期试探性 16-marker 已主动收缩到这 5 类显式 failure marker 以避免误报。门 6 的 grep `Definition <name>` 是 rocq-of-rust 翻译产物的固定形式（每个 fn item 必生成 `Definition <name> ...`），合法代码不会缺；N-attempt 不引入新误报路径——对确定性翻译路径（占 corpus 大头）3 次产物 byte-identical，AND-reduce 与单次结果相同。

**形式严格性 — 0 漏报（不高估能力）**：⚠️ 实测验证 0 漏报，但**不可形式证明**。rocq-of-rust **设计上不用 exit code 表达 partial**（永远 exit 0，对所有 unsupported 用 rustc warning），所以 oracle 只能靠产物字面 grep + 产物 shape 检测；理论上 rocq-of-rust 上游可能引入新 fallback 路径不带这些 marker，或新的非确定性翻译路径在 3 次 attempt 中恰好都不 drop entry_fn。门 6 N-attempt 把已知非确定性 silent skip 闭环（2 个 thread_local! 类 entry 在 v4 重跑下稳定 FAILED）。

**漏报盲点**：
- 上游引入新 silent fallback 路径不带已知 markers（实测在 examples corpus 0 现象）
- 完全 skip item 类（`use` / `extern crate` / `macro_rules!` 在 `top_level.rs:349-390` 直接 `vec![]`）：这些是 rustc 编译时已被处理的 import / macro，**不需要在产物里有 declaration**，所以是合理 skip，**不算漏报**——但若 entry_fn 名误指向这些 item（错误 corpus 设置），门 6 会捕获
- 新非确定性翻译路径中，N 次 attempt 都恰好采到含 entry_fn 的变体——可通过把 N 增大缓解（`ROCQ_OF_RUST_N_ATTEMPTS` env 已暴露；当前默认 N=7 在 P(drop)=0.4 时 catch rate 99.84%）

## 安装

上游：<https://github.com/formal-land/rocq-of-rust>

本测试基线：commit `a8a76a4d`（cli/，搭配 nightly toolchain `nightly-2024-12-07`）。

按上游文档自行安装（rustup 装好 nightly toolchain 后从 cli/ `cargo install` 自行处理）。`rocq-of-rust` 本体由 PATH 解析；本工具不直接配置 binary 路径，但需要把对应 nightly toolchain 的 sysroot 目录填到 `.env` 的 `TS_ROCQ_OF_RUST_TOOLCHAIN_SYSROOT`（runtime 注入 `DYLD_LIBRARY_PATH` 与 `PATH`，使 rocq-of-rust 找到自身 `librustc_driver-*.dylib` 与 nightly `rustc`）。本项目不提供安装脚本或步骤教程，避免在工具版本变迁后误导。

## 本框架配置

参见 `tool.toml` + `rocq-of-rust-wrapper.sh`。关键参数：

- **command**：`tool.toml` 通过 `env` 把 `ROCQ_OF_RUST_TOOLCHAIN_SYSROOT` 传给 `rocq-of-rust-wrapper.sh`；wrapper 内部循环 N=7 次调用 `rocq-of-rust translate --path src/lib.rs --output-path rocq_translation_<i>`，对每次产物逐项跑 7 道门 AND-reduce。
- **entry_mode**：未设置（默认 `bin`）；harness 写入 `src/bin/__ts_harness.rs`，但 rocq-of-rust 只读 `src/lib.rs`，harness 不参与翻译。
- **DYLD_LIBRARY_PATH**：wrapper 内设置，指向 nightly-2024-12-07 sysroot 的 `lib/`，使 `librustc_driver-*.dylib` 在 macOS 上可被动态链接器找到。
- **PATH 注入**：wrapper 内设置，将 nightly sysroot 的 `bin/` 置于 PATH 最前，确保 rocq-of-rust 内部调用 `rustc --print=sysroot` 时返回 nightly sysroot（而非 stable）。
- **N-attempt 可调**：wrapper 读 `ROCQ_OF_RUST_N_ATTEMPTS` env（默认 7）。要在 corpus 调试新非确定性现象时可以临时调大。
- **静默吞错升级**：rocq-of-rust 对不支持的构造 exit 0 但在 `.v` 文件中嵌入 `(* Unexpected ... *)` 等注释块；wrapper 第 5 道门 grep 把这类静默失败升级为非 0 exit，映射为 FAILED。第 7 道门 N-attempt 把 entry_fn silent skip（确定性的 + 非确定性的）一并捕获。
- **产物 symlink**：wrapper 成功后建 `rocq_translation/` symlink 指向 `rocq_translation_1/`，便于下游消费者（如 typecheck wrapper / 手动检查）按历史路径访问产物。

## 与 hax-lean 的可运行性对比

ror 与 hax-lean 都是 Rust → prover IR 的翻译工具，但**产物形态不同**，导致档 2/3（evaluate / 与 Rust 一致）的可达性截然不同。详见 [`docs/research/ror-runnable-deep-dive-2026-05-11.md`](../../docs/research/ror-runnable-deep-dive-2026-05-11.md)。

**深嵌入（deep embedding）vs 浅嵌入（shallow embedding）**：

- **ror 是 deep embedding**。产物形如 `Definition fn (a b : Value.t) : M.t LowM.t (Value.t + Exception.t) := let* ... call_closure BinOp.Wrap.add ...`。`Value.t` 是 inductive 包装类型，`M : LowM.t (...)` 是 effect monad；所有 op（`alloc` / `read` / `call_closure` / `call_primitive`）是 inductive constructor，**无 Compute 语义**。`vm_compute` / `native_compute` 直接 SIGSEGV（axiom-laden `Run.t` proof tree 在 native compute 下 crash）。
- **hax-lean 是 shallow embedding**（参考定位）。产物形如 `def fn (a b : Int32) : RustM Int32 := RustM.ok (a + b)`，Lean `#eval fn 3 4` 一行直接出 `RustM.ok 7`。

**ror 上游"官方运行模式"**：

- API：`SimulateM.eval` / `SimulateM.eval_f`（`simulate/M.v:343 / 445`）。
- 性质：**propositional 解释器**，不是 native compute。
- 输入：`LinkM.t R Output` + 需要 `Run.Trait` 实例（用户用 `run_symbolic` tactic 推导）。
- 输出：`SimulateM.t` inductive，**不是** native `Z`。
- 与"值"关系：propositional（`🌲` notation 是 `Run.t value (eval_f run stack)` 的 sugar），用 `repeat (eapply Run.Call || apply Run.Pure)` 证明。
- 性能：per-entry **5–50+ 行手工 Coq tactic**；递归 fn 需手工 well-founded induction。

**档可达性总结表**：

| 档 | ror | hax-lean |
| --- | --- | --- |
| 档 0 前端接受 | ✅ `tools/rocq-of-rust`（本工具）| ✅ `tools/hax-lean` |
| 档 1 typecheck | ✅ `tools/rocq-of-rust-typecheck` | ✅ feasibility 实测 |
| 档 2 auto evaluate | ❌ **架构上不可达** | ✅ `#eval` 实测 |
| 档 2 半人工 lemma | ⚠️ per-entry 5–50 行手工证明 | — |
| 档 3 与 Rust 一致 | ❌ 除非档 2 解决 | ✅ byte-identical 实测 |

**项目决策**：

- **投入档 1 自动化**（已上线 `tools/rocq-of-rust-typecheck`）。
- **不投入档 2/3 自动化**：per-entry 手工证 vs corpus ~150 entries 规模严重不匹配。这是 ror **设计选择**（deep embedding 为形式证明优化），不是 bug。
- 严格说，本工具（rocq-of-rust 档 0）测的是"翻译产物在 Coq 里是有效的 Coq 项"——**结构正确，语义不验证**；语义验证属于 ror 上游"link + simulate"人工证明工作流，超出本 testsuite "特性覆盖广度筛选"任务范围。

## 已知限制 / 坑

- **静默吞错**：工具无 `--abort-on-error` 等价旗标，遇不支持的构造不报 exit 非 0，必须依赖 grep 检测 `.v` 文件中的占位注释来识别翻译失败。
- **单文件输入**：rocq-of-rust 通过 `rustc_interface` 直接处理 `.rs` 文件，不读 Cargo.toml，不支持跨 crate 依赖。
- **toolchain 锁定**：必须使用 nightly-2024-12-07 构建 binary 并在运行时注入对应 sysroot；换 toolchain 需重新 `cargo install`。
- **输出路径结构**：`--output-path` 指定目录后，输出路径为 `<output-path>/<绝对输入路径>.v`，需提前 `mkdir -p`。
- **翻译质量**：Rust 语言特性覆盖不完整，复杂 trait 实现、`unsafe` 指针操作、宏展开后的代码等易触发占位注释。

## 已知限制 / 平台兼容

**当前测试运行环境**：macOS aarch64（Apple Silicon）。

**平台特定配置**：

- `rocq-of-rust-wrapper.sh` 内 `export DYLD_LIBRARY_PATH="$SYSROOT/lib"`（macOS-specific dynamic linker 变量，让 `librustc_driver-*.dylib` 可被解析）
- `version_command` 同样使用 `DYLD_LIBRARY_PATH`
- `.env` 中 `TS_ROCQ_OF_RUST_TOOLCHAIN_SYSROOT` 期望指向 `nightly-2024-12-07-aarch64-apple-darwin` toolchain 目录
- 用户可通过修改 wrapper 适配其他平台：
  - Linux：`DYLD_LIBRARY_PATH` 改为 `LD_LIBRARY_PATH`，`TS_ROCQ_OF_RUST_TOOLCHAIN_SYSROOT` 改指向对应 Linux toolchain（如 `nightly-2024-12-07-x86_64-unknown-linux-gnu`）
  - macOS x86_64：`DYLD_LIBRARY_PATH` 不变，`TS_ROCQ_OF_RUST_TOOLCHAIN_SYSROOT` 改指向 `nightly-2024-12-07-x86_64-apple-darwin`

未在 Linux / Windows / macOS x86_64 上测试。

## 关联 sub-tests

本工具未派生限制集 agent，无 `examples/rocq-of-rust-limit/`。

翻译成功（且 `.v` 文件无占位注释）的样例预期 SUCCESS；触发静默错误的样例预期 FAILED（grep 将 exit 0 翻转为非 0）。
