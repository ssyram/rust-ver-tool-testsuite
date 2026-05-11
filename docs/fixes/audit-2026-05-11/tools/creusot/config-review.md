# creusot config Review

## §1 问题意识

creusot 是矩阵中第一个真正"侵入"样例的工具——cargo-creusot 要求样例 lib 自身 `use creusot_std::prelude::*;` 且 Cargo.toml 字面含 `creusot-std` dep。这是宪法 §四-A 位阶澄清的关键案例：原始磁盘字面零修改，隔离副本上声明式工具特定填充（`extra_cargo_deps` + `entry_mode = "lib"`）。

oracle 设计：`cargo-creusot` 默认不带 subcommand 即只翻译到 `.coma`，不调用 why3——形式可证 0 误报 / 0 漏报（`crash_and_error / span_err / span_fatal` 单一通路）。

恶意角度考察：
1. extra_cargo_deps 注入是否真在隔离副本上做（不污染原始磁盘）？
2. cargo-creusot 默认行为（无 subcommand 即只翻译）是否稳定？
3. timeout 900s 与 cargo-creusot 启动开销匹配？

## §2 审查方法

参照源：

- [`docs/design/principles.md`](../../../design/principles.md) §四-A 位阶澄清、§六-1 / §六-2；
- [`docs/design/architecture.md`](../../../design/architecture.md) §一-A 位阶澄清的具体落地；
- [`tools/creusot/tool.toml`](../../../../tools/creusot/tool.toml)；
- [`tools/creusot/harness.rs.tera`](../../../../tools/creusot/harness.rs.tera)；
- [`runner/src/exec.rs`](../../../../runner/src/exec.rs) L82-87 patch_cargo_deps；
- [`deep-reports/cc-reports/creusot.md`](../../../../deep-reports/cc-reports/creusot.md)；
- cargo-creusot 0.11.0。

## §3 审查现象

#1 (严重度: 高) — extra_cargo_deps + entry_mode = "lib" 实现宪法 §四-A 位阶澄清

**现象**：[`tools/creusot/tool.toml`](../../../../tools/creusot/tool.toml) L3-4：

```toml
extra_cargo_deps = ['creusot-std = "0.11.0"']
entry_mode       = "lib"
```

[`runner/src/exec.rs`](../../../../runner/src/exec.rs) L83-86 patch_cargo_deps 把 dep line inject 到副本 Cargo.toml（不污染原始）。

**违反**：未违反——按 [`architecture.md`](../../../design/architecture.md) §一-A 位阶澄清"原始磁盘字面零修改 + 隔离副本上声明式工具特定填充"。这是 creusot 必需的两个机制。

**推理链**：creusot 是矩阵中**最严苛验证宪法 §四-A 位阶澄清**的工具——cargo-creusot 在 cargo 解析阶段就检查 `creusot-std` dep（README L57），缺失则拒；creusot-rustc 对顶级 module 检查 `creusot_std` import（README L57），仅在 harness（bin）中 `use` 不够。所以**必须** extra_cargo_deps + entry_mode = "lib" 两机制并用。

**决策性**：非决策点——是 creusot 接入的必要机制。

**建议**：无须改。

#2 (严重度: 中) — `command = ["${TS_CARGO_CREUSOT}"]` 单元素 argv——默认无 subcommand

**现象**：[`tools/creusot/tool.toml`](../../../../tools/creusot/tool.toml) L1：

```toml
command = ["${TS_CARGO_CREUSOT}"]
```

[`tools/creusot/README.md`](../../../../tools/creusot/README.md) L22：

> `cargo-creusot` **默认无 subcommand 时即只翻译，不进入 prove**——所以现行 tool.toml 已经天然停在前端边界。

**违反**：未违反——按 [`principles.md`](../../../design/principles.md) §六-1 前端 / 后端切分。cargo-creusot 默认行为停在前端（翻译到 .coma），需 `prove` subcommand 才进 why3 SMT。**这是 creusot 上游设计的自然切线**，无需额外 flag。

**推理链**：与 charon-mono `--abort-on-error` / kani `--only-codegen` 同性质——上游提供的精确切线。creusot 这条最干净（无 flag 需要）。

**决策性**：非决策点。

**建议**：无须改。

#3 (严重度: 中) — `cargo-creusot` 启动孙子进程 (why3, alt-ergo, cvc)——但 prove subcommand 才启动

