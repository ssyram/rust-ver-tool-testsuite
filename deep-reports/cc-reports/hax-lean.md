# hax-lean — 特性支持评估报告（v6 baseline）

## 元数据

- **数据源**：`runs/run-1778560393-59119/`（2026-05-12 v6 final）
- **工具配置**：`tools/hax-lean/`（`tool.toml`、`harness.rs.tera`、`README.md`）
- **工具版本**：`hax untagged-git-rev-30949eb870`（commit `30949eb87058895c24f963df90dd30ef11b0dc1a`）；nightly toolchain `nightly-2025-11-08`；OCaml `hax-engine` + Rust frontend driver `driver-hax-frontend-exporter`
- **本工具实测**：n=161 / SUCCESS=125 / FAILED=34 / UNKNOWN=2，通过率 **77.6%**
- **时长分布**：avg 2662 ms / median 1401 ms / p90 6106 ms / max 15699 ms（无 entry 触达 600 s timeout）
- **宪法 baseline**：`principles.md` v8（双根本问题 + 原则 A/B/C + Oracle 责任 + UNKNOWN 严格语义）
- **时效声明**：本快照锚定上述 run id + hax commit + nightly 工具链 + corpus，不构成长期承诺。hax 上游对 Lean printer 持续开发中（README 标 active development），新版本对 mut-ref / dyn / sentinel-body 路径处理变化会让本快照失效。

## pipeline + 前端边界

```
rustc + driver-hax-frontend-exporter
  → THIR JSON
  → hax-engine（OCaml）
  → phase pipeline（reject phases + 改写 passes）
  → Lean Printer（OCaml → Lean 4 文本生成）
  → 写出 <work>/proofs/lean/extraction/<crate>.lean
```

**前端 / 后端切割**：hax-lean 是纯翻译工具，pipeline 终点是 `.lean` 文件落盘。下游 Lean 编译 / Mathlib / 用户证明完全不在测试范围。本测试关心 `cargo +nightly-2025-11-08 hax -C --lib ';' into lean` 这条命令的"前端通过"。

**项目维护边界**：`tool.toml` 的 oracle 段（sh -c 内联脚本）是我们维护的产物级 grep gate（双重：term-position sorry + entry_fn 存在性）。除此之外的工具内部 pipeline 全部是上游 hax 的。`-C --lib ;` 让 cargo 只翻 lib，跳过 runner 注入的 `src/bin/__ts_harness.rs` harness——`entry_mode = "bin"` 与 `-C --lib` 配合使 harness bin 与 hax 翻译互不干扰。

## SUCCESS 信号 + 形式严格性

**判定式**：

```
SUCCESS ⟺ cargo hax exit 0
        ∧ 产物（strip Lean `--` 行注释后）grep 不命中 term-position sorry
        ∧ entry_fn 在产物中存在 `def <fn>` / `opaque <fn>` 定义
```

**主信号通路**：`cargo hax` exit code。
- exit 1 = engine emit `FromEngine::Diagnostic`（带 `[HAX####]` 错误码 + GitHub issue 链接）→ FAILED

**wrapper 补抓通路（项目维护，按宪法 §六-2 不允许 partial 派生）**：

1. **silent sentinel path 抓**：`rust-engine/src/backends/lean.rs:1287, 2163` 的 `PatKind::Error` / `error_node` 路径直接 emit `text!("sorry")` 但**不发 Diagnostic**——cargo hax 仍 exit 0。oracle 先 `awk '{ sub(/--.*/, ""); print }'` 剥 Lean `--` 行注释，再 `grep -E '(:=|pure|mk|,)\s*sorry\b|\bsorry\s*[,)\]]'` 抓 term-position sorry——不抓 binder 位置（用户合法 `let sorry : i32 := 5;` 不触发）。
2. **silent skip-item path 抓**（P17 D1 加 gate）：`rust-engine/src/backends/lean.rs:1521 ItemKind::RustModule | ItemKind::Use { .. } => nil!()` 把 `Use` 类型 item 渲染为空 document。oracle grep `^(def|opaque)\s+$TS_ENTRY_FN(\s|\()` 验证 entry_fn 真有定义体——`ItemKind::Fn` 必经 `def <name>` / `opaque <name>`（lean.rs:1472）。

