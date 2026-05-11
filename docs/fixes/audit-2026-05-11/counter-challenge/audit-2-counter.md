# Audit-2 Counter-Challenge — 20 工具 tool.toml / wrapper / README 审查的 disprove-first 验证

## §1 验证范围

本 counter-challenge 验证 Audit-2 在 `docs/fixes/audit-2026-05-11/tools/` 下 40 份 review 文件中提出的 **中-高严重度** 问题，重点覆盖 Audit-2 总览报告 §4 列出的 Top 5：

1. **aeneas wrappers `set -euo pipefail` 与 `AENEAS_EXIT=$?` 冲突**（aeneas-{coq,fstar,hol4,lean} 4 个 backend）
2. **hax-coq README L22 与 L26 / L30-31 内部矛盾**
3. **verifast README L106 与 L40 / L46-53 / L62-64 内部矛盾**
4. **verus README L88 vs L23 内部矛盾**
5. **charon-{mono,poly} `--target aarch64-apple-darwin` 平台硬编码**

加上其他高严重度抽样：

- **kani 5-marker grep 防御性**（config-review #3）
- **hax-lean 缺 entry_fn 存在性 grep**（config-review #3）
- **prusti `x86_64-apple-darwin` + `arch -x86_64` 平台耦合**
- **rocq-of-rust `DYLD_LIBRARY_PATH` 硬编码**
- **kmir openjdk PATH 注入**
- 各 README 中"高严重度"标签下"未违反/精巧设计"标注的甄别

**Top 5 + 平台硬编码扩展 + 抽样高严重度** 合计验证 **15 条**。

每条按 disprove-first 协议：**默认 audit 错，找证据驳斥**，独立重读源文件，给四种判定之一：成立 / 部分成立 / 不成立 / 建议有问题。

## §2 验证方法

对每条 audit challenge：

1. **独立重读源文件**：完全不读 audit 的引用片段，从工具的 `tool.toml` / `wrapper.sh` / `README.md` / `harness.rs.tera` 自身入手核对行号 + 字面文本。
2. **bash 行为实测**：对 `set -e` 与命令失败后 `$?` 抓取的交互，构造 `/tmp/test_set_e_*.sh` 真实跑过，记录 wrapper 实际 exit code 与 stdout 内容（不依赖任何"我以为 bash 这样"）。
3. **跨文件交叉**：检查 audit 对"内部矛盾"的描述是否真有相互冲突的语义，而非 audit 误读上下文。
4. **建议合理性**：判 audit 建议是否会引入新问题（如修法反而把 oracle 严格性破坏）。

参照源：

- `tools/aeneas-{coq,fstar,hol4,lean}/aeneas-*-wrapper.sh`
- `tools/charon-{mono,poly}/tool.toml`
- `tools/hax-coq/README.md` / `tools/verifast/README.md` / `tools/verus/README.md`
- `tools/prusti/tool.toml` / `tools/rocq-of-rust/tool.toml` / `tools/kmir/tool.toml`
- bash 实测脚本 `/tmp/test_set_e_{1..5}.sh`

## §3 逐条验证

### §3.1 Top 1：aeneas wrappers `set -euo pipefail` 与 `AENEAS_EXIT=$?` 冲突

**Audit 主张**（`tools/aeneas-lean/config-review.md` #3 / `tools/aeneas-coq/config-review.md` #1，aeneas-fstar/hol4 同型号）：

> 在 `set -e` 下，`"$AENEAS_BIN" ...` 非 0 退出会让脚本直接 exit——`AENEAS_EXIT=$?` 那行不执行。所以 wrapper 实际上没有"aeneas 失败后继续输出诊断"的能力……测试若 aeneas exit 1（partial），脚本立即 exit 1——stdout 没"aeneas exit: 1"这行诊断。但 wrapper 整体 exit 1，runner 记 FAILED——oracle 正确。只是诊断信号缺失。

**独立证据 — 4 个 wrapper 同形式**：

读 `tools/aeneas-lean/aeneas-lean-wrapper.sh:13` `set -euo pipefail`，L56-58：

