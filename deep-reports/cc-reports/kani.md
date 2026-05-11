# kani 深度报告

## 元数据

- **run**：`runs/run-1778466265-63960/`（2026-05-11，3-tool P13-B 重跑：146 entries × {kani, hax-fstar, hax-coq}；host：Apple M5 / macOS / aarch64 / 24 GB / 10 cpu）
- **历史 run（对照）**：`runs/run-1778226613-5282/`（2026-05-08 主 run，旧 oracle 下 kani 144/146）
- **工具版本**：`cargo-kani 0.67.0`
- **通过率**：**136/146 = 93.2%**（P13-B 重跑，封堵 §C1 codegen-with-unsupported-stub 漏报后；旧 oracle 下 144/146 = 98.6%）
- **时长**（毫秒）：avg 3563 / median 455 / p90 13955 / max 28988
- **oracle 改造**：`tools/kani/kani-strict-wrapper.sh` 包装 `cargo kani --only-codegen --bin __ts_harness`，stdout 命中 5 markers（`TerminatorKind::InlineAsm` / `simd_cast` / `catch_unwind` / `ptr_mask` / `C string literal`）任一 → 重写 exit 2 + 诊断。详 `docs/fixes/oracle-leak-rules-implementation-2-2026-05-11.md` §2.1
- **时效声明**：本快照锚定上述 run id + 工具版本 + corpus，不构成长期承诺。

## 工具内部 pipeline + 前端边界

Kani 由 AWS 出品，基于 CBMC 的 Rust bounded model checker。pipeline 是：cargo-kani 把 rustc 替换为 `kani-compiler`（rustc plugin）；kani-compiler 接管 rustc 前端后把 MIR 翻译为 GotoC IR，写出 goto-binary（`*.symtab.out`）+ 元数据；最后 `cargo kani` 把 goto-binary 喂给 CBMC 做 SAT/SMT 求解判 reachability / assertion violation。

本测试用 `--only-codegen` 把测试边界精确切在 codegen 完成之处——**前端**（kani-compiler 的 MIR → GotoC + 类型 / 模型构造）算本工具范围；**后端**（CBMC 求解）不算。让 kani 跑完整 SAT 求解会失公（其他工具都在前端层停下，kani 却因求解超时频繁 FAILED）。

## SUCCESS 信号 + 形式严格性

**判定式**（P13-A 后）：

```
SUCCESS ⟺ kani-strict-wrapper.sh exit 0
       ⟺ cargo kani --only-codegen --bin __ts_harness exit 0
         ∧ stdout 不命中 5 markers 任一：
           - TerminatorKind::InlineAsm
           - simd_cast
           - catch_unwind
           - ptr_mask
           - C string literal
```

5 markers 由 kani 自陈的 `Found the following unsupported constructs:` warning list 选出——kani 文档原话："Verification will fail if one or more of these constructs is reachable"。换言之 kani 自承认这些 MIR 构造没真翻译，只 emit 了 stub；按宪法 §六-2 反作弊精神（"不允许 partial / 不允许 silent skip"），命中即 FAILED。

**为什么是这 5 markers 而非完整 warning list**：实测在 v1 run 上扫 144 SUCCESS：
- `caller_location` 60/144 命中（std panic 路径，几乎每个 non-trivial entry 都触发）
- `foreign function` 63/144 命中（std alloc 的 `posix_memalign`/`memcpy`）

这两条是 kani 对 std 内部的标准 stub 处理，不是用户代码触发的 hard-unsupported MIR。把它们纳入会让 ~44% SUCCESS 翻车（包括 `bigint-arith` / `hello/basic-hello` / 各 `industrial/*`）—— 结构性误报。5-marker subset 是工程上"既封堵真漏报又避免大规模假阳性"的精筛集合。这同时构成 0 漏报盲点：`caller_location` / `foreign function` 路径仍未封堵，存疑由 cc-report 修订小组定。

**形式严格性**：

