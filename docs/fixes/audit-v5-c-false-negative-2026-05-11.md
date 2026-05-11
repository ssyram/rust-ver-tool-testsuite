# v5 0 漏报独立 audit (c 阶段)

> R5-c：基于 v5 matrix（`runs/run-1778500291-90812/`）的 0 漏报独立审查。
>
> **只观察、不修改、不 commit**。漏报候选留待 cc 阶段 counter-challenge 后由用户决策。
>
> 数据基线：2026-05-11T11:51:31Z run，host = Apple M5 / macOS 25.4.0 / 10 cores / 24 GB / parallelism 10。

---

## §0 元数据 + 漏报定义

### 0.1 数据源

- **v5 run**：`runs/run-1778500291-90812/results.json` —— 20 工具 × 161 entries = 3220 results
- **状态分布**：SUCCESS = 2216 / FAILED = 892 / UNKNOWN = 112
- **每工具 SUCCESS 数**：

| 工具 | SUCCESS | total | rate |
|---|---:|---:|---:|
| cargo-check | 161 | 161 | 100% |
| miri | 157 | 161 | 97.5% |
| charon-poly | 154 | 161 | 95.7% |
| charon-mono | 153 | 161 | 95.0% |
| kani | 151 | 161 | 93.8% |
| hax-fstar | 128 | 161 | 79.5% |
| hax-lean | 125 | 161 | 77.6% |
| rocq-of-rust | 124 | 161 | 77.0% |
| rocq-of-rust-typecheck | 124 | 161 | 77.0% |
| soteria | 124 | 161 | 77.0% |
| creusot | 121 | 161 | 75.2% |
| hax-coq | 111 | 161 | 68.9% |
| aeneas-coq | 102 | 161 | 63.4% |
| aeneas-fstar | 102 | 161 | 63.4% |
| aeneas-lean | 102 | 161 | 63.4% |
| prusti | 71 | 161 | 44.1% |
| aeneas-hol4 | 66 | 161 | 41.0% |
| verus | 66 | 161 | 41.0% |
| kmir | 61 | 161 | 37.9% |
| verifast | 13 | 161 | 8.1% |

### 0.2 漏报定义（按宪法 §六 + tool-integration.md §四）

> **漏报 = oracle 判 SUCCESS 但工具内部实际 silent partial / skip / 不完整翻译 / 不完整执行**。

即"虚假 SUCCESS"——oracle 没抓住的"工具自陈我没全干完"。这是 0 漏报硬指标的核心。

> "工具自陈'我没全干完'必须被尊重" —— `principles.md` §六-2 Oracle 责任

---

## §1 审查方法

### 1.1 可用 audit 数据

- `runs/run-1778500291-90812/raw/<tool>/<entry>.{stdout,stderr,exit}` —— 完整保留
- `work/` 目录被运行时清理 —— **产物级 grep 无法重现**

所以本次 audit **只能基于 stdout/stderr + 源码层论证**，不能直接看 `.lean` / `.fst` / `.v` / `.vpr` 等翻译产物。

### 1.2 审查目标

按 tool-integration.md §四 0 漏报三层：

1. **形式可证 0 漏报**（aeneas × 4 / charon × 2 / creusot / cargo-check / miri / kani / kmir / prusti / soteria / verifast / verus）：源码层是否真单一通路
2. **实测验证 0 漏报**（hax × 3 / ror × 2）：实测样本是否够多 + 防漏报机制（grep / 多门）是否真覆盖

### 1.3 审查路径

对每个 SUCCESS entry 的 stderr / stdout：

- 翻译类：grep `not yet supported / not handled / generated.*warnings / Please report / Type error after transformations / silent / sorry / Admitted / Inline assembly`
- 求解类：grep `does not support concurrency / atomic intrinsic / sequential code / over-approximation`
- 执行类：grep `K-stuck / unsupported MIR / caller_location` 等

### 1.4 漏报候选筛选门槛

漏报候选必须满足三条：

1. `status == SUCCESS`（oracle 判 pass）
2. stderr / stdout 含**工具自陈** partial signal
3. signal 不是工具的"合理设计选择"（如 creusot 对 external fn 不强制 contract、verus `--no-verify` 跳 SMT 层）

---

## §2 各工具漏报扫描

### 2.1 cargo-check（161 SUCCESS / 100%）

**已知盲点**：无（README 自陈"形式可证 0 漏报"）

**扫描结果**：所有 SUCCESS stderr 干净（无非 lint warning）。rustc exit code 即 type check / borrow check 完成的全充要条件——形式严格性自洽。

**漏报候选**：0

---

### 2.2 miri（157 SUCCESS / 97.5%）

**已知盲点**：无（README："miri exit 0 ⇔ 解释执行完整跑完且无 UB / unsupported operation"；"miri 不存在 silent skip"）

**扫描结果**：

| entry | stderr 信号 | 判定 |
|---|---|---|
| `unsafe-ptr/raw-ptr-const/raw_ptr_const_match` | `warning: integer-to-pointer cast ... Miri might miss pointer bugs in this program` | **不算漏报**——miri 仍完整执行，是"可能漏 bug"提醒（与 `MIRIFLAGS=-Zmiri-strict-provenance` 相关），不是 partial 翻译/执行。但 README 声明"无盲点"过严，应明示此种 provenance-soundness 缺口属于 miri 已知的语义近似 |

**漏报候选**：0（边界声明完善建议 1 条）

---

### 2.3 charon-poly（154 SUCCESS / 95.7%）& charon-mono（153 SUCCESS / 95.0%）

**已知盲点**：无（README："`--abort-on-error` + `register_error!` panic 路径已封死所有 silent skip"）

