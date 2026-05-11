# 0 误报审计：各工具失败样例归因（2026-05-11）

## §0 元数据

- **审计日期**：2026-05-11
- **审计触发**：prusti viper_tools 损坏事件（旧 run 中 prusti 全 161 FAILED 实际是 SCRIPT_DIR 算路径错导致 Java backend 启动不了的环境问题；按宪法 §六 "0 误报硬指标"，环境损坏不应当被 oracle 算作工具 partial）
- **审查范围**：全 matrix run `run-1778492036-50081`（20 工具 × 161 entry × 1 run = 3220 次运行）；prusti 由于 `run-1778492036-50081` 当时 viper_tools 损坏导致 161 全 FAILED，本次审计中 **prusti 列改用 `run-1778494055-14621`**（viper_tools 修复后的重跑，105 FAILED）作为有效样本。
- **误报定义**：oracle 判 FAILED，但根因 **不是** "工具内部 reject / 工具自我披露 unsupported / 工具发现 issue / 工具崩溃在工具自身边界内"，而是落到下述任一外部根因：
  1. runner harness 设计造成的编译失败（entry 本身合法 Rust，工具能 type-check）；
  2. vendor / external crate 在工具特定 toolchain 下编译失败（entry 本身合法）；
  3. 工具 pipeline 不通过 cargo 解决 deps，因此外部 crate 全部 unresolved（entry 本身合法 + 在 cargo-check 上 SUCCESS）；
  4. 工具锁死老 cargo 不支持 edition 2024，但 entry 用了 2024 edition（entry 本身合法）；
  5. 环境损坏（如 viper_tools / opam switch / Java JVM）；
  6. 资源问题（OOM / SIGKILL no-timeout / 磁盘满）；
  7. runner spawn 失败 / cp 失败。
- **真 FAILED 定义**：上述以外，即 "工具自己说不支持 / 工具显式拒绝 entry / 工具内部 panic 在工具自身代码 / 工具发现 entry 中的 issue / oracle 反作弊判 vacuous-pass"，均视为工具能力问题的真 FAILED。
- **本次审计不修任何 tool.toml / wrapper / runner / oracle**，仅作归因 + 修复路径建议。

---

## §1 审计方法

### 1.1 数据源

每个 FAILED 样本读取其 `raw_stdout` + `raw_stderr`，剥除 ANSI 颜色码后做关键字归类：

- `error[E0061]` + `runnable/*` → 归 §2.1 类 (runner harness)
- `error[E0432]: unresolved import` / `error[E0433]: failed to resolve` → 归 §2.2 类
- `unused_qualifications` + `x509-parser` 源路径 → 归 §2.3 类
- `this version of Cargo is older than the` + `2024` → 归 §2.4 类
- 其他 → 真 FAILED（再细分工具自身机制）

### 1.2 判据原则

按宪法 §六 "形式指标 0 误报 / 0 漏报"：

> "工具不能 type-check / encode 此 entry" 才算 partial 或 fail；
> "工具 pipeline 上游（cargo / Cargo.toml edition / vendor crate / harness）失败" 算误报。

判据边界澄清：
- **verifast 的 vacuous-pass**（118 个）**不算误报** — 这是 oracle 的反作弊设计（spec-less entry verifast 默认 trivial pass without verification，oracle 把它判 FAIL 是合理的"工具没干活 ≠ SUCCESS"决定）。在这次审计中归为"真 FAILED（oracle 反作弊判定）"。
- **kmir 的 "multi-exec projects currently not supported"**（59 个）**不算误报** — kmir 自我披露不支持，是工具能力问题。
- **工具自身 panic / ICE / stack overflow / OCaml uncaught exception**（如 charon-driver panic / aeneas `[Error] Internal error` / hax `[HAX0002] Fatal error` / verus `panicked at rustc_mir_build/...verus.rs`）**不算误报** — 工具内部 bug 在工具能力问题这一侧。
- **rocq-of-rust 的 silent-skip-but-no-entry_fn**（11 个）**不算误报** — 是 oracle 反作弊检测 rocq-of-rust 静默丢 item 的设计。

---

## §2 各工具失败样例分桶（按误报类）

### §2.1 FP 类 A：runnable harness 编译失败（15 entries × 9 工具）

**根因**：runnable corpus 的 entry function 都是带参数的零次函数（如 `fn fact(n: u32) -> u32`、`fn gcd(a: i64, b: i64) -> i64`），但 runner 在 bin-mode 下生成的 harness 一律是：

