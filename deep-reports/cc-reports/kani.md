# kani — 特性支持评估报告（v6 final post-P35 baseline）

## 元数据

- **数据源**：`runs/run-1778560393-59119/`（2026-05-12 v6 final post-P35）
- **工具配置**：`tools/kani/`（含 `kani-strict-wrapper.sh` 5-marker codegen-stub 封堵）
- **工具版本**：`cargo-kani 0.67.0`
- **本工具实测**：n=161 / SUCCESS=151 / FAILED=10 / UNKNOWN=0，通过率 **93.8%**（151 / 161）
- **时长分布**：avg 3080 ms / median 1004 ms / p90 7236 ms / max 31437 ms
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

**项目自维护组件**：`tools/kani/kani-strict-wrapper.sh` —— runner 调 wrapper 而非直接调 `cargo kani`，wrapper 跑 `cargo kani --only-codegen --bin __ts_harness` 后 grep stdout 5-marker subset；命中即重写 exit 0 → exit 2 + 诊断（封堵 §C1 codegen-with-unsupported-stub 漏报）。

## SUCCESS 信号 + 形式严格性

按宪法 §六 双通路 partial 暴露：

**判定式**：

```
SUCCESS ⟺ kani-strict-wrapper.sh exit 0
       ⟺ cargo kani --only-codegen --bin __ts_harness exit 0
         ∧ stdout 不命中 5 markers 任一：
           - TerminatorKind::InlineAsm   (inline asm MIR terminator)
           - simd_cast                   (packed-SIMD cast intrinsic)
           - catch_unwind                (panic recovery)
           - ptr_mask                    (raw-pointer bit-mask intrinsic)
           - C string literal            (c"..." 字面量 MIR rvalue, Rust 2024)
```

- **主信号通路**：`cargo kani --only-codegen` exit code（kani-compiler 自身报错 → exit ≠ 0 直通 FAILED）
- **wrapper 补抓通路**：5-marker grep gate（kani 自陈"我没把这条干完只 emit stub"——按 §六 Oracle 责任不藏，命中即 FAILED）

**为什么是 5-marker 子集而非完整 `Found the following unsupported constructs:` warning list**：实测 60-63/144 SUCCESS 命中 `caller_location` / `foreign function`，这两条来自 std panic / std alloc 路径几乎每个 non-trivial entry 都触发，纳入会让 ~44% SUCCESS 翻车（含 hello/basic-hello、bigint-arith、industrial/rsa、industrial/sha256-digest）—— 是结构性误报。5-marker 是反误报实测校准出的精筛集合（详 `docs/fixes/oracle-leak-audit-2-2026-05-11.md` §3.1）。

**形式严格性**（不作过强声明）：

- **0 误报状态**：实测 + wrapper 双通路封堵。任意合法 SUCCESS（不触发 5 markers）在 wrapper 下保持 SUCCESS（hello/basic-hello、bigint/bigint-arith、industrial/rsa-pkcs8、industrial/sha256-digest 实测均 SUCCESS）。**注**：未来 kani-compiler 演进可能引入新警告格式或新合法用例命中现有 marker；非形式证明，仅实测校准。
- **0 漏报状态**：5 markers 是 kani 自陈"Verification will fail if one or more of these constructs is reachable"的字面诉求，与"工具接受"互斥。但 `caller_location` / `foreign function` 路径未封堵（结构性排除），属可争议剩余口径——见漏报盲点。

## 失败分桶（按 P31 §四.5 归因分类 + §六 当前 crate 焦点切割）

v6 实测 10 个 FAILED 全部命中 5-marker，按"marker 来源 = entry crate 自己 vs deps / std"细分两桶。该切分是 P34 §六 "当前 crate 焦点（宽度切割）"派生原则的直接应用：测量对象是 entry crate，外部依赖路径下的 opaque / stub 不应入册。

### 桶 1：entry crate 自己写的 MIR 触发 5-marker（2 case，FAILED）

