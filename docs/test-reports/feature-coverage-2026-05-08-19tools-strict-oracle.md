# 19 工具特性覆盖度综合报告（2026-05-08，新精神 + 严格 oracle）

> 本报告综合 19 份 cc-report（`deep-reports/cc-reports/<name>.md`），基于 `runs/run-1778226613-5282` 的全矩阵 run，按宪法（`docs/design/principles.md`）口径汇总每个工具在本次 run 上的实际作用与边界。

---

## 一、元数据

- **锚定 run**：`runs/run-1778226613-5282`
- **起止**：2026-05-08T07:50:13Z – 08:16:08Z UTC，1555 s wall（25 min 55 s）
- **Host**：`ssyramdeMacBook-Air.local` / macOS aarch64 / kernel 25.4.0 / Apple M5 / 24 576 MB / 10 cores
- **并发**：parallelism = 10
- **总 task**：19 工具 × 146 entries = **2774 task**
- **结果分布**：SUCCESS 1900 / FAILED 874 / **UNKNOWN 0** / **TIMEOUT 0**
- **runner 健康**：本次跑期间 0 panic / 0 internal timeout / 0 work 残留 / ≈ 98% 并发利用率 / 9 SIGABRT 全部正确归为 FAILED

### 19 工具版本快照（取自 `results.json`）

| tool | version |
|---|---|
| cargo-check | `cargo 1.95.0 (f2d3ce0bd 2026-03-21)` |
| miri | `miri 0.1.0 (cb40c25f6a 2026-05-04)` @ nightly |
| kani | `cargo-kani 0.67.0` |
| charon-mono | `charon 0.1.184` (toolchain `nightly-2026-02-07`) |
| charon-poly | `charon 0.1.184` (toolchain `nightly-2026-02-07`) |
| creusot | `cargo-creusot 0.11.0` · `nightly-2026-02-27` · Why3 1.8.2 · alt-ergo 2.6.2 · z3 4.15.4 · cvc4 1.8 · cvc5 1.3.1 |
| hax-coq / hax-fstar / hax-lean | hax `untagged-git-rev-30949eb870` (commit `30949eb87058895c24f963df90dd30ef11b0dc1a`) @ `nightly-2025-11-08` |
| aeneas-coq / aeneas-fstar / aeneas-hol4 / aeneas-lean | aeneas `a14083a6` + 自家 charon `0.1.184` (commit `ed22146b`) |
| prusti | Prusti 0.2.2 · commit `a0681ee` (2023-08-22) · `nightly-2023-08-15` · arch -x86_64 · JDK 17 |
| verus | `Verus 0.2026.05.03.8b81855` · profile release · platform `macos_aarch64` · toolchain `1.95.0-aarch64-apple-darwin` |
| verifast | `VeriFast 26.01 (released 2026-01-21)` · prebuilt macOS arm64 · provers Z3v4.5 / Redux |
| soteria | soteria-rust commit `3c212781` · Obol commit `ddea5ca5` · OCaml 5.4.0 |
| kmir | mir-semantics commit `84bea09` · stable-mir-json commit `62a239d7` · K Framework v7.1.282 · `nightly-2024-11-29` · kmir Python 0.3.181 |
| rocq-of-rust | `rocq_of_rust_cli 0.1.0` · commit `a8a76a4d` · `nightly-2024-12-07` |

### 时效性声明

按宪法 §三-3-1：本报告锚定 2026-05-08 上述具体工具版本组合与 146-entry corpus 的实测快照。**报告不构成项目对任何工具长期能力的承诺**。任一工具升级、上游 oracle 漂移、corpus 扩充都会让本快照解释力线性衰减。读者引用本报告任何数字时务必同时引用 run id (`run-1778226613-5282`) 与对应工具版本字符串。

### 与历史报告的关系

本报告是 2026-05 系列的第四份，前置三份保留：

| 报告 | run id | 状态 | 主要差异 |
|---|---|---|---|
| `feature-coverage-2026-05-07.md` | （早期）| 8 工具版 | 旧 corpus 子集 |
| `feature-coverage-2026-05-07-19tools.md` | `run-1778148197-53283` | 19 工具，harness 缺陷未修 | 缺 industrial、`*-limit` hyphen 误判 |
| `feature-coverage-2026-05-07-fixed-harness.md` | `run-1778155001-26161` | 19 工具，harness fix 后 | 75% 全矩阵通过率，但部分 oracle 仍偏松 |
| **本报告（2026-05-08）** | **`run-1778226613-5282`** | **19 工具，严格 oracle 后** | prusti NEW config / kmir K-stuck / hax sentinel-body / aeneas partial-file 等多处 oracle 收紧后的真实接受面 |

## 二、本次 run 在宪法精神演进上的位置

精神宪法在 2026-05-07 → 2026-05-08 之间强化了几条与 oracle 行为直接相关的约束：

1. **双轨 schema**（§三-A 形式定义下的"单文件轨 + 目录轨"）：`hirusttest.toml` 与 `.hirusttest/` 共存禁令、cargo 行为字节级一致判据。本次 run 不依赖该精神变化，但 runner 现有的 discover 已实现对应判定。
2. **严格 0 误报 + 下限诚实**（§三-3-2.b/c）：上限保证（不冤枉能力）和下限诚实（不高估能力）必须**对每工具明确声明形式严格性状态** —— (a) 是否能形式证明 0 误报，(b) 是否能形式证明 0 漏报，(c) 已知漏报盲点。本报告 §七 是该精神的横向汇总。
3. **不允许 partial**（§六-2）：SUCCESS = 工具完整完成它的工作单元；任何 partial / silent skip / 半翻译都必须 → FAILED。这条精神在本次 run 上对 5 类工具触发了 oracle 收紧（详见下表）。
4. **形式严格性声明义务**（§三-3-2.c 末段）：每个工具的 README 必须明确声明 (a)/(b)/(c) 三状态；19 份 cc-report 都按此口径汇报。

**对比 2026-05-07 fixed-harness 报告（`run-1778155001-26161`）的关键 oracle 改动**：