**形式严格性**：

- **0 误报**：⚠️ 实测验证，**不可形式证明**。
  - sorry grep：用户合法 `let sorry: i32 = 5;` 与 doc comment 含 `sorry` 字面字符串都不触发；本 corpus 内多个 SUCCESS entry（hello / enum / arc / bigint / creusot-limit/mutual-recursion）均通过 → 当前 corpus 0 false-positive。
  - 但理论上无法形式排除未来 hax 输出引入新合法语法触发误命中。
- **0 漏报**：⚠️ 实测验证，**不可形式证明**。
  - 两条已知 silent path（sentinel sorry / skip-item Use）均封堵。
  - **漏报盲点**：
    - hax engine 完全 skip item（既不写 sorry / 既不发 Diagnostic / item 不出现）——oracle 抓不到，需对应性脚本暴露；本 corpus 当前 0 现象。
    - 上游未来引入第三条新 silent path 而本 oracle grep 滞后——理论残余风险。
    - 上游 PR #1672 合并后 lean.rs 的 sorry path 应消失，届时 sorry grep 自然失效（无害）。

宪法 §六 "不藏" 落地：以上盲点未隐瞒，列入本节。

## 失败分桶（按 P31 §四.5 归因分类）

### 桶 A：silent sentinel sorry path（oracle 触发，20 case）

代表 entry：`closure/fn-fnmut/closure_fnmut` / `hax-limit/ret-mut-ref/hax_limit_ret_mut_ref` / `unsafe-ptr/raw-read/raw_ptr_read` 等。

stderr 特征：

```
[hax-lean-oracle] FAIL: silent partial — sorry in term position
(lean.rs:1287/2163 PatKind::Error / error_node path)
```

完整清单（按字典序）：

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

**归因**：工具不支持。hax-lean 的 Lean Printer 在遇到不能完整翻译的节点时把对应 AST 替换为 sentinel `sorry` 继续打印——这些条目在 hax-fstar / hax-coq 上多以 `[HAX0001/0003/0008]` 显式 reject，hax-lean 走 silent path。按宪法 §六-2 "不允许 partial"：sentinel body 即工具自陈"我没全干完"必须 → FAILED。
**处理**：不修。本地性原则下 FAILED 站得住，工具开发者不能驳回——oracle 把不显示的 partial 暴露出来正是宪法精神落地。

注：`hax-limit/mut-in-assoc-type` 同时触发 prettyplease backtrace formatting panic（`prettyplease-0.2.37/src/item.rs:1225` `not implemented: ImplItem::Verbatim`）——这是 hax engine 用于诊断打印的依赖在生成错误信息时 panic 但 engine 继续执行并 emit sorry。仍归 A 桶（最终触发是 oracle sorry 抓）。

### 桶 B：`[HAX0001]` Lean Printer 显式 todo / not implemented（10 case）

代表 entry：`trait-obj/dyn-dispatch/dyn_dispatch`、`hrtb/for-all-lifetime/hrtb_apply`、`creusot-limit/dyn-trait-forbidden/trigger_call_dyn_display`。

stderr 特征：

```
error: [HAX0001] something is not implemented yet.
Unsupported `dyn` traits
This is discussed in issue https://github.com/hacspec/hax/issues/1708.
Note: the error was labeled with context `Lean Printer`.
```

完整清单：

```
aeneas-limit/fnmut-closure-unit-return/trigger_fnmut_unit_return
charon-limit/generic-to-dyn-unsize/boxed_display_from_u32
concurrency/thread-mutex/thread_mutex_join
creusot-limit/dyn-trait-forbidden/trigger_call_dyn_display
hrtb/for-all-lifetime/hrtb_apply
kani-limit/stack-unwinding/trigger_divide_with_recovery
lifetime/static-bound/static_bound
miri-limit/thread-interleaving-partial/unsynchronised_counter_race
prusti-limit/spec-entailment-unsupported/trigger_spec_entailment_unsupported
trait-obj/dyn-dispatch/dyn_dispatch
```

