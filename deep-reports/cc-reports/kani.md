# kani — 特性支持评估报告（v6 baseline）

## 元数据

- **数据源**：`runs/run-1778560393-59119/`（2026-05-12 v6 final，合并 verus rerun + R7 5-tool rerun）
- **工具配置**：`tools/kani/`（含 `kani-strict-wrapper.sh` 5-marker codegen-stub 封堵）
- **工具版本**：`cargo-kani 0.67.0`
- **本工具实测**：n=161 / SUCCESS=151 / FAILED=8 / UNKNOWN=2，通过率 **93.8%**（151 / 161）
- **时长分布**：avg 2976 ms / median 1004 ms / p90 7236 ms / max 31437 ms
- **宪法 baseline**：`principles.md` v8（P27 修宪后 / P31 法律传导后；§六 UNKNOWN 严格语义两类 + 本地性原则）
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

切割理由（公平性）：其他工具（cargo-check / charon / hax / creusot / aeneas 等）均在前端层停下；让 kani 跑完整 CBMC 求解会在集合 / 并发 / 递归枚举等场景下超时频繁 FAILED，制造假阴性。

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
- **wrapper 补抓通路**：5-marker grep gate（kani 自陈"我没把这条干完只 emit stub"——按 §六 反作弊不允许 partial，命中即 FAILED）

**为什么是 5-marker 子集而非完整 `Found the following unsupported constructs:` warning list**：实测 60-63/144 SUCCESS 命中 `caller_location` / `foreign function`，这两条来自 std panic / std alloc 路径几乎每个 non-trivial entry 都触发，纳入会让 ~44% SUCCESS 翻车（含 hello/basic-hello、bigint-arith、industrial/rsa、industrial/sha256-digest）—— 是结构性误报。5-marker 是反误报实测校准出的精筛集合（详 `docs/fixes/oracle-leak-audit-2-2026-05-11.md` §3.1）。

**形式严格性**：

- **0 误报（不冤枉能力）**：实测 + wrapper 双通路封堵。任意合法 SUCCESS（不触发 5 markers）在 wrapper 下保持 SUCCESS（hello/basic-hello、bigint/bigint-arith、industrial/rsa-pkcs8、industrial/sha256-digest 实测均 SUCCESS）。**注**：按 P30 v6 cc audit 精神，不作"形式可证 0 误报"绝对声明——5-marker 是从合法 SUCCESS 反向校准的精筛集，未来 kani-compiler 演进可能引入新警告格式或新合法用例命中现有 marker。
- **0 漏报（不高估能力）**：实测 + 源码层封堵。5 markers 是 kani 自陈"Verification will fail if one or more of these constructs is reachable"的字面诉求，与"工具接受"互斥。但按宪法 §六-2 严格解读，`caller_location` / `foreign function` 路径仍未封堵，属可争议剩余口径 —— 见漏报盲点。

## 失败分桶（按 P31 §四.5 归因分类）

### 桶 1：MIR codegen 完成但 emit stub（5-marker 命中；6 case，FAILED）

代表 entry 与命中标记：

| entry | 命中 markers | stub 来源 |
|---|---|---|
| `charon-limit/inline-asm/nop_via_asm` | `TerminatorKind::InlineAsm (1)` | 用户代码 `unsafe { asm!("nop") }` |
| `concurrency/thread-mutex/thread_mutex_join` | `C string literal (1)` + `catch_unwind (3)` + `ptr_mask (1)` | `std::thread::spawn` 路径 → `__rust_panic_cleanup` 的 catch_unwind + `Mutex::new` 的 ptr_mask |
| `deps-complex/bigint-serde/bigint_serde` | `TerminatorKind::InlineAsm (5)` + `simd_cast (2)` | serde derive + num-bigint-dig 的 lazy_static 路径 asm + SIMD |
| `deps-complex/chrono-serde/chrono_serde` | `TerminatorKind::InlineAsm (5)` + `simd_cast (2)` | 同上 |
| `deps-complex/collections-serde/collections_serde` | `TerminatorKind::InlineAsm (5)` + `simd_cast (2)` | 同上 |
| `deps-complex/error-chain/error_chain` | `catch_unwind (1)` + `ptr_mask (1)` + `simd_cast (1)` | thiserror 派生 + std 错误链 |
| `kani-limit/stack-unwinding/trigger_divide_with_recovery` | `catch_unwind (1)` | 用户代码显式 `std::panic::catch_unwind(...)` |
| `miri-limit/thread-interleaving-partial/unsynchronised_counter_race` | `C string literal (1)` + `catch_unwind (4)` + `ptr_mask (1)` | `std::thread::spawn` 路径同 thread_mutex_join |

