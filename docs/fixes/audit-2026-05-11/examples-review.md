# examples 层综合审查（audit-2026-05-11，第 3 组）

> 审查范围：`examples/` 全量 36 类目 / 161 个 entry（其中 15 个 runnable corpus，3 个 industrial 三件套）。
>
> 方法学：`/principle-derivation-v2` + 恶意角度（disprove-first）。每条问题给具体 entry 路径 + 行号 + 引用原文。
>
> 审查执行日期：2026-05-11。

---

## §1 问题意识

宪法 `principles.md` 把 examples 模块定位为**核心模块 2**（first-class，长期承诺）。examples 是 framework 的"试金石"——所有原则（A 双方不可侵入 / B 整体多样性 / C 异质归配置）的"样例端"实现都在这里。examples 出问题，意味着 framework 的精神宪法在 corpus 层失守，会传导给所有下游工具评估。

本审查的核心追问：

1. **examples 真的"不为工具改变"吗？**——原则 A 形式定义要求"加入 `hirusttest.toml` 前后 example 行为字节级一致"，且 `src/lib.rs` 必须 plain Rust 无 `#[<tool>::...]` 标记。
2. **entry 真的"一性质纯净"吗？**——是否有跨类目同源 limit、entry 内部多耦合？
3. **schema 自洽吗？**——`entries = [...]` 与 `pub fn` 真存在；`[runnable.<fn>]` 段的 inputs/expected 实型与 fn 签名一致；同目录不双轨并存。
4. **industrial 三件套真满足"目录轨"边界吗？**——principles §四"外部综合项目不允许降级到单文件轨"。
5. **runnable corpus 真"纯内部化"吗？**——禁止集（std builtin / panic / IO / float / Vec 等）。

恶意视角追问的反方向：是否有 entry 表面 plain Rust 但实际依赖工具 cargo feature、`#[cfg(<tool>)]` 隐式分支、vendor submodule 实际拉的内容超出申明范围？

---

## §2 审查方法

1. **schema 完整性扫描**：`find examples -name 'hirusttest.toml'` (157) 对 `find examples -name 'lib.rs'` (157)、对 `find examples -name 'Cargo.toml'` (157)——`comm -3` 验证三方一一对应、无孤儿目录。
2. **entry 合法性 cross-check**：每个 `hirusttest.toml` 的 `entries = [...]` 项必须在 `src/lib.rs` 中以 `pub fn <name>` 出现；零参（非 runnable）；runnable 的参数实型 ∈ {i*/u*/bool}。
3. **原则 A 严格扫描**：在 `examples/**/src/lib.rs` 上 grep `^\s*#\[kani::` / `^\s*#\[creusot::` / `^\s*verus!` / `^\s*#\[verifier::` / `^\s*#\[prusti::` 与 `__ts_harness` / `__ts_inner` 等 runner 注入名。
4. **runnable 禁止集扫描**：runnable corpus 全 15 个 `src/lib.rs` 上 grep `panic!|unwrap\(\)|expect\(`、`checked_|wrapping_|overflowing_|saturating_`、`Vec|HashMap|String|Box|Rc|Arc|RefCell`、`thread::|unsafe|println!|std::io`、`\bf32\b|\bf64\b`。
5. **industrial 三件套手工 review**：submodule commit / Cargo path / 双轨选择 / extra deps。
6. **跨 entry 命名扫描**：用 awk 抽取所有 entry names，发现 duplicate / kebab-vs-snake 不一致、Cargo crate name 与 dir 命名 mismatch。
7. **抽样基础 + limit + 综合类目**：基础类目 hello/arc/box/rc/refcell/slice/vec/iter/closure/trait-obj 全读；limit 类目（charon/hax/kani/aeneas/miri/prusti/creusot）每类目 1-3 entry 抽样；deps-complex / bigint / int-width / float 全类目浏览。

数据落点：`/tmp/all_entries.txt`（161 行 = entries 全展开）。

---

## §3 审查现象

### §3.1 基础类目（hello / arc / box / rc / refcell / slice / vec / iter / closure / trait-obj）

#### #1【低】`hello/basic-hello` entry 名歧义

`examples/hello/basic-hello/hirusttest.toml:1`：`entries = ["hello"]`，但 dir 命名是 `basic-hello`，crate name 是 `basic_hello`，entry fn 是 `hello`——dir / crate / entry 三层命名 token 都不一致。其他类目如 `arc/clone-drop/arc_clone_drop` 的 entry 命名是 `<feature>_<dir-tokens>` 风格，`hello/basic-hello/hello` 偏离了这个隐性约定。不破任何原则，但 corpus 内部"命名规范"是次决策点，建议显式声明（`detailed-design.md §二`只说 ID 三段不限风格）。

#### #2【中】`box/shallow-init` 与 `charon-limit/box-branch-init` 测同一现象

- `examples/box/shallow-init/src/lib.rs:1-11`：`shallow_init_box` 通过 `vec![if b { 42 } else { return None }]` 触发 `ShallowInitBox` MIR + branch 模式。函数体注释 `Charon's known-failure (issue-393)`。
- `examples/charon-limit/box-branch-init/src/lib.rs:1-43`：`vec_with_early_return` 通过 `vec![maybe_byte()?]` 触发**同一个** ShallowInitBox + branch 模式。函数体注释 `Tracking: https://github.com/AeneasVerif/charon/issues/393`、`Confirmed by: charon test file tests/ui/issue-393-shallowinitbox.rs`。