| entry | 命中 markers | 来源（entry 源码层） |
|---|---|---|
| `charon-limit/inline-asm/nop_via_asm` | `TerminatorKind::InlineAsm (1)` | `src/lib.rs` 直接写 `unsafe { core::arch::asm!("nop", ...) }` |
| `kani-limit/stack-unwinding/trigger_divide_with_recovery` | `catch_unwind (1)` | `src/lib.rs` 直接调 `std::panic::catch_unwind(|| numerator / denominator)` |

stderr 特征（举 `inline-asm/nop_via_asm` 一例）：

```
warning: Found the following unsupported constructs:
             - TerminatorKind::InlineAsm (1)
[kani-oracle] FAIL: codegen completed with hard-unsupported MIR constructs
```

**归因**：工具不支持。两个 entry 都是边界样例（`charon-limit/` 与 `kani-limit/`），样例本意就是触发该构件；entry crate 源码层确有 `asm!` / `catch_unwind` 调用，kani-compiler 自陈不能 codegen → emit stub。这是 kani-compiler MIR codegen 的真实能力边界。
**处理**：不修。本地性原则下 FAILED 站得住，工具开发者不能驳回（kani 自家 warning list 就明示了这两类）。

### 桶 2：entry crate 不写 5-marker MIR，markers 全来自 deps / std 内部（8 case，FAILED — **§六 当前 crate 焦点疑虑**）

| entry | 命中 markers | 触发位置（非 entry 源码） |
|---|---|---|
| `concurrency/thread-mutex/thread_mutex_join` | `C string literal (1)` + `catch_unwind (3)` + `ptr_mask (1)` | std `thread::spawn` / `Mutex::new` 内部 |
| `miri-limit/thread-interleaving-partial/unsynchronised_counter_race` | `C string literal (1)` + `catch_unwind (4)` + `ptr_mask (1)` | std `thread::spawn` / `AtomicUsize` 内部 |
| `deps-complex/bigint-serde/bigint_serde` | `InlineAsm (5)` + `simd_cast (2)` | num-bigint / serde derive 内部（lazy_static asm + SIMD） |
| `deps-complex/chrono-serde/chrono_serde` | `InlineAsm (5)` + `simd_cast (2)` | chrono / serde 内部 |
| `deps-complex/collections-serde/collections_serde` | `InlineAsm (5)` + `simd_cast (2)` | std collections / serde 内部 |
| `deps-complex/error-chain/error_chain` | `catch_unwind (1)` + `ptr_mask (1)` + `simd_cast (1)` | anyhow / thiserror / std 内部 |
| `industrial/x509-parser/cert-parse/x509_parse_der` | `catch_unwind (1)` + `ptr_mask (1)` | vendor `x509-parser` + 传递依赖 std panic / alloc 内部 |
| `industrial/x509-parser/cert-parse/x509_subject_extensions` | `catch_unwind (1)` + `ptr_mask (1)` | 同上 |

读 entry 源码确认：8 个 entry 的 `src/lib.rs` 都不直接写 `asm!` / `catch_unwind` / `ptr_mask` / `simd_cast` / `c"..."` —— markers 由 deps（serde derive / num-bigint / chrono / anyhow / thiserror / x509-parser）或 std 内部（`thread::spawn` / `Mutex` / `AtomicUsize`）产生。

stderr 特征（举 `concurrency/thread-mutex` 一例）：

```
warning: Found the following unsupported constructs:
             - C string literal (1)
             - catch_unwind (3)
             - ptr_mask (1)
[kani-oracle] FAIL: codegen completed with hard-unsupported MIR constructs
```

**归因疑虑（§六 当前 crate 焦点）**：宪法 §六 第三段（宽度切割）明示：

> 测量对象是 example 的 entry crate。工具如何处理外部依赖（std / core / 第三方）不计入测量——把外部依赖翻译为 opaque / placeholder / stub 是工具合法的设计选择，不算 silent partial。
> ...
> 外部依赖路径下的 opaque / skip / stub → 不触发 partial 判定

