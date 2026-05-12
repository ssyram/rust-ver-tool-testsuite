# prusti — 特性支持评估报告（v6 baseline）

## 元数据

- **数据源**：`runs/run-1778560393-59119/`（2026-05-12 v6 final，合并 verus rerun + R7 5-tool rerun；prusti 一次跑完，不在 R7 rerun 集内）
- **工具配置**：`tools/prusti/`（`tool.toml` + `prusti-strict-wrapper.sh` + README）
- **工具版本**：Prusti 0.2.2，commit `a0681ee`（2023-08-22 上游 release tag `v-2023-08-22-1715`），rustc `1.73.0-nightly (180dffba1 2023-08-14)`，pin 在 `nightly-2023-08-15-x86_64-apple-darwin`，整工具链通过 `arch -x86_64` 经 Rosetta 跑 x86_64 binary + x86_64 JDK 17 (Temurin 17.0.19+10) 经 JNI 调 Viper。`results.json` `tools[]` 的 `version` 字段含一行 `thread 'main' panicked at prusti-launch/src/bin/prusti-rustc.rs:35:51:`——`version_command` 在矩阵开始前 sanity check 时打的，prusti-rustc 单跑 `--version` 没设全 5 个 env 会 panic，但**矩阵主跑里**每个 entry 用全 env 经 cargo-prusti，**不受此 sanity panic 影响**；建议未来把 `version_command` 改成 `cargo-prusti --version`。
- **本工具实测**：n=161 / SUCCESS=71 / FAILED=90 / UNKNOWN=0，通过率 **44.10%**
- **时长分布**：avg 10942 ms / median 9649 ms / p90 18439 ms / max 48618 ms（timeout 上限 900 s 远未触达）
- **宪法 baseline**：`principles.md` v8（P27 修宪后 + P31 法律传导后）
- **时效声明**：本快照锚定上述 run id + Prusti 0.2.2 commit `a0681ee` + nightly-2023-08-15 + JDK 17 + corpus（v6 161 entries），不构成长期承诺。Prusti 上游不再积极维护此 commit（团队已转向新一代 Rust verifier 工作），本快照随上游迭代失效。

## pipeline + 前端边界

```
cargo-prusti（cargo wrapper）
  → prusti-rustc（rustc + prusti-driver plugin）
  → rustc parse + macro 展开 + type-check + borrow check
  → MIR construction
  → prusti CollectPrustiSpecVisitor 收集所有 fn item
  → Encoder::process_encoding_queue（MIR → Viper VIR → Viper AST）
  → 通过 JNI + JDK 把 VIR 序列化为 .vpr 文件
  → [PRUSTI_PRINT_HASH=true cut here]
  → new_viper_verifier() → Silicon JVM verifier → Z3 SMT
```

**前端 / 后端切割**（语义对应 commit `a0681ee` 的 `prusti-utils/src/config.rs` 与 `prusti-server/src/process_verification.rs`）：

| flag | 作用 |
|------|------|
| `PRUSTI_NO_VERIFY=false` | 进入 `verify(env, def_spec)` 路径；触发 `Encoder::process_encoding_queue`。`CollectPrustiSpecVisitor` 默认收集所有 fn item，**无需** entry 加 `use prusti_contracts::*` 或 `#[ensures]` |
| `PRUSTI_DUMP_VIPER_PROGRAM=true` | encoder 完成后把 Viper program 写到 `target/verify/log/viper_program/<crate_module>--<fn>-Both.vpr` |
| `PRUSTI_PRINT_HASH=true` | `process_verification_request` 在 dump 之后、`new_viper_verifier()` 之前直接 `return Success`。**Silicon/Z3 永不启动**（无 JVM verifier 实例化、无 SMT 进程） |

5 个 env 缺一不可（PATH / CARGO / RUSTC / RUSTUP_TOOLCHAIN / JAVA_HOME），`arch -x86_64` 让整进程在 Rosetta 下跑。`entry_mode = "bin"`（默认）。

**前端是什么 / 后端是什么**：本测试关心 encoder 是否把 entry 函数 lower 到合法 Viper VIR 并写出 `.vpr`；不关心 Silicon 能否求得 verification verdict。这是宪法 §六"前端测量"在 prusti 上的具体实例化。

**项目维护边界**：`prusti-strict-wrapper.sh` 是项目自家脚本（属"我们 wrapper"）；其余 `cargo-prusti` → `prusti-rustc` → `prusti-driver` 都是上游官方二进制。

## SUCCESS 信号 + 形式严格性

