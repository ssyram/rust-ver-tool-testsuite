# verifast 深度报告

## 元数据

- **run**: `run-1778226613-5282`（2026-05-08T07:50:13Z → 08:16:08Z UTC，146 entries × 19 工具，host：Apple M5 / macOS aarch64 / 24 GB / 10 cpu）
- **工具版本**：`VeriFast 26.01 (released 2026-01-21) for C and Java`，prover 列表含 `Z3v4.5` / `Redux`，原生 macOS arm64 prebuilt
- **通过率**：116/146 = **79%**（FAILED 30 个，TIMEOUT 0，UNKNOWN 0；FAILED 全部 exit=1）
- **时长（ms）**：avg 282 / median 200 / p90 547 / max 1479
- **时效声明**：本快照锚定 VeriFast 26.01 + 上述 run id + corpus，不构成长期承诺。后续版本（含 Rust 前端 / IR 构造规则 / `-skip_specless_fns` 行为）变化都可能改写边界。

## 工具内部 pipeline + 前端边界

VeriFast 不经过 cargo / rustc，直接读 `src/lib.rs` 跑自家 pipeline：

```
verifast binary → 内嵌 rustc-style Rust parser
              → MIR → VeriFast IR（含 separation-logic share predicate 等结构）
              → 符号执行（按 //@ req/ens/inv/pred 注解驱动）
              → SMT prover (Z3v4.5 / Redux)
```

本测试 `tool.toml` 命令：

```
verifast -target macOS -shared -skip_specless_fns -ignore_unwind_paths
         -disable_overflow_check -read_options_from_source_file src/lib.rs
```

`-skip_specless_fns` 是**前端 dry-run 等价 flag**：无 `//@ req/ens` 注解的函数全跳过符号执行。`entry_mode` 默认 `bin`：runner 写 harness 到 `src/bin/__ts_harness.rs`，但 verifast 命令只读 `src/lib.rs` —— harness 被忽略（VeriFast 不通过 cargo 链接 bin 与 lib，不解析跨 crate 调用）。

## SUCCESS 信号 + 形式严格性

- **形式指标**：verifast exit 0
- **0 误报**：✅ 形式可证。exit 0 ⇔ 前端 IR 构造完成 + 无可证伪项
- **0 漏报**：✅ 形式可证。任何 IR 构造失败 → exit ≠ 0；symex verify error → exit 1
- **漏报盲点**：无（针对 oracle 形式语义而言）

**关键限定（语义降级）**：本 corpus 全集 grep `//@\s*(req|ens|inv|pred)` **0 命中**——所有样例无 spec 注解。配合 `-skip_specless_fns + -disable_overflow_check + -ignore_unwind_paths` 三件套，SUCCESS 实际语义降级为 **vacuous pass**：

> rustc-verifast 接受源码进入 IR + 没有 spec 可证伪

而**不**等于"symex 在用户代码上完成"。本矩阵 116 个 SUCCESS 中 104 个报告同一 baseline `37 statements verified`（来自 verifast 自身 prelude / harness 的 IR 统计），剩 12 个为 38–40（差 0–3 条）；用户函数体多数被 `-skip_specless_fns` 跳过、不计入。这与"覆盖度只看是否在册接受"的宪法原则一致——前端 IR 接受是真发生的——但读 verifast 的 SUCCESS 数字时务必留意：**SUCCESS ≠ verification 实际发生**。

## 实测结果

### 按 feature 类目分布

全 SUCCESS 类目（约 31 项）：`arc / assoc-type / box / closure / closure-adv / collections / concurrency / const / creusot-limit (7/7) / drop / error / float (10/10) / gat / generic / hello / hrtb / impl-trait / int / int-width (14/14) / iter / lifetime / miri-limit (7/7) / panic / rc / refcell / slice / trait / trait-obj / unsafe-adv / unsafe-ptr / vec`。

部分通过：`aeneas-limit 7/8 · charon-limit 5/7 · enum 1/2 · hax-limit 7/8 · industrial 0/6 · kani-limit 6/7 · prusti-limit 6/8 · repr 1/2`。

按 feature 失败计数（前 5）：`bigint 8 · deps-complex 7 · industrial 6 · prusti-limit 2 · charon-limit 2`。

