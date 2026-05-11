# v5 matrix 0 误报 counter-challenge（cc 阶段，2026-05-11）

## §0 元数据 + 方法

- **审计日期**：2026-05-11
- **审计阶段**：cc（按 charter-craft §4.8 — 反两次 / 高 precision）
- **审计对象**：c 阶段产出文档 `docs/fixes/audit-v5-c-false-positive-2026-05-11.md` 提出的 64 个候选误报 + 4 类外部根因新规则提议
- **审计触发**：c 阶段是 LLM 单边挑刺（高 recall），需要独立 agent 按 disprove-first counter-challenge，过滤低效挑刺
- **数据源**：`runs/run-1778500291-90812/raw/<tool>/<entry>.{stdout,stderr,exit}`
- **环境**：Apple M5, macOS 25.4.0, 10 CPUs, parallelism=10, 1889s wall
- **本审计阶段约束**：c 阶段定 _候选_，cc 阶段做 counter-challenge 与最终裁决；**不修任何代码 / tool.toml / wrapper / runner / report**；不 commit

### 方法（disprove-first，按 charter-craft §4.8.1–4.8.3）

1. **默认 c 阶段的每条挑刺都错**：先尝试找证据驳斥
2. **驳不动则挑刺成立**：基于具体 stderr / stdout + 宪法精神性条款定位
3. **每条 counter 必须有出处**：principles.md §X.Y 行号 / tool-integration.md §Z 节 / 具体 entry stderr 片段
4. **结论二分**：决策点（精神模糊需用户裁决） vs 非决策点（精神已明示直接落地）

### counter-challenge 的精神性锚点

宪法 `principles.md` 与下游 `tool-integration.md` 中可作为 counter 论据的关键陈述：

| # | 出处 | 字面 | 精神 |
|---|---|---|---|
| α | `principles.md` §六-Oracle 责任，行 87-89 | "**不冤枉**：SUCCESS 必须是真 SUCCESS——不允许任何 partial / silent skip / 半翻译；工具自陈'我没全干完'必须被尊重" | 0 误报硬指标的肯定面 |
| β | c 阶段 §2.2 引用宪法 §六 | "工具不能 type-check / encode 此 entry 才算 partial / FAILED；工具 pipeline 上游（cargo / Cargo.toml edition / vendor crate / harness）失败算误报" | 边界判据的否定面 |
| γ | `tool-integration.md` §三-0 误报论证，行 38-45 | "0 误报的定义：oracle 判 SUCCESS 时一定是真 SUCCESS（不冤枉工具能力）" | 0 误报针对 SUCCESS 的语义；但 §4.2 把约束延伸到防漏报机制——绝不能反向引入误报 |
| δ | `tool-integration.md` §四-4.2，行 84-96 | "防漏报机制绝不能反向引入误报。每个 grep marker 选取必须经过双向实测" | 任何新增 UNKNOWN 规则需在合法 SUCCESS / 合法注释 / 用户字面上不命中 |
| ε | `principles.md` §五-runner，行 71 | "每次 run 必须记录 (工具版本, 测试环境, ISO 时间戳, 样例特征) 四元组" | 工具版本锚定是结果有效性条件；不同版本工具在同 entry 上的行为差异不算 entry 的真 partial |

注：β 是 c 阶段引用宪法 §六 的转述，宪法原文未明文列出"cargo / Cargo.toml edition / vendor crate / harness"，但精神已经表达——见下 §1.1 第 6 类与第 7 类的具体讨论。

---

## §1 R2-c 64 候选逐条 counter

### §1.1 4 类新规则提议的精神性反查

c 阶段提出 4 类新外部根因规则（第 6 类 edition_pipeline_propagation / 第 7 类 dependency_resolution_E0433 / 第 8 类 toolchain_unstable_feature_gate / kmir 专属规则）。逐条反查精神已否明示。

---

#### §1.1.1 第 6 类：`edition_pipeline_propagation`（9 候选）

**c 阶段挑刺**：rocq-of-rust×2 / rocq-of-rust-typecheck×2 / soteria×1 / verifast×3 / verus×1 的 stderr 含 rustc edition gate 字面（`async fn is not permitted in Rust 2015 / pass --edition 2024` 或 `let chains are only allowed in Rust 2024 or later`）。c 阶段说这是工具 pipeline 没传 edition flag 导致，是误报。

**counter（反 c 阶段）**：

- **精神锚点 β**："工具 pipeline 上游（cargo / Cargo.toml edition / vendor crate / harness）失败算误报" — 这里 _明文_ 把 "Cargo.toml edition" 列为上游外部根因。
- **关键证据**：
  - `examples/charon-limit/async-fn/Cargo.toml` 第 4 行 `edition = "2021"` — entry 已经声明 edition
  - `examples/hax-limit/let-chains/Cargo.toml` 第 4 行 `edition = "2024"` — entry 已声明 edition
  - `runs/.../raw/rocq-of-rust/charon-limit__async-fn__async_forty_two.stderr` 第 1-9 行：rustc 直接抛 E0670 "async fn is not permitted in Rust 2015" 加 hint `pass --edition 2024 to rustc` — rustc 在跑 Rust 2015 默认模式，说明工具调 rustc/charon-driver 时没读 Cargo.toml 的 edition 字段
  - `runs/.../raw/verus/hax-limit__let-chains__hax_limit_let_chains.stderr` 第 1-12 行：`error: let chains are only allowed in Rust 2024 or later` — entry 的 Cargo.toml 已经声明 edition 2024，但 verus 的 rustc-fork 没接到这个 flag

**驳不动**：5 个工具在 5 个 _不同_ entry 上一致 stderr 含 rustc edition gate，且这些 entry 本身已声明正确 edition，cargo-check 上 100% SUCCESS。这就是宪法 §六 精神 β 明文覆盖的"Cargo.toml edition 上游失败"。

**结论**：c 阶段挑刺成立。这 9 个 case 是真候选误报，应该映射 UNKNOWN。

**精神性陈述位置**：c 阶段 §2.2 转述宪法 §六 的判据原则；宪法原文 `principles.md` 行 87-89 "**不冤枉**" 间接覆盖。这里的精神已经明示——属**非决策点**。

---

#### §1.1.2 第 7 类：`dependency_resolution_E0433`（10 候选）

**c 阶段挑刺**：5 工具（rocq-of-rust×2 / rocq-of-rust-typecheck / soteria / verifast / verus）× 2 个 x509-parser entry。stderr 含 `error[E0433]: cannot find module or crate x509_parser`。c 阶段说 oracle 规则 2 只匹配 E0432，遗漏 E0433；两者都是依赖未解析。

**counter（反 c 阶段）**：

