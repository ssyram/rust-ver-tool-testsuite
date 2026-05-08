# rocq-of-rust 深度报告

## 元数据

- **run**: `run-1778226613-5282`（2026-05-08T07:50:13Z → 08:16:08Z UTC，146 entries × 19 工具，host：Apple M5 / macOS aarch64 / 24 GB / 10 cpu）
- **工具版本**：`rocq_of_rust_cli 0.1.0` @ commit `a8a76a4d`，nightly-2024-12-07 toolchain
- **通过率**：121/146 = **82%**（FAILED 25 个；exit=1 共 1 个，exit=101 共 24 个，TIMEOUT 0）
- **时长（ms）**：avg 167 / median 129 / p90 342 / max 485
- **时效声明**：本快照锚定上述 run id + 工具版本 + corpus，不构成长期承诺。rocq-of-rust 设计上几乎永远 exit 0，silent fallback 通过产物字面 marker 表达；新版本可能引入新的 fallback 路径不带已知 marker。

## 工具内部 pipeline + 前端边界

rocq-of-rust 是 Rust → Rocq（原 Coq）的轻量 syntactic transcoder：

```
rocq-of-rust translate --path src/lib.rs --output-path rocq_translation
  → 通过 rustc_interface 抓 HIR / typed AST
  → 直接搬运到 Rocq monadic embedding（M.closure / M.borrow / M.get_trait_method / Pointer.Kind.MutRef 等）
  → 写出 .v 文件到 rocq_translation/<absolute-source-path>.v
  → 每个 fn 翻译为 Definition <name> + Global Instance Instance_IsFunction_<name> ... Admitted.
```

`rustc_interface` 直接读 `.rs` 文件，**不读 Cargo.toml** —— 命令直接喂 `src/lib.rs`。`entry_mode` 默认 `bin`：harness 写到 `src/bin/__ts_harness.rs` 但被忽略（rocq-of-rust 只读 `src/lib.rs`）。

DYLD_LIBRARY_PATH 指向 nightly-2024-12-07 sysroot `lib/`，PATH 注入对应 `bin/`，使 rocq-of-rust 内部调用 `rustc --print=sysroot` 时返回 nightly sysroot（不是 stable）。

rocq-of-rust 是**纯翻译工具**，没有内置 Coq type-check / 证明阶段——pipeline 终点就是 `.v` 文件写盘。本工具"前端 = 全过程 = 翻译到 .v"；下游 `coqc` 是否能 type-check 该 .v 文件**不在本测试范围**——这依赖 RocqOfRust runtime library 提供 std/外部 crate 的 binding。

## SUCCESS 信号 + 形式严格性

**SUCCESS 必须满足 5 道门**（在 `tool.toml` 包装 sh 中实现）：

1. exit code = 0
2. 至少一个 `.v` 产物存在（`find rocq_translation -name '*.v' -print -quit` 非空）
3. 无 0-byte `.v`（`find ... -size 0 | wc -l` = 0）
4. 至少一个 `.v` > 200 字节（`find ... -size +200c | wc -l` > 0）
5. 产物不含显式 failure marker：`grep -rqE '\(\* (Error |Unexpected |Please report!|thir failed to compile|Unimplemented )' rocq_translation`

任何门未满足 → FAILED（rc=1）。

- **partial 暴露机制**：rocq-of-rust **设计上不用 exit code 表达 partial**——几乎永远 exit 0，对所有 unsupported 用 rustc warning，不影响 exit。所以 oracle 完全靠产物 grep + 产物 shape 检测——这是工具自身设计决定的"前端测试范围"切割方式
- **0 误报**：⚠️ 实测验证，不可形式证明。oracle 用保守的 marker 集——只抓 rocq-of-rust 自己 emit 的 explicit failure comment 块（`(* Error / Unexpected / Please report! / thir failed to compile / Unimplemented`），用户合法代码极难误命中
- **0 漏报**：⚠️ 实测验证，不可形式证明。理论上 rocq-of-rust 上游可能引入新 fallback 路径不带这些 marker
- **漏报盲点**：
  - 上游引入新 silent fallback 路径不带已知 markers（实测在 examples corpus 0 现象——见下文 §"实测结果"）
  - 完全 skip item 类（`use` / `extern crate` / `macro_rules!` 在 `top_level.rs:349-390` 直接 `vec![]`）—— 这些是 rustc 编译时已被处理的 import / macro，**不需要在产物里有 declaration**，所以是合理 skip，**不算漏报**

## 实测结果

### 按 feature 类目分布

全 SUCCESS 类目（约 30 项）：`arc / assoc-type / box / closure / closure-adv / collections / concurrency / const / drop / enum / error / float (10/10) / gat / generic / hello / hrtb / impl-trait / int / int-width (14/14) / iter / lifetime / panic / rc / refcell / slice / trait / trait-obj / unsafe-adv / unsafe-ptr / vec`。

7 个 *-limit 类目大部分通过：`aeneas-limit (8/8) / charon-limit (6/7) / creusot-limit (7/7) / hax-limit (7/8) / kani-limit (6/7) / miri-limit (7/7) / prusti-limit (8/8)`。

部分通过：`repr 1/2`（仅 union 失败）。

按 feature 失败计数：`bigint 8 · deps-complex 7 · industrial 6 · repr 1 · charon-limit 1 · hax-limit 1 · kani-limit 1`。

### 失败模式归类（基于 raw stderr 实读）

