# kani — 特性支持评估报告（v6 final post-P37 baseline）

## 元数据

- **数据源**：`runs/run-1778560393-59119/`（2026-05-12 v6 final post-P37）
- **工具配置**：`tools/kani/`（含 `kani-strict-wrapper.sh` 5-marker codegen-stub 封堵 + P37 §六 当前 crate 焦点反向证明）
- **工具版本**：`cargo-kani 0.67.0`
- **本工具实测**：n=161 / SUCCESS=159 / FAILED=2 / UNKNOWN=0，通过率 **98.8%**（159 / 161）
- **时长分布**：avg 3416 ms / median 1009 ms / p90 9474 ms / max 32041 ms
- **宪法 baseline**：`principles.md` v8（含 §六 UNKNOWN 严格语义 + 当前 crate 焦点 + 前端测量切割 + Oracle 责任）
- **时效声明**：本快照锚定上述 run id + `cargo-kani 0.67.0` + 当前 corpus（161 entries），不构成长期承诺。

## pipeline + 前端边界

kani 是 AWS 出品的基于 CBMC 的 Rust bounded model checker。完整 pipeline：

```
rustc 前端 → kani-compiler (rustc plugin)
           → MIR → GotoC IR codegen
           → 写出 goto-binary (*.symtab.out) + 元数据
           → [本测试切割点 = --only-codegen 终止于此]
           → cargo kani 喂 goto-binary 给 CBMC
           → CBMC SAT/SMT 求解 reachability / assertion
```

**前端**（本测试度量范围）：kani-compiler 的 MIR → GotoC codegen + 类型 / 模型构造。
**后端**（本测试不度量）：CBMC SAT 求解。

切割理由（公平性）：其他前端测量工具（cargo-check / charon / hax / creusot / aeneas 等）均在前端层停下；让 kani 跑完整 CBMC 求解会在集合 / 并发 / 递归枚举等场景下超时频繁 FAILED，制造假阴性。

**项目自维护组件**：`tools/kani/kani-strict-wrapper.sh` —— runner 调 wrapper 而非直接调 `cargo kani`，wrapper 跑 `cargo kani --only-codegen --bin __ts_harness` 后 grep stdout 5-marker subset；命中即 P37 §六 反向证明（os.walk entry crate `src/` + 关键字正则）决定是 entry 自用（→ FAILED）还是 deps-only（→ SUCCESS 豁免）。

## SUCCESS 信号 + 形式严格性

按宪法 §六 双通路 partial 暴露：

**判定式**：

```
SUCCESS ⟺ kani-strict-wrapper.sh exit 0
       ⟺ cargo kani --only-codegen --bin __ts_harness exit 0
         ∧ ( stdout 不命中 5 markers 任一
             ∨ ( 命中但 entry crate src/ 无对应触发关键字 → §六 deps-only 豁免 ) )

         5 markers：
           - TerminatorKind::InlineAsm   (inline asm MIR terminator)
           - simd_cast                   (packed-SIMD cast intrinsic)
           - catch_unwind                (panic recovery)
           - ptr_mask                    (raw-pointer bit-mask intrinsic)
           - C string literal            (c"..." 字面量 MIR rvalue, Rust 2024)
```

- **主信号通路**：`cargo kani --only-codegen` exit code（kani-compiler 自身报错 → exit ≠ 0 直通 FAILED）
- **wrapper 补抓通路**：5-marker grep gate + P37 §六 反向证明（命中 marker 后，对 entry crate `src/` 走 os.walk 收集 `.rs` 文本 + 关键字正则匹配；仅当 entry 自写触发关键字时升 FAILED）

**为什么是 5-marker 子集而非完整 `Found the following unsupported constructs:` warning list**：实测 60-63/144 SUCCESS 命中 `caller_location` / `foreign function`，这两条来自 std panic / std alloc 路径几乎每个 non-trivial entry 都触发，纳入会让 ~44% SUCCESS 翻车（含 hello/basic-hello、bigint-arith、industrial/rsa、industrial/sha256-digest）—— 是结构性误报。5-marker 是反误报实测校准出的精筛集合（详 `docs/fixes/oracle-leak-audit-2-2026-05-11.md` §3.1）。