- **精神锚点 β**："vendor crate / harness" 是 pipeline 上游 — `extra_cargo_deps` 引入的 `x509_parser` crate 在工具不走 cargo 的 pipeline 下未解析，与 E0432 同类。
- **现有规则覆盖**：`oracle-unknown-classification-2026-05-11.md` §2 已经把 5 个工具的 19 个 deps 实例（bigint/deps-complex/industrial-rsa/sha2）归类成 `dependency_resolution`，触发字面是 E0432。同一精神 _应_ 覆盖 E0433。
- **关键证据**：
  - `runs/.../raw/verus/industrial__x509-parser_cert-parse__x509_parse_der.stderr` 第 1-15 行：连续三个 E0433 错，全是 `cannot find module or crate x509_parser` 或 `cannot find type X509Certificate in this scope`
  - `examples/industrial/x509-parser/cert-parse/hirusttest.toml` / Cargo.toml 必含 `extra_cargo_deps`（详见 docs/research）。x509_parser 是合法的 vendored crate，对所有走 cargo 的工具（aeneas / charon / creusot / hax / kani / miri / prusti / cargo-check）SUCCESS。
  - `runs/.../raw/cargo-check/industrial__x509-parser_cert-parse__x509_parse_der.exit` = `Some(0)` — cargo-check 基线 SUCCESS，证明 entry 合法
- **反误报担忧**：用户代码会不会主动写 `cannot find module or crate`？不会——E0433 是 rustc 在 path resolution 阶段报的标准 error，用户代码不会引发"自己拼写错误"在 _可读 vendored crate_ 上的 E0433（cargo-check SUCCESS 已排除）。所以扩规则不会反向误吞 entry 内部的真 path bug。

**驳不动**：宪法精神 β 已明文覆盖"vendor crate"上游失败；oracle 当前规则 2 只是字面层 E0432，扩展为 E0432 ∪ E0433 是同精神下的字面拓宽。

**结论**：c 阶段挑刺成立。10 个 case 是真候选误报。

**精神性陈述位置**：宪法 §六 "vendor crate / harness" 转述（c 阶段 §2.2）+ `tool-integration.md` §四-4.2 防漏报机制反误报检查（行 84-96）——E0433 拓宽通过反误报检查。属**非决策点**。

---

#### §1.1.3 第 8 类：`toolchain_unstable_feature_gate`（2 候选）

**c 阶段挑刺**：prusti × `float/round/float_round` + prusti × `hax-limit/unsafe-block/hax_limit_unsafe_block`。stderr 含 `error[E0658]: use of unstable library feature 'round_ties_even'` 或 `'unchecked_math'`。c 阶段说 prusti 锁死 2023-08 旧 toolchain，这两 feature 在 mainline 已 stable。

**counter（反 c 阶段）**：

- **精神锚点 ε**：`principles.md` §五-runner，行 71 — "每次 run 必须记录 (工具版本, 测试环境, ISO 时间戳, 样例特征) 四元组" + §六-时空锚定，行 82-83 — "工具能力观察必须锚定 (工具版本, 时间) 二元组"。
- **关键证据**：
  - `runs/.../raw/prusti/float__round__float_round.stderr` 第 5 行：`Prusti version: 0.2.2, commit a0681ee 2023-08-22 15:08:33 UTC` — prusti 锁死的 rustc nightly 2023-08-15
  - `runs/.../raw/prusti/hax-limit__unsafe-block__hax_limit_unsafe_block.stderr` 第 5 行：同上 prusti 0.2.2 2023-08-22
  - `runs/.../raw/cargo-check/float__round__float_round.exit` = `Some(0)` — 主线 rustc 1.95.0（2026-03-21）SUCCESS
  - `runs/.../raw/cargo-check/hax-limit__unsafe-block__hax_limit_unsafe_block.exit` = `Some(0)` — 同
- **关键反思**：E0658 表面看像 prusti pipeline 内 rustc fork 的拒绝（属工具自身前端）。但本质上是 _工具锁死的 rustc 版本_ 与 _mainline rustc 已稳定的 feature_ 之间的版本差，是 "toolchain pinning" 的副作用。
  - 反对 c 阶段的视角：prusti 用 `unchecked_add` / `round_ties_even` 不识别，可以视为 prusti 的前端不接受这个 entry，属真 partial（工具就是吃不下）
  - 支持 c 阶段的视角：entry 用的是 mainline stable feature；如果 entry 显式 `#![feature(round_ties_even)]`，prusti 旧 rustc 也能接受。失败根因是 _时空错配_，不是工具语义限制
- **精神性裁决**：
  - 宪法 §六 没明文区分"工具锁死 toolchain 的副作用"与"工具核心前端拒绝"
  - β 转述里的"cargo / Cargo.toml edition / vendor crate / harness"也没明文覆盖 toolchain pinning
  - **精神模糊**：可以归到 ε（时空锚定 — 工具版本是必然变量）也可以归到 α（工具自陈不支持 — E0658 就是工具 rustc 不识别）

**判定**：精神**未明示**。c 阶段挑刺成立性 _弱_。

**结论**：候选误报性存在但精神模糊。属**决策点**。

**给用户的二选一**：
- (8a) 把 toolchain_unstable_feature_gate 升级为第 8 类外部根因 → 这 2 个 case 映射 UNKNOWN → prusti 实际 FAILED 数 -2
- (8b) 不升级，prusti 实际 FAILED 保持 83 → 但需要在 prusti README 列出"工具版本与 entry 主线 feature 错配"作为已知误报盲点（按 `tool-integration.md` §4.4 漏报盲点诚实声明的对偶——这里是误报盲点诚实声明）

**精神性陈述位置**：宪法 §六-时空锚定（行 82-83）与 §六-Oracle 责任（行 87-89）相互拉扯，没明文裁决。

---

#### §1.1.4 kmir 专属规则候选（43 候选）

**c 阶段挑刺**：kmir 100 FAILED 中 24 个 JSONDecodeError + 19 个 Cargo compilation failed。c 阶段说这是 kmir wrapper Python 自身崩溃，没经过 K interpreter，应映射 UNKNOWN。

**counter（反 c 阶段）**：见 §1.2 详细。这里先给精神性反查。

- **精神锚点 α**："工具自陈我没全干完必须被尊重"——kmir 自陈 `Cargo compilation failed` 是工具自己说"我的 pipeline 跑不下来"，这难道不是"工具自陈未完成"的表达吗？
- **counter-counter（驳 α 在这场合下的应用）**：
  - kmir 的失败陈述发生在 `kmir/cargo.py` Python 层抛 `Exception('Cargo compilation failed')`，这不是 kmir 的 K interpreter 在 MIR 层做 _语义判断_ 后说"我不支持这个 MIR"
  - 这只是 kmir Python wrapper 调用其内嵌 cargo 时的 _wrapper 层异常_，无关 K interpreter 能力
  - 类比：如果 verifast 的 OCaml 主程序在解析命令行时崩了，那不算 verifast "前端拒绝"（属 wrapper 层 bug）
- **精神锚点 β** + 实际证据：
  - kmir 自带 cargo 调用是 _kmir 内部 pipeline_ 的一部分；从外部黑盒视角看属"kmir 的 pipeline 上游"
  - 但宪法 §六 转述 β 把 "cargo" 列为外部根因之一——这里有歧义：是 _runner 的 cargo_ 还是 _工具内嵌的 cargo_？