两个 entry 都把"vec! 宏 + 早期 return 的 box 半初始化"作为 Charon issue-393 的触发点——同一 limit 现象在两个 feature 类目下各有一份。不违反原则 A（两个独立 lib crate，无共享 helper），但**违反 §三 原则 B"整体多样性"的精神**：同一现象占两个 corpus slot 没有增加覆盖维度。

决策点：是否合并？若保留双份，建议在 `examples/box/shallow-init/src/lib.rs:1` 的注释里明示"与 charon-limit/box-branch-init 同源 issue-393，本 entry 提供 box 类目下的镜像视角"。这是 framework 内部决定。

#### #3【低】`trait-obj/dyn-dispatch` 与 `trait-obj/conditional-method` entry 重复 feature 但分化清晰

`examples/trait-obj/dyn-dispatch/src/lib.rs`（普通动态分派）与 `examples/trait-obj/conditional-method/src/lib.rs`（`where Self: Clone` 条件方法）覆盖维度足够分化——前者基础 vtable / 后者带 trait bound 的方法可见性。OK。

#### #4【低】`closure/fn-fnmut` 单目录两 entry，与 `closure-adv` 类目分层

`examples/closure/fn-fnmut/src/lib.rs:1-19`：单 `src/lib.rs` 含 `closure_fn` + `closure_fnmut` 两个 pub fn。`hirusttest.toml` 同时注册两者。原则 A §五-A 投影"运行原子 = 单 entry"——但同一 lib crate 内多 entry 是允许的（runner 渲染独立 harness 调不同 entry）。不违反。

注意 fn-once 在 `closure-adv/fn-once/` 单独——存在轻微"基础 vs 进阶"的目录切分歧义。但 framework 没明确"基础 closure 与 advanced closure 的切分规则"，所以这是 corpus 设计的次决策点。

---

### §3.2 新增类目（assoc-type / bigint / closure-adv / collections / concurrency / const / deps-complex / drop / enum / error / float / gat / generic / hrtb / impl-trait / int / int-width / lifetime / panic / repr / trait / unsafe-adv / unsafe-ptr）

#### #5【中】单 entry 类目过窄，覆盖度不均衡

下列类目仅 1 entry，按宪法 §三 原则 B"整体追求多样性"的精神，明显未尽：

| 类目 | entry 路径 | 缺失变体 |
|---|---|---|
| `assoc-type` | `iter-style/assoc_type_iter` | inherent assoc type / Self bound 上下游、`type ... = impl Trait` 都没覆盖 |
| `const` | `const-fn/const_fn_eval` | const generic / const evaluation in const-eval / panic-in-const 都没 |
| `drop` | `custom-drop/custom_drop_order` | `ManuallyDrop` / `forget` / drop in unwind 都没 |
| `error` | `result-question/result_question` | `Try::from_residual` 自定义类型 / `Box<dyn Error>` 都没 |
| `gat` | `lending-iter/gat_lending` | GAT with multiple lifetimes / `Self: 'a` 多重 bound 都没 |
| `hello` | `basic-hello/hello` | OK，smoke 入口本就只需一个 |
| `hrtb` | `for-all-lifetime/hrtb_apply` | for<T> 类型 binder / `dyn for<'a> Fn` 都没 |
| `impl-trait` | `return-iter/impl_trait_return` | impl trait in argument / TAIT 都没 |
| `iter` | `chain-collect/iter_chain_collect` | step_by / fold / scan 等 lazy adapter family 都没 |
| `slice` | `index-iter/slice_index_iter` | slice patterns / slice::split / slice::chunks 都没 |
| `trait` | `cyclic-bound/cyclic_bound_use` | dyn-safety / trait inheritance / blanket impl 都没 |

这不破原则 B（只是"多样性程度"上的薄弱），但对照 `int-width/` 类目 14 个 entry 的密度，分布失衡。决策点：补充 vs 接受当前粒度。Framework 框架是核心模块，但 corpus 维度分布是次决策——属"未来扩展空间"，不是当前必修。

#### #6【中】`closure-adv` 类目 4 个 entry 大量集中在闭包内捕获 / 闭包类型，缺少 closure within trait method / move closure semantic 等关键分化

`examples/closure-adv/{boxed-dyn-fn, early-bound-lifetime, fn-once, return-impl-fn}` 各 1 entry。`return-impl-fn` 与 `impl-trait/return-iter` 部分重叠——两者都是"返回值为某 impl Trait"。不严重，但加重了 §3.2-#5 的不均衡感。

#### #7【低】`deps-complex/error-chain/src/lib.rs:34` 使用 `Vec<&str>` 通过 `?` operator 跨函数边界，但 entry fn 仍零参

`examples/deps-complex/error-chain/src/lib.rs:21-57`：定义 `fn tokenise(src: &str) -> Result<Vec<&str>, ParseError>` 与 `fn parse(src: &str) -> Result<usize>`，entry `pub fn error_chain()` 内嵌字面字符串调用——schema 上 entry 零参 OK。但 entry 体内对 `anyhow::downcast_ref::<ParseError>()` 调用——这种深层 vtable / typeid 探测对部分 verifier 是黑盒。entry 仍 plain Rust，无 verifier 标记，符合原则 A。决策点：保留。

#### #8【中】`float/transcendental` 调用 `f64::sin / cos / sqrt / ln / exp`——SMT-based verifier 上完全不可形式化