**扫描结果**：所有 SUCCESS stderr / stdout 干净。`--abort-on-error` 实测有效——查询 SUCCESS 全集均无 `warning:` 或 `error:` 字面（除 rustc lint）。

**反例验证**：`charon-limit/copy-deref-closure/deref_copy_in_closure` 在 aeneas-* 上 stderr 含三条 `error: Type error after transformations`，但 charon-mono / charon-poly 该 entry stderr 干净——因 mono / poly 路径都加 `--abort-on-error`，且 mono 重写后这些 `Bound(1, 0)` type error 不触发（aeneas pipeline 用的是不同 charon 调用，见 §2.5）。

**漏报候选**：0

---

### 2.4 kani（151 SUCCESS / 93.8%）

**已知盲点**（README）：

- `caller_location` / `foreign function` 高频警告——kani 对 std panic / alloc 路径的标准 stub；不抓避免误报
- 5 markers `(TerminatorKind::InlineAsm | simd_cast | catch_unwind | ptr_mask | C string literal)` 抓 hard-unsupported codegen stub
- 未来 kani 新增 unsupported MIR 节点类别需扩展 5 markers list

**扫描结果**：v5 SUCCESS entries 中，有 8 个 entries 出现 **`Kani currently does not support concurrency. The following constructs will be treated as sequential operations`** —— kani 自陈把并发原语**当作 sequential 替换**，但 5 marker list **不**覆盖这条 warning。具体构造：

| entry | 不支持构造（stdout） | grep 片段 |
|---|---|---|
| `arc/clone-drop/arc_clone_drop` | atomic_xsub (5) / atomic_xadd (5) / atomic_fence (4) | `does not support concurrency` |
| `charon-limit/arc-slice-unsize/arc_array_to_slice` | atomic_fence (4) / atomic_xsub (5) | 同 |
| `collections/hashmap/hashmap_basic` | thread local (replaced by static variable) (1) | 同 |
| `concurrency/atomic/atomic_seqcst` | atomic_store (3) / atomic_load 等 | 同 |
| `creusot-limit/thread-local-ref/read_thread_local` | thread local (replaced by static variable) (1) | 同 |
| `industrial/rsa/rsa-pkcs8/rsa_pkcs1v15_encrypt` | atomic_singlethreadfence (4) | 同 |
| `lifetime/thread-local/thread_local_read` | thread local (replaced by static variable) (1) | 同 |
| `miri-limit/weak-memory-incomplete/relaxed_load_may_not_observe_all_stores` | atomic_store (3) | 同 |

stdout 完整片段（示例 `arc/clone-drop`）：

```
warning: Kani currently does not support concurrency. The following constructs will be treated as sequential operations:
             - atomic_xsub (5)
             - atomic_xadd (5)
             - atomic_fence (4)
```

**判定**：这是 kani 明确自陈"我把这条并发原语替换为 sequential 等价"——与 README 的"treated as stub"性质同构。但 5 marker list 故意只列**完全 unsupported** 的 MIR 节点（InlineAsm / simd_cast / catch_unwind / ptr_mask / C string literal），漏掉了**并发→sequential 转换**这个软 stub 类。

- **概念边界辨析**：kani 把 `atomic_*` 替换为 sequential 在**单线程语义**下等价（kani 是 BMC，本就不模拟多线程交错）——所以从 kani 自身能力定义看不是"partial"，是"kani 设计上不模拟交错"。
- **按宪法 §六-2 严格解读**：工具自陈"我没把这条并发原语干完整"，与"完整完成"精神冲突。这条 warning 字面就是 kani 自陈 partial（"treated as sequential operations"=替换语义）。

**漏报候选**：8 entries（按宪法严格解读）

**修复路径建议**（仅建议，不动手）：在 kani-strict-wrapper.sh 的 5 marker grep 中追加：

```
\bdoes not support concurrency\b
```

或单独追加 atomic_* / thread local 作为 markers。**风险**：可能引入误报（任何用 `Mutex` / `Atomic*` 的 std code 都会触发，包括 `arc/clone-drop` 这种合法 ARC 使用）。须经 cc 阶段反双向验证。

**漏报盲点 README 自陈 vs 实测对比**：README 5 markers 漏掉 concurrency stub 类——README §"漏报盲点"应补"kani 不模拟并发原语交错，atomic_* / thread local 被替换为 sequential semantics，在多线程语义下是 silent partial"。

---

### 2.5 aeneas-coq / aeneas-fstar / aeneas-lean / aeneas-hol4

各 102 / 102 / 102 / 66 SUCCESS。aeneas 系列 README 都自陈"✅ 形式可证 0 漏报。aeneas exit 0 ⇔ `Errors.error_list` 空"。

**关键缺口**：aeneas-* wrapper 的 pipeline 是 **charon → aeneas**，wrapper 用 `charon cargo --preset=aeneas`（aeneas-{coq,fstar,lean}-wrapper.sh 均如此）。这个调用**不**带 `--abort-on-error`——与 charon-mono / charon-poly 的 tool.toml 不同！

**实测发现**：

#### 2.5.1 `charon-limit/inline-asm/nop_via_asm`（4 backends 都 SUCCESS）

stderr（aeneas-lean 实测，3 个 backend 同）：

```
warning: Inline assembly is not supported
  --> src/__ts_inner.rs:31:13
   |
31 |             core::arch::asm!("");
   |             ^^^^^^^^^^^^^^^^^^^^

warning: The extraction generated 1 warnings
```

stdout 显示：`Translated opaque functions: 1/1` / `Translated transparent functions: 0/0`。