**A. 输入模式：单文件 + 不读 Cargo.toml（21）**——`error[E0432]: unresolved import`：
- `bigint/*` 8 个：`num_bigint` / `num_traits` / `num_integer` / `num_rational` / `num_complex`
- `deps-complex/*` 7 个：`chrono` / `serde` / `serde_json` / `itertools` / `anyhow` / `thiserror`
- `industrial/*` 6 个：`rsa` / `rand` / `sha2` / `x509_parser` 等

stderr 形态：`help: consider importing the X crate ... 5 + extern crate X;`。这**不是 rocq-of-rust 的"翻译能力边界"**——是它的 input 模式与 dep-heavy entry 不匹配：rustc 在 import resolution 阶段就拒了，rocq-of-rust 没机会对这些代码做翻译尝试。

**B. nightly toolchain 默认 edition / unstable feature（3）**——
- `charon-limit/async-fn/async_forty_two`：`E0670: 'async fn' is not permitted in Rust 2015`，`= help: pass --edition 2024 to rustc`
- `kani-limit/async-await/run_async_add`：同上
- `hax-limit/let-chains/hax_limit_let_chains`：`E0658: 'let' expressions in this position are unstable`，`= help: add #![feature(let_chains)] to the crate attributes to enable`

rocq-of-rust 内部用 nightly-2024-12-07 toolchain 调 rustc 但**未透传 cargo manifest 里的 `edition = "2024"`**——默认走 2015 edition，async fn 直接拒；nightly feature 如 `let_chains` 也未自动启。同样**不是翻译能力边界**——是 input 流水线层面：rustc 没接受 input → rocq-of-rust 无机会翻译。

**C. 翻译阶段产物含 explicit failure marker（1，exit=1）**——`repr/union/repr_union`：rocq-of-rust 自身 exit 0 + `.v` 产物已写出（stdout：`Translating: src/lib.rs / Starting to translate "src/lib.rs"... / 11 ms have passed to translate: "src/lib.rs" / Finished.`），但 oracle 第 5 道门 grep 命中 explicit failure marker → 翻为 exit 1 + 写 `[rocq-oracle] FAIL: product contains explicit failure marker` 到 stderr。

这是本矩阵唯一一个**真正落在 rocq-of-rust 翻译能力边界上的失败**——`repr/union/repr_union` 的 union 类型在 `top_level.rs` 翻译时走 `TopLevelItem::Error(Variant::Union)` 路径，emit `(* Error <Variant> *)` marker。5 道门 grep 把它从 silent fallback 翻为显式 FAILED。

合计 21 + 3 + 1 = 25，与 results.json 一致（其中 24 个 exit=101 来自 A+B 类，1 个 exit=1 来自 C 类）。

## 与本次测试边界的关系

**5 道门 grep guard 抓 silent partial**：rocq-of-rust 几乎永远 exit 0，silent fallback 通过产物字面 marker 表达。本矩阵实测：
- 24/25 FAILED 在 rustc 端就死（A+B 类），rocq-of-rust 没机会翻译——这些不在 rocq-of-rust 的"翻译能力"层面
- 1/25 FAILED（`repr/union`）落在第 5 道门 grep（C 类）——是真正的翻译能力边界触发

121 个 SUCCESS 上 grep 没命中任何 `Unexpected / Please report! / thir failed to compile / Unimplemented / Error` marker——说明 rocq-of-rust 在它能接受 input 的 entry 上**给出了完整翻译，没出现工具自己标记的占位**。

**合理 skip 不算漏报**：rocq-of-rust 在 `top_level.rs:349-390` 对 `use` / `extern crate` / `macro_rules!` 直接返回 `vec![]`——这些是 rustc 编译时已被处理的 import / macro，不需要在产物里有 Rocq declaration。如果 oracle 把这种 skip 当 silent partial，会大量误报合法代码。当前 5-marker 集（`Error / Unexpected / Please report! / thir failed to compile / Unimplemented`）只抓 rocq-of-rust 自己 emit 的 explicit failure comment 块，避免误伤合理 skip 路径。

**单一 syntactic 通路覆盖大量在其他工具上分化的特性**：GAT、HRTB、`impl Trait` 返回、闭包返回、`unsafe` 指针操作、`Drop`、`#[repr]`（非 union）、并发原语（Arc / Mutex）、trait object——这些在多数工具上出现 SUCCESS 率分化的特性，rocq-of-rust 全部 SUCCESS。原因：所有这些构造统一翻成 `M.closure / M.borrow / Pointer.Kind.MutRef / M.get_trait_method "<trait_path>" "<method>"` 等 syntactic 节点，**不在翻译阶段做语义筛**——`&mut` 翻成字符串标签 `"MutRef"`、trait method 翻成字符串查表。这种设计意味着"工具接受这段 Rust"和"工具能在这段 Rust 上推 borrow 安全 / 解 trait"是分开的两件事，本测试只测前者。

每个 Definition 显式标记 `Admitted.`——这是 rocq-of-rust 设计上明确把"证明义务"留给 Rocq 端的人手写规约。本测试把它视为 rocq-of-rust 的"翻译完成"成功状态。

## 历史快照声明

本报告所有数字与归类锚定 commit `a8a76a4d` + nightly-2024-12-07 toolchain + 当前 5 道门 oracle（marker 集 = `Error / Unexpected / Please report! / thir failed to compile / Unimplemented`）。rocq-of-rust 升级（含新 silent fallback 路径、marker 文本变更、edition 默认透传）后归类可能改写。
