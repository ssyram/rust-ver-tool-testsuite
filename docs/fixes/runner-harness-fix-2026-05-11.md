# Runner Harness Argument-Mismatch Fix (Task Y, 2026-05-11)

## 1. 现象与根因

`docs/fixes/false-positive-audit-2026-05-11.md` §2.1 / §4.1 列出全 matrix 中 134 个误报来自同一机制：

- runnable corpus 的 15 个 entry 全部为带参 `pub fn`（如 `add_two(a: i32, b: i32) -> i32`、`gcd(a: i64, b: i64) -> i64`）；
- 但 runner 在 bin-mode 渲染的 harness 模板写死 `runnable_<crate>::<entry_fn>()`（零参调用）；
- 结果 9 个工具（cargo-check / kani / miri / kmir / prusti / verifast / creusot / soteria / verus）的 harness 编译阶段一律 `error[E0061]: this function takes N arguments but 0 arguments were supplied`，被 oracle 全数记成 FAILED；
- 这 134 个 FAILED 的 oracle 语义本应是"工具拒绝该 Rust 特性"，但实际工具的 type-check / encoder / interpreter 根本没机会跑——是 runner harness 设计与 entry 签名不匹配，按 `principles.md` §六 "0 误报硬指标"应当算误报。

## 2. 修复方案

修最少必要面：

### 2.1 `runner/src/discover.rs`

1. `HirusttestToml` 加 `runnable: HashMap<String, RunnableSpec>` 字段（`#[serde(default)]`，无段时空 map）；
2. `RunnableSpec { inputs: Vec<toml::Value>, expected: Option<toml::Value> }`，仅消费 `inputs[0]`；
3. 新 helper `render_runnable_row_as_rust_args` 把 `inputs[0]` 的 toml 数组（v1 类型矩阵：`i*/u*/bool`）渲染成 Rust 实参逗号串（如 `[3, 4]` → `"3, 4"`）；
4. `Example` 加 `entry_args: HashMap<String, String>`，discover 阶段为每个有 `[runnable.<entry>]` 段的 entry 填一行。

### 2.2 `runner/src/exec.rs`

Tera context 多注入一个变量：

```rust
let entry_args = example.entry_args.get(entry).map(String::as_str).unwrap_or("");
ctx.insert("entry_args", entry_args);
```

非 runnable entry 取空串，模板渲染成 `entry_fn()`——与修复前完全等价（向后兼容 146 个 base entry）。

### 2.3 各 bin-mode + lib-mode-with-invoke harness 模板

全部把 `{{ entry_fn }}()` 改为 `let _ = {{ entry_fn }}({{ entry_args }});`：

- `tools/cargo-check/harness.rs.tera`
- `tools/charon-mono/harness.rs.tera`
- `tools/charon-poly/harness.rs.tera`
- `tools/kani/harness.rs.tera`
- `tools/kmir/harness.rs.tera`
- `tools/miri/harness.rs.tera`
- `tools/prusti/harness.rs.tera`
- `tools/hax-coq/harness.rs.tera`
- `tools/hax-fstar/harness.rs.tera`
- `tools/hax-lean/harness.rs.tera`
- `tools/creusot/harness.rs.tera`（lib-mode，但 `__ts_invoke` 里调 entry）
- `tools/soteria/harness.rs.tera`（lib-mode，但 `fn main` 里调 entry）
- `tools/verus/harness.rs.tera`（lib-mode，但 `__ts_invoke` 里调 entry）

不动：
- `tools/aeneas-{coq,fstar,hol4,lean}/harness.rs.tera` 不调 entry，仅 `mod __ts_inner; pub use __ts_inner::*;`
- `tools/verifast/harness.rs.tera` 是空 `fn main() {}`（verifast 不用 harness）
- `tools/rocq-of-rust{,-typecheck}/harness.rs.tera` 是占位 `pub fn {{ entry_fn }}() -> u32 { 42 }`（写在 `src/bin/__ts_harness.rs`，与 lib 中真 entry 不冲突，且 rocq-of-rust pipeline 直接读 `src/lib.rs` 不读 harness）

`let _ = ...` 包裹是为消除工具特定的"unused result"警告——`fn(i32, i32) -> i32` 返回值在零参 fn 上是 `()`，加 let \_ 对两种情况都干净。

## 3. 实测验证

### 3.1 单 entry 验证

```bash
set -a && source .env && set +a
target/release/runner --tool cargo-check --entry 'runnable/add-two/add_two'
```

输出 `[SUCCESS] cargo-check runnable/add-two/add_two (60ms)`——修复前是 `error[E0061]`。

### 3.2 全 runnable corpus 验证

```bash
target/release/runner --tool cargo-check --entry 'runnable/*'
```

输出 `Total: 15 succeeded / 0 failed / 0 unknown / 15 total`——15/15 SUCCESS。

### 3.3 全 matrix 反误报验证

```bash
target/release/runner --tool cargo-check
```

输出 `Total: 161 succeeded / 0 failed / 0 unknown / 161 total`。

按 `false-positive-audit-2026-05-11.md` §2.5 表：cargo-check 真 FAILED = 0（cargo-check 是基准），15 个 runnable harness FP 应当全部修好 → 期望 161/161 SUCCESS。**实测吻合**。

### 3.4 其他工具的运行验证

`target/release/runner --tool prusti --entry 'runnable/add-two/add_two'` → SUCCESS。

## 4. 反误报论证

| 维度 | 防漏 | 反误 |
|---|---|---|
| runnable | 15/15 cargo-check SUCCESS 实测 | 仅在 entry 有 `[runnable.<fn>]` 段时填 entry_args；非 runnable entry 行为字节级不变 |
| base 146 entry | 161 全 SUCCESS（之前是 146 SUCCESS + 15 runnable FAILED） | `entry_args = ""` → `entry_fn()` 与修复前完全相同 |
| Cargo 编译 | `let _ = ...` 不破坏返回非 `()` 的合法 invocation | 不破坏返回 `()` 的合法 invocation |

## 5. 决策点 vs 非决策点（按 `principles.md` §八）

**非决策点**：
- 模板变量名 `entry_args`（命名偏好）
- `let _ = ...` 包裹（消除工具警告的细节）
- HashMap vs 其他 map 类型（实现细节）
- v1 类型矩阵 = 整数 + bool；其他类型 → discover 阶段 Err（与 detailed-design.md §一已有约束一致）

**决策点候选**：
- 是否要在 `detailed-design.md` §一中改"已知代价"段——原文写"runnable 在 zero-arg 工具上 FAILED 是已知代价"，本次修复后不再是。需要用户确认是否进入文档修订。本任务**不动 detailed-design.md**，仅在此 fix 文档登记需要后续处理。

## 6. 影响范围

- 改：`runner/src/{discover,exec}.rs`、13 个 harness.rs.tera
- 不动：`docs/design/*`、`examples/*`、其他工具配置
- 不 commit（按任务指示）