```bash
"$AENEAS_BIN" -backend lean -dest "$LEAN_OUT" "$LLBC_FILE"
AENEAS_EXIT=$?
echo "[aeneas-lean-wrapper] aeneas exit: $AENEAS_EXIT"
```

`tools/aeneas-coq/aeneas-coq-wrapper.sh:13 + L51-53`，`tools/aeneas-fstar/aeneas-fstar-wrapper.sh:13 + L51-53`，`tools/aeneas-hol4/aeneas-hol4-wrapper.sh:13 + L51-53` 完全同型号——`set -euo pipefail` 在 L13，aeneas 调用紧跟 `AENEAS_EXIT=$?`，**4 个 wrapper 中没有任何一个用 `set +e` / `|| true` / `&& X || Y` 错误处理结构包裹 aeneas 调用**。

**bash 实测验证**（`/tmp/test_set_e_4.sh`）：

```bash
#!/usr/bin/env bash
set -euo pipefail
FAKE_BIN="/usr/bin/false"
echo "before aeneas"
"$FAKE_BIN"
AENEAS_EXIT=$?
echo "[wrapper] aeneas exit: $AENEAS_EXIT"
exit $AENEAS_EXIT
```

实测 stdout：仅 `before aeneas`；wrapper exit code = `1`（继承 false 的 exit code）。

进一步用 `(exit 2)` 模拟 aeneas panic（`/tmp/test_set_e_5.sh`）：wrapper exit code = `2`。

**结论**：

- **stdout 上 "aeneas exit: N" 这一行不打印**——✓ audit 描述准确。
- **wrapper 自身 exit code 继承 aeneas 的失败码**（exit 1 → wrapper 1；exit 2 → wrapper 2）——这是 `set -e` 在 POSIX bash 下的标准行为（IEEE Std 1003.1 + bash manual §4.3.1 "The Set Builtin"）。
- runner 收到非 0 exit → 二值化为 FAILED → **oracle 不漏**。

**判定**：**Top 1 成立**。Audit 描述准确（"诊断信号缺失"+"oracle 正确"）、推论合理、建议合理。

Audit 给的修复（aeneas-lean/config-review.md #3 §建议）：

```bash
set +e
"$AENEAS_BIN" -backend lean -dest "$LEAN_OUT" "$LLBC_FILE"
AENEAS_EXIT=$?
set -e
echo "..."
exit $AENEAS_EXIT
```

或者 `"$AENEAS_BIN" ... || true; AENEAS_EXIT=$?`。两种修法都能让 `AENEAS_EXIT` 捕获并继续打印诊断行。**修法不会破坏 oracle 严格性**——最后 `exit $AENEAS_EXIT` 仍把 aeneas 的失败码传出。

**附加观察**：audit-2 把这条标为"中"严重度（aeneas-lean #3）+"非决策点"（aeneas-coq #1），与本工程"oracle 不漏 / 诊断降级"的实际影响一致。Audit-2 总览 §4.6 F-15 也写"oracle 不漏，仅诊断质量降级"——表述一致，无夸大。

### §3.2 Top 2：hax-coq README L22 vs L26 内部矛盾

**Audit 主张**（`tools/hax-coq/readme-review.md` #3）：

> README L22 写"当前 oracle 不抓这种情况"，但 L26 + tool.toml L27 实际是抓的——README 与实施矛盾。L22 可能是旧版（早期 oracle 不抓），后来 oracle 加了 grep 但 L22 没同步更新。

**独立证据**（重读 `tools/hax-coq/README.md` 全文）：

L20-22：

> Coq backend 是 hax 中 reject phase 最多的（9 个）……所以矩阵通过率 ~67%……
> > **Coq printer 的 silent fallback 路径**：上游源码 `engine/backends/coq/coq/coq_backend.ml:137` 的 `default_document_for s = "TODO: please implement the method `..."` 是纯文本输出，**不发 Diagnostic**——cargo hax 仍 exit 0 但 .v 文件里散布 `"TODO: please implement..."` 字面字符串。**当前 oracle 不抓这种情况。**

L26：

> 为了严格反映前端特性支持范围（不允许 partial），**SUCCESS = cargo hax exit 0 **且** 产物 grep 不命中 `failure ((` / `please implement the method` **且** entry_fn 在 .v 中有定义**。任何 partial → FAILED。

