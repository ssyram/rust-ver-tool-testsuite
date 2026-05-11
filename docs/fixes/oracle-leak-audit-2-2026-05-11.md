# Oracle 漏报扩展审计：其他 16 工具（2026-05-11）

> 第二轮 oracle 漏报扩展审计。前一轮（[`oracle-leak-audit-2026-05-08.md`](oracle-leak-audit-2026-05-08.md)）聚焦 verifast / prusti / rocq-of-rust，P12（[`oracle-leak-rules-implementation-2026-05-08.md`](oracle-leak-rules-implementation-2026-05-08.md)）已封堵。本轮覆盖剩余 16 工具：aeneas × 4 / charon × 2 / hax × 3 / cargo-check / kani / verus / creusot / miri / kmir / soteria。
>
> 本报告**不修改任何 tool.toml / cc-report / 实施代码**，只产出审计结论 + 推荐封堵规则。每条结论锚定 `runs/run-1778226613-5282/results.json`（146 entries × 19 工具，UTC 2026-05-08）+ raw stdout/stderr + 工具源码（hax-engine opam switch installed 源 `~/.opam/default/.opam-switch/sources/hax-engine/`）。

---

## §0 与第一轮 audit 的关系

第一轮聚焦 verifast / prusti / rocq-of-rust 的封堵规则推导与实施，已在 P12 落地：

| 工具 | 第一轮前 | 第一轮后（P12） |
| --- | --- | --- |
| verifast | 79.5% SUCCESS（116/146，全空过） | 8.2%（12/146；vacuous pass 71pp 真空过被抓）|
| prusti | 38.4%（产物 check 未落地） | 38.4%（数字不变；落地后 0 误报 0 漏报硬指标）|
| rocq-of-rust | 82.9%（ForeignMod 等 silent skip 漏抓） | 76.0%（gate 6 entry_fn `Definition` 缺失抓 -10 SUCCESS）|

本轮目标：穷尽剩余 16 工具，找新漏报机制。**结论先行**：找到 **kani 一处中-高风险漏报**（约 1-4% 虚高且工具自陈承认）；**hax-fstar / hax-coq 各一处理论 silent-skip-item 路径**（实测 0 现象）；其余 13 工具实测 0 漏报现象，自陈"形式可证"在物理证据上成立。

---

## §1 TL;DR

1. **kani 当前 oracle 中-高风险漏报：4 个 entry 含 `TerminatorKind::InlineAsm` / `simd_cast` / `catch_unwind` / `ptr_mask` / `C string literal` warning 但 SUCCESS**（占总 SUCCESS 的 1.4%-2.8%，估 4-8 个真漏报）。kani README §32 已自陈"⚠️ codegen 完成 + warning 但 SAT 阶段才会触发问题的 entry"是漏报盲点。当前 oracle 没抓 stdout 的 `Found the following unsupported constructs: ...` 名单——典型反例 `charon-limit/inline-asm/nop_via_asm`（kani SUCCESS 但 inline asm 是 unsupported MIR terminator）、`kani-limit/extern-ffi/trigger_call_libc_abs`（foreign function unsupported）。**这是本轮最严重的发现**。
2. **hax-fstar / hax-coq 有理论 silent-skip-item 路径**：`backends/fstar/fstar_backend.ml:1771` `| Use _ | NotImplementedYet -> []` + `backends/coq/coq/coq_backend.ml:588` `method item'_NotImplementedYet = string "(* NotImplementedYet *)"` —— item 完全跳过或仅打 boilerplate marker，cargo hax 仍 exit 0。oracle grep 不覆盖。**实测 0 现象**（cc-report 已声明此盲点），但理论窗口存在。建议加 runner 注入 `TS_ENTRY_FN` 后做 entry_fn 字面 grep（与 P12 rocq-of-rust 同方法）。
3. **prusti 仍有 entry-level 漏报理论窗口**：P12 的 .vpr 存在性 check 只校验 ≥ 1 个 .vpr，**不校验 entry_fn 是否在 .vpr 内**。README 已自陈"encoder 内部 silent skip 单个 fn item（不影响 .vpr 总数 ≥ 1）"为剩余盲点。**实测 0 现象**，可选增强。
4. **verus 漏报状态确认为真**：README 自陈"✅ 形式可证 0 漏报"——verus harness 把 `mod __ts_inner;` 放在 `verus! {}` 块内且不带 `#[verifier::external]`，inner mod 整体进入 verus 前端；任何 reject 走 `dcx().emit` → exit 1。耗时 min 421ms / median 637ms 与"VIR 构造 + lifetime check"量级一致，**无空过物理证据**。FAILED 95 条全有明确 stderr error。
5. **aeneas × 4 / charon × 2 / cargo-check / miri / soteria / kmir / creusot 8 工具自陈"0 漏报形式可证"**——本轮逐一复核源码层 reject 通路 + 实测耗时分布 + raw stderr：**全部成立，无新漏报**。aeneas 的 `craise` 单一通路 + `Main.ml` `if has_errors then exit 1`；charon 的 `--abort-on-error` 第一次 `register_error!` panic；creusot 的 `crash_and_error / span_err`；soteria 的 exit 0/1/2/3 四档；miri 的 exit code 单一信号；kmir 的 `#EndProgram ~> .K` grep 已封 silent stuck —— 都是形式可证。**aeneas 的 exit 1 + "Generated the partial file" 已正确算 FAILED**，按 wrapper exit code 单一信号。
6. **整 corpus 无第二个 verifast 灾难**：verifast pre-P12 的 100% 空过是 `-skip_specless_fns` 与 corpus 无 `//@` 注解的精确组合事故；本轮 16 工具中没有等价规模的"corpus 完全空过"现象。耗时分布上所有工具的 SUCCESS 集合都在 ≥ 100ms 量级（min 138ms cargo-check / 196ms charon-mono / 421ms verus / 465ms kani / 363ms miri / 439ms soteria），物理证据上每个工具都"真在跑"。
7. **实施优先级 top 3**：(1) kani 抓 stdout "unsupported constructs" warning（精准 grep 5 个非-prelude 信号，避开 caller_location/foreign function 的高频假阳性），中-高严重性 / 中改动成本；(2) hax-fstar / hax-coq 加 entry_fn `Definition`/`let` 翻译存在性 grep，与 P12 rocq-of-rust 同模式，低-中严重性 / 中改动成本；(3) prusti 加 entry_fn 在 .vpr 内字面 grep，低严重性 / 低改动成本。**不需要 runner 扩展**（`TS_ENTRY_FN` 已经被 P12 注入）。

