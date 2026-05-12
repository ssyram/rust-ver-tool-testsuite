# rocq-of-rust — 特性支持评估报告（v6 baseline）

## 元数据

- **数据源**：`runs/run-1778560393-59119/`（2026-05-12 v6 final，合并 verus rerun + R7 5-tool rerun）
- **工具配置**：`tools/rocq-of-rust/`
- **工具版本**：`rocq_of_rust_cli 0.1.0` @ commit `a8a76a4d`，nightly-2024-12-07 toolchain
- **本工具实测**：n=161 / SUCCESS=123 / FAILED=38 / UNKNOWN=0，通过率 **76.4%**
- **时长分布**：avg 705ms / median 775ms / p90 1018ms / max 1657ms（N=7 attempt wrapper translate 主导，仍远低于 120s timeout）
- **宪法 baseline**：`principles.md` v8（P27 修宪后 / P31 法律传导后 + P28 Gate 7 / D3.1 后）
- **时效声明**：本快照锚定上述 run id + 工具版本 + corpus + 7 道门 + N=7-attempt wrapper oracle，不构成长期承诺。rocq-of-rust 设计上几乎永远 exit 0，silent fallback 通过产物字面 marker / silent skip-item / stderr `is not yet supported` 表达；新版本可能引入新的 fallback 路径不带已知 marker / 不被现有 7 道门抓。

## pipeline + 前端边界

rocq-of-rust 是 Rust → Rocq（原 Coq）的轻量 syntactic transcoder：

```
rocq-of-rust-wrapper.sh
  → 循环 N=7 次：rocq-of-rust translate --path src/lib.rs --output-path rocq_translation_<i>
       → rustc_interface 抓 HIR / typed AST
       → 搬运到 Rocq monadic embedding（M.closure / M.borrow / M.get_trait_method / Pointer.Kind.MutRef 等）
       → 每个 fn 翻译为 Definition <name> + Global Instance Instance_IsFunction_<name> ... Admitted.
       → 写出 .v 文件到 rocq_translation_<i>/<absolute-source-path>.v
  → 对每次 attempt 跑 7 道门
  → 任一 attempt 任一门未满足 → exit ≠ 0
  → 全过则 symlink rocq_translation → rocq_translation_1
```

`rustc_interface` 直接读 `.rs` 文件，**不读 Cargo.toml** —— 命令直接喂 `src/lib.rs`。`entry_mode` 默认 `bin`：harness 写到 `src/bin/__ts_harness.rs` 但被忽略（rocq-of-rust 只读 `src/lib.rs`）。

`DYLD_LIBRARY_PATH` 指向 nightly-2024-12-07 sysroot `lib/`，`PATH` 注入对应 `bin/`，使 rocq-of-rust 内部调用 `rustc --print=sysroot` 时返回 nightly sysroot（不是 stable）。

**前端 / 后端切割**：rocq-of-rust 是**纯翻译工具**，没有内置 Coq type-check / 证明阶段——pipeline 终点就是 `.v` 文件写盘。本工具"前端 = 全过程 = 翻译到 .v"。下游 `coqc` 是否能 type-check 该 .v 文件**不在本档评估范围**（由配套 `tools/rocq-of-rust-typecheck` 覆盖档 1）。

**项目维护的 wrapper**：`tools/rocq-of-rust/rocq-of-rust-wrapper.sh` 是项目层脚本，实施 N=7-attempt loop + 7 道门 AND-reduce + sysroot 注入 + product symlink。`rocq-of-rust translate` binary 是工具自身的官方 driver。

## SUCCESS 信号 + 形式严格性

按宪法 §六 双通路 partial 暴露：

- **主信号通路**：每次 attempt 的 `rocq-of-rust translate` exit code = 0
- **wrapper 补抓通路**（项目 wrapper 加的 6 道门）：rocq-of-rust 设计上几乎永远 exit 0，silent fallback 全靠产物字面 + stderr grep

**7 道门**（实现见 `rocq-of-rust-wrapper.sh:94-150`，对每次 N=7 attempt 都生效）：

