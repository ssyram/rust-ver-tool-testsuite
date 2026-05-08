# 内部综评 2026-05-08（暴论版）

## 锚点（全文不再重复）

- **run id**：`run-1778226613-5282`
- **时间窗**：2026-05-08T07:50:13Z – 08:16:08Z UTC（≈26 分钟）
- **corpus**：146 entries × 19 工具 = 2774 任务，0 UNKNOWN，0 TIMEOUT
- **host**：Apple M5 / macOS aarch64 / 24 GB / 10 cpu / parallelism 10
- **19 工具版本**（自 results.json metadata 抓）：

| 工具 | 版本锁定 |
| --- | --- |
| cargo-check | cargo 1.95.0 (f2d3ce0bd 2026-03-21) |
| miri | miri 0.1.0 (cb40c25f6a 2026-05-04) on nightly |
| kani | cargo-kani 0.67.0 |
| verus | 0.2026.05.03.8b81855 |
| kmir | mir-semantics 84bea09 + stable-mir-json 62a239d7 + K 7.1.282 |
| verifast | VeriFast 26.01 (released 2026-01-21) macOS arm64 prebuilt |
| soteria | soteria-rust @ 3c21278 + Obol @ ddea5ca5 (OCaml 5.4.0) |
| prusti | 0.2.2 commit a0681ee (2023-08-22) on nightly-2023-08-15-x86_64 via Rosetta |
| creusot | cargo-creusot 0.11.0 + nightly-2026-02-27 + Why3 1.8.2 |
| charon-mono / charon-poly | charon 0.1.184 (commit ed22146b) on nightly-2026-02-07 |
| aeneas-{coq,fstar,lean,hol4} | aeneas a14083a6 + charon 0.1.184 |
| hax-{coq,fstar,lean} | hax untagged-git-rev-30949eb87 on nightly-2025-11-08 |
| rocq-of-rust | rocq_of_rust_cli 0.1.0 @ a8a76a4d on nightly-2024-12-07 |

外溢免责见 `README.md` 顶部"关于 deep-reports/cc-reports/"段；本报告不重复。

---

## TL;DR — 七条暴论

1. **verifast 79.5% 是空过率，不是验证率**。整 corpus grep `//@\s*(req|ens|inv|pred)` 0 命中，配合 `-skip_specless_fns` 三件套，116 个 SUCCESS 里 104 个报同一 baseline `37 statements verified`（来自 verifast prelude 自身）。**没有任何 entry 真的让 prover 跑过**。
2. **aeneas 4 个 backend 不是同分**——前一份草稿示例里写"4 backend 同分 59%"是错的：`coq/fstar/lean` 三个 byte-identical 87/146（59.6%），但 **`hol4` 跌到 51/146（34.9%）**，单一根因是 `Extract.ml:3166 extract_trait_decl` 在 HOL4 backend 上对 trait decl 调 `Option.get None`，**LLBC 含任意 trait declaration（包括 core::fmt::Display、FnOnce、Iterator）就 panic**。这不是数据噪声，是 hol4 printer 的硬天花板。
3. **prusti 38.4% 是真前端接受率，不是它弱**——上一份配置（`PRUSTI_NO_VERIFY=true`）下勉强 67%，但那是 cargo-check 等价路径（encoder 根本不跑），违反反作弊。新配置 `NO_VERIFY=false ∧ DUMP_VIPER ∧ PRINT_HASH` 让 encoder 真跑 + Z3 永不启动，38.4% 才是 MIR → Viper VIR encoder 的真实接受率。**降的 28pp 都是真触及 prusti encoder 边界的 entry**，不是吃亏。
4. **kani 98.6% / miri 97.3% 是地板天花板**。kani 的 2 个 FAILED 都是 vendor x509-parser 的 `#![deny(unstable_features, unused_qualifications)]` 与 kani 注入的 `#![feature(register_tool)]` 撞车——**没有一个 FAILED 是 GotoC codegen 边界**。miri 的 4 个 FAILED 全部是 corpus 故意撒的 unsupported（inline-asm / FFI / network isolation / uninit-memory），**这不叫 corpus 没踩到，是 corpus 踩了 miri 自陈的所有边界都接住了**。
5. **charon-mono 94.5% / charon-poly 95.2% 不是命运攸关的 0.7pp**——poly 与 mono 在 142 个 entry 上一致，**剩 4 个 entry 行为不互含**：mono 单态化展开 `Box<dyn Display/Any>` 时在 vtable drop preshim 索引计算 panic（`translate_trait_objects.rs:1707`），poly 不展开 vtable 所以躲过；反过来 poly 在 std TLS 内部 polymorphic 路径踩 unsupported，mono 单态化后避开。**两边 FAILED 集合对称差 5 个 entry，互不蕴含**。
6. **kmir 31.5%、verus 34.9%、aeneas-hol4 34.9%、prusti 38.4% — 这四个垫底者根因各不同**。kmir 是 stable-mir-json schema 漂移 + K rule 缺失（K-stuck 56 条）；verus 是 vstd spec 边界 + verus-driver 内部 panic（A 桶 29 + D 桶 12 = 41 条不在语言子集层面）；aeneas-hol4 是 hol4 printer Option.get None；prusti 是 encoder MIR→Viper 真边界。**把它们并列在一张表上比较是地狱级的方法学错误**。
7. **工业三件套（rsa / sha2 / x509-parser）才是真分水岭**——cargo-check 6/6 全过，miri 6/6 全过，kani 4/6（x509 上撞 kani 注入 lint）；charon×2 6/6 全过；aeneas-{coq,fstar,lean} 在 x509 上过、在 rsa 和 sha2 上挂在 `Region ids should not be visited directly`（`interp/Interp.ml:550`）；creusot 在 x509 上过、在 rsa/sha2 上挂在 byte-string `b"hello rsa"` 的 `Unsupported constant value: Scalar(allocN) of type &[u8; M]`；其余 12 个工具集体 0/6。**这 6 个 entry 把"自带 cargo 集成"和"单文件读 src/lib.rs"两类工具一刀切干净**。

---

## §1 怎么测的