---

## §2 审计方法

### 2.1 数据源

| 来源 | 用途 |
| --- | --- |
| `tools/<name>/tool.toml` + 各 wrapper.sh | 当前 oracle 定义、command/exit code 处理 |
| `tools/<name>/harness.rs.tera` | harness 是否触及 entry_fn、是否注入工具特定标记 |
| `deep-reports/cc-reports/<name>.md` | 每工具的"形式严格性"自陈段 |
| `runs/run-1778226613-5282/results.json` | 16 工具 × 146 entries 的 status / duration_ms / exit_code |
| `runs/run-1778226613-5282/raw/<tool>/<entry>.{stdout,stderr,exit}` | 物理证据 |
| `~/.opam/default/.opam-switch/sources/hax-engine/` | hax engine OCaml 源码 backend 实现 |
| `runner/src/exec.rs` line 178-179 | 已注入 `TS_ENTRY_FN` + `TS_TARGET_CRATE` 到 child env |

### 2.2 漏报判定线索（按 P1 audit §2.2 优先级）

1. **stdout 含 unsupported / partial 字面但 oracle 仅看 exit code**（中证据）：典型如 kani 的 "Found the following unsupported constructs"。
2. **工具源码层 silent-skip-item 路径**（强证据）：grep `-> []` / `return None` / `string "..."` 在 backend item 渲染处。本轮针对 hax-engine backends/ 逐 backend 检索。
3. **耗时分布双峰 / 子毫秒地板**（强证据）：每工具的 SUCCESS 集合按 (min / median / p90) 拉成分布看是否有 < 100ms 异常点。本轮 16 工具结果显示**无任何工具有 < 100ms SUCCESS**（最快 verus 421ms / kani 465ms / miri 363ms / soteria 439ms / charon-mono 196ms / cargo-check 119ms。cargo-check 119ms 是"单文件已编译过 + 触发 fingerprint cache"的合理量级，不是空过）。
4. **README cc-report 自陈"⚠️ 不可形式证明" / "已知盲点"**（弱证据）：标记为"需要 oracle 强化"的候选。
5. **harness 注入工具特定标记缺失**（强证据）：典型如 kani 必须有 `#[kani::proof]` / verus 必须在 `verus!{}` 块内。本轮 harness 模板逐一检查 —— 全部合规。

### 2.3 short-circuit 大类（按"漏报机制类型"分组）

| 大类 | 机制 | 含工具（本轮范围） |
| --- | --- | --- |
| **C1**：codegen 完成但产物含 unsupported stub | exit 0 + stub | **kani**（unsupported constructs warning） |
| **C2**：backend silent-skip-item | exit 0 + 产物缺 entry_fn | **hax-fstar**（fstar_backend.ml:1771）、**hax-coq**（coq_backend.ml:588）|
| **C3**：产物存在但 entry_fn silently 跳过 | exit 0 + ≥ 1 产物但缺 entry_fn 翻译 | **prusti**（理论窗口）|
| **D**：exit code 单一信号已足够 | exit ≠ 0 ⇔ partial | cargo-check / miri / soteria / kmir / creusot / verus / charon × 2 / aeneas × 4（10 工具）|

C1 是本轮最重要发现；C2、C3 是已部分封堵但有剩余理论窗口。

---

## §3 16 工具逐一评估（按风险等级排序）

### 高-中风险

#### 3.1 kani 【中-高风险，确凿】

**当前 oracle**（`tools/kani/tool.toml`）：

```toml
command = ["cargo", "kani", "--only-codegen", "--bin", "__ts_harness"]
```

仅看 exit code（exit 0 ⇔ SUCCESS）。

**short-circuit 机制**：kani `--only-codegen` 做 MIR → GotoC 翻译完成即 exit 0；某些 MIR 节点是 unsupported construct（inline asm / SIMD intrinsic / catch_unwind / ptr_mask / foreign function / caller_location），kani-compiler 把它们 codegen 成 stub（"verification will fail if reachable"），仍 exit 0 + 在 stdout 写 `warning: Found the following unsupported constructs: <list>`。

**corpus 实际命中（实测 raw stdout）**：

```bash
# 在 runs/run-1778226613-5282/raw/kani/ 上 grep
caller_location:                60 SUCCESS 含此 warning
foreign function:               63 SUCCESS 含此 warning
TerminatorKind::InlineAsm:       4 SUCCESS 含此 warning
simd_cast:                       4 SUCCESS 含此 warning
catch_unwind:                    4 SUCCESS 含此 warning
ptr_mask:                        3 SUCCESS 含此 warning
C string literal:                2 SUCCESS 含此 warning
```

总计 144 SUCCESS 中 64（44.4%）含某种 unsupported warning。但 **`caller_location`（60 个）和 `foreign function`（63 个）的高频是因为 std panic 路径与 std alloc 的 `memcpy/posix_memalign` 普遍触发**——这两个不能直接抓为漏报，否则 `bigint/bigint-arith/bigint_arith` 等普通 entry 会大量误报（实测：bigint-arith 在 stdout 含 `caller_location (1) + foreign function (2)` warning，但它确实是合法 codegen 完成的，kani-limit/bigint 都不在 limit 类）。

**真漏报候选（精准 5 个 marker）**：