按宪法 §六 双通路 partial 暴露：

- **主信号通路**：`cargo-prusti` 退出码 0（encoder 完整跑过且无 `[Prusti: unsupported feature]` / `[Prusti: internal error]` 报告）
- **wrapper 补抓通路**：`prusti-strict-wrapper.sh` post-check `target/verify/log/viper_program/` 下 `.vpr` 文件数 ≥ 1，否则改写 exit 1 → FAILED

**判定式**：

```
SUCCESS ⟺ cargo-prusti exit 0  ∧  find target/verify/log/viper_program -name '*.vpr' | wc -l ≥ 1
```

**形式严格性 0 误报 / 0 漏报状态**：

- **0 误报**：实测 + 形式可证。Reject 条件 `(exit 0 ∧ 0 .vpr)` 在 commit `a0681ee` + NEW config 下构造性不可达——`PRUSTI_DUMP_VIPER_PROGRAM=true` 是 unconditional dump 站点，`PRUSTI_PRINT_HASH=true` 在 dump 之后才 short-circuit Success；任何 exit 0 路径必经 dump → 必有 ≥ 1 .vpr。
- **0 漏报**：实测 + wrapper 双通路。Prusti unsupported feature → `[Prusti: ...]` marker + exit ≠ 0；internal error / closure ICE → exit ≠ 0；commit drift 下若 encoder fast-path 跳 lower 但仍 exit 0 → wrapper 检 .vpr 数为 0 → FAILED。
- **漏报盲点**：理论窗口存在——encoder 内部对 *单个 fn item* silent skip 但 *仍写出非空 .vpr*（其他 fn 写了 .vpr），目前测试集未触发任何 entry。实测下未观察到。

按 P27 后宪法严格语义评估：这是"实测 + wrapper 双通路封堵"的强声明，但并不等价于"源码层证明"——commit `a0681ee` 的源码读分析只覆盖 dump 站点 + short-circuit 站点；CollectPrustiSpecVisitor 内部对个别 fn item 的 silent skip 路径未做穷举源码审计，故保留上述盲点声明。

## 失败分桶（按 P31 §四.5 归因分类）

90 条 FAILED 全部按主导 stderr 信号清洁分桶（互斥分类，5 桶加总 68 + 8 + 7 + 5 + 2 = 90 ✓）。

### 桶 A：`[Prusti: unsupported feature]` graceful 拒绝（68 case）

代表 entry：`aeneas-limit/fnmut-closure-unit-return`、`bigint/bigint-conv`、`closure/fn-fnmut/closure_fn`、`gat/*`、`hrtb/*`、`industrial/*`、`iter/*` 等。

stderr 特征：

```
error: [Prusti: unsupported feature] iterators are not fully supported yet
  --> src/lib.rs:XX:YY
```

**A 桶 phrase 频次**（非互斥，单 entry 可同时打多个 phrase）：

| 次数 | phrase（已规范化） |
|---:|---|
| 19 | `unsupported constant <...>` |
| 14 | `iterators are not fully supported yet` |
| 11 | `unsizing X into Y is not supported` |
| 11 | `unsupported type <...>`（含 `Alias(Opaque,...)` async fn / `Binder(fn(...) -> T)` 等）|
| 10 | `cast statements that create loans are not supported` |
| 9  | `access to reference-typed fields is not supported` |
| 6  | `unsupported cast from <T1> to <T2>` |
| 5  | `higher-ranked lifetimes and types are not supported` |
| 4  | `determining the region of a dereferentiation is not supported` |
| 4  | `unsupported const kind <...>` |
| 3  | `array indexing is not supported in arbitrary operand positions` |
| 3  | `only calls to closures are supported. The term is a F/#0, not a closure` |
| 3  | `unions are not supported` |
| 2  | `raw pointers are not supported` |
| 2  | `references to thread-local storage are not supported` |
| 2  | `unsupported creation of shallow borrows (implicitly created when lowering matches)` |
| 2  | `bitwise operations on non-boolean types are experimental and disabled by default` |
| 2  | `casts IntToFloat are not supported` |
| 2  | `raw addresses of expressions or casting a reference to a raw pointer are not supported` |
| 1+ | `casts FloatToFloat are not supported` 等单点拒绝 |

**归因**：工具不支持（Prusti encoder 自陈的 MIR → Viper 翻译边界）。
**处理**：**不修**。最大善意已尽（按上游推荐姿势 + 上游锚定的 nightly + 上游 release artefact 装）；本地性原则下 FAILED 站得住。