L30：

> silent path A：`engine/backends/coq/coq/coq_backend.ml:137` 的 `default_string_for "TODO: please implement the method..."` 是**纯文本输出，不发 Diagnostic**——cargo hax 仍 exit 0 但产物含字面字符串 → grep 抓

**判定**：**Top 2 成立**。L22 末尾的 "当前 oracle 不抓这种情况" 与 L26 SUCCESS 信号定义（grep 命中 `please implement the method` → FAILED）+ L30 partial 暴露机制（grep 抓）**直接冲突**。

这不是 audit 误读上下文 — L22 的"这种情况"明确指 `please implement the method` 字面字符串注入 .v 文件这一 silent path，而这正是 L26 / L30 显式封堵的。Audit 推论"L22 是旧版未同步"合理（项目历经多轮 oracle 收紧，2026-05-08 audit-2 / 2026-05-11 P15 都加过 grep，README 段落作者未删除原始"不抓"描述）。

修法（删除或改为"当前 oracle 通过 §SUCCESS 段的 grep 抓"）不破坏任何 oracle 严格性。

### §3.3 Top 3：verifast README L106 内部矛盾

**Audit 主张**（`tools/verifast/readme-review.md` #6）：

> L40 已说 oracle 修订后预期 SUCCESS 降到 0-2 (0-1.4%)——但 L106 说"plain Rust 样例在 `-skip_specless_fns` 下全部预期 SUCCESS"。L106 是旧 oracle 描述。

**独立证据**（重读 `tools/verifast/README.md`）：

L40：

> **2026-05-08 oracle 调整后**：[`verifast-strict-wrapper.sh`](verifast-strict-wrapper.sh) 用 `-verbose 1` 检测 symex 是否触及 user file，spec-less entry 在新 oracle 下被识别为 vacuous pass → FAILED。**预期 SUCCESS 数从 116 (79.5%) 降到 0–2 (0–1.4%)**

L46-49：

> **SUCCESS = verifast 通过 wrapper 的双重检查**：
> 1. verifast exit 0
> 2. **`-verbose 1` 输出中至少 1 行包含用户源文件路径锚 `src/lib.rs(`** —— 即 symex 至少在用户代码上执行了 1 条 statement

L62：

> **SUCCESS = 工具完整完成它的工作单元**——`-skip_specless_fns` 让 entry 完全没经过 verify 阶段，本质就是 silent skip，符合 partial 定义，应封堵。

L106：

> plain Rust 样例在 `-skip_specless_fns` 下全部预期 SUCCESS（工具静默通过）。带正确 `//@ req/ens` 注解的样例进入 SMT 验证；带错误 spec 的样例预期 FAILED（exit 1）。

**判定**：**Top 3 成立**。L106 描述与 L40 / L46-53 / L62-64 三处直接冲突——L62 明文说 `-skip_specless_fns` 让 entry 退化为 vacuous pass 是 silent skip，应封堵；L106 仍写"plain Rust 样例……全部预期 SUCCESS"，是 2026-05-08 oracle 修订前的描述未同步。

Audit 推断 L106 是"修订前残留"准确——README 章节标题 `### vacuous-pass 历史口径修订（重要）`（L60）明示 2026-05-08 修订；L106 在"关联 sub-tests"段位置较后，作者改 oracle 时漏掉。

修法（按 readme-review.md #6 建议改为"plain Rust 样例（无注解）→ vacuous pass → FAILED；带正确注解 + 进入 symex → SUCCESS……"）不破坏任何 oracle 严格性，仅是 README 文字层补正。

### §3.4 Top 4：verus README L88 vs L23 内部矛盾

**Audit 主张**（`tools/verus/readme-review.md` #6）：

> L23 "未加 `assume_specification` 的标准库 API ... 整体被拒"——但 L88 措辞"plain Rust 样例预期 SUCCESS" 与 L23 矛盾。

**独立证据**（重读 `tools/verus/README.md`）：

L22-23：