**判定**：entry_fn `nop_via_asm` 包含 `asm!("")`——唯一的 fn。charon **silent** 把它降级为 opaque function（即"我不会翻这条，把它声明为外部黑盒"），aeneas 接受 1/1 opaque + 0/0 transparent。aeneas exit 0，但**entry_fn 在产物里没有 body**——这是**真 silent partial**。

#### 2.5.2 `charon-limit/copy-deref-closure/deref_copy_in_closure`（aeneas-coq / fstar / lean 3 backend 都 SUCCESS，hol4 FAILED）

stderr（aeneas-lean 实测）：

```
error: Type error after transformations:
       Found incorrect clause var: Bound(1, 0)
       Visitor stack:
         charon_lib::ast::types::TraitRef
         ...
       Binding stack (depth 2):
         0: 
         1: <'_0, T>
  --> src/__ts_inner.rs:20:15
   |
20 |     let get = || *x;
   |               ^^^^^

(此 error 重复 3 次)

warning: The extraction generated 3 warnings
```

**判定**：charon 内部 `--preset=aeneas` 路径触发 3 个**type error**——但因为没有 `--abort-on-error`，charon 把 type-error 降级为 warning，继续输出 .llbc。aeneas 接收（不完整的）.llbc，跑完 `Translated transparent functions: 6/6`，exit 0。

这是 **charon → aeneas pipeline 的 silent partial**——aeneas 本身没违反 craise 唯一通路，但 **charon stage emit error 后继续**，aeneas 接受的 .llbc 是 partial。

**漏报候选清单**：

| 工具 | entry | 证据 | 修复路径 |
|---|---|---|---|
| aeneas-coq | charon-limit/inline-asm/nop_via_asm | stderr "Inline assembly is not supported" + "The extraction generated 1 warnings" + stdout "Translated opaque functions: 1/1" / "transparent: 0/0" | charon `--preset=aeneas` 调用追加 `--abort-on-error`，或 wrapper grep charon stderr 含 `extraction generated.*warnings`/`Inline assembly is not supported`/`Type error after transformations` 时 FAILED |
| aeneas-fstar | charon-limit/inline-asm/nop_via_asm | 同 | 同 |
| aeneas-lean | charon-limit/inline-asm/nop_via_asm | 同 | 同 |
| aeneas-hol4 | charon-limit/inline-asm/nop_via_asm | 同 | 同 |
| aeneas-coq | charon-limit/copy-deref-closure/deref_copy_in_closure | stderr `error: Type error after transformations` 3 次 + `extraction generated 3 warnings` | 同 |
| aeneas-fstar | 同 | 同 | 同 |
| aeneas-lean | 同 | 同 | 同 |

**漏报候选**：7 entries（4 backends × 2 partial entries，hol4 在第 2 个上 FAILED）

**README 自陈 vs 实际验证**：

aeneas-* README 自陈"✅ 形式可证 0 漏报"——但论证只覆盖 **aeneas 本身的 craise 唯一通路**，**没有把 charon stage 的 partial 包入 pipeline 的 0 漏报论证**。pipeline-level 0 漏报需要：

```
[ charon stage exit 0 ∧ charon stderr 无 partial 信号 ] ∧ [ aeneas error_list 空 ] ⟹ 真 SUCCESS
```

但 wrapper 的 `charon cargo --preset=aeneas` **没** `--abort-on-error`，charon stage 的"silent partial via type-error-degraded-to-warning"是真漏报路径。

aeneas-* README 应补 charon stage 0 漏报论证（或加 `--abort-on-error`，或加 wrapper grep）。

---

### 2.6 hax-fstar（128 SUCCESS / 79.5%）& hax-lean（125 SUCCESS / 77.6%）& hax-coq（111 SUCCESS / 68.9%）

**已知盲点**（README）：

- hax engine 完全 skip item（item 既不写 sorry / failure 也不发 Diagnostic，**不出现在产物里**）——已通过 entry_fn 定义存在性 grep（`def`/`opaque` for lean，`let`/`let rec`/`and` for fstar，`Definition`/`Fixpoint`/`Lemma` for coq）防御
- 上游引入新 silent path 不带已知 markers

**扫描结果**：所有 hax-* SUCCESS stderr 干净——纯 cargo 编译信息 + `hax: wrote file ...`。

**可用 audit 数据局限**：本次只能看 stderr/stdout。产物级 grep（如 `sorry` term position / failure marker / entry_fn `def` 存在性）**已在 wrapper 内部执行**——若失败则 oracle 已 FAILED。所以"已 SUCCESS"意味着 wrapper grep 路径未命中——**当前 audit 阶段无法独立重验这条路径**（产物已清）。

**源码层论证审视**（不可独立验证但可质疑）：

- hax-lean 的 silent path：`rust-engine/src/backends/lean.rs:1287, 2163` 的 `PatKind::Error / error_node` → `text!("sorry")`。文档说"两行都是 PatKind::Error / error_node 路径"——但用户没独立 audit 整个 lean.rs 是否有第三个 silent path
- hax-fstar：`fstar_backend.ml:1771` 的 `Use _ | NotImplementedYet -> []`——唯一 silent path？文档说"line 506-512 的 `pexpr` SpanFreeError catch 不构成 silent path"。这是源码层论证，未独立验证
- hax-coq：`coq_backend.ml:588 method item'_NotImplementedYet = string "(* NotImplementedYet *)"`——抓 entry_fn 缺失代替 marker grep。boilerplate header 与 item-level skip 字面同，逻辑通

**漏报候选**：0（v5 数据 + stdout/stderr 层 0 现象）

**风险声明**：