每个 entry 是单一 lib crate，必要时通过 `.hirusttest/config.toml` 注入 manifest deps（如 num-bigint、serde、chrono、rsa、sha2、x509-parser 等）；runner 为每个工具复制隔离副本并按 `entry_mode`（`bin` / `lib`）渲染 `__ts_harness`。判定是 entry × tool 二值（SUCCESS / FAILED；TIMEOUT / UNKNOWN 在本次 0 触发）；按 `principles.md` §六-2 "不允许 partial"——工具产出含 `[Error]` / `Generated the partial file` / silent sorry / silent `please implement` / silent `failure ((` 等"我没全干完"信号，一律翻 FAILED。

每工具 oracle 见各自 `tools/<name>/tool.toml`：

- 单门 exit code：cargo-check / miri / kani / verus / charon×2 / soteria / prusti / creusot / verifast / aeneas×4
- exit + stdout grep：kmir 抓 `#EndProgram ~> .K`（K 终止 signature）
- exit + 产物 grep：hax-{coq,fstar,lean} 抓 sentinel sorry / `failure ((` / `please implement the method`；rocq-of-rust 5 道门（exit + `.v` 存在 + 非 0-byte + > 200 字节 + 5-marker grep）

aeneas 的"产物 partial 但 exit 0"理论上是 silent path，但实测 aeneas 的 `Errors.error_list` 与 `craise` 把所有 unsupported 推 exit ≠ 0——单门 exit 即够。

## §2 测了什么特性

41 个 feature 类目 / 146 entries，类目分布如下（部分类目只 1 entry，不构成统计样本）：

```
基础语言：generic (4) / closure (2) / closure-adv (4) / lifetime (3) / hrtb (1) / impl-trait (1)
        / trait (1) / trait-obj (2) / assoc-type (1) / gat (1) / hello (1)
数值：int (2) / int-width (14) / float (10) / bigint (8)
内存：box (2) / drop (1) / refcell (1) / rc (1) / arc (1) / vec (1) / slice (1)
        / collections (2) / iter (1)
unsafe：unsafe-ptr (2) / unsafe-adv (3)
并发：concurrency (2)
其他：const (1) / panic (2) / error (1) / repr (2) / enum (2)
工具自陈限制集：aeneas-limit (8) / charon-limit (7) / creusot-limit (7) / hax-limit (8)
              / kani-limit (7) / miri-limit (7) / prusti-limit (8) — 共 52 entries
跨依赖 / 工业：deps-complex (7) / industrial (6) — 共 13 entries
```

**11 个 entry 全 19 工具都过**（"easy 路径"基准）：`hello/basic-hello`、`generic/identity-fn`、`int/wrapping`、`int-width/{cast-i64-u64-i128, cast-sign-extend, cast-truncate, wrapping-u8}`、`lifetime/multi-outlives`、`panic/div-zero`、`repr/c-struct`、`creusot-limit/mutual-recursion/trigger_is_even`。这些都是无 dyn / 无 closure / 无 raw-ptr / 无外部 dep / 无浮点的最简形态。

**3 个 entry 几乎全 19 工具都挂**（最难顶端）：`kani-limit/async-await/run_async_add` 3/19、`charon-limit/async-fn/async_forty_two` 4/19、`repr/union/repr_union` 6/19。async fn / coroutine 是当前 Rust 验证生态的大黑洞，union 紧随其后。

## §3 工具按"容易错的大类"分组评

按内部 pipeline 上"最容易错的大类"分组。每组一行小结，附辛辣观察。

### A. 编译型 baseline — cargo-check（1 个）

**100.0%**，146/146。本来设计上就该 100%——它就是 stable rustc 跑到 borrow check 完。这不是它的"能力"，是 corpus 整体在 stable Rust 接受面之内的反向证据。**意义**：其他 18 个工具任意一个在 entry X 上 FAILED，cargo-check 在同 entry 上一定 SUCCESS——所有 FAILED 都是工具自身能力差异，不是样例 Rust 写错了。

### B. 机械语义执行 — miri / kmir（2 个）

| 工具 | 通过率 | 注解 |
| --- | --- | --- |
| miri | 97.3% (142/146) | nightly toolchain 内置，纯 MIR 解释器 + UB 检测 |
| kmir | 31.5% (46/146) | K Framework MIR semantics + stable-mir-json schema |

**辛辣观察**：65.8 个百分点的差距不构成可比性。miri 是工业级工具——失败的 4 个 entry 全部是 corpus 故意撒的 miri 自陈不支持（inline-asm / extern FFI / TCP socket isolation / 故意触发的 UB `MaybeUninit::assume_init()`）。kmir 是研究原型——56 条 K-stuck（K rule 缺失对应 reduction：closure aggregation / float intrinsic / atomic / GAT）+ 35 条 SMIR JSON 解析失败（stable-mir-json schema 在 dyn / Box<dyn> / Arc<[T]> / 第三方 crate 上崩）+ 9 条 cargo build 失败。**kmir 的 31% 不是它"语义模型有问题"，是 stable-mir-json schema 漂移让 84bea09 的 K definitions 接不上**——README 自陈"实证扫到 102 个原始 SUCCESS 中 52 个是 K-stuck 假阳性"，新 oracle（grep `#EndProgram ~> .K`）才把它落到真实的 31%。

**最大短板**：closure aggregation、async/coroutine、float intrinsic、atomic 都让 K-stuck 集体爆炸；任何 Box/Arc + dyn 组合让 SMIR JSON 解析直接挂。`closure/fn-fnmut/{closure_fn, closure_fnmut}` 同形态 K-stuck 在 closure aggregation 路径。

### C. SMT 验证 — kani / verus / prusti / creusot（4 个）

| 工具 | 通过率 | 切割点 |
| --- | --- | --- |
| kani | 98.6% (144/146) | `--only-codegen`（GotoC codegen，CBMC 后端不跑） |
| verus | 34.9% (51/146) | `--no-verify`（VIR 构造，AIR/Z3 不跑） |
| prusti | 38.4% (56/146) | `NO_VERIFY=false ∧ DUMP_VIPER ∧ PRINT_HASH`（Viper encoder 真跑，Silicon 不启动） |
| creusot | 72.6% (106/146) | binary 无 subcommand（Coma 翻译完成，Why3 不跑） |

