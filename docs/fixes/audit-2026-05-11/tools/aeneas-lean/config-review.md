# aeneas-lean config Review

## §1 问题意识

aeneas-lean 是 4 个 aeneas backend 之一（共享 binary，差 backend flag）。pipeline 两阶段：charon → .llbc → aeneas -backend lean → .lean。oracle 设计：exit 0 = SUCCESS；exit ≠ 0 = FAILED（partial 或 panic）。aeneas 内部用 `Errors.error_list` 单一信号 + `Main.ml:773 if has_errors then exit 1`——形式可证 0 误报 / 0 漏报。

恶意角度考察：
1. 两阶段 pipeline 任一阶段失败如何呈现？
2. `${TS_CHARON_BIN}` / `${TS_AENEAS_BIN}` 展开正确（避开 P12 expand_env trap）？
3. opam env 注入是否对所有 macOS 用户都能找到 zarith / gmp？
4. wrapper 与 README 形式严格性自陈一致？

## §2 审查方法

参照源：

- [`docs/design/principles.md`](../../../design/principles.md) §六-1 / §六-2；
- [`docs/design/tool-integration.md`](../../../design/tool-integration.md) §三 / §四；
- [`tools/aeneas-lean/aeneas-lean-wrapper.sh`](../../../../tools/aeneas-lean/aeneas-lean-wrapper.sh)；
- [`runner/src/discover.rs`](../../../../runner/src/discover.rs) `expand_env` 函数；
- [`docs/fixes/oracle-leak-audit-2-2026-05-11.md`](../../oracle-leak-audit-2-2026-05-11.md)；
- aeneas 上游 commit `a14083a6`。

## §3 审查现象

#1 (严重度: 中) — wrapper L13 `set -euo pipefail` + L35 `"$CHARON_BIN" cargo --preset=aeneas` —— charon 阶段失败会让 wrapper 在 `set -e` 下直接 exit，aeneas 阶段不启动

**现象**：[`tools/aeneas-lean/aeneas-lean-wrapper.sh`](../../../../tools/aeneas-lean/aeneas-lean-wrapper.sh) L13 + L35-36：

```bash
set -euo pipefail
...
"$CHARON_BIN" cargo --preset=aeneas
echo "[aeneas-lean-wrapper] charon exit: $?"
```

**违反**：未违反——按 [`tool-integration.md`](../../../design/tool-integration.md) charon 阶段失败时 wrapper exit ≠ 0 = FAILED，行为正确。但 `echo "[...] charon exit: $?"` 在 `set -e` 下永远只输出 `0`——因为非 0 会让脚本提前 exit，echo 这行不执行。诊断信号有缺。

**推理链**：若用户读 stdout 想看 charon exit code，会看到 `charon exit: 0` 或根本看不到这行（脚本已 exit）。echo 这行对诊断无用。

**决策性**：非决策点——但诊断质量有提升空间。

**建议**：可选：`"$CHARON_BIN" cargo --preset=aeneas || { rc=$?; echo "charon exit: $rc"; exit $rc; }` —— 让 charon exit code 显式输出。

#2 (严重度: 中) — wrapper L40-44 检查 `.llbc` 文件存在性——若 charon 退出 0 但产物路径不在 cwd 顶层会误判

**现象**：[`tools/aeneas-lean/aeneas-lean-wrapper.sh`](../../../../tools/aeneas-lean/aeneas-lean-wrapper.sh) L39-44：

```bash
LLBC_FILE="$(ls *.llbc 2>/dev/null | head -1)"
if [[ -z "$LLBC_FILE" ]]; then
    echo "[aeneas-lean-wrapper] ERROR: no .llbc file found after charon" >&2
    exit 1
fi
```

**违反**：未违反——charon 默认把 `<crate>.llbc` 写到 cwd 顶层（aeneas charon 的 `--preset=aeneas` 行为）。但如果未来 charon 改产物路径，wrapper silent FAILED 而非清晰错误。

**推理链**：当前实施合规。但 `ls *.llbc | head -1` 在多 llbc 时随机选——若副本目录残留之前的 .llbc，选错。但 runner per-execution cp 隔离，副本目录是 fresh，应该只有一个 .llbc。

