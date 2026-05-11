# v5 0 漏报 counter-challenge（cc 阶段）

## §0 元数据与方法

- **创建时间**：2026-05-11
- **任务**：对 [`audit-v5-c-false-negative-2026-05-11.md`](audit-v5-c-false-negative-2026-05-11.md) 的 22 个漏报候选按 `charter-craft` §4.8.4 counter-challenge 协议（disprove-first，默认挑刺错）逐条反查
- **数据基线**：`runs/run-1778500291-90812/raw/` 完整 stdout/stderr
- **审查者**：cc 阶段独立 agent
- **不修改**：任何代码 / tool.toml / wrapper / README

### 0.1 disprove-first 协议精神反查锚点

按 [`principles.md`](../design/principles.md) §六：

> **Oracle 责任 — 不冤枉**：SUCCESS 必须是真 SUCCESS——不允许任何 partial / silent skip / 半翻译；**工具自陈"我没全干完"必须被尊重**

按 [`tool-integration.md`](../design/tool-integration.md) §四：

> 0 漏报 = oracle 判 SUCCESS 时**真的没漏抓任何 partial**（不高估能力）

按 §六-3 前端测量：

> 工具能力的测量限于工具自身**前端**（parser / 类型检查 / 翻译 / 模型构造），**求解层不计入**

按 §4.2 反误报硬约束：

> 防漏报机制**绝不能反向引入误报**

### 0.2 精神性问题（贯穿全文）

每条漏报候选要回答**两个分立问题**：

- **Q1：是否真的是"silent partial"？** —— 工具自陈"我没全干完"的字面是否实际指代翻译 / 模型构造的不完整？还是工具的**已记录设计选择**（语义近似 / 多义性消解 / over-approximation）？
- **Q2：oracle 是否本应抓？** —— 即使工具确实自陈不完整，按宪法 §六-3 前端测量 + §4.2 反误报双向约束，oracle 抓住此 marker 是否会引入误报（伤合法 SUCCESS）？

只有 **Q1=是 ∧ Q2=是** 才构成真漏报落地；其他三种组合（设计选择 / 抓不到、设计选择 / 抓到、partial / 不该抓）均不算"成立"。

### 0.3 P13 audit-2 已有判例（关键先例）

[`oracle-leak-audit-2-2026-05-11.md`](oracle-leak-audit-2-2026-05-11.md) §3.1（kani）确立先例：

- kani 的 `Found the following unsupported constructs` warning 含 7 类构造
- **抓 5 类**（InlineAsm / simd_cast / catch_unwind / ptr_mask / C string literal）—— 都是用户代码层的 hard stub
- **特意排除 2 类**（caller_location / foreign function）—— 60-63/144 SUCCESS 高频触发，几乎所有非 trivial entry 都因 std panic / alloc 内部触发；抓它们会大规模假阳性

**先例核心**：**warning 字面 ≠ 必抓**，"工具自陈"必须经"是否高频 + 是否用户代码 vs std 内部"二次筛。

---

## §1 22 候选逐条 counter-challenge

按 R5-c 候选清单顺序逐条 §4.8.4 处理。

### #1-#2 rocq-of-rust × 2 — `unsafe-ptr/raw-ptr-const/raw_ptr_const_match`

**R5-c 判定**：漏报。stderr 7 attempts 都含 `warning: This kind of constant in patterns is not yet supported.` + 源码 [`thir_pattern.rs:165-176`] `emit_warning_with_note + Rc::new(Pattern::Wild)` silent 替换为 wildcard。

**Counter Q1**：
- 精神反查：[`tools/rocq-of-rust/README.md`](../../tools/rocq-of-rust/README.md) §漏报盲点已明示 "上游引入新 silent fallback 路径不带已知 markers（**实测在 examples corpus 0 现象**）"
- 实测：unsafe-ptr/raw-ptr-const/raw_ptr_const_match 在 v5 corpus 上**7/7 attempts 都触发** —— "0 现象" 声明被推翻
- 源码层论证：thir_pattern.rs:165-176 确实是 silent path —— `Rc::new(Pattern::Wild)` 替换不写 Comment 到产物（不像 thir_expression.rs 的 fallback 写 `Expr::Comment(...)` 能被 gate 5 marker `Please report!` 抓）。产物字面是 `_ => 1`，typecheck 通过，gate 5/6 都不抓 —— **silent partial 真实存在**