| 工具 | 旧 oracle 行为 | 新 oracle 行为 | 影响 |
|---|---|---|---|
| **prusti** | `PRUSTI_NO_VERIFY=true`（rustc 解析通过即 SUCCESS） | `PRUSTI_NO_VERIFY=false + DUMP_VIPER_PROGRAM=true + PRINT_HASH=true`（encoder 真跑） | 通过率 93% → **38%**：旧路径退化为 cargo-check 等价（违反 §六-4 反作弊），新路径暴露 prusti 真实 encoder 接受面 |
| **kmir** | 仅看 exit code | exit 0 **加** stdout grep `#EndProgram ~> .K` | 通过率 69% → **31%**：旧 oracle 把 K-stuck（K cell 卡在 unsupported terminator）记 SUCCESS，新 oracle 翻为 partial → FAILED |
| **hax-lean** | cargo hax exit 0 即 SUCCESS | exit 0 **且** 产物剥行注释后 grep 不命中 term-position sorry | 把 lean.rs sentinel-body silent path（`PatKind::Error` → `text!("sorry")`）抓回 FAILED，A 桶 20 条不再被冤判 SUCCESS |
| **hax-fstar / hax-coq** | exit 0 即 SUCCESS | exit 0 **且** 产物 grep 不命中 `failure ((` / `Rust_primitives.Hax.failure` / `please implement the method` | 防御性兜底；本 corpus 上 0 触发，但形式上封死了未来 silent path |
| **aeneas × 4** | exit 0 即 SUCCESS | wrapper 单一 exit 信号；exit 1 + `Generated the partial file (because of N errors)` 一律 FAILED | 把 aeneas mid-end 显式声明 partial 的 entry 抓回 FAILED |
| **soteria** | 含糊 | exit ≠ 0 一律 FAILED（含 bug detect / OCaml exception / 前端 crash） | bug-as-FAILED 与 §六-2 对齐：符号执行被 bug 中断 = 没完整跑完 = partial |
| **rocq-of-rust** | 仅看 exit code（永远 0） | 5 道门：exit 0 + `.v` 存在 + 无 0-byte + 至少一 > 200B + 产物 grep 不命中 explicit failure marker | 第 5 道门把工具自陈的 `(* Error … *)` 占位 FAILED |
| **verifast** | 仅看 exit code | exit 0；承认 vacuous pass 语义（spec 注解 0 命中时降级为"IR 接受"，README 强声明） | 数字不变，但 §七 显式标注语义降级 |

**核心精神**：测试支持率是真实能力的**严格上限**——读者拿到本报告的支持率数字时，应理解为"工具实际能力 ≤ 测得支持率"。若 oracle 偏松（partial 被记 SUCCESS），数字会虚高、误导读者高估工具能力；新精神下的 prusti / kmir / hax-lean 显著下降是 oracle 收紧的**正向信号**——下降的部分是真实的 partial 形态，不是工具能力倒退。

## 三、整体数字

### 总体

| 状态 | 数 | 占比 |
|---|---:|---:|
| SUCCESS | 1900 | **68.5%** |
| FAILED | 874 | 31.5% |
| UNKNOWN | 0 | 0% |
| TIMEOUT | 0 | 0% |

19 × 146 = 2774 task，0 runner-internal 故障；timeout 配置（120 - 900 s 不等）远未触达。

### 按通过率排序的 19 工具总表

时长字段仅作环境上下文，**非工具评分**。

| tool | n | S | F | rate | avg(ms) | p50 | p90 | max |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| **cargo-check** | 146 | 146 | 0 | **100%** | 2124 | 222 | 6913 | 26420 |
| **kani** | 146 | 144 | 2 | **98%** | 3871 | 1108 | 9952 | 40023 |
| **miri** | 146 | 142 | 4 | **97%** | 2800 | 721 | 7031 | 35727 |
| charon-poly | 146 | 139 | 7 | 95% | 2954 | 367 | 7232 | 34693 |
| charon-mono | 146 | 138 | 8 | 94% | 2296 | 347 | 6912 | 31899 |
| rocq-of-rust | 146 | 121 | 25 | 82% | 167 | 129 | 342 | 485 |
| verifast | 146 | 116 | 30 | 79% | 282 | 200 | 547 | 1479 |
| hax-fstar | 146 | 115 | 31 | 78% | 3315 | 1333 | 7785 | 33132 |
| hax-lean | 146 | 110 | 36 | 75% | 3646 | 1859 | 8957 | 26768 |
| soteria | 146 | 109 | 37 | 74% | 1846 | 1135 | 4008 | 12690 |
| creusot | 146 | 106 | 40 | 72% | 40047 | 39972 | 52222 | 64641 |
| hax-coq | 146 | 98 | 48 | 67% | 3735 | 1790 | 8004 | 26428 |
| aeneas-coq | 146 | 87 | 59 | 59% | 4075 | 1743 | 9947 | 35173 |
| aeneas-fstar | 146 | 87 | 59 | 59% | 3984 | 1906 | 8251 | 31561 |
| aeneas-lean | 146 | 87 | 59 | 59% | 3854 | 1834 | 8056 | 36310 |
| **prusti** | 146 | 56 | 90 | **38%** | 12593 | 10434 | 22761 | 76666 |
| aeneas-hol4 | 146 | 51 | 95 | 34% | 3989 | 1870 | 7805 | 32647 |
| **verus** | 146 | 51 | 95 | **34%** | 562 | 514 | 909 | 2312 |
| **kmir** | 146 | 46 | 100 | **31%** | 8196 | 7395 | 13353 | 105497 |

加粗的 4 个工具是 oracle 收紧后从虚高位置下移的——这些下降是宪法 §六-2 / §六-4 对 partial 与反作弊精神的正向落地，不是工具能力倒退。

## 四、工具类别与边界（5 类）

按宪法 §三-3-2.a"形式指标"切割工具类别。每类对应不同的"前端边界形式"，所以**不同类间的支持率绝对数字不可直接横比**——只能在类内或者按宪法精神横向对照"形式严格性"。

### 类 A：编译基线（1 工具）

**cargo-check**：调 stable rustc 完成 parse → macro 展开 → 类型 / borrow check → MIR build；exit 0 即接受。`cargo-check` 在本矩阵作为"健全性基准"——它 100% 通过反向证明 corpus 整体在 stable Rust 接受面之内，其他工具的 FAILED 都来自工具自身能力差异，不来自样例 Rust 错误。

**整体表现**：146/146 = 100%。

### 类 B：解释执行类（2 工具）

**miri / kmir**：在 MIR / SMIR 上做解释执行，没有"前端 / 后端"分界——工作内容 = 解释 + UB 检测（miri）或 K 语义化简（kmir）。SUCCESS = 解释完整跑完无中断（无 UB / unsupported / panic / K-stuck）。

**整体表现**：miri 97% / kmir 31%。差异来自实现成熟度：miri 是 Rust 官方组件，覆盖面与 nightly 同步；kmir 是 K Framework 上的研究系统，对 stable-mir-json schema 与 K rule 库的覆盖度低（README 自陈"56 个 K-stuck / 44 个 SMIR / cargo 链失败"）。