1. exit code = 0
2. 至少一个 `.v` 产物存在
3. 无 0-byte `.v`
4. 至少一个 `.v` > 200 字节
5. 产物不含显式 failure marker：`\(\* (Error |Unexpected |Please report!|thir failed to compile|Unimplemented )`
6. entry_fn `Definition <fn>` 必须出现：`^[[:space:]]*Definition[[:space:]]+$TS_ENTRY_FN[[:space:]]`（runner 通过 `TS_ENTRY_FN` env 注入，`runner/src/exec.rs:178`）
7. **stderr 不含 `is not yet supported`**（2026-05-12 P28 / D3.1 加；rocq-of-rust 对 silent `Pattern::Wild` 退化等 partial 走 stderr warning + exit 0，本门拦截）

任何一次 attempt 的任何门未满足 → FAILED + stderr 诊断 `[rocq-oracle] FAIL (attempt i/N): ...`。

**形式严格性**（按 P27 后宪法严格语义评估）：

- **0 误报（不冤枉能力）**：⚠️ 实测验证 0 误报，**不可形式证明**。oracle 用保守 marker 集 + entry_fn-level grep + N-attempt AND-reduce；用户合法代码极难误命中（合法 entry 必为 `hirusttest.toml` 列出的 fn 名，必为 `src/lib.rs` 顶层或嵌套模块内 fn item，rocq-of-rust 对每个 fn item 都生成 `Definition <name>`，gate 6 必命中）。N-attempt 不引入新误报路径——对确定性翻译路径产物 byte-identical，AND-reduce 与单次结果相同。Gate 7 `is not yet supported` 是 rocq-of-rust 上游 emit 的固定 warning 模板，与合法翻译路径不会重叠。
- **0 漏报（不高估能力）**：⚠️ 实测验证 0 漏报，**不可形式证明**。rocq-of-rust **设计上不用 exit code 表达 partial**（永远 exit 0，对所有 unsupported 用 rustc warning），所以 oracle 只能靠产物字面 grep + 产物 shape + stderr grep。理论上上游可能引入新 fallback 路径不带这 5 类 marker + 不写 `is not yet supported` warning。门 6 N-attempt 把已知非确定性 silent skip 闭环（thread_local! 类 entry 在 v4 重跑下稳定 FAILED）。门 7 把 v5 → v6 反向暴露的 Pattern::Wild 退化路径闭环。
- **漏报盲点**：
  - 上游引入新 silent fallback 路径不带已知 markers 且 entry_fn 仍被生成 + 不写 `is not yet supported`（理论窗口；本 corpus 0 现象）
  - rocq-of-rust 引入新的非确定性翻译路径，N=7 次 attempt 都恰好采到含 entry_fn 的变体——可通过把 N 增大缓解（`ROCQ_OF_RUST_N_ATTEMPTS` env 已暴露）
  - 合理 skip 类（`use` / `extern crate` / `macro_rules!` 在 `top_level.rs:349-390` 直接 `vec![]`）—— 这些不是 fn item，gate 6 不针对（`TS_ENTRY_FN` 永远是 fn 名），属合理 skip，**不算漏报**

## 失败分桶（按 P31 §四.5 归因分类）

38 个 FAILED 分 5 桶，全部归"工具不支持 / 工具能力边界"——无"我们导致"项。

### 桶 A：rocq-of-rust 单文件 + 不读 Cargo.toml 输入模式与 dep-heavy entry 不匹配（21 case，exit=101）

代表 entry：`bigint/bigint-arith/bigint_arith` / `deps-complex/chrono-serde/chrono_serde` / `industrial/rsa/rsa-pkcs8/rsa_pubkey_from_pkcs8` / `industrial/x509-parser/cert-parse/x509_parse_der`

stderr 特征：

```
error[E0432]: unresolved import `num_bigint`
 --> src/lib.rs:5:5
  |
5 | use num_bigint::BigInt;
  |     ^^^^^^^^^^ you might be missing crate `num_bigint`
...
[rocq-oracle] FAIL (attempt 1/7): rocq-of-rust translate exit 101
```