```rust
fn main() {
    runnable_<crate>::<entry_fn>();   // 调零参
    let _ = ();
}
```

所以 harness 编译失败：`error[E0061]: this function takes N arguments but 0 arguments were supplied`。

这类 entry 本身是 **完全合法的 Rust**：cargo-check 在 lib-mode（不通过 harness）下可以 type-check；creusot 的 cargo 编译同样在 type-check 阶段失败而非 creusot encoder 拒绝；kani 没机会跑 codegen；miri 没机会跑解释器；verifast/verus 没机会跑符号执行；kmir 没机会跑 K interpreter。

**完整影响清单**：

| 工具 | runnable FAILED | runnable FP 数 |
|---|---|---|
| cargo-check | 15/15 | 15 |
| creusot | 15/15 | 15 |
| kani | 15/15 | 15 |
| kmir | 15/15 | 15 |
| miri | 15/15 | 15 |
| prusti | 15/15 | 15 |
| soteria | 15/15 | 15 |
| verus | 15/15 | 15 |
| verifast | 14/15 | 14（`struct-norm/manhattan_of` 上反而 SUCCESS — verifast 的 `-skip_specless_fns` 把它当 vacuous skip 然后归 SUCCESS，14 vs 15 的差异不影响定性）|

**典型 stderr 摘录**（cargo-check 上 `runnable/abs/my_abs`）：

```
error[E0061]: this function takes 1 argument but 0 arguments were supplied
 --> src/bin/__ts_harness.rs:2:5
  |
2 |     runnable_abs::my_abs();
  |     ^^^^^^^^^^^^^^^^^^^^^^ argument #1 of type `i32` is missing
```

**归类小计**：15 × 9 工具 = 135 个 FP（其中 verifast 是 14，所以 134）。但严格按"哪些工具受 harness 影响"看，9 工具中 8 个有 15，verifast 是 14。

合计 **134 个 FP** 来自 runnable harness 设计。

---

### §2.2 FP 类 B：unresolved-import（工具不通过 cargo，deps 没接到）

**根因**：verus / verifast / soteria / rocq-of-rust(-typecheck) 五个工具的 pipeline **不通过 cargo** 跑（直接调单文件 `src/lib.rs`），因此 entry 用 `[extra_cargo_deps]` 拉外部 crate 时，工具看到 `use num_bigint::BigInt;` 就 `error[E0432]: unresolved import num_bigint`。

这些 entry 本身完全合法 — 在 cargo-check（50081 run）上全部 SUCCESS。是工具 pipeline 的设计选择（如 verus 因 cargo-verus 当前 upstream bug 不能用、转而单文件调）导致 entry 在它们上无法被吃下，但不是工具拒绝 entry 中的 Rust 特性。

**典型 stderr**（verus 上 `bigint/bigint-arith/bigint_arith`）：

```
error[E0432]: unresolved import `num_bigint`
 --> src/__ts_inner.rs:5:5
  |
5 | use num_bigint::BigInt;
  |     ^^^^^^^^^^ use of unresolved module or unlinked crate `num_bigint`
```

**完整影响清单**（21 个 entries × {verus, verifast, soteria}，19 × {rocq-of-rust, rocq-of-rust-typecheck}）：

涉及的 entry crate 桶：
- `num_bigint` (bigint-arith / bigint-bitwise / bigint-conv / bigint-modpow / num-integer-gcd / num-rational-arith / num-traits-abstract / bigint-serde) — 8 entries
- `num_complex` (num-complex-ops) — 1 entry
- `chrono` (chrono-bigint / chrono-serde) — 2 entries
- `serde` (collections-serde / trait-serde-generic) — 2 entries
- `anyhow` (error-chain) — 1 entry
- `itertools` (itertools-multi) — 1 entry
- `rsa` (rsa_pubkey_from_pkcs8 / rsa_pkcs1v15_encrypt) — 2 entries
- `sha2` (sha256_digest_one_shot / sha256_digest_incremental) — 2 entries
- `x509_parser` (x509_parse_der / x509_subject_extensions) — 2 entries

verus / verifast / soteria 全 21 都中招；rocq-of-rust(-typecheck) 是 19（少 x509 那 2 个，因为 x509 不仅 unresolved-import 还触发 vendor crate 编译失败，rocq-of-rust 把它归一类，但归类成 unresolved-import 的是 19）。

**实际归到 unresolved-import 一类的 FP 数**：