### 类 C：符号执行类（2 工具）

**soteria / verifast**：在 IR 上做有界符号执行 / separation logic 验证。soteria 用 OCaml 实现的原生 Tree Borrows + Obol 翻译；verifast 自带 rustc-style parser → VeriFast IR → Z3 / Redux。SUCCESS = 符号执行 / IR 构造完整完成，不允许中途 bug detect / verify-err / OCaml exception。

**整体表现**：soteria 74% / verifast 79%。verifast 数字附带强力**vacuous pass** 声明（本 corpus 0 spec 注解 + `-skip_specless_fns` 让 SUCCESS 实际语义为"IR 接受 + 无 spec 可证伪"）；soteria 用单文件直读模式，industrial / bigint / deps-complex 共 21 条因 unresolved import 死在 rustc 阶段。

### 类 D：翻译类（10 工具）

**charon × 2 / creusot / hax × 3 / aeneas × 4 / rocq-of-rust**：纯翻译工具，pipeline 终点是落盘的 IR 文件（LLBC / Coma / `.fst` / `.lean` / `.v` / `.sml`），不调用 SMT / 模型检查器。SUCCESS = "工具自陈完整翻译"（exit 0 且无 partial 标志）。

**整体表现**：

| 工具 | 通过率 | 备注 |
|---|---:|---|
| charon-poly | 95% | 翻译到 LLBC（保留泛型）|
| charon-mono | 94% | 翻译到 LLBC（单态化展开）|
| rocq-of-rust | 82% | 翻译到 Rocq monadic embedding，5 道门 oracle 抓 silent fallback |
| hax-fstar | 78% | hax engine + F\* printer，几乎不走 silent path |
| hax-lean | 75% | hax engine + Lean printer，sentinel-body grep oracle 抓回 20 条 silent partial |
| creusot | 72% | rustc + creusot translation passes → Coma；本 run 不调 `cargo creusot prove` |
| hax-coq | 67% | hax engine + Coq printer，reject phase 数量最多（9 个） |
| aeneas-coq / fstar / lean | 59% / 59% / 59% | 三 backend byte-identical，共享 mid-end |
| aeneas-hol4 | 34% | HOL4 backend `Option.get None` 在 trait decl 时硬天花板 |

翻译类是 19 工具中最多的类别（10/19）。它们的共同特点：**只筛"工具能不能吃下这段代码并产出预期形状的输出"**（宪法原则 B），不评翻译产物的语义忠实度（按宪法 §二排除）。

### 类 E：编码 + 后端 SMT 类（4 工具）

**kani / prusti / verus / creusot**：把 Rust 编码到中间表示（GotoC / Viper VIR / VIR-AIR / Coma），后端调 SMT solver 做求解。本测试**精确切在编码完成处**（kani `--only-codegen` / prusti `PRUSTI_PRINT_HASH=true` / verus `--no-verify` / creusot 默认无 subcommand）。

**整体表现**：

| 工具 | 通过率 | 切割点 |
|---|---:|---|
| kani | 98% | MIR → GotoC codegen 完成（不调 CBMC SAT/SMT） |
| creusot | 72% | rustc + creusot pass 完成 → `.coma` 落盘（不调 Why3 / SMT solver）|
| prusti | 38% | encoder 完整跑完 + Viper `.vpr` 落盘（不调 Silicon JVM verifier / Z3） |
| verus | 34% | VIR 构造完成（不调 AIR / Z3） |

注：creusot 同时属类 D（翻译 → `.coma`）与类 E（后端 SMT），按当前测试切割归在类 E 的"前端 = encode 完成"语义。

**核心观察**：编码 + 后端 SMT 类的前端切割比纯翻译类更严苛——不仅要"翻译完成"，还要"被工具的 type / mode / encoder 全部接受"。prusti 38% 与 verus 35% 反映 SMT-based encoder 在不加任何 user spec 的 plain Rust 上接受面**确实狭窄**——这不是工具缺陷，是其设计意图：encoder 拒收 raw pointer / closure 捕获 mut / async fn / 部分迭代器等构造，因为这些构造无法直接表达为 Viper / VIR 的可证伪 spec。

## 五、19 工具实际作用与效果

### 1. cargo-check — 100% (146/146) — 健全性基准

**定位**：调 stable rustc 完成完整前端，pipeline 终点 = MIR build 完成、未进入 codegen。**形式严格性**：✅ 0 误报形式可证 / ✅ 0 漏报形式可证（rustc 单一 exit signal）。**漏报盲点**：无。

**本 run 揭示**：作为基线 100% 通过，反向验证 corpus 整体在 stable Rust 接受面之内。所有 41 个 feature 类目（含 7 个 `*-limit` + 6 个 industrial vendor crate）全部 SUCCESS。median 222 ms / max 26 s（dep-heavy entry 拖慢，与 rustc 前端能力无关）。

**作用**：反向过滤"corpus 自身合法性"问题——任一其他工具 FAILED 而 cargo-check 也 FAILED，问题在样例不在工具。本 run 因 cargo-check 0 失败，所有其他工具的 FAILED 都在工具自身能力差异维度可读。

### 2. kani — 98% (144/146) — MIR → GotoC codegen 边界

**定位**：AWS 出品的 Rust → CBMC bounded model checker。本 run 通过 `--only-codegen` 切在 MIR → GotoC 翻译完成处，不调 CBMC SAT/SMT。**形式严格性**：✅ 0 误报形式可证（exit 0 ⇔ codegen 无错误）/ ⚠️ 0 漏报实测验证（理论上 unsupported warning 可与 codegen 完成共存，本 corpus 未观察）。

**本 run 揭示**：2 个 FAILED 都在 `industrial/x509-parser/*` —— vendor crate `lib.rs:122` 的 `#![deny(unstable_features, unused_qualifications)]` 与 kani 注入的 `#![feature(register_tool)]` 交互冲突，错误发生在 lint 阶段、未到 MIR → GotoC 翻译。`kani-limit/*` 7/7 全 SUCCESS——这些 entry 故意触发 Kani 自声明的"不支持"特性，但 unsupported 性质本应在 CBMC 求解阶段才显现，`--only-codegen` 路径上 codegen 全过。

### 3. miri — 97% (142/146) — MIR 解释执行

**定位**：Rust 官方维护的 MIR 解释器，工作 = 解释 + UB 检测。**形式严格性**：✅ 0 误报形式可证 / ✅ 0 漏报形式可证（任何 UB / unsupported / panic 必触发 exit ≠ 0）。**漏报盲点**：无。

