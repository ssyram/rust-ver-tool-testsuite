# hax-lean — 特性支持评估报告

## 元数据

- **数据源**：`runs/run-1778226613-5282/`（2026-05-08，146 entries × 19 工具矩阵；host: Apple M5 / macOS aarch64 / 24 GB / 10 cpus，并发 10）
- **工具版本**：`hax untagged-git-rev-30949eb870`（commit `30949eb87058895c24f963df90dd30ef11b0dc1a`）；nightly toolchain `nightly-2025-11-08`；OCaml `hax-engine` + Rust frontend driver `driver-hax-frontend-exporter`
- **工具配置**：`tools/hax-lean/`（`tool.toml`、`harness.rs.tera`、`README.md`）
- **通过率**：SUCCESS 110 / 146 ≈ **75.3%**（FAILED 36，TIMEOUT 0）
- **耗时分布**：avg 3647 ms / median 1859 ms / p90 8957 ms / p95 15833 ms / max 26768 ms（无 entry 触达 600 s timeout）
- **时效声明**：本快照锚定上述 run id + hax commit + nightly 工具链 + corpus，不构成长期承诺。hax 上游对 Lean printer 持续开发中（README 标 active development），新版本对 mut-ref / dyn / sentinel-body 路径的处理变化会让本快照失效。

## 工具内部 pipeline + 前端边界

```
rustc + driver-hax-frontend-exporter
  → THIR JSON
  → hax-engine（OCaml）
  → phase pipeline（reject phases + 改写 passes）
  → Lean Printer（OCaml→Lean 4 文本生成）
  → 写出 <work>/proofs/lean/extraction/<crate>.lean
```

本测试关心 `cargo +nightly-2025-11-08 hax -C --lib ; into lean` 这条命令的"前端通过"。**前端 / 后端切割**：hax 是纯翻译工具，没有 SMT 求解后端——pipeline 终点 `.lean` 落盘即终点。下游 Lean 编译 / Mathlib / 用户证明完全不在测试范围。

`-C --lib ;` 参数语义：把 `--lib` 限制传给底层 cargo（跳过 runner 注入的 `src/bin/__ts_harness.rs` 这层 harness），`;` 是 hax 自己的 cargo-args 终结符。`entry_mode = "bin"`（默认）+ `-C --lib` 配合，使 harness bin 与 hax 翻译互不干扰：harness 仅用于 cargo-check 类工具的 entry-fn 引用，hax 不读它。

## SUCCESS 信号 + 形式严格性

**判定式**（`tool.toml` oracle）：

```
SUCCESS ⟺ cargo hax exit 0 ∧ 产物（strip Lean `--` 行注释后）
          grep 不命中 term-position sorry
```

term-position sorry 的精准 grep：先 `awk '{ sub(/--.*/, ""); print }'` 剥行注释，再 `grep -E '(:=|pure|mk|,)\s*sorry\b|\bsorry\s*[,)\]]'` 抓 `:= sorry` / `pure sorry` / `mk sorry` / `, sorry` / `sorry,)` 等位置——不抓 binder 位置（用户合法 `let sorry : i32 := 5;` 不触发）。

**双轨 partial 暴露机制**：
1. **官方信号**：cargo hax exit 1 = engine emit `FromEngine::Diagnostic`（带 `[HAX####]` 错误码 + GitHub issue 链接）
2. **silent path**：`rust-engine/src/backends/lean.rs:1287, 2163` 的 `PatKind::Error` / `error_node` 路径直接 emit `text!("sorry")`，**不发 Diagnostic**——cargo hax 仍以 exit 0 收尾，但产物含 term-position sorry。这条路径是 hax-lean **特有的** silent partial（hax-fstar / hax-coq 的 backend 没有这条），oracle 必须额外抓住。

**形式严格性**：
- **0 误报**：⚠️ 实测验证，不可形式证明。grep 模式经实测：用户合法 `let sorry: i32 = 5;` 与 doc comment 含 `sorry` 字面字符串都不触发——但理论上无法形式排除未来上游改动引入新合法语法
- **0 漏报**：⚠️ 实测验证，不可形式证明。grep 抓 hax-lean 已知 silent path；上游可能引入新 silent path 而 grep 滞后
- **漏报盲点**：hax engine 完全 skip item（item 既不写 sorry 也不发 Diagnostic、不出现在产物里）—— oracle 抓不到，需对应性脚本暴露（实测在当前 corpus 0 现象）

## 实测结果

### 按 feature 类目分布

下列 feature 类目下 hax-lean **全部 entry 通过**（每类全 SUCCESS）：

```
arc / bigint(8/8) / box / closure-adv(3/4 见下) / collections / const / deps-complex(7/7)
drop / enum / error / float(10/10) / generic / hello / impl-trait / int / int-width(14/14)
iter / panic / prusti-limit(7/8) / rc / slice / vec
```

**部分通过**（数字为 S/总）：`aeneas-limit` 5/8、`charon-limit` 4/7、`closure-adv` 3/4、`concurrency` 1/2、`creusot-limit` 5/7、`hax-limit` 4/8、`industrial` 4/6、`kani-limit` 5/7、`lifetime` 1/3、`miri-limit` 5/7、`prusti-limit` 7/8、`repr` 1/2、`trait-obj` 1/2、`unsafe-adv` 1/3。

**全 FAILED**：`assoc-type`(0/1)、`closure`(0/2)、`gat`(0/1)、`hrtb`(0/1)、`refcell`(0/1)、`trait`(0/1)、`unsafe-ptr`(0/2)。样本量都很小（≤2）。

### 失败模式归类（基于 raw stderr）

36 个 FAILED 按 raw stderr 字面信号归类：