- 本审仅 stderr/stdout 层 + 设计层。**未独立验证产物 grep 命中率**——cc 阶段可重跑 v5 并 dump 产物做产物级 audit
- hax-* 的"实测验证 0 漏报"基于 corpus 实测 0 现象 + 源码层论证。这两条都成立但不构成形式蕴含——README §"形式严格性"已明示"⚠️ 实测验证 0 漏报，但不可形式证明"，与实际一致

---

### 2.7 rocq-of-rust（124 SUCCESS / 77.0%）& rocq-of-rust-typecheck（124 SUCCESS / 77.0%）

**已知盲点**（README）：

- 上游引入新 silent fallback 路径不带已知 markers（**实测在 examples corpus 0 现象**）
- 完全 skip item 类（`use` / `extern crate` / `macro_rules!`）——合理 skip，不算漏报
- 非确定性翻译路径——N=7 attempts 99.84% catch rate

**oracle 6 gates**：

1. exit 0
2. ≥1 .v 产物
3. 无 0-byte
4. ≥1 产物 > 200 bytes
5. 无 failure marker：`(Error |Unexpected |Please report!|thir failed to compile|Unimplemented )`
6. entry_fn 在某 .v 中以 `Definition <fn>` 形式存在

每 gate 在 N=7 attempts 上 AND-reduce。

**扫描发现**：

#### 2.7.1 `unsafe-ptr/raw-ptr-const/raw_ptr_const_match`（SUCCESS in ror + ror-typecheck）

stderr（7 次 attempts 各一次，共 7 个 warning blocks）：

```
warning: This kind of constant in patterns is not yet supported.
 --> src/lib.rs:9:9
  |
9 |         DISGUISED_INT => 1,
  |         ^^^^^^^^^^^^^

warning: 1 warning emitted
```

**源码追踪**：

```rust
// /tmp/rocq-of-rust-clone/lib/src/thir_pattern.rs:165-176
                    // TODO: handle other kinds of constants
                    _ => {}
                }
            }
            emit_warning_with_note(
                env,
                &pat.span,
                "This kind of constant in patterns is not yet supported.",
                None,
            );

            Rc::new(Pattern::Wild)   // 替成 wildcard
        }
```

**判定**：

- ror 把不支持的 const pattern（`DISGUISED_INT` 是 `*const ()` 类型 const）**silent 替换为 wildcard `_`**
- 产物里 entry_fn `raw_ptr_const_match` 仍以 `Definition <fn>` 存在 —— gate 6 命中
- 产物里 match 现在是 `match p with | _ => 1 | _ => 0` —— 第一个 case 永远命中，**语义改变**
- 产物里**没有** marker `(* Error |Unexpected |Please report!|thir failed to compile|Unimplemented *)`——所以 gate 5 不抓

stderr 里 "warning: This kind of constant in patterns is not yet supported" 是 ror 通过 `tcx.sess.dcx().struct_span_warn` 发的 rustc-style warning（不升 error）——**所有 7 次 attempts 都触发**（注意：stderr 含 7 个相同 block，说明 ror 在所有 N attempts 上一致 silent skip）。

**README 自陈 vs 实测不符**：README §"漏报盲点"明示"上游引入新 silent fallback 路径不带已知 markers（**实测在 examples corpus 0 现象**）"——但 v5 实测有 **1 个 entry 现象**！应更新 README "0 现象 → 1 已知现象（unsafe-ptr/raw-ptr-const/raw_ptr_const_match，详见 audit-v5-c-false-negative-2026-05-11.md §2.7.1）"。

#### 2.7.2 其他 thir_pattern.rs silent paths（理论存在但 v5 0 现象）

`thir_pattern.rs` 还有其他 `emit_warning_with_note + Rc::new(Pattern::Wild)` silent path：
- `PatKind::Range(_)`：`"Ranges in patterns are not yet supported."`
- `PatKind::Never`：`"Never patterns are not yet supported."`
- `PatKind::Error(_)`：`"Error patterns are not yet supported."`
- `PatKind::DerefPattern { .. }`：`"Deref patterns are not yet supported."`

v5 corpus 上**均未触发**——但理论上可以漏报。

#### 2.7.3 `thir_expression.rs` silent fallback

多处 `emit_warning_with_note` + `Expr::Comment(...)` —— Comment 形式会把 message 写入产物字面，能被 gate 5 marker grep 抓住（如 `Please report!`）。这条路径**不是漏报**。

但 `thir_expression.rs:452` 的 `emit_warning_with_note(env, span, "Unknown integer type", Some("Please report 🙏"))` 与 `"unknown_kind_of_integer".to_string()` 字面替换——这条**不**写 Comment 到产物（直接拼字符串），可能漏。v5 0 现象。

**漏报候选**：

| 工具 | entry | 证据 | 修复路径 |
|---|---|---|---|
| rocq-of-rust | unsafe-ptr/raw-ptr-const/raw_ptr_const_match | stderr 7 个 attempts 都含 "warning: This kind of constant in patterns is not yet supported." + 源码 `thir_pattern.rs:165-176` silent `Pattern::Wild` 替换 | gate 5 marker 扩展加 `"is not yet supported"` 字面（但需双向验证，hax / ror prelude 是否有合法用户字面）；或 wrapper grep ror stderr 含 "warning: .* is not yet supported" 时 FAILED |
| rocq-of-rust-typecheck | 同 | 同 | 同（tier-1 依赖 tier-0 6 gates，应同步加） |

**漏报候选**：2 entries（同 entry，2 个工具）

---

### 2.8 soteria（124 SUCCESS / 77.0%）

**已知盲点**：无（README："✅ 形式可证 0 漏报。exit 1/2/3 完整覆盖 bug detect / symex crash / 前端 crash 三类 partial"）

**扫描发现**：