**本 run 揭示**：4 个 FAILED 全部预期 corpus 触发——3 个为 miri 自声明 unsupported（inline-asm / FFI on macOS / network in isolation mode），1 个为真实 UB 检出（`MaybeUninit::<u8>::uninit().assume_init()`）。`industrial/*` 6/6 全过（含 RSA pkcs1v15 加密 + sha2 incremental + x509-parser ASN.1 DER 解析）。`unsafe-ptr/raw-ptr-const` 走 `warning: integer-to-pointer cast` 不当 UB（permissive provenance 默认）→ SUCCESS。

### 4. charon-poly — 95% (139/146) — Rust → LLBC（保留泛型）

**定位**：AeneasVerif 出品的纯翻译工具，多态模式输出带类型变量的 LLBC，为 Aeneas 等后端提供统一前端。**形式严格性**：✅ 0 误报 / ✅ 0 漏报形式可证（`--abort-on-error` + `register_error!` panic 路径封死 silent skip）。

**本 run 揭示**：7 个 FAILED 分 3 桶——4 个 charon 自声明 unsupported（Coroutine / InlineAsm / Cast constant）、2 个 thread-local references（`thread_local!` 宏展开内的 std TLS 实现）、1 个 cyclic trait bound 触发 rustc stack overflow (SIGABRT 转 exit 101)。`bigint/*` 8/8 + `deps-complex/*` 7/7 + `industrial/*` 6/6 全过——能接得住完整 cargo 依赖图。

### 5. charon-mono — 94% (138/146) — Rust → LLBC（单态化展开）

**定位**：与 charon-poly 共享同一 binary，多 `--monomorphize` flag——翻译时把所有泛型实例展开。**形式严格性**：与 poly 相同。

**本 run 揭示**：8 个 FAILED 中 4 个与 poly 共享（A 桶 unsupported + 1 stack overflow），3 个为 mono 模式独有的 vtable drop preshim panic（`Could not determine method index for drop in vtable` 在 `Box<dyn Display>` / `Box<dyn Any>` 上触发）。两 mode 在 FAILED 集合上**不互含**——poly 与 mono 在 thread-local 与 vtable drop preshim 上呈对称差，对应"前端边界 ≠ 后端边界"的细分实例。

### 6. rocq-of-rust — 82% (121/146) — Rust → Rocq syntactic transcoder

**定位**：Rust → Rocq（原 Coq）的轻量 syntactic 翻译器，每个 fn 翻成 `Definition + Global Instance ... Admitted.`。**形式严格性**：⚠️ 0 误报实测验证 / ⚠️ 0 漏报实测验证（5 道门 oracle：exit 0 + `.v` 存在 + 无 0-byte + 至少一 > 200B + 产物 grep 不命中 `Error / Unexpected / Please report! / thir failed to compile / Unimplemented`）。

**本 run 揭示**：25 个 FAILED 中 24 个落在 input 流水线（21 个外部 crate unresolved + 3 个 edition / feature gate），1 个落在 5 道门第 5 门（`repr/union/repr_union` 的 explicit `(* Error Variant *)` marker）。121 个 SUCCESS 上 grep 0 命中——rocq-of-rust 在它能接受 input 的 entry 上**给出完整翻译，没出现工具自己标记的占位**。该工具的 syntactic 通路覆盖面广（GAT、HRTB、闭包返回、`unsafe` 指针、Drop、Arc/Mutex、trait object 全 SUCCESS），原因是统一翻成 `M.closure / M.borrow / Pointer.Kind.MutRef` 等节点，**不在翻译阶段做语义筛**。

### 7. verifast — 79% (116/146) — Rust → VeriFast IR + separation logic

**定位**：自带 rustc-style parser，直接读 `src/lib.rs`（不走 cargo），构造 VeriFast IR + Z3/Redux 后端。本 run 加 `-skip_specless_fns` 跳过无 spec 函数。**形式严格性**：✅ 0 误报 / ✅ 0 漏报形式可证。**关键限定**：本 corpus 0 spec 注解 + `-skip_specless_fns` 三件套 → SUCCESS 实际语义降级为 **vacuous pass**（"rustc-verifast 接受源码进入 IR + 无 spec 可证伪"），**不**等于"symex 完成"。

**本 run 揭示**：30 个 FAILED 分 5 桶——21 个不解析 cargo 依赖（bigint / deps-complex / industrial 21 个 unresolved import）、3 个 float 在复合类型中（`enum Variant(f64)` / `union { f: f32 }` / `Value(f64)` 变体）、2 个 const generics 结构定义、1 个 `&[T]` 字段 share predicate 自动生成失败、3 个 edition 边界（async fn / let-chains in Rust 2015）。子毫秒级响应（median 200 ms）—— prover 几乎不被使用。

### 8. hax-fstar — 78% (115/146) — Rust → F\* via hax-engine

**定位**：hax-engine（OCaml）+ F\* printer。三 hax backend 共享同一 OCaml binary，差异仅在 printer 阶段。F\* backend 在 hax 中较成熟（README 声明），phase pipeline 上 F\* 走的 reject phase 多（含 `reject_RawOrMutPointer` / `reject_ArbitraryLhs` / `reject_TraitItemDefault` 等），**几乎不走 silent path**。**形式严格性**：⚠️ 0 误报实测验证 / ⚠️ 0 漏报实测验证（grep `Rust_primitives.Hax.failure` 防御性兜底，本 corpus 0 触发）。

**本 run 揭示**：31 个 FAILED 全部走 cargo hax exit 1（hax engine 主动 emit `[HAX####]` Diagnostic）。最高频信号 `[HAX0008] reject_RawOrMutPointer`（22 次）、`[HAX0003]/[HAX0006]/[HAX0010]` mut-ref 系（11 桶）、`[HAX0001] FunctionalizeLoops`（3）、`[HAX0001] AST import`（4）等。

### 9. hax-lean — 75% (110/146) — Rust → Lean 4 via hax-engine

**定位**：hax-engine + Lean printer。Lean printer 上 hax upstream 标记为 active development，部分 reject phase 走 sentinel `text!("sorry")` 路径而非 emit Diagnostic——这是 hax-lean **特有**的 silent partial。**形式严格性**：⚠️ 0 误报实测验证 / ⚠️ 0 漏报实测验证（产物 grep 抓 term-position sorry，剥行注释后 `:= sorry / pure sorry / mk sorry / , sorry / sorry,)`）。

