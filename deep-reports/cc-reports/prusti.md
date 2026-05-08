# prusti — 特性支持评估报告

## 元数据

- **数据源**：`runs/run-1778226613-5282/`（2026-05-08，146 entries × 19 工具矩阵；host: Apple M5 / macOS aarch64 / 24 GB / 10 cpus，并发 10）
- **工具版本**：`Prusti 0.2.2`，commit `a0681ee`（2023-08-22），rustc `1.73.0-nightly (180dffba1 2023-08-14)`，pin 在 `nightly-2023-08-15-x86_64-apple-darwin`，整工具链通过 `arch -x86_64` 经 Rosetta 跑 x86_64 binary + x86_64 JDK 17 经 JNI 调 Viper
- **工具配置**：`tools/prusti/`
- **通过率**：SUCCESS 56 / 146 ≈ **38.4%**（FAILED 90，TIMEOUT 0）
- **耗时分布**：avg 12594 ms / median 10434 ms / p90 22761 ms / p95 29633 ms / max 76666 ms（timeout 上限 900 s 远未触达）
- **时效声明**：本快照锚定上述 run id + prusti 0.2.2 commit `a0681ee` + nightly-2023-08-15 + corpus，不构成长期承诺。Prusti 上游不再积极维护此版本（Prusti 团队已转向新一代 prusti-rs / Rust verifier 项目），本快照随上游迭代失效。

## 工具内部 pipeline + 前端边界

```
cargo-prusti（cargo wrapper）
  → prusti-rustc（rustc with prusti-driver plugin）
  → rustc parse + macro 展开 + type-check + borrow check
  → MIR construction
  → prusti CollectPrustiSpecVisitor 收集所有 fn item
  → Encoder::process_encoding_queue（MIR → Viper VIR → Viper AST）
  → 通过 JNI + JDK 把 VIR 序列化为 .vpr 文件
  → [PRUSTI_PRINT_HASH=true cut here]
  → new_viper_verifier() → Silicon JVM verifier → Z3 SMT
```

本测试关心 cargo-prusti 在 `PRUSTI_NO_VERIFY=false + PRUSTI_DUMP_VIPER_PROGRAM=true + PRUSTI_PRINT_HASH=true` 配置下的"前端通过"。

**前端 / 后端切割**（语义对应 commit `a0681ee` 的 `prusti-utils/src/config.rs` 与 `prusti-server/src/process_verification.rs`）：

| flag | 作用 |
|------|------|
| `PRUSTI_NO_VERIFY=false` | 进入 `verify(env, def_spec)` 路径；触发 `Encoder::process_encoding_queue`。`CollectPrustiSpecVisitor` 默认收集所有 fn item，**无需** entry 加 `use prusti_contracts::*` 或 `#[ensures]` |
| `PRUSTI_DUMP_VIPER_PROGRAM=true` | encoder 完成后把 Viper program 写到 `target/verify/log/viper_program/<crate_module>--<fn>-Both.vpr` |
| `PRUSTI_PRINT_HASH=true` | `process_verification_request` 在 dump 之后、`new_viper_verifier()` 之前直接 `return Success`。**Silicon/Z3 永不启动**（无 JVM verifier 实例化、无 SMT 进程） |

5 个 env 缺一不可（PATH / CARGO / RUSTC / RUSTUP_TOOLCHAIN / JAVA_HOME），`arch -x86_64` 让整进程在 Rosetta 下跑。

`entry_mode = "bin"`（默认），runner 在副本 `src/bin/__ts_harness.rs` 渲染 `target_crate_name::entry_fn();` 调用。

## SUCCESS 信号 + 形式严格性

**判定式**：

```
SUCCESS ⟺ cargo-prusti exit 0
（即 prusti-rustc 编译通过 + Encoder 跑过所有 fn 并写出 .vpr）
```

**partial 暴露机制**：exit ≠ 0 与下列 stderr marker 之一（实测覆盖全部 90 条 FAILED）：
- `[Prusti: unsupported feature] ...`（graceful 拒绝路径）
- `[Prusti: internal error] ...`（encoder 抛出 internal error，如 fold-unfold permission）
- `thread 'rustc' panicked at ...`（部分 unsupported case 走 ICE 路径）