- **0 误报**：✅ 实测验证。kani exit 0 + 5 markers 不命中 ⇔ codegen 完成且 stdout 无 hard-unsupported stub 警告。5 markers 在 6 个 真 SUCCESS 上实测**不命中**（`hello/basic-hello/hello` / `industrial/rsa/rsa-pkcs8/*` / `bigint/bigint-arith/bigint_arith` 等），双向反误报实测过（详 implementation log §2.1）
- **0 漏报**：⚠️ 实测验证。5 markers 在 8 个原 SUCCESS entry 上命中且都被 P13-B 重跑翻为 FAILED，但仍有 `caller_location` / `foreign function` 路径未抓 —— 属"宪法精神 vs cc-report 现行口径"分歧的暂未解项。理论上还有 SAT 阶段才会触发问题的 entry（本次未观察到）
- **漏报盲点**：`caller_location` / `foreign function` warning 的 stub codegen（≈ 60/144 命中且属合法 std 路径，未封堵）；codegen 完成 + 其他 warning 但 SAT 阶段才会触发问题的 entry（本次未观察到）

## 实测结果

### 按 feature 类目分布

136 SUCCESS / 10 FAILED。10 个 FAILED 分布：

```
工业三件套：industrial 4/6（x509-parser 上 2 条 FAILED，rsa / sha2 全过）
P13-A 新封堵：
  charon-limit 6/7（inline-asm/nop_via_asm 命中 TerminatorKind::InlineAsm）
  concurrency 1/2（thread-mutex/thread_mutex_join 命中 C string literal + catch_unwind + ptr_mask）
  deps-complex 3/7（bigint-serde / chrono-serde / collections-serde 命中 InlineAsm+simd_cast，error-chain 命中 catch_unwind+ptr_mask+simd_cast）
  kani-limit 6/7（stack-unwinding/trigger_divide_with_recovery 命中 catch_unwind）
  miri-limit 6/7（thread-interleaving-partial/unsynchronised_counter_race 命中 C string literal + catch_unwind + ptr_mask）
```

其余 35 个 feature 类目全部 SUCCESS——包括 `bigint/* 8/8`、`float/* 10/10`、`int-width/* 14/14`、`closure-adv/* 4/4`、`hax-limit/* 8/8`、`prusti-limit/* 8/8`、`creusot-limit/* 7/7`、`aeneas-limit/* 8/8`，以及多数 `industrial`/`deps-complex` 中不撞 5 markers 的 entry。

注意 `kani-limit/*` 仍 6/7（仅 stack-unwinding 在 P13-A 后翻 FAILED）——其余 6 条（inline-assembly / async-await / float-overapprox / thread-interleaving-partial 在 kani-limit 下、extern-ffi / uninit-memory）仍是 SUCCESS：其中 `inline-assembly/trigger_add_via_asm` 的 stdout 是 `caller_location` + `foreign function` warning（被 5-marker subset 故意排除），不是 5 markers；`extern-ffi/trigger_call_libc_abs` 同理（foreign function 路径）。kani-limit 7 条里只有 stack-unwinding 进 `catch_unwind` 路径——这些"不支持"的差异化恰反映 kani 自陈警告分类。

### 失败模式归类

#### A. crate `#![deny(unstable_features, unused_qualifications)]` 与 kani 注入的 `#![feature(register_tool)]` 冲突（2/10）

| entry | 触发 |
|---|---|
| `industrial/x509-parser/cert-parse/x509_parse_der` | vendor x509-parser `lib.rs:122` 设 `#![deny(unstable_features, unused_qualifications, ...)]` |
| `industrial/x509-parser/cert-parse/x509_subject_extensions` | 同一 vendor crate |

stdout 节选：

```
error: unnecessary qualification
   --> /.../vendor/x509-parser/src/cri_attributes.rs:31:41
note: the lint level is defined here
   --> /.../vendor/x509-parser/src/lib.rs:123:31
123 |         unused_import_braces, unused_qualifications)]

error: use of an unstable feature
   --> <crate attribute>:1:12
  1 | #![feature(register_tool)]
note: the lint level is defined here
   --> /.../vendor/x509-parser/src/lib.rs:122:9

error: Failed to execute cargo (exit status: 101). Found 8 compilation errors.
```