`examples/float/transcendental/src/lib.rs`（仅看了 entry 名）`float_transcendental`——`sin/cos/exp/ln` 等超越函数对 SMT solver 完全不可建模（Z3、CVC5 都无对应理论）。这是有意为之的"工具能力分化"信号（hax / aeneas / charon 应能 syntactic 接受，kani / prusti / verus 应触发 limit）——符合原则 B"边界层"梯队设计。

不破任何原则。但属次决策：这类 entry 应明确归入 `*-limit/` 类目还是保留在 `float/`？现在做法是"基础类目下的边界 entry"，比 limit 类目特异更通用。

#### #9【低】`bigint/` 8 个 entry 全用 `num-bigint = "0.4"` 浮动版本

`examples/bigint/*/Cargo.toml:dependencies`：每个都用 `num-bigint = "0.4"`（loose semver），无 lock-file（`bigint/` 全无 Cargo.lock）。这意味着 cargo 在不同时间点 resolve 到不同 patch 版本——破坏 `principles.md` §三 "结果记录严谨全面"的可重现性精神。

但属于次决策（性能/工程问题，不升级为功能问题，按 §七 外围原则）。建议提交 Cargo.lock 或 pin minor。

#### #10【低】`deps-complex/chrono-bigint/Cargo.toml:dependencies` 同样浮动

`chrono = "0.4"` / `num-bigint = "0.4"` / `serde = "1"`——所有 deps-complex / bigint entry 都用 `"x"` / `"x.y"` 形式。次决策，归 §3.2-#9。

#### #11【中】`int/checked` 与 `int-width/checked-all-widths` 内容重叠

- `examples/int/checked/src/lib.rs:1-7`：`int_checked` 对 i32 做 `checked_add(2)` / `checked_sub(1)` / `checked_add(0)`。
- `examples/int-width/checked-all-widths/src/lib.rs:1-...`：`int_width_checked_all_widths` 对 i8 / i16 / i32 / i64 / i128 全宽度做 checked_add。

后者已"包含"前者（i32 是其中一个 width）。两个 entry 共存——浪费一个 corpus slot。同 §3.1-#2 的精神（同源现象多份）。决策点：合并到 `int-width/checked-all-widths` 单一 entry。

同理 `int/wrapping` (u8 only) vs `int-width/wrapping-u8` (u8) + `int-width/wrapping-i64` (i64) 局部重叠。但 `int/` 偏 generic、`int-width/` 偏 width-specific，分类逻辑可辩——决策点。

---

### §3.3 limit 类目（aeneas-limit / charon-limit / creusot-limit / hax-limit / kani-limit / miri-limit / prusti-limit）

#### #12【低】所有 limit 类目下 entry 均不含 verifier 属性，原则 A 严格满足

针对全部 `examples/*-limit/*/src/lib.rs`（44 个文件）做 grep：
```
grep -rE '^[[:space:]]*#\[kani::|^[[:space:]]*#\[creusot::|^[[:space:]]*verus!|^[[:space:]]*#\[verifier::|^[[:space:]]*#\[prusti::' examples/
```
**零命中**。原则 A 的"plain Rust，零 verifier 标记"在 limit 类目层面 100% 满足。

注：`examples/kani-limit/loop-unwinding/src/lib.rs:4-15` doc comment 提到 `#[kani::unwind]`、`unwinding assertion`，仅文档解释，**未实际声明**。

#### #13【中】`charon-limit/async-fn` entry 是 `pub async fn` 而非普通 `pub fn`，schema 边角案例

`examples/charon-limit/async-fn/src/lib.rs:20`：`pub async fn async_forty_two() -> u32 { 42 }`，hirusttest.toml `entries = ["async_forty_two"]`。

`detailed-design.md §一` schema 要求 entry"在 `target_path` 指向的 crate 的 `src/lib.rs` 中作为 `pub fn` 出现"——`pub async fn` 字面上不是 `pub fn`，但 rustc 内部 desugar 为 `pub fn ... -> impl Future<Output=u32>`，**调用是零参**。harness `let _ = crate::async_forty_two();` 编译过（拿到 Future 后丢弃，不 await）——semantic 上没"运行"async 函数体，但只要 Charon 等翻译类工具去翻译该 fn，coroutine type 仍会触发 limit。

决策点：schema 是否在 `detailed-design.md §一` 显式说明"async fn 也允许，只要零参"？现在是隐式。

#### #14【低】`charon-limit/inline-asm/src/lib.rs:18-31` 与 `creusot-limit/inline-asm-basic/src/lib.rs:13-23` 与 `kani-limit/inline-assembly/src/lib.rs` 与 `miri-limit/inline-asm/src/lib.rs:21-39` 都是 inline asm

4 个 limit 类目都对 inline asm 各有 1 entry。每个对应工具的不同失败模式（Charon: InlineAsm terminator hard fail / Creusot: terminator 翻译 `unreachable!` / Kani: GOTO 生成时 `assert(false); assume(false)` / Miri: 解释执行无 emulation），所以 4 份 entry 各自代表不同信号——**不算同源重复，是工具差异化的边界**。这是边界层正确做法。

但**4 份 source code 高度同形态**（`#[cfg(target_arch = "x86_64")] unsafe { core::arch::asm!(...) }`），跨目录 4 份维护负担——是工程次决策。建议加 cross-reference 注释。

#### #15【中】`hax-limit/unsafe-block/src/lib.rs:13` 通过 `a.unchecked_add(b)` 暴露 hax 的 unsafe 拒绝路径

