# verus config Review

## §1 问题意识

verus 是矩阵中 SMT-based 验证工具——前端是 verus! 宏块内的 Rust 子集检查 + VIR 构造，后端是 AIR / SMT-LIB / Z3。oracle 用 `--no-verify` 切到 VIR 构造后停下。entry_mode = "lib" 必需（verus 要求 crate top-level 含 `use vstd::prelude::*`）。

恶意角度考察（按宪法 §六-4 反作弊推论）：
1. harness 是否真把 `mod __ts_inner` 放进 `verus! {}` 块内？若放外，Verus 把 inner 直接透传 rustc——SUCCESS 退化为 cargo-check；
2. `--no-verify` 是否切到正确边界（VIR 构造 vs AIR vs Z3）？
3. cargo-verus 不可用 + 直接调 `verus` binary 的选择是否合规？
4. extra_cargo_deps 为何无 vstd？

## §2 审查方法

参照源：

- [`docs/design/principles.md`](../../../design/principles.md) §六-1 / §六-4 反作弊推论；
- [`tools/verus/tool.toml`](../../../../tools/verus/tool.toml)；
- [`tools/verus/harness.rs.tera`](../../../../tools/verus/harness.rs.tera)；
- [`deep-reports/cc-reports/verus.md`](../../../../deep-reports/cc-reports/verus.md)；
- verus release `0.2026.05.03`。

## §3 审查现象

#1 (严重度: 高) — harness `mod __ts_inner` **在** `verus! {}` 块内——反作弊关键不变量

**现象**：[`tools/verus/harness.rs.tera`](../../../../tools/verus/harness.rs.tera) L13-15：

```rust
verus! {
    mod __ts_inner;
    pub use __ts_inner::*;
    ...
}
```

**违反**：未违反——这是宪法 §六-4 反作弊推论的关键不变量。README L55 + L74 详细论证："如果把 `mod __ts_inner;` 写在 `verus! {}` 外，块内只剩个 `__ts_invoke` 壳，inner 的真实代码就**没经过 Verus 自身前端**，等价于 cargo-check"。

**推理链**：[`tools/verus/harness.rs.tera`](../../../../tools/verus/harness.rs.tera) L4-9 注释也强调："We put `mod __ts_inner` *inside* `verus! {}` so Verus actually processes the example's Rust through its own front-end... If `mod` is left outside, Verus passes the inner to stock rustc"。

**决策性**：决策点——已经做出了正确决策，但维护时必须保持。harness 改动必须保住此不变量。

**建议**：harness L4-9 的注释已经充分警告。也可在 tool.toml 加 `# DO NOT TOUCH: mod __ts_inner must be inside verus! {}` 注释。

#2 (严重度: 中) — `--no-verify` all-or-nothing 切——README 已诚实声明 AIR 构造也被切

**现象**：[`tools/verus/tool.toml`](../../../../tools/verus/tool.toml) L9-12：

```
# --no-verify all-or-nothing 同时切 AIR 与 Z3，无独立切线；最深前端
# 表示就是 VIR。AIR ≈ SMT-LIB 的前置形式，按精神 AIR 算后端，所以 VIR 即为
# 前端边界。
```

**违反**：未违反——按 [`principles.md`](../../../design/principles.md) §六-1 前端 / 后端切分，"AIR ≈ SMT-LIB 的前置形式，按精神 AIR 算后端"——所以 VIR 是前端边界。诚实声明 `--no-verify` 同时切 AIR + Z3。

**推理链**：合规。但与 kani `--only-codegen`（切到 GotoC 后停，是真正的前端边界）有差——kani 切到 MIR → GotoC codegen 之间；verus 切到 VIR 构造之后但 AIR 构造之前（其实是同时切 AIR + Z3）。

**决策性**：非决策点。

**建议**：无须改。

#3 (严重度: 中) — `--log vir` 落 VIR 产物——可补 entry_fn 存在性 grep 作为反作弊封堵

**现象**：[`tools/verus/tool.toml`](../../../../tools/verus/tool.toml) L17：`"--log", "vir"`。注释 L14-16："`--log vir`: 把 VIR 写到 .verus-log/crate.vir，便于函数级对应性校验...不加此 flag 时 --no-verify 不写产物，oracle 只能看 exit code，无法独立验证 VIR 是否真构造完成"。