**本 run 揭示**：36 个 FAILED 中 **20 个**通过产物 grep oracle 抓回（A 桶 silent sorry path），其余 16 个走官方 Diagnostic（B 桶 `[HAX0001]` Lean Printer todo + C 桶 Name rendering）。20 个 silent sorry 不抓产物的旧版本会齐刷刷 SUCCESS，把通过率从 75% 拉到约 89%。这正是宪法 §六-2"不允许 partial"的具体落地。

### 10. soteria — 74% (109/146) — 有界符号执行 + 原生 Tree Borrows

**定位**：OCaml 实现的符号执行 + Tree Borrows，pipeline = Obol 翻译 (Rust → ULLBC/LLBC) + soteria-rust SE。本 run `tool.toml` 让 soteria 完整编译 + 符号执行（README 自陈无 dry-run flag）。**形式严格性**：✅ 0 误报 / ✅ 0 漏报形式可证（exit 0 = 编译完成 + SE 完整跑完无 bug；exit 1/2/3 完整覆盖 bug detect / 内部 crash / 前端 crash）。

**本 run 揭示**：37 个 FAILED 分 6 类——A 21 个 rustc 编译前置（单文件不读 Cargo.toml）、B 1 edition、C 4 Obol 翻译层不支持（inline-asm / Coroutine / Missing Global）、D 7 内核未实现的 intrinsic / extern（atomic_xsub / float transmute / extern abs / 多 socket）、E 2 OCaml exception（thread-mutex 系）、F 2 真符号执行检出（`hashmap` aarch64 SIMD false-positive + `kani-limit/uninit-memory` 真 UB）。`aeneas-limit/*` 8/8 + `prusti-limit/*` 8/8 + `int-width/*` 14/14 + `unsafe-adv/*` + `unsafe-ptr/*` 全过——Tree Borrows 对这些构造接受度高。

### 11. creusot — 72% (106/146) — Rust → Coma (Why3 IR)

**定位**：creusot-rustc（rustc + creusot translation passes）→ Why3 Coma IR；本 run 不调 `cargo creusot prove`，停在 Coma 翻译完成。**形式严格性**：✅ 0 误报 / ✅ 0 漏报形式可证（creusot 用 `crash_and_error / span_err / dcx().span_err` 把所有 unsupported 升级为 rustc error，无 silent path）。

**本 run 揭示**：40 个 FAILED 全 exit=101，分 7 桶——A 7 显式 forbidden dyn type（含用户 trait `Greeter` 与 `dyn Display/Any/Fn`）、B 6 Unsupported pointer cast / coercion（`IntToFloat` 在 4 条 / `PointerCoercion(ReifyFnPointer)` 等）、C 1 raw pointer dereference（错误信息主动给出替代方案 `creusot_std::ghost::perm::Perm<*const T>`）、D 10 Unsupported constant value/expression（`industrial/*` 4 条 byte-string `b"..."` + `deps-complex/*` 5 条 serde-derive 的 `"name"` 字面）、E 4 creusot-rustc 内部 panic（union / async / inline asm 的 `unreachable`）、F 8 spec 层未覆盖（`DeepModel` / `IteratorSpec` / `NaN is not yet supported`）、G 4 其他显式拒。中位数 40 s——`cargo-creusot 0.11.0` 慢主要是每次 entry 重 build creusot-std。

### 12. hax-coq — 67% (98/146) — Rust → Coq via hax-engine

**定位**：hax-engine + Coq printer。Coq backend 在三 hax backend 中**reject phase 数量最多**（9 个：`reject_Unsafe / reject_RawOrMutPointer / reject_Arbitrary_lhs / reject_Dyn / reject_TraitItemDefault / AndMutDefsite / DirectAndMut / LocalMutation / CfIntoMonads`）。**形式严格性**：⚠️ 0 误报实测验证（hax-coq **不翻译 Rust doc comment**，user code 极难写出 silent marker `failure ((` / `please implement the method`）/ ⚠️ 0 漏报实测验证（本 corpus oracle 对 silent path 0 次触发改判，所有 FAILED 走 `[HAX####]` Diagnostic）。

**本 run 揭示**：48 个 FAILED 分 12 桶——`reject_Dyn` 7 / `reject_RawOrMutPointer` 5 / `reject_Unsafe` 8 / mut-ref 系 9 / FunctionalizeLoops 5 / AST import 4 / Coq printer Unreachable on serde-derive 4 / 其他单点。Coq backend 在 `deps-complex/*` 上 4/7（vs hax-fstar / hax-lean 都 7/7）的差距完全由 Coq printer 在 serde derive 形态上的 `Unreachable` 失败导致。

### 13. aeneas-coq — 59% (87/146) — Rust → Coq via aeneas

**定位**：两段 pipeline：stage 1 charon → LLBC + stage 2 aeneas (-backend coq) → `.v`。`aeneas-coq-wrapper.sh` 把两段打包成单一 tool，`set -euo pipefail`。**形式严格性**：✅ 0 误报 / ✅ 0 漏报形式可证（aeneas 用 `craise` 单一信号通路把所有 unsupported 推 `error_list`；`Main.ml` 末尾 `if has_errors then exit 1`；exit 0 ⇔ error_list 空）。

**本 run 揭示**：59 个 FAILED 分 3 桶——A 1 charon stack overflow、B 49 aeneas mid-end / backend "partial file" (`Generated the partial file (because of N errors)`)、C 9 OCaml uncaught exception。B 桶细分：14 `Improperly typed constant value`（整 `float/*` 10 条 + 含浮点 const 项）、8 `Invalid inputs for binop/unop`、12 多种 `Unsupported / Not yet supported` 显式 todo（nested borrows / shallow-init-box / arrow types / 等）、14 `Internal error / Region ids should not be visited directly`（含 `industrial/rsa/*` 2 条）、4 charon → LLBC IR 不完整。

### 14. aeneas-fstar — 59% (87/146) — Rust → F\* via aeneas

**定位**：与 aeneas-coq 完全相同 pipeline，只换 stage 2 `-backend fstar`。**形式严格性**：与 aeneas-coq 相同。

**本 run 揭示**：59 个 FAILED **逐 entry 与 aeneas-coq / aeneas-lean exit code byte-identical**（diff = 0）。F\* printer 与 Coq / Lean printer 在本矩阵 entry 上无差异——三 backend 的全部失败发生在 charon stage 1（A 桶 1）或 aeneas mid-end（B/C 桶 58）；`Extract.ml` printer 阶段三 backend 共享 mid-end 输出，对本 corpus 无分化。