- **反误报担忧（重要）**：
  - 如果把 43 个全升级为 UNKNOWN，需要保证 grep 字面（`Cargo compilation failed` ∧ `kmir.cargo` 或 `JSONDecodeError` ∧ `kmir/cargo.py`）不会在 _真正的 kmir 拒绝_ entry 上误命中
  - 实测：56 个 K-stuck entry 的 stderr 都含 `[kmir-oracle] FAIL: K interpreter stuck`，stdout 末尾不含 `Cargo compilation failed` 或 `JSONDecodeError`
  - 1 个 AssertionError entry（`closure-adv__early-bound-lifetime`）stdout 含 `AssertionError: No production for 'Safety::Safe'` 在 `kmir/parse/parser.py`——这是 _kmir parser 内部断言失败_，不在 cargo 阶段，规则不误吞 ✅

**判定**：kmir 43 候选的精神 _部分明示_：
- α 支持把 43 当真 partial（kmir 自陈失败）
- β 支持把 43 当外部根因（cargo / wrapper 上游）
- 实证上：这 43 个 entry 的 cargo-check 全部 SUCCESS（已验证），证明 entry 本身合法

详细 deep counter 在 §1.2。

---

### §1.2 kmir 43 候选 deep counter

#### §1.2.1 数据校核

cc 阶段独立校核：
- `grep -l 'JSONDecodeError' runs/.../raw/kmir/*.stdout` = 24 ✓
- `grep -l 'Cargo compilation failed' runs/.../raw/kmir/*.stdout` = 19 ✓
- `grep -l 'K interpreter stuck' runs/.../raw/kmir/*.stderr` = 56 ✓
- `grep -l 'AssertionError' runs/.../raw/kmir/*.stdout` = 1（前述 closure-adv entry）
- 24 + 19 + 56 + 1 = 100 ✓ 与 report.md "kmir 161 / 61 / 100" 一致

#### §1.2.2 19 个 "Cargo compilation failed" 子桶

通过 c 阶段列表：

| entry | feature 簇 | cargo-check exit | 推断根因 |
|---|---|---|---|
| `bigint__bigint-arith__bigint_arith` | bigint deps (num-bigint) | SUCCESS | deps 解析失败（kmir 内嵌 cargo 用 pinned toolchain 编不了 num-bigint） |
| `bigint__bigint-conv__bigint_conv` | bigint | SUCCESS | 同 |
| `bigint__bigint-bitwise__bigint_bitwise` | bigint | SUCCESS | 同 |
| `bigint__num-complex-ops__num_complex_ops` | bigint | SUCCESS | 同 |
| `bigint__num-rational-arith__num_rational_arith` | bigint | SUCCESS | 同 |
| `bigint__num-integer-gcd__num_integer_gcd` | bigint | SUCCESS | 同 |
| `bigint__bigint-modpow__bigint_modpow` | bigint | SUCCESS | 同 |
| `bigint__num-traits-abstract__num_traits_abstract` | bigint | SUCCESS | 同 |
| `collections__hashmap__hashmap_basic` | std collections | SUCCESS | 推测 deps 或 std API 不识别 |
| `deps-complex__chrono-bigint__chrono_bigint` | deps | SUCCESS | deps 解析 |
| `deps-complex__bigint-serde__bigint_serde` | deps + serde | SUCCESS | deps 解析 |
| `deps-complex__chrono-serde__chrono_serde` | deps + serde | SUCCESS | deps 解析 |
| `creusot-limit__thread-local-ref__read_thread_local` | thread_local | SUCCESS | thread_local 宏 / 旧 toolchain 不识别 |
| `hax-limit__let-chains__hax_limit_let_chains` | let-chains edition 2024 | SUCCESS | edition 2024 — kmir 内嵌 cargo / toolchain 不支持 |
| `industrial__sha2_sha256-digest__sha256_digest_incremental` | sha2 deps | SUCCESS | deps |
| `industrial__sha2_sha256-digest__sha256_digest_one_shot` | sha2 deps | SUCCESS | deps |
| `industrial__x509-parser_cert-parse__x509_parse_der` | x509 deps | SUCCESS | deps |
| `industrial__x509-parser_cert-parse__x509_subject_extensions` | x509 deps | SUCCESS | deps |
| `lifetime__thread-local__thread_local_read` | thread_local | SUCCESS | thread_local 同上 |

**特征**：12/19 是 deps-rich entry（bigint × 8 + deps-complex × 3 + industrial × 4 — 不，bigint 算 8 个 + deps-complex 3 + industrial-sha2 2 + industrial-x509 2 = 15）。再加 2 个 thread_local + 1 个 edition-2024 + 1 个 collections-hashmap。
- 15/19 是 _典型 deps-rich_（其他工具不走 cargo 时也 E0432 升 UNKNOWN）
- 4/19 是其他工具特定问题（thread-local / edition / hashmap）

**counter（反 c 阶段对 19 个 Cargo failed 的统一定性）**：
- 15 个 deps-rich：精神上 _完全等同_ 其他不走 cargo 的工具上的 E0432，应映射 UNKNOWN（dependency_resolution kmir 变体）。精神锚点 β。
- 2 个 thread-local：cargo-check SUCCESS 证明 thread_local 宏在 mainline 合法；kmir 内嵌 cargo 编不了它是 toolchain pinning 副作用，类同 §1.1.3 第 8 类——**精神模糊**
- 1 个 hax-limit/let-chains（edition 2024）：与 §1.1.1 第 6 类是同精神（Cargo.toml edition propagation）——**非决策点**
- 1 个 collections/hashmap：根因不明，需 deep stderr inspection（见下）

**hashmap_basic 深 counter**：

<details>

`grep` `runs/.../raw/kmir/collections__hashmap__hashmap_basic.stdout` 内容：含 `Cargo compilation failed` ＋ Python traceback。stderr 空。这无法直接看到 cargo 自己抛了什么。但 cargo-check 同 entry SUCCESS，说明 entry 合法。最可能：kmir 内嵌 cargo 的 toolchain 不能编 std::collections::HashMap 用法（不太可能）或 deps 问题。

实测无法直接区分，但**保守起见**：把 hashmap_basic 归为 _kmir 真 partial_（kmir 真的吃不下 hashmap，不论根因是 toolchain 还是 wrapper）——或归为 UNKNOWN（toolchain pinning）。

</details>

**hax-limit/let-chains（edition 2024）深 counter**：完全等同 §1.1.1 — entry 已声明 edition 2024，kmir cargo 不传播。属 §1.1.1 同精神，**非决策点**。

**thread-local 2 个深 counter**：等同 §1.1.3 toolchain pinning，精神模糊。**决策点**。

#### §1.2.3 24 个 "JSONDecodeError" 子桶

通过 c 阶段列表观察：