**为什么需要 P37 §六 反向证明**：kani-compiler 的 5-marker warning 只聚合 counts，不带 source span；`--message-format=json` 在当前 cargo-kani 不可用。无法直接从 kani 输出区分"entry crate 自用"vs"deps / std 内部"。P37 反向证明绕过此局限：在 entry crate 自己的 `src/` 目录扫所有 `.rs` 文件文本，匹配 marker 对应的触发关键字（如 `asm!` / `catch_unwind` / `simd_*` / `ptr::mask` / `c"..."`）。entry crate 不含这些关键字时，markers 必然来自 deps —— 按 §六 当前 crate 焦点（宽度切割）豁免 FAILED。

**形式严格性**（不作过强声明）：

- **0 误报状态**：实测 + wrapper 双通路封堵 + P37 §六 反向证明。任意合法 SUCCESS（不触发 5 markers，或触发但 entry crate src/ 无对应关键字）在 wrapper 下保持 SUCCESS。**注**：未来 kani-compiler 演进可能引入新警告格式或新合法用例命中现有 marker；P37 反向证明的关键字正则保守宽（偏向 false-positive FAILED 而非 false-negative SUCCESS），但非形式证明，仅实测校准。
- **0 漏报状态**：5 markers 是 kani 自陈"Verification will fail if one or more of these constructs is reachable"的字面诉求，与"工具接受"互斥。entry 自用经反向证明锁定后必入 FAILED。但 `caller_location` / `foreign function` 路径未封堵（结构性排除），属可争议剩余口径——见漏报盲点。

## 失败分桶（按 P31 §四.5 归因分类 + §六 当前 crate 焦点切割 + P37 反向证明）

v6 final post-P37 实测 2 个 FAILED，全部命中 5-marker **且 entry crate `src/` 自含触发关键字**（P37 反向证明锁定为 entry 自用）。

### 桶 1：entry crate 自写 MIR 触发 5-marker（2 case，FAILED）

| entry | 命中 markers | 来源（entry 源码层）| 反向证明关键字 |
|---|---|---|---|
| `charon-limit/inline-asm/nop_via_asm` | `TerminatorKind::InlineAsm (1)` | `src/lib.rs` 直接写 `unsafe { core::arch::asm!("nop", ...) }` | `\b(asm\|global_asm)\s*!` 命中 |
| `kani-limit/stack-unwinding/trigger_divide_with_recovery` | `catch_unwind (1)` | `src/lib.rs` 直接调 `std::panic::catch_unwind(|| numerator / denominator)` | `\b(catch_unwind\|catch_panic)\b` 命中 |

stderr 特征（`inline-asm/nop_via_asm`）：

```
[kani-oracle] FAIL: codegen with hard-unsupported markers; entry crate src/
[kani-oracle]       self-uses triggering keyword(s) for: ['TerminatorKind::InlineAsm']
[kani-oracle]       Per project §六-2 反作弊, entry-self partial → FAILED.
```

**归因**：工具不支持。两个 entry 都是边界样例（`charon-limit/` 与 `kani-limit/`），样例本意就是触发该构件；entry crate 源码层确有 `asm!` / `catch_unwind` 调用，kani-compiler 自陈不能 codegen → emit stub。这是 kani-compiler MIR codegen 的真实能力边界。
**处理**：不修。本地性原则下 FAILED 站得住，工具开发者不能驳回（kani 自家 warning list 就明示了这两类）。

### （v6 post-P36 桶 2 已消解，P37 § 六 反向证明豁免）

post-P36 时桶 2（8 case）：5-marker 命中但 entry crate src/ 不含触发关键字 —— markers 来自 deps / std 内部。P37 wrapper 升级直接实施 §六 当前 crate 焦点（宽度切割）的反向证明判定，8 个 entries 全部豁免 → SUCCESS：