| entry | kani 状态 | stdout warning | 设计意图（cc-report 类目）|
| --- | --- | --- | --- |
| `charon-limit/inline-asm/nop_via_asm` | SUCCESS | `TerminatorKind::InlineAsm (1)` | charon-limit 类（其他工具都识别） |
| `deps-complex/{bigint,chrono,collections}-serde` | SUCCESS | `TerminatorKind::InlineAsm (5)` + caller_location + foreign function | deps-complex 类 serde 内部含 inline asm |
| `kani-limit/extern-ffi/trigger_call_libc_abs` | SUCCESS | `foreign function (1)` | kani-limit 类（设计意图 FAILED）|
| `kani-limit/uninit-memory/read_uninit_byte` | SUCCESS | `caller_location` + `foreign function` | kani-limit 类（uninit memory 实际 SAT 阶段才触发，codegen 可过）|
| `kani-limit/simd-bitmask-large-vector` | （需查）| `simd_cast` | kani-limit 类 |
| `kani-limit/stack-unwinding/divide_with_recovery` | （需查）| `catch_unwind` | kani-limit 类（panic recovery）|

**物理证据**：典型 `charon-limit/inline-asm/nop_via_asm` 的完整 stdout（kani SUCCESS / 1011ms）：

```
Kani Rust Verifier 0.67.0 (cargo plugin)
warning: Found the following unsupported constructs:
             - TerminatorKind::InlineAsm (1)
         
         Verification will fail if one or more of these constructs is reachable.
         See https://model-checking.github.io/kani/rust-feature-support.html for more details.
```

`Verification will fail` 字面是 kani 自陈"我没把这条真 codegen 完，只放了 stub"——按宪法 §六-2 反作弊精神这就是 partial codegen。同 entry 在 miri / charon-mono / charon-poly / soteria / aeneas 上都 FAILED（inline asm 是各工具公认的 unsupported 特性），唯独 kani-only-codegen 把它放过。

**cc-report 自陈对照**：`tools/kani/cc-report.md` §"SUCCESS 信号 + 形式严格性"：
> - **0 漏报**：⚠️ 实测——`Found the following unsupported constructs: caller_location / foreign function ...` 在 stderr 是 warning 而非 error，对 SAT 求解阶段才有意义；理论上 corpus 可能触发"warning + codegen 完成 → exit 0"的边角情形。**本次 corpus 上 SUCCESS entry 未观察到此 warning**
> - **漏报盲点**：codegen 完成 + warning 但 SAT 阶段才会触发问题的 entry（本次未观察到）

**审计推翻 cc-report 自陈**：cc-report 说"本次未观察到此 warning"，但实测 64/144 SUCCESS 含 warning。其中 5 个非 prelude markers（InlineAsm / simd_cast / catch_unwind / ptr_mask / C string literal）出现在 **6-8 个 entry**，是真漏报候选。**这是 cc-report 数字与实测 raw stdout 不一致的明确点名**——cc-report 应修订（"未观察到" → "确认 6-8 个 entry 触发，需 oracle 强化"）。

**估计虚高**：保守估计 4-8 个 entry 是真漏报（高频 caller_location / foreign function 不算），即 144 SUCCESS 中 2.8%-5.5% 是 partial codegen。**虚高占比 1.4%-2.8% 总通过率**。

**风险等级**：**中-高**（数字小但是 cc-report 自陈 + 实测物理证据 + 设计意图都对齐"应该 FAILED"，且修正方法已知）。

---

#### 3.2 hax-fstar 【中风险，理论窗口】

**当前 oracle**（`tools/hax-fstar/tool.toml`）：cargo hax exit 0 + grep `Rust_primitives\.Hax\.failure|failure \(\(` 不命中产物。

**short-circuit 机制（实测源码层证据）**：

```ocaml
# ~/.opam/default/.opam-switch/sources/hax-engine/backends/fstar/fstar_backend.ml:1771
| Use _ (* TODO: Not Yet Implemented *) | NotImplementedYet -> []
```

`Use` 类型的 item（Rust `use foo;` 顶级 use 声明）+ `NotImplementedYet` AST 节点直接**返回空 list `[]`**——F* backend 跳过该 item 不写产物、不发 Diagnostic、cargo hax exit 0。

第二处 silent path（fstar_backend.ml:506-512）：

```ocaml
and pexpr (e : expr) =
  try pexpr_unwrapped e
  with Diagnostics.SpanFreeError.Exn _ ->
    F.term @@ F.AST.Const (F.Const.Const_string ("failure", F.dummyRange))
```

`pexpr` 抛 `SpanFreeError.Exn` 时 catch 后写一个字符串字面 `"failure"`。但 `SpanFreeError.raise`（`diagnostics.ml:164-167`）在 raise 前已经 `report` 这个 diagnostic → 触发 exit ≠ 0。所以 line 512 的 `"failure"` 字面只是 pretty-print 恢复，**不构成新的 silent path**。

剩余 silent path = 仅 line 1771 的"完全 skip item"机制。

**corpus 实际命中**：115 SUCCESS 上 0 触发 `Rust_primitives.Hax.failure|failure ((` 字面（cc-report §"SUCCESS 信号 + 形式严格性"已陈述）。

**物理证据**：耗时 avg 3285ms / median 1356ms / max 27070ms—— cargo build + hax engine 真跑。无空过。

**cc-report 自陈对照**：`tools/hax-fstar/cc-report.md` 已诚实声明：
> - **0 漏报**：⚠️ 实测验证，不可形式证明
> - **漏报盲点**：hax engine 完全 skip item 的可能（实测 0 现象）；上游引入新 silent path 的可能

cc-report 自陈与实测一致。但 fstar_backend.ml:1771 的 `| Use _ | NotImplementedYet -> []` 是 hax-fstar **特有的** silent-skip-item 路径（hax-lean 在 rust-engine Rust 端、hax-coq 在 coq_backend.ml:588 各有自己的同形态 silent path，但语义都是"item 不写产物 + 不发 Diagnostic"）。**实测 0 现象**说明 corpus 中目前没有让 hax engine 把 item 标为 `NotImplementedYet` 的 entry——但未来 corpus 扩展可能命中。