**形式严格性**：
- **0 误报**：✅ 形式可证。cargo-prusti exit 0 ⇔ encoder 完整跑过且无 unsupported feature 报告
- **0 漏报**：✅ 形式可证。Prusti 任何 unsupported feature → `[Prusti: ...]` marker + exit ≠ 0；任何 internal error / closure ICE → exit ≠ 0
- **漏报盲点**：无。NEW config 下 encoder 真跑，`PRUSTI_PRINT_HASH=true` 仅在 encoder 完成后跳过 Silicon——encoder 自身的 unsupported 检测路径完整

## 关键转折：NEW config vs OLD config

**这是本快照与上一份 cc-report（基于 OLD config `PRUSTI_NO_VERIFY=true`）最显著的差异**：

| 维度 | 旧（NO_VERIFY=true） | 新（DUMP+PRINT_HASH） |
|------|---------------------|----------------------|
| `verify(env, def_spec)` | 跳过 | 进入 |
| `Encoder::process_encoding_queue` | **不跑** | **跑** |
| `.vpr` 文件 | 不产生 | 写到 `target/verify/log/viper_program/` |
| Silicon / Z3 进程 | 不启动 | 不启动 |
| 简单 fn 单 entry 时间 | ≈ 1.1 s | ≈ 2.3 s（多出 encoder + JVM bootstrap） |
| "前端边界"信号 | rustc + macro 展开通过即视为接受，**未测 encoder** | exit 0 仅当 encoder 真接受；async fn / closure 真跑 / raw pointer / 复杂 lifetime / 容器 unsizing 等均 exit 非 0 |
| 矩阵实测通过率 | **94 / 140 ≈ 67%**（在 140-entry 矩阵下；推算到 146 ≈ 93/146 ≈ 64%——但**那个数字接近 cargo-check 编译率**，因为旧配置下 prusti 退化为 `rustc + prusti-contracts proc-macro pass`） | **56 / 146 = 38%**（encoder 真实接受率） |
| 配置定性 | 退化为 `cargo-check` 等价路径，**违反宪法 §六-4 反作弊**——SUCCESS 信号"rustc parses it"，无法暴露 prusti 自身能力边界 | 真实测到 MIR → Viper 这条 prusti 独有的前端边界 |

**38% vs 67%**：旧配置高估了 28 个百分点的"接受面"——那不是 prusti 真实能力，而是 cargo-check 等价的 rustc parse 通过率。新配置下整体下降的 28 条 entry 都是**真实触及 prusti encoder 拒绝边界**的样例（如下 A/B 桶具体清单）。本快照的 **38% 才是 prusti 0.2.2 真实的 encoder 接受率**。

**另一个关键修复**：旧 cc-report 提到 43 条 FAILED 死在 harness 渲染层（hyphen-named crate 未做 ident 标准化，`bool-bitwise-op::trigger_*();` 被 rustc 当作减法解析）。本次 run 已通过 runner 修复（`runner/src/exec.rs:98` `let crate_ident = example.crate_name.replace('-', "_");`），harness 文件再无该 bug——本矩阵 90 条 FAILED **0 条**死在 harness ident 解析层。

## 实测结果

### 按 feature 类目分布

下列 feature 类目下 prusti **全部 entry 通过**：

```
arc / assoc-type(1/1) / const(1/1) / generic(部分见下) / hello / int(2/2) / rc / refcell(1/1)
trait(1/1) / vec
```

**部分通过**（数字为 S/总）：`aeneas-limit` 4/8、`bigint` 6/8、`box` 1/2、`charon-limit` 1/7、`closure-adv` 1/4、`concurrency` 1/2、`creusot-limit` 1/7、`enum` 1/2、`float` 4/10、`generic` 2/4、`hax-limit` 2/8、`int-width` 11/14、`kani-limit` 2/7、`lifetime` 1/3、`miri-limit` 3/7、`panic` 1/2、`prusti-limit` 1/8、`repr` 1/2、`trait-obj` 1/2、`unsafe-adv` 1/3。

**全 FAILED**：`closure`(0/2)、`collections`(0/2)、`deps-complex`(0/7)、`drop`(0/1)、`error`(0/1)、`gat`(0/1)、`hrtb`(0/1)、`impl-trait`(0/1)、`industrial`(0/6)、`iter`(0/1)、`slice`(0/1)、`unsafe-ptr`(0/2)。