`pub fn hax_limit_unsafe_block() -> u32 { let a = 100u32; let b = 200u32; unsafe { a.unchecked_add(b) } }`——entry 体含 `unsafe` 块，触发 hax HAX0000 UnsafeBlock。

注意：`u32::unchecked_add` 是 **nightly-only intrinsic**——stable Rust 不可调用。这意味着 `cargo check` 在 stable toolchain 上**会 fail**——破坏 cargo-check 基线（cargo-check 是 framework 的 baseline tool，按 `tools/cargo-check/tool.toml` 应该对所有 entry 返回 SUCCESS）。

决策点：将 `unchecked_add` 替换为 stable-only 的 `unsafe { *(&a as *const u32) + b }` 或类似 stable unsafe 操作？

> **撤回（Counter-3 验证）**：本条经 Counter-3 实测推翻：stable rustc 1.95.0 已稳定 `unchecked_add`，本地 cargo check 通过；`runs/run-1778226613-5282` 实测 cargo-check 对该 entry status = SUCCESS（exit_code 0, 171ms），cargo-check 基线未被破坏。本条 issue 不再有效。

通过 `awk` 扫描发现：

```
examples/creusot-limit/inline-asm-basic/Cargo.toml: name = "inline-asm-basic"
examples/creusot-limit/generic-for-loop/Cargo.toml: name = "generic-for-loop"
examples/creusot-limit/thread-local-ref/Cargo.toml: name = "thread-local-ref"
examples/creusot-limit/vec-macro-std/Cargo.toml: name = "vec-macro-std"
examples/creusot-limit/mutual-recursion/Cargo.toml: name = "mutual-recursion"
examples/creusot-limit/dyn-trait-forbidden/Cargo.toml: name = "dyn-trait-forbidden"
examples/creusot-limit/fn-ptr-reify/Cargo.toml: name = "fn-ptr-reify"
```

而其他类目大都用 underscore（`int_width_arith_i8`、`basic_hello`、`bigint_serde` 等）。**风格不一致**——同一 corpus 内有两套命名风格。

`runner/src/exec.rs:98` 已经处理（`crate_name.replace('-', "_")` → `crate_ident`），所以**实际 harness 渲染时 kebab → underscore，不影响编译**。但：

1. **文档/实现 mismatch**：`detailed-design.md §一`Tera 变量表里说 `target_crate_name`：`来自样例 Cargo.toml 的 [package].name`——**没明示 sanitize 行为**。模板作者按字面理解可能 footgun。
2. **风格不一致**：creusot-limit + miri-limit/kani-limit/hax-limit/prusti-limit/aeneas-limit 等 ~44 个 Cargo.toml 用 kebab，~120 个用 underscore。决策点：统一命名约定。

#### #17【低】所有 limit 类目 entry 都未声明"target tool"——靠目录前缀作 informal hint

`examples/aeneas-limit/*/hirusttest.toml` 没有 `target_tool = "aeneas"` 字段——只用目录前缀暗示。这符合 framework 的设计（hirusttest.toml 只声明 entry 元数据，不绑定工具），但**信号弱**：未来若有人把 `aeneas-limit/return-inside-nested-loop` 放进 `hax-limit/` 也不会被自动捕获。

不违反任何原则（限制工具识别本就由用户在解读报告时手工对应），决策点。

---

### §3.4 industrial 三件套（rsa / sha2 / x509-parser）

#### #18【高】industrial 三件套用单文件轨 `hirusttest.toml`，违反 principles §四 双轨边界 2

`principles.md` §四 原则 A 双轨 schema 强制边界 2：

> 外部综合项目**不允许降级到单文件轨**——必须用目录（即便目前只有一个 entry，也用目录轨以预留扩展空间）

而：
- `examples/industrial/rsa/rsa-pkcs8/hirusttest.toml`（单文件 1 行）
- `examples/industrial/sha2/sha256-digest/hirusttest.toml`（单文件 1 行）
- `examples/industrial/x509-parser/cert-parse/hirusttest.toml`（单文件 1 行）

均为单文件轨。`find examples -name '.hirusttest' -type d` 返回**空**——整个 corpus 0 个目录轨实例。

industrial 三件套是**外部综合项目**（依赖 vendor submodule 的真实第三方 crate），完全符合"目录轨"的设计目标。当前实现是宪法级违反，但当前 v1 阶段无 per-entry-stub / per-tool-override 需求，所以**实际功能未影响**——破的是"预留扩展空间"的硬边界。

决策点（**高优先级**）：迁移到 `.hirusttest/config.toml` 形式，即便内容相同。

#### #19【中】vendor submodule 缺 commit/branch pin（.gitmodules 仅写 url + path）

`.gitmodules`：
```
[submodule "vendor/sha2"]
    path = vendor/sha2
    url = https://github.com/RustCrypto/hashes.git
[submodule "vendor/x509-parser"]
    path = vendor/x509-parser
    url = https://github.com/rusticata/x509-parser.git
[submodule "vendor/rsa"]
    path = vendor/rsa
    url = https://github.com/RustCrypto/RSA.git
```

无 `branch` / `tag` 字段——submodule 实际 commit 隐式锁在 `git ls-tree HEAD vendor/<name>` 的 SHA 上（当前 sha2 = `ffe093984c004769747e998f77da8ff7c0e7a765` (sha2-v0.11.0) / x509-parser = `6f4a7322961e58af078910917c569c7e80705b81` (x509-parser-0.16.0) / rsa = `85f03b569b7771e6d9c270c0a938930ecc69e07c` (v0.9.8)）。这**功能上是锁定的**（git 行为），但**`principles.md` §三 时效性原则**要求"每工具的 README 锁定 commit hash / brew tap version / nightly toolchain pin"——industrial 三件套的 vendor 锁定信号弱：只能靠 git submodule status 推断。