**Counter Q2**（oracle 是否本应抓 + 反误报）：
- 现 gate 5 markers：`(Error |Unexpected |Please report!|thir failed to compile|Unimplemented )` —— 都是 ror 内部生成的字面，正常用户字面不会触发
- 拟加 marker：`"is not yet supported"` 字面 —— **风险**：双向验证未做。此字面是 rustc-style warning（通过 `tcx.sess.dcx().struct_span_warn` emit），出现在 stderr 而非产物 .v 内。如果作为 wrapper grep stderr 加，需验证：(a) 是否会因 rust prelude / dep 触发同 message（实测 corpus 中只此 1 entry 触发 —— **暂时无误报，但样本面小**）；(b) 是否其他 silent path（PatKind::Range / Never / Error / DerefPattern）都会触发同 prefix —— 是的，所以一并 catch（但 v5 corpus 0 现象）

**判定**：**成立**（真 silent partial 已被实测推翻 README 的"0 现象"声明）

**精神锚点**：宪法 §六-2 "工具自陈'我没全干完'必须被尊重"。`is not yet supported` 字面是 ror 自陈不完整翻译；产物里 `_ => 1` 替换了原 `DISGUISED_INT => 1`，语义已变。

**决策点级别**：高优先级（README 声明被实测推翻，需更新 README + 加 grep marker）

---

### #3-#10 kani × 8 — concurrency stub 类

R5-c 列 8 entries：`arc/clone-drop`, `charon-limit/arc-slice-unsize`, `collections/hashmap`, `concurrency/atomic`, `creusot-limit/thread-local-ref`, `industrial/rsa-pkcs8`, `lifetime/thread-local`, `miri-limit/weak-memory-incomplete`。

stdout 含 `warning: Kani currently does not support concurrency. The following constructs will be treated as sequential operations` + atomic_xsub/xadd/fence/store/load/singlethreadfence/thread_local 等。

**Counter Q1**（是否真 silent partial）：