**决策性**：非决策点。

**建议**：可选——加 sanity check `if (( $(ls *.llbc 2>/dev/null | wc -l) > 1 )); then echo "multiple .llbc" >&2; exit 1; fi`——预防意外多文件。

#3 (严重度: 中) — wrapper L13 `set -euo pipefail` 与 L52-53 `AENEAS_EXIT=$?` 有 set-e 矛盾

**现象**：[`tools/aeneas-lean/aeneas-lean-wrapper.sh`](../../../../tools/aeneas-lean/aeneas-lean-wrapper.sh) L13 / L51-58：

```bash
set -euo pipefail
...
"$AENEAS_BIN" -backend lean -dest "$LEAN_OUT" "$LLBC_FILE"
AENEAS_EXIT=$?
echo "[aeneas-lean-wrapper] aeneas exit: $AENEAS_EXIT"
```

**违反**：在 `set -e` 下，`"$AENEAS_BIN" ...` 非 0 退出会让脚本直接 exit——`AENEAS_EXIT=$?` 那行不执行。所以 wrapper 实际上没有"aeneas 失败后继续输出诊断" 的能力。

**推理链**：测试若 aeneas exit 1（partial），脚本立即 exit 1——stdout 没"aeneas exit: 1" 这行诊断。但 wrapper 整体 exit 1，runner 记 FAILED——oracle 正确。只是诊断信号缺失。

按 [`tool-integration.md`](../../../design/tool-integration.md) §二 wrapper 应"错误诊断信号清晰"——当前缺。

**决策性**：决策点——是否调整 `set -e` 范围以保留诊断。

**建议**：可选——把 aeneas 调用改为：
```bash
set +e
"$AENEAS_BIN" -backend lean -dest "$LEAN_OUT" "$LLBC_FILE"
AENEAS_EXIT=$?
set -e
echo "[aeneas-lean-wrapper] aeneas exit: $AENEAS_EXIT"
exit $AENEAS_EXIT
```

或用 `|| true`：`"$AENEAS_BIN" ... || true; AENEAS_EXIT=$?`。

但当前实施虽然诊断缺失，oracle 不漏。

#4 (严重度: 低) — `${TS_CHARON_BIN}` 等 env 变量在 tool.toml 展开方式：与 P12 expand_env trap 相符

**现象**：[`tools/aeneas-lean/tool.toml`](../../../../tools/aeneas-lean/tool.toml) L5-13：

```
command = [
  "env",
  "CHARON_BIN=${TS_CHARON_BIN}",
  "AENEAS_BIN=${TS_AENEAS_BIN}",
  "${TS_PROJECT_ROOT}/tools/aeneas-lean/aeneas-lean-wrapper.sh"
]
```

**违反**：未违反——按 [`discover.rs:294-335`](../../../../runner/src/discover.rs) `expand_env`，`${VAR}` 形式在 runner 启动时（read tool.toml）展开。`TS_CHARON_BIN` / `TS_AENEAS_BIN` / `TS_PROJECT_ROOT` 在 runner 启动前由 `.env` source 进 runner process env——所以展开成功。

**推理链**：与 hax-coq / hax-fstar 的 `$TS_ENTRY_FN`（无大括号）trap 不同——后者依赖运行时 child env（runner spawn 前注入 TS_ENTRY_FN）。aeneas 这里只用 runner 启动时已有的 env，所以 `${...}` 合适。

**决策性**：非决策点。

**建议**：无须改。

#5 (严重度: 中) — wrapper 用 `opam env --set-switch=default` 假设 default switch 有 aeneas 依赖

**现象**：[`tools/aeneas-lean/aeneas-lean-wrapper.sh`](../../../../tools/aeneas-lean/aeneas-lean-wrapper.sh) L22-24：

```bash
if command -v opam >/dev/null 2>&1; then
    eval "$(opam env --set-switch=default 2>/dev/null)" || true
fi
```

**违反**：[`tool-integration.md`](../../../design/tool-integration.md) §一——用户安装 aeneas 时可能在自定义 switch（如 `aeneas-build`），不在 default。wrapper 强制走 default switch——若用户 default 没装 zarith / gmp，aeneas 启动时 dyld load 失败。