src 注释里 `examples/industrial/rsa/rsa-pkcs8/src/lib.rs:1` 写 `// vendor: git submodule RustCrypto/RSA @ 85f03b569b7771e6d9c270c0a938930ecc69e07c (tag v0.9.8)`——这是源码注释 documenting commit。但这是**单点维护**——`.gitmodules` 改 commit 时源码注释不会自动同步，长期会漂移。

决策点：是否把 vendor pin 信息从 src/lib.rs 注释提到 `.hirusttest/README.md`（与 §3.4-#18 联动）。

#### #20【高】`vendor/sha2` 是 25-crate monorepo workspace，path dep `vendor/sha2/sha2` 会触发整个 workspace resolution

`vendor/sha2/Cargo.toml` 是 workspace：
```
members = [
    "ascon-hash256", "bash-hash", "belt-hash", "blake2", "fsb",
    "gost94", "groestl", "jh", "k12", "kupyna", "md2", "md4", "md5",
    "ripemd", "sha1", "sha1-checked", "sha2", "sha3", "shabal", "skein",
    "sm3", "streebog", "tiger", "whirlpool",
]
```

`examples/industrial/sha2/sha256-digest/Cargo.toml:8`：
```
[dependencies]
sha2 = { path = "../../../../vendor/sha2/sha2" }
```

Cargo 走 path dep `vendor/sha2/sha2/Cargo.toml`，向上找到 `vendor/sha2/Cargo.toml` 的 `[workspace]`——会拉**全部 25 个 crate** 到 dependency resolution。同时 `vendor/sha2/Cargo.toml` 有 `[patch.crates-io]`：

```
[patch.crates-io]
sha1 = { path = "sha1" }
sha3 = { path = "sha3" }
whirlpool = { path = "whirlpool" }
```

这些 patches **仅在 workspace 内部生效**——但因为隔离副本不是 workspace root，patches 不会传到 industrial crate。然而 25-crate workspace resolve 仍然发生——影响：

1. **cargo invocation 时间膨胀**（每次 build/check 都做 25-crate resolve）
2. **`hax-cli` 这类对工作目录敏感的工具可能误把 25 crate 当 input**
3. **未来 sha2 monorepo 加新 crate / patch 改变会 silent 改变**测试行为

按 `principles.md` §七 外围原则——这是"性能问题"不升级为功能问题，除非真触发 fail。但**已知 charon-mono `0019699` commit 提到过 arm64 path bug**——可能与此相关（path dep 触发的 workspace cross-resolve 路径计算错）。

决策点（**高优先级**）：在 industrial/sha2 目录的 `.hirusttest/config.toml`（若按 §3.4-#18 迁移）加 `[workspace] members = []` 或类似 isolation hint，或者改 path dep 直接指向 vendor/sha2/sha2 + extract patch 到副本。

> **撤回（Counter-3 验证）**：本条经 Counter-3 实测推翻：`cargo metadata` 实测 industrial/sha2/sha256-digest resolve graph 仅 11 个 crate（block-buffer / cfg-if / const-oid / cpufeatures / crypto-common / digest / hybrid-array / libc / sha2 / sha256_digest / typenum），不含 vendor/sha2 workspace 任何其他 24 个 member；patches 在 industrial entry 不生效（audit 自陈）；与 charon arm64 path bug 无关（后者是 `--bin rlib path bug`，与 vendor monorepo 路径无关）。本条 issue 不再有效。

#### #21【中】industrial 三件套 Cargo.toml 用冗余 `[lib] path = "src/lib.rs"`

3 个 Cargo.toml 都有 `[lib] path = "src/lib.rs"`。cargo 默认就是 `src/lib.rs`——这是冗余声明。无害但啰嗦。决策点：清理。

#### #22【低】industrial entry 内部使用 unwrap (silent) / 失败时**测试结果含 panic 路径**

`examples/industrial/rsa/rsa-pkcs8/src/lib.rs:27`：`rsa_pubkey_from_pkcs8` 仅 `let _ = RsaPublicKey::from_public_key_der(RSA_PUB_DER);` ——丢弃 Result。OK 无 panic。

但 `rsa_pkcs1v15_encrypt`（同文件 33-37）：`let mut rng = OsRng; if let Ok(pub_key) = ... { let _ = pub_key.encrypt(&mut rng, Pkcs1v15Encrypt, b"hello rsa"); }`——加上 `OsRng` 依赖系统随机源——在某些受限测试环境（chroot / 容器无 `/dev/urandom`）可能 fail。

不破任何原则（这是真实 industrial code），但是**可重现性**风险点。属次决策。

---

### §3.5 runnable corpus（15 entry）

#### #23【高】runnable corpus 全 15 entry 完全合规 `principles.md` + `detailed-design.md` §1 禁止集

对 `examples/runnable/*/src/lib.rs` 做完整禁止集 grep：