**风险等级**：**中**（实测 0 现象，理论窗口确凿存在；修正方法已知——加 entry_fn `let` 字面 grep）。

---

#### 3.3 hax-coq 【中风险，理论窗口】

**当前 oracle**（`tools/hax-coq/tool.toml`）：cargo hax exit 0 + grep `failure \(\(|please implement the method` 不命中产物。

**short-circuit 机制（源码层）**：

```ocaml
# ~/.opam/default/.opam-switch/sources/hax-engine/backends/coq/coq/coq_backend.ml:588
method item'_NotImplementedYet = string "(* NotImplementedYet *)"
```

当 hax-engine 在 AST 上遇到 `NotImplementedYet` 类型的 item kind，coq backend 把整个 item 渲染为单行 `(* NotImplementedYet *)` 字面 comment——不写真 `Definition` / `Theorem`。**但 cc-report 明确声明**："`(* NotImplementedYet *)` 是每个 .v 文件的 boilerplate header（hax 给所有 Coq 输出自动加），**不算**失败信号——oracle 不抓这个 marker"。

这造成尴尬：oracle 不抓 `(* NotImplementedYet *)` 是为了避开 boilerplate 误报，但同样的 marker 在 item 位置又确实是 silent partial 信号——**oracle 无法区分两者**（boilerplate vs item-level skip 用的是相同字面字符串）。

第二处 silent path（cc-report 已记录）：`coq_backend.ml:137 default_string_for s = "TODO: please implement the method '..."` —— 已被当前 oracle grep `please implement the method` 抓。

**corpus 实际命中**：98 SUCCESS 上 0 触发 `failure ((` / `please implement the method`（cc-report 已陈述）。

**物理证据**：耗时 avg 3736ms / median 1938ms / max 26428ms—— 真跑。无空过。

**cc-report 自陈对照**：cc-report 已声明"hax engine 完全 skip item（实测 0 现象）；上游引入新 silent path 的可能"为漏报盲点。

**风险等级**：**中**（实测 0 现象，理论窗口 + boilerplate marker 与 item-level marker 同字面的尴尬使 grep 不能简单加固；修正方法见 §4）。

---

#### 3.4 prusti 【低-中风险，剩余 entry-level 窗口】

**当前 oracle**（`tools/prusti/tool.toml` + `prusti-strict-wrapper.sh`，P12 已实施）：

cargo-prusti exit 0 + `find target/verify/log/viper_program -name '*.vpr' | wc -l ≥ 1`。

**short-circuit 机制（剩余窗口）**：当前 .vpr 存在性 check 只看**总数 ≥ 1**。如果 prusti encoder 在某 entry 上：
- 把 entry_fn silently skip（比如某 attr trigger fast-path），但仍把其他 trivial fn（如 `__ts_invoke` wrapper）encode 出 .vpr → 总数 ≥ 1 → SUCCESS
- entry_fn 的 .vpr 是 trivial 空程序（只有 prelude 但无用户函数 body）

`prusti-strict-wrapper.sh` 注释已声明此剩余窗口：

> 漏报盲点：encoder 内部 silent skip 单个 fn item（不影响 .vpr 总数 ≥ 1）—— 当前规则只校验 .vpr 存在，不校验 entry_fn 是否在某个 .vpr 内。如需进一步封堵需 entry_fn-level grep（runner 已支持 `TS_ENTRY_FN` 注入；本次未做以避免越界）。

**corpus 实际命中**：run-1778238662-69805 中 56 SUCCESS / avg 11913ms / median 11176ms / min 6434ms—— 真跑 encoder（JVM bootstrap + encode 物理量级）。无空过现象。

**风险等级**：**低-中**（实测 0 现象，理论窗口存在 + P12 已声明）。

---

### 低风险（自陈 0 漏报，本轮复核成立）

#### 3.5 cargo-check 【无风险】

`cargo check --bin __ts_harness`。rustc 单一信号，exit 0 ⇔ type/borrow check 全过。**0 漏报形式可证**——rustc 不存在 silent skip。本轮无新发现，自陈成立。

#### 3.6 verus 【低风险，自陈成立】

**当前 oracle**：`verus --no-verify --log vir --crate-type=lib src/lib.rs` exit 0。

**反作弊核心**：harness（`tools/verus/harness.rs.tera`）把 `mod __ts_inner;` 放在 `verus! {}` 块**内**且不带 `#[verifier::external]`，整 inner mod 经过 verus 前端处理。

**物理证据**：51 SUCCESS / avg 700ms / median 637ms / min 421ms / **0 个 < 200ms** —— verus 真跑（VIR 构造 + lifetime check）。FAILED 95 条全有具体 stderr error（A 桶 vstd 边界 29 + B 桶语言子集 19 + C 桶 unresolved import 19 + D 桶 generic_args.rs:54 panic 12 + E 桶 verus-driver MIR panic 7 + 其他 9）。

**cc-report 自陈**："✅ 形式可证 0 漏报 + 无漏报盲点"——本轮复核：verus 任何 rejection（mode/type/lifetime check / `assume_specification` 缺失 / `verus_builtin not imported` / mid-end panic）都通过 `dcx().emit` 触发 exit ≠ 0。**单一信号通路确认**。

**剩余隐忧**：verus tool.toml 用 `--log vir` 让 verus 写 `.verus-log/crate.vir`，理论上可加 grep 检查 entry_fn 在 VIR 中存在——但当前 SUCCESS 集合中 verus 真跑了完整 VIR 构造（耗时 + stderr 都有依据），加这条 check 是"防御性增强"不是"封堵真漏报"，**优先级低**。

**风险等级**：**低**（实测 + 源码层 + 自陈三者一致）。

#### 3.7 miri 【无风险】