| 工具 | unresolved-import FP |
|---|---|
| verus | 21 |
| verifast | 21 |
| soteria | 21 |
| rocq-of-rust | 19 |
| rocq-of-rust-typecheck | 19 |

合计 **101 个 FP**。

**复杂判断**：可以辩 — "verus / verifast / soteria 在 cargo 整合不能 work 时，oracle 应当判 UNKNOWN 还是 FAILED？"按宪法 §六 "形式指标"，把"工具吃不下 deps-rich entry"算 partial 是合理的反作弊（避免工具假装支持复杂场景），但**严格按 0 误报**：oracle 在 stderr 看到纯 `error[E0432]: unresolved import` 且 cargo-check 在同 entry SUCCESS 时，应该判 UNKNOWN（"工具 pipeline 与 entry 不匹配"），不是 FAILED。

---

### §2.3 FP 类 C：vendor x509-parser 在工具特定 toolchain 下编译失败

**根因**：`vendor/x509-parser` 是用户为 industrial corpus 添加的 vendored crate，其 `src/lib.rs` 顶部设置：

```rust
#![deny(... unused_qualifications)]
```

x509-parser v0.16.0 的源码有若干 `crate::cri_attributes::parser::parse_attribute` 之类的"过度限定路径"。在新版 rustc 上这个 lint 被升级或触发更频繁，于是 vendor 在 hax (`nightly-2025-11-08`) 与 kani 的 toolchain 下编译失败。

但这 entry 本身（`industrial/x509-parser/cert-parse/{x509_parse_der, x509_subject_extensions}`）的代码只是几行调用，是合法 Rust。问题在 vendor crate + 工具特定 toolchain 的 lint level。

**典型 stderr**（kani）：

```
error: unnecessary qualification
   --> vendor/x509-parser/src/cri_attributes.rs:31:41
    |
 31 |   let (i, parsed_attribute) = crate::cri_attributes::parser::parse_attribute(i, &oid)
note: the lint level is defined here  ← `unused_qualifications` deny
...
error: could not compile `x509-parser` (lib) due to 8 previous errors
```

**影响**：

| 工具 | x509 vendor FP |
|---|---|
| aeneas-hol4 | 2 |
| hax-coq | 2 |
| hax-fstar | 2 |
| hax-lean | 2 |
| kani | 2 |

合计 **10 个 FP**。

注：aeneas-coq/fstar/lean 也通过 cargo 编译，但实测在 `nightly-2026-02-07` 与默认 toolchain 上没触发 lint 升级 → 不在影响列表。aeneas-hol4 这 2 个的 FAILED 还伴随 aeneas backend "translated_crate_of_json failed on x509-parser" 类报错，但根因仍是 vendor 编译失败。

---

### §2.4 FP 类 D：prusti 锁死的 cargo 不支持 edition 2024（7 entries）

**根因**：prusti `0.1.0 (2023-09-19)` 整套发布带的 cargo 版本太老，只支持 `2015 / 2018 / 2021` editions，看到 `edition = "2024"` 直接：

```
error: failed to parse manifest at ...
Caused by:
  this version of Cargo is older than the `2024` edition, and only supports `2015`, `2018`, and `2021` editions.
```

**影响 entries**（7 个，全部是 industrial corpus 与 hax-limit/let-chains，它们 Cargo.toml 用 edition = "2024"）：

- `hax-limit/let-chains/hax_limit_let_chains`
- `industrial/rsa/rsa-pkcs8/rsa_pubkey_from_pkcs8`
- `industrial/rsa/rsa-pkcs8/rsa_pkcs1v15_encrypt`
- `industrial/sha2/sha256-digest/sha256_digest_one_shot`
- `industrial/sha2/sha256-digest/sha256_digest_incremental`
- `industrial/x509-parser/cert-parse/x509_parse_der`
- `industrial/x509-parser/cert-parse/x509_subject_extensions`

是 **prusti 工具 release 锁死老 cargo 的环境限制**，不是 prusti front-end 不支持这些 entry 中的 Rust 特性。entry 本身完全合法。

**注**：industrial 的 RSA / SHA / x509 entries 选 edition 2024 是被依赖 crate 推着走的（rsa / sha2 / x509-parser 新版要求）；hax-limit/let-chains 是因为 let-chains 是 edition 2024 才稳定特性。

**FP 数**：7（仅 prusti 工具）。

---

### §2.5 各工具完整 FAILED 分桶汇总表