| entry | stdout 信号 | 判定 |
|---|---|---|
| `concurrency/atomic/atomic_seqcst` | `warning: An atomic intrinsic was encountered; it will be executed as sequential code` | soteria 自陈 "执行为 sequential code"——partial 执行 |
| `miri-limit/weak-memory-incomplete/relaxed_load_may_not_observe_all_stores` | 同 | 同 |
| `float/transcendental/float_transcendental` | `warning: A complex floating point intrinsic was encountered; it will be executed with a significant over-approximation.` | soteria 自陈 "用 significant over-approximation 执行"——partial 执行（语义近似） |
| `kani-limit/float-overapprox/trigger_check_sin_cos_identity` | 同 | 同 |

**判定**：与 kani 同类问题——soteria 把无法精确建模的并发 / 复杂浮点 intrinsic **silent 替换**为简化语义。soteria README 说 "soteria exit 0 ⇔ 符号执行完成且无 bug"——技术上**仍完成**，但与"完整完成"精神冲突（按 §六-2"工具自陈'我没全干完'必须被尊重"）。

**漏报候选**：4 entries

**修复路径建议**：soteria wrapper 加 grep stdout `warning:.*intrinsic was encountered` 或 `executed as sequential|over.approximation` —— 双向验证 OK 后 FAILED。

**README 自陈 vs 实际**：README "无盲点" 与实际"4 个 entries 含 silent partial intrinsic 替换"不符——应补"atomic / complex float intrinsic 被替换为 sequential / over-approximation，按完整完成精神是 silent partial"。

---

### 2.9 creusot（121 SUCCESS / 75.2%）

**已知盲点**：无（README："creusot 用 `crash_and_error / span_err / span_fatal` 把所有 unsupported 升级为 rustc error，无 silent path"）

**扫描发现**：

非常多 SUCCESS entries 含 `warning: calling external function 'X' with no contract will yield an impossible precondition`——粗略统计 35+ entries。例如：

- `bigint/bigint-bitwise/bigint_bitwise`：`bitand` / `bitor` / `bitxor`
- `collections/hashmap/hashmap_basic`：`new` / `to_string` / `insert`
- `concurrency/atomic/atomic_seqcst`：`new` / `fetch_add`
- `float/transcendental/float_transcendental`：`sqrt`
- `miri-limit/networking-unsupported/tcp_connect_attempt`：`connect`

**判定**：creusot 对**外部 fn**（不在当前 crate 内）不强制 contract，只 warning。这是 creusot 的设计选择——**外部 fn 不属于当前 crate 翻译范围**，按 §六-3 前端测量边界（当前 crate 翻完即 SUCCESS），**不算漏报**。

但**边界声明缺失**——README 应补："creusot 对当前 crate 外部 fn 不强制 contract，只 warning"——读者可知 SUCCESS 不蕴含"已 verify 所有 fn"。

**漏报候选**：0（边界声明缺失，建议补 README，但不动手）

---

### 2.10 verus（66 SUCCESS / 41.0%）

**已知盲点**：无（README："✅ 形式可证 0 漏报。Verus 任何 rejection 都通过 `dcx().emit` 触发 exit ≠ 0"）

**扫描发现**：

| entry | stderr 信号 | 判定 |
|---|---|---|
| `aeneas-limit/float-types/make_measurement` | `warning: autoderive Clone impl does not take the form Verus expects; continuing, but without adding a specification for the derived Clone impl` | verus 自陈 "**continuing, but without adding a specification**"——silent skip 一个 derived Clone impl 的 spec |

**判定**：

- 表面：verus exit 0，且 stderr 含明确"我跳过了 X 的 spec"
- 设计辨析：verus 用 `--no-verify`，本就只做 VIR 构造——derived Clone impl 是否构造 spec 是 verus 内部决策。这条 warning 是 verus 对 `#[derive(Clone)]` 的特殊处理"我没生成 default spec"——**VIR 仍构造完成**
- 按"完整完成"精神严格解读："continuing without adding a specification" 是工具自陈 partial（spec 没加）

**漏报候选**：1 entry（按严格解读）

**修复路径建议**：verus wrapper 加 grep stderr `warning:.*continuing.*without adding` 时 FAILED。**风险**：可能误报合法的 `--no-verify` workflow——cc 阶段必须双向验证。

**README 自陈 vs 实际**：README "无盲点" 与"v5 1 个 entry 含 silent spec skip" 不符。verus 设计上 "VIR 构造完成" 不蕴含 "所有 derived impl 都有 spec"——README 应补此边界。

---

### 2.11 prusti（71 SUCCESS / 44.1%）

**已知盲点**：无（README："Prusti 任何 unsupported feature → `[Prusti: ...]` marker + exit ≠ 0；即使未来 toolchain drift 让 encoder fast-path silent skip lower → wrapper 检 .vpr 数量为 0 → FAILED"）

**扫描发现**：所有 SUCCESS stderr 干净——无 `[Prusti: unsupported feature]` 或 `[Prusti: internal error]` 字面。

prusti wrapper 双门：exit 0 + .vpr 数量 ≥1。第二门是产物层（已清，无法独立 audit）。

**漏报候选**：0（基于 stderr/stdout 层）

---

### 2.12 verifast（13 SUCCESS / 8.1%）

**已知盲点**：无（README："`-skip_specless_fns` 跳过 user fn 时 verifast 不会发任何带用户源文件路径的 verbose 行（仅 prelude 行）—— 0 漏报由 verifast 设计强制"）

**扫描发现**：13 个 SUCCESS 都有 ≥22 个 `src/lib.rs` 字符串 mention（grep verbose 行）——wrapper 阈值 ≥1 line，全部远超。oracle 实测有效。

**漏报候选**：0