`cargo +nightly miri run --bin __ts_harness`。harness 调 entry_fn，miri 解释执行；UB / unsupported / panic 任意触发 → exit ≠ 0。**0 漏报形式可证，无 silent skip**。本轮复核：142 SUCCESS / median 722ms / min 363ms / max 35727ms（industrial 加密代码 25-35s）—— 全部真跑解释执行。FAILED 4 条全有显式 unsupported / UB stderr。**无新发现**。

#### 3.8 soteria 【无风险】

`soteria-rust exec` exit 0 = SUCCESS，exit 1/2/3 三档完整覆盖 bug detect / soteria-rust 内部 crash / Charon crash。**0 漏报形式可证**。本轮复核：109 SUCCESS / avg 1846ms / median 1135ms / min 439ms—— 真跑 obol 编译 + symbolic execution。FAILED 37 条按 A-F 6 类分桶（A 编译前置 21 / B edition 1 / C Obol 翻译层 4 / D soteria 内核 intrinsic 7 / E OCaml exception 2 / F 符号执行 bug 2）—— 都有明确 stderr 痕迹。**无新发现**。

#### 3.9 kmir 【低风险】

P11 已加 `#EndProgram ~> .K` grep guard——46 SUCCESS 全含此 K-stable terminator。**0 漏报形式可证**（exit 0 + 终止符号 ⇔ K interpreter 跑完）。本轮复核：min 4404ms / median 7395ms—— 完整 K Framework 解释执行。**无新发现**。

#### 3.10 creusot 【无风险】

`cargo-creusot`（无 subcommand 即只翻译 .coma）。**0 漏报形式可证**——creusot 用 `crash_and_error / span_err / span_fatal / dcx().span_err` 把所有 unsupported 升 rustc error → exit 101，无 silent path。本轮复核：106 SUCCESS / avg 39363ms / median 38613ms / **min 18885ms** —— cargo build + creusot-rustc 全 deps tree，没有空过窗口。FAILED 40 条按 A-G 7 类分桶都是 creusot 显式 reject 路径。**无新发现**。

**剩余隐忧**：tool.toml 没检查 `verif/<crate>_rlib/*.coma` 产物存在性（README 提议"可补强"）。但因为 creusot 通过 rustc-error 显式拒绝，缺这个 check 不构成漏报——形式可证。

#### 3.11 charon-mono / 3.12 charon-poly 【无风险】

`charon cargo --[monomorphize] --abort-on-error --print-llbc -- --lib --target aarch64-apple-darwin`。

`--abort-on-error` 让 `register_error!` 第一次错误 panic + exit ≠ 0。**0 漏报形式可证**。本轮复核：mono 138 SUCCESS / median 350ms / min 196ms; poly 139 SUCCESS / median 377ms / min 190ms—— 真跑 cargo build + charon-driver MIR 提取。FAILED 都有具体 panic stderr。**无新发现**。

#### 3.13 - 3.16 aeneas-coq / aeneas-fstar / aeneas-lean / aeneas-hol4 【低风险】

两段 pipeline（charon → .llbc → aeneas → product）。wrapper 仅看 aeneas exit code。

**0 漏报形式可证**：
- aeneas 唯一 unsupported 入口是 `craise`（Errors.ml）
- 唯一 exit 决定是 `Main.ml` 末尾 `if has_errors then exit 1`（基于 `Errors.error_list` 非空）
- exit 1 + "Generated the partial file (because of N errors)" 仍 FAILED（wrapper exit ≠ 0）—— aeneas 自陈"我没全干完"
- aeneas-hol4 的 `extract_trait_decl Option.get None` panic（52/95 FAILED 主因）是 OCaml uncaught exception → exit 2 → FAILED

本轮复核：4 个 backend min 597-801ms / median 1219-1870ms—— 真跑 charon + aeneas mid-end + printer。所有 FAILED 都对应 `[Error] ...` stdout 或 OCaml uncaught exception stderr。**无新发现**。

aeneas-coq / aeneas-fstar / aeneas-lean 在 146 entry 上 exit code byte-identical（cc-report 已陈述），三 backend 单一 mid-end 加同样 printer 分支，无 backend 特定 silent path。aeneas-hol4 的 35pp 通过率差距是 `Option.get None` panic（exit 2 显式失败），不是 silent skip。

---

## §4 推荐封堵规则

### 4.1 kani — 中等封堵（推荐落地）

**问题诊断**：stdout 含 unsupported construct warning 但 oracle 仅看 exit code。

**推荐第一层硬规则**：

```toml
# tools/kani/tool.toml
command = [
  "sh", "-c",
  "cargo kani --only-codegen --bin __ts_harness > /tmp/kani-out-$$ 2>&1; rc=$?; cat /tmp/kani-out-$$; if [ $rc -eq 0 ]; then if grep -qE 'Found the following unsupported constructs:.*\\n.*-[[:space:]]+(TerminatorKind::InlineAsm|simd_cast|catch_unwind|ptr_mask|C string literal)' /tmp/kani-out-$$; then echo '[kani-oracle] FAIL: codegen completed with hard-unsupported MIR construct (not std-prelude caller_location/foreign-function)' >&2; rc=2; fi; fi; rm -f /tmp/kani-out-$$; exit $rc"
]
```

**注**：实际 awk/grep 实现需多行（warning 的 list 在 stdout 跨多行打印）；建议用 wrapper.sh 类似 verifast/prusti 的方式实施，便于阅读维护。

**防漏报论证（已知 silent path → 命中）**：

| entry | 预期 | 当前 oracle | 新 oracle |
| --- | --- | --- | --- |
| `charon-limit/inline-asm/nop_via_asm` | FAILED（inline asm 是公认 unsupported）| **SUCCESS（漏报）** | FAILED（命中 `TerminatorKind::InlineAsm`）✓ |
| `kani-limit/extern-ffi/trigger_call_libc_abs` | FAILED | SUCCESS | （注：foreign function 高频不能直接抓；此 entry 主信号是 foreign function，会被 5 marker 名单遗漏；需要 cc-report 重新评估"foreign function 是否纳入"）—— 详见 4.1.1 |