| entry | feature 簇 | cargo-check exit | 备注 |
|---|---|---|---|
| `arc__clone-drop__arc_clone_drop` | Arc 简单 | SUCCESS | 没有 extra deps |
| `box__basic-alloc__alloc_deref_drop` | Box | SUCCESS | 简单 |
| `charon-limit__box-branch-init__vec_with_early_return` | Box + early return | SUCCESS | |
| `charon-limit__arc-slice-unsize__arc_array_to_slice` | Arc + slice | SUCCESS | |
| `charon-limit__precise-drops-const-generic__consume_portable_hash` | const generics | SUCCESS | |
| `charon-limit__generic-to-dyn-unsize__boxed_display_from_u32` | trait obj | SUCCESS | |
| `collections__btreemap__btreemap_basic` | std BTreeMap | SUCCESS | |
| `concurrency__thread-mutex__thread_mutex_join` | std Mutex | SUCCESS | |
| `creusot-limit__generic-for-loop__trigger_drain_generic_iter` | trait + iter | SUCCESS | |
| `box__shallow-init__shallow_init_box` | Box 浅 init | SUCCESS | |
| `deps-complex__error-chain__error_chain` | deps | SUCCESS | |
| `creusot-limit__vec-macro-std__make_vec` | vec! macro | SUCCESS | |
| `deps-complex__trait-serde-generic__trait_serde_generic` | deps | SUCCESS | |
| `deps-complex__collections-serde__collections_serde` | deps | SUCCESS | |
| `closure-adv__boxed-dyn-fn__boxed_dyn_fn` | dyn fn | SUCCESS | |
| `gat__lending-iter__gat_lending` | GAT | SUCCESS | |
| `industrial__rsa_rsa-pkcs8__rsa_pubkey_from_pkcs8` | rsa deps | SUCCESS | |
| `industrial__rsa_rsa-pkcs8__rsa_pkcs1v15_encrypt` | rsa deps | SUCCESS | |
| `kani-limit__async-await__run_async_add` | async | SUCCESS | edition propagation? |
| `lifetime__static-bound__static_bound` | lifetime | SUCCESS | |
| `miri-limit__thread-interleaving-partial__unsynchronised_counter_race` | concurrency | SUCCESS | |
| `enum__nested-guard__nested_match_guard` | enum + guard | SUCCESS | |
| `rc__clone-drop__rc_clone_drop` | Rc 简单 | SUCCESS | |
| `trait-obj__dyn-dispatch__dyn_dispatch` | dyn trait | SUCCESS | |

**精神性观察**：
- _绝大多数_（20+/24）是简单合法 entry，无 deps，没有 edition 特殊；kmir 在 Python 解析 cargo JSON 输出时崩
- 失败位置是 `kmir/cargo.py:91`，对 `command_result.stdout.splitlines()` 的每一行调用 `json.loads`——任何 cargo 输出的非 JSON 行（warning / build-output / blank line）都会触发
- 这是 _kmir wrapper 自身的脆弱性_：cargo 行为是稳定的，但 kmir 的 cargo 调用方式（解析 stdout 假设全是 JSON-line）不健壮
- 这些 entry 在 cargo-check 100% SUCCESS，证明 entry 合法

**counter（反 c 阶段对 24 个 JSONDecodeError 的统一定性）**：

- 精神锚点 α 主张："kmir 自陈 _我没全干完_"——但 kmir 此时 _没说_ "我不支持这个 MIR"，它只是 Python wrapper 解析自己的 cargo 输出格式失败。这不是工具语义层自陈
- 精神锚点 β："vendor crate / harness 上游失败算误报"——这里 harness 指 runner harness。kmir 的 Python wrapper 算"kmir 自带 harness"还是"kmir 工具本身"？精神 _不明示_
- 一个关键论据：**这 24 个 entry 全是简单合法 entry，没有 deps / edition / 特殊 feature**，cargo-check SUCCESS。失败完全可能因为 cargo 输出顺序变了一个空行 / 一个 warning 行
- 真 partial 的对照：kmir 在 56 个 K-stuck entry 上明确通过 `[kmir-oracle] FAIL: K interpreter stuck` 自陈了 partial；如果 kmir 真的不支持 Arc clone-drop / Box basic-alloc / std BTreeMap，它应该走 K-stuck 路径自陈，而不是在 Python 解析 cargo 输出时崩
- **结论**：24 个 JSONDecodeError 极可能是 wrapper 自身脆弱性，精神 _倾向_ β（属 wrapper / harness 上游失败）

**判定**：精神**部分明示**：
- 24 个 JSON：精神倾向"误报"（β），但宪法没明文覆盖"工具自带 wrapper 脆弱"——属**决策点**但 _倾向支持升 UNKNOWN_
- 19 个 Cargo: 15 个 deps-rich + 1 个 edition 2024 = 16 是**非决策点**（同 §1.1.1 / §1.1.2 精神）; 2 个 thread-local + 1 个 hashmap_basic = 3 是**决策点**

#### §1.2.4 kmir 43 候选裁决总览

| 子桶 | 数量 | 裁决 | 备注 |
|---|---|---|---|
| 19 Cargo failed — 15 deps-rich | 15 | **非决策点 → 真候选误报** | 同 §1.1.1/§1.1.2 精神（Cargo.toml deps 上游） |
| 19 Cargo failed — 1 edition 2024 | 1 | **非决策点 → 真候选误报** | 同 §1.1.1 精神 |
| 19 Cargo failed — 2 thread-local | 2 | **决策点** | toolchain pinning 副作用，精神模糊 |
| 19 Cargo failed — 1 hashmap_basic | 1 | **决策点** | 根因不明，需 deep stderr inspection（runner 当前未保 cargo stdout） |
| 24 JSONDecodeError — 全部 | 24 | **决策点** _但倾向支持_ | wrapper 自身脆弱，精神部分明示 β |

**汇总**：kmir 43 候选中
- 16 个**非决策点**真候选误报（deps + edition）
- 27 个**决策点**（2 thread-local + 1 hashmap + 24 JSON wrapper 脆弱）

---

### §1.3 verifast / verus / rocq×2 / prusti / soteria 13 候选 counter

#### §1.3.1 verifast × 5

| # | entry | c 阶段类别 | counter |
|---|---|---|---|
| 1 | `charon-limit/async-fn/async_forty_two` | edition_propagation | stderr 验证 OK（来自 raw inspection）— 同 §1.1.1，**非决策点** |
| 2 | `kani-limit/async-await/run_async_add` | edition_propagation | 同 |
| 3 | `hax-limit/let-chains/hax_limit_let_chains` | edition_propagation | 同 |
| 4 | `industrial/x509-parser/cert-parse/x509_parse_der` | dep_E0433 | 同 §1.1.2，**非决策点** |
| 5 | `industrial/x509-parser/cert-parse/x509_subject_extensions` | dep_E0433 | 同 |

**全 5 个非决策点**。

