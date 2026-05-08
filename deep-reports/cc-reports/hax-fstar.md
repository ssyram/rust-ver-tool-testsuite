# hax-fstar — 特性支持评估报告

## 元数据

- **数据源**：`runs/run-1778226613-5282/`（2026-05-08，146 entries × 19 工具矩阵；host: Apple M5 / macOS aarch64 / 24 GB / 10 cpus，并发 10）
- **工具版本**：`hax untagged-git-rev-30949eb870`（commit `30949eb87058895c24f963df90dd30ef11b0dc1a`）；nightly toolchain `nightly-2025-11-08`；OCaml `hax-engine` + Rust frontend driver `driver-hax-frontend-exporter`
- **工具配置**：`tools/hax-fstar/`
- **通过率**：SUCCESS 115 / 146 ≈ **78.8%**（FAILED 31，TIMEOUT 0）
- **耗时分布**：avg 3315 ms / median 1333 ms / p90 7785 ms / p95 13759 ms / max 33132 ms（无 timeout 触发，timeout 上限 300 s 远未触达）
- **时效声明**：本快照锚定上述 run id + hax commit + nightly 工具链 + corpus，不构成长期承诺。三个 hax backend 共享同一 `hax-engine` OCaml binary，但 F\* Printer 与 Lean / Coq Printer 是各自独立的 OCaml 模块，未来上游对各 printer 的拒绝边界改动会让本快照失效。

## 工具内部 pipeline + 前端边界

```
rustc + driver-hax-frontend-exporter
  → THIR JSON
  → hax-engine（OCaml）
  → phase pipeline（reject phases + 改写 passes）
  → F* Printer（OCaml→F* 文本生成）
  → 写出 <work>/proofs/fstar/extraction/<Crate>.fst
```

本测试关心 `cargo +nightly-2025-11-08 hax -C --lib ; into fstar` 的"前端通过"。**前端 / 后端切割**：hax 是纯翻译工具，pipeline 终点是 `.fst` 落盘——下游 F\* type-check / Karamel / 用户证明完全不在测试范围。

差异于 hax-lean：F\* backend 在 hax 中较成熟（README 声明），`hax-engine` 内部 phase pipeline 上 F\* 走的 reject phases 比 Lean Printer 多（含 `reject_RawOrMutPointer` / `reject_ArbitraryLhs` / `reject_TraitItemDefault` 等显式拒收节点），所以**几乎不走 silent path**——unsupported 都让 cargo hax 在 phase 阶段 emit `[HAX0008]` Diagnostic 并 exit 1。

## SUCCESS 信号 + 形式严格性

**判定式**：

```
SUCCESS ⟺ cargo hax exit 0 ∧ 产物 grep 不命中
          'Rust_primitives.Hax.failure' 或 'failure ((' 字面
```

`Rust_primitives.Hax.failure` 是 hax 内部 `hax_failure_expr` 渲染为 F\* literal 时的完整路径，作为防御性 silent-path 检测。本矩阵下 oracle 对 hax-fstar **0 次触发该路径**——所有 31 条 FAILED 都通过 cargo hax exit 1（即 hax engine 主动 emit Diagnostic）信号识别，silent partial 在本 corpus 上未观察到。

**形式严格性**：
- **0 误报**：⚠️ 实测验证，不可形式证明。grep 模式 `Rust_primitives.Hax.failure` 是完整路径 literal，用户合法代码极难写出该字面字符串。本矩阵实测：用户 doc comment + 局部变量 `let failure: i32 = 5;` 都不触发——但不能形式排除
- **0 漏报**：⚠️ 实测验证，不可形式证明。F\* backend 较成熟、几乎不走 silent path，本 corpus 0 触发。grep 是防御性检测；理论上上游引入新 silent path 的可能存在
- **漏报盲点**：hax engine 完全 skip item 的可能（实测 0 现象）；上游引入新 silent path 的可能（实测无）

## 实测结果

### 按 feature 类目分布

下列 feature 类目下 hax-fstar **全部 entry 通过**：

```
arc / bigint(8/8) / box / collections / const / deps-complex(7/7) / drop / enum / error
float(10/10) / generic / hello / hrtb(1/1) / impl-trait / int / int-width(14/14) / iter
panic / prusti-limit(8/8) / rc / slice / vec
```

**部分通过**（数字为 S/总）：`aeneas-limit` 5/8、`charon-limit` 5/7、`closure-adv` 3/4、`concurrency` 1/2、`creusot-limit` 6/7、`hax-limit` 2/8、`industrial` 4/6、`kani-limit` 6/7、`lifetime` 2/3、`miri-limit` 6/7、`repr` 1/2、`trait-obj` 1/2、`unsafe-adv` 1/3。

**全 FAILED**：`assoc-type`(0/1)、`closure`(0/2)、`gat`(0/1)、`refcell`(0/1)、`trait`(0/1)、`unsafe-ptr`(0/2)。

### 失败模式归类（基于 raw stderr）

31 个 FAILED stderr 几乎全部带稳定的 `[HAX####]` 错误码 + 关联 GitHub issue 链接。按主导错误码分桶（一个 entry 可能同时触发多码，按主信号归桶）：