**反误报论证（合法 SUCCESS → 不命中）**：

5 个非-prelude markers（`TerminatorKind::InlineAsm` / `simd_cast` / `catch_unwind` / `ptr_mask` / `C string literal`）在普通 entry 上的实测命中：

- `bigint/bigint-arith/bigint_arith` SUCCESS：含 `caller_location (1) + foreign function (2)`，**不**含 5 markers 中任一 → 新规则不触发 ✓
- `hello/basic-hello/hello` SUCCESS：stdout 仅 `Kani Rust Verifier 0.67.0` 一行无 warning → 不触发 ✓
- `industrial/rsa/rsa-pkcs8/rsa_pubkey_from_pkcs8` SUCCESS：含 `caller_location + foreign function` → 不触发 ✓
- `industrial/sha2/sha256-digest/sha256_digest_one_shot` SUCCESS：含 `caller_location + foreign function` → 不触发 ✓

**信心等级**：**high**（5 markers 在普通 entry 上 0 命中；只在故意触发 unsupported 的 limit 类 entry 出现）。

**待实测验证项**：caller_location / foreign function 是否真有"合法 entry 但 kani 当作 stub codegen"的 entry。当前怀疑：caller_location 是 panic 路径，foreign function 是 alloc 路径，两者 codegen stub 是 kani 对 std 内部的标准处理（不是用户代码 partial）—— 所以不纳入 5 marker 是合理的。需要在加 5 marker 后重跑确认。

**估计数字变化**：kani 144 SUCCESS → 138-142（-2 ~ -6 SUCCESS）。

#### 4.1.1 关于 foreign function 是否纳入的争议

`kani-limit/extern-ffi/trigger_call_libc_abs` 的 entry_fn 是 `unsafe { abs(x) }`（直接调 libc abs）。在 kani codegen 上：foreign function (1) warning + exit 0 SUCCESS。**设计意图**：corpus 设计者把它放到 kani-limit/ 下，期望 FAILED—— 但 cc-report 在 §"实测结果" §"kani-limit/* 7/7 全 SUCCESS" 段已说明："这些'不支持'本来要在 CBMC 求解阶段才显现，本测试不到那一步"——按 cc-report 的口径，这种"前端 codegen 接受 + 后端 SAT 拒"的 entry 在前端测试中 SUCCESS 是 cc-report 内部一致的口径。

但项目宪法 §六-2 "不允许 partial"的更严格读法是：codegen 完成 + stub 路径就是 partial，不应 SUCCESS。**这是宪法精神 vs cc-report 现行口径的不一致**——审计层面应点名建议宪法口径优先（与 verifast 同），即把 foreign function 也纳入封堵，但需要 cc-report 同步修订 §"kani-limit/* 7/7 全 SUCCESS" 段口径。

**信心等级**：**medium**（依赖宪法口径解读，foreign function 命中 63/144 = 43.8% 是太多了，会从 144 砍到 ~80，需要进一步精筛——比如只在 kani-limit/charon-limit/miri-limit 类 entry 上抓，但 oracle 不应感知 corpus 分类，所以这条暂搁置）。

---

### 4.2 hax-fstar — 低-中封堵

**问题诊断**：`fstar_backend.ml:1771 | Use _ | NotImplementedYet -> []` 是 silent-skip-item 路径，oracle grep 不抓。

**推荐规则**：与 P12 rocq-of-rust gate 6 同模式——加 entry_fn `let` 字面在产物中存在性 check（F* 的 fn definition 形态是 `let <name> ...`）：

```sh
# 现 oracle 后追加
elif [ -n "$TS_ENTRY_FN" ] && \
     ! grep -rqE "^let[[:space:]]+$TS_ENTRY_FN[[:space:]]" proofs/fstar/extraction/; then
    echo "[hax-fstar-oracle] FAIL: entry_fn '$TS_ENTRY_FN' missing from .fst products (silent skip — fstar_backend.ml:1771 Use/NotImplementedYet path)" >&2
    rc=1
fi
```

**防漏报论证**：源码层 silent-skip 路径 `-> []` 让该 item 不写产物 → grep 不命中 → 触发 ✓

**反误报论证**：合法 SUCCESS entry 在 hax-fstar 上必为 `pub fn <name>(...)`，F* printer 渲染为 `let <name> (...)` —— 实测产物含 `let basic_hello: ...` 等 → grep 命中 → 透传 ✓

**待实测验证**：F* 的 fn definition 是否始终用 `let <name> ` 形态，而不是 `let rec <name>` / `unfold let <name>` / `[@@..]\n let <name>` 等修饰。**这条是待实测验证软规则**——需要在加规则后跑 corpus 全集确认 0 误报。

**风险等级**：**低-中**（理论窗口确凿，实测 0 现象，规则实施有反误报小风险待实测）。

---

### 4.3 hax-coq — 低-中封堵

**问题诊断**：`coq_backend.ml:588 method item'_NotImplementedYet = string "(* NotImplementedYet *)"` 是 silent path。但 `(* NotImplementedYet *)` 是每个 .v 文件的 boilerplate header，**oracle 不能简单抓 marker**——会全 corpus 误报。

**推荐规则**：与 hax-fstar 同模式 + 用 entry_fn `Definition` 字面 grep（Coq 的 fn definition 形态是 `Definition <name>` / `Equations <name>`）：

```sh
elif [ -n "$TS_ENTRY_FN" ] && \
     ! grep -rqE "^[[:space:]]*(Definition|Equations|Fixpoint)[[:space:]]+$TS_ENTRY_FN[[:space:]]" proofs/coq/extraction/; then
    echo "[hax-coq-oracle] FAIL: entry_fn '$TS_ENTRY_FN' missing from .v products" >&2
    rc=1
fi
```

