# charon-poly readme Review

## §1 问题意识

charon-poly README 重点：(1) 多态模式与单态模式（charon-mono）差异；(2) `--abort-on-error` 反 silent path 论据；(3) macOS arm64 workaround。结构与 charon-mono 同形。

## §2 审查方法

参照源：

- [`docs/fixes/audit-2026-05-11/tools/charon-mono/readme-review.md`](../charon-mono/readme-review.md) shared issues；
- [`tools/charon-poly/README.md`](../../../../tools/charon-poly/README.md)；
- [`deep-reports/cc-reports/charon-poly.md`](../../../../deep-reports/cc-reports/charon-poly.md)。

## §3 审查现象

#1 (严重度: 低) — README 8 章节齐全，体例与 charon-mono 一致

**现象**：[`tools/charon-poly/README.md`](../../../../tools/charon-poly/README.md) 结构完整。

**违反**：未违反。

**决策性**：非决策点。

**建议**：无须改。

#2 (严重度: 低) — README L26-30 形式严格性论证扎实

**现象**：[`tools/charon-poly/README.md`](../../../../tools/charon-poly/README.md) L26-30 与 charon-mono L26-30 形式严格性论证字面相同（共享 `--abort-on-error` + `register_error!` 单一通路）。

**违反**：未违反。

**决策性**：非决策点。

**建议**：无须改。

#3 (严重度: 低) — README L45 / L56 `--abort-on-error` 论据 + macOS arm64 workaround 诚实

**现象**：[`tools/charon-poly/README.md`](../../../../tools/charon-poly/README.md) L45-47 / L55-58 与 charon-mono 体例一致。

**违反**：未违反。

**决策性**：非决策点。

**建议**：无须改。

#4 (严重度: 低) — 章节"标题格式 SUCCESS 信号(严格反映前端特性支持范围）" 中文括号未关闭

**现象**：[`tools/charon-poly/README.md`](../../../../tools/charon-poly/README.md) L22：

```
### SUCCESS 信号(严格反映前端特性支持范围）
```

**违反**：低优先 typo——左括号是英文 `(`，右括号是中文 `）`。比较 charon-mono README L22：`### SUCCESS 信号（严格反映前端特性支持范围）`——两边都是中文括号。

**推理链**：纯排版瑕疵，不影响内容。

**决策性**：非决策点。

**建议**：可选——把 `(` 改为 `（` 与 charon-mono 对齐。

#5 (严重度: 低) — 关联 sub-tests 与 charon-mono 共享

**现象**：[`tools/charon-poly/README.md`](../../../../tools/charon-poly/README.md) L61 与 charon-mono L62 文字一致。

**违反**：未违反。

**决策性**：非决策点。

**建议**：无须改。

## §4 决策点 vs 非决策点

- 决策点：0
- 非决策点：5

## §5 审查结论

charon-poly README 是 charon-mono 的标准复用，差异仅在描述多态模式特征（"保留泛型参数" / "翻译代价相对低"）。论证质量与 charon-mono 同等。

唯一可调整：L22 标题中英文括号 typo——不影响内容。

**无 critical 问题**。