### 15. aeneas-lean — 59% (87/146) — Rust → Lean 4 via aeneas

**定位**：与 aeneas-coq / fstar 完全同 pipeline，stage 2 `-backend lean`。**形式严格性**：与其他 aeneas backend 相同。

**本 run 揭示**：与 aeneas-coq / aeneas-fstar byte-identical。三 backend 数字一致是 mid-end 共享的预期结果，不是巧合。

### 16. prusti — 38% (56/146) — Rust → Viper via Prusti encoder

**定位**：Prusti = `cargo-prusti` → `prusti-rustc` (rustc + prusti-driver plugin) → MIR → Viper VIR → `.vpr` 文件。本 run `PRUSTI_NO_VERIFY=false + DUMP_VIPER_PROGRAM=true + PRINT_HASH=true` 切在 encoder 完成、Silicon JVM verifier 启动前。**形式严格性**：✅ 0 误报 / ✅ 0 漏报形式可证。

**本 run 揭示**：90 个 FAILED 分 5 桶——A 68 `[Prusti: unsupported feature]` graceful 拒绝（最高频短语：`iterators are not fully supported yet` 14、`cast statements that create loans are not supported` 10、`access to reference-typed fields is not supported` 9、`higher-ranked lifetimes and types are not supported` 5 等）、B 8 `[Prusti: internal error]` encoder fold-unfold permission、C 7 cargo manifest edition = 2024 拒绝（prusti 锁 `nightly-2023-08-15` 的 cargo 不识别新 edition）、D 5 compiler ICE panic、E 2 `error[E0658]: unstable library feature`。

**与上一份 cc-report (OLD config 67%) 的关键差异**：旧 `PRUSTI_NO_VERIFY=true` 不跑 encoder——SUCCESS = "rustc + prusti-contracts proc-macro 通过"，与 cargo-check 等价（违反 §六-4 反作弊）；新 38% 是 prusti **真实 encoder 接受率**。28 个百分点的下降是 oracle 收紧的正向信号，不是工具能力倒退。

### 17. aeneas-hol4 — 34% (51/146) — Rust → HOL4 via aeneas

**定位**：与 aeneas-coq / fstar / lean 共享 mid-end，stage 2 `-backend hol4` → `.sml`。**形式严格性**：✅ 0 误报 / ✅ 0 漏报形式可证。

**本 run 揭示**：95 个 FAILED 比其他 3 backend 多 36 条——多出的 36 条全部发生在 stage 2 的 HOL4-specific extract / pretty-print 阶段：

- C1（52 条 HOL4-only 主因）：`Aeneas__Extract.extract_trait_decl` 在 `Option.get None` 触发 `Invalid_argument "option is None"`——`ExtractBase.ml:1412 type_decl_kind_to_qualif` 在 HOL4 backend 分支对 trait decl 始终返回 `None`，**LLBC 含 ≥1 个 trait declaration 即抛 panic**（含 `FnOnce / From / Iterator / Display / 用户 trait` 等）。这是 aeneas-hol4 upstream 的硬天花板。
- 其余 4 条 `Constant generics are not supported yet when generating code for HOL4`（HOL4 backend 自身写死的 guard，同 4 条在 fstar/coq/lean 上 SUCCESS）。

C1 桶占 95 个 FAILED 的 55%——是 aeneas-hol4 通过率显著低于其他 3 个 backend 的**单一最大根因**。

### 18. verus — 34% (51/146) — Rust → VIR via verus-driver

**定位**：verus binary 以 `--no-verify` 切在 VIR 构造完成（`--no-verify` 同时切 AIR + Z3，最深前端到 VIR）。harness `mod __ts_inner` 在 `verus! { }` 块内（不带 `#[verifier::external]`），让 inner items 被 verus 前端逐项检查（满足 §六-4 反作弊）。**形式严格性**：✅ 0 误报 / ✅ 0 漏报形式可证。

**本 run 揭示**：95 个 FAILED 分 8 桶——A 29 vstd 没给 std lib API 挂 spec（数值方法 / 浮点 / 内存原语 / 容器迭代器 / 字符串 / panic / hint / 原子等）、B 19 verus 语言子集还不支持的 Rust 构造（`bitwise AND/OR for bools` / `internal item statements` / `dyn with more than one trait` / `function pointer types` / `inline-asm` / `dereferencing a pointer` / `deref_mut for RefMut not yet supported` 等）、C 19 输入流水线层（外部 crate `unresolved import`）、D 12 verus 内部 panic (`generic_args.rs:54:14 index out of bounds`)、E 7 verus-driver MIR 构造 panic、F 2 closure 捕获 `&mut`、G 1 edition 边界、其他杂项 6。

**对比 prusti 38%**：两者都是 SMT-based encoder，verus 35% 反映 vstd spec 边界 + verus 语言子集的真实接受面狭窄——这不是工具缺陷，是其设计意图（plain Rust 不写 verus spec 时 encoder 拒收 raw pointer / dyn / function pointer 等无法直接表达为可证伪 spec 的构造）。

### 19. kmir — 31% (46/146) — Stable MIR → K Framework semantics

**定位**：三段链：(a) `cargo build with RUSTC=stable-mir-json` 把 entry 编为 SMIR JSON；(b) Python 端把 JSON 解析为 K term；(c) K LLVM backend 在 MIR 操作语义里解释执行。**形式严格性**：✅ 0 误报 / ✅ 0 漏报形式可证（exit 0 + stdout grep `#EndProgram ~> .K`，K interpreter 完整跑到终止；任何 K-stuck → FAILED）。

**本 run 揭示**：100 个 FAILED 分 3 类——A 56 K-stuck (exit=2，K cell 卡在 unsupported terminator，含 closure aggregation / inline-asm terminator / atomic / float intrinsic / GAT / closure mut capture)、B 35 SMIR JSON 解析失败（`json.decoder.JSONDecodeError`，stable-mir-json 没产合法 JSON—典型在 dyn trait / Box<dyn> / Arc<[T]> / std::collections / GAT / nested match guard / 第三方 crate）、C 9 cargo build 失败（含 `bigint/*` 8 条 unresolved external crate + 1 edition）。

**与上一份 cc-report (旧 oracle 70%) 的关键差异**：旧 oracle 仅看 exit code，K-stuck 时 CLI 仍 exit 0 → 102/146 假阳性 SUCCESS。新 oracle grep `#EndProgram ~> .K` 抓 K-stuck，把 52 个虚高翻为 FAILED → 真实 31%。新 oracle 把 kmir 与 miri 对齐：`charon-limit/inline-asm/nop_via_asm` 在 miri 与 kmir 上现在都 FAILED（前者通过显式 unsupported feature 错误，后者通过 K-stuck）。