**归因**：工具不支持（hax-lean Lean Printer 内 todo 路径，工具上游显式 reject + 链接 GitHub issue）。
**处理**：不修。FAILED 站得住。

### 桶 C：`[HAX0002]` Name rendering / engine fatal（3 case）

stderr 特征：

```
error: [HAX0002] Fatal error: something we considered as impossible occurred!
Please report this by submitting an issue on GitHub!
```

清单：

```
creusot-limit/thread-local-ref/read_thread_local      （thread_local! 宏展开的 InlineConst）
lifetime/thread-local/thread_local_read               （thread_local! 宏展开的 InlineConst）
repr/union/repr_union                                 （DefIdInner kind: Union）
```

**归因**：工具不支持。hax engine 在路径名渲染阶段对 `thread_local!` 宏展开的 anonymous const（`InlineConst`）与 `union { ... }` 类型项无法生成名字，落到 `[HAX0002]` Name rendering 路径——这是 hax 自陈的内部"impossible"状态。
**处理**：不修。FAILED 站得住。

### 桶 E：底层 rustc / hax frontend 栈溢出（1 case）

清单：

```
trait/cyclic-bound/cyclic_bound_use
```

stderr 特征：

```
thread 'rustc' has overflowed its stack
fatal runtime error: stack overflow, aborting
error: could not compile `trait_cyclic_bound` (lib)
process didn't exit successfully: driver-hax-frontend-exporter ... (signal: 6, SIGABRT)
```

**归因**：工具不支持。栈溢出发生在 `driver-hax-frontend-exporter` 阶段（hax 前端 driver，对 cyclic trait bound 处理时栈递归）——属工具自身前端 crash，不是 rustc 通用栈溢出 / 不是我们环境损坏。
**处理**：不修。工具前端 crash → FAILED 站得住。

### UNKNOWN：vendored crate lint strictness（2 case）

清单：

```
industrial/x509-parser/cert-parse/x509_parse_der
industrial/x509-parser/cert-parse/x509_subject_extensions
```

stderr 特征：

```
error: unnecessary qualification
error: unnecessary qualification
...
```

**归因**：**我们 corpus / 配置 bug**。`vendor/x509-parser` 是项目引入的 vendored crate，nightly toolchain 把它的 `unnecessary qualification` lint 升级为 error，driver-hax-frontend-exporter 在 cargo check 阶段就 fail 出来——hax engine 没机会跑。这是 §六 (b) 类 UNKNOWN（"我们这边可识别的问题且暂未修：我们引入的 vendored crate 触发 lint"），oracle 记 `error: 'external_fault: vendor_lint_strictness'`、exit 101。
**处理**：**修**。属"我们导致"。具体方案见下方修订建议。

## v5.1 → v6 ΔS 解释

v5.1（`runs/run-1778491781-11043`，2026-05-11）SUCCESS=125 / 161 = 77.6%；v6（本快照）SUCCESS=125 / 161 = 77.6%，**ΔS = 0**。

桶级变化：v5.1 报告把 vendored x509 两条计入 D 桶（"我们 corpus 引入的 vendored crate lint→error"）作 FAILED；v6 oracle 把这两条以 (b) 类 UNKNOWN 标出（external_fault: vendor_lint_strictness），SUCCESS 数不动，FAILED 36 → 34，UNKNOWN 0 → 2。这是分类合理化、非通过率变化。

## 修订建议清单（仅"我们导致"失败）

| # | 桶 | 涉及 case | 修复方案 | 优先级 |
|---|---|---|---|---|
| 1 | UNKNOWN：vendor x509 lint→error | 2 | 给 `vendor/x509-parser` 加 `#![allow(unused_qualifications)]` lib attr，或在 hirusttest 配置里为这两 entry 注入 `RUSTFLAGS=-A unused-qualifications` / `cap-lints=warn`；属"我们引入的 vendored crate 触发新 toolchain lint"的 (b) 类，可治源 | 中 |

桶 A / B / C / E 共 34 case 均为**工具不支持 / 工具自身 crash**，按"本地性 + 工具能力边界"原则不修，FAILED 站得住，不在修订范围。

**"我们导致"项总数：1（影响 2 entry）**。所有其他 FAILED 均为工具能力边界，不修。