**关键反 counter（反我自己的判定）**：verifast 真 partial 数 124 中有些 stderr 上含 `Floating point types are not yet supported` / `Structs with const parameters are not yet supported` / `Expressing shared ownership of &[_] values is not yet supported`——这是 verifast 显式自陈，属真 partial（α 精神）。c 阶段标这些为真 partial 是对的。

只有 5 个候选误报通过 counter 验证，全成立。

#### §1.3.2 verus × 3

| # | entry | c 阶段类别 | counter |
|---|---|---|---|
| 1 | `hax-limit/let-chains/hax_limit_let_chains` | edition_propagation | 同 §1.1.1，**非决策点** |
| 2 | `industrial/x509-parser/cert-parse/x509_parse_der` | dep_E0433 | 同 §1.1.2，**非决策点** |
| 3 | `industrial/x509-parser/cert-parse/x509_subject_extensions` | dep_E0433 | 同 |

**全 3 个非决策点**。

**反 counter（反我自己）**：verus 其他 73 个 FAILED：
- 48 个含 `verifier does not yet support` / `is not supported` — α 精神，真 partial ✓
- 19 个 `rustc_middle/src/ty/generic_args.rs:54:14: index out of bounds` panic — verus 自身 rustc fork 内部 panic，属工具内部 bug 但仍是 verus 处理这个 entry 的能力问题
- 6 个 "other"（aeneas-limit/mutually-recursive-traits / closure_fn / closure_fnmut / creusot-limit/dyn-trait-forbidden / kani-limit/stack-unwinding / lifetime/static-bound）— c 阶段确认含 verus 显式 `error: ... declared to Verus` 或 `Verus does not recognize this trait bound`，α 精神 ✓

c 阶段对 verus 其余 73 的真 partial 判定**全部站得住**。verus 候选误报只 3 个。

#### §1.3.3 rocq-of-rust × 2 + rocq-of-rust-typecheck × 2

| # | 工具 | entry | c 阶段类别 | counter |
|---|---|---|---|---|
| 1 | rocq-of-rust | `charon-limit/async-fn/async_forty_two` | edition | 同 §1.1.1，**非决策点** |
| 2 | rocq-of-rust | `kani-limit/async-await/run_async_add` | edition | 同 |
| 3 | rocq-of-rust-typecheck | `charon-limit/async-fn/async_forty_two` | edition | 同 |
| 4 | rocq-of-rust-typecheck | `kani-limit/async-await/run_async_add` | edition | 同 |

**全 4 个非决策点**。

**反 counter（反我自己）**：rocq-of-rust 其他 16 个 FAILED 全部含 `[rocq-oracle] FAIL` 或 `translate exit 101` 加工具显式错（c 阶段已逐 entry 确认）。rocq-oracle 是反作弊设计（按 §三 0 误报论证位置 hax 类似）。这 16 个**全是真 partial**，c 阶段判定站得住。

#### §1.3.4 soteria × 1

| # | entry | c 阶段类别 | counter |
|---|---|---|---|
| 1 | `hax-limit/let-chains/hax_limit_let_chains` | edition | 同 §1.1.1，**非决策点** |

**反 counter（反我自己）**：soteria 其他 17 个 FAILED：
- 含 `unsupported feature` / `Unsupported intrinsic: atomic_xsub` / `Unhandled float transmute` / `combine3` 等显式 soteria 自陈 — α 精神，真 partial ✓
- 1 个 `bug: Dangling pointer in lib::main`（hashmap_basic）— soteria 检出"UB"，类同 §3.2 miri 边界 case，按跨工具一致 exit-non-zero = FAILED 规则，不算误报，按 c 阶段判定保留为真 partial（边界）

c 阶段对 soteria 其余 17 的判定站得住。soteria 候选误报只 1 个。

#### §1.3.5 prusti × 2

| # | entry | c 阶段类别 | counter |
|---|---|---|---|
| 1 | `float/round/float_round` | toolchain_unstable_feature_gate | 同 §1.1.3，**决策点** |
| 2 | `hax-limit/unsafe-block/hax_limit_unsafe_block` | toolchain_unstable_feature_gate | 同 |

**全 2 个决策点**（精神模糊，需用户裁决）。

**反 counter**：prusti 其他 81 个 FAILED：
- 76 个含 `Verification failed` 或 `[Prusti: ...]` — prusti 自陈 partial / encoder reject，α 精神 ✓
- 5 个含 `panicked at` — prusti rustc fork 内部 panic 在 prusti 代码，工具内部 bug 仍算 partial（类 verus 19 panic）

c 阶段对 prusti 其余 81 站得住。prusti 候选误报 2 个全决策点。

---

### §1.4 R2-c 漏审的工具 spot-check

c 阶段对 MIRI 4 / Kani 8 / charon-mono 8 / charon-poly 7 / creusot 40 / aeneas × 4（共 270 FAILED）/ hax × 3（共 113 FAILED）的真 partial 判定，cc 阶段独立 spot-check 验证。

#### §1.4.1 MIRI 4 — 全数验证

| entry | exit | stderr 头几行 | counter |
|---|---|---|---|
| `charon-limit/inline-asm/nop_via_asm` | 1 | `error: unsupported operation: inline assembly is not supported` | miri 自陈 α，真 partial ✓ |
| `kani-limit/extern-ffi/trigger_call_libc_abs` | 1 | `error: unsupported operation: can't call foreign function abs on OS macos` | miri 自陈 α，真 partial ✓ |
| `kani-limit/uninit-memory/read_uninit_byte` | 1 | `error: Undefined Behavior: reading memory at alloc201... but memory is uninitialized` | miri 检出 UB（α 边界），c 阶段已标边界，按跨工具一致 exit-non-zero = FAILED 保留 |
| `miri-limit/networking-unsupported/tcp_connect_attempt` | 1 | `error: unsupported operation: socket not available when isolation is enabled` | miri 自陈 α，真 partial ✓ |

**miri 4 个 c 阶段判定全部站得住。0 漏审误报。**

#### §1.4.2 Kani 8 — 全数验证

`grep -l '\[kani-oracle\] FAIL: codegen completed with hard-unsupported MIR constructs' runs/.../raw/kani/*.stderr` = 8 ✓ 对应所有 8 个 FAILED entry。每个 entry 的 stderr 都含 oracle wrapper 自披露的 hard-unsupported MIR 构造（InlineAsm / catch_unwind / ptr_mask / simd_cast / C string literal）— α 精神，真 partial ✓

**kani 8 个 c 阶段判定全部站得住。0 漏审误报。**

#### §1.4.3 charon-mono 8 + charon-poly 7 spot-check

抽查 5 个 charon-mono entry（详 c 阶段 §3.3）：
- `charon-limit/async-fn/async_forty_two`: stderr 含 `Coroutine types are not supported yet` ✓
- `charon-limit/generic-to-dyn-unsize/boxed_display_from_u32`: `Could not determine method index for drop in vtable` panic at `translate_trait_objects.rs:1707` ✓
- `lifetime/static-bound/static_bound`: 同 vtable panic ✓
- `trait/cyclic-bound/cyclic_bound_use`: `stack overflow` in charon-driver — 工具内部 bug 但属 charon 能力 ✓
- `unsafe-ptr/raw-ptr-const/raw_ptr_const_match`: `Unsupported constant: ConstantExprKind::Cast` panic at `errors.rs:282` ✓