**辛辣观察一**：kani 的 98.6% 与其余三个的 34.9% / 38.4% / 72.6% 看起来惊天差距，**根因是切割点深度不同**——kani 的 GotoC codegen 几乎覆盖整个 stable Rust 输入面（除 inline-asm / FFI 之类），verus 的 VIR 构造要过严格的 mode/lifetime/vstd 三重 check，prusti 的 Viper encoder 要把 MIR borrow 翻成 Viper separation logic permission。**这不是"kani 比 verus 强 64pp"，是 kani 在它的前端边界上几乎无 reject 路径，verus 在它的前端边界上 reject 路径密集**。

**辛辣观察二**：creusot 72.6% 是这组的中位，表面强势——但 deps-complex 7 条只过 1 条（`chrono-bigint`），float 10 条只过 5 条（NaN 在 creusot_std 不支持），industrial 6 条只过 2 条（x509 上 rsa/sha2 byte-string 触发 `Unsupported constant value: Scalar(allocN) of type &[u8; M]`）。creusot 在普通 entry 上几乎都接住，**但凡进 serde derive 展开、float NaN spec、byte-string literal 三类，立刻全挂**。

**辛辣观察三**：verus 失败的 95 条里 **A 桶 29（vstd 边界）+ B 桶 19（语言子集 todo）= 48 条是 verus 自陈的 yet-to-support**——这是真实的能力边界。但 D 桶 12 条 + E 桶 7 条 = 19 条是 **verus-driver 内部 panic**（`generic_args.rs:54:14: index out of bounds: the len is 0 but the index is 0` 12 次同形态）。这 19 条按"不允许 partial"翻 FAILED，但从工具能力视角是 panic site，不是 reject 路径。

**共同短板**：四个 SMT 派工具全都在 `unsafe-ptr`、`async fn`、`industrial/rsa-sha2`、`closure-adv/boxed-dyn-fn`、`repr/union` 上挂。kani 是其中唯一一个在前 4 类上不挂的，**因为它的 codegen 不做 borrow / capability / spec 检查**。

### D. 模型检查 / 符号执行 — soteria / verifast（2 个）

| 工具 | 通过率 | 注解 |
| --- | --- | --- |
| verifast | 79.5% (116/146) | 单文件读 `src/lib.rs`，**vacuous pass**（见暴论 1） |
| soteria | 74.7% (109/146) | 单文件读 `src/lib.rs`，符号执行真跑 |

**辛辣观察**：表面接近的两个数字下藏着完全不同的语义。

- verifast：116 个 SUCCESS 里 104 个报 `37 statements verified` 同一 baseline——**没有任何 entry 让 prover 真跑**。-skip_specless_fns + corpus 0 spec → SUCCESS 退化为"rustc-verifast 接受 IR + 无 spec 可证伪"。子毫秒级响应（max 1479ms / median 200ms）就是不调 cargo + 不跑 prover 的物理证据。
- soteria：command 没有 dry-run flag，每个 SUCCESS 都真符号执行了——但因为单文件模式（`exec src/lib.rs`），所有需要外部 crate 的 entry（bigint 8 + deps-complex 7 + industrial 6 = 21 个）在 rustc 编译前置阶段就死，没机会跑符号执行。**soteria 的 74.7% 是真符号执行通过率（在能跑的 entry 上）**，verifast 的 79.5% 是 IR 接受率。

两者在工业三件套上 **0/6 vs 0/6**——单文件模式与 cargo 集成之间的鸿沟一刀切。

### E. MIR 中段翻译 — charon-mono / charon-poly / aeneas-{coq, fstar, lean, hol4}（6 个）

| 工具 | 通过率 |
| --- | --- |
| charon-poly | 95.2% (139/146) |
| charon-mono | 94.5% (138/146) |
| aeneas-coq | 59.6% (87/146) |
| aeneas-fstar | 59.6% (87/146) |
| aeneas-lean | 59.6% (87/146) |
| aeneas-hol4 | 34.9% (51/146) |

**辛辣观察**：这是最有"父子继承"关系的一组。charon 单跑 95%，aeneas 共享同一份 charon binary + 同一份 OCaml engine（mid-end），唯一差异是最后 `Extract.ml` 的 printer 分支选择 → 三个 backend 整齐 byte-identical 87/146。**aeneas vs charon 的 35.6pp 落差吃在 mid-end（borrow forward / backward translation, mut→functional update）+ printer 之间，不在 charon 翻译层**。

**hol4 的孤狼数据 34.9%**：与其他三个 backend 同一 LLBC、同一 mid-end，但 `extract_trait_decl` 在 HOL4 backend 分支调 `Option.get None`——LLBC 含任意 trait declaration（即便是 core::fmt::Display 这种 implicit dependency）就 panic。35.6pp 落差里**有 36 条 entry 是 HOL4-only fail**（fstar/coq/lean 全过、hol4 挂），这 36 条 100% 集中在含 closure / iterator / trait / 序列化 / Box<dyn> 的 entry。

**charon 的 mono vs poly 4 处对称差**（共 5 个 entry 行为不一致，`creusot-limit/dyn-trait-forbidden` 是 mono 独有 fail）：

| entry | poly | mono | 解释 |
| --- | --- | --- | --- |
| `lifetime/thread-local` | FAILED | SUCCESS | poly 进 std TLS polymorphic 路径踩 unsupported；mono 单态化避开 |
| `creusot-limit/thread-local-ref` | FAILED | SUCCESS | 同上 |
| `charon-limit/generic-to-dyn-unsize` | SUCCESS | FAILED | mono 展开 Box<dyn Display> vtable drop preshim panic |
| `lifetime/static-bound` | SUCCESS | FAILED | mono 展开 Box<dyn Any> vtable drop preshim panic |
| `creusot-limit/dyn-trait-forbidden` | SUCCESS | FAILED | mono 展开 Box<dyn Display> vtable drop preshim panic |

**0.7pp 数字差是巧合，能力差是确凿的**。

### F. syntactic 浅翻译 — hax-{coq, fstar, lean} / rocq-of-rust（4 个）