（部分 industrial/x509-parser 触发 `E0433: failed to resolve`，本质同——都是 rocq-of-rust 内部 rustc 在 import resolution 阶段就拒了。）

**归因**：工具不支持（输入模式边界 / 工具 pipeline 设计不读 deps）。rocq-of-rust 内部 `rustc_interface` 直接吃 `.rs` 而不解析 Cargo.toml，导致跨 crate 依赖在 rustc import resolution 阶段就 fail，rocq-of-rust 没机会对这些代码做翻译尝试。按 `principles.md` §六"工具 pipeline 设计不读 Cargo.toml" 明示属工具能力边界。

**处理**：不修。本地性原则下 FAILED 站得住。

涉及 entry：`bigint/*` 8 个（`num_bigint` / `num_traits` / `num_integer` / `num_rational` / `num_complex`）+ `deps-complex/*` 7 个（`chrono` / `serde` / `serde_json` / `itertools` / `anyhow` / `thiserror`）+ `industrial/*` 6 个（`rsa` / `sha2` / `x509_parser`）。

### 桶 B：nightly toolchain 默认 edition / unstable feature（3 case，exit=101）

代表 entry：`charon-limit/async-fn/async_forty_two` / `kani-limit/async-await/run_async_add` / `hax-limit/let-chains/hax_limit_let_chains`

stderr 特征：

```
error[E0670]: `async fn` is not permitted in Rust 2015
  --> src/lib.rs:21:5
  ...
  = help: pass `--edition 2024` to `rustc`
[rocq-oracle] FAIL (attempt 1/7): rocq-of-rust translate exit 101
```

或 `E0658: 'let' expressions in this position are unstable`。

**归因**：工具不支持（工具 pipeline 设计**未透传 cargo manifest 里的 `edition = "2024"` 给内部 rustc**）。按 `principles.md` §六"官方 wrapper 不传 --edition" 一类明示属工具能力边界。

**处理**：不修。FAILED 站得住。

### 桶 C：翻译阶段产物含 explicit failure marker（1 case，exit=1，Gate 5）

涉及 entry：`repr/union/repr_union`

stderr 特征：

```
[rocq-oracle] FAIL (attempt 1/7): product contains explicit failure marker
```

产物含 `(* Error Variant *)`（来自 rocq-of-rust 自身 `top_level.rs` 的 `TopLevelItem::Error(Variant::Union)` 分支）。

**归因**：工具不支持（rocq-of-rust 明示对 union 翻译走 Error 分支）。

**处理**：不修。FAILED 是 rocq-of-rust 自陈的 partial。

### 桶 D：Gate 6 entry_fn silent skip-item（12 case，exit=1）

涉及 entry：

| entry | rocq-of-rust 行为 | gate 6 引入版本 |
| --- | --- | --- |
| `aeneas-limit/mutually-recursive-traits/trigger_mutually_recursive_traits` | exit 0 + 产物无 `Definition trigger_mutually_recursive_traits`（确定性 silent skip） | P12-A |
| `aeneas-limit/trait-impl-mut-param-mismatch/trigger_trait_impl_mut_param_mismatch` | 同上（确定性） | P12-A |
| `kani-limit/float-overapprox/trigger_check_sin_cos_identity` | 同上（确定性） | P12-A |
| `miri-limit/simd-bitmask-large-vector/trigger_bitmask_over_64_elements` | 同上（确定性） | P12-A |
| `miri-limit/soundness-not-guaranteed/trigger_safe_wrapper_hides_ub` | 同上（确定性） | P12-A |
| `miri-limit/weak-memory-incomplete/relaxed_load_may_not_observe_all_stores` | 同上（确定性） | P12-A |
| `prusti-limit/loan-crosses-loop-boundary/trigger_loan_crosses_loop_boundary` | 同上（确定性） | P12-A |
| `prusti-limit/ref-typed-struct-field/trigger_ref_typed_struct_field` | 同上（确定性） | P12-A |
| `prusti-limit/shallow-borrow-match-guard/trigger_shallow_borrow_match_guard` | 同上（确定性） | P12-A |
| `prusti-limit/spec-entailment-unsupported/trigger_spec_entailment_unsupported` | 同上（确定性） | P12-A |
| `creusot-limit/thread-local-ref/read_thread_local` | **非确定性**：P(drop fn)≈0.6，N=7 catch rate ≈99.84% | P15 |
| `lifetime/thread-local/thread_local_read` | **非确定性**：同上 | P15 |