### 桶 B：`[Prusti: internal error]` fold-unfold permission 失败（8 case）

代表 entry：`aeneas-limit/closure-if-capture`、`charon-limit/copy-deref-closure/deref_copy_in_closure`、`closure/fn-fnmut/closure_fn`、`closure/fn-fnmut/closure_fnmut`、`hax-limit/mut-in-assoc-type` 等（8 条均见 fold-unfold 子串）。

stderr 特征：

```
error: [Prusti: internal error] Prusti encountered an unexpected internal error
  = note: This is likely to be a bug in Prusti. ...
  = note: Details: cannot generate fold-unfold Viper statements. The required
          permission Pred(_X.closure_N.val_ref, write/read) cannot be obtained.
```

**归因**：工具不支持（encoder 的 capability lattice 求解在涉及 closure 捕获 / mut-borrow 复合 / 深 deref 路径上失败；与 A 桶同源——上游声明的语言子集约束）。
**处理**：**不修**。这是 prusti 0.2.2 在这类 case 上的设计内 reject 路径（与 A 桶仅 marker 形式不同，归属仍是工具能力边界）。

### 桶 C：cargo manifest edition = 2024 拒绝（7 case）

完整列表：

```
hax-limit/let-chains/hax_limit_let_chains
industrial/rsa/rsa-pkcs8/rsa_pubkey_from_pkcs8
industrial/rsa/rsa-pkcs8/rsa_pkcs1v15_encrypt
industrial/sha2/sha256-digest/sha256_digest_one_shot
industrial/sha2/sha256-digest/sha256_digest_incremental
industrial/x509-parser/cert-parse/x509_parse_der
industrial/x509-parser/cert-parse/x509_subject_extensions
```

stderr 特征：

```
error: failed to parse the `edition` key
this version of Cargo is older than the `2024` edition
```

**归因**：工具不支持。prusti 0.2.2 上游绑定 nightly-2023-08-15 的 cargo，该 cargo 不识别 edition = 2024——这是 prusti 自选 toolchain 不识 stable feature 的典型形态。
**处理**：**不修**（D1 立场）。按 §六 UNKNOWN 严格语义末段："工具自选 toolchain 不支持新特性"明示属工具能力边界，**FAILED**。装其要求的老 nightly 是最大善意的具体表现，最大善意尽到后老 nightly 不读 edition 2024 → 工具能力边界 → FAILED 站得住。industrial vendor 子 crate Cargo.toml 写 edition = "2024" 是其上游 author 的合理选择（这些 crate 在现代 Rust ecosystem 里就该用 2024）；不是我们改的，也不应为了迁就 prusti 改 vendor。

### 桶 D：compiler ICE panic（5 case）

完整列表（**所有 5 条都触及 `core::arch::asm!` inline assembly**）：

```
charon-limit/inline-asm/nop_via_asm
creusot-limit/inline-asm-basic/nop_via_asm
kani-limit/inline-assembly/trigger_add_via_asm
miri-limit/inline-asm/trigger_add_via_inline_asm
prusti-limit/closures-unsupported/trigger_closures_unsupported
```

（前 4 条 `*-limit/inline-asm` 是 inline asm，第 5 条 `prusti-limit/closures-unsupported` 是 closure ICE——prusti 0.2.2 在 `mir_storage.rs:85` panic 而非 graceful reject）

stderr 特征：

```
error: the compiler unexpectedly panicked. this is a bug.
note: we would appreciate a bug report: https://github.com/viperproject/prusti-dev/issues/new
note: please attach the file at <work>/rustc-ice-2026-05-12T*.txt
note: Prusti version: 0.2.2, commit a0681ee 2023-08-22 ...
```

**归因**：工具不支持（这是 prusti 在边界 case 上 graceful reject 路径未覆盖→ ICE，按 §六 反作弊"内部异常等同于 partial"延伸记 FAILED；不是 graceful reject 但属工具自身缺陷）。
**处理**：**不修**。Prusti issue tracker 的事。

### 桶 E：rustc unstable feature gate（2 case）

完整列表：

```
float/round/float_round                  → use of unstable library feature 'round_ties_even'
hax-limit/unsafe-block/hax_limit_unsafe_block → use of unstable library feature 'unchecked_math'
```

stderr 特征：

```
error[E0658]: use of unstable library feature '<name>'
```