| 工具 | 通过率 |
| --- | --- |
| rocq-of-rust | 82.9% (121/146) |
| hax-fstar | 78.8% (115/146) |
| hax-lean | 75.3% (110/146) |
| hax-coq | 67.1% (98/146) |

**辛辣观察一**：syntactic 派的中位 77%，**比 SMT 派（kani 98.6% 排除掉之后剩三个均值 48.6%）和 model-check 派（mean 77.1%）都高**。原因：syntactic 不做 mode / lifetime / borrow / spec 检查，把 `&mut` 翻成字符串标签 `"MutRef"`、把 trait method 翻成字符串查表（rocq-of-rust 的 `M.get_trait_method "<trait_path>" "<method>"`），翻译阶段不做语义筛——"工具接受"≠"工具能在这段 Rust 上推 borrow 安全"。

**辛辣观察二**：hax 三个 backend（coq 67% / fstar 79% / lean 75%）的差距集中在 printer-level 与 reject phase 数量。fstar Printer 在 phase 阶段显式拒 mut-ref / raw-ptr / closure-captures-mut（`[HAX0003]/[HAX0008]/[HAX0010]/[HAX0011]` 系），coq Printer 增加 `reject_Dyn` / `reject_Unsafe` 多 reject 几条；lean Printer 反过来——它在 mut-ref / raw-ptr 上不显式拒，**走 silent sorry path**（`lean.rs:1287/2163 PatKind::Error / error_node` 直接 emit `text!("sorry")`）。oracle 用 grep 把 silent sorry 抓回 FAILED——hax-lean 的 36 条 FAILED 里 **20 条**靠 oracle 翻 silent path 才捕获，否则会被 cargo hax exit 0 误判为 SUCCESS。

**辛辣观察三**：rocq-of-rust 82.9% 是**单 syntactic 通路的全胜**——25 条 FAILED 里 21 条是单文件读 `src/lib.rs` 不读 Cargo.toml（bigint / deps-complex / industrial 全死在 unresolved import）+ 3 条是 nightly toolchain edition / unstable feature 默认（async-fn / let-chains），**真正落在它"翻译能力边界"上的只有 1 条 `repr/union/repr_union`**。在能接受的 entry 上 rocq-of-rust 给出完整翻译，没出现 silent partial。

**共同短板**：syntactic 派 4 个工具在工业三件套上 hax×3 各 4/6（x509 集体挂在 `unnecessary qualification` lint 升级为 error）/ rocq-of-rust 0/6（不读 Cargo.toml）。

---

## §4 高难度特性 × 工具矩阵

每行一个 entry，单元格 `S/M`（S 个工具过 / M 工具该组总数）。组缩写：**baseline=cargo-check (1)**、**exec=miri+kmir (2)**、**smt=kani+verus+prusti+creusot (4)**、**mc=soteria+verifast (2)**、**mir=charon×2+aeneas×4 (6)**、**syn=hax×3+rocq-of-rust (4)**。

### §4.1 generic / GAT / assoc-type / impl-trait / trait

| entry | base | exec | smt | mc | mir | syn |
| --- | --- | --- | --- | --- | --- | --- |
| generic/array-len/const_generic_array | 1/1 | 2/2 | 3/4 | 2/2 | 5/6 | 4/4 |
| generic/identity-fn/generic_identity | 1/1 | 2/2 | 4/4 | 2/2 | 6/6 | 4/4 |
| generic/pair-struct/generic_pair | 1/1 | 2/2 | 3/4 | 2/2 | 6/6 | 4/4 |
| generic/sum-bound/generic_sum_bound | 1/1 | 1/2 | 2/4 | 2/2 | 5/6 | 4/4 |
| gat/lending-iter/gat_lending | 1/1 | 1/2 | 2/4 | 2/2 | **2/6** | 1/4 |
| assoc-type/iter-style/assoc_type_iter | 1/1 | 2/2 | 3/4 | 2/2 | 5/6 | 1/4 |
| impl-trait/return-iter/impl_trait_iter | 1/1 | 1/2 | 2/4 | 2/2 | 5/6 | 4/4 |
| trait/cyclic-bound/cyclic_bound_use | 1/1 | 2/2 | 3/4 | 2/2 | **0/6** | 1/4 |

`trait/cyclic-bound`：MIR 翻译派**全军覆没**——charon×2 都触发 rustc stack overflow（cyclic trait bound 让 charon trait/type resolution 路径无限递归），aeneas×4 跟着挂；syntactic 派 3/4 也挂。`gat/lending-iter`：mir 翻译派 4 个 aeneas + 2 个 charon 中只过 2 个（charon×2，aeneas 全挂在 `Aeneas__Translate.trait_impl_is_builtin Not_found`）；syntactic 派 hax 三个全挂在 `[HAX0001] FunctionalizeLoops`，rocq-of-rust 过。

### §4.2 closure-adv / trait-obj / dyn

| entry | base | exec | smt | mc | mir | syn |
| --- | --- | --- | --- | --- | --- | --- |
| closure-adv/boxed-dyn-fn/boxed_dyn_fn | 1/1 | 1/2 | 1/4 | 2/2 | 2/6 | 1/4 |
| closure-adv/early-bound-lifetime | 1/1 | 1/2 | 1/4 | 2/2 | 2/6 | 4/4 |
| closure-adv/fn-once | 1/1 | 1/2 | 3/4 | 2/2 | 5/6 | 4/4 |
| closure-adv/return-impl-fn | 1/1 | 1/2 | 3/4 | 2/2 | 5/6 | 4/4 |
| trait-obj/conditional-method | 1/1 | 2/2 | 3/4 | 2/2 | 5/6 | 2/4 |
| trait-obj/dyn-dispatch | 1/1 | 1/2 | 2/4 | 2/2 | 2/6 | 2/4 |
| closure/fn-fnmut/closure_fn | 1/1 | 1/2 | 2/4 | 2/2 | 5/6 | 1/4 |
| closure/fn-fnmut/closure_fnmut | 1/1 | 1/2 | 2/4 | 2/2 | 5/6 | 1/4 |