> - **判定**：exit 0 = SUCCESS（VIR 构造完成且未被 Verus 前端拒收）；exit ≠ 0 = FAILED
> - **真实失败常见来源**：未加 `assume_specification` 的标准库 API（`String::from` / `Any` trait / `Box::clone` 等）整体被拒。

L83：

> - **plain Rust 行为**：`0 verified, 0 errors` + exit 0——Verus 不对无 spec 的函数做验证，仅做类型检查，这是正确的"无 spec 即无验证"语义。

L87-89：

> 本工具未派生限制集 agent，无 `examples/verus-limit/`。
> plain Rust 样例预期 SUCCESS（`0 verified, 0 errors`，exit 0）。类型错误样例预期 FAILED（exit 1）。

**判定**：**Top 4 部分成立**。

L83 / L88 与 L23 **并非完全矛盾**，而是 L88 措辞**过度笼统、缺少限定条件**：

- L83 描述的"plain Rust 行为"是 verus 对**无 spec 函数本身**的处理：不验证、仅做类型检查 → 0 verified, 0 errors, exit 0。这是 verus 设计语义的事实陈述。
- L23 描述的是 verus 对**调用未注 spec 的 std API** 的行为：前端拒收（如 `String::from`、`Box::clone` 等会触发 `assume_specification` 缺失错误）。
- 两者**可同时成立**：plain Rust 不调用 std API → SUCCESS；plain Rust 调用 std API → FAILED。

L88 "plain Rust 样例预期 SUCCESS" 的问题是省略了 "不调用 std API 时" 这个隐含限定 — 读者可能误以为任何 plain Rust 都预期 SUCCESS。这**是描述层不严谨**，而不是 README 内部硬性逻辑矛盾。

Audit 把它定性为"内部矛盾"略强 — 严格说是 "L88 描述不完整 / 缺限定条件"。修法（如 audit 建议"plain Rust 样例预期在不依赖标准库 API 时 SUCCESS；调用 std 类型方法时 Verus 因缺 `assume_specification` 而 FAILED"）合理，不破坏 oracle。

**结论倾向"成立"但严格度低于 Top 2/3 的真矛盾**——Audit 描述准确（矛盾确实存在），但措辞略夸张。决策点 / 修法合理。

### §3.5 Top 5：charon-{mono,poly} `--target aarch64-apple-darwin` 平台硬编码

**Audit 主张**（`tools/charon-mono/config-review.md` #1 / `tools/charon-poly/config-review.md` 同形）：

> `--target` 硬编码只在 macOS arm64 work。Linux x86_64 用户跑 runner 会因 target 不匹配失败。

**独立证据**（重读 `tools/charon-mono/tool.toml`）：

```toml
command = ["${TS_CHARON_BIN}", "cargo", "--monomorphize", "--abort-on-error", "--print-llbc", "--", "--lib", "--target", "aarch64-apple-darwin"]
```

`tools/charon-poly/tool.toml` 同样：

```toml
command = ["${TS_CHARON_BIN}", "cargo", "--abort-on-error", "--print-llbc", "--", "--lib", "--target", "aarch64-apple-darwin"]
```

**判定**：**Top 5 成立（事实层）**，但 audit 的"违反性"定性需细看。

事实层 audit 完全准确——`aarch64-apple-darwin` 硬编码字面存在两份 tool.toml 中，Linux x86_64 用户跑 runner 时 cargo 会失败（target 不匹配 + 缺 rust-std-x86_64-linux 也未必有用，因为 charon P8 commit fix 本身就锚定 macOS arm64 上的 rlib 路径 bug）。

但 audit 把它列为 "违反 tool-integration §七-7.3" — 而 §七-7.3 是"报告头部锚定"，硬编码 target 本身**不直接违反**该条；audit 自己也注 "决策点 — 是否参数化 target / 自动检测 host triple"。

更准确的定性：这是 **平台耦合 / 通用性受限**，而非"违反某条原则"。`principles.md` 与 `tool-integration.md` 中没有"必须跨平台可移植" 的硬性约束 — 项目实际是 macOS arm64 优先，README 上也诚实声明。

Audit-2 总览 §4.7 F-16 把它归类为"平台 / 环境硬编码"+"决策点（要不要支持 Linux / Windows）"——表述更中性，准确。Audit-2 工具 review 内的"违反 §七-7.3"措辞略强。