按此精神严格解读，**桶 2 的 8 个 FAILED 应重审**：entry crate 自己的 MIR 完整出现在 kani goto 产物里、kani-compiler 也没在 entry crate item 层 emit stub；stub 出现在 deps / std 路径下，属"工具对外部依赖的合法 placeholder 选择"。当前 5-marker gate 不区分 marker 出处（entry crate vs deps），是 stdout 文本级 grep，无法定位到具体 crate。

**当前实施**：保留 FAILED 不归 SUCCESS。理由有二：

1. **5-marker gate 实施层局限**：kani stdout 的 `Found the following unsupported constructs:` warning header 不带 crate 归属，wrapper 无法区分 entry 内部触发 vs deps 触发。若放宽 gate 一律不 grep，会丢桶 1（entry 自写）的真 FAILED。
2. **保守 default**：宁过严 FAILED，避免越界归 SUCCESS（违反 §六 Oracle 责任"不冤枉"对偶——这里风险方向反过来：宽 gate 才不冤枉，但严 gate 在不能区分时更安全）。

**处理**：暂不修；明示疑虑入 cc-report 修订小组待裁定项。可能的解 / backlog 见漏报盲点 §3。

## 漏报盲点（诚实声明）

**已通过 wrapper gate 封堵**：

- 5 个 hard-unsupported MIR construct stub 路径：`TerminatorKind::InlineAsm` / `simd_cast` / `catch_unwind` / `ptr_mask` / `C string literal`
- 任一命中 → wrapper 重写 exit 2 + 诊断 → 框架记 FAILED

**仍存在的盲点 / 待裁定项**：

1. **`caller_location` / `foreign function` warning 不抓**：实测 60-63/144 SUCCESS 命中。这两条由 std panic / std alloc 路径几乎每个 non-trivial entry 都触发；kani 视为 std 内部标准 stub 处理。按宪法 §六 严格解读这是可争议剩余口径——纳入会大规模假阳性（含 hello/basic-hello），排除则有遗漏。当前选择"排除以避高频假阳性"——属 cc-report 修订小组暂未裁定项。
2. **concurrency 单线程语义警告不抓**：v6 中 8 个 SUCCESS entries 含 kani `"Kani currently does not support concurrency. The following constructs will be treated as sequential operations"` warning（atomic_* / thread_local / fence）。kani-compiler **真 codegen 原子操作**（atomic_block / SKIP / binop），不是 stub —— 这是 BMC 单线程语义约束（不模拟多线程交错），属求解层假设而非前端 partial。按宪法 §六 前端测量原则**不抓 marker**，这些 entries 保持 SUCCESS。该 warning 表征求解层简化口径，列出以诚实声明。
3. **桶 2 deps-only 5-marker 触发归 FAILED 的疑虑（§六 当前 crate 焦点）**：见上 §"桶 2"。当前 5-marker gate 是 stdout 文本级，不带 crate 归属。**理想解**需要 wrapper 解析 kani-compiler 诊断的 source span（如 `--message-format=json`），仅在 marker span 落在 `TS_TARGET_CRATE` 锚定的 entry crate 内时 reject。该改造未实施 → 桶 2 当前以"严归 FAILED"运行；属 cc-report 修订小组暂未裁定项 + 工程 backlog（可在后续 wrapper 升级合并）。该疑虑直接影响 v6 通过率：若桶 2 应改归 SUCCESS，则通过率从 93.8% 升至 159/161 ≈ 98.8%。
4. **新的 unsupported MIR 节点类别**：kani-compiler 演进可能引入新 stub 路径，需扩展 5 markers list；当前 5 markers list 是 0.67.0 实测对齐。
5. **codegen 完成 + SAT 阶段才会触发的能力问题**：本次未观察到（按"前端测量"原则也不在本测试度量范围）。

## v5.1 → v6 ΔS 解释

v5.1（`run-1778466265-63960` / 146 entries）：136 SUCCESS / 10 FAILED / 0 UNKNOWN，通过率 93.2%。
v6（`run-1778560393-59119` / 161 entries）：151 SUCCESS / 10 FAILED / 0 UNKNOWN，通过率 93.8%。