（注：上表 8 行 = 5-marker bucket 全部 8 个 FAILED；为表义清晰一并列出，分布形态分三类：用户代码直接触发 / std 内部间接触发 / vendor crate 间接触发）

stderr 特征（举 `inline-asm/nop_via_asm` 一例）：

```
[kani-oracle] FAIL: codegen completed with hard-unsupported MIR constructs
[kani-oracle]       (kani self-disclosed via 'Found the following unsupported
[kani-oracle]       constructs:' warning). The matched markers are:
             - TerminatorKind::InlineAsm (1)
[kani-oracle]       kani replaced these constructs with stubs ("Verification
[kani-oracle]       will fail if one or more of these constructs is reachable"
[kani-oracle]       — kani's own words) but still exits 0 because --only-codegen
[kani-oracle]       does not invoke CBMC. Per project §六-2 反作弊 (no partial
[kani-oracle]       / silent skip), this is a partial-codegen漏报 and must be
[kani-oracle]       FAILED.
```

**归因**：工具不支持。kani 自陈"这些 construct 没真翻译，只 emit stub"——这是 kani-compiler 的 MIR codegen 能力边界。
**处理**：不修。本地性原则下 FAILED 站得住，工具开发者不能驳回（kani 自己的 warning list 就明示了）。

### 桶 2：vendor crate `#![deny(unstable_features, unused_qualifications)]` × kani 注入 `#![feature(register_tool)]` 冲突（2 case，UNKNOWN）

代表 entry：

- `industrial/x509-parser/cert-parse/x509_parse_der`
- `industrial/x509-parser/cert-parse/x509_subject_extensions`

stdout 节选：

```
error: unnecessary qualification
   --> vendor/x509-parser/src/cri_attributes.rs:31:41
note: the lint level is defined here
   --> vendor/x509-parser/src/lib.rs:123:31
123 |         unused_import_braces, unused_qualifications)]

error: use of an unstable feature
   --> <crate attribute>:1:12
  1 | #![feature(register_tool)]
note: the lint level is defined here
   --> vendor/x509-parser/src/lib.rs:122:9

error: Failed to execute cargo (exit status: 101). Found 8 compilation errors.
```

诊断：vendor crate 自陈 `#![deny(unstable_features, unused_qualifications, ...)]`，但 kani-compiler 注入 `#![feature(register_tool)]`（用于 `#[kani::proof]` 的 `register_tool(kanitool)`），同时 kani 的 lint 配置让 `unused_qualifications` 命中 8 处，被 vendor crate 自己的 deny 升为 error。

对比 cargo-check 在同 entry 的行为：cargo-check 也触发 `unused_qualifications` lint，但只是 warning（stable rustc 不需要注入 `register_tool`），所以 cargo-check exit 0。

**归因**：**我们 corpus bug**——我们引入的 vendor crate（`vendor/x509-parser/`）自带 `#![deny(unstable_features, unused_qualifications)]`，与 kani 注入的 `register_tool` feature 撞车。这不是 kani 的 codegen 能力问题（错误发生在 lint 检查阶段未到 MIR 翻译），也不是 vendor crate 设计意图的体现。

**处理**：**已部分修**——runner `classify_external_fault()` 把"`unused_qualifications` + `vendor/` 路径"模式归 `vendor_lint_strictness` UNKNOWN 标签（exit=1），不是 FAILED。属宪法 §六 (b) "我们这边可识别的问题"，但 fix 尚未在 corpus 层落地（vendor crate 未 patch / lint denial 未 drop）。详 §"修订建议清单"。

## 漏报盲点（诚实声明）

**已通过 wrapper gate 封堵**：

- 5 个 hard-unsupported MIR construct stub 路径：`TerminatorKind::InlineAsm` / `simd_cast` / `catch_unwind` / `ptr_mask` / `C string literal`
- 任一命中 → wrapper 重写 exit 2 + 诊断 → 框架记 FAILED

**仍存在的盲点**：

1. **`caller_location` / `foreign function` warning 不抓**：实测 60-63/144 SUCCESS 命中。这两条由 std panic / std alloc 路径几乎每个 non-trivial entry 都触发；kani 视为 std 内部标准 stub 处理。按宪法 §六-2 严格解读这是可争议剩余口径——把它们纳入会大规模假阳性，排除则有遗漏。当前选择"排除以避高频假阳性"——属 cc-report 修订小组暂未裁定项。
2. **concurrency 单线程语义警告不抓**（D3.4 / 2026-05-12 README 补完）：8 个 v5 SUCCESS entries 含 kani `"Kani currently does not support concurrency. The following constructs will be treated as sequential operations"` warning（atomic_* / thread_local / fence）。kani-compiler **真 codegen 原子操作**（atomic_block / SKIP / binop），不是 stub —— 这是 BMC 单线程语义约束（不模拟多线程交错），属求解层假设而非前端 partial。按宪法 §六-3 前端测量原则**不抓 marker**，这些 entries 保持 SUCCESS。该 warning 表征求解层简化口径，不属漏报盲点；列出以诚实声明。
3. **新的 unsupported MIR 节点类别**：kani-compiler / hax-engine 演进可能引入新 stub 路径，需要扩展 5 markers list；当前 5 markers list 是 0.67.0 实测对齐。
4. **codegen 完成 + 其他 warning 但 SAT 阶段才会触发问题的 entry**：本次未观察到（按"前端测量"原则也不在本测试度量范围）。