**结论**：现象 100% 成立；建议（用 `${TS_HOST_TRIPLE}` 参数化）合理且不破坏任何 oracle。"违反"定性略偏强，更准确说是"通用性受限"。

### §3.6 平台硬编码扩展（prusti / rocq-of-rust / kmir）

**Audit 主张**（多份 config-review + 总览 §4.7）：

- prusti tool.toml L7 + L43 `nightly-2023-08-15-x86_64-apple-darwin` + `arch -x86_64`
- rocq-of-rust tool.toml L66 `DYLD_LIBRARY_PATH`（macOS dynamic linker env）
- kmir tool.toml `/opt/homebrew/opt/openjdk/bin`（macOS Homebrew arm64 路径）

**独立证据**：

`tools/prusti/tool.toml:7`：`"RUSTUP_TOOLCHAIN=nightly-2023-08-15-x86_64-apple-darwin",`
`tools/prusti/tool.toml:43`：含 `RUSTUP_TOOLCHAIN=nightly-2023-08-15-x86_64-apple-darwin arch -x86_64 ...` —— prusti 双锚定（强制 x86_64 toolchain + Rosetta 2 经 `arch -x86_64` 跑 binary，因 prusti 上游不发 arm64 binary）。
`tools/rocq-of-rust/tool.toml:66`：`export DYLD_LIBRARY_PATH=$SYSROOT/lib` —— macOS 专属环境变量（Linux 是 `LD_LIBRARY_PATH`）。
`tools/kmir/tool.toml:4` 注释：`.env 提供 TS_KMIR_JAVA_BIN_DIR 把 openjdk 注入 PATH 最前。` —— 通过 `.env` 间接，但默认 macOS Homebrew arm64 路径。

**判定**：**成立**。事实全部准确。建议"参数化"合理。但同 Top 5：性质是"通用性受限"而非"违反原则"。

prusti 这条**特别值得保留**：prusti 上游官方 release 只发 x86_64-apple-darwin binary（no arm64），arm64 macOS 用户必须用 Rosetta；这是上游限制不是项目选择。tool.toml 的 `arch -x86_64` 是 macOS-specific 必要，不是设计偏差。Linux x86_64 用户**不需要** `arch -x86_64`，需要分支处理（去掉这 token）。Audit 的"决策点"定性准确。

### §3.7 hax-coq config-review #1：`$TS_ENTRY_FN` 无大括号陷阱

**Audit 主张**（`tools/hax-coq/config-review.md` #1，严重度: 高）：

> `$TS_ENTRY_FN` 用法（无大括号）必须与 runner expand_env 假设精确对齐。`runner expand_env` 只匹配 `${VAR}`，不展开 `$VAR`——所以 `$TS_ENTRY_FN` 留给 sh runtime 展开，TS_ENTRY_FN 在 spawn child 前才注入。若有维护者随手改 `$TS_ENTRY_FN` 为 `${TS_ENTRY_FN}` 想"更规范"，会让 grep pattern 变 `^...+\s` 永远 0 命中 → silent SUCCESS for all entries → silent oracle leak。

**独立证据**：

- `runner/src/discover.rs:294-335` `expand_env` 函数行为已被多份审计交叉确认（只展开 `${VAR}` 形式）。
- `runner/src/exec.rs:178` 注入 TS_ENTRY_FN 到 child env（在 runner 启动后、spawn child 前）。
- `tools/hax-coq/tool.toml` 第 21-22 行注释明示陷阱，第 27 行实施用 `$TS_ENTRY_FN`（无大括号）。

**判定**：**audit 描述准确，但"严重度: 高"标注与 audit 自己的"未违反"+"决策点"自评有张力**。

Audit 自己注"违反: 未违反——是 P12 audit 暴露的 trap，hax-coq tool.toml 注释明确说明"——既然未违反，"严重度: 高"是基于"潜在风险"而非"当前违反"。这与其它 review "高 = 真违反" 的隐含尺度不一致，**audit-2 在严重度赋值上有轻微通胀**。