诊断指向 vendor crate 自身的 `lib.rs`：crate 用 `#![deny(unstable_features)]`，但 kani 在编译该 crate 时注入了 `#![feature(register_tool)]`（用于 `#[kani::proof]` 的 `register_tool(kanitool)`）；同时 kani 的 lint 配置让 `unused_qualifications` 命中 8 处，被 `#![deny(unused_qualifications)]` 升为 error。两类共 8 errors → cargo exit 101。

对比同 entry 在 cargo-check 上的行为：cargo-check 也会触发 `unused_qualifications` 的 lint，但只是 warning（不在 deny 升级路径里——stable rustc 不需要注入 `register_tool`），所以 cargo-check exit 0。**这 2 条 FAILED 是 kani-compiler 注入 `register_tool` feature × vendor crate `deny(unstable_features)` 的交互**——既不在样例的 Rust 合法性层面（cargo-check SUCCESS），也不在 GotoC codegen 能力层面（错误发生在 lint 检查阶段，未到 MIR 翻译）。

#### B. unsupported MIR codegen stub（5 markers 命中，P13-A 新封堵；8/10）

| entry | 命中 markers | 来源 stub |
|---|---|---|
| `charon-limit/inline-asm/nop_via_asm` | `TerminatorKind::InlineAsm (1)` | 用户代码 `unsafe { asm!("nop") }` |
| `concurrency/thread-mutex/thread_mutex_join` | `C string literal (1)` + `catch_unwind (3)` + `ptr_mask (1)` | `std::thread::spawn` 路径走 `__rust_panic_cleanup` 的 catch_unwind + `Mutex::new` 的 ptr_mask |
| `deps-complex/bigint-serde/bigint_serde` | `TerminatorKind::InlineAsm (5)` + `simd_cast (2)` | serde derive 展开 + num-bigint-dig 在 lazy_static 路径上 asm + SIMD |
| `deps-complex/chrono-serde/chrono_serde` | `TerminatorKind::InlineAsm (5)` + `simd_cast (2)` | 同上 |
| `deps-complex/collections-serde/collections_serde` | `TerminatorKind::InlineAsm (5)` + `simd_cast (2)` | 同上 |
| `deps-complex/error-chain/error_chain` | `catch_unwind (1)` + `ptr_mask (1)` + `simd_cast (1)` | thiserror 派生 + std 错误链 |
| `kani-limit/stack-unwinding/trigger_divide_with_recovery` | `catch_unwind (1)` | 用户代码显式 `std::panic::catch_unwind(...)` |
| `miri-limit/thread-interleaving-partial/unsynchronised_counter_race` | `C string literal (1)` + `catch_unwind (4)` + `ptr_mask (1)` | `std::thread::spawn` 路径同 thread_mutex_join |

stderr 完整诊断（举 `inline-asm/nop_via_asm` 一例）：

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

注意分布形态：
- `charon-limit/inline-asm` / `kani-limit/stack-unwinding` 是**用户代码直接触发**（inline asm / catch_unwind）—— 这是 corpus 设计意图踩到的边界
- `concurrency/thread-mutex` / `miri-limit/thread-interleaving-partial` / `error-chain` 是**std/std::thread spawn 路径间接触发**——用户代码看起来人畜无害（`thread::spawn(|| ...).join()`、`Result<>` chain），但 std 内部走 `catch_unwind` 包裹 + `Mutex::new` 走 `ptr_mask`
- `deps-complex/{bigint,chrono,collections}-serde` 是**vendor crate 间接触发**——serde 的 derive 展开 + num-bigint-dig 的 lazy_static / 优化路径用 inline asm 与 SIMD

### 时长尾端观察

`max=29s` 出现在 `deps-complex/*` / `industrial/*` 一段。**v3 max（28988 ms）显著低于 v1 max（40023 ms）**——v3 是 3-tool 重跑（kani / hax-fstar / hax-coq 三工具并发 10 路），与 v1 19-tool × 146 = 2774 task 的 4-5 倍 cargo cache 竞争相比，I/O 压力显著下降；avg 也从 v1 3871 ms 降到 3563 ms。耗时差异是环境上下文（host 上 cargo 并发数）—— 不是工具能力变化。timeout 设 120s 在两次 run 都未触发。