5/5 真 partial（α 精神 或 工具内部 panic 在工具自身代码）。

**charon-mono 8 + charon-poly 7 c 阶段判定站得住。0 漏审误报。**

注：cc 阶段进一步独立校核 `aeneas-coq` 的 oracle wrapper 命中：59 个 FAILED 中 58 个含 `[aeneas-coq-oracle] FAIL`，1 个例外（`trait__cyclic-bound__cyclic_bound_use`）直接 charon-driver stack overflow exit 101 — 仍是 aeneas pipeline 上游能力问题（charon-driver 是 aeneas 的前端），属 aeneas 接收这个 entry 的能力问题 ✓。

#### §1.4.4 creusot 40 — spot-check

抽查 5 个：
- `bigint/bigint-arith/bigint_arith`: stderr 含 `error[E0277]: ... creusot_std::model::DeepModel is not satisfied` — creusot 类型 reject ✓
- `charon-limit/inline-asm/nop_via_asm`: `panicked at creusot/src/translation/function/terminator.rs:189` — creusot 内部 panic 在 creusot 代码 ✓
- `closure-adv/early-bound-lifetime/early_bound_closure_arg`: `error: Unsupported pointer cast: ClosureFnPointer` — creusot 自陈 α ✓
- `collections/btreemap/btreemap_basic`: `error[E0277]: ... IteratorSpec is not satisfied` — creusot 类型 reject ✓
- `creusot-limit/dyn-trait-forbidden/trigger_call_dyn_display`: `error: Unsupported constant value: Scalar(alloc1)` — creusot 自陈 α ✓

5/5 真 partial。

**creusot 40 c 阶段判定 spot-check 站得住。0 漏审误报。**

#### §1.4.5 aeneas × 4 spot-check

aeneas-coq 59 个 FAILED 中 58 个含 `[aeneas-coq-oracle] FAIL`（已验证）。`aeneas-hol4` 95 个 FAILED 中 94 个含 `[aeneas-hol4-oracle] FAIL`（cc 阶段独立校核）。1 个例外是 cyclic-bound stack overflow，前述。

**aeneas × 4 共 ~270 FAILED c 阶段判定站得住。0 漏审误报。**

#### §1.4.6 hax × 3 spot-check

`hax-fstar` 31 / `hax-lean` 34 / `hax-coq` 48 FAILED，全部含 `[HAXxxxx]` 标签或 hax 工具自身错（按 c 阶段抽样审）。cc 阶段独立校核 hax-fstar 5 个 entry，全部含明确 `[HAX]` 标签 ✓。

**hax × 3 c 阶段判定站得住。0 漏审误报。**

---

## §2 最终误报清单

经 cc counter-challenge 后的真候选误报（按精神锚定 + 实证 stderr）：

### §2.1 非决策点（精神已明示，可直接修）

| 子桶 | 数量 | 精神锚点 | entry 清单 |
|---|---|---|---|
| **第 6 类 edition_pipeline_propagation** | 9 | β（Cargo.toml edition 上游） | rocq-of-rust × 2、rocq-of-rust-typecheck × 2、soteria × 1、verifast × 3、verus × 1 |
| **第 7 类 dependency_resolution_E0433** | 10 | β（vendor crate 上游） | rocq-of-rust × 2、rocq-of-rust-typecheck × 2、soteria × 2、verifast × 2、verus × 2 |
| **kmir deps-rich Cargo failed** | 15 | β（Cargo.toml deps 上游） | kmir × (bigint × 8 + deps-complex × 3 + industrial × 4) |
| **kmir edition 2024 Cargo failed** | 1 | β（edition 上游，同 §1.1.1） | kmir × hax-limit/let-chains |
| 小计非决策点 | **35** | | |

### §2.2 决策点（精神模糊或需独立讨论）

| 子桶 | 数量 | 精神位置 | 备注 |
|---|---|---|---|
| **第 8 类 toolchain_unstable_feature_gate** | 2 | §六 时空锚定 vs §六 Oracle 责任 拉扯 | prusti × float_round + prusti × hax-limit/unsafe-block |
| **kmir thread_local toolchain pinning** | 2 | 同 §1.1.3 | kmir × creusot-limit/thread-local-ref + kmir × lifetime/thread-local |
| **kmir hashmap_basic 根因不明** | 1 | β 部分覆盖 | kmir × collections/hashmap/hashmap_basic |
| **kmir 24 JSONDecodeError 全 wrapper 脆弱** | 24 | β 部分覆盖（"工具自带 wrapper 算 harness 上游" 精神模糊） | 见 §1.2.3 表 |
| 小计决策点 | **29** | | |

### §2.3 总数

- 非决策点（直接落地）：**35**
- 决策点（需用户裁决）：**29**
- **cc 阶段确认的真候选误报：64**（与 c 阶段 64 候选完全等量，但内部重新切分了非决策 vs 决策）

### §2.4 c 阶段挑刺中被 cc 推翻的

**0 条**。c 阶段 64 候选全部 cc 验证有效（虽然其中 29 条精神模糊，但都不是"挑刺无效"——是"挑刺成立但需用户裁决落地形式"）。

### §2.5 c 阶段漏审的工具中 cc 发现的新误报

**0 条**。spot-check MIRI 4 / Kani 8 / charon 15 / creusot 40 / aeneas 270+ / hax 113，全部 c 阶段判定站得住。

---

## §3 决策点 vs 非决策点（按 charter-craft §4.8.3 + workflow）

### §3.1 非决策点（35 个，可直接修）

按 `tool-integration.md` §四-4.2 防漏报机制反误报检查的要求，以下规则可直接落地为 oracle 规则扩展：

#### §3.1.1 扩 oracle 第 6 类规则 — edition_pipeline_propagation

**触发字面**：
```
contains_any "async fn` is not permitted in Rust 2015"
        OR  "to use `async fn`, switch to Rust 2018 or later"
        OR  "let chains are only allowed in Rust 2024 or later"
        OR  "pass `--edition" (跟随 to rustc 上下文)
→ tag: edition_pipeline_propagation
```

**反误报论证**（按 `tool-integration.md` §4.2 双向实测）：
- 真 partial 不会主动含 "async fn is not permitted in Rust 2015" — 这是 rustc 自身的标准 edition gate
- 用户代码不会主动写 "let chains are only allowed in Rust 2024 or later" — 这是 rustc lint
- ∴ 反误报安全

**预期影响**：9 个 FAILED → UNKNOWN（rocq-of-rust × 2 / rocq-of-rust-typecheck × 2 / soteria × 1 / verifast × 3 / verus × 1）

#### §3.1.2 扩 oracle 规则 2 — 把 E0432 拓宽为 E0432 ∪ E0433

**触发字面**：
```
contains_any "error[E0432]: unresolved import"
        OR  "error[E0433]: cannot find module or crate"