prusti 行用 14621 run（105 FAILED），其他用 50081 run。

| 工具 | tot_FAILED | 真 FAILED | FP-runnable | FP-x509 | FP-unresolved | FP-prusti-2024 | FP 率 |
|---|---|---|---|---|---|---|---|
| aeneas-coq | 59 | 59 | 0 | 0 | 0 | 0 | 0.0% |
| aeneas-fstar | 59 | 59 | 0 | 0 | 0 | 0 | 0.0% |
| aeneas-hol4 | 95 | 93 | 0 | 2 | 0 | 0 | 2.1% |
| aeneas-lean | 59 | 59 | 0 | 0 | 0 | 0 | 0.0% |
| cargo-check | 15 | 0 | 15 | 0 | 0 | 0 | **100.0%** |
| charon-mono | 8 | 8 | 0 | 0 | 0 | 0 | 0.0% |
| charon-poly | 7 | 7 | 0 | 0 | 0 | 0 | 0.0% |
| creusot | 55 | 40 | 15 | 0 | 0 | 0 | 27.3% |
| hax-coq | 50 | 48 | 0 | 2 | 0 | 0 | 4.0% |
| hax-fstar | 33 | 31 | 0 | 2 | 0 | 0 | 6.1% |
| hax-lean | 36 | 34 | 0 | 2 | 0 | 0 | 5.6% |
| kani | 25 | 8 | 15 | 2 | 0 | 0 | **68.0%** |
| kmir | 115 | 100 | 15 | 0 | 0 | 0 | 13.0% |
| miri | 19 | 4 | 15 | 0 | 0 | 0 | **78.9%** |
| prusti | 105 | 83 | 15 | 0 | 0 | 7 | 21.0% |
| rocq-of-rust | 37 | 18 | 0 | 0 | 19 | 0 | 51.4% |
| rocq-of-rust-typecheck | 37 | 18 | 0 | 0 | 19 | 0 | 51.4% |
| soteria | 52 | 16 | 15 | 0 | 21 | 0 | **69.2%** |
| verifast | 148 | 113 | 14 | 0 | 21 | 0 | 23.6% |
| verus | 110 | 74 | 15 | 0 | 21 | 0 | 32.7% |
| **合计** | **1124** | **872** | **134** | **10** | **101** | **7** | **22.4%** |

**整体审计结论**：全 matrix 1124 个 FAILED 中，**252 个是误报，误报率 22.4%**。

---

## §3 各工具真 FAILED 的工具能力图谱（保留以供对照）

挑几个典型的"真 FAILED"分布来确认我们没把工具拒绝当误报：

### 3.1 prusti（真 83）

主要分桶：
- `[Prusti: unsupported feature] iterators are not fully supported yet` × 13
- `[Prusti: internal error]` + 偶尔的 mir_analyses panic × 7
- `[Prusti: unsupported feature] unsizing Box<[T;N]> into Box<[T]>` × 多个
- `[Prusti: unsupported feature] unsupported constant type &'? ...` × 多个
- `[Prusti: unsupported feature] access to reference-typed fields` × 3
- `[Prusti: unsupported feature] unions are not supported` × 3
- `[Prusti: unsupported feature] higher-ranked lifetimes` × 2
- `[Prusti: unsupported feature] casts (IntToFloat / FloatToFloat) are not supported` × 多个
- `[Prusti: unsupported feature] thread-local storage` × 2
- `[Prusti: unsupported feature] inline-asm`（伴随 mir_analyses panic）× 3
- 其他 prusti-specific unsupported / 单点 panic

这些都是 prusti front-end 自己说"不支持此特性"，归真 FAILED。

### 3.2 verifast（真 113）

主要分桶：
- `[verifast-oracle] FAIL: vacuous pass — symex executed 0 statements` × 113 — verifast 对 spec-less entry 默认 `-skip_specless_fns` 跳过用户代码、只验证 prelude，oracle 判 vacuous = FAIL（反作弊设计，合理的真 FAILED）。
- `error: Floating point types are not yet supported` × 3
- `error: Structs with const parameters are not yet supported` × 2
- `error: let chains are only allowed in Rust 2024 or later` × 1（rustc front-end 报，verifast 的 rust toolchain pinned 较老）
- `error: async fn is not permitted in Rust 2015` × 2（同上 rust edition pinning）
- 其他 verifast pipeline 内部错误