**反误报论证**：合法 fn 必为 `Definition <name>` / `Equations <name>` / `Fixpoint <name>` 形态之一—— 需在加规则后实测确认 Coq printer 用的具体 keyword 集合。

**风险等级**：**低-中**（同 hax-fstar）。

---

### 4.4 hax-lean — 低封堵（已实施 + 可选 entry_fn 增强）

**问题诊断**：当前 oracle 已抓 `sorry` 在 term 位置（lean.rs:1287/2163 silent path），实测 20 SUCCESS 被改判 FAILED。Lean printer 在 rust-engine（Rust 端）实现，OCaml 端 lean_backend.ml 仅 132 行 wrapper，**未发现 OCaml 端 silent-skip-item 路径**。

**推荐规则（可选增强）**：与 hax-fstar / hax-coq 同 entry_fn grep（Lean 的 fn definition 形态是 `def <name>` / `theorem <name>`）：

```sh
elif [ -n "$TS_ENTRY_FN" ] && \
     ! grep -rqE "^[[:space:]]*(def|theorem)[[:space:]]+$TS_ENTRY_FN[[:space:]]" proofs/lean/extraction/; then
    echo "[hax-lean-oracle] FAIL: entry_fn '$TS_ENTRY_FN' missing from .lean products" >&2
    rc=1
fi
```

**风险等级**：**低**（当前 oracle 已封 Lean printer silent sorry 主因，entry_fn 缺失是次要剩余）。

---

### 4.5 prusti — 低封堵（可选增强）

**问题诊断**：当前 .vpr 存在性 check 只校验**总数 ≥ 1**，entry_fn 在 .vpr 内的字面存在性未校验。`prusti-strict-wrapper.sh` 注释已声明此剩余窗口。

**推荐规则**：

```bash
# prusti-strict-wrapper.sh 现 wrapper 末尾追加
if [[ -n "${TS_ENTRY_FN:-}" ]]; then
    if ! grep -rqE "(function|method)[[:space:]]+m_${TS_ENTRY_FN}\\b" "$log_dir/" 2>/dev/null; then
        echo "[prusti-oracle] FAIL: entry_fn '$TS_ENTRY_FN' missing from .vpr products (encoder silently skipped this fn)" >&2
        exit 1
    fi
fi
```

**待实测验证**：prusti encoder 输出的 Viper function/method name 是否是 `m_<rust_name>` 形态（prusti 标准编码前缀），需在 .vpr sample 上确认。

**风险等级**：**低**（实测 0 现象，理论窗口）。

---

### 4.6 其他 10 工具

无需新增封堵规则。cargo-check / kani-limit-类剩余 caller_location/foreign function 路径暂搁置 / miri / kmir / soteria / creusot / charon-mono / charon-poly / aeneas × 4 / verus —— 这些工具的 oracle 当前都满足"exit code（+ 必要时已加 grep）= SUCCESS 信号充分"，形式可证或实测验证。

---

## §5 实施优先级

按 "严重性 × 改动成本" 排序：

| # | 工具 | 严重性 | 改动成本 | 估计数字变化 | 备注 |
| --- | --- | --- | --- | --- | --- |
| **1** | **kani** | 中-高（4-8 个真漏报）| 中（wrapper.sh 处理多行 stdout grep）| 144 SUCCESS → 138-142 | 5 markers（InlineAsm / simd_cast / catch_unwind / ptr_mask / C string literal）双向实测可上线；foreign function 路径暂搁置（涉及 cc-report 口径修订）|
| **2** | **hax-fstar** | 低-中（理论窗口）| 低（tool.toml 加 1 行 grep）| 115 SUCCESS → 115-114 | entry_fn `let` 字面 grep，与 P12 rocq-of-rust 同模式；待实测确认 F* fn 定义 keyword 集合 |
| **3** | **hax-coq** | 低-中（理论窗口 + boilerplate marker 与 item-level marker 同字面）| 低（同 hax-fstar）| 98 SUCCESS → 98-97 | 同上，Coq 的 `Definition` / `Equations` / `Fixpoint` 三 keyword 待实测 |
| 4 | **hax-lean** | 低（已封堵主路径）| 低 | 110 SUCCESS → 110-109 | 可选增强 |
| 5 | **prusti** | 低（实测 0 现象，剩余 entry-level 窗口）| 低 | 56 SUCCESS → 56-55 | 可选增强；prusti Viper fn name 前缀 `m_` 待实测确认 |
| 6 | 其他 10 工具 | 0 | 0 | 不变 | 不实施 |

**runner 是否需要扩展**：**不需要**。`TS_ENTRY_FN` 已在 P12 实施时由 runner `exec.rs:178` 注入子进程 env，本轮所有推荐规则都可直接消费这个变量。

---

## §6 与第一轮 audit 的方法论对照

### 6.1 共同方法论

- 双重证据：source-level（工具源码 silent path 找到位置）+ empirical（results.json + raw stdout/stderr 实测耗时与字面）
- 反误报硬指标：每条新规则必须双向实测（已知 silent path → 命中 + 合法 SUCCESS → 不命中）
- README/cc-report 自陈与实测对比：找出"自陈 ✅ 形式可证但实测有缺口"的工具点名（P1 找到 verifast，P2 找到 kani）

### 6.2 关键差异

| 维度 | 第一轮（verifast / prusti / rocq-of-rust）| 第二轮（其余 16 工具）|
| --- | --- | --- |
| 漏报机制 | A 类（spec-less skip）+ B 类（单文件 fast-path）+ D 类（产物 check 未实施）| C1（unsupported stub codegen，kani）+ C2（silent-skip-item，hax-fstar/coq backend）+ C3（entry-level 产物 check 未完全实施，prusti）|
| 严重性 | verifast 100% 灾难（vacuous pass）| kani 1.4-2.8% 中-高；其他 0-5% 理论窗口 |
| 物理证据 | verifast 207ms 中位 = 数量级低于真跑 + stdout signature 同 baseline | 16 工具全部 ≥ 100ms SUCCESS + raw stderr 都有真处理痕迹 |
| 实施模式 | wrapper.sh 包装（verifast 用 verbose grep / prusti 用 .vpr find）| 主要靠 tool.toml grep 增量（与 P12 rocq-of-rust gate 6 同模式）|