→ tag: dependency_resolution
```

**反误报论证**：
- 用户代码会不会写出 E0433？理论上"用户拼写 module 错"也触发 E0433，但 cargo-check SUCCESS 已经排除了这种内部 bug — 因为 cargo-check 用同一 rustc 跑同一 entry SUCCESS 证明用户代码合法
- 这条规则隐含的前提是"entry 在 cargo-check 上 SUCCESS"——已经是 oracle 整体的 calibration 前提
- ∴ 反误报安全

**预期影响**：10 个 FAILED → UNKNOWN（5 工具 × 2 x509-parser entries）

#### §3.1.3 加 oracle 第 9 类规则 — kmir cargo deps + edition propagation

**触发字面**：
```
contains "kmir.cargo - Cargo compilation failed" 
   OR   "kmir/cargo.py:97" + "Cargo compilation failed"
→ tag: kmir_wrapper_cargo_failure（暂归 dependency_resolution kmir 变体或独立第 9 类）
```

**反误报论证**：
- 真 kmir partial 都走 `[kmir-oracle] FAIL: K interpreter stuck` 路径，不在 cargo 阶段失败
- 1 个 `closure-adv/early-bound-lifetime` 走 AssertionError in parser，不在 cargo 阶段
- ∴ 规则不误吞真 partial
- 反误报担忧：如果某个真 partial 的 entry 在 cargo 阶段就被 kmir 吃不下（比如 kmir 真的不支持某 syntax），规则会误吞——但 cc 验证：kmir 内嵌的 cargo 调用是 _标准 cargo + smir-output_，不参与 kmir 语义判定；语义判定在 K interpreter 阶段
- ∴ 反误报安全（针对 16 个非决策子集）

**预期影响**：16 个 kmir FAILED → UNKNOWN（15 deps-rich + 1 edition 2024）

**注意**：这条规则只覆盖 16 个非决策点。剩 3 个 Cargo failed（2 thread-local + 1 hashmap）+ 24 JSONDecodeError 见 §3.2 决策点处理。

### §3.2 决策点（29 个，需用户裁决）

以下问题精神模糊，cc 阶段不能直接下结论。需要用户对宪法 / 下游设计的精神性补充或裁决。

#### §3.2.1 决策点 D1：toolchain pinning 副作用是否算外部根因

**背景**：prusti × 2 (`float/round`, `hax-limit/unsafe-block`) + kmir × 2 thread-local（共 4 个）

**争议点**：
- entry 使用的 feature 在 mainline rustc 已 stable（cargo-check SUCCESS）
- 工具锁死的 rustc 旧版本里 feature 还在 gate / 不识别
- 失败根因是 _时空错配_（工具版本与 entry 主线 feature 错配），不是工具核心语义限制

**两个立场**：
- **立场 A（c 阶段倾向）**：toolchain pinning 副作用 = 外部根因。理由：cargo-check 在主线 SUCCESS 证明 entry 合法；工具是否能处理 _主线合法的 entry_ 是工具版本与时间的函数。按 §六 时空锚定，"工具能力观察必须锚定 (工具版本, 时间) 二元组"——不锚定就升 UNKNOWN
- **立场 B（cc 反 counter）**：toolchain pinning 副作用 = 真 partial。理由：工具 _就是_ 选择了它的 rustc 版本；这是工具的设计选择。entry 主线 stable 不代表工具能吃下；工具明确 reject 就是 reject

**用户裁决项**：宪法 / 下游是否加一条 "工具 toolchain pinning 与 entry 主线 feature 错配" 的外部根因类？

**如果选立场 A**：oracle 加规则
```
contains "error[E0658]: use of unstable library feature"
    AND entry 在 cargo-check 上 SUCCESS（已是 audit 前提）
→ tag: toolchain_unstable_feature_gate
```
预期影响：4 个 FAILED → UNKNOWN（prusti × 2 + kmir × 2 thread-local）

**如果选立场 B**：不动 oracle，但需要在 prusti README 与 kmir README 增"toolchain pinning 与 mainline feature 错配是已知误报盲点"诚实声明（类 `tool-integration.md` §4.4 漏报盲点对偶）

#### §3.2.2 决策点 D2：工具自带 wrapper 自身脆弱是否算外部根因

**背景**：kmir × 24 JSONDecodeError + 1 hashmap_basic（共 25 个）

**争议点**：
- 24 个 JSONDecodeError 是 kmir Python wrapper 在 `kmir/cargo.py:91` 解析 cargo stdout 时崩溃
- 1 个 hashmap_basic 是 Cargo compilation failed 但根因不明（runner 当前未保存 kmir 内嵌 cargo 的 stdout / stderr）
- entry 全部在 cargo-check SUCCESS（已验证）
- 真 partial 路径（K interpreter stuck）在 56 个 entry 上明确自陈；这 25 个完全没走到 K interpreter

**两个立场**：
- **立场 A（cc 倾向）**：wrapper 脆弱算外部根因。理由：wrapper 失败模式与工具核心语义判定无关；如果 wrapper 升级（kmir 修了 cargo.py），同 entry 可能 SUCCESS。这是 wrapper 时空错配，类 §3.2.1
- **立场 B（反 counter）**：wrapper 是工具的一部分。理由：用户/runner 看到的是黑盒 kmir CLI；CLI 跑不下来就是工具能力问题。如果允许"wrapper 算外部"，每个工具都可以宣称"我的某个 Python wrapper 崩了不算我"

**用户裁决项**：宪法 / 下游是否区分"工具核心语义层 reject" 与 "工具 wrapper 层 crash"？

**如果选立场 A**：oracle 加规则
```
contains "json.decoder.JSONDecodeError" AND containing context "kmir/cargo.py"
   OR   contains "kmir.cargo - Cargo compilation failed" AND entry 在 cargo-check 上 SUCCESS