stderr 特征：

```
[rocq-oracle] FAIL (attempt i/7): entry_fn '<fn>' missing from .v products
  (silent skip — top-level kind likely emitted vec![] or item dropped,
   or rocq-of-rust non-deterministic translate path dropped entry on this attempt).
```

**归因**：工具不支持（rocq-of-rust 在 `top_level.rs:349-390` 对部分 top-level kind 直接返回 `vec![]` 但 exit 0 + 5 道门全通过；或翻译路径非确定性导致 entry 被随机 drop）。这是 rocq-of-rust 设计层面的 silent partial—— exit code 不表达，必须靠产物存在性检测。

**处理**：不修。FAILED 是 rocq-of-rust 自陈"我没翻这个 fn"的兑现。

### 桶 E：Gate 7 stderr `is not yet supported`（1 case，exit=1，v5 → v6 新增）

涉及 entry：`unsafe-ptr/raw-ptr-const/raw_ptr_const_match`

stderr 特征：

```
warning: This kind of constant in patterns is not yet supported.
 --> src/lib.rs:9:9
  |
9 |         DISGUISED_INT => 1,
  |         ^^^^^^^^^^^^^
warning: 1 warning emitted

[rocq-oracle] FAIL (attempt 1/7): rocq-of-rust emitted 'is not yet supported' warning
  (silent partial translation; exit 0 but feature dropped from .v product)
```

**归因**：工具不支持（rocq-of-rust 对 `Pattern::Wild` 上 raw-ptr constant 退化为 silent skip，stderr warning + exit 0 + 产物存在但语义不完整）。按 `principles.md` §六"不冤枉 / 不藏"：工具自陈"我没全干完"必须被尊重 → FAILED。

**处理**：不修。Gate 7 是 P28 / D3.1 oracle 紧缩，把以前漏报的 silent partial 翻为 FAILED——v5 → v6 ΔS = −1 全部来源于此。

## 漏报盲点（诚实声明）

- 已通过 wrapper gate 封堵：
  - Gate 5 — 5 类显式 failure marker（`Error / Unexpected / Please report! / thir failed to compile / Unimplemented`）
  - Gate 6 — entry_fn `Definition` 存在性 + N=7 AND-reduce（覆盖确定性 silent skip + 非确定性翻译路径 silent drop）
  - Gate 7 — stderr `is not yet supported` warning（覆盖 silent Pattern 退化等 partial）
- 仍存在的盲点：
  - 上游引入新 silent fallback 路径不带这 5 类 marker + 不写 `is not yet supported` warning（理论窗口；本 corpus 0 现象；新版本需重审 marker 集合）
  - rocq-of-rust 引入新的非确定性翻译路径中 N=7 次 attempt 都恰好采到含 entry_fn 的变体（修复 backlog：`ROCQ_OF_RUST_N_ATTEMPTS` env 已暴露，可临时调大）
  - 合理 skip 类（`use` / `extern crate` / `macro_rules!` 直接 `vec![]`）—— 不算漏报，但若 entry_fn 名误指向这些 item（错误 corpus 设置），门 6 会捕获

## v5.1 → v6 ΔS 解释

v5.1 baseline（N=7-attempt + 6 道门 wrapper）：124 SUCCESS / 37 FAILED / 0 UNKNOWN，通过率 77.0%。
v6（加 Gate 7 `is not yet supported`）：123 SUCCESS / 38 FAILED / 0 UNKNOWN，通过率 76.4%。

**ΔS = −1**，全部来自 P28 / D3.1 Gate 7 紧缩：

- `unsafe-ptr/raw-ptr-const/raw_ptr_const_match`：v5.1 在旧 6 道门下 SUCCESS（5 类 marker 不命中 + entry_fn 在产物中），v6 加 Gate 7 后捕获 stderr `This kind of constant in patterns is not yet supported.` → FAILED

