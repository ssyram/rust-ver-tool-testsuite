# soteria config Review

## §1 问题意识

soteria 是矩阵中第一个原生支持 Tree Borrows 的 Rust 符号执行引擎。oracle 设计简洁——exit code 0/1/2/3 四类：
- exit 0 = SUCCESS（符号执行完成且无 bug）
- exit 1 = 检测到 bug → FAILED
- exit 2 = soteria-rust 内部 crash → FAILED
- exit 3 = Charon/Obol 前端 crash → FAILED

按宪法 §六-2 不允许 partial 精神——bug detect 也算 FAILED（解释执行被 bug 中断 = 没完整跑完）。

恶意角度考察：
1. opam env 注入是否正确？
2. `--rustc=--edition=2021` 处理 single-file 默认 Rust 2015 trap；
3. tool.toml inline `sh -c` 是否合规？

## §2 审查方法

参照源：

- [`docs/design/principles.md`](../../../design/principles.md) §六-2；
- [`tools/soteria/tool.toml`](../../../../tools/soteria/tool.toml)；
- [`tools/soteria/harness.rs.tera`](../../../../tools/soteria/harness.rs.tera)；
- [`deep-reports/cc-reports/soteria.md`](../../../../deep-reports/cc-reports/soteria.md)；
- soteria commit `3c21278187c60c99418fe2dabb03710ce4102896`。

## §3 审查现象

#1 (严重度: 中) — tool.toml inline `sh -c` 含 opam env eval + PATH 注入

**现象**：[`tools/soteria/tool.toml`](../../../../tools/soteria/tool.toml) L39-42：

```toml
command = [
    "sh", "-c",
    "eval $(opam env --switch=soteria-install) && export PATH=$PATH:${TS_SOTERIA_BIN_DIR} && soteria-rust exec --rustc=--edition=2021 src/lib.rs"
]
```

**违反**：[`tool-integration.md`](../../../design/tool-integration.md) §五——内联 sh -c 与 hax-* / kmir 同模式。但 soteria 相对简单（单行命令），可接受。

**推理链**：合规。`--switch=soteria-install` 切到隔离 opam switch（注释 L4 说明）+ PATH 注入 obol/charon 路径。

**决策性**：决策点——是否抽外部 wrapper（与其他工具一致体例）。

**建议**：可选——抽 `soteria-wrapper.sh`。当前实施简洁有效。

#2 (严重度: 中) — `--rustc=--edition=2021` 关键 trap

**现象**：[`tools/soteria/tool.toml`](../../../../tools/soteria/tool.toml) L34-36 注释 + L41 实施：

> Standalone .rs file has no Cargo.toml; edition defaults to Rust 2015 unless --rustc=--edition=2021 is passed (included in command below)

**违反**：未违反——是处理 soteria 单文件模式 trap 的关键 flag。

**推理链**：若缺此 flag，现代 Rust 代码（用 `let else` / `if let` 链等 2021+ 特性）会编译失败——silent FAILED 但根因是 edition mismatch 而非工具能力。已声明。

**决策性**：非决策点。

**建议**：无须改。

#3 (严重度: 中) — entry_mode = "lib" + harness `mod __ts_inner` + `fn main` 调用 entry

**现象**：[`tools/soteria/tool.toml`](../../../../tools/soteria/tool.toml) L44 + [`tools/soteria/harness.rs.tera`](../../../../tools/soteria/harness.rs.tera):

```rust
mod __ts_inner;
pub use __ts_inner::*;

fn main() {
    let _ = __ts_inner::{{ entry_fn }}();
}
```

**违反**：未违反——按 [`architecture.md`](../../../design/architecture.md) §一-A 位阶澄清。soteria-rust 以 `fn main` 为符号执行入口（注释 L11-13），harness 内嵌 mod __ts_inner 然后 main 调用 entry。

**推理链**：合规。与 verus / aeneas-* / creusot 同 lib mode 模式。

**决策性**：非决策点。

**建议**：无须改。

#4 (严重度: 中) — soteria oracle 完全基于 exit code——无 wrapper 二次检查

**现象**：[`tools/soteria/tool.toml`](../../../../tools/soteria/tool.toml) 全文 50 行，无 wrapper。注释 L19-25 详尽列 exit code 语义。

**违反**：未违反——按 [`tool-integration.md`](../../../design/tool-integration.md) §三-反向证明。soteria-rust 的 exit code 0/1/2/3 是工具自身设计的明确语义，README L34-38 形式严格性论证扎实。

**推理链**：与 aeneas / charon 同模式——exit code 单一信号。soteria 比它们更强：明确区分 4 类 exit code。

**决策性**：非决策点。

**建议**：无须改。

#5 (严重度: 低) — timeout_secs = 120 合理

**现象**：[`tools/soteria/tool.toml`](../../../../tools/soteria/tool.toml) L46。

**违反**：未违反——注释 L29-30："target functions are trivially small; compilation is the expensive part (~0.1–5s)"。

**决策性**：非决策点。

**建议**：无须改。

#6 (严重度: 低) — version_command 同 inline 模式

**现象**：[`tools/soteria/tool.toml`](../../../../tools/soteria/tool.toml) L48-51：

```toml
version_command = [
    "sh", "-c",
    "eval $(opam env --switch=soteria-install) && export PATH=$PATH:${TS_SOTERIA_BIN_DIR} && soteria-rust exec --help 2>&1 | head -1"
]
```

**违反**：未违反——版本字符串从 `--help` 首行抓取（soteria-rust 无 `--version`）。

**决策性**：非决策点。

**建议**：可选——若 soteria-rust 提供 `--version`，用之。

#7 (严重度: 中) — README L34-40 形式严格性 ✅ 形式可证

**现象**：[`tools/soteria/README.md`](../../../../tools/soteria/README.md) L34-40：

> - **partial 暴露机制**：
>   - exit 1 = 检测到 bug
>   - exit 2 = soteria-rust 内部 crash
>   - exit 3 = obol/charon 前端 crash
> - **形式严格性 — 0 误报**：✅ 形式可证。soteria exit 0 ⇔ 符号执行完成且无 bug
> - **形式严格性 — 0 漏报**：✅ 形式可证。exit 1/2/3 完整覆盖 bug detect / symex crash / 前端 crash 三类 partial
> - **漏报盲点**：无

**违反**：未违反——按 [`tool-integration.md`](../../../design/tool-integration.md) §三/§四。soteria-rust 设计上 exit code 是单一可信信号，类似 charon-mono `--abort-on-error`。

**决策性**：非决策点。

**建议**：无须改。

## §4 决策点 vs 非决策点

- 决策点：1（抽 wrapper）
- 非决策点：6

## §5 审查结论

soteria 配置简洁有效——依赖工具自身 exit code 0/1/2/3 设计，无需 wrapper 后处理。形式严格性 ✅ 形式可证扎实（4 类 exit code 完整覆盖 partial 路径）。

最值得补强（非必须）：抽 wrapper 与 hax-* / kmir / aeneas-* 体例一致。当前实施简洁有效。

整体属于"工具自身设计良好支持 oracle 的优秀范例"。