**归因**：工具不支持（同 C 桶根因——prusti 锁定 nightly-2023-08-15 比这两个 API 稳定化早；样例代码不加 `#![feature(...)]` 是因为它们在现代 stable 是稳定 API，加 feature gate 反而是污染样例。宪法 §四 原则 A 禁样例为工具改动；nightly-2023-08-15 不读现代 stable feature 是工具能力边界）。
**处理**：**不修**（D1 立场，与 C 桶同源——工具自选老 nightly 不识 stable feature）。

## 漏报盲点（诚实声明）

- **已通过 wrapper gate 封堵**：
  - `(exit 0 + 0 .vpr)` 路径（README §"检测条件" 的文字承诺现已通过 `prusti-strict-wrapper.sh` 提升为可执行 oracle）
  - encoder fast-path 跳 lower 但仍 exit 0 的 commit-drift 理论窗口
- **仍存在的盲点**：
  - encoder *内部* 对单个 fn item silent skip 但仍写出非空 `.vpr`（其他 fn 写了 dump）的极端理论窗口——未做源码层穷举证明，实测下 0 现象，但不能宣称"形式可证 0 漏报"
  - 修复 backlog：若未来出现此现象，可加 wrapper 二级 gate"每个 entry fn 名应至少出现一次于 .vpr 文件名中"。当前不预先实施（无证据，不做过度防御）

## v5.1 → v6 ΔS 解释

| 维度 | v5.1（`run-1778238662-69805`）| v6（`run-1778560393-59119`）| Δ |
|---|---|---|---|
| corpus 大小 | 146 | 161 | +15 |
| SUCCESS | 56 | 71 | **+15** |
| FAILED | 90 | 90 | 0 |
| UNKNOWN | 0 | 0 | 0 |
| 通过率 | 38.4% | 44.1% | +5.7 pp |

**ΔS = +15 全部来源于 v6 新增 `examples/runnable/*` 15 entries**（abs / add-two / add-u32 / bool-ops / digit-sum / enum-classify / fact / fib / gcd / max3 / parity / power / saturating / struct-norm / sub-clamped）。这 15 个 entry 都是基础算术 / 布尔 / 简单结构操作，全部落在 prusti encoder 的 well-supported MIR 子集内，故全 SUCCESS。**5 个原失败桶的桶内分布完全不变**（68/8/7/5/2）——这印证 P12 后 prusti 在原 146 entry 上的能力边界稳定。

**P22 prusti viper_tools 修复**（`docs/fixes/prusti-java-env-fix-2026-05-11.md`）：matrix `run-1778492036-50081` 一度 0/161（macOS `/tmp` 周期清理把 `viper_tools/backends/*.jar` 吃掉，JVM 起来后 `AstFactory.backend_bv32_type()` → ClassNotFoundException → unwrap panic 触发 161 entry 全 JavaException FAILED）。修复路径：重装 prusti release + JDK 迁出 `/tmp` 到 `~/.local/share/ts-tools/`（`.env` 更新）。修复后 `run-1778494055-14621` 56/161 → 后续 v6 final `run-1778560393-59119` 71/161（多 15 是 runnable 新增）。**修复后 0 条 JavaException FAILED**——本快照的 90 FAILED 全部是 prusti encoder 真实拒绝。

## 修订建议清单

| # | 桶 | 涉及 case | 修复方案 | 优先级 |
|---|---|---:|---|---|
| — | — | 0 | 无需修订——所有 90 条 FAILED 均为工具能力边界（A 桶 unsupported feature 自陈 / B 桶 internal error 自陈 / C 桶老 nightly cargo 不识 edition 2024 = 工具自选 toolchain 边界 / D 桶 ICE = 工具内部异常 / E 桶 nightly-2023-08-15 不识 stable feature = 工具自选 toolchain 边界）。FAILED 按 §六 严格语义站得住，工具开发者不能驳回。 | — |

**"我们导致"项**：**无**。本快照所有 FAILED 都归"工具不支持"；P22 我们环境层（`/tmp` 清理）一次性事故已治源迁至 `~/.local/share/ts-tools/prusti/` 与 `~/.local/share/ts-tools/jdk-x64/`，本快照里不再现身。`prusti-strict-wrapper.sh` 在本快照中**未触发 reject**（71 SUCCESS 全有 ≥ 1 `.vpr`；冗余防御层成立但本快照里没改变任何 verdict）。

**次要遗留**：`tool.toml` `version_command` 用 `prusti-rustc --version` sanity 时会 panic（见 §元数据），不影响矩阵主跑但污染了 `results.json` `tools[].version`。建议改成 `cargo-prusti --version` 或一个完整 env 包裹的 `prusti-rustc --version`。**优先级**：低（不影响 verdict 正确性，仅影响版本字符串可读性）。