corpus 未变化（仍 161 entries），oracle 紧缩 1 个新 silent partial。FAILED 分桶分布（A=21 / B=3 / C=1 / D=12 / E=1）与 v5.1 比仅 E 桶 +1。

## 翻译产物可运行性

本工具档 0 测 "翻译落盘 + 无 silent fallback + entry_fn 存在 + stderr 无 partial warning"——严格说测的是"产物在 Coq 里是有效的 Coq 项"，**结构正确，语义不验证**。档 1（typecheck）由配套 `tools/rocq-of-rust-typecheck` 覆盖；档 2/3（evaluate / 与 Rust 一致）**架构上不可达**。详见 `docs/research/ror-runnable-deep-dive-2026-05-11.md`。

**深嵌入 vs 浅嵌入**：

- **ror 产物 = deep embedding**：`Definition fn (a b : Value.t) : M.t LowM.t (Value.t + Exception.t) := let* ... call_closure BinOp.Wrap.add ...`。`Value.t` 是 inductive 包装类型，`M` 是 effect monad；所有 op（`alloc` / `read` / `call_closure` / `call_primitive`）是 inductive constructor，**无 Compute 语义**。`vm_compute` / `native_compute` 在 axiom-laden `Run.t` proof tree 上 SIGSEGV。
- **hax-lean 产物 = shallow embedding**（对照）：`def fn (a b : Int32) : RustM Int32 := RustM.ok (a + b)`，Lean `#eval fn 3 4` 一行出 `RustM.ok 7`。

**档可达性总结表**：

| 档 | ror | hax-lean |
| --- | --- | --- |
| 档 0 前端接受 | ✅ 本工具（`tools/rocq-of-rust`）| ✅ `tools/hax-lean` |
| 档 1 typecheck | ✅ `tools/rocq-of-rust-typecheck` | ✅ feasibility 实测 |
| 档 2 auto evaluate | ❌ **架构上不可达** | ✅ `#eval` 实测 |
| 档 2 半人工 lemma | ⚠️ per-entry 5–50 行手工证明 | — |
| 档 3 与 Rust 一致 | ❌ 除非档 2 解决 | ✅ byte-identical 实测 |

**项目决策**：投入档 1 自动化（已上线 `tools/rocq-of-rust-typecheck`），不投入档 2/3 自动化（per-entry 手工证 vs corpus ~160 entries 规模严重不匹配）。这是 ror **设计选择**（deep embedding 为形式证明优化），不是 bug；上游每个 Definition 显式标记 `Admitted.` 已声明"翻译完成 ≠ 可运行"的立场。本档把这视为 rocq-of-rust 的"翻译完成"成功状态。

## 修订建议清单（仅"我们导致"失败）

**无需修订**。所有 38 个 FAILED 均为工具能力边界（rocq-of-rust 单文件 pipeline 不读 deps / 内部 rustc 未透传 edition / TopLevelItem::Error union 分支 / `top_level.rs:349-390` silent `vec![]` / 非确定性翻译路径 silent drop / `Pattern::Wild` 退化）。本工具无任何"我们 wrapper bug / 我们 corpus 引入的 lint / 环境损坏"类失败：

- `rocq-of-rust-wrapper.sh` 是项目维护的脚本，但本 run 中 wrapper 自身未触发任何 IO 错 / 解析错 / shell 语法错——所有 wrapper FAIL 信号都是按设计转发工具自身的 partial 信号（exit code / 产物 grep / stderr grep）
- `hirusttest.toml` / harness 模板未在本 run 触发任何错——所有 stderr 都指向 rocq-of-rust 内部 rustc 或 rocq-of-rust 翻译阶段
- 环境（nightly sysroot / `DYLD_LIBRARY_PATH` / `PATH`）在所有 161 entry 上一致工作，无环境损坏

按 `principles.md` §一"本地性 / 当前性 + 社区惯例 + 最大善意"——最大善意已尽到（装对其要求的 toolchain + 按其文档姿势用工具），仍 FAILED → 工具能力边界，FAILED 站得住。