注：verifast 的 `async fn is not permitted` / `let chains` 这种和 prusti edition-2024 同源属于 toolchain-pinning 类，但 verifast 的 oracle 已知此特性属于 verifast 不能处理的边缘（async / let-chains 都是 verifast 已知不支持），所以归真 FAILED。和 prusti edition-2024 的区别：prusti 7 个 industrial entry 是 RSA / SHA / x509，这些 entry 用 edition 2024 完全是因为 deps 需要，**与 prusti 是否支持任何 entry 中的 Rust 特性无关**。

### 3.3 verus（真 74）

主要分桶：
- `panicked at rustc_middle/...generic_args.rs / verus.rs` × 18（verus 自身 ICE，真 FAILED）
- `The verifier does not yet support the following Rust feature: ...` × 多个真 unsupported（bool bitwise / pattern bind / dyn 多 trait / unsizing / FFI / inline-asm / pointer deref / 等）
- `is not supported (note: you may be able to add a Verus specification ...)` × 多个对标准库类型缺 spec
- `Verus does not currently support closures capturing a mutable reference` × 2

### 3.4 kani（真 8）

- `[kani-oracle] FAIL: codegen completed with hard-unsupported MIR constructs` × 8，自我披露 `InlineAsm / catch_unwind / ptr_mask / simd_cast / C string literal` 等 codegen 不支持的 MIR 构造。这是 kani 自我披露 + oracle 抓 partial → FAIL，是标准的"工具能力问题"。

### 3.5 miri（真 4）

- `error: unsupported operation: inline assembly is not supported`
- `error: unsupported operation: can't call foreign function abs on OS macos`
- `error: unsupported operation: socket not available when isolation is enabled`
- `error: Undefined Behavior: reading memory at ...` (miri 真发现 entry 里的 UB)

### 3.6 kmir（真 100）

- 56 个 `[kmir-oracle] FAIL: K interpreter stuck (no #EndProgram ~> .K terminator)` — kmir 跑到 stuck，是 kmir 对该 entry 的 MIR 不能完整 reduce，工具能力问题。
- 44 个 `multi-exec projects currently not supported` + Python `JSONDecodeError` — kmir 自我披露不支持 multi-exec 项目（即 bin 模式下含 lib + harness 同时存在的 cargo project），算工具能力问题。

### 3.7 aeneas-coq/fstar/lean/hol4（真 59/59/59/93）

全部是 aeneas backend 报的 `[Error] ...`：
- `Improperly typed constant value` × 13~14（每个后端）
- `Internal error, please file an issue` × 4~12
- `Inconsistent projection`、`Region ids should not be visited directly`、`Arrow types are not supported yet`、`We don't support arrow types with locally quantified regions`、`Assertion failed: new value doesn't have the same type as its destination`、`Constant generics and type definitions with trait clauses are not supported yet when generating code for HOL4`、`Can not extract trait associated types with parameters`、`Unsupported operation: shallow-init-box`、`Invalid inputs for binop`、等等。
- aeneas-hol4 额外 24 `Uncaught exception (Invalid_argument "option is None")` — aeneas HOL4 backend 内部 OCaml exception (`Aeneas__Extract.extract_trait_decl` Option.get failure)，是工具内部 bug 但属能力问题，归真 FAILED。

### 3.8 hax-coq/fstar/lean（真 48/31/34）

- `[HAX0001] something is not implemented yet` × 多
- `[HAX0003] The mutation of this &mut is not allowed here` × 多（hax issue #420）
- `[HAX0008] Explicit rejection by a phase in the Hax engine` × 多
- `[HAX0002] Fatal error: something we considered as impossible occurred` × 多
- `[HAX0010] At this position, Hax was expecting an expression of the shape &mut _`
- `[HAX0011] The support in hax of function with one or more inputs of type &mut _ is limited`
- `fatal runtime error: stack overflow, aborting` × 1 (`trait/cyclic-bound/cyclic_bound_use`, rustc front-end stack overflow on cyclic trait bounds 通过 hax frontend exporter 调 — 也算 hax pipeline / 通过 rustc 的能力问题)
- hax-lean 额外的 19 个 `[hax-lean-oracle] FAIL: silent partial — sorry in term position` 是 oracle 对 hax-lean backend 发 `sorry` 的反作弊（hax-lean 把 unsupported AST 节点降级成 `sorry` 字面量，oracle 抓 sorry → FAIL）。这是 oracle 反作弊设计正确捕到 hax-lean 静默部分翻译 → 真 FAILED。
- `prettyplease panicked` × 1（hax-lean）— hax 用 prettyplease 格式化输出，遇到 `# [feature(register_tool)] type f_Output : TodoPrintRustBoundsTyp ;` 这种 hax-injected 语法 prettyplease 不识别 panic。是 hax 工具链内部问题（hax 喂给 prettyplease 不合法语法），归真 FAILED。