**违反**：[`principles.md`](../../../design/principles.md) §六-4 反作弊推论——`--log vir` 是为函数级对应性预留接口，但当前 oracle 不查 VIR 产物。

**推理链**：可仿照 hax-coq / hax-fstar / rocq-of-rust 的 entry_fn 存在性 grep：grep `.verus-log/crate.vir` 中含 `(Function :name (Fun :path lib::<crate>::<entry_fn>) ... :body ...)`——README L20 已说明 VIR 产物形态。当前 oracle 仅看 exit code——未利用 `--log vir` 产物。

**决策性**：决策点——是否补 entry_fn VIR 存在性 grep。

**建议**：可选——加 wrapper 抓 `.verus-log/crate.vir` 中 `:name (Fun :path .*::<entry_fn>)`。当前 oracle 形式可证（README L30-32），但补强 belt-and-braces 防 verus 引入 silent skip 路径。

#4 (严重度: 中) — `entry_mode = "lib"` + harness `mod __ts_inner;` 注入——架构合规

**现象**：[`tools/verus/tool.toml`](../../../../tools/verus/tool.toml) L22 + [`tools/verus/harness.rs.tera`](../../../../tools/verus/harness.rs.tera)。

**违反**：未违反——按 [`architecture.md`](../../../design/architecture.md) §一-A 位阶澄清，"隔离副本上声明式工具特定填充"。runner 把 lib.rs 改名 __ts_inner.rs，harness 作 new lib.rs——合规。

**决策性**：非决策点。

**建议**：无须改。

#5 (严重度: 中) — `--crate-type=lib src/lib.rs` 直接调 verus binary，绕开 cargo-verus

**现象**：[`tools/verus/tool.toml`](../../../../tools/verus/tool.toml) L18-19 + 注释 L42-49：

> cargo-verus (the cargo subcommand) tries to re-compile vstd from crates.io through the verus binary as RUSTC_WRAPPER. As of the 0.2026.05.03 release binary, this panics...
> 必须绕开，直接调用 `verus` binary。

**违反**：未违反——上游 cargo-verus bug 的合理 workaround。README L80 也明示。

**推理链**：合规。

**决策性**：非决策点。

**建议**：无须改。

#6 (严重度: 中) — 无 extra_cargo_deps——verus binary 自带 vstd

**现象**：[`tools/verus/tool.toml`](../../../../tools/verus/tool.toml) 无 `extra_cargo_deps`。注释 L54-58："The verus binary bundles vstd; it does NOT use Cargo to resolve vstd."

**违反**：未违反——verus binary 自带 vstd，无需注入 cargo dep。与 creusot 必须注入 `creusot-std` 不同。

**决策性**：非决策点。

**建议**：无须改。

#7 (严重度: 低) — version_command 用 `verus --version`

**现象**：[`tools/verus/tool.toml`](../../../../tools/verus/tool.toml) L23-26：

```toml
version_command = [
  "${TS_VERUS_BIN}",
  "--version",
]
```

**违反**：未违反——按 [`tool-integration.md`](../../../design/tool-integration.md) §一。

**决策性**：非决策点。

**建议**：无须改。

#8 (严重度: 低) — timeout_secs = 120 偏短

**现象**：[`tools/verus/tool.toml`](../../../../tools/verus/tool.toml) L21：`timeout_secs = 120`。

**违反**：未违反——`--no-verify` 切到 VIR 构造后停，无 SMT 求解，120s 充足。

**决策性**：非决策点。

**建议**：无须改。

## §4 决策点 vs 非决策点

- 决策点：2（VIR entry_fn 存在性 grep 补强 / harness 不变量警告）
- 非决策点：6

## §5 审查结论

verus 配置体现了对宪法 §六-4 反作弊推论的深刻理解——harness `mod __ts_inner` **在** `verus! {}` 块内是最关键的不变量，README 与 harness comment 都充分警告。无 critical 问题。

最值得补强（非必须）：

1. **VIR entry_fn 存在性 grep**：基于 `--log vir` 产物，加 wrapper 抓 entry_fn 在 `.verus-log/crate.vir` 中以 `(Function :name (Fun :path .*::<entry_fn>))` 出现——belt-and-braces 防 verus 上游引入 silent skip-item 路径；
2. **更醒目 DO NOT TOUCH 警告**：harness 中 `mod __ts_inner` 位置不变量——若被新维护者改到块外，oracle silent 退化为 cargo-check。

整体属于"经反作弊 audit 的稳态配置"。