**推理链**：README L52-54 安装说明也未明确 switch 名。当前 wrapper 假设有点强。但 `|| true` 容许 opam 不存在，所以 wrapper 不 hard fail；只是 aeneas 启动失败 silent。

**决策性**：决策点——是否暴露 switch 名作为 env var（如 `TS_AENEAS_OPAM_SWITCH`）。

**建议**：可选——增加 `TS_AENEAS_OPAM_SWITCH` env，默认 `default`。当前实施在多数 macOS 用户机器上 work，但不够通用。

#6 (严重度: 低) — version_command 用 `${TS_AENEAS_BIN} -version 2>&1 | tail -1`——抽取最后一行

**现象**：[`tools/aeneas-lean/tool.toml`](../../../../tools/aeneas-lean/tool.toml) L16：

```
version_command = ["sh", "-c", "${TS_AENEAS_BIN} -version 2>&1 | tail -1"]
```

**违反**：未违反——aeneas `-version` 可能多行（含 build info），`tail -1` 取版本字符串。

**推理链**：aeneas 上游 commit `a14083a6` 的版本输出格式假设 `-version` 输出 commit hash 在最后一行。若上游改格式，version_command 抓错。但 results.json metadata 是后续追溯的依据——抓错有 fallback 能力（仍有 raw stderr）。

**决策性**：非决策点。

**建议**：无须改。

#7 (严重度: 中) — entry_mode = "lib" 选择合规

**现象**：[`tools/aeneas-lean/tool.toml`](../../../../tools/aeneas-lean/tool.toml) L21：`entry_mode = "lib"`。

**违反**：未违反——按 [`tool-integration.md`](../../../design/tool-integration.md) entry_mode 选择，aeneas 翻译 lib target，不需要 bin harness。lib mode 让 harness 取代原 lib.rs，原 lib 内嵌为 `mod __ts_inner`。harness L9-10：`mod __ts_inner; pub use __ts_inner::*;`——纯透传，charon/aeneas 看到的就是完整 lib。

**推理链**：合规。但 harness L7-8 注释提到 `{{ target_crate_name }}` / `{{ entry_fn }}` "both referenced here for documentation"——实际 harness 没使用这两个变量，只是用 tera 注释引用文档说明。这是无害的 dead-text。

**决策性**：非决策点。

**建议**：无须改。

#8 (严重度: 低) — wrapper 与 README 形式严格性自陈一致

**现象**：[`tools/aeneas-lean/README.md`](../../../../tools/aeneas-lean/README.md) L42-46：

> **形式严格性 — 0 误报（不冤枉能力）**：✅ 形式可证。aeneas exit 0 ⇔ `Errors.error_list` 空 ⇔ 翻译完整
> **形式严格性 — 0 漏报（不高估能力）**：✅ 形式可证。aeneas 用 `craise` 把所有 unsupported 项 push error_list；`Main.ml:773` `if has_errors then exit 1` 是单一信号通路。

**违反**：未违反——这是 [`tool-integration.md`](../../../design/tool-integration.md) §三-反向证明 + §四-4.1 形式证明的典型例子。aeneas 因 `craise` 单一通路，源码层穷尽可证。

**推理链**：aeneas-lean 是矩阵中少数真"形式可证"的工具之一（与 aeneas-coq / aeneas-fstar / charon-mono / charon-poly / creusot 同属）。论证质量高。

**决策性**：非决策点。

**建议**：无须改。

## §4 决策点 vs 非决策点

- 决策点：2（set -e 范围 / opam switch 名外露）
- 非决策点：6

## §5 审查结论

aeneas-lean 配置 + wrapper + harness 整体优秀——形式严格性扎实（craise 单一通路 + Main.ml:773 单一 exit），README 与 wrapper 论证一致。最值得补强：

1. wrapper `set -e` 与 `$?` 抓 aeneas exit 的小冲突——影响诊断输出不影响 oracle；
2. opam switch 名硬编码 `default` —— 通用性有限。

整体属于"形式严格的优秀集成范例"，可作为其他工具的参考。