| entry | 命中 markers（kani 报）| 反向证明结果 |
|---|---|---|
| `concurrency/thread-mutex/thread_mutex_join` | `C string (1)` + `catch_unwind (3)` + `ptr_mask (1)` | entry src/ 无关键字 → 豁免 |
| `miri-limit/thread-interleaving-partial/unsynchronised_counter_race` | `C string (1)` + `catch_unwind (4)` + `ptr_mask (1)` | entry src/ 无关键字 → 豁免 |
| `deps-complex/bigint-serde/bigint_serde` | `InlineAsm (5)` + `simd_cast (2)` | entry src/ 无关键字 → 豁免 |
| `deps-complex/chrono-serde/chrono_serde` | `InlineAsm (5)` + `simd_cast (2)` | entry src/ 无关键字 → 豁免 |
| `deps-complex/collections-serde/collections_serde` | `InlineAsm (5)` + `simd_cast (2)` | entry src/ 无关键字 → 豁免 |
| `deps-complex/error-chain/error_chain` | `catch_unwind (1)` + `ptr_mask (1)` + `simd_cast (1)` | entry src/ 无关键字 → 豁免 |
| `industrial/x509-parser/cert-parse/x509_parse_der` | `catch_unwind (1)` + `ptr_mask (1)` | entry src/ 无关键字 → 豁免 |
| `industrial/x509-parser/cert-parse/x509_subject_extensions` | `catch_unwind (1)` + `ptr_mask (1)` | entry src/ 无关键字 → 豁免 |

stderr 特征（`concurrency/thread-mutex`）：

```
[kani-oracle] kani 5-markers fired ['catch_unwind', 'ptr_mask', 'C string literal'] but entry crate src/ has no triggering
[kani-oracle]       keyword; markers must originate from deps (std/registry/vendor).
[kani-oracle]       Per principles.md §六 当前 crate 焦点 (宽度切割), external-dep
[kani-oracle]       partial does not count — suppressing FAILED → SUCCESS.
```

**归因**：markers 由 deps（serde derive / num-bigint / chrono / anyhow / thiserror / x509-parser）或 std 内部（`thread::spawn` / `Mutex` / `AtomicUsize`）产生；entry crate 自己的 MIR 完整出现在 kani goto 产物里。按 §六：外部依赖路径下的 opaque / skip / stub 不触发 partial 判定 → SUCCESS。
**处理**：已修（P37 wrapper 升级落地）。

## 漏报盲点（诚实声明）

**已通过 wrapper gate 封堵**：

- 5 个 hard-unsupported MIR construct stub 路径：`TerminatorKind::InlineAsm` / `simd_cast` / `catch_unwind` / `ptr_mask` / `C string literal`
- entry 自用判定：P37 §六 反向证明（entry crate `src/` 关键字正则）锁定 entry crate 自写 → 升 FAILED
- deps-only 命中：P37 §六 反向证明豁免 → 不冤枉 SUCCESS

**仍存在的盲点 / 待裁定项**：

1. **`caller_location` / `foreign function` warning 不抓**：实测 60-63/144 SUCCESS 命中。这两条由 std panic / std alloc 路径几乎每个 non-trivial entry 都触发；kani 视为 std 内部标准 stub 处理。按宪法 §六 严格解读这是可争议剩余口径——纳入会大规模假阳性（含 hello/basic-hello），排除则有遗漏。当前选择"排除以避高频假阳性"——属 cc-report 修订小组暂未裁定项。
2. **concurrency 单线程语义警告不抓**：v6 中部分 SUCCESS entries 含 kani `"Kani currently does not support concurrency. The following constructs will be treated as sequential operations"` warning（atomic_* / thread_local / fence）。kani-compiler **真 codegen 原子操作**（atomic_block / SKIP / binop），不是 stub —— 这是 BMC 单线程语义约束（不模拟多线程交错），属求解层假设而非前端 partial。按宪法 §六 前端测量原则**不抓 marker**，这些 entries 保持 SUCCESS。该 warning 表征求解层简化口径，列出以诚实声明。
3. **P37 反向证明关键字正则的覆盖完备性**：当前 patterns 字典对每个 marker 用启发式关键字正则。已知模式：
   - `TerminatorKind::InlineAsm`：`\b(asm|global_asm)\s*!`
   - `simd_cast`：`\b(simd_*|core::simd|std::simd)\b`
   - `catch_unwind`：`\b(catch_unwind|catch_panic)\b`
   - `ptr_mask`：`(::mask\s*\(|ptr::mask|pointer::mask)`
   - `C string literal`：`(\bc"|\bcr"|CStr::from_bytes_with_nul|cstr!|c_str!)`

   保守策略偏向 false-positive FAILED（宁可漏过宽 → entry 被误判 FAILED，也不要漏过严 → entry 被误判 SUCCESS）。剩余风险：用户用别名 / 宏二次包装 / proc-macro 生成等隐式路径触发 marker 时关键字正则可能漏判。未来若发现新模式，扩展 patterns 字典。