**现象**：[`tools/creusot/tool.toml`](../../../../tools/creusot/tool.toml) L2：`timeout_secs = 900`。

[`tools/creusot/README.md`](../../../../tools/creusot/README.md) L49：

> **timeout**：900 秒（Why3 + SMT solver 验证耗时，且会启动孙子进程 why3 / alt-ergo / cvc；runner 用 `kill(-pgid)` 杀整个进程组）

**违反**：未违反。但 README L49 提到"Why3 + SMT solver 验证耗时"——这是 prove subcommand 行为，与当前"默认翻译"不一致。

**推理链**：默认无 subcommand 时不调 why3 / alt-ergo / cvc，timeout 900s 偏宽——但 cargo-creusot 启动 + cargo 编译 + creusot-rustc 翻译每个 fn 仍可能秒级到分钟级（对复杂 lib）。900s 是上限保护。

**决策性**：非决策点——README L49 描述偏旧（针对 prove subcommand），但 timeout 设置仍合理。

**建议**：可选——更新 README L49 措辞"timeout：900 秒——cargo-creusot 默认翻译阶段开销较低，但 cargo 编译 + creusot-rustc 翻译多 fn 时可能耗时"。

#4 (严重度: 中) — harness `extern crate creusot_std; use creusot_std::prelude::*;` 在 mod __ts_inner 之外

**现象**：[`tools/creusot/harness.rs.tera`](../../../../tools/creusot/harness.rs.tera):

```rust
extern crate creusot_std;
use creusot_std::prelude::*;

mod __ts_inner;
pub use __ts_inner::*;

#[allow(dead_code)]
fn __ts_invoke() {
    __ts_inner::{{ entry_fn }}();
}
```

**违反**：未违反——但与 verus harness 不同（verus 把 mod __ts_inner 放进 `verus! {}` 块内）。creusot 不需要把 inner 放在某宏块内——creusot-rustc 作为 RUSTC 替换处理所有 fn item。

**推理链**：creusot 的 RUSTC 替换机制让 inner 自动受 creusot 处理。无 verus 那样的"块内 vs 块外" 反作弊不变量。

**决策性**：非决策点。

**建议**：无须改。

#5 (严重度: 低) — README L31-34 形式严格性 ✅ 形式可证

**现象**：[`tools/creusot/README.md`](../../../../tools/creusot/README.md) L31-34：

> - **partial 暴露机制**：creusot 用 `crash_and_error / span_err / span_fatal / dcx().span_err` 把任何 unsupported 升级为 rustc error → exit 101
> - **形式严格性 — 0 误报**：✅ 形式可证。cargo-creusot exit 0 ⇔ 翻译完整无 rustc error
> - **形式严格性 — 0 漏报**：✅ 形式可证。creusot 用 `crash_and_error / span_err / span_fatal` 把所有 unsupported 升级为 rustc error，无 silent path

**违反**：未违反——按 [`tool-integration.md`](../../../design/tool-integration.md) §四-4.1 形式证明。与 aeneas / charon 同模式——单一通路。

**推理链**：合规。但 README 引用 4 个 API（crash_and_error / span_err / span_fatal / dcx().span_err）——未给具体源码行号。可补强。

**决策性**：非决策点。

**建议**：可选——补 creusot 源码行号锚定（如 `creusot/src/translation.rs:XXX span_err(...)`）。

#6 (严重度: 低) — version_command 用 `sh -c "cargo creusot version"`

**现象**：[`tools/creusot/tool.toml`](../../../../tools/creusot/tool.toml) L5：

```toml
version_command = ["sh", "-c", "cargo creusot version"]
```

**违反**：未违反——`cargo creusot version` 是 cargo-creusot 子命令。

**决策性**：非决策点。

**建议**：可选——直接 `["${TS_CARGO_CREUSOT}", "version"]` 不用 shell 包装。当前实施 work。

## §4 决策点 vs 非决策点

- 决策点：0
- 非决策点：6

## §5 审查结论

creusot 配置**是宪法 §四-A 位阶澄清的关键案例**：

- extra_cargo_deps + entry_mode = "lib" 是隔离副本上的声明式工具特定填充——原始磁盘字面零修改；
- 形式严格性 ✅ 形式可证扎实（crash_and_error / span_err / span_fatal 单一通路）；
- cargo-creusot 默认无 subcommand 即只翻译——上游设计的天然切线。

无 critical 问题；整体属于"宪法 A 位阶澄清的接入典范"。