### 失败模式归类（基于 raw stderr/stdout 实读）

30 个 FAILED 全 exit=1，按 verifast 输出形态归 5 类：

**A. 不解析 cargo 依赖（21）**——verifast 命令直接读 `src/lib.rs`，不读 `Cargo.toml`，任何 `use foo::*;` 中 `foo` 不在 std 内就报 `error[E0432]: unresolved import 'foo'`：
- `bigint/*` 8 个：`num_bigint` / `num_traits` / `num_integer` / `num_rational` / `num_complex`
- `deps-complex/*` 7 个：`chrono` / `serde` / `serde_json` / `itertools` / `anyhow` / `thiserror`
- `industrial/*` 6 个：`rsa` / `sha2` / `x509-parser`（vendor crate 即使在隔离副本里能由 cargo 解析，verifast 不通过 cargo 仍报 unresolved）

stderr 由 verifast 内嵌的 rustc-style parser 给出（带 `help: you might be missing a crate named X`）。

**B. 自家 parser 拒绝 float 在复合类型中（3）**——`error: Floating point types are not yet supported`：`aeneas-limit/float-types/make_measurement`（`Value(f64)` 变体）、`enum/data-variants/enum_match_data`（`Circle(f64)` 变体）、`repr/union/repr_union`（`f: f32` 字段）。注意：`float/*` 单测点 10/10 SUCCESS，但 float 类型一旦出现在 enum 变体 / union 字段 / 复合结构定义中，verifast 26.01 的 parser 显式 reject。

**C. const generics 结构定义不支持（2）**——`error: Structs with const parameters are not yet supported`：`charon-limit/precise-drops-const-generic`（`pub struct PortableHash<const K: usize>`）、`prusti-limit/const-generics/entry_const_generic_buffer`（`pub struct Buffer<const N: usize>`）。

**D. shared ownership of `&[_]` 字段不支持（1）**——`prusti-limit/ref-typed-struct-field`：`error: Error while building default .share predicate conjunct for field data: Expressing shared ownership of &[_] values is not yet supported`，触发于 `pub struct View<'a> { pub data: &'a [u32] }`。verifast 在为含 `&[T]` 字段的 struct 自动生成 separation-logic share predicate 时拒。

**E. edition 边界（3）**——verifast 内嵌 parser 默认 Rust 2015：
- `charon-limit/async-fn/async_forty_two`：`error[E0670]: 'async fn' is not permitted in Rust 2015`
- `kani-limit/async-await/run_async_add`：同上
- `hax-limit/let-chains/hax_limit_let_chains`：`error: let chains are only allowed in Rust 2024 or later`

合计 21 + 3 + 2 + 1 + 3 = 30，与 results.json 一致。

## 与本次测试边界的关系

**vacuous pass 的强制承认**：本 corpus 0 spec 注解 → SUCCESS 实际是"rustc-verifast 接受 IR + 无 spec 可证伪"，**不等于** verification 实际发生。本工具在本矩阵的 79% 是 IR 接受率，不是验证通过率。如果未来给 entry 加 `//@ req/ens` 注解（不符合 entry 自包含原则），或不加 `-skip_specless_fns`（plain Rust 会因隐式溢出 / panic 路径触发误判），SUCCESS 的语义才会上移到"symex 完成"。

**宪法 §六-2 的判定**：按"工具完成它自己的工作 = symex 完整跑完且无中断"精神，verify-err（exit 1）一律记为 FAILED。本矩阵中无 verify-err entry（0 个真触发 prover），所有 30 个 FAILED 都在更早阶段（parser / IR 构造）就被拒。

`industrial/*` 6 个全 FAILED 是 A 类（不读 Cargo.toml），不蕴含 verifast 不能处理 rsa/sha2/x509-parser 的代码——只蕴含它的单文件输入模式与外部 crate 不兼容。

子毫秒级响应（median 200ms / max 1479ms）反映：不调 cargo（无 dependency resolve）+ skip_specless 路径下 prover 几乎不被使用。

## 历史快照声明

本报告所有数字与归类锚定 VeriFast 26.01 / 2026-01-21 prebuilt + macOS arm64 + 上述 6 件 flag 组合。版本升级、edition 默认值变化、share predicate 自动生成规则修订都可能改写本归类。