---

### 2.13 kmir（61 SUCCESS / 37.9%）

**已知盲点**：无（README："K-stuck（K cell 卡在 unsupported terminator）grep 已封死 silent path"）

**扫描结果**：所有 SUCCESS stdout 都含 `#EndProgram ~> .K`——oracle 实测有效。

**漏报候选**：0

---

### 2.14 aeneas-hol4（66 SUCCESS / 41.0%）

aeneas-hol4 在 SUCCESS 上的 charon stage 同 §2.5 — 见 inline-asm 漏报候选 1 个（`copy-deref-closure` 在 hol4 已 FAILED 所以不在此重复）。

---

## §3 候选漏报清单（按工具汇总）

| # | 工具 | entry | 漏报证据 | 修复路径建议 |
|---:|---|---|---|---|
| 1 | rocq-of-rust | unsafe-ptr/raw-ptr-const/raw_ptr_const_match | stderr 7×"This kind of constant in patterns is not yet supported." + 源码 `thir_pattern.rs:165-176` silent `Pattern::Wild` | gate 5 加 marker 或 wrapper grep stderr |
| 2 | rocq-of-rust-typecheck | unsafe-ptr/raw-ptr-const/raw_ptr_const_match | 同 1 | 同 1 |
| 3 | kani | arc/clone-drop/arc_clone_drop | stdout "Kani currently does not support concurrency" + atomic_xsub/xadd/fence | wrapper 5 marker 扩展（注意误报风险） |
| 4 | kani | charon-limit/arc-slice-unsize/arc_array_to_slice | 同 (atomic_fence/xsub) | 同 |
| 5 | kani | collections/hashmap/hashmap_basic | 同 (thread local replaced by static) | 同 |
| 6 | kani | concurrency/atomic/atomic_seqcst | 同 (atomic_store) | 同 |
| 7 | kani | creusot-limit/thread-local-ref/read_thread_local | 同 (thread local replaced by static) | 同 |
| 8 | kani | industrial/rsa/rsa-pkcs8/rsa_pkcs1v15_encrypt | 同 (atomic_singlethreadfence) | 同 |
| 9 | kani | lifetime/thread-local/thread_local_read | 同 (thread local replaced by static) | 同 |
| 10 | kani | miri-limit/weak-memory-incomplete/relaxed_load_may_not_observe_all_stores | 同 (atomic_store) | 同 |
| 11 | soteria | concurrency/atomic/atomic_seqcst | stdout "atomic intrinsic ... executed as sequential code" | wrapper grep "atomic intrinsic.*sequential" |
| 12 | soteria | miri-limit/weak-memory-incomplete/relaxed_load_may_not_observe_all_stores | 同 | 同 |
| 13 | soteria | float/transcendental/float_transcendental | stdout "complex floating point intrinsic ... over-approximation" | wrapper grep "intrinsic.*over.approximation" |
| 14 | soteria | kani-limit/float-overapprox/trigger_check_sin_cos_identity | 同 | 同 |
| 15 | verus | aeneas-limit/float-types/make_measurement | stderr "continuing, but without adding a specification for the derived Clone impl" | wrapper grep "continuing.*without adding" |
| 16 | aeneas-coq | charon-limit/inline-asm/nop_via_asm | stderr "Inline assembly is not supported" + "extraction generated 1 warnings" + stdout "Translated opaque 1/1, transparent 0/0" | wrapper 加 `--abort-on-error` 或 grep charon stderr |
| 17 | aeneas-fstar | 同 | 同 | 同 |
| 18 | aeneas-lean | 同 | 同 | 同 |
| 19 | aeneas-hol4 | 同 | 同 | 同 |
| 20 | aeneas-coq | charon-limit/copy-deref-closure/deref_copy_in_closure | stderr 3×"error: Type error after transformations" + "extraction generated 3 warnings" | 同 16 |
| 21 | aeneas-fstar | 同 | 同 | 同 |
| 22 | aeneas-lean | 同 | 同 | 同 |

**漏报候选总数：22 entries**（覆盖 8 个独立 entry × 2-7 个工具）

按工具分布（漏报候选数）：

1. **kani**：8 entries（concurrency stub 缺失）
2. **aeneas-coq / aeneas-fstar / aeneas-lean**：各 2 entries（charon stage silent partial）
3. **soteria**：4 entries（intrinsic silent replace）
4. **rocq-of-rust / rocq-of-rust-typecheck**：各 1 entry（thir_pattern silent skip）
5. **aeneas-hol4**：1 entry（charon stage silent partial）
6. **verus**：1 entry（derived Clone spec skip）

按独立 entry 分布（唯一 entries 含漏报候选）：

1. `concurrency/atomic/atomic_seqcst`（kani + soteria）
2. `miri-limit/weak-memory-incomplete/relaxed_load_may_not_observe_all_stores`（kani + soteria）
3. `arc/clone-drop/arc_clone_drop`（kani）
4. `charon-limit/arc-slice-unsize/arc_array_to_slice`（kani）
5. `collections/hashmap/hashmap_basic`（kani）
6. `creusot-limit/thread-local-ref/read_thread_local`（kani）
7. `industrial/rsa/rsa-pkcs8/rsa_pkcs1v15_encrypt`（kani）
8. `lifetime/thread-local/thread_local_read`（kani）
9. `float/transcendental/float_transcendental`（soteria）
10. `kani-limit/float-overapprox/trigger_check_sin_cos_identity`（soteria）
11. `unsafe-ptr/raw-ptr-const/raw_ptr_const_match`（ror + ror-typecheck）
12. `aeneas-limit/float-types/make_measurement`（verus）
13. `charon-limit/inline-asm/nop_via_asm`（aeneas × 4）
14. `charon-limit/copy-deref-closure/deref_copy_in_closure`（aeneas × 3）