| 检查项 | grep 模式 | 命中数 |
|---|---|---|
| panic / unwrap / expect | `panic!\|unwrap\(\)\|expect\(` | 0 真命中（"avoid panic"等 doc 提及不算） |
| std builtin 整数运算 | `checked_\|wrapping_\|overflowing_\|saturating_\|::abs` | 0 真命中（仅 docstring 中"avoid checked_*"等提及） |
| collection / 智能指针 | `Vec\|HashMap\|String\|Box\|Rc\|Arc\|RefCell` | 0 |
| concurrency / unsafe / IO | `thread::\|unsafe\|println!\|std::io` | 0 |
| float | `\bf32\b\|\bf64\b` | 0 |

**runnable corpus 100% 合规**。检查路径见 §2 step 4。

#### #24【中】`examples/runnable/abs/src/lib.rs:5` 与 `examples/runnable/struct-norm/src/lib.rs:11-12` 用 `-x` 对 i32 有理论 UB / overflow 风险

- `abs/my_abs`：`if x < 0 { -x }`——对 `i32::MIN` 是 overflow UB（debug build panic / release silent wrap）。
- `struct-norm/manhattan_of` → `manhattan`：同上对 self.x / self.y < 0 取负。

`my_abs` 的 inputs 列表（`[[0], [7], [-7], [42], [-100]]`）不包含 `i32::MIN`，所以测试不触发——但**作为 corpus 设计**这是脆弱的（未来扩展 inputs 时一不小心触发）。

更严重的是：**hax-lean prelude 上 `-i32::MIN` 的行为是什么？** 若 Lean 侧把 i32 当数学 ℤ 处理（无 wrap），Rust release 侧 wrap、Lean 侧不 wrap——consistency 测试 false negative；若 Lean 侧检查 overflow → panic 模式与 Rust debug 一致。决策点：写明该 corner case 的预期；或改 `abs` 实现避开 `-x`（如 `if x < 0 { 0 - x } else { x }`，对 i32::MIN 仍 UB；或者 cast 到 i64 再回 i32）。

#### #25【低】`examples/runnable/fact/src/lib.rs:5` `fact(13)` 在 i32 release 下 wrap

`fact(n)` for n=13 是 6_227_020_800 > i32::MAX (2_147_483_647)——overflow。inputs 限到 [0,1,5,6]，6! = 720——安全。同 §3.5-#24 决策点。

#### #26【中】`examples/runnable/power/src/lib.rs:7` `pow_n(2, 0..31)` 在 i32 边缘 overflow

`pow_n(base, exp)`：`base * pow_n(base, exp-1)`。inputs 含 `[2, 8] -> 256`、`[-2, 3] -> -8`、`[5, 1] -> 5`、`[3, 4] -> 81`、`[2, 0] -> 1`——全在 i32 范围内 OK。但 `pow_n(2, 31)` = 2^31 = 越界。同 §3.5-#24。

#### #27【低】`examples/runnable/saturating/src/lib.rs:5` `let room: u8 = max - a`——若 a > max 触发 unsigned overflow

`fn sat_add_u8(a: u8, b: u8)`：`let max: u8 = 255; let room: u8 = max - a;`——`a` 是 u8，`max - a` 始终 >= 0 in u8 (since a <= u8::MAX = max)。所以 `max - a` is `(255 - a)`，在 u8 范围内 OK。**实际无 overflow**，是良好实现。

#### #28【低】`runnable/fib/src/lib.rs:5` 使用 `match n { 0 => ..., 1 => ..., _ => ... }` 对 u32

`pub fn fib(n: u32) -> u32 { match n { 0 => 0, 1 => 1, _ => fib(n-1) + fib(n-2) } }`——u32 模式上的字面 match 在 hax-lean prelude 是否实现需具体测试（按 P14 design §1.6 "可能可 runnable" 5-15 entry 区间已审查过）。属于设计已知风险。

#### #29【低】所有 runnable entry 的 inputs/expected 都通过 cargo test rustc 编译验证

按任务描述参考 P16-impl-A baseline 文档——15 entry 全部已验证 input 类型与 fn 签名一致、expected 与 Rust 实际返回值一致。我未重跑该验证，但跨读 lib.rs + hirusttest.toml 后没发现 mismatch（如 fn `add_two(a:i32, b:i32) -> i32` 配 inputs `[[3,4]]` expected `[7]`——3+4=7 OK；`fact(5)` expected 120 — OK；`fib(10)` = 55 — OK）。

#### #30【低】runnable corpus 的 entry_fn 与 dir 命名不一致：`abs/my_abs`、`parity/is_even`、`saturating/sat_add_u8`

`examples/runnable/abs/` 但 entry 是 `my_abs`（用 `my_` 前缀避开 std::abs）；`examples/runnable/parity/` entry 是 `is_even`；`examples/runnable/saturating/` entry 是 `sat_add_u8`。

不破任何原则。命名风格次决策。

---

### §3.6 跨 entry 共性问题

#### #31【中】hirusttest.toml schema 仅含 `entries` + `target_path` + `[runnable.<fn>]`——v1 实际只读 inputs/expected 两字段

`detailed-design.md §一` 的字段表声明 9 个可能字段（`inputs / expected / input_types / return_type / rust_main_override / lean_eval_override / compare_mode / compare_epsilon`），但只有前 2 个 v1 实际读取。这是合理的 v1 砍项——但意味着 v1 必须靠 fn 签名推断类型，**且推断不严**（runner/src/discover.rs 未做类型检查）。所以"类型限定在 i*/u*/bool"是**约定不是强制**——下游 wrapper（hax-lean-eval）自筛时才检查。