4. **新的 unsupported MIR 节点类别**：kani-compiler 演进可能引入新 stub 路径，需扩展 5 markers list + 对应反向证明关键字；当前列表是 0.67.0 实测对齐。
5. **codegen 完成 + SAT 阶段才会触发的能力问题**：本次未观察到（按"前端测量"原则也不在本测试度量范围）。

## v5.1 → v6 post-P36 → v6 final post-P37 ΔS 解释

- v5.1（`run-1778466265-63960` / 146 entries）：136 SUCCESS / 10 FAILED / 0 UNKNOWN，通过率 93.2%。
- v6 post-P36（`run-1778560393-59119` 旧解读 / 161 entries）：151 SUCCESS / 10 FAILED / 0 UNKNOWN，通过率 93.8%。
- v6 final post-P37（同 run + wrapper 升级覆盖写）：159 SUCCESS / 2 FAILED / 0 UNKNOWN，通过率 **98.8%**。

**v5.1 → v6 post-P36 ΔS = +15**：corpus 从 146 增至 161（+15 entries 全在不触发 5 markers 的 feature 类目，均 SUCCESS）；FAILED 列表完全一致。

**v6 post-P36 → v6 final post-P37 ΔS = +8**：wrapper 升级 P37 §六 反向证明落地，原 8 个 deps-only 命中（concurrency/thread-mutex、miri-limit/thread-interleaving-partial、deps-complex × 4、industrial/x509-parser × 2）从 FAILED → SUCCESS。FAILED 缩到 2 个（charon-limit/inline-asm/nop_via_asm + kani-limit/stack-unwinding/trigger_divide_with_recovery —— 真 entry 自用）。

通过率口径：v6 final = 159/161 ≈ 98.8%；vs v5.1 = 136/146 ≈ 93.2%。差异主要来自 corpus 扩容稀释（+15 SUCCESS）+ P37 反向证明豁免 deps-only（+8 SUCCESS / -8 FAILED）。

## 修订建议清单（仅"我们导致"失败）

**无需修订。所有 FAILED 均为工具能力边界。**

- v6 post-P36 时唯一的中优先级 backlog（"升级 JSON 模式 / 桶 2 §六 当前 crate 焦点 wrapper 升级"）**已落地 P37**。当时方案 A 的理想态（`--message-format=json` 解析诊断 span 锚 `TS_TARGET_CRATE`）受 cargo-kani 不支持 JSON 输出限制；P37 采等效替代方案：直接在 entry crate `src/` 做反向证明（os.walk + 关键字正则）。功能等价：entry crate 自含触发关键字 ⟺ entry 自用 → FAILED；entry crate 无关键字 ⟺ marker 来自 deps → SUCCESS。
- 桶 1（2/2）**不修**——entry crate 自写 `asm!` / `catch_unwind`，kani 自陈"我没把这条干完"，本地性原则下 FAILED / 工具开发者不能驳回。这是 kani-compiler 真实能力边界。

## 历史快照声明

本报告是 2026-05-12 v6 final post-P37 运行 `runs/run-1778560393-59119` 的实测快照；锚定 `cargo-kani 0.67.0` × 5-marker oracle subset × P37 §六 反向证明 × 当前 corpus（161 entries）。kani 升级、kani-compiler 修复 lint 注入、上游 `Found the following unsupported constructs:` warning 格式变化、cc-report 修订小组对 `caller_location` / `foreign function` 路径口径裁定、P37 关键字正则覆盖扩展、vendor crate 演化等任一变化后均需重测。