源码层验证（[kani-compiler `compiler_interface.rs:758` + `intrinsic.rs`](https://github.com/model-checking/kani)）：

1. **kani 有 TWO 独立 buckets**：
   - `unsupported_constructs` ——发 `Found the following unsupported constructs: <list>. Verification will fail if one or more of these constructs is reachable` warning + codegen 为 unimplemented stub（5 markers 抓的就是此类）
   - `concurrent_constructs` —— 发 `Kani currently does not support concurrency. The following constructs will be treated as sequential operations` warning + codegen 为 **真正的 atomic_block / SKIP / binop**（不是 stub）

2. **atomic_xadd / xsub / load / store / fence 的实际 codegen**（intrinsic.rs:302-320）：
   - `AtomicFence` / `AtomicSingleThreadFence` → `codegen_atomic_noop` 生成 SKIP stmt + `Stmt::atomic_block` 包裹
   - `AtomicXadd` / `AtomicXsub` / 其他 binop → `codegen_atomic_binop!(plus/sub/...)` 生成普通 binop，包在 `Stmt::atomic_block` 里
   - `AtomicLoad` / `AtomicStore` → 分别 codegen 为 load / assign，包在 `Stmt::atomic_block` 里

   即 **kani 真的 codegen 了原子操作，且模型了单线程下的原子性**（atomic_block）。"will be treated as sequential operations" 的精确含义是：**单线程 BMC 不模拟多线程交错** —— 这是 kani BMC 设计的核心约束，不是"我没翻译完这条指令"。

3. **与 5 markers 性质对比**：
   - 5 markers（InlineAsm / simd_cast / catch_unwind / ptr_mask / C string literal）—— kani-compiler emit `codegen_unimplemented_expr` 替换为 stub，**MIR→GotoC 实际不完整**
   - concurrency 类 —— kani-compiler 真 codegen 完整 GotoC，仅在多线程交错语义层不模拟

   **本质不同**：5 markers 是"前端翻译没干完"，concurrency 是"前端干完了，但 BMC 不在多线程层求解" —— 后者是工具自身后端语义边界，按宪法 §六-3 **前端测量**原则属于求解层 vs 前端层切分中的求解层一侧。

4. **与 P13 audit-2 caller_location / foreign function 排除逻辑对比**：
   - P13 排除 caller_location（60/144 SUCCESS 触发）/ foreign function（63/144 SUCCESS）的核心论证：
     - **高频**：std panic / alloc 路径在几乎所有非 trivial entry 上触发
     - **std 内部行为**：用户 entry 不显式调用 caller_location，但 `Vec::push` / `String::from` / panic macros 全都内含
     - **codegen 完成**：kani 仍翻译完整 MIR，warning 仅指示"此 stub 在 SAT 阶段才显现"
   - concurrency 类 8 个 entries 的具体构造分布：
     - `arc/clone-drop` —— `Arc::clone` 内部用 `fetch_add` / `Drop` 用 `fetch_sub` + `atomic_fence`（std 内部）
     - `industrial/rsa-pkcs8` —— RSA 算法本身**不**用 atomic，但 `lazy_static!` / `once_cell` / `sync::OnceLock` 用 `atomic_singlethreadfence`（std/dep 内部）
     - `collections/hashmap` —— std HashMap 用 `thread_local` 作 RandomState seed 来源（std 内部）
     - `creusot-limit/thread-local-ref` / `lifetime/thread-local` —— 用户级 `thread_local!`（**用户代码**）
     - `concurrency/atomic/atomic_seqcst` —— 用户直接 `AtomicUsize::fetch_add(_, Ordering::SeqCst)`（**用户代码**）
     - `miri-limit/weak-memory-incomplete` —— 用户级 `AtomicU32::load(Relaxed)`（**用户代码**）
     - `charon-limit/arc-slice-unsize` —— Arc 同 `arc/clone-drop`（std 内部）

   即 8 entries 中 **5 个是 std 内部触发**（用户不显式用 atomic），**3 个是用户代码直接用 atomic / thread_local**

5. **频率验证**：8/151 SUCCESS 触发 ≈ 5.3%（vs caller_location 60/144 ≈ 42%，foreign function 63/144 ≈ 44%）

**Counter Q2**（oracle 是否本应抓 + 反误报）：

拟加 marker：`"does not support concurrency"` 或 atomic_xsub/xadd/fence 等具体字面。

- 误报风险（实测）：
  - `arc/clone-drop` 是 **合法 Arc 使用**（用户期望 kani SUCCESS：表达"kani 对 Arc clone 的 codegen 没问题"）—— 加 marker 会让该 entry FAILED，但 entry 设计意图是 SUCCESS（不在 `kani-limit/`）
  - `industrial/rsa-pkcs8` 是 **行业级 RSA 算法**，不涉及多线程；entry 设计意图是测 kani 对工业代码 codegen 能力 —— 加 marker 会让该 entry FAILED
  - `collections/hashmap` 是 **基础 HashMap 使用**；加 marker 会让该 entry FAILED
  - 同样的 reasoning：5/8 entries 是合法用户代码上 std 内部触发 —— 抓 marker 会大规模假阳性，**违反 §4.2 反误报硬约束**

**Q2 判定**：拟加 marker 不能通过双向实测；按宪法 §4.2 "宁可保留漏报盲点（按 §4.4 诚实声明），不可引入误报"。

**判定**（按子类细分）：

| Entry | 触发类 | Q1（真 partial？）| Q2（应抓？）| 终判 |
|---|---|---|---|---|
| arc/clone-drop | std-内部 atomic | 否（kani BMC 设计选择 + std 内部） | 否（伤合法 entry） | **不成立 / 设计选择** |
| charon-limit/arc-slice-unsize | std-内部 atomic | 否 | 否 | **不成立 / 设计选择** |
| collections/hashmap | std-内部 thread_local | 否 | 否 | **不成立 / 设计选择** |
| industrial/rsa-pkcs8 | dep-内部 fence | 否 | 否 | **不成立 / 设计选择** |
| creusot-limit/thread-local-ref | 用户级 thread_local | 部分（kani 自陈替换） | 否（误报风险，且 kani 已 sequential codegen） | **不成立 / 设计选择** |
| lifetime/thread-local | 用户级 thread_local | 部分 | 否 | **不成立 / 设计选择** |
| concurrency/atomic/atomic_seqcst | 用户级 atomic | 部分（用户直接测 SeqCst，但 kani 已声明 BMC 不模拟） | 否（设计契约已声明） | **设计选择 / 不算漏报** |
| miri-limit/weak-memory-incomplete | 用户级 Relaxed | 部分 | 否 | **设计选择 / 不算漏报** |

**核心论证**：

- README §SUCCESS 已声明 kani `--only-codegen` 限于 **MIR → GotoC codegen** 边界，**不到 SAT 求解**。concurrency 类 8 entries 中 kani 全部完成 codegen（atomic_block / SKIP / binop 全部 emit），warning 是"求解阶段不会枚举多线程交错"的预先告知，**未跨越前端 codegen 边界**
- 按宪法 §六-3 前端测量原则：求解层不计入测量；concurrency stub 属求解层的 BMC 单线程约束，不属前端 codegen
- 与 P13 audit-2 排除 caller_location / foreign function 的**同构论证**：高频 + std 内部 + warning 是求解阶段提示，不是 codegen 不完整
- **本应该写到 README §漏报盲点**：当前 README 漏报盲点只列 caller_location / foreign function；concurrency 也属同类盲点。**这是文档完整性问题，不是 0 漏报指标突破**

**判定**：8 entries 全部 **不成立 / 设计选择**（建议 README 文档补完，不抓 marker）

**精神锚点**：宪法 §六-3 前端测量边界 + P13 audit-2 §3.1 的 caller_location/foreign function 排除先例 + §4.2 反误报硬约束

**决策点级别**：低（README 文档补完即可）

---

### #11-#14 soteria × 4 — intrinsic stub 类

R5-c 列 4 entries：`concurrency/atomic/atomic_seqcst`, `miri-limit/weak-memory-incomplete`, `float/transcendental/float_transcendental`, `kani-limit/float-overapprox/trigger_check_sin_cos_identity`。

stdout 含 `warning: An atomic intrinsic was encountered; it will be executed as sequential code` 或 `warning: A complex floating point intrinsic was encountered; it will be executed with a significant over-approximation`。

**Counter Q1**（按 intrinsic 类细分）：

#### 11-12 atomic intrinsic（concurrency/atomic + miri-limit/weak-memory-incomplete）

- 与 kani concurrency 类完全同构：soteria 是 **symbolic execution**，单线程符号执行不模拟多线程交错
- "executed as sequential code" 不是"我没模型这个 atomic" —— 是"我在单线程符号语义下把它当 sequential" —— 操作仍执行
- soteria README §44（[`tools/soteria/README.md`](../../tools/soteria/README.md)）声明 "soteria exit 0 ⇔ 符号执行完成且无 bug" —— **符号执行确实跑完了**，只是不枚举交错
- 按 §六-3 前端测量：soteria 的多线程交错枚举属求解层的设计选择，不属前端 codegen / 模型构造
- **同 kani 同样的 README 盲点**：soteria README 当前声明 "漏报盲点：无" —— 但 atomic intrinsic 的 sequential 替换 + 复杂 float intrinsic 的 over-approximation 都是 silent partial 的工具自陈，至少应**入盲点声明**

#### 13-14 复杂 float intrinsic（float/transcendental + kani-limit/float-overapprox）

- "executed with a significant over-approximation" 是 **soundness-preserving abstraction** —— soteria 对 sin/cos/exp 等 transcendental 用 over-approximation 表示（即返回非确定性值约束在合理区间内）
- over-approximation 在符号执行中通常表示"任何后续 path 都仍能正确发现 bug，只是 spurious counter-example 可能增多"
- 这与 silent skip 性质不同 —— over-approximation 是 **soundness 保留的精度损失**，不是"我跳过这条指令"
- 但是按宪法 §六-2 严格解读：soteria 自陈"我用了 significant over-approximation" —— "工具自陈"字面落地

**Counter Q2**（误报反向验证）：

拟加 marker：`"intrinsic was encountered.*sequential"` 或 `"intrinsic.*over.approximation"`。

- 反误报：4 entries 全部是 **本来就涉及 intrinsic** 的 entry（concurrency/atomic 显式 AtomicUsize、float/transcendental 显式 sqrt/sin、miri-limit/weak-memory 显式 Relaxed atomic、kani-limit/float-overapprox 显式 sin/cos）
- 即抓 marker 不会伤"不涉及 intrinsic 的合法 entry"（不像 kani 的 Arc/HashMap 高频 std 内部触发）
- **但反向**：用户写 sqrt / sin / atomic 时，soteria 的 over-approximation 本就是 soteria 处理这类语义的标准方式 —— 抓 marker = 把 soteria 对 transcendental 的全部支持归零（任何用 sqrt 的 entry 都 FAILED）。这违反"工具能力测量"精神：sqrt 在数学样例中极常见

**判定**：

| Entry | Q1（真 partial？）| Q2（应抓？）| 终判 |
|---|---|---|---|
| concurrency/atomic/atomic_seqcst | 部分（同 kani concurrency） | 否（设计选择） | **设计选择 / 不算漏报** |
| miri-limit/weak-memory-incomplete | 部分 | 否 | **设计选择 / 不算漏报** |
| float/transcendental | 部分（over-approximation） | 否（误报 sqrt 类） | **设计选择 / 不算漏报** |
| kani-limit/float-overapprox | 部分 | 否 | **设计选择 / 不算漏报** |

**精神锚点**：宪法 §六-3 前端测量 + §4.2 反误报双向约束 + soundness-preserving over-approximation 不构成 partial

**决策点级别**：低（建议 soteria README 补盲点声明 "atomic intrinsic sequential 替换 / complex float intrinsic over-approximation 属求解层语义近似"）

---

### #15 verus × 1 — derived Clone spec skip

R5-c：`aeneas-limit/float-types/make_measurement` SUCCESS。stderr `warning: autoderive Clone impl does not take the form Verus expects; continuing, but without adding a specification for the derived Clone impl`。

**Counter Q1**（是否真 partial）：

- warning 字面：`continuing, but without adding a specification for the derived Clone impl`
- "continuing" 字面：verus VIR 构造继续完成（即翻译完成）
- "without adding a specification" 字面：仅 **derived Clone impl 的 auto-spec 没生成**

- verus 的工作流：
  - **前端**：Rust → VIR 翻译（lifetime check 借 rustc，再 lower VIR）
  - **spec 自动生成**：对 `#[derive(Clone)]` 等内置 derive，verus 试图生成 default spec（如 `Clone::clone` returns a deep copy）
  - **后端**：SMT 求解（本框架用 `--no-verify` 不到此层）

- 按宪法 §六-3 前端测量：仅前端 / 模型构造算入；spec 生成是 verus 走向 SMT 求解的中间步骤
- 该 warning 字面 "continuing" 明确表示前端 / VIR 构造**完成**；缺的是 derived impl 的 auto-spec
- 用户没显式调用 `<X as Clone>::clone` 的 verus contract —— 即使有 spec verus 也不在 `--no-verify` 下用
- entry `make_measurement` 不验证 Clone 语义，只是 `#[derive(Clone)]` 这个标注

**Counter Q2**（反误报）：

拟加 marker：`"continuing.*without adding.*specification"`。

- 反误报：v5 corpus 2 entries 触发（make_measurement SUCCESS + conditional_method FAILED），其他 159 SUCCESS 都不触发 —— 似乎可以 marker
- 但语义：`#[derive(Clone)]` 是 Rust 极常见构造 —— 任何含浮点 / 复杂泛型的 struct 用 derive(Clone) 都可能触发此 warning（verus 对部分 derive form 有 spec 推断不全的盲点）
- 若 corpus 扩大，可能大量 entries 触发，导致 verus 通过率断崖式下降 —— 实际并不反映 verus 前端能力

**判定**：

- Q1：**不算 partial**（VIR 构造完成，仅 spec 生成边界缺失，属 verus → SMT 的中间步骤）
- Q2：抓 marker 风险大（derive(Clone) 极常见）

终判：**设计选择 / 不算漏报**

**精神锚点**：宪法 §六-3 前端测量（spec gen 属求解前置，不属翻译）

**决策点级别**：低（建议 verus README 补盲点声明 "derive(Clone) 等 derive 的 auto-spec 生成在某些 form 上 silent skip"）

---

### #16-#22 aeneas × 4 backends × 2 entries — charon stage silent partial

R5-c 列 7 entries（aeneas-coq / aeneas-fstar / aeneas-lean 各 2 个 entry + aeneas-hol4 1 个 entry，hol4 在 copy-deref-closure 上 FAILED）：

- `charon-limit/inline-asm/nop_via_asm` × 4 backends
- `charon-limit/copy-deref-closure/deref_copy_in_closure` × 3 backends

#### #16-#19 inline-asm × 4 backends

stderr：`warning: Inline assembly is not supported` + `warning: The extraction generated 1 warnings`。
stdout：`Translated opaque functions: 1/1` + `Translated transparent functions: 0/0`。

**Counter Q1**（是否真 silent partial）：

源码层 + 实测：

1. **charon 行为**：`charon cargo --preset=aeneas` 调用，**Preset::Aeneas 不设 `--abort-on-error`**（见 [`charon/src/options.rs:363-380`](https://github.com/AeneasVerif/charon)）
2. **inline asm 在 charon 中的降级**：charon 不能翻译 inline asm，把 entry_fn 降级为 **opaque function**（即只声明类型，不翻译 body）。charon exit 0
3. **aeneas 接收**：aeneas 接收 .llbc，翻译 1/1 opaque + 0/0 transparent，exit 0
4. **产物**：lean-out/CharonLimitInlineAsm.lean 中 entry_fn `nop_via_asm` 以 **`opaque def`** 形式存在（无 body）
5. **`Translated opaque functions: 1/1` 是 aeneas info-level**：aeneas 不知道 charon 是否自愿 mark opaque，对 aeneas 看来 "charon 给我 1 个 opaque、我翻完了 1 个"

按宪法 §六-2 "工具自陈'我没全干完'必须被尊重"：

- charon stage 自陈：`warning: Inline assembly is not supported`（loud）
- 整个 pipeline 自陈：`warning: The extraction generated 1 warnings`（aeneas pretty-print 时合并 charon 的 warning）
- entry_fn `nop_via_asm` 是该 entry 唯一 fn —— body 没翻 = entry 没翻

**真 silent partial**：pipeline 视角，entry_fn 落产物时**只是 opaque 声明，无 body**。`aeneas exit 0` ≠ "entry 翻译完成"。

**aeneas-lean / aeneas-fstar / aeneas-coq README §"形式严格性 — 0 漏报"** 声明只覆盖 aeneas 自身 craise 单一通路（[`Main.ml:773`]）；**未覆盖 charon 上游 silent partial**。即 README 自陈 0 漏报论证 **范围不全**（pipeline-level 论证缺失）。

**Counter Q2**（反误报）：

修复路径建议：

**方案 A**：在 aeneas wrapper 第一阶段 charon 调用 grep stderr：
- `warning: The extraction generated .* warnings` —— aeneas pretty-print 后字面统一
- `Inline assembly is not supported`
- `Type error after transformations`
- 命中 → FAILED

双向验证（v5 raw 实测）：
- 已知 silent path：inline-asm SUCCESS entry 都触发 `extraction generated X warnings` —— ✅ 命中
- 合法 SUCCESS entries（其他 100 aeneas-lean SUCCESS）：grep `extraction generated` —— **需重 audit raw stderr 是否有合法 entry 也触发此字面**（产物 audit 应单独跑）

**方案 B**：让 wrapper 在 charon 调用中加 `--abort-on-error`（覆盖 Preset::Aeneas 默认）。
- 风险：可能伤"charon 半翻 + aeneas 仍翻部分"的真 charon-edge entries（如果有的话）

实测样本对 inline-asm 有效，对 copy-deref-closure 有效（type error case）。

**判定**：

| Entry | Backend | Q1 | Q2 | 终判 |
|---|---|---|---|---|
| inline-asm/nop_via_asm | aeneas-coq | 是（charon 把唯一 fn 降级 opaque） | 是（marker 双向验证可行） | **成立** |
| inline-asm/nop_via_asm | aeneas-fstar | 是 | 是 | **成立** |
| inline-asm/nop_via_asm | aeneas-lean | 是 | 是 | **成立** |
| inline-asm/nop_via_asm | aeneas-hol4 | 是 | 是 | **成立** |

**精神锚点**：宪法 §六-2 + tool-integration §4.4（aeneas README 自陈 0 漏报论证未覆盖 pipeline-level，按 §4.4 应明示 charon stage 的 silent partial 是已知盲点）+ §4.2 反误报需 cc 阶段产物层双向 audit

**决策点级别**：高（aeneas × 4 都受影响，pipeline-level 0 漏报论证有真实缺口）

#### #20-#22 copy-deref-closure × 3 backends（hol4 FAILED）

stderr：`error: Type error after transformations: Found incorrect clause var: Bound(1, 0)` × 3 次 + `warning: The extraction generated 3 warnings`。

**Counter Q1**：
- charon stage 内部 **`error:` 字面** × 3，但 charon stage **exit 0**（因 Preset::Aeneas 没 abort-on-error）
- charon 把含 type error 的 fn 仍写入 .llbc（partial）
- aeneas 接收 partial .llbc，翻译 `Translated transparent functions: 6/6` —— 但 6 个 fn 中 closure 相关 ones 可能 body 有误或 silently degraded
- **重点**：stderr 字面是 `error:` 而非 `warning:` —— charon **明确分类为 error**，但因 Preset::Aeneas 行为，error 不升 exit

这是比 inline-asm 更严重的 pipeline-level 漏报 —— **charon 自己声明 error 但仍 exit 0**。

**Counter Q2**：
- 修复路径与 #16-#19 同（marker `error: Type error after transformations` 或 `extraction generated.*warnings`）
- 双向验证：v5 raw 只有此 1 entry × 3 backends 触发 —— 误报风险低

**判定**：

| Entry | Backend | Q1 | Q2 | 终判 |
|---|---|---|---|---|
| copy-deref-closure | aeneas-coq | 是（3× type error，未升 exit） | 是 | **成立** |
| copy-deref-closure | aeneas-fstar | 是 | 是 | **成立** |
| copy-deref-closure | aeneas-lean | 是 | 是 | **成立** |

**精神锚点**：同 #16-#19，更严格 —— charon 自陈 `error:` 都不升 exit，按宪法 §六-2 必须升 FAILED

**决策点级别**：高

---

## §2 22 候选 cc 验证后的真漏报清单

### 2.1 cc 验证总览

| # | R5-c 候选 | cc 判定 | 真漏报数 |
|---:|---|---|---:|
| 1-2 | ror × 2 (raw_ptr_const_match) | **成立** | 2 |
| 3-10 | kani × 8 (concurrency stub) | **不成立 / 设计选择** | 0 |
| 11-14 | soteria × 4 (intrinsic stub) | **不成立 / 设计选择** | 0 |
| 15 | verus × 1 (derived Clone) | **不成立 / 设计选择** | 0 |
| 16-19 | aeneas × 4 (inline-asm) | **成立** | 4 |
| 20-22 | aeneas × 3 (copy-deref-closure) | **成立** | 3 |

**经 cc 验证真漏报总数**：**9 entries**（vs R5-c 的 22 候选；13 个被 counter 驳回 / 设计选择）

### 2.2 真漏报清单（按工具汇总）

| 工具 | entry | 备注 |
|---|---|---|
| rocq-of-rust | `unsafe-ptr/raw-ptr-const/raw_ptr_const_match` | thir_pattern.rs Pattern::Wild silent 替换 |
| rocq-of-rust-typecheck | 同上 | tier-1 继承 tier-0 缺口 |
| aeneas-coq | `charon-limit/inline-asm/nop_via_asm` | charon stage opaque 降级 |
| aeneas-fstar | 同 | 同 |
| aeneas-lean | 同 | 同 |
| aeneas-hol4 | 同 | 同 |
| aeneas-coq | `charon-limit/copy-deref-closure/deref_copy_in_closure` | charon stage type error 未升 exit |
| aeneas-fstar | 同 | 同 |
| aeneas-lean | 同 | 同 |

### 2.3 13 个驳回候选的精神共识

**kani 8 个 + soteria 4 个 + verus 1 个共 13 个候选**全部驳回，核心论据：

1. **前端 vs 求解层切分**（宪法 §六-3）：concurrency / atomic intrinsic / float over-approximation 的"sequential / over-approximation"是**求解层语义约束**，工具前端 codegen 完成 —— 不属"前端 partial"
2. **同 P13 audit-2 caller_location / foreign function 排除先例**：当 warning 出现高频 / std 内部触发 / soundness-preserving abstraction 时，按 §4.2 反误报硬约束不抓
3. **README 文档盲点而非 0 漏报指标失守**：13 个候选共同点是工具 README "漏报盲点：无"声明欠诚实，按 §4.4 应补盲点条目；但这是 README 文档完整性问题，不是 oracle 抓不到的 silent partial 落地

---

## §3 漏报修复建议（不实施，留决策点）

按宪法 §4.2 反误报硬约束 + charter-craft §4.8.3 决策点 vs 非决策点判据：所有真漏报**只留决策点**，**不直接修**（修可能引入新误报）。

### 3.1 ror × 2 漏报修复路径

- **修复路径**：gate 5 markers 扩展加 `"is not yet supported"` 字面 wrapper grep stderr
- **修复风险**：
  - 双向验证（已知 silent path 命中）：unsafe-ptr/raw-ptr-const 实测 7/7 attempts 触发 ✅
  - 反误报（合法 SUCCESS 不命中）：v5 corpus 中 124 SUCCESS 仅此 1 entry 触发，**短期无误报**；但理论上 thir_pattern.rs / thir_expression.rs / thir_type.rs 还有多处 `is not yet supported` 字面 emit point，未来 corpus 扩展可能触发其他 entries
  - 关键反向：是否 ror 上游有合法 `is not yet supported` 字面在 stderr（例 prelude pretty-print）—— **本审范围内未独立验证**
- **优先级**：**高**（README 声明被推翻，需更新 README + 加 marker；marker 实施前必须做 corpus 全集 stderr grep 双向 audit）

### 3.2 aeneas × 4 inline-asm 漏报修复路径

- **修复路径 A**：aeneas wrapper 在 charon 调用后 grep stderr：
  ```
  warning: The extraction generated [0-9]+ warnings
  ```
  或：
  ```
  Inline assembly is not supported
  ```
  命中 → FAILED
- **修复路径 B**：charon 调用追加 `--abort-on-error`（覆盖 Preset::Aeneas 默认）
- **修复风险**：
  - 方案 A 反误报：v5 raw 中 7 entries 触发 "extraction generated.*warnings"（4 inline-asm + 3 copy-deref-closure）—— 其他 SUCCESS 不触发；**但 v5 corpus 161 entries 不全覆盖；需在更大 corpus 上验证**
  - 方案 B 反误报：可能伤"charon 半翻 + aeneas 仍翻部分"的真 charon-edge entries（v5 中未观察到，但 charon 上游若引入新降级路径可能命中）
- **优先级**：**高**（aeneas × 4 都受影响，4 entries × 1-4 backends = 4-16 case 数量级；pipeline-level 0 漏报论证已自陈缺口）

### 3.3 aeneas × 3 copy-deref-closure 漏报修复路径

- **修复路径**：同 3.2
- **修复风险**：同 3.2，且 `error: Type error after transformations` 字面是 charon 极强信号 —— grep 此字面误报风险更小
- **优先级**：**高**（charon 自陈 `error:` 仍 exit 0 是 pipeline 严重缺陷）

### 3.4 13 个驳回候选的"文档盲点补完"建议（非修复，只是 README 补丁）

按宪法 §4.4 "漏报盲点的诚实声明"：

- **kani README §漏报盲点**：补"`Kani currently does not support concurrency` 类 warning（atomic_*/thread local/fence）是 kani BMC 设计上的单线程语义约束，前端 codegen 完成、SAT 阶段不模拟多线程交错 —— 按 §六-3 前端测量不属前端 partial，但读者引用 SUCCESS 时应知此盲点"
- **soteria README §漏报盲点**：补"`atomic intrinsic ... executed as sequential code` 与 `complex floating point intrinsic ... over-approximation` 是 soteria 符号执行的求解层语义近似，前端 / 符号执行完成 —— 不属前端 partial，但读者应知"
- **verus README §漏报盲点**：补"`continuing, but without adding a specification for the derived Clone impl` 是 verus 对部分 derive form 的 auto-spec 生成边界 —— VIR 构造完成、spec gen 部分缺失，不影响 `--no-verify` 下的前端测量"

**优先级**：中（README 完整性 + 长期诚实，不影响当下 v5 数据）

---

## §4 进决策点累积文档的 case

预备给 R5 后续步骤把以下追加到 [`decisions-2026-05-11.md`](decisions-2026-05-11.md) 作为 **D3 漏报清单**：

- **D3.1**：ror × 2 raw_ptr_const_match 漏报 + thir_pattern.rs silent path 缺口
- **D3.2**：aeneas × 4 backends inline-asm 漏报（charon `--preset=aeneas` 不 abort-on-error 设计）
- **D3.3**：aeneas × 3 backends copy-deref-closure 漏报（charon type error 不升 exit）
- **D3.4**：kani README §漏报盲点补完（concurrency 类）
- **D3.5**：soteria README §漏报盲点补完（intrinsic 类）
- **D3.6**：verus README §漏报盲点补完（derived auto-spec 类）

D3.1-3.3 是真漏报落地决策点（修不修 / 怎么修）；D3.4-3.6 是 README 文档补完决策点（不影响 oracle，只为长期诚实）。

---

## §5 cc 阶段方法学心得（写给 R5 后续）

1. **精神反查链 ≠ 字面 grep**：R5-c 用 "工具自陈 partial" 字面识别候选高 recall，但落地必须经"是否前端 partial（§六-3）+ 反误报验证（§4.2）"双门
2. **P13 audit-2 caller_location 排除先例 = 通用判例**：任何工具的 warning 类候选都应套 "是否高频 / 是否 std 内部 / 是否 soundness-preserving" 三筛 —— 不通过则不抓
3. **Pipeline-level 漏报需 README pipeline-level 论证**：aeneas × 4 case 暴露 README "0 漏报形式可证"论证只覆盖 aeneas 本体、未覆盖 charon stage 的 pipeline 缺口 —— 任何 multi-stage 工具 README 都应明示 pipeline-level 论证范围
4. **disprove-first 杀低质挑刺**：22 候选 → 9 真漏报 = 59% 驳回率（13/22）—— 充分体现 disprove-first 的过滤价值，符合宪法 §八审查协议的实用主义经验

---

> 报告创建时间：2026-05-11
>
> 审查者：cc 阶段独立 agent（与 R5-c 派遣 agent 不同 session）
>
> 数据：`runs/run-1778500291-90812/raw/`