## 六、跨工具关键发现

### 1. 数字一致性

- **aeneas-coq / aeneas-fstar / aeneas-lean 三 backend byte-identical**（87/146 各，逐 entry exit code 完全一致）：三 backend 共享 charon stage 1 + aeneas mid-end，差异只在 `Extract.ml` printer 分支。本矩阵 entry 集合下，F\* / Coq / Lean printer 在 mid-end 输出上无可观察差异；aeneas 项目以 F\* 为最早 backend，三 printer 在常见 case 上对齐。
- **hax × 3 通过率梯度**（fstar 78% > lean 75% > coq 67%）：三 backend 共享 hax-engine OCaml binary，差异只在 printer 阶段。F\* printer 走 reject phase 多但几乎不走 silent path；Lean printer 部分 silent sorry path 被 oracle 抓回（相当于把约 14 个百分点的虚高扣回）；Coq printer 的 reject phase 阵列最完整、最严格——但 `deps-complex/*` 上 Coq printer 在 serde derive 形态触发 `Unreachable` 失败 4 条（fstar / lean 同样 entry 全过）。

### 2. 可解释的差异

- **aeneas-hol4 vs 其他 3 backend 的 35 pp 落差**（34% vs 59%）：单点根因明确——`extract_trait_decl` 在 HOL4 backend 上的 `Option.get None` panic（55% 的 FAILED 来自此 panic），LLBC 含任意 trait declaration 即触发，与 entry 是否真"用"该 trait 无关。这不是 aeneas mid-end 问题，是 HOL4-specific extract 的 upstream 实现限制。
- **charon-mono vs charon-poly 的 4 条对称差**：thread-local references 在 poly 上 FAILED（进入 std TLS polymorphic 路径触发 unsupported），mono 上 SUCCESS（specialized 实例避开）；vtable drop preshim 在 mono 上 FAILED（展开 `Box<dyn Display>` / `Box<dyn Any>` 时 panic），poly 上 SUCCESS（不展开 vtable）。两 mode 在 FAILED 集合上**不互含**——同工具的两个 IR 形式呈现真实接受面分化。

### 3. 真前端能力差异

- **prusti 38% vs verus 34%**：两者都是 SMT-based encoder，本 run 都精确切在编码完成处。两者下降到 30+% 真实反映了它们的 encoder 接受面——plain Rust 不加 user spec 时，Viper / VIR encoder 拒收 raw pointer / closure 捕获 mut / async / function pointer / dyn with multiple traits / 部分迭代器 等构造，因为这些构造无法直接表达为可证伪 spec。这与 hax-fstar 78% 的差异并非工具好坏，而是 SMT 编码 vs syntactic 翻译的设计区别。
- **rocq-of-rust 82% 高于 hax-coq 67%**：两者都翻到 Rocq，但 rocq-of-rust 是 syntactic transcoder（`M.closure / M.borrow / Pointer.Kind.MutRef` 字符串标签，不在翻译阶段做语义筛），hax-coq 是 phase-based engine（9 个 reject phase 在前端检查 unsafe / raw-ptr / dyn / mut-ref / arbitrary-lhs / 等）。这一对比在矩阵上揭示"翻译深浅"对接受面的影响——但按宪法 §六-3"不区分翻译深浅"原则，**两者都算"在册接受"，本测试不加权**。

### 4. corpus 偏向声明

- **verifast vacuous pass**：corpus 全集 grep `//@ (req|ens|inv|pred)` 0 命中——`-skip_specless_fns` 让 SUCCESS 实际语义为"IR 接受 + 无 spec 可证伪"，**不**等于 verification 实际发生。本 corpus 中 116 个 SUCCESS 的 104 个报告同一 baseline `37 statements verified`（来自 verifast 自身 prelude）。读 verifast 数字时务必引用此降级。
- **kani-limit / miri-limit / charon-limit 等 `*-limit/*` 类目**：这些 entry 故意触发对应工具自声明的"不支持"特性，但能否被翻译边界拒收**依工具内部 pipeline 设计而异**——本 run kani-limit 在 kani 上 7/7 全 SUCCESS（`--only-codegen` 在 codegen 阶段全过，"unsupported" 性质本应在 CBMC 求解阶段才显现）。这是当前测试切面与 corpus 设计的一种"前后端边界不重合"现象。
- **industrial 6 entry 在 9 个工具上全 FAILED**：aeneas-hol4 / verifast / verus / kmir / prusti / soteria / rocq-of-rust——前 5 个因单文件 input mode 不读 Cargo.toml 或 cargo / nightly 不兼容；prusti 锁的 `nightly-2023-08-15` cargo 不识别 industrial vendor 的 edition 2024。industrial 全 SUCCESS 的工具是 cargo-check / charon × 2 / hax × 3 / miri 7 个；creusot 2/6（接得住 vendor crate 编译，失败在 byte-string / serde derive）；kani 4/6（`x509-parser` 的 `deny(unstable_features)` × `register_tool` 注入交互）。

## 七、形式严格性合规度

按宪法 §三-3-2.b/c：每工具 README 必须明确声明 (a) 0 误报、(b) 0 漏报、(c) 已知漏报盲点。

### 0 误报合规率：19/19 ✅

19 份 cc-report 都明确声明 0 误报状态：

| 形式可证 0 误报（13 工具） | 实测验证 0 误报（6 工具）|
|---|---|
| cargo-check / kani / miri / charon-poly / charon-mono / creusot / aeneas-coq / aeneas-fstar / aeneas-lean / aeneas-hol4 / prusti / verus / soteria | rocq-of-rust / verifast / hax-fstar / hax-lean / hax-coq / kmir |

注：verifast 的"0 误报"是关于 oracle 形式语义的，不掩盖 vacuous pass 的语义降级——SUCCESS 的语义边界已在 README + cc-report 显式声明。kmir 的 0 误报是关于 K-stuck grep 的形式可证（`#EndProgram ~> .K` 是 K Framework 终止 signature）。

### 0 漏报状态

| 形式可证 0 漏报（14 工具） | 实测验证 0 漏报（5 工具）|
|---|---|
| cargo-check / miri / charon-poly / charon-mono / creusot / aeneas-coq / aeneas-fstar / aeneas-lean / aeneas-hol4 / prusti / verus / soteria / verifast / kmir | **kani / hax-fstar / hax-lean / hax-coq / rocq-of-rust** |