| 类别 | 数量 | 触发信号 |
|---|---:|---|
| **A. silent sorry path（oracle 触发）** | 20 | cargo hax exit 0 但产物含 term-position sorry，oracle 把 rc 改写为 1 并附 `[hax-lean-oracle] FAIL: silent partial — sorry in term position (lean.rs:1287/2163 PatKind::Error / error_node path)` |
| **B. `[HAX0001]` Lean Printer 阶段 todo** | 9 | `Unsupported `dyn` traits`（issue #1708）/ `Unsupported equality constraints on associated types of parent trait`（issue #1923）/ 其他 Lean Printer 内部 todo |
| **C. `[HAX0002]` Name rendering（InlineConst / Union）** | 3 | `repr/union/repr_union`（DefIdInner kind: Union）、`creusot-limit/thread-local-ref` 与 `lifetime/thread-local`（thread_local! 宏展开的 InlineConst） |
| **D. industrial 工业代码 lint→error** | 2 | `industrial/x509-parser/cert-parse/{x509_parse_der, x509_subject_extensions}`：vendor crate 的 `unnecessary qualification` lint 在 nightly 下被升级为 error，`error: could not compile x509-parser (lib) due to 7 previous errors`，hax-engine 没机会跑 |
| **E. 底层 rustc / hax frontend 栈溢出** | 1 | `trait/cyclic-bound/cyclic_bound_use`：`thread 'rustc' has overflowed its stack / fatal runtime error: stack overflow / signal: 6, SIGABRT`，发生在 `driver-hax-frontend-exporter` 阶段 |
| **F. industrial 其他失败** | 1 | x509-parser 第二条已计入 D；归桶后实际 D=2，F=0 |

**A 桶（silent sorry）的 20 条**清单（按 entry_id 字典序）：

```
aeneas-limit/return-inside-nested-loop/outer_break_label
aeneas-limit/trait-impl-mut-param-mismatch/trigger_trait_impl_mut_param_mismatch
assoc-type/iter-style/assoc_type_iter
charon-limit/async-fn/async_forty_two
charon-limit/inline-asm/nop_via_asm
closure/fn-fnmut/closure_fn
closure/fn-fnmut/closure_fnmut
closure-adv/boxed-dyn-fn/boxed_dyn_fn
gat/lending-iter/gat_lending
hax-limit/closure-mutates-outer/closure_mutates_outer
hax-limit/mut-in-assoc-type/hax_limit_mut_in_assoc_type
hax-limit/mut-ref-alias/hax_limit_mut_ref_alias
hax-limit/ret-mut-ref/hax_limit_ret_mut_ref
kani-limit/async-await/run_async_add
miri-limit/simd-bitmask-large-vector/trigger_bitmask_over_64_elements
refcell/borrow/refcell_borrow_mut
unsafe-adv/maybe-uninit/unsafe_maybe_uninit
unsafe-adv/ptr-write/unsafe_ptr_write
unsafe-ptr/raw-ptr-const/raw_ptr_const_match
unsafe-ptr/raw-read/raw_ptr_read
```

这 20 条在 hax-fstar / hax-coq 上几乎都以 `[HAX0001]` / `[HAX0003]` / `[HAX0008]` 形态显式 fail 出来；hax-lean 的 Lean Printer 把对应 AST 节点替换为 sentinel `sorry` 后继续打印，cargo hax 退出 0——**不依赖产物 grep oracle 就会被冤判 SUCCESS**。这是宪法 §六-2"不允许 partial"在 hax-lean 上的具体落地：sentinel body 即工具自陈"我没全干完"，必须 → FAILED。

**B 桶 stderr 形态稳定**，举一例（`trait-obj/dyn-dispatch/dyn_dispatch`）：

```
error: [HAX0001] something is not implemented yet.
Unsupported `dyn` traits
This is discussed in issue https://github.com/hacspec/hax/issues/1708.
Note: the error was labeled with context `Lean Printer`.
```

**C 桶**：stderr 显示 hax 在路径名渲染阶段对 `thread_local!` 宏展开的 anonymous const（`InlineConst`）和 `union { ... }` 类型项无法生成名字，落到 `[HAX0002]` Name rendering 路径。

## 与本次测试边界的关系

- 测试切割点：`.lean` 文件落盘 + cargo hax exit 0 + oracle 产物 grep 不命中 → SUCCESS。**未触达**：生成 `.lean` 是否 Lean 可编译、是否需要 hax Lean prelude（`hax/proof-libs/lean/`）、Mathlib 引入与否——这些超出本测试范围。
- A 桶（silent sorry path）20 条在 oracle 不抓产物的旧版本配置下会齐刷刷 SUCCESS，把整体通过率从 75% 拉高到约 89%。这是宪法 §六-2 的核心约束实例：partial 翻译以 sentinel body 落盘的形态被精确切除。
- 三 hax backend 的 `hax-limit/*` 表现差异最显著：本矩阵下 hax-lean 4/8（fstar 2/8，coq 0/8）。原因可读：`hax-limit/*` 是 hax 项目自声明限制集，多条触发 mut-ref / closure-mutates-outer / mut-arg-pattern 等族——hax-fstar / hax-coq 在 phase 阶段以 `[HAX0003]/[HAX0008]/[HAX0011]` 显式 reject，hax-lean 的 Lean Printer 把这些 AST 节点替换为 `sorry` 继续打印，oracle 才把它们打回 FAILED。

## 历史快照声明

本报告所有数字基于 `runs/run-1778226613-5282`（2026-05-08）+ hax `30949eb8` + nightly-2025-11-08 + opam hax-engine。如 hax 合并上游 PR #1672（消除 lean.rs sentinel sorry path），oracle 仍正确——届时 silent path 自然消失，A 桶 20 条会以 `[HAX0001]` 形态落到 B 桶（或被 backend 真正接受），通过率结构相应改变。