`closure-adv/boxed-dyn-fn` 是 8/19——`Box<dyn Fn(i32)→i32>` 让 SMT 派全死（kani 过的是 codegen，verus/prusti/creusot 全挂）、aeneas×4 全挂在 `shallow-init-box` + `Dynamic trait types are not supported yet`、syntactic 只 hax-fstar 过。`closure/fn-fnmut/closure_{fn,fnmut}` 同形态：hax 三个 backend 全挂（`[HAX0003] DirectAndMut: closure 对外层局部变量赋值的 LocalMutation 阶段显式拒`）；唯独 rocq-of-rust 把 `closure_fn` 翻进去（但实际上它跟 closure_fnmut 行为一致都 0/4 hax + rocq syntactic 只 1 个过）。

### §4.3 lifetime（含 hrtb）

| entry | base | exec | smt | mc | mir | syn |
| --- | --- | --- | --- | --- | --- | --- |
| lifetime/multi-outlives | 1/1 | 2/2 | 4/4 | 2/2 | 6/6 | 4/4 |
| lifetime/static-bound/static_bound | 1/1 | 1/2 | 1/4 | 1/2 | **1/6** | 2/4 |
| lifetime/thread-local/thread_local_read | 1/1 | 1/2 | 1/4 | 2/2 | **1/6** | 1/4 |
| hrtb/for-all-lifetime | 1/1 | 1/2 | 3/4 | 2/2 | 2/6 | 3/4 |

`static-bound` 7/19、`thread-local` 7/19——TLS 与 `'static + Box<dyn Any>` 是验证生态的痛点。MIR 翻译派只过 1（`thread-local`：charon-mono 单态化避开 std TLS polymorphic 路径，但 charon-poly 挂；`static-bound`：charon-poly 过 / charon-mono 在 vtable drop preshim panic）。

### §4.4 unsafe-ptr / unsafe-adv

| entry | base | exec | smt | mc | mir | syn |
| --- | --- | --- | --- | --- | --- | --- |
| unsafe-ptr/raw-ptr-const | 1/1 | 1/2 | 1/4 | 2/2 | **0/6** | 1/4 |
| unsafe-ptr/raw-read | 1/1 | 1/2 | 1/4 | 2/2 | 2/6 | 1/4 |
| unsafe-adv/maybe-uninit | 1/1 | 1/2 | 2/4 | 2/2 | 6/6 | 1/4 |
| unsafe-adv/ptr-write | 1/1 | 2/2 | 2/4 | 2/2 | 2/6 | 1/4 |
| unsafe-adv/transmute | 1/1 | 2/2 | 3/4 | 2/2 | 6/6 | 3/4 |

`raw-ptr-const`（`let p = 43 as *const ();`）6/19，是单条 entry 第三难——MIR 翻译派 0/6（charon 显式拒 `Unsupported constant: ConstantExprKind::Cast {..}`，aeneas 走不到）。`raw-read` 也只 7/19。**hax 三个 backend 在 unsafe-ptr 上靠 `[HAX0008] reject_RawOrMutPointer` 全部显式拒**——这是 hax 设计内的 reject phase。

### §4.5 async / inline-asm

| entry | base | exec | smt | mc | mir | syn |
| --- | --- | --- | --- | --- | --- | --- |
| charon-limit/async-fn/async_forty_two | 1/1 | 1/2 | 1/4 | 0/2 | 0/6 | 1/4 |
| kani-limit/async-await/run_async_add | 1/1 | 0/2 | 1/4 | 0/2 | 0/6 | 0/4 |
| charon-limit/inline-asm/nop_via_asm | 1/1 | 0/2 | 2/4 | 0/2 | 0/6 | 3/4 |

async-await 3/19、async-fn 4/19——**这是 corpus 上最难的两个 entry**。整 SMT 派只剩 kani（`--only-codegen` 不评 SAT 求解）；mir 派全部 0/6（`Coroutine types are not supported yet`、aeneas 跟着挂）；mc 派也 0/2（verifast `error[E0670] async fn is not permitted in Rust 2015`，soteria 同源 Coroutine reject）。**这两条 entry 单纯就是 corpus 的"async 杀手"，非常成功地把所有 verifier 都 KO 了**。

### §4.6 bigint / float / int-width / panic / drop

| entry | base | exec | smt | mc | mir | syn |
| --- | --- | --- | --- | --- | --- | --- |
| bigint/bigint-arith | 1/1 | 1/2 | 2/4 | **0/2** | 5/6 | 3/4 |
| bigint/bigint-modpow | 1/1 | 1/2 | 3/4 | **0/2** | 5/6 | 3/4 |
| bigint/num-complex-ops | 1/1 | 1/2 | 3/4 | **0/2** | 2/6 | 3/4 |
| float/cast-int | 1/1 | 2/2 | 1/4 | 2/2 | 2/6 | 4/4 |
| float/nan-prop | 1/1 | 1/2 | 3/4 | 2/2 | 2/6 | 4/4 |
| float/total-order | 1/1 | 1/2 | 1/4 | 2/2 | 2/6 | 4/4 |
| int-width/cast-float-int | 1/1 | 2/2 | 2/4 | 2/2 | 2/6 | 4/4 |
| int-width/wrapping-u8 | 1/1 | 2/2 | 4/4 | 2/2 | 6/6 | 4/4 |
| panic/explicit | 1/1 | 2/2 | 2/4 | 2/2 | 6/6 | 4/4 |
| drop/custom-drop | 1/1 | 1/2 | 2/4 | 2/2 | 6/6 | 4/4 |

**bigint 8/8 在 mc 派 0/16 全挂**——verifast/soteria 单文件模式 unresolved import num_bigint。aeneas mid-end 在 `num-complex-ops` 上挂在 `Improperly typed constant value`（浮点字面常量不进 mid-end），其他 7 条 aeneas×4 都接住。

**float 整类是 aeneas 的免疫区**——`float/*` 10 条 aeneas×4 全 0，aeneas mid-end 在 LLBC 常量类型检查阶段拒浮点字面。但 charon×2 全过——**这是 mid-end 边界，不是 charon 翻译边界**。

### §4.7 box / repr / collections / error / const