**ΔS = +15**，来源拆解：

- **+15 SUCCESS**：corpus 从 146 增至 161（+15 entries 全在不触发 5 markers 的 feature 类目，均 SUCCESS）。
- **FAILED 列表完全一致**：v5.1 与 v6 实测同样 10 个 FAILED entries（charon-limit/inline-asm、concurrency/thread-mutex、deps-complex/{bigint,chrono,collections}-serde、deps-complex/error-chain、industrial/x509-parser × 2、kani-limit/stack-unwinding、miri-limit/thread-interleaving-partial）。
- **UNKNOWN 状态**：v5.1 与 v6 实测均为 0 UNKNOWN。x509-parser 两条在两个版本下都是直接命中 5-marker gate（`catch_unwind` + `ptr_mask`）→ wrapper 重写 exit 2 → 直接 FAILED，未经 runner 的 `vendor_lint_strictness` UNKNOWN 标签路径。（旧 v5/v6 cc-report 文本声称的 "8 FAILED / 2 UNKNOWN"与本次实测不符；本快照以 results.json 实测为准。）

通过率口径：v6 = 151/161 ≈ 93.8%；v5.1 = 136/146 ≈ 93.2%。差异主要来自 corpus 扩容稀释（FAILED 绝对数不变，新增 entries 全 SUCCESS）。

## 修订建议清单（仅"我们导致"失败）

| # | 桶 | 涉及 case | 修复方案 | 优先级 |
|---|---|---|---|---|
| 1 | 桶 2 §六 当前 crate 焦点疑虑 | 8（concurrency/thread-mutex、miri-limit/thread-interleaving-partial、deps-complex × 4、industrial/x509-parser × 2） | **方案 A（理想）**：升级 wrapper 用 `cargo kani --only-codegen --message-format=json` 解析诊断 span，仅在 marker span 在 `TS_TARGET_CRATE` 内时 reject —— 真正实现 §六 当前 crate 焦点。**方案 B（保守暂行）**：保持现状，文档化疑虑等修订小组裁定。本报告采方案 B + 暴露疑虑入 cc-report，等待裁定后再实施方案 A。 | 中（属 cc-report 修订小组待裁定 + 工程 backlog；不阻塞 v6 baseline） |

**说明**：

- 桶 1（2/10：charon-limit/inline-asm/nop_via_asm + kani-limit/stack-unwinding/trigger_divide_with_recovery）**不修**——entry crate 自写 `asm!` / `catch_unwind`，kani 自陈"我没把这条干完"，本地性原则下 FAILED / 工具开发者不能驳回。这是 kani-compiler 真实能力边界。
- 桶 2（8/10）的归因争议属"我们的 oracle 实施层粒度不够"——不是 kani 工具的锅，也不是 entry corpus 的锅。按 §六 (b) "我们这边可识别的问题"严格解读，理论上应归 UNKNOWN + 附"会修计划"；但当前实施层 gate 是 stdout 文本级，没有可靠的"deps 触发 vs entry 触发"区分手段，强制改 UNKNOWN 反而违反 §六 (a)/(b) 严格语义（无明确归因路径）。本次暂保持 FAILED + 明示疑虑 / backlog，等修订小组裁定（要么实施方案 A，要么修宪法接受当前粒度）。
- 无其他"我们导致"失败：除桶 2 疑虑外，10 个 FAILED 的实施层判定路径都站得住。

## 历史快照声明

本报告是 2026-05-12 v6 final 运行 `runs/run-1778560393-59119` 的实测快照；锚定 `cargo-kani 0.67.0` × 5-marker oracle subset × 当前 corpus（161 entries）。kani 升级、kani-compiler 修复 lint 注入、上游 `Found the following unsupported constructs:` warning 格式变化、cc-report 修订小组对 `caller_location` / `foreign function` 路径口径裁定、桶 2 §六 当前 crate 焦点 wrapper 升级、vendor crate 演化等任一变化后均需重测。