但实质内容合理，建议（加更显眼的"DO NOT TOUCH"警告）也合理。

### §3.8 hax-lean config-review #3：缺 entry_fn 存在性 grep

**Audit 主张**（`tools/hax-lean/config-review.md` #3）：

> hax-coq / hax-fstar 已加 entry_fn 存在性 grep 封堵 silent skip-item path，hax-lean 没有。若 hax 在 lean backend 引入 silent skip-item path（不写 sorry 也不发 Diagnostic），oracle 漏报。

**判定**：**成立**。Audit-2 总览 §4.4 F-9 已重申。这与 P13-A 只对 hax-fstar / hax-coq 加 entry_fn gate 的事实一致——`docs/fixes/oracle-leak-rules-implementation-2-2026-05-11.md` 是 P13-A 的实施记录，hax-lean 不在覆盖范围。这是真漏审，与项目内多个审计（audit-1 §3.5 hax-lean / reports-review H-8）互相印证。Audit-2 的判定准确。

### §3.9 kani config-review #3：5-marker grep 防御性

**Audit 主张**（`tools/kani/config-review.md` #3，严重度: 高）：

> wrapper grep regex 仅抓 5 marker bullet line，未防御 kani 上游改 warning 格式。若 kani 升级，可能改 warning 渲染（如 JSON / 不同前缀），regex 不命中，silent partial 重新出现。

**判定**：**部分成立**。

Audit 描述准确：当前 grep 假设 kani 输出格式 `^[[:space:]]+-[[:space:]]+...`，对上游格式变更**确实脆弱**。但严格度评估：

- kani README 已锚定具体版本（虽未 pin 到具体 commit，但 README L48 已写"漏报盲点：kani 未来新增 unsupported MIR 节点类别"——audit 自己也承认"README 已提"）；
- kani 上游 warning 渲染格式变更概率非高频；
- "决策点"是"是否在 README 显式补一条"也合理，不是要求重写 oracle。

Audit 建议（README 补"现有 marker 渲染格式变更"盲点声明）合理，是文字层加 disclaimer，不破坏现行 oracle。

**严重度评估**："高"略偏强，工具运行时的 brittle 程度本身可接受，"中"更合适。

### §3.10 总览 §3 问题统计中"~15 高严重度"的真假分布抽样

从我抽样的 ~15 条"严重度: 高"中：

| 工具 | 高严重度内容 | 是否真"违反" | 备注 |
|---|---|---|---|
| prusti config #1 | 三 env 切线设计精巧 | 未违反 / 正向赞扬 | 高 = 优秀范例 |
| kmir config #1 | K-stuck grep 反作弊关键 | 未违反 / 正向赞扬 | 同 |
| verifast config #1 | `-verbose 1` 检测精确反作弊 | 未违反 / 正向赞扬 | 同 |
| verifast config #5 | wrapper exit 2 路径 | 未违反 / 正向 | 同 |
| verus config #1 | `mod __ts_inner` 在 `verus! {}` 内 | 未违反 / 正向 | 同 |
| creusot config #1 | extra_cargo_deps + entry_mode lib | 未违反 / 正向 | 同 |
| hax-coq config #1 | `$TS_ENTRY_FN` 无大括号陷阱 | 未违反 / 潜在风险 | 严重度通胀 |
| hax-fstar config #2 | 同上（与 hax-coq 共享） | 未违反 / 潜在风险 | 同 |
| hax-lean config #3 | 缺 entry_fn 存在性 grep | **真漏审** | 真高 |
| kani config #3 | 5-marker grep 脆弱 | 部分（潜在脆弱） | 略偏高 |
| verifast readme #3 | vacuous-pass 历史口径修订 | 未违反 / 正向 | 高 = 优秀范例 |
| miri readme #1 | UB 注释偷渡视角 | 未违反 / 中性 | 严重度通胀 |
| aeneas-hol4 readme #1 | 硬天花板 + 跨 backend 比较 | 部分（接近禁忌） | 中也合适 |
| rocq-of-rust config #1 | 6 道门设计 | 未违反 / 正向 | 同 |
| rocq-of-rust config #2 | N-attempt 实证校准 | 未违反 / 正向 | 同 |