### 3.9 rocq-of-rust（-typecheck）（真 18）

- `entry_fn missing from .v products (silent skip)` × 多 — oracle 反作弊抓 rocq-of-rust 静默丢 item（top-level 翻译器对某些 kind 直接 `vec![]`），真 FAILED。

### 3.10 charon-mono/poly（真 8/7）

- `warning: Coroutine types are not supported yet` + `panicked at src/errors.rs:282:13` — charon `--abort-on-error` 在 first warning 时 abort + driver panic，是 charon 自我披露不支持 + 配置抓 partial → 真 FAILED。
- `warning: Inline assembly is not supported` + panic（同）
- `Could not determine method index for drop in vtable` + `Thread panicked when extracting item core::fmt::Display` — charon 内部 trait object vtable 处理 bug。
- `stack overflow` × 1（cyclic_bound_use）— charon-driver 对深递归 trait 解析 stack overflow。

### 3.11 creusot（真 40）

- `[creusot] Unsupported constant value: Scalar(allocN) of type &'? [u8; N_usize]` × 多（creusot 不能处理 byte-string literal）
- `Unsupported pointer cast: ... (FloatToFloat / IntToFloat / FloatToInt / PointerWithExposedProvenance / ClosureFnPointer)` × 多
- `panicked at creusot/src/translation/specification.rs / ty.rs / terminator.rs` × 3 — creusot 内部 ICE
- `error[E0277]: the trait bound 'X: creusot_std::prelude::IteratorSpec' is not satisfied` × 多（creusot 标准库适配缺失）
- `unsupported definition kind DefId(...)` × 多

### 3.12 soteria（真 16）

soteria 的 24 个 "opam-note + 真错误"，包含：
- `bug: Dangling pointer in lib::main` × 多（soteria 对标准库 `HashMap.insert` 触发的 NEON intrinsics 误判 dangling — 但严格按 oracle 这是 soteria 自己的判定，归真 FAILED）
- `unsupported feature, Unsupported intrinsic: atomic_xsub` × 2
- `Can't execute function ... GAst.Error/Missing` × 多
- `Extern function {abs, getpid} is not handled` × 2
- `Unsupported syntax / GAst.Error` × 多

---

## §4 关键发现 Top 5

### 4.1 Finding #1（最严重）：runnable corpus 在 9 个 bin/build-crate 工具上 100% 机械 FAILED — 134 个 FP

15 个 runnable entry × {cargo-check, creusot, kani, kmir, miri, prusti, soteria, verifast(14), verus} = **134 个误报，单一最大 FP 类**。

**根因**：runner 在 bin mode 下生成的 harness 是 `runnable_<crate>::<entry_fn>()` 无参调用，但 runnable corpus 的 entry function **全都带参**。harness 编译失败，工具 pipeline 还没走到自己的 type-check / encoder / interpreter 就 die 了。

**这是 oracle / runner 设计上严重的 0 误报违反**：

- runnable entry 本身是合法 Rust 代码（lib-mode 的 cargo-check 在它们上一定 SUCCESS，但 50081 cargo-check 配置 entry_mode = "bin"，所以表现为 FAILED）；
- 工具实际**没有机会**判断 entry 中的 Rust 特性是否支持；
- "工具不支持带参 main" 不是工具能力问题（C 语言 main 也带参，没人这么解读）。

按 0 误报硬指标：runnable corpus 在 bin/build-crate 工具上应该是 **UNKNOWN** 或 **N/A**，不是 **FAILED**。

### 4.2 Finding #2：单文件 pipeline 工具的 deps-rich entry 全 FP — 101 个 FP

verus / verifast / soteria 配置 `entry_mode = "lib"` 但直接调单文件 `src/lib.rs`，不通过 cargo 解决 deps。rocq-of-rust(-typecheck) 同样不通过 cargo。

这 5 个工具在 21 个 deps-rich entry（bigint / chrono / serde / rsa / sha2 / x509 / itertools / anyhow / num_*）上全部因 `error[E0432]: unresolved import` FAILED。