| 桶 | 数量 | 主导 reject 信号 |
|---|---:|---|
| **A. `[HAX0008] reject_RawOrMutPointer`** | 6 | `unsafe-ptr/raw-read`、`unsafe-ptr/raw-ptr-const`、`unsafe-adv/ptr-write`、`creusot-limit/thread-local-ref`、`lifetime/thread-local`、`kani-limit/async-await`（VTable 内 raw ptr） |
| **B. `[HAX0003]/[HAX0006]/[HAX0010] DirectAndMut/LocalMutation/AndMutDefsite`** | 11 | mut-ref + 闭包 mutate captured 系：`closure/fn-fnmut/{closure_fn,closure_fnmut}`、`hax-limit/{closure-mutates-outer,mut-in-assoc-type,mut-ref-alias,ret-mut-ref}`、`aeneas-limit/{fnmut-closure-unit-return,trait-impl-mut-param-mismatch}`、`miri-limit/simd-bitmask-large-vector`、`unsafe-adv/maybe-uninit`、`concurrency/thread-mutex` |
| **C. `[HAX0001] FunctionalizeLoops`** | 3 | 循环 monadic lowering 未实现：`gat/lending-iter`、`assoc-type/iter-style`、`aeneas-limit/return-inside-nested-loop` |
| **D. `[HAX0001] AST import` 其他** | 4 | `hax-limit/let-chains`（issue #2018）、`charon-limit/async-fn`（issue #924 coroutines）、`hax-limit/mut-arg-pattern`（issue #1405 `[HAX0011] AndMutDefsite`）、`closure-adv/boxed-dyn-fn`（type Dyn with non-trait predicate） |
| **E. `[HAX0008] reject_ArbitraryLhs`** | 1 | `refcell/borrow/refcell_borrow_mut`（`*c.borrow_mut() = 42;` 左值非 place expr） |
| **E. `[HAX0008] reject_TraitItemDefault`** | 1 | `trait-obj/conditional-method` |
| **F. `[HAX0002]` F\* backend OCaml 异常** | 1 | `repr/union/repr_union`：F\* backend 自身抛 `Failure`/`Not_found`-like 异常，stderr 末尾打印完整 OCaml 调用栈，定位 `backends/fstar/fstar_backend.ml:241` |
| **G. industrial 工业代码 lint→error** | 2 | `industrial/x509-parser/cert-parse/{x509_parse_der, x509_subject_extensions}`：vendor crate 的 `unnecessary qualification` lint 在 nightly 下被升级为 error，cargo build 在 hax-engine 启动前失败 |
| **H. 底层 rustc / hax frontend 栈溢出** | 1 | `trait/cyclic-bound/cyclic_bound_use`：`thread 'rustc' has overflowed its stack` |
| **I. `[HAX0002]` AST import 内部 error** | 1 | `charon-limit/inline-asm/nop_via_asm`：`expression Todo InlineAsm(...)` |

数量加总：6+11+3+4+1+1+1+2+1+1 = 31 ✓

stderr 形态稳定，举一例（`closure/fn-fnmut/closure_fnmut`）：

```
error: [HAX0003] The mutation of this &mut is not allowed here.
This is discussed in issue https://github.com/hacspec/hax/issues/420.
Note: the error was labeled with context `DirectAndMut`.
  --> src/lib.rs:11:20
   |
11 |       let mut incr = || {
12 | |         count += 1;
```

经常 `[HAX0003] DirectAndMut` 与 `[HAX0006] LocalMutation` 一起出现——hax 在闭包对外层局部变量做赋值的 `LocalMutation` 阶段显式拒。或与 `[HAX0010]` 一起出现，对应 `*x.borrow_mut() = ...` 这种"通过 deref 赋值"形态。

**`[HAX0008] reject_RawOrMutPointer` 是 hax-fstar 在本矩阵下的最高频信号**——22 次出现（含同 entry 多触发器叠加）。这一族 reject 完全没在 hax-lean 的 stderr 里出现过：相同源代码下 hax-lean 的 Lean Printer 把这些节点替换为 `sorry` 继续打印（被 hax-lean oracle 抓为 silent partial），而 hax-fstar 在 phase 阶段就显式拒。

## 与本次测试边界的关系

- 测试切割点：`.fst` 落盘 + cargo hax exit 0 + 产物 grep 不命中 failure 字面 → SUCCESS。**未触达**：生成 `.fst` 是否 F\* 端可 type-check、是否需要 Karamel 引入——超出测试范围
- 三 hax backend 的核心差异在 printer-level：F\* Printer 与 Lean Printer 在 `[HAX0003]/[HAX0008]/[HAX0010]/[HAX0011]` 这族 reject phase 的触发上**对称且相反**——本矩阵下 mut-ref / raw-ptr / closure-captures-mut 类构造在 hax-fstar 上稳定 fail（B 桶 11 + A 桶 6 = 17 条），在 hax-lean 上则走 silent sorry path（hax-lean 的 oracle 才把它们抓回 FAILED）；反过来 `dyn Trait` / `equality constraints on associated types` 类构造在 hax-lean 的 Lean Printer 上 `[HAX0001]` 拒（hax-lean B 桶 9 条），在 hax-fstar 上很多反而 SUCCESS（如 `lifetime/static-bound/static_bound`、`trait-obj/dyn-dispatch`）
- `hax-limit/*` 是 hax 项目自声明的"已知不支持构造"集合。本矩阵下 hax-fstar 在 8 个 entry 上 6 个 FAILED（hax-lean 4 个 FAILED，hax-coq 8 个全 FAILED）——hax 自陈的限制范围在 F\* backend 上**部分兑现**，剩余两条（`labelled-break`、`unsafe-block`）在 hax-fstar 上反而 SUCCESS

## 历史快照声明

本报告所有数字基于 `runs/run-1778226613-5282`（2026-05-08）+ hax `30949eb8` + nightly-2025-11-08 + opam hax-engine。F\* Printer 在本快照下"几乎不走 silent path"是经验事实，未来上游修改 phase 顺序或新增构造翻译规则可能改变这条性质——届时 oracle 的 grep 防御层（`Rust_primitives.Hax.failure` / `failure ((`）将作为兜底信号生效。