## 关键发现 / 暴论

### kani 是第二个 verifast 类语义滑移案例

v1（旧 oracle）下 cc-report 自陈"0 漏报本次未观察到"，audit-2 §3.1 在 raw stdout 上 grep 5 markers 找出 8 个 SUCCESS 命中 `Found the following unsupported constructs:` 警告里的 hard-unsupported 类——P13-B 重跑确证这 8 条原本属 codegen stub 漏报。**通过率 98.6% → 93.2%，-8 entries / -5.5pp**。

与 verifast 的对照：

| 维度 | verifast（P12）| kani（P13）|
|---|---|---|
| 原口径 | exit 0 = SUCCESS | exit 0 = SUCCESS（含 unsupported warning）|
| 实测语义降级 | `-skip_specless_fns` + 0 spec corpus → vacuous pass | `--only-codegen` + unsupported warning → codegen stub |
| 工具自陈口径 | "0 errors found" 来自 prelude，不验用户 | "Verification will fail if one or more constructs is reachable"——kani 自承没真翻 |
| oracle 封堵后通过率 | 79.5% → 8.2%（-71pp）| 98.6% → 93.2%（-5.5pp）|
| 漏报机制类型 | A 类（spec-less skip）| C1 类（codegen 完成 + unsupported stub）|

两者的共同点是：**工具默认接受范围 > 工具实际处理能力**，oracle 默认只看 exit code 时会把工具自身警告里的"我没真干完"翻成 SUCCESS。规模差异巨大（verifast 71pp vs kani 5.5pp），但机制本质同——都是 §六-2 "不允许 partial" 在源码层 + 实测层的兑现。

### audit 推荐 5 markers 子集的精筛源于实测约束

audit-2 §3.1 初版给的是完整 `Found the following unsupported constructs:` warning list grep；实测发现纳入完整 list 会让 ~44% SUCCESS 翻车（含 hello/basic-hello / bigint-arith / industrial/rsa），原因是 `caller_location` / `foreign function` 是 std panic / std alloc 的标准 stub，几乎每个 non-trivial entry 都触发。**5 markers 是从合法 SUCCESS 反向校准出的精筛集**——这种 "audit 给规则，实施按反误报实测校正" 的方法学，与 P12 verifast `N≤40` 阈值被 falsify 同源。

## 与本次测试边界的关系

- **测试切割点**：本测试不调用 CBMC、不评 SAT/SMT 求解器在 GotoC 上的验证结果。SUCCESS 仅蕴含"kani-compiler 完成 MIR → GotoC codegen **且 stdout 不含 5 markers 中任一**"。下游 reachability / assertion 验证不在测量范围
- **已知 corpus 偏向**：`kani-limit/*` 7 个 entry 故意触发 Kani 自声明的"不支持"特性。P13-B 重跑后 6/7 通过（仅 stack-unwinding 因 `catch_unwind` 命中翻 FAILED）——其余 6 条不命中 5 markers 子集（如 inline-assembly / async-await 在 stdout 是 `caller_location` + `foreign function` warning）。**corpus 设计 7 条全失败的意图未完全兑现**，但分化形态精确反映了 kani 警告分类——若把 corpus 改成显式 `asm!{}` 块或 `catch_unwind` 块的形态，剩余 6 条也会翻 FAILED
- **本次未触达**：kani 的 GotoC codegen 因 MIR 节点 unsupported 而 hard-reject 的案例（区别于本次的 stub-with-warning）。10 个 FAILED 中 2 个在 lint 层面（x509 vendor）、8 个在 5-markers stub 层面，**0 个在 hard-reject 层面**

## 历史快照声明

本报告是 2026-05-11 P13-B 运行 `runs/run-1778466265-63960` 的实测快照；锚定 `cargo-kani 0.67.0` × 5-marker oracle subset × 当前 corpus（146 entries，含 6 个 industrial vendor crate entries）。kani 升级、kani-compiler 修复 lint 注入、上游 `Found the following unsupported constructs:` warning 格式变化、cc-report 修订小组对 `caller_location` / `foreign function` 路径口径裁定等任一变化后均需重测。