cargo-check 在同样这些 entry 上全部 SUCCESS（cargo-check 用 cargo + extra_cargo_deps），确认 entry 本身合法。

**应当判 UNKNOWN**：工具不通过 cargo 是工具 pipeline 设计选择（如 verus 因 cargo-verus upstream bug 不可用，被迫单文件），不是工具拒绝 entry 中的 Rust 特性。

### 4.3 Finding #3：prusti 锁死老 cargo 不支持 edition 2024 — 7 个 FP

prusti `0.1.0 (2023-09-19)` 整套发布带的 cargo 太老，看到 `edition = "2024"` 直接 fail-parse-manifest。这 7 个 entry（industrial RSA × 2, SHA × 2, x509 × 2, hax-limit/let-chains × 1）本身合法，是因 prusti pinned 老 cargo 而不能处理。

这 7 个 FP 本质和 §4.1 同源："工具 pipeline 上游"问题，不是 prusti front-end 不支持这些 entry 中的 Rust 特性。

按 0 误报：oracle 应当在 stderr 看到 `this version of Cargo is older than the 2024 edition` 时判 **UNKNOWN**（"工具锁死的 cargo 与 entry 不兼容"），不是 FAILED。

### 4.4 Finding #4：vendor x509-parser 在工具特定 toolchain 触发 unused_qualifications deny — 10 个 FP

vendor/x509-parser/src/lib.rs 第 123 行 `#![deny(... unused_qualifications)]`，新版 rustc 把它升级为 hard error，导致 vendor 编译失败。

x509-parser 在 cargo-check / charon / creusot / kmir / miri / prusti / verus / verifast 等用更新版 rustc 或更宽容 lint 配置的工具上编译 OK，所以 entry 本身合法；但在 hax (`nightly-2025-11-08`) 和 kani 的 toolchain 上编译失败 → 10 个 FP。

这是 vendor crate 在工具特定 toolchain 下的 lint level 不兼容。应当判 UNKNOWN（"工具 toolchain 与 vendor 不兼容"），不是 FAILED。

### 4.5 Finding #5：cargo-check FAILED 全是 runnable harness FP — 该工具的 0 误报率是 100% — 没真 FAILED

cargo-check 在 50081 run 全部 15 FAILED **都** 是 runnable harness 误报。换言之：cargo-check 这个"基准"工具，对全 161 entry 中没有一个是真"工具能力问题"FAILED — 它对所有合法 Rust 都 SUCCESS。这从基准角度反向印证：**任何工具上的 runnable 15 FAILED 也都是 harness 问题，不是工具能力问题**。

也即 cargo-check 这一行 100% FP 率反过来是审计验证的强基准。

---

## §5 推荐修复路径

### §5.1 修 runner harness：runnable corpus 的"机械 FAILED"

**根因**：runner discover 阶段把 `entry_fn` 当作零参函数生成 harness，没看 entry_fn 的参数签名。

**修复路径（推荐）**：

- 选项 A：runner 检测到 `entry_fn` 有参数时，harness 一律改为 lib-mode（不生成 bin harness，runnable corpus 只跑 lib-checking）。这样 runnable 在 bin/build-crate 工具上的 status 改判 UNKNOWN 或直接不跑（视配置）。
- 选项 B：runner discover 阶段 parse entry_fn 参数类型，harness 用合法字面量自动喂值（如 i32 → 0, bool → false）。这种方法风险：用户的 entry_fn 不应当对 0 / false 等输入 panic（panic 会变成"工具发现 issue"，混淆语义）。不推荐。
- 选项 C：runnable corpus 的 toml 加 `harness_args = [...]`，每个 entry 自己声明该传什么参。可行但增加 entry 维护成本。
- 选项 D（最快保守）：oracle 端在每个 bin-mode 工具上加规则——若 harness 编译失败（exit 101 + `error[E0061]: ... function takes ... arguments but 0 ... supplied`），改判 UNKNOWN 不判 FAILED。**最小侵入，最快收益 134 个 FP 修复**。

推荐先做 D（oracle 兜底，立即收益），再做 A 或 C（runner 层正确性）。

### §5.2 oracle 区分"unresolved-import + cargo-check SUCCESS" → UNKNOWN

verus / verifast / soteria / rocq-of-rust(-typecheck) 这 5 个工具：
- oracle 在 stderr 看到 `error[E0432]: unresolved import` / `error[E0433]: cannot find module or crate` 且 entry 有 `extra_cargo_deps` 时，改判 **UNKNOWN**。
- 这样 101 个 FP 改正，工具列上的"真 unsupported"统计才干净。