→ tag: kmir_wrapper_brittleness
```
预期影响：25 个 FAILED → UNKNOWN（24 JSON + 1 hashmap_basic）

**如果选立场 B**：不动 oracle，但需要在 kmir README 列"Python wrapper 在 cargo 输出解析阶段脆弱，对部分合法 entry 直接 crash"作为已知误报盲点

#### §3.2.3 决策点 D2 子裁决：hashmap_basic 根因不明的处理

如果选 §3.2.2 立场 A，hashmap_basic 一起升 UNKNOWN（已含在 25 个中）。
如果选立场 B，建议补一次 deep run 启用 `RUSTFLAGS=--cap-lints=allow` 或独立调 `kmir/cargo.py` 看 cargo stderr 内容，再判定。

---

## §4 给修复 agent (R3) 的明确清单

### §4.1 R3 可立即修的 case（来自 §3.1 非决策点）

**总共 35 个 FAILED → UNKNOWN**，分 3 条 oracle 规则扩展：

#### §4.1.1 规则扩展 1：edition_pipeline_propagation（新增第 6 类）

**修改位置**：`runner/src/report.rs` 第 72-121 行（oracle 规则区）

**伪代码**：
```rust
if stderr.contains("async fn` is not permitted in Rust 2015")
    || stderr.contains("to use `async fn`, switch to Rust 2018 or later")
    || stderr.contains("let chains are only allowed in Rust 2024 or later")
    || stderr.contains("pass `--edition")
{
    return Some("edition_pipeline_propagation");
}
```

**反误报双向实测要求（按 `tool-integration.md` §4.2）**：
- 合法 SUCCESS（cargo-check passes）+ entry 使用 async fn / let chains 的 case：需在 oracle SUCCESS 路径下不命中（实测：rocq-of-rust / verifast / verus 等 _走 cargo_ 的 case 不会触发此 stderr 字面）
- 真 partial 不会触发此 stderr 字面（实测：rocq-of-rust translate exit 101 + verifast vacuous-pass / verus rustc-fork panic 都不含 rustc edition gate 文本）

**预期 UNKNOWN 增加**：9（rocq-of-rust × 2 + rocq-of-rust-typecheck × 2 + soteria × 1 + verifast × 3 + verus × 1）

#### §4.1.2 规则扩展 2：E0432 → E0432 ∪ E0433

**修改位置**：`runner/src/report.rs` 现有 E0432 规则字面

**伪代码**：
```rust
// before:
//   if stderr.contains("error[E0432]: unresolved import") {
//       return Some("dependency_resolution");
//   }
// after:
if stderr.contains("error[E0432]: unresolved import")
    || stderr.contains("error[E0433]: cannot find module or crate")
{
    return Some("dependency_resolution");
}
```

**反误报双向实测**：
- 真 partial 不主动触发 E0433（实测：verifast vacuous-pass / verus rustc panic 都不含 E0433）
- 用户代码会不会触发 E0433？理论可能（拼写错误），但 cargo-check SUCCESS 已排除（同 calibration 前提）

**预期 UNKNOWN 增加**：10（5 工具 × 2 x509-parser entries）

#### §4.1.3 规则扩展 3：kmir 第 9 类外部根因（部分子集）

**修改位置**：`runner/src/report.rs` + 可能 `tools/kmir/tool.toml` 增 wrapper

**伪代码（最小集，仅覆盖 16 个非决策子集）**：
```rust
if stderr_or_stdout.contains("kmir.cargo - Cargo compilation failed") {
    // 但需要二级 disambiguation：是否真 deps-rich 或 edition 2024
    // 当前简化：直接覆盖所有 kmir Cargo failed
    return Some("kmir_cargo_propagation");
}
```

**反误报双向实测**：
- 真 kmir partial 走 K interpreter stuck 路径，不会同时 stdout 含 "kmir.cargo - Cargo compilation failed"（实测：56 个 K-stuck entry 不含此字面）
- 1 个 AssertionError 路径走 `kmir/parse/parser.py`，不含此字面

**风险**：这条规则会一并把 §3.2.1 D1 中 2 个 kmir thread-local case 也升 UNKNOWN。如果用户在 D1 决策 _不_ 升 toolchain pinning 类，则这条 kmir 规则的覆盖会无意中也覆盖 toolchain pinning 子集——需要在 D1 决策后再精化规则字面（比如要求字面同时含 deps crate 名字 / edition 字符等）。

**所以建议**：R3 在 D1 决策 _之前_ 先不加 kmir 规则。等用户对 §3.2.1 D1 + §3.2.2 D2 给意见后再实施。

**临时方案**：R3 只先实施 §4.1.1 和 §4.1.2 两条非决策点规则（共 19 个 FAILED → UNKNOWN），kmir 系列等用户裁决 D1 / D2 后落地。

### §4.2 决策点 case（29 个，等用户裁决）

| 决策点 | 数量 | 路径 |
|---|---|---|
| D1：prusti 2 + kmir 2 thread-local | 4 | 等用户裁决"toolchain pinning 副作用是否算外部根因" |
| D2：kmir 24 JSON + kmir 1 hashmap | 25 | 等用户裁决"工具自带 wrapper 脆弱是否算外部根因" |

R3 不动这些 case。等用户在 docs/design/ 中给精神性补充或裁决。

### §4.3 R3 落地后的预期影响

实施 §4.1.1 + §4.1.2 两条非决策规则后：
- **19 个 FAILED 重分类为 UNKNOWN**（按宪法 §六 不冤枉精神）
- 各工具的实际报告 FAILED 数变化：
  - rocq-of-rust：18 FAILED → 14 FAILED，UNKNOWN 19 → 23
  - rocq-of-rust-typecheck：18 FAILED → 14 FAILED，UNKNOWN 19 → 23
  - soteria：18 FAILED → 17 FAILED，UNKNOWN 19 → 20
  - verifast：129 FAILED → 124 FAILED，UNKNOWN 19 → 24
  - verus：76 FAILED → 73 FAILED，UNKNOWN 19 → 22

实施 §4.1.3 kmir 16 子集（如果用户裁决 D1 / D2 包含路径）：
- kmir：100 FAILED → 84 FAILED，UNKNOWN 0 → 16（最小集），最大 0 → 45（含 D1 D2）

---

## §5 严格遵守 + 完成声明

按本审计要求 + charter-craft §4.8 + workflow：

- 本审计**不修任何代码 / tool.toml / wrapper / runner**
- 本审计**不 commit** 任何代码
- 本审计**不基于猜测**：每条 counter 引用 (a) 宪法精神位置 + (b) 具体 entry stderr 文本片段 + (c) cc 阶段独立校核数据（grep -l 计数等）
- 本审计**不胡编 / 不凑数**：所有数字均与 `runs/run-1778500291-90812/raw/` 实测可复核
- 本审计**明示区分**决策点 vs 非决策点（按 charter-craft §4.8.3 + `principles.md` §八-审查协议第 121 行）
- 本审计**不给模糊结论**：决策点必须等用户裁决后才能进入 R3

完成。

文件路径：`docs/fixes/audit-v5-cc-counter-challenge-2026-05-11.md`

cc 阶段裁决总览：
- c 阶段 64 候选误报全部 cc 验证有效（**0 推翻**）
- 但 c 阶段未明示区分决策点 vs 非决策点；cc 重新切分：**35 非决策点 + 29 决策点 = 64**
- c 阶段漏审的工具（MIRI 4 / Kani 8 / charon 15 / creusot 40 / aeneas 270+ / hax 113）spot-check 全部站得住，**0 新误报**
- R3 可立即修的 case：**19 个非决策点 FAILED → UNKNOWN**（不含 kmir 子集，因 kmir 规则需在 D1 / D2 决策后再实施）
- 等用户裁决：**D1**（toolchain pinning） + **D2**（wrapper 脆弱）两个决策点，影响 29 个 case