| entry | base | exec | smt | mc | mir | syn |
| --- | --- | --- | --- | --- | --- | --- |
| box/shallow-init/shallow_init_box | 1/1 | 1/2 | 3/4 | 2/2 | 2/6 | 4/4 |
| repr/union/repr_union | 1/1 | 1/2 | 1/4 | 1/2 | 2/6 | **0/4** |
| error/result-question | 1/1 | 1/2 | 1/4 | 2/2 | 2/6 | 4/4 |
| collections/btreemap | 1/1 | 1/2 | 1/4 | 2/2 | 2/6 | 4/4 |
| collections/hashmap | 1/1 | 1/2 | 2/4 | 1/2 | 5/6 | 4/4 |

`repr/union` 6/19——syntactic 派**全军覆没** 0/4。rocq-of-rust 在 `top_level.rs` 走 `TopLevelItem::Error(Variant::Union)` 路径 emit `(* Error *)` marker 被 oracle 抓；hax 三个 backend 全在 Lean / Coq Printer 阶段 `[HAX0002]` Name rendering Union 拒；F\* backend 直接 OCaml uncaught exception。

### §4.8 工业三件套（最关键）

| entry | base | exec | smt | mc | mir | syn |
| --- | --- | --- | --- | --- | --- | --- |
| industrial/rsa/rsa-pkcs8/rsa_pkcs1v15_encrypt | 1/1 | 1/2 | 1/4 | **0/2** | 2/6 | 3/4 |
| industrial/rsa/rsa-pkcs8/rsa_pubkey_from_pkcs8 | 1/1 | 1/2 | 1/4 | **0/2** | 2/6 | 3/4 |
| industrial/sha2/sha256-digest/sha256_digest_incremental | 1/1 | 1/2 | 1/4 | **0/2** | 2/6 | 3/4 |
| industrial/sha2/sha256-digest/sha256_digest_one_shot | 1/1 | 1/2 | 1/4 | **0/2** | 2/6 | 3/4 |
| industrial/x509-parser/cert-parse/x509_parse_der | 1/1 | 1/2 | 1/4 | **0/2** | 5/6 | **0/4** |
| industrial/x509-parser/cert-parse/x509_subject_extensions | 1/1 | 1/2 | 1/4 | **0/2** | 5/6 | **0/4** |

详见 §5。

---

## §5 工业三件套专评

工业三件套（rsa / sha2 / x509-parser）是 corpus 设计上的"真分水岭"——比单 feature entry 难度跳了大概 5–10 级。

**代码量**（vendor 目录 src/ 排除 target/）：

| crate | LOC | 用了什么 |
| --- | --- | --- |
| x509-parser | 6367 | nom 解析器组合子 + asn1-rs（DER/BER）+ HashMap + 大量 lifetime 参数 |
| rsa | 6309 | num-bigint-dig + base64ct + pkcs1/pkcs8 + spki + pem-rfc7468 + rand_core |
| sha2 (crate src) | 3151 | digest + crypto-common + hybrid-array + cpufeatures（含 riscv_zknh 路径） |

每条 entry 触发的 cargo build 把上述 crate + 它们的传递依赖（serde / typenum / num-traits / num-integer / generic-array 等）全部拖进编译，然后让工具在用户 entry（约 30–40 行）上做翻译/验证。**这是真正的 cargo + 工具链集成压力测试**。

### 5.1 cargo-check / miri / kani / charon×2 — 6/6 全过（5 个工具）

cargo-check 显然过；miri 在 RSA 加密最长 35.7s 解释执行（涉及大量 BigInt modpow + SHA-256 内部循环）；kani GotoC codegen 4/6 全过（x509 上 2 条挂在 vendor crate 的 `#![deny(unstable_features)]` × kani 注入 `#![feature(register_tool)]`，**不是 codegen 边界**）；charon×2 把 vendor 完整 deps tree 全部翻译到 LLBC，industrial 在 charon 层无差异。

**这 5 个工具有共同特征**：自带 cargo 集成、不假设单文件输入、对 vendor crate 不做强行 spec 接管。

### 5.2 aeneas-{coq,fstar,lean} — 2/6（x509 过、rsa+sha2 全挂）

x509-parser 翻译成功是 surprise——里面到处是 `lifetime + generic + nom combinator`，比 rsa 复杂得多。但 rsa/sha2 上 aeneas 集体挂在：

```
[Error] Region ids should not be visited directly; the visitor should catch
        cases that contain region ids earlier.
[Warn] Could not translate the body of function 'rsa_pkcs8::__ts_inner::rsa_pkcs1v15_encrypt'
       Compiler source: interp/Interp.ml, line 550
```

触发点是 `der::asn1::any::AnyRef` / `BitStringRef` 这类带 region 参数的"unknown type declaration"在 `TypesAnalysis.ml:148` 被警告，再到 `interp/Interp.ml:550` panic。**aeneas mid-end 对 region-parametric unknown type 的处理路径在 rsa/sha2 dep 树上炸了，但在 x509-parser dep 树上没炸**——同 dep（asn1-rs / der）在两边都用，差异在 entry 函数体怎么用这些类型。

aeneas-hol4 在 industrial 上 0/6（除了 x509 外其他都跟 coq/fstar/lean 一样挂；x509 上 hol4 也挂在 trait declaration 引发的 Option.get None panic）。

### 5.3 creusot — 2/6（同 aeneas：x509 过、rsa+sha2 全挂）

cargo build 跑通完整 deps tree（`sha2 v0.11.0` / `rsa v0.9.8` 都编过），creusot-rustc 接受 vendor 的 trait/struct 定义，**死在 entry 用户代码里的 byte-string 字面量**：

```
error: Unsupported constant value: Scalar(alloc4) of type &'?7 [u8; 9_usize]
  --> src/__ts_inner.rs:33:60
   |
33 |         let _ = pub_key.encrypt(&mut rng, Pkcs1v15Encrypt, b"hello rsa");
```

`b"hello rsa"`（9 字节）和 `b"the quick brown fox..."` 这类 byte string literal 在 creusot 翻译为 Coma 的常量值时没覆盖路径。**rsa/sha2 的两条 entry 都在用 byte string 当输入数据**，所以全挂；x509 不用 byte string（用 `include_bytes!` 直接 reference DER 数据），所以过。