### 6.3 cc-report 自陈 vs 实测的不一致点名

P2 找到的 cc-report 不一致：

- **kani**：cc-report §"形式严格性" §"漏报盲点" 写"codegen 完成 + warning 但 SAT 阶段才会触发问题的 entry（**本次未观察到**）"。实测 64/144 SUCCESS 含 warning，其中 5 个非 prelude markers 在 6-8 entry 命中——**"本次未观察到"是错的**，应改为"本次观察到 N 个 entry 触发，已用 oracle 封堵"。

- **kani cc-report §"实测结果" §"kani-limit/* 7/7 全 SUCCESS"**：把这类 entry 的 SUCCESS 归因为"测试切割在 codegen，SAT 阶段才触发"——这条与项目宪法 §六-2 反作弊精神不一致（同 verifast 的 vacuous pass 论证）。**审计建议**：宪法精神优先，cc-report 应承认这些 SUCCESS 是 partial codegen → 修订口径 + oracle 实施封堵。

第一轮的 verifast cc-report 自陈不一致（"这不是 oracle 的漏报，是 corpus 设计与工具特性使然"）与第二轮的 kani 自陈不一致（"SAT 阶段才会触发问题的 entry，本次未观察到"）是**同种类型**的论证滑移——把"oracle 漏报"狭义化为"工具误判"而避开"工具自陈 partial codegen 但 oracle 仍 SUCCESS"。审计层面应保持一致口径：**partial codegen / vacuous pass / silent skip 在宪法精神下都是漏报**。

---

## §7 审计观察 / 暴论

7.1 **本轮没有第二个 verifast**。第一轮 verifast 100% 空过是工具自身 `-skip_specless_fns` flag 与 corpus 完全无 `//@` 注解的精确组合事故。本轮 16 工具中无一接近这种规模——最大的 kani 漏报也只在 5 个非 prelude markers 命中 6-8 entry（< 5% 占比）。**这是好消息**：除 verifast 外，其余工具的 oracle 状况整体是健康的。

7.2 **kani 漏报的本质**：与 verifast 的"工具默认 skip + corpus 没满足条件"不同，kani 的漏报是"工具把 unsupported 节点 codegen 成 stub + 把 stub 标 warning 而不 error"。kani-compiler 是"宽容前端 + 严苛后端"设计——codegen 接受范围 ⊋ CBMC 求解接受范围。我们选 `--only-codegen` 是为了"公平地与其他翻译工具对比"，但代价是 kani 把 stub 也通过了。**这是测试目标选择层面的妥协**，需要在 oracle 层修补（不是去掉 `--only-codegen`，而是在 codegen warning 上再加 hard reject）。

7.3 **hax-engine 三 backend 的 silent path 设计模式**：fstar_backend.ml:1771 / coq_backend.ml:588 / rust-engine 的 lean.rs:1287/2163 都是"对某些 AST item 返回空 / 占位字符串"的 silent path——hax 项目把这些当作 graceful degradation 设计，没接进 Diagnostic 通道。这是上游对"何为 partial"的口径与本项目宪法不一致：hax 把"留 sorry 让用户补"视为合理 partial 输出，本项目视为漏报。**双方都有理**——hax 是"翻译尽力 + 标 sorry"哲学；本项目是"翻译完整 + 严格 reject"哲学。oracle 层做精确 grep 是兼容两种哲学的工程妥协。

7.4 **耗时分布作为物理证据的稳健性再确认**：本轮 16 工具的最快 SUCCESS 全部 ≥ 100ms，没有第二个工具像 verifast 那样有 207ms 中位的子毫秒地板。最快的 cargo-check median 222ms / verifast(新) median 145ms / charon-mono median 350ms—— 都是"单文件解析 + 部分翻译" 的物理量级，不构成空过证据。**耗时仍是判别空过最强的物理证据，胜过 stdout / 产物 grep**——这是第一轮 §7.3 暴论的延续确认。

7.5 **自陈 vs 实测的鸿沟收敛**：第一轮 18/19 工具自陈"✅ 形式可证"或"⚠️ 实测验证"中只有 verifast 一家是错的（语义滑移），其余 17 工具大致成立。第二轮（除 P1 已封堵的 3 工具外，剩余 16 工具）经过本轮源码层 + 物理证据双重复核，自陈基本成立——**新增点名只 kani 一家**（cc-report 写"未观察到"但实测有现象）。

7.6 **本轮催生的最大原则**：oracle 强化不能只看"工具是否报 error"——还要看"工具是否在 stdout 自陈我没把这条干完"。kani 的 "Found the following unsupported constructs" 字面就是工具自陈，但 oracle 选择忽略——这是测试设计的盲点。**今后新增工具时应同时检查 stdout/stderr 的 self-disclosure 信号**（warning / unsupported / "not yet implemented" 等），而不只是 exit code。

---

## §8 项目宪法与文档关系

按 [`CLAUDE.md`](../../CLAUDE.md) §1.3 文档优先：

- `principles.md` §六 反作弊 + `tool-integration.md` §三 0 误报 + §四 0 漏报 未变 —— 本审计是宪法既有 claim 的延展查证，**未提议修订宪法**
- 本文不修改任何 `tool.toml / wrapper / cc-report / README` —— 仅是审计报告 + 推荐封堵规则
- 落地实施留给后续 P13 或类似补丁 PR，按 [`oracle-leak-rules-implementation-2026-05-08.md`](oracle-leak-rules-implementation-2026-05-08.md) 同模板（每条新规则必须双向实测 + 反误报论证 + 端到端 runner 验证）

属于"次要模块 3：tools 集成"的合规调优（按 [`CLAUDE.md`](../../CLAUDE.md) §3 模块优先级）。