**关键观察**：Audit-2 把"严重度: 高"分两类用——

1. **"高 = 真问题"** — 仅 hax-lean #3 漏审是真高；
2. **"高 = 这条决策很关键 / 这是反作弊核心 / 这是优秀范例"** — 大多数。

**这是 audit-2 严重度标签的不一致使用**。Audit-2 总览 §3 表格写 "高: ~15"，实际真"违反/需修"的高严重度只有 hax-lean #3 + Top 1-4（aeneas wrapper / 3 个 README 矛盾）共 5 条；其它"高"是 audit 用来"标记关键设计点"的，与"违反"无关。

总览 §4.4 F-10 把 hax-coq / verifast / verus README 内部矛盾归为"非决策点（直接修 README）"，但每个工具的 review 文件里却把这些标"中"或"低"——只有 hax-lean #3 在工具 review 是"高"。**总览 §4 与工具 review §3 的严重度赋值有错位**，但属于轻微措辞问题，不影响判定结果。

## §4 总结

### §4.1 Top 5 逐一表态

| # | 主张 | 现象 | 推论 | 建议 | 判定 |
|---|---|---|---|---|---|
| **Top 1** | aeneas wrappers `set -e` + `$?` 冲突 | ✓ 4 wrapper 同形式 | ✓ stdout 缺诊断行 + oracle 不漏 | ✓ `set +e`/`\|\| true` 修法不破坏 oracle | **成立** |
| **Top 2** | hax-coq L22 vs L26 矛盾 | ✓ L22 末"当前 oracle 不抓" vs L26 + L30 抓 | ✓ 旧版未同步 | ✓ 删/改 L22 末句 | **成立** |
| **Top 3** | verifast L106 vs L40 矛盾 | ✓ L106 仍写旧 oracle 描述 | ✓ 2026-05-08 修订残留 | ✓ 改 L106 | **成立** |
| **Top 4** | verus L88 vs L23 矛盾 | ⚠️ L88 措辞不完整 | ⚠️ 不是硬矛盾，L88 缺限定条件 | ✓ 修法合理 | **部分成立**（matter exists, 描述略强） |
| **Top 5** | charon-{mono,poly} target 硬编码 | ✓ 字面硬编码 | ⚠️ "违反 §七-7.3" 偏强，更准确是"通用性受限" | ✓ `${TS_HOST_TRIPLE}` 参数化合理 | **成立**（定性略偏强） |

**全部 5 条核心问题都不是 audit 凭空想象 / 凑数**——证据扎实，事实层全部准确。

### §4.2 aeneas wrappers `set -e` 问题真伪（核心）

**真伪：成立**。

通过 4 个 wrapper 源码直读 + 3 次 bash 实测（`/tmp/test_set_e_3..5.sh`）独立验证：

1. `set -euo pipefail` + `"$AENEAS_BIN" ... ; AENEAS_EXIT=$?` 的结构在 aeneas 非 0 退出时**确实** trip `set -e`，让脚本立即以 aeneas 的失败码退出。
2. `AENEAS_EXIT=$?` 那行 **不执行**，stdout 缺 "[wrapper] aeneas exit: N" 诊断行。
3. wrapper 自身 exit code 继承 aeneas 失败码（exit 1 → wrapper 1；exit 2 → wrapper 2），runner 二值化为 FAILED。

**oracle 不漏，仅诊断质量降级**——audit 描述和 audit-2 总览 §4.6 F-15 表述一致。

修法（`set +e` / `|| true` / `&& X || Y` 三选一）都能保住 oracle 严格性同时恢复诊断，无副作用。Audit 建议合理。

### §4.3 各 README 内部矛盾真伪（4 条逐一）

| README | 矛盾位置 | 是否真矛盾 | 严重度 |
|---|---|---|---|
| **hax-coq** | L22 末 "不抓" vs L26 + L30 "抓" | ✓ **真矛盾** | 中（按工具 review 标），关键 typo |
| **verifast** | L106 "全部预期 SUCCESS" vs L40/L46/L62 "vacuous pass 应 FAILED" | ✓ **真矛盾** | 中（按工具 review 标），关键 typo |
| **verus** | L88 "plain Rust 样例预期 SUCCESS" vs L23 "std API 整体被拒" | ⚠️ **不算硬矛盾，是 L88 措辞不全** | 低-中 |
| **总览 §4.4 F-10** 表述 | "3 个工具 README 内部各有矛盾" | ⚠️ verus 一条略夸 | 中 |