### 5.4 hax-{coq,fstar,lean} — 4/6（rsa+sha2 过、x509 全挂）

**与 aeneas / creusot 相反**——hax 在 rsa+sha2 上 4/4（hax 三个全过这两条，加 hax-coq 一条 = 在 rsa+sha2 上累计 12/12 哦不应该是 hax×3 各 4 条 = 12/12 对），x509 全挂。

x509 上 hax 三个挂在 vendor crate 的 lint：

```
warning: hiding a lifetime that's elided elsewhere is confusing
   --> .../vendor/x509-parser/src/x509.rs:578:31
... 7 errors emitted
warning: hax: running `cargo build` was not successful, continuing anyway.
```

vendor x509-parser 的 7 处 `unnecessary qualification` + `hiding lifetime that's elided` 在 hax 的 nightly-2025-11-08 上被升级为 error → cargo build fail → hax engine 没机会跑。**这跟 hax 翻译能力无关，是 nightly toolchain × vendor crate 的版本鸿沟**。

### 5.5 verifast / soteria — 0/6 整队覆灭

两个都是单文件 `src/lib.rs` 模式，**进入 rustc 解析阶段就死**：`error[E0432]: unresolved import 'rsa'` / `'rand'` / `'sha2'` / `'x509_parser'`。verifast 内嵌 rustc parser 解析失败、soteria 的 Obol 编译前置失败——都不是符号执行 / VeriFast IR 构造能力边界，是**输入流水线层与 cargo deps 不兼容**。

工业三件套**把 19 个工具一刀切干净**：

- 5 个工具 6/6 全过：cargo-check / miri / kani(4/6) / charon-mono / charon-poly
- 2 个工具 4/6（rsa+sha2 过 / x509 挂）：hax-fstar / hax-lean / hax-coq（注：hax-coq 在 rsa-encrypt 上挂在另一条原因，记 3/6）
- 2 个工具 2/6（x509 过 / rsa+sha2 挂）：aeneas-coq / aeneas-fstar / aeneas-lean / creusot
- 0/6：aeneas-hol4 / verus / prusti / kmir / verifast / soteria / rocq-of-rust

**这是本 corpus 上最强的工具能力分化信号**。

### 5.6 industrial 暴露的盲点

- aeneas 的 `Region ids should not be visited directly` 在小 entry 上从未观察到——industrial 才暴露此 path。
- creusot 的 byte-string `b"..."` 翻译盲点在小 entry 上从未触发——只 industrial 触发。
- kani 注入 `#![feature(register_tool)]` × vendor `#![deny(unstable_features)]` 的撞车也是 industrial 独有。
- hax 的 nightly toolchain × vendor lint 升级 error 也是 industrial 独有（在 deps-complex 上未观察到）。

工业三件套的设计意图——"看哪些工具能在真实 cargo + vendor crate 上工作"——完美兑现。

---

## §6 时长（avg / max）

| 工具 | avg (ms) | max (ms) |
| --- | --- | --- |
| rocq-of-rust | 168 | 485 |
| verifast | 283 | 1479 |
| verus | 563 | 2312 |
| soteria | 1846 | 12690 |
| cargo-check | 2124 | 26420 |
| charon-mono | 2297 | 31899 |
| miri | 2801 | 35727 |
| charon-poly | 2955 | 34693 |
| hax-fstar | 3315 | 33132 |
| hax-lean | 3647 | 26768 |
| hax-coq | 3736 | 26428 |
| aeneas-lean | 3855 | 36310 |
| kani | 3872 | 40023 |
| aeneas-fstar | 3985 | 31561 |
| aeneas-hol4 | 3990 | 32647 |
| aeneas-coq | 4075 | 35173 |
| kmir | 8197 | 105497 |
| prusti | 12594 | 76666 |
| creusot | 40048 | 64641 |

**异常者**：

- creusot avg 40s（远高于其他）——cargo + nightly-2026-02-27 + creusot-rustc 替换 rustc 跑整条 cargo 流水线，每个 entry 都重编 creusot-std + Why3 IR 翻译。
- prusti avg 12.6s + max 76.7s——Prusti via Rosetta + JDK 17 经 JNI + Viper encoder 真跑。
- kmir max 105.5s——K Framework LLVM backend 解释执行（K interpreter 重）。
- verifast avg 283ms——不调 cargo 不调 prover（vacuous pass，§3D 已述）。

---

## §7 暴论 / 内部观察 / 规律

7.1 **支持率前 5 名都是不写产物 .v / .lean 的工具**：cargo-check 100% / kani 98.6% / miri 97.3% / charon-poly 95.2% / charon-mono 94.5%。其中 cargo-check / kani / miri 不产生形式化 IR 文件，charon×2 产 LLBC 但不进任何 prover 后端。**写产物文件就要面对 printer / silent partial 问题**——hax-lean 的 silent sorry path、aeneas-hol4 的 Option.get None、rocq-of-rust 的 5-marker grep guard，全是写产物的工具特有的 oracle 问题。

7.2 **"不读 Cargo.toml 的工具"在 bigint/deps-complex/industrial 三类上全 0**：verifast / soteria / verus / rocq-of-rust 都吃过这个亏。**单文件输入 = 工业级 corpus 死刑**——这 21 个 entry 占 14.4%，**工具支持率最多被砍 14pp 起步**。

7.3 **"工具自陈限制集"在该工具自己上不一定全 fail**：
- prusti-limit 8 条在 prusti 上 1/8（设计意图实现：prusti 期望失败的样例真 fail 7 个）
- kani-limit 7 条在 kani 上 7/7（kani --only-codegen 切割点早于 kani 自陈的 unsupported——这些 unsupported 在 CBMC 求解阶段才显现）
- miri-limit 7 条在 miri 上 6/7（只有 networking 触发了 miri 的 isolation reject，其余 6 条在 miri 上其实都过；fail 的反而散在其他 corpus）
- aeneas-limit 8 条在 aeneas-coq/fstar/lean 上 4/8（半数预期 fail）