更彻底：让 oracle 知道 cargo-check 的对应 entry 是否 SUCCESS — 若 cargo-check SUCCESS 但单文件工具 unresolved-import → 工具的 pipeline 与 entry 不匹配 = UNKNOWN。但这要求 oracle 跨工具查 cargo-check 结果，增加耦合。短期上简单规则即可。

### §5.3 oracle 区分"prusti edition-2024" → UNKNOWN

prusti 看到 `this version of Cargo is older than the` 的 manifest parse error 时改判 UNKNOWN，立即修 7 FP。同时建议在 prusti README 中明确"prusti 锁死老 cargo，对 edition 2024 整组 entry 不可达"，避免误读为"prusti 不支持 RSA / SHA / x509 中的 Rust 特性"。

### §5.4 oracle 区分"vendor x509-parser unused_qualifications" → UNKNOWN

oracle 抓 `unused_qualifications` lint + `vendor/x509-parser` 路径模式 → UNKNOWN。短期内做。

更彻底（推荐）：修 `vendor/x509-parser/src/lib.rs` 把 `unused_qualifications` 从 `deny` 改 `allow`，或者整体改 `#![allow(warnings)]` —— 这是 vendor 一次性修改而非每工具都加 oracle 规则。我们 vendor 该 crate 本就是为了去 cargo 拉它（不可重现性），可以直接 patch lint level。**这是最小代码修改 → 一次性修 10 FP**。

### §5.5 关于 prusti viper_tools / opam switch 类环境损坏

prusti 14621 run 已经修过 viper_tools 路径错误。在 oracle 端可以加防御：

- 检测 stderr 有 `Caused by: java.lang.ClassNotFoundException` / `viperserver` 路径不存在 / `opam: SWITCH_DIR not found` 等环境损坏信号 → 改判 **UNKNOWN**，不判 FAILED。

这是历史遗留的 prusti viper_tools 损坏类事件的"再发生防御"。

---

## §6 与 prusti viper_tools 事件的关系

**prusti viper_tools 事件回顾**：旧 run 中 prusti 全 161 FAILED，源头是 prusti wrapper 的 `SCRIPT_DIR` 算路径错，导致 viper_tools 不被加载，Java backend 启动不了，每个 entry 都 fail-spawn。oracle 看到所有 entry 都 FAILED，但用户的判断是"工具没有任何能力被测到，应当判 UNKNOWN 不是 FAILED"。

**本次审计揭示的同源系统性 gap**：

oracle 当前的设计是"凡是工具非 0 exit / oracle 看到 fail-marker 即判 FAILED，且不论根因"。这违反宪法 §六 0 误报硬指标的精神：**"工具能力问题"应当狭义指 "工具内部 pipeline 在 entry 上 reject / partial / 自披露不支持"**，不应包含：

- runner harness 设计与 entry 签名不匹配
- 工具 pipeline 上游（cargo / 锁死 cargo / 老 cargo 不支持新 edition）
- vendor crate 在工具 toolchain 下编译失败
- 工具环境损坏（viper_tools / opam switch / Java JVM 不可用）
- 工具不通过 cargo 解决 deps，因此外部 crate unresolved

**审计中发现的 252 个 FP（22.4% 误报率）**完全是上述五类的具体实例。prusti viper_tools 事件只是 oracle 没正确区分"环境损坏"vs"工具能力" 的特例 — 实际全 testsuite 中此类 oracle "把外部根因当工具拒绝"的系统性 gap 还很广。

**这次审计与 prusti 修复同源**：都是"oracle 对 FAILED 根因不分层"的具体表现。修复路径（§5）大致两条线：

1. **oracle 加规则识别 5 类常见外部根因 → UNKNOWN**（runnable harness E0061 / unresolved import + has-deps / prusti cargo-2024 / vendor x509 lint / java-viper-not-found / kmir K-stuck-from-multi-exec 等）
2. **runner / vendor 一次性修源**（runnable 改 lib-mode；x509-parser allow lint）

只做 oracle 兜底就能立即修 252 个 FP 中的 **134 (runnable) + 101 (unresolved-import) + 7 (prusti-2024) + 10 (x509-vendor) = 全部 252 个** —— 让整套 testsuite 的"真 FAILED 数"从虚高 1124 降到实际 872，工具能力的 cc-reports / 内部讨论才有清洁的数据基础。