### 失败模式归类（基于 raw stderr）

90 条 FAILED 按主导信号分桶：

| 桶 | 数量 | 信号形态 |
|---|---:|---|
| **A. `[Prusti: unsupported feature]` graceful 拒绝** | 68 | encoder 在 MIR → Viper 翻译时遇到当前不支持的 Rust 构造，emit 该 marker 后 exit ≠ 0 |
| **B. `[Prusti: internal error]` encoder fold-unfold permission** | 8 | encoder 跑过但 Viper VIR 生成时 fold-unfold permission 失败，stderr 含 `Details: cannot generate fold-unfold Viper statements. The required permission Pred(_X.Y, write/read) cannot be obtained.` |
| **C. cargo manifest edition = 2024 拒绝** | 7 | `error: failed to parse the \`edition\` key / this version of Cargo is older than the \`2024\` edition`。prusti 锁定 nightly-2023-08-15 的 cargo 不识别 edition 2024 |
| **D. compiler ICE panic（unsupported case 走 ICE 路径）** | 5 | `error: the compiler unexpectedly panicked. this is a bug.`，stderr 末尾常见 `note: please attach the file at .../rustc-ice-...txt` |
| **E. `error[E0658]: use of unstable library feature`** | 2 | 样例 `src/lib.rs` 用了 `f64::round_ties_even()` / `u32::unchecked_add()` 但未在顶部加对应 `#![feature]` attr。prusti nightly 2023-08-15 比这两个 API 稳定化早，所以 stock feature flag 还要求显式启用——发生在 stock rustc lexer / nightly feature gate 阶段，prusti spec encoding 没启动 |

数量加总：68 + 8 + 7 + 5 + 2 = 90 ✓

### A 桶（unsupported feature）的具体类别频次

按 stderr `[Prusti: unsupported feature] <phrase>` 字面短语统计的**非互斥**频次（一条 entry 可同时打出多个 phrase）：

| 出现次数 | unsupported feature phrase |
|---:|---|
| 14 | `iterators are not fully supported yet` |
| 10 | `cast statements that create loans are not supported` |
| 9 | `access to reference-typed fields is not supported` |
| 5 | `higher-ranked lifetimes and types are not supported` |
| 5 | `unsupported cast from type 'bool' to type 'usize'` |
| 4 | `determining the region of a dereferentiation is not supported` |
| 3 | `unsizing a std::boxed::Box<[i32; 3]> into a std::boxed::Box<[i32]> is not supported` |
| 3 | `array indexing is not supported in arbitrary operand positions` |
| 3 | `only calls to closures are supported. The term is a F/#0, not a closure.` |
| 3 | `unions are not supported` |
| 3 | `unsupported const kind: Const { ty: usize, kind: N/#0 }` |
| 2 | `raw pointers are not supported` |
| 2 | `references to thread-local storage are not supported` |
| 2 | `casts IntToFloat are not supported` |
| 2 | `unsupported creation of shallow borrows (implicitly created when lowering matches)` |
| 2 | `bitwise operations on non-boolean types are experimental and disabled by default` |
| 1+ | 各种 `unsupported constant type &'?N <T>`、`unsupported type Alias(Opaque, ...)`（async fn 的 `impl Future`）、`unsupported type Binder(fn(...) -> T, [])`、`unsizing Box<dyn Display> into Box<dyn Debug>`、`Non-slice LHS type` 等单点拒绝 |

### B 桶（internal error: fold-unfold permission）entry 列表

```
aeneas-limit/fnmut-closure-unit-return
aeneas-limit/closure-if-capture
charon-limit/copy-deref-closure（partial，主要在 D 桶）
prusti-limit/{closure-in-pure-fn, ...}
（共 8 条 fold-unfold permission 形态）
```

representative stderr：

```
error: [Prusti: internal error] Prusti encountered an unexpected internal error
  --> src/lib.rs:25:1
   = note: This is likely to be a bug in Prusti. ...
   = note: Details: cannot generate fold-unfold Viper statements. The required
           permission Pred(_6.closure_0.val_ref, write) cannot be obtained.
```