## v5.1 → v6 ΔS 解释

v5.1（`run-1778466265-63960` / 146 entries）：136 SUCCESS / 10 FAILED / 0 UNKNOWN，通过率 93.2%。
v6（`run-1778560393-59119` / 161 entries）：151 SUCCESS / 8 FAILED / 2 UNKNOWN，通过率 93.8%。

**ΔS = +15**，来源拆解：

- **+15 SUCCESS**：corpus 从 146 增至 161（+15 entries 全在不触发 5 markers 的 feature 类目，均 SUCCESS）
- **-2 FAILED / +2 UNKNOWN**：原属"桶 A：crate deny 与 kani register_tool 冲突"的 2 条 x509-parser entry（v5.1 报 FAILED）在 v6 被 runner `classify_external_fault()` 重归 `vendor_lint_strictness` UNKNOWN。这是 P31 法律传导 + v8 §六 UNKNOWN 严格语义 (b) 类"我们这边可识别的问题"判据落地的结果，属归因升级而非工具能力变化。
- **桶 1（5-marker stub）8 条 FAILED 保持不变**：v5.1 实测 8 条 stub 命中，v6 同 8 条同样命中（charon-limit/inline-asm、concurrency/thread-mutex、deps-complex/{bigint,chrono,collections}-serde、deps-complex/error-chain、kani-limit/stack-unwinding、miri-limit/thread-interleaving-partial）。

通过率口径变化：v5.1 93.2% 是 136/146（FAILED 入分母）；v6 93.8% 是 151/161（UNKNOWN 入分母，但 UNKNOWN 不视为通过）。若 v6 也按 v5.1 的"剔除 UNKNOWN"口径，则 151/(161-2) = 95.0%；按统一口径（任何非 SUCCESS 入分母），v6 = 93.8%、v5.1 = 93.2%，差异主要来自 corpus 扩容稀释。

## 修订建议清单（仅"我们导致"失败）

| # | 桶 | 涉及 case | 修复方案 | 优先级 |
|---|---|---|---|---|
| 1 | 桶 2 vendor crate `#![deny]` × kani `register_tool` 冲突 | 2（x509-parser cert-parse 两条）| **方案 A（推荐）**：patch `vendor/x509-parser/src/lib.rs:122-123` 移除 `unstable_features` 与 `unused_qualifications` 两条 deny —— 仅修我们引入的 vendor crate 自带的严格 lint，不动 example 源码、不动工具，符合宪法 §四 A 双方不可侵入（vendor 属第三方中介，不是 example 也不是工具）。**方案 B**：保留 vendor crate 原样，在 hirusttest.toml 加 per-tool override 在 kani 下跳过这两条 entry —— 但这违反"每 entry 自包含"精神，且制造工具特定 corpus 偏移，不推荐 | 中 |

**说明**：

- 桶 1（6/8 entry 命中 5 markers 的 codegen stub）**不修**——这是 kani-compiler 真实能力边界，工具自陈"我没把这条干完"，本地性原则下 FAILED / 工具开发者不能驳回。
- v6 中桶 2 已由 runner classifier 归为 UNKNOWN 标签 `vendor_lint_strictness`，符合 §六 (b) 第二类 UNKNOWN 严格语义；但 corpus 层 fix（patch vendor lib.rs）未落地——长期保持 UNKNOWN 是合规的"我们这边可识别且暂未修"，符合宪法 §六 "附明确归因 + 会修计划"。短期 priority 中（不阻塞 v6 baseline），属可在后续 corpus 维护批次合并的 fix。
- 无其他"我们导致"失败：所有 8 个 FAILED 均为工具能力边界。

## 历史快照声明

本报告是 2026-05-12 v6 final 运行 `runs/run-1778560393-59119` 的实测快照；锚定 `cargo-kani 0.67.0` × 5-marker oracle subset × 当前 corpus（161 entries，含 6 个 industrial vendor crate entries + 新增 15 entries 的 feature 扩展）。kani 升级、kani-compiler 修复 lint 注入、上游 `Found the following unsupported constructs:` warning 格式变化、cc-report 修订小组对 `caller_location` / `foreign function` 路径口径裁定、vendor crate lint patch 等任一变化后均需重测。