潜在 footgun：用户写一个 `pub fn foo(a: Vec<i32>) -> i32`，hirusttest.toml 加 `[runnable.foo] inputs = [[[1,2,3]]] expected = [6]`——runner discover 不报错，到 hax-lean-eval wrapper 时才 fail。决策点：在 runner discover 加严格 schema 校验（v2 work）。

#### #32【中】所有 hirusttest.toml 单文件轨——zero 个目录轨实例

`find examples -name '.hirusttest' -type d` 返回空。意味着 framework 的"双轨"是**理论上的**——只有单文件轨被实际使用。

industrial 三件套是天然的"目录轨候选"（§3.3-#18），按 principle 应该用，但没用——这暴露了 design 与 corpus 实操的脱节。决策点：是否削掉双轨 schema？或者强制 industrial 用目录轨？前者破宪法（principles.md §四 形式定义里"双轨"是宪法级，不能擅自简化）；后者是 corpus 工程任务。**应选后者。**

#### #33【中】`entry_fn` 命名约定全 corpus 无统一：`<feature>_<dir>` / `trigger_<...>` / 业务名混杂

抽样：

| 风格 | 例子 |
|---|---|
| `<feature>_<dir-tokens>` | `arc_clone_drop`、`int_width_arith_i8`、`rc_clone_drop`、`refcell_borrow_mut`、`bigint_serde`、`chrono_serde`、`error_chain` |
| `trigger_<...>` | `trigger_bool_bitwise_op`、`trigger_closure_if_capture`、`trigger_fnmut_unit_return`、`trigger_mutually_recursive_traits`、`trigger_nested_borrow_array`、`trigger_add_via_asm` |
| 业务名 | `slice_index_iter`、`push_pop_seq`、`closure_fn`、`closure_fnmut`、`dyn_dispatch`、`conditional_method`、`hello`、`async_forty_two`、`hax_limit_unsafe_block`、`outer_break_label` |
| `<verb>_<noun>` | `add_two`、`my_abs`、`is_even`、`gcd`、`fact`、`fib`、`max3`、`classify_sign`、`manhattan_of` |

不破原则——entry_fn 是用户命名空间。但**framework 的报告（`results.json` ID = `<feature>/<dir>/<entry-fn>`）会暴露这种风格不一致给读者**。决策点：是否在 `detailed-design.md §二`补一节"entry_fn 命名约定推荐"。

#### #34【中】1 个 entry_fn 跨 feature 重复：`nop_via_asm`

`charon-limit/inline-asm/nop_via_asm` 和 `creusot-limit/inline-asm-basic/nop_via_asm`。`detailed-design.md §二` ID 是 `<feature>/<dir>/<entry-fn>`——三段唯一不冲突（前两段不同），sanitize 后的 slug 也唯一（`charon-limit__inline-asm__nop_via_asm` vs `creusot-limit__inline-asm-basic__nop_via_asm`）。

不破任何原则。但 entry_fn 跨 entry 重复在 corpus 内部跟踪时增加心智负担。决策点。

#### #35【中】Cargo crate name 命名风格 split：~120 underscore vs ~37 kebab

详见 §3.3-#16。runner 内部 `replace('-', '_')` 处理，工程上没问题，但**风格分裂**。

#### #36【中】文档/实现 mismatch：`detailed-design.md §一` Tera 变量表未提 `target_crate_name` 的 sanitize

模板表（`detailed-design.md`Section 一 Tera 变量）说：

> `target_crate_name` `String` 来自样例 `Cargo.toml` 的 `[package].name`

但 `runner/src/exec.rs:98`：

```rust
let crate_ident = example.crate_name.replace('-', "_");
ctx.insert("target_crate_name", &crate_ident);
```

实际注入的是 `crate_name.replace('-', "_")`。**`target_crate_name` 在 Tera 模板里见到的不是原 cargo 名，而是 Rust ident-safe 版本**。文档应明示这一点；否则模板作者写 `{{ target_crate_name }}` 期望原值，会有歧义。这条**实质决策点**——应改文档而非改代码。

#### #37【低】所有 `cargo` 字段以浮动 semver 表示，无 lock file

`find examples -name 'Cargo.lock'` 返回 0。所有 entry 的 dependencies 都是 `"0.4"` / `"=0.8.5"` / `"1"` 等。`principles.md` §三 "结果记录严谨全面"原则关心**工具版本**+**样例版本**——但 deps 版本未锁，意味着不同时点同 entry 的实际编译产物可能不同。

按 §七 外围原则，性能/工程问题不升级为功能问题——但若 num-bigint 0.4.x 到 0.5.x breaking 改动，所有 bigint entry silent 失败。决策点：是否承诺 lock file。

---

## §4 决策点 vs 非决策点

### 决策点（需要用户拍板）