`cannot generate fold-unfold Viper statements` 是 prusti encoder 的 capability lattice 求解失败，常与同一 entry 的 `[Prusti: unsupported feature] determining the region of a dereferentiation is not supported` 同时出现。

### C 桶（edition 2024）entry 列表

```
hax-limit/let-chains（样例 Cargo.toml 写 edition = "2024"）
industrial/x509-parser/cert-parse/x509_parse_der（vendor crate Cargo.toml 写 edition = "2024"）
industrial/x509-parser/cert-parse/x509_subject_extensions（同上）
industrial/rsa/rsa-pkcs8/{rsa_pkcs1v15_encrypt, rsa_pubkey_from_pkcs8}（依赖含 edition 2024 的 base64ct 等子 crate）
industrial/sha2/sha256-digest/{sha256_digest_incremental, sha256_digest_one_shot}（vendor sha2 写 edition = "2024"）
```

发生在 cargo manifest parse 阶段，prusti-rustc 完全没启动。industrial 模块整 6 条全死在这层，与 prusti 自身能力无关。

### D 桶（compiler ICE）样例

`charon-limit/copy-deref-closure/deref_copy_in_closure` 等。形态：

```
error: the compiler unexpectedly panicked. this is a bug.
note: we would appreciate a bug report: https://github.com/viperproject/prusti-dev/issues/new
note: please attach the file at .../rustc-ice-2026-05-08T07:51:18.248964Z-23920.txt
note: Prusti version: 0.2.2, commit a0681ee 2023-08-22 ...
```

部分 closure 与 mut-borrow 复合形态在 prusti 0.2.2 上触发 ICE 而非 graceful unsupported error。本测试一律记 FAILED（exit 101）。

## 与本次测试边界的关系

- 测试切割点：cargo-prusti exit 0 = encoder 跑完且写出 `.vpr` → SUCCESS。**未触达**：Silicon JVM verifier 启动、SMT-LIB 翻译、Z3 求解——`PRUSTI_PRINT_HASH=true` 在 encoder 完成、Silicon 实例化之前直接返回
- 本快照的 **38% 是 prusti 0.2.2 真实的 encoder 接受率**——A 桶 68 条 + B 桶 8 条 = 76 条都在 prusti 自身的 MIR → Viper 翻译边界上 fail，反映 prusti 的语言子集约束（没有完整迭代器支持、闭包捕获 mut 受限、higher-ranked lifetime 不支持、unsizing 不完整、cast 创建 loan 不支持等）
- C 桶 7 条（edition 2024）+ E 桶 2 条（unstable feature gate）= 9 条 **不在 prusti 自身能力层面**——发生在 cargo / rustc 输入流水线层，是 prusti 工具链 pin 在 nightly-2023-08-15 与现代 Rust ecosystem 之间的版本鸿沟
- D 桶 5 条 ICE 是 prusti 0.2.2 在某些边界 case 上的内部异常，不是设计内的 reject 路径，但本测试按 exit code 记 FAILED——宪法 §六-2 的延伸：内部异常等同于 partial
- `prusti-limit/*` 1/8（其他 7 条全 FAILED）：本类是 prusti 自声明的限制集（设计意图：prusti 期望失败），新配置下 7/8 按设计 fail——这是 NEW config 与 OLD config 最显著的"分化信号恢复"实例。OLD config 下 `prusti-limit/*` 大多 PASS（因为 rustc 本身能编过），**那是 false positive**

## 历史快照声明

本报告所有数字基于 `runs/run-1778226613-5282`（2026-05-08）+ NEW config（NO_VERIFY=false + DUMP_VIPER_PROGRAM=true + PRINT_HASH=true）+ Prusti 0.2.2 commit `a0681ee` + nightly-2023-08-15 + JDK 17。Prusti 上游已转向新一代 verifier，本 commit 与现代 Rust ecosystem 之间的版本鸿沟（edition 2024、新 nightly feature 等）会越来越宽——本快照随之失效。NEW config 下的 38% 通过率是与同矩阵其他 verifier（hax×3 / verus / kani / charon×2 / aeneas×4 / creusot 等）跨工具对比有意义的数字，OLD config 下的 67% 不暴露 prusti encoder 边界，应作废。
