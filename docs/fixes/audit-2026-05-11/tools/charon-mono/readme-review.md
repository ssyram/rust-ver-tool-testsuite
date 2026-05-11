# charon-mono readme Review

## §1 问题意识

charon-mono README 重点：(1) 单态化模式与多态模式（charon-poly）的差异；(2) `--abort-on-error` 反 silent path 论据；(3) macOS arm64 workaround 诚实声明；(4) `Box<dyn Any>` vtable drop preshim 实测 panic 案例（README L57 实测证据）。

## §2 审查方法

参照源：

- [`docs/design/tool-integration.md`](../../../design/tool-integration.md) §五 必含章节、§六 禁忌；
- [`tools/charon-mono/README.md`](../../../../tools/charon-mono/README.md)；
- [`deep-reports/cc-reports/charon-mono.md`](../../../../deep-reports/cc-reports/charon-mono.md)；
- [`tools/charon-poly/README.md`](../../../../tools/charon-poly/README.md) 体例对照。

## §3 审查现象

#1 (严重度: 低) — README 8 章节齐全

**现象**：[`tools/charon-mono/README.md`](../../../../tools/charon-mono/README.md) 完整。

**违反**：未违反。

**决策性**：非决策点。

**建议**：无须改。

#2 (严重度: 低) — README L26-30 形式严格性论证扎实

**现象**：[`tools/charon-mono/README.md`](../../../../tools/charon-mono/README.md) L26-30：

> **partial 暴露机制**：`--abort-on-error` 让 charon 内部任何 unsupported 项触发 panic + exit 1。`charon-driver/driver.rs:143` 设 `error_ctx.continue_on_failure = false`，`register_error!` 在第一次错误就 panic
> **形式严格性 — 0 误报**：✅ 形式可证。
> **形式严格性 — 0 漏报**：✅ 形式可证。`--abort-on-error` + `register_error!` panic 路径已封死所有 silent skip
> **漏报盲点**：无

**违反**：未违反——按 [`tool-integration.md`](../../../design/tool-integration.md) §三-源码层穷尽，论证扎实。

**推理链**：与 aeneas 同模式——单一 unsupported 入口（`register_error!`）+ 单一 exit 决定（`--abort-on-error` panic）。形式可证。

**决策性**：非决策点。

**建议**：无须改。

#3 (严重度: 中) — README L43 / L56 引用"实测 mono 模式下 `Box<dyn Any>` vtable drop preshim 触发 panic"——实测证据扎实

**现象**：[`tools/charon-mono/README.md`](../../../../tools/charon-mono/README.md) L46 / L56：

> `--abort-on-error`：同 poly——将 charon 内部 panic 暴露为非 0 exit（实测 mono 模式下 `Box<dyn Any>` vtable drop preshim 触发 panic，缺此 flag 会静默 exit 0）

**违反**：未违反——[`tool-integration.md`](../../../design/tool-integration.md) §四-4.2 双向实测验证。这条实测证据扎实——"缺此 flag silent exit 0" 是反向（命中 silent path 而 oracle 不抓），加 flag 后正向抓住。

**推理链**：合规——README 实测证据 = "Box<dyn Any> vtable drop preshim 触发 panic"——这是 charon 实施特定 silent path 的实测发现。诚实声明。

**决策性**：非决策点。

**建议**：无须改。

#4 (严重度: 低) — README L33-37 安装段共享 charon binary 与 charon-poly 一致

**现象**：[`tools/charon-mono/README.md`](../../../../tools/charon-mono/README.md) L35-37：

> 本测试基线：v0.1.184（commit `ed22146b`），与 charon-poly 共用同一可执行文件。

**违反**：未违反——按 [`tool-integration.md`](../../../design/tool-integration.md) §一 commit hash 锁定。

**推理链**：与 aeneas-* 4 个 backend 共享 binary 同模式。

**决策性**：非决策点。

**建议**：无须改。

#5 (严重度: 低) — README L53-58 已知限制 / 坑 描述准确

**现象**：[`tools/charon-mono/README.md`](../../../../tools/charon-mono/README.md) L55-58：

> - 单态化展开可能引发实例数量爆炸，对泛型使用密集的样例耗时显著高于 poly 模式
> - `--abort-on-error` 在 mono 模式下尤为关键：实测 `Box<dyn Any>` 的 vtable drop preshim 会触发 charon 内部 panic...
> - macOS arm64 同样必须加 `--lib --target aarch64-apple-darwin`
> - 高阶生命周期、部分 unsafe raw pointer 等超出 charon 翻译域，exit 非 0

**违反**：未违反——按 [`tool-integration.md`](../../../design/tool-integration.md) §五 (7) 已知限制 / 坑。

**推理链**：4 条限制都是工具自身行为或平台 workaround，合规。"超出 charon 翻译域，exit 非 0"是工具自陈而非测试结论，合规。

**决策性**：非决策点。

**建议**：无须改。

#6 (严重度: 低) — README L60-62 关联 sub-tests 与 charon-poly 共享

**现象**：[`tools/charon-mono/README.md`](../../../../tools/charon-mono/README.md) L62：

> `examples/charon-limit/` 是本工具自声明的限制集——这些 entry 故意触发 charon 的"不支持"特性，期望本工具在这些 entry 上 FAILED。

**违反**：未违反——两个 charon backend 共享 limit 类目。

**决策性**：非决策点。

**建议**：无须改。

## §4 决策点 vs 非决策点

- 决策点：0
- 非决策点：6

## §5 审查结论

charon-mono README 内容完整、形式严格性扎实，与 cc-report、wrapper / 配置实施一致。**无 critical 问题**。

整体属于"高质量集成"——与 charon-poly / aeneas family 同级。