**"limit" 命名假设这些样例真触发该工具限制——但工具切割点决定了实际触发率，不是 corpus 标签决定的**。

7.4 **aeneas 4 个 backend 不是同分**——常被误以为"4 backend 同分 59%"。实际是 coq/fstar/lean **三个** byte-identical 87/146（59.6%），hol4 跌到 51/146（34.9%）。**hol4 在共享 mid-end 上加了 36 条 HOL4-only fail**，单一根因 `extract_trait_decl Option.get None`。这 36 条 100% 集中在含 trait declaration 的 entry。

7.5 **charon-mono 与 charon-poly 的 0.7pp 数字差是骗人的**——FAILED 集合对称差 5 个 entry，**两个 mode 的翻译能力互不蕴含**：mono 在 vtable drop preshim 索引上特有 panic（3 个 entry：generic-to-dyn-unsize / static-bound / dyn-trait-forbidden），poly 在 std TLS polymorphic 路径上特有 panic（2 个 entry：thread-local / thread-local-ref）。**这是 charon mid-end 的对称交易**——单态化避开 polymorphic 路径但要走 vtable 实例化路径，反之亦然。

7.6 **hax 三个 backend 的 7pp 差距全在 printer + reject phase 数量**：fstar 79% > lean 75% > coq 67%。fstar Printer 较成熟（reject_RawOrMutPointer / reject_ArbitraryLhs / reject_TraitItemDefault 等显式 reject），所以 fstar 的 silent path 很少；lean Printer 在 mut-ref / raw-ptr 上**走 silent sorry**，oracle 用 grep 把 20 条 silent fallback 抓回 FAILED；coq 多 reject 几个 phase（reject_Dyn / reject_Unsafe），所以 coq 通过率最低但是"显式 reject 最干净"。

7.7 **kani --only-codegen 切割点几乎覆盖整个 stable Rust 输入面**。除了 vendor lint 撞车的 2 条 industrial-x509，没有任何 entry 触发 GotoC codegen 拒收路径——`kani-limit/*` 7 条全过（这些"不支持"在 SAT 求解阶段才显现）、bigint 8/8、deps-complex 7/7、float 10/10、closure-adv 4/4。**kani 的 98.6% 是 codegen 接受率，不是验证通过率**——后者在 SAT 求解阶段才有意义，本测试不进。

7.8 **miri 的 4 个 FAILED 全部是 corpus 故意设计的 miri 边界触发**：inline-asm / extern FFI / TCP socket / `MaybeUninit::assume_init()` UB。**这是 corpus 设计意图的完美兑现**——不是 miri 能力问题，是 corpus 在测 miri 边界且 miri 被测出。

7.9 **prusti 38.4% > verus 34.9% > kmir 31.5% 的并列误导性极强**。prusti 是 Viper VIR encoder 真跑的接受率（A 桶 68 条 unsupported feature 是真实能力边界），verus 是 VIR 构造率（A 桶 vstd 边界 + B 桶语言子集边界 + D 桶 verus-driver 内部 panic），kmir 是 K interpreter 终止率（K-stuck 56 条是 K rule 缺失）。**三个工具在做完全不同的事，比通过率 = 比方法学错误**。

7.10 **整个矩阵最难顶的是 async fn / coroutine**——`charon-limit/async-fn` 4/19，`kani-limit/async-await` 3/19，全行业（除 cargo-check / miri 之一 / kani --only-codegen / 个别 syntactic）集体失败。这是当前 Rust 验证生态的真黑洞——**async desugar 产生 Coroutine MIR 节点 + impl Future opaque type，几乎所有验证器都在前端就死**。corpus 上这两条 entry 是"async 杀手"，非常成功地把所有真验证器都打趴。

---

## §8 当下我方观察

我们当前的判断（基于本 run，不锚定其他时刻）：

- **verifast 的 79.5% 在我们的 corpus 上完全是 vacuous pass**——若想测它能力的真实部分，需要给 entry 加 `//@ req/ens` 注解（违反 entry 自包含原则）或不加 `-skip_specless_fns`（plain Rust 会因隐式溢出 / panic 路径触发误判）。**当前 verifast 数字最不解释力**。

- **aeneas 在我们 corpus 上像是 charon 的一层 ~36pp 衰减**——不是 charon 错，是 aeneas mid-end 的 borrow rewrite + printer 多吃了一层翻译边界。这是 aeneas 项目设计上的"翻译深度"代价，与"接受面广度"是两回事。我们 corpus 这个量级下的实测落差稳定在 35.6pp（87 vs 138 = 51 个 entry mid-end 拒）。

- **hol4 backend 的 35pp 落差是 upstream 单一 bug**（Option.get None on trait decl）——若 upstream 修补 `extract_trait_decl` 路径，hol4 数字会瞬间向 fstar/coq/lean 看齐。这是少见的"单个 commit 可消解 35pp 数字差"的边界。

- **kmir 31.5% 不能反映其语义模型**——是 stable-mir-json schema 漂移让当前 K definitions 接不上多数 SMIR 节点。在能跑通 SMIR JSON 的 entry 上，K interpreter 经常完整跑到 `#EndProgram ~> .K`。这工具的瓶颈在适配层，不在 K 上。

- **kani / miri 的高数字在我们 corpus 上是结构性的**：kani 切在 `--only-codegen`，几乎不踩 reject 路径；miri 是 nightly toolchain 内置 + corpus 故意撒的 miri-limit 也只触发 4 个。把这两个工具的"高数字"读为"工具碾压"是错的——读为"切割点接受面广 + corpus 没设计针对它们的杀手 entry"才对。

- **真分水岭在工业三件套**——5 个工具 6/6 / 4 个工具 4/6 / 4 个工具 2/6 / 6 个工具 0/6 的清晰四档分层，远比单 feature entry 的工具排序更说明问题。日后 corpus 扩展应继续往工业方向加。

---

## §9 收尾

19 个工具在 146 entries 上 2774 个判定的尘埃落定。数字是客观的；分布是结构性的；解释要看版本与切割点。下一次 run 之前，先想清楚把哪些 entry 加进来能让现在 90%+ 的几个工具下到 80%——那才是 corpus 增量真有信号的方向。