---

## §4 形式严格性自陈 vs 实际验证

按 README "形式严格性 — 0 漏报"声明 vs c 阶段实测验证：

| 工具 | README 声明 | 实测验证 | 差异 |
|---|---|---|---|
| cargo-check | ✅ 形式可证（rustc exit 单一通路） | ✅ 0 漏报候选 | 一致 |
| miri | ✅ 形式可证（无 silent skip） | 1 个 entry 有 "Miri might miss pointer bugs"（不算漏报） | 一致；建议补"provenance-soundness 缺口"边界 |
| charon-mono | ✅ 形式可证（`--abort-on-error` + `register_error!`） | ✅ 0 漏报候选 | 一致 |
| charon-poly | ✅ 形式可证 | ✅ 0 漏报候选 | 一致 |
| creusot | ✅ 形式可证（crash_and_error / span_err / span_fatal） | ✅ 0 漏报候选（external fn 不算） | 一致；建议补"external fn 不强制 contract"边界 |
| kmir | ✅ 形式可证（K-stuck grep） | ✅ 0 漏报候选 | 一致 |
| prusti | ✅ 形式可证（+ wrapper .vpr 检查） | ✅ 0 漏报候选（stderr 层） | 一致（产物层未独立 audit） |
| soteria | ✅ 形式可证（exit 1/2/3 覆盖三类 partial） | ❌ 4 个漏报候选（intrinsic silent replace） | **不符**——exit code 三类不覆盖"intrinsic silent stub" |
| verifast | ✅ 实测 + 设计论证（`-skip_specless_fns` + verbose lines） | ✅ 0 漏报候选 | 一致 |
| verus | ✅ 形式可证（`dcx().emit` 触发 exit ≠ 0） | ❌ 1 漏报候选（derived Clone spec skip） | **不符**——warning 路径不升 error |
| **kani** | ✅ 实测 + 源码层封堵（5 markers） | ❌ 8 漏报候选（concurrency stub） | **不符**——5 markers 漏 concurrency 类 |
| **aeneas-coq** | ✅ 形式可证（aeneas craise 唯一通路） | ❌ 2 漏报候选（charon stage） | **不符**——aeneas 本身正确，但 pipeline-level charon `--preset=aeneas` 不带 `--abort-on-error` |
| **aeneas-fstar** | 同 | 同 ❌ 2 | 同 |
| **aeneas-lean** | 同 | 同 ❌ 2 | 同 |
| **aeneas-hol4** | 同 | ❌ 1（仅 inline-asm；copy-deref-closure 已 FAILED） | 同 |
| **rocq-of-rust** | ⚠️ 实测验证（"corpus 0 现象"） | ❌ 1 漏报候选（raw_ptr_const_match） | **不符**——README "0 现象"已被推翻 |
| **rocq-of-rust-typecheck** | ✅ 基本可形式证明（coqc typecheck 确定性） | ❌ 1 漏报候选（继承 tier-0） | tier-0 缺口未被 tier-1 修复 |
| hax-coq | ⚠️ 实测 + 源码层封堵 | ✅ 0 漏报候选（stderr 层） | 一致；产物层未独立 audit |
| hax-fstar | 同 | 同 | 同 |
| hax-lean | 同 | 同 | 同 |

**核心差异工具点名**：

- **kani**：5 markers 漏 concurrency 类（8 entries 漏报候选）
- **aeneas × 4**：pipeline-level charon stage 漏报（4-7 entries）
- **soteria**：intrinsic silent replace（4 entries）
- **rocq-of-rust × 2**：thir_pattern silent `Pattern::Wild`（1 entry, README "0 现象"被推翻）
- **verus**：derived Clone spec skip（1 entry）

---

## §5 用户怀疑反方向 spot-check

### 5.1 charon × 2 高通过率（95-96%）是否藏漏报

**结论**：**否**。charon-mono / charon-poly 用 `--abort-on-error`，所有 SUCCESS stderr / stdout 干净。`--abort-on-error` + `register_error!` panic 路径设计上有效——v5 实测 0 漏报候选。

charon 的高通过率反映"charon 翻译能力实际就强"——不是藏漏报。

### 5.2 miri 97.5% 是否藏漏报

**结论**：**否**（基于本审范围）。157 SUCCESS 中只有 1 个 entry 含 "Miri might miss pointer bugs"——但 miri 仍完整 interpret 执行（没 short-circuit）。这是 provenance-soundness 缺口，不是"miri 没执行完"。

miri exit 0 ⇔ UB-free + 无 unsupported operation 在源码层成立（任何 UB / unsupported / panic 都升 error）。

### 5.3 kani 93.8% 是否藏漏报

**结论**：**是**。详 §2.4 — 5 markers 漏 concurrency stub 类，8 个 SUCCESS entries 实际 silent partial。

但漏报量级 **8 entries** ≠ kani 通过率"过高"——kani 在其他 143 SUCCESS entries 上是真完整 codegen（5 markers + caller_location/foreign function 都干净）。

### 5.4 rocq-of-rust-typecheck 77% 是否藏漏报

**结论**：**是**，但量级小（1 entry，继承自 tier-0 ror）。tier-1 typecheck 本身（coqc exit 0）是形式可证 0 漏报——问题在 tier-0 ror 把 unsupported const pattern 翻成 `_` 后产物 typecheck **仍通过**（语义已变但 type 不变）。

**tier-1 的诚实边界**：archive："只测产物可在 Rocq 中编译通过；不测语义正确"。这条 entry 上 silent partial 的语义改变被 typecheck 忽略——是 tier-1 档 1 的合理边界。