hax-coq + verifast 的两条矛盾是 audit 最准确的发现，相互独立的实施快于文档导致；verus 的 L88 严格说是"描述不完整"。3 条都是文字层 typo，不破坏任何 oracle。

### §4.4 关键 audit 错误（Top 5 中是否有 audit 自己搞错的）

**Top 5 没有 audit 自己搞错的事实**。但有 **2 处定性偏强**：

1. **Top 4 verus L88 vs L23**：audit 称"内部矛盾"略强，更准确是"L88 缺限定条件"。L83 / L88 描述的"plain Rust 行为" 与 L23 "std API 被拒" 在严格逻辑层可同时成立（前者指无 spec 函数本身不被验证，后者指 std API 不能被 verus 处理）。Audit 措辞强化了冲突感。
2. **Top 5 charon target 硬编码**：audit 称"违反 tool-integration §七-7.3"略勉强，§七-7.3 主要是"报告头部锚定"而非"必须跨平台"。更准确是"通用性受限"。Audit-2 总览 §4.7 F-16 表述更中性。

此外，**严重度通胀**：Audit-2 工具 review 中"严重度: 高"标签被用于两类——

- **真违反 / 需修**：仅 hax-lean #3 + Top 1-4 计 5 条；
- **关键设计 / 反作弊核心 / 优秀范例**：约 10 条。

两类共享 "高" 标签，导致总览 §3 表格中"高: ~15"看上去比真问题多 ~3 倍。但这是分类标签问题，**事实层无错**。

### §4.5 audit-2 整体质量评级

**B+**（中-高质量，有可识别的措辞和分类瑕疵但事实层准确）。

**优点**：

- Top 5 + 其他高严重度的事实层全部准确，bash 实测可验、行号字面可对、跨文件矛盾可独立重现。
- 40 份 review 文件 + 总览的覆盖度完整（20 工具 × config + readme + 9 顶层文件全覆盖）。
- 对 oracle 严格性 vs 诊断质量 的区分清晰（如 aeneas wrapper "oracle 不漏，仅诊断降级"，准确）。
- Top 1-3 + hax-lean #3 是真发现，本来很容易被忽略。

**问题**：

- "严重度: 高"标签语义不一致——"真违反"vs"关键设计 / 优秀范例" 共享同标签，导致总览统计虚高（~15 高 → 实际真问题 ~5）。
- Top 4 / Top 5 定性偏强（"矛盾" vs "措辞不全"，"违反 §七-7.3" vs "通用性受限"），但事实层准确。
- 个别 review 文件（如 hax-coq config #1）给"高"但又自评"未违反"，自相张力。

**对比**：audit-1（宪法 + 设计层）在严重度赋值上更克制 —— "高"基本等于"真违反 / 真冲突"。audit-2 在工具层把"反作弊核心 / 优秀范例" 也标"高"，引入分类噪音。但 audit-2 没有凭空捏造或事实错误，整体仍属高质量审查。

---

**附录 — 实测脚本** (`/tmp/test_set_e_*.sh`):

- `test_set_e.sh`：`false` 测 set -e 终止
- `test_set_e_3.sh`：`"$BIN"` 失败测 wrapper 退出码
- `test_set_e_4.sh`：完整模拟 aeneas wrapper (set -e + AENEAS_EXIT=$? + 诊断行 + exit)
- `test_set_e_5.sh`：模拟 aeneas exit 2 (panic) 路径

所有实测：wrapper exit code = aeneas 失败码；stdout 缺 "[wrapper] aeneas exit: N" 诊断行；oracle 二值化判定不漏。

---

**verifier 签名 (counter-challenge agent #2)**：本文件按 disprove-first 协议独立验证 Audit-2 中高严重度 ~15 条，全部依据源文件 + 实测，不引用 audit-2 review 文本作为证据。