5 个 ⚠️ 实测验证（不可形式证明）的工具：

- **kani**：unsupported warning 与 codegen 完成可共存（理论上 `--only-codegen` 路径上 warning 不影响 exit 0），本 corpus 上 SUCCESS entry 未观察到此 warning。
- **hax-fstar**：grep `Rust_primitives.Hax.failure` / `failure ((` 是防御性兜底，本 corpus 0 触发，但不可形式排除未来上游引入新 silent path。
- **hax-lean**：grep 抓 term-position sorry（剥行注释后），实测对用户合法 `let sorry: i32 = 5` 与 doc comment `sorry` 字面字符串都不触发。
- **hax-coq**：grep `failure ((` / `please implement the method`，本 corpus 0 触发；hax-coq 不翻译 Rust doc comment。
- **rocq-of-rust**：5 道门 grep 抓 explicit failure marker（`Error / Unexpected / Please report! / thir failed to compile / Unimplemented`）。

### 已知漏报盲点清单

按 cc-report 自陈：

| 工具 | 漏报盲点 |
|---|---|
| cargo-check / miri / charon-poly / charon-mono / creusot / aeneas × 4 / prusti / verus / soteria / verifast / kmir | 无 |
| kani | codegen 完成 + unsupported warning 但 SAT 阶段才会触发的 entry（本次未观察到）|
| hax-fstar | hax engine 完全 skip item（实测 0 现象）；上游引入新 silent path 的可能 |
| hax-lean | hax engine 完全 skip item；上游可能引入新 silent path 而 grep 滞后 |
| hax-coq | hax engine 完全 skip item（实测 0 现象）；上游引入新 silent path 的可能 |
| rocq-of-rust | 上游引入新 silent fallback 路径不带已知 markers（实测 0 现象） |

注：hax × 3 与 rocq-of-rust 共享一类盲点—— "hax engine 完全 skip item"。这类 silent partial 不通过 grep 检测，需要对应性脚本（输入 fn 列表 vs 输出 declaration 列表 diff）暴露。本 run 在所有 19 工具上**实测 0 现象**——但这是经验事实，不是形式承诺。

## 八、与项目目标的对齐

### 项目目标（宪法 §二）

**项目目标 = 构建 Rust 工具特性覆盖率测试框架**——核心模块是 runner（测试运行与结果分析框架）+ examples（样例库）。tools / 实测报告是**次要模块**，定位为框架的应用展示，**不作为项目核心目标**。

### 本报告的位阶

1. **本报告是次要模块产出**——不是核心模块的承诺。19 个工具的具体测试数据是框架"用得起来"的展示，但项目长期承诺的是框架本身（runner 0 panic / 0 timeout / 0 work 残留 / SCHEMA 双轨等）。
2. **不构成对任何工具能力的长期承诺**——按宪法 §三-3-1：所有数字锚定 2026-05-08 上述具体工具版本组合 + 146-entry corpus。任一工具升级、上游 oracle 漂移、corpus 扩充都让本快照解释力线性衰减。
3. **不评工具语义忠实度 / 后端求解能力**——按宪法 §二排除（"测必要条件，非语义对错"）：本报告只问"工具能不能吃下这段代码并产出预期形状的输出"，不问 LLBC / `.fst` / `.lean` / `.v` / `.coma` / `.vpr` / VIR 产物的语义忠实度，不问下游 verifier / Coq / Lean / F\* type-check 通过与否。
4. **不区分翻译深浅**——按宪法 §六-3：syntactic 搬运（rocq-of-rust）/ 深 MIR 翻译（aeneas）/ verifier dialect 接受（prusti / verus）一视同仁，本报告不做工具间能力排序；§五的工具罗列按通过率排序仅为读者方便定位，不带价值评判。

### 时效性快照

本报告锚定：

- run id: `run-1778226613-5282`
- 时间：2026-05-08T07:50:13Z – 08:16:08Z UTC
- 工具版本：见 §一
- corpus：146 entries 跨 41 features
- runner 健康：本次跑 0 panic / 0 internal timeout / 0 work 残留 / 0 false-positive timeout

读者引用本报告任何数字时务必同时引用 run id + 工具版本字符串。

## 九、附录

### 引用：每工具 cc-report

19 份深度报告路径：

```
deep-reports/cc-reports/cargo-check.md
deep-reports/cc-reports/kani.md
deep-reports/cc-reports/miri.md
deep-reports/cc-reports/charon-mono.md
deep-reports/cc-reports/charon-poly.md
deep-reports/cc-reports/rocq-of-rust.md
deep-reports/cc-reports/verifast.md
deep-reports/cc-reports/hax-fstar.md
deep-reports/cc-reports/hax-lean.md
deep-reports/cc-reports/soteria.md
deep-reports/cc-reports/creusot.md
deep-reports/cc-reports/hax-coq.md
deep-reports/cc-reports/aeneas-coq.md
deep-reports/cc-reports/aeneas-fstar.md
deep-reports/cc-reports/aeneas-lean.md
deep-reports/cc-reports/prusti.md
deep-reports/cc-reports/aeneas-hol4.md
deep-reports/cc-reports/verus.md
deep-reports/cc-reports/kmir.md
```

### 引用：宪法 / 架构 / 细化 / runner

```
docs/design/principles.md          — 项目精神宪法（核心约束）
docs/design/architecture.md         — 核心模块架构设计
docs/design/detailed-design.md      — 函数级细化、schema 完整定义
runner/src/                         — 核心模块 1 实现
tools/<name>/{tool.toml, harness.rs.tera, README.md}  — 19 工具配置
examples/<feature>/<dir>/           — 146 entry 样例库
```

### 引用：本次 run 数据

```
runs/run-1778226613-5282/results.json   — 裸 JSON 数据（host / 19 工具版本 / 全 task metadata）
runs/run-1778226613-5282/report.md      — runner 自动生成的 Markdown 表格（含完整 entry × tool 矩阵）
runs/run-1778226613-5282/raw/<tool>/<entry_id>.{stdout,stderr}  — 每 task 完整 raw output
```

### 引用：历史报告

```
docs/test-reports/feature-coverage-2026-05-07.md             — 早期 8 工具版
docs/test-reports/feature-coverage-2026-05-07-19tools.md     — 19 工具版（harness 缺陷未修）
docs/test-reports/feature-coverage-2026-05-07-fixed-harness.md — 19 工具 fix 后（旧 oracle）
docs/test-reports/feature-coverage-2026-05-08-19tools-strict-oracle.md  — 本报告（新 oracle）
```