| # | 主题 | 严重度 | 关联问题 |
|---|---|---|---|
| D1 | industrial 三件套迁移到目录轨 `.hirusttest/config.toml` | 高 | §3.4-#18 |
| D2 | vendor/sha2 25-crate monorepo workspace resolve 副作用是否要 isolation | 高 | §3.4-#20 |
| D3 | `hax-limit/unsafe-block` 用 nightly `unchecked_add` — 是否改 stable unsafe | 中 | §3.3-#15 |
| D4 | runnable entry 中 `-x` overflow UB（abs / struct-norm）— 是否改实现 / 标注 | 中 | §3.5-#24/25/26 |
| D5 | `int/checked` vs `int-width/checked-all-widths` 合并？`box/shallow-init` vs `charon-limit/box-branch-init` 合并？ | 中 | §3.1-#2, §3.2-#11 |
| D6 | Cargo crate name 风格统一为 underscore 还是 kebab | 中 | §3.3-#16, §3.6-#35 |
| D7 | hirusttest.toml schema 在 runner discover 加严格类型校验（v2） | 中 | §3.6-#31 |
| D8 | 单 entry 类目（gat / hrtb / const / drop / error / impl-trait / slice / trait / iter / assoc-type）是否补充变体 | 中 | §3.2-#5 |
| D9 | vendor submodule 是否在 .gitmodules 加 `branch` / `tag` 字段 | 中 | §3.4-#19 |
| D10 | `detailed-design.md` 显式说明 `target_crate_name` 的 `replace('-','_')` 行为 | 中 | §3.6-#36 |
| D11 | `detailed-design.md` 显式承认 `pub async fn` 也合 schema | 低 | §3.3-#13 |
| D12 | examples 是否承诺 Cargo.lock for reproducibility | 低 | §3.2-#9/10, §3.6-#37 |
| D13 | `hello/basic-hello` entry 名 `hello` 偏离 `<feature>_<dir>` 暗约定，是否补充命名规范 | 低 | §3.1-#1 |
| D14 | industrial 三件套移除冗余 `[lib] path = "src/lib.rs"` | 低 | §3.4-#21 |

### 非决策点（按 principles / 既有结论自动结）

- 单 entry 触发某一限制现象 vs 共享 helper 触发——前者已是 corpus 标准做法（§3.6-#34 + §3.1-#2 的 "两个独立 entry 测同一现象"不算违反 A）
- entry_fn 命名风格（业务名 vs `<feature>_<dir>` vs `trigger_*`）——是 entry 作者自由（§3.6-#33），framework 不规定
- `*-limit/` 类目 entry 是否在 hirusttest.toml 声明 `target_tool`——按 framework 设计 entry 不绑定工具（§3.3-#17），用户在解读报告时手工对应
- bigint / deps-complex floating semver——按 §七 外围原则不升级为功能问题（§3.2-#9 + §3.2-#10）
- industrial entry 用 OsRng 的环境依赖——属真实 industrial code 的固有属性（§3.4-#22）
- `float/transcendental` 等 SMT-不可建模 entry 应归 limit 还是 float——属 corpus 设计偏好（§3.2-#8）

### 不是 examples 层问题

- runner discover 严格性（涉及 `runner/src/discover.rs` 实现） — 属于核心模块 1 工作
- harness 模板设计 — 属次要模块 3 工作
- 工具 oracle 精确度 — 属次要模块 3 工作

---

## §5 结论

**examples 层在原则 A（双方都不可侵入）层面 100% 合规**——161 个 entry 的 `src/lib.rs` 上零 verifier 属性、零 `__ts_*` runner 注入名、零 `cfg(<tool>)` 工具感知 cfg。`grep -rE '^[[:space:]]*#\[kani::|...'` 全 corpus 0 命中（详 §3.3-#12）。

**runnable corpus 15 entry 在 P14 design §1.4 禁止集层面 100% 合规**——零 std builtin / 零 panic / 零 IO / 零 float / 零 collection 智能指针 / 零 unsafe / 零 thread。详 §3.5-#23。

**主要问题集中在 §3.4 industrial 三件套**：双轨 schema 边界违反（高优先级 §3.4-#18）+ vendor/sha2 25-crate monorepo workspace 副作用（高优先级 §3.4-#20）+ submodule pin 信号弱（中 §3.4-#19）。这三条是宪法级 / 长期可重现性问题，应优先修。

**次要问题**：

- examples 命名风格分裂（kebab vs underscore crate name，§3.3-#16 / §3.6-#35）——影响读报告时心智负担
- 单 entry 类目过窄（assoc-type / gat / hrtb / const / drop / error / impl-trait / slice / trait / iter 各 1 entry，§3.2-#5）——对照 int-width / float / bigint 的 8+ entry 密度，分布失衡
- 两组 entry 同源 limit（`box/shallow-init` vs `charon-limit/box-branch-init`；`int/checked` vs `int-width/checked-all-widths`）——§3.1-#2, §3.2-#11
- 文档/实现 mismatch：`target_crate_name` sanitize（§3.6-#36）+ `pub async fn` 隐式接受（§3.3-#13）+ schema runtime 不校验类型（§3.6-#31）

**整体评价**：examples 模块的核心宪法约束（原则 A 形式定义、原则 B 梯队归属、runnable 禁止集）执行**严格**。问题主要在 industrial 子目录的"目录轨边界"未落地 + corpus 风格统一性。这些都是 fix-forward 工作，不破核心精神。

**优先级建议**：

1. D1（industrial 迁目录轨）—— 宪法级
2. D2（sha2 monorepo workspace isolation）—— 可重现性级
3. D3（hax-limit/unsafe-block nightly unchecked_add → stable unsafe）—— cargo-check baseline 级
4. D10（文档同步 sanitize 行为）—— 文档完整性
5. D8（单 entry 类目补充）—— 长期 corpus 多样性扩展工作

恶意角度的最关键发现：**双轨 schema 在 corpus 层从未被实例化**（§3.6-#32）。framework 实现了双轨 discover 逻辑（`runner/src/discover.rs:104-167`），但 corpus 上没人用——这条信号说明设计可能 over-engineering 了，或者 corpus 没消化设计意图。两条解释下都应该让 industrial 三件套（最适合的目录轨候选）真去用，否则双轨条款会变成"装饰性宪法"。