---

## §6 总结 + cc 阶段建议

### 6.1 量化总结

- **22 个漏报候选**（覆盖 14 个独立 entries × 1-4 工具）
- **5 个工具**形式严格性自陈与实测不符：kani / aeneas × 4 / soteria / ror × 2 / verus
- **核心缺陷模式**：
  - 软 stub 类（kani concurrency / soteria intrinsic / ror Pattern::Wild）
  - Pipeline-level 上游 partial 不暴露（aeneas → charon stage）
  - Spec 边界缺失（verus derived impl）

### 6.2 cc 阶段反质询要点

漏报候选必须经过 cc 阶段 counter-challenge 后才能落入决策点。以下要点供 cc 阶段反质询：

1. **kani concurrency 类**：
   - 反质询：concurrency stub 在 kani 自身语义（BMC，单线程）下是否构成 partial？
   - 答辩：按"完整完成"精神 + kani 自陈"will be treated as sequential operations"，是 partial；按 kani BMC 设计本就不模拟交错——不是 partial
   - 决策点：用户决定按哪条解读

2. **aeneas × 4 pipeline-level**：
   - 反质询：aeneas 自身 0 漏报（craise 唯一通路）成立；charon stage 的 silent partial 是否属于 aeneas 工具的漏报？
   - 答辩：pipeline-level oracle 是 charon ∧ aeneas，目前 wrapper 只查 aeneas 半边；按整体精神（aeneas-* tool 评测的是 "对样例的翻译能力"）应包入 charon stage
   - 决策点：是否加 `--abort-on-error` 到 `charon --preset=aeneas`（会让 7 个 entries 转 FAILED，影响通过率）

3. **soteria intrinsic 类**：
   - 反质询：soteria 自身设计对 atomic / 复杂 float 用近似是否算"未完成"？
   - 答辩：soteria 自陈"will be executed as sequential code / over-approximation"——按精神是 partial；但 soteria 设计语义如此，"完整完成"在 soteria 上口径需重新校准
   - 决策点：soteria README 改"形式可证"为"⚠️ 实测验证"并加盲点

4. **ror Pattern::Wild**：
   - 反质询：1 个 entry 实测现象推翻 README "0 现象"声明，但这本就是 README §"漏报盲点"列的理论盲点
   - 答辩：README 应更新"v5 实测发现 1 个 entry"，不算盲点声明根本错——但理论盲点是否落地需明示
   - 决策点：更新 README 文字 + 是否补 marker

5. **verus derived Clone**：
   - 反质询：verus 在 `--no-verify` 模式下，derived impl spec 缺失是否算 partial？
   - 答辩：verus VIR 仍构造完成；缺的是"为 derived impl 自动生成 spec 模板"——属 verus spec-gen 边界而非 VIR-construct 边界
   - 决策点：verus README 是否需补"derived impl spec gen 边界"

### 6.3 修复风险

任何漏报修复路径都可能引入新误报，违 0 误报硬指标：

- kani 5 marker 扩加 "concurrency" / "atomic_*"：可能误报合法 ARC / Mutex 使用（如 `arc/clone-drop` 是用户合法 ARC 使用，不应 FAILED）
- aeneas wrapper 加 charon `--abort-on-error`：可能影响"charon stage 真过、aeneas 失败"的纯 aeneas 漏报信号分离能力
- soteria wrapper 加 intrinsic marker：可能误报包含合法 `Mutex` / `f64::sqrt` 的 entries（如 sqrt 在常见数学样例都用）
- ror gate 5 加 "is not yet supported"：双向验证字面是否会在合法注释 / prelude 中出现
- verus wrapper 加 "continuing.*without adding"：可能误报其他 `#[derive]` 形式

**结论**：所有候选必须经过 cc 阶段 **双向 grep 实测**（已知 silent path → 命中 + 合法 SUCCESS → 不命中）才能上线，按 tool-integration.md §四-4.2。

### 6.4 c 阶段产出范围

- 本文 = c 阶段 audit 结果（observation only）
- **未修改任何代码 / tool.toml / wrapper / 文档**
- 漏报候选清单不直接落地——必须经 cc 阶段 counter-challenge

---

## §7 审查局限性声明

1. **work/ 已清空**：本审基于 stdout/stderr 层 + 源码层论证，**未独立产物级验证**。hax × 3 / ror × 2 / aeneas × 4 / prusti / verus 的产物 grep / .vpr / .vir 路径都依赖 wrapper 内部检查——若 wrapper 实现有 bug，本审无法发现

2. **N=7 attempts 重跑**：ror 已用 7 次 attempts 抓非确定性 silent skip。本审看 stderr 是 7 次 attempts 合并的 stderr——若 ror 在某次 attempt 上 silently 翻完整、另一次 silent skip，N=7 AND-reduce 应抓到。但本审无法独立验证 N=7 在每个 entry 上都触发

3. **源码 commit drift**：本审引用的源码（lean.rs / fstar_backend.ml / coq_backend.ml / thir_pattern.rs）基于 README 标的 commit 版本。若工具上游 push 新 commit 引入新 silent path，本审滞后

4. **样例 corpus 偏置**：v5 corpus 161 个 entries 覆盖 11 类（aeneas-limit / bigint / box / charon-limit / ... industrial / unsafe-ptr / 等）。已知漏报候选都基于 corpus 现有 entries——corpus 不覆盖的 silent path 在本审中"0 现象"，**不代表理论上不存在**

---

> 报告创建时间：2026-05-11
>
> 审查者：独立 agent（与 R5-c 任务派遣 agent 不同 session）
>
> 数据：`runs/run-1778500291-90812/`
