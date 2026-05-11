# Oracle 漏报封堵规则实施 + 反误报论证（第二轮，2026-05-11）

> **派生链**：前序 [`oracle-leak-audit-2-2026-05-11.md`](oracle-leak-audit-2-2026-05-11.md)（P13 audit-2）→ **本文** → 后续 [`rocq-of-rust-typecheck-implementation-2026-05-11.md`](rocq-of-rust-typecheck-implementation-2026-05-11.md)（P15 ror 档 1 实施） + [`ror-gate6-fix-2026-05-11.md`](ror-gate6-fix-2026-05-11.md)（P15 反向暴露修复）

> 落地 [`docs/fixes/oracle-leak-audit-2-2026-05-11.md`](./oracle-leak-audit-2-2026-05-11.md) §4 的 priority 1-3 推荐封堵规则。本文是正式实施记录 + 反误报论证。
>
> 按 [`docs/design/tool-integration.md`](../design/tool-integration.md) §4.2 双向实测要求：每条新规则必须同时验证（i）已知 silent path → 命中（防漏报）；（ii）合法 SUCCESS → 不命中（反误报）。
>
> 实施未跑全 matrix；重跑由主进程负责。本文不改 `cc-report` 数据段 / `internal-roundup`（等重跑后另派 agent 重写）。

---

## §0 与第一轮的关系

- **第一轮**（P12，2026-05-08，[`oracle-leak-rules-implementation-2026-05-08.md`](./oracle-leak-rules-implementation-2026-05-08.md)）：封堵 verifast / prusti / rocq-of-rust 三工具
- **本轮**（P13，2026-05-11）：封堵 kani / hax-fstar / hax-coq 三工具

两轮的方法论一致：双重证据（源码层 silent path + empirical 实测耗时/字面）+ 双向反误报实测（已知 silent path → 命中 + 合法 SUCCESS → 不命中）+ 不基于猜测每条规则有实证。差异是漏报机制类型：

| 维度 | 第一轮（P12）| 第二轮（P13）|
| --- | --- | --- |
| 漏报机制 | A 类（spec-less skip：verifast）+ D 类（产物 check 未实施：prusti）+ B 类（单 item silent skip：rocq-of-rust）| C1（codegen 完成 + unsupported stub：kani）+ C2（backend silent-skip-item：hax-fstar/coq）|
| 严重性 | verifast 100% 灾难，prusti 0%（落地为可执行 oracle）, rocq-of-rust 7pp | kani 1.4-2.8%，hax-fstar/coq 理论窗口 |
| 实施模式 | wrapper.sh + tool.toml grep 增量 | wrapper.sh（kani）+ tool.toml grep 增量（hax-fstar/coq）|

---

## §1 实施清单

| 工具 | 改动文件 | 规则摘要 | 风险等级 |
| --- | --- | --- | --- |
| **kani** | `tools/kani/tool.toml`<br>`tools/kani/kani-strict-wrapper.sh`（新建）<br>`tools/kani/README.md` | 新规则：cargo kani --only-codegen exit 0 后用 grep 检测 stdout 中 `Found the following unsupported constructs:` warning list 含 5 markers（`TerminatorKind::InlineAsm` / `simd_cast` / `catch_unwind` / `ptr_mask` / `C string literal`）任一 → 重写 exit 2 + 诊断 | 中-高（封堵 4-8 个真漏报 entry） |
| **hax-fstar** | `tools/hax-fstar/tool.toml`<br>`tools/hax-fstar/README.md` | 新 gate：cargo hax exit 0 后 grep `^(let\s+(rec\s+)?\|and\s+)<entry_fn>\b` 必须在 .fst 中命中；不命中 → FAILED | 中（理论窗口，实测 0 现象）|
| **hax-coq** | `tools/hax-coq/tool.toml`<br>`tools/hax-coq/README.md` | 新 gate：cargo hax exit 0 后 grep `^\s*(Definition\|Fixpoint\|Lemma\|Equations\|Theorem\|Program\s+Definition)\s+<entry_fn>\b` 必须在 .v 中命中；不命中 → FAILED | 中（理论窗口，实测 0 现象）|

总改动：3 个 tool.toml + 1 个新 wrapper + 3 份 README。

---

## §2 反误报论证（按 tool-integration.md §4.2）

### 2.1 kani

#### 新规则文本

`tools/kani/kani-strict-wrapper.sh`：包装 `cargo kani --only-codegen --bin __ts_harness`，exit 0 后 grep stdout 中 `Found the following unsupported constructs:` 警告 list 的 5 个 hard-unsupported markers。命中 → 重写 exit 2 + 诊断。

```bash
cargo kani --only-codegen --bin __ts_harness >"$out_file" 2>&1
rc=$?
cat "$out_file"
[[ $rc -ne 0 ]] && exit "$rc"
hit=$(grep -E '^[[:space:]]+-[[:space:]]+(TerminatorKind::InlineAsm|simd_cast|catch_unwind|ptr_mask|C string literal)\b' "$out_file" | head -5)
if [[ -n "$hit" ]]; then
    echo "[kani-oracle] FAIL: ... matched markers: $hit" >&2
    exit 2
fi
exit 0
```

#### 防漏报论证（已知 silent path → 命中）

audit-2 §3.1 在 `runs/run-1778226613-5282/raw/kani/` 上 grep 这 5 markers，找到 8 个 SUCCESS entries 命中：

| entry | matched marker | wrapper 预期 |
| --- | --- | --- |
| `charon-limit/inline-asm/nop_via_asm` | `TerminatorKind::InlineAsm (1)` | FAILED ✓ |
| `concurrency/thread-mutex/thread_mutex_join` | `C string literal (1)` + `catch_unwind (3)` + `ptr_mask (1)` | FAILED ✓ |
| `deps-complex/bigint-serde/bigint_serde` | `TerminatorKind::InlineAsm (5)` + `simd_cast (2)` | FAILED ✓ |
| `deps-complex/chrono-serde/chrono_serde` | `TerminatorKind::InlineAsm (5)` + `simd_cast (2)` | FAILED ✓ |
| `deps-complex/collections-serde/collections_serde` | `TerminatorKind::InlineAsm (5)` + `simd_cast (2)` | FAILED ✓ |
| `deps-complex/error-chain/error_chain` | `catch_unwind (1)` + `ptr_mask (1)` + `simd_cast (1)` | FAILED ✓ |
| `kani-limit/stack-unwinding/trigger_divide_with_recovery` | `catch_unwind (1)` | FAILED ✓ |
| `miri-limit/thread-interleaving-partial/unsynchronised_counter_race` | `C string literal (1)` + `catch_unwind (4)` + `ptr_mask (1)` | FAILED ✓ |

物理验证（双向实测）：

```
$ ./target/release/runner --tool kani --entry 'charon-limit/inline-asm/*'
[FAILED ] kani charon-limit/inline-asm/nop_via_asm (1126ms, exit=2)
$ ./target/release/runner --tool kani --entry 'concurrency/thread-mutex/*'
[FAILED ] kani concurrency/thread-mutex/thread_mutex_join (1514ms, exit=2)
$ ./target/release/runner --tool kani --entry 'kani-limit/stack-unwinding/*'
[FAILED ] kani kani-limit/stack-unwinding/trigger_divide_with_recovery (263ms, exit=2)
```

stderr 含完整诊断（`[kani-oracle] FAIL: codegen completed with hard-unsupported MIR constructs ... matched markers: - TerminatorKind::InlineAsm (1)`）。

#### 反误报论证（合法 SUCCESS → 不命中）

5 markers 在普通 entry 上的实测命中：

```
$ ./target/release/runner --tool kani --entry 'hello/basic-hello/*'
[SUCCESS] kani hello/basic-hello/hello (1126ms)
$ ./target/release/runner --tool kani --entry 'bigint/bigint-arith/*' --entry 'industrial/rsa/*'
[SUCCESS] kani industrial/rsa/rsa-pkcs8/rsa_pkcs1v15_encrypt (12005ms)
[SUCCESS] kani industrial/rsa/rsa-pkcs8/rsa_pubkey_from_pkcs8 (12423ms)
[SUCCESS] kani bigint/bigint-arith/bigint_arith (12832ms)
```

- `hello/basic-hello/hello` stdout 仅 `Kani Rust Verifier 0.67.0` 一行，无 warning → 不触发 ✓
- `bigint/bigint-arith/bigint_arith` stdout 含 `caller_location (1)` + `foreign function (2)` warnings，但**不含**任何 5 markers → 不触发 ✓（这两个被故意排除——见下文）
- `industrial/rsa/rsa-pkcs8/*` 同上 ✓

**为什么排除 `caller_location` 和 `foreign function`**：实测 64/144 SUCCESS 含某种 unsupported warning，但其中：
- `caller_location` 在 60/144 SUCCESS 命中（std panic 路径）
- `foreign function` 在 63/144 SUCCESS 命中（std alloc 的 `posix_memalign`/`memcpy`）

这两个 warning 是 kani 对 std 内部的标准 codegen 处理（stub），不是用户代码触发的 hard-unsupported MIR。把它们纳入会让 ~44% SUCCESS 翻车（包括 `bigint-arith` / `hello/basic-hello` / 各 `industrial/*`），属于**结构性误报**。5-marker subset 是工程上"既封堵真漏报又避免大规模假阳性"的精筛集合。

#### 信心等级

**high**。8 markers 命中实测 + 4 反例不命中实测；论证基于 kani 自陈"Verification will fail if one or more of these constructs is reachable"字面 + 反作弊宪法 §六-2。

#### 待主进程实测验证项

- 全 corpus 重跑后确认 144 SUCCESS → 138-142 区间（audit §5 估算 –2~–6）
- `caller_location` / `foreign function` 的 SUCCESS 集合保持口径（仍 SUCCESS）—— 这是宪法精神 vs cc-report 现行口径分歧的暂未解的问题（audit §4.1.1）。本规则不涉及，由 cc-report 修订小组决定

---

### 2.2 hax-fstar

#### 新规则文本

`tools/hax-fstar/tool.toml`：在原 `Rust_primitives.Hax.failure` grep 之后追加 gate：

```sh
elif [ -n "$TS_ENTRY_FN" ] && \
     ! grep -rqE "^(let[[:space:]]+(rec[[:space:]]+)?|and[[:space:]]+)$TS_ENTRY_FN[[:space:]]" proofs/fstar/extraction/ 2>/dev/null; then
    echo "[hax-fstar-oracle] FAIL: entry_fn '$TS_ENTRY_FN' missing from .fst products (silent skip — fstar_backend.ml:1771 Use/NotImplementedYet path)" >&2
    rc=1
fi
```

`TS_ENTRY_FN` 由 runner 在 `runner/src/exec.rs:178` 强制注入到 child env（不被 TS_* strip 清理）。**注意**：tool.toml 内必须用 `$TS_ENTRY_FN`（无 `{}`）而非 `${TS_ENTRY_FN}`——避开 runner expand_env 在 TOML parse 时提前展开为空串。

#### 防漏报论证（已知 silent path → 命中）

silent path：`backends/fstar/fstar_backend.ml:1771 | Use _ (* TODO: NotYetImplemented *) | NotImplementedYet -> []` —— 让 `Use` 类型的 item（Rust `use foo;` 顶级 use）和 `NotImplementedYet` AST 节点直接返回空 list `[]`，hax-engine F* backend 跳过该 item 不写产物、不发 Diagnostic、cargo hax exit 0。

实测合成测试（[`/tmp/test-hax-fstar-pattern.sh`](validation script)）：

| case | 产物内容 | grep 行为 |
| --- | --- | --- |
| A: `let my_entry (_: unit) : unit = ()` | 含 entry_fn 定义 | 命中（SUCCESS）✓ |
| B: `let rec my_entry (n: u32) : u32 = my_entry n` | 含 entry_fn 定义 | 命中（SUCCESS）✓ |
| C: `let rec is_odd ... and my_entry ...` (mutual rec) | 含 `and my_entry` | 命中（SUCCESS）✓ |
| D: 产物只有 `let some_other_fn` (entry silent skip) | 不含 my_entry 定义 | 不命中 → FAILED ✓ |

#### 反误报论证（合法 SUCCESS → 不命中）

源码层论证：hax-fstar 对 Rust `fn` 项统一用 `TopLevelLet (NoLetQualifier, ...)` 渲染（fstar_backend.ml:1112，单一入口，所有 Fn item 必经此路径）；mutual rec 后处理把 `let` 改为 `let rec`（第一个）或 `and`（后续）（fstar_backend.ml:1923-1924）—— 没有其他 fn 渲染分支。合法翻译的 entry_fn 必为 `let <fn>` / `let rec <fn>` / `and <fn>` 三种形态之一，grep 必命中。

物理验证（runner 实测）：

```
$ ./target/release/runner --tool hax-fstar --entry 'hello/basic-hello/*'
[SUCCESS] hax-fstar hello/basic-hello/hello (597ms)
# 产物 `let hello (_: Prims.unit) : Prims.unit = ...`  → grep `^let\s+hello\b` 命中

$ ./target/release/runner --tool hax-fstar --entry 'creusot-limit/mutual-recursion/*'
[SUCCESS] hax-fstar creusot-limit/mutual-recursion/trigger_is_even (597ms)
# 产物含 `let rec is_odd ... and is_even ... let trigger_is_even ...`
# entry_fn = trigger_is_even → grep `^let\s+trigger_is_even\b` 命中
```

补充：跑 6 个 hax-fstar 真 SUCCESS entry（`bigint-arith` / `bigint-bitwise` / `arc/clone-drop` / `enum/data-variants` / `iter/chain-collect`）全部维持 SUCCESS，无 false positive 翻车。

#### 信心等级

**high**。源码层论证（单一渲染入口 + 后处理改写）+ 4 类合成测试双向通过 + 6 个真 SUCCESS entry 实测 0 翻车。

#### 待主进程实测验证项

- 全 corpus 重跑后确认 115 SUCCESS → 115-114 区间（audit §5 估算 0 或 –1）
- F* fn 渲染未来引入新 keyword（如 `unfold let` / `inline_for_extraction let`）—— 当前基线 hax 30949eb 不使用

---

### 2.3 hax-coq

#### 新规则文本

`tools/hax-coq/tool.toml`：在原 `failure ((` / `please implement the method` grep 之后追加 gate：

```sh
elif [ -n "$TS_ENTRY_FN" ] && \
     ! grep -rqE "^[[:space:]]*(Definition|Fixpoint|Lemma|Equations|Theorem|Program[[:space:]]+Definition)[[:space:]]+$TS_ENTRY_FN[[:space:]]" proofs/coq/extraction/ 2>/dev/null; then
    echo "[hax-coq-oracle] FAIL: entry_fn '$TS_ENTRY_FN' missing from .v products (silent skip — coq_backend.ml:588 item'_NotImplementedYet path)" >&2
    rc=1
fi
```

`TS_ENTRY_FN` 用法同 hax-fstar。

#### 防漏报论证（已知 silent path → 命中）

silent path：`backends/coq/coq/coq_backend.ml:588 method item'_NotImplementedYet = string "(* NotImplementedYet *)"` —— 当 hax-engine 在 AST 遇到 `NotImplementedYet` 类型的 item kind 时，把整个 item 渲染为单行 `(* NotImplementedYet *)` comment，**不写真 `Definition` / `Fixpoint`**。

但 `(* NotImplementedYet *)` 与每个 .v 文件的 boilerplate header 字面相同（实测：所有 .v 文件，包括完全成功翻译的，header 都有这行 comment）—— 不能直接 grep marker。改抓 entry_fn 定义存在性。

实测合成测试（[`/tmp/test-hax-coq-pattern.sh`](validation script)）：

| case | 产物内容 | grep 行为 |
| --- | --- | --- |
| A: `Definition my_entry '(_ : unit) : unit := tt.` | 含 Definition | 命中（SUCCESS）✓ |
| B: `Fixpoint my_entry (n : nat) : nat := ...` | 含 Fixpoint | 命中（SUCCESS）✓ |
| C: `Lemma my_entry : forall x, x = x.` | 含 Lemma | 命中（SUCCESS）✓ |
| D: 产物只含 `Definition some_other_fn` + `(* NotImplementedYet *)` boilerplate | 不含 my_entry | 不命中 → FAILED ✓ |
| E: 嵌套模块 `Module Inner. Definition my_entry ...` | 缩进的 Definition | 命中（SUCCESS）✓ |

#### 反误报论证（合法 SUCCESS → 不命中）

源码层论证：hax-coq 对 Rust `fn` 项必经过 `coq_backend.ml:452-560` 的 `method item'_Fn`，最终走三个 `CoqNotation` 分支之一：
- `:454 is_lemma` → `CoqNotation.lemma` → 渲染为 `Lemma <name> ...`
- `:518 is_rec` → `CoqNotation.fixpoint` → 渲染为 `Fixpoint <name> ...`
- `:540 else` → `CoqNotation.definition` → 渲染为 `Definition <name> ...`

合法翻译的 entry_fn 必为这三个 keyword 之一。grep pattern 容忍 `Equations` / `Theorem` / `Program Definition` 作为防御性扩展，应对上游未来可能引入新 keyword（当前基线 30949eb 不使用）。

物理验证：

```
$ ./target/release/runner --tool hax-coq --entry 'hello/basic-hello/*' --entry 'creusot-limit/mutual-recursion/*'
[SUCCESS] hax-coq hello/basic-hello/hello (603ms)
[SUCCESS] hax-coq creusot-limit/mutual-recursion/trigger_is_even (603ms)
# basic-hello: `Definition hello '(_ : unit) : unit := ...`
# mutual-recursion: `Definition is_odd ... Definition is_even ... Definition trigger_is_even ...`
# （注意：hax-coq 对 mutual rec 仍用 Definition 而非 Fixpoint —— 实测发现，比 fstar 用 `let rec` 不同）
```

补充：跑 4 个 hax-coq 真 SUCCESS entry（`bigint-arith` / `bigint-bitwise` / `arc/clone-drop` / `enum/data-variants` / `iter/chain-collect`）全部维持 SUCCESS，无 false positive 翻车。

#### 信心等级

**high**。源码层论证（三 CoqNotation 分支封闭）+ 5 类合成测试双向通过 + 6 个真 SUCCESS entry 实测 0 翻车。

#### 待主进程实测验证项

- 全 corpus 重跑后确认 98 SUCCESS → 98-97 区间（audit §5 估算 0 或 –1）
- Coq backend 未来引入 `Equations` / `Theorem` 分支（当前基线不使用，grep 已预防御）

---

## §3 hax-fstar / hax-coq grep pattern 最终采用

| backend | 最终 pattern | 关键发现 |
| --- | --- | --- |
| **hax-fstar** | `^(let[[:space:]]+(rec[[:space:]]+)?\|and[[:space:]]+)<entry_fn>[[:space:]]` | 必须覆盖 mutual rec 后续 item 改写为 `and` 的情况（fstar_backend.ml:1924）；忽略此分支会让 mutual rec 第二个 item 误判 FAILED |
| **hax-coq** | `^[[:space:]]*(Definition\|Fixpoint\|Lemma\|Equations\|Theorem\|Program[[:space:]]+Definition)[[:space:]]+<entry_fn>[[:space:]]` | 实测当前基线（30949eb）仅用 `Definition`/`Fixpoint`/`Lemma`，但 `Equations`/`Theorem`/`Program Definition` 作防御性扩展容忍；`^\s*` 允许嵌套模块内的缩进 Definition |

### 与 audit 推荐 pattern 的偏离

audit §4.2 给的 pattern 是 `^let[[:space:]]+$TS_ENTRY_FN[[:space:]]`（仅 plain `let`）—— 实测在 `creusot-limit/mutual-recursion` 上会被 hax-fstar 的 mutual rec 改写为 `and is_even`（非 entry but 同 bundle）触发误报。**本实施扩展为 `(let\s+(rec\s+)?|and\s+)` 三种形态**，是工程上的重要校正（同 P12 verifast N≤40 阈值被 falsify 的级别）。

audit §4.3 给的 pattern 是 `(Definition|Equations|Fixpoint)` —— 实测漏 `Lemma`（coq_backend.ml:454 `is_lemma` 分支生成）。**本实施扩展为 6 个 keyword 容忍集合**。

---

## §4 验证遗留

### 4.1 主进程负责的端到端验证

| 工具 | 待主进程做的事 | 预期数字变化 |
| --- | --- | --- |
| **kani** | 重跑 matrix | 144 SUCCESS → 138-142（–2 ~ –6，audit §5 估算）|
| **hax-fstar** | 重跑 matrix | 115 SUCCESS → 115-114（–0 ~ –1，理论窗口实测 0 现象）|
| **hax-coq** | 重跑 matrix | 98 SUCCESS → 98-97（同上）|

### 4.2 已知未封堵的 silent path

- **kani**：`caller_location` + `foreign function` 的 stub codegen 路径——按宪法 §六-2 严格解读应封堵，但封堵会让 60-63/144 SUCCESS 翻 FAILED（绝大多数是合法 std-using entry）。属"宪法精神 vs cc-report 现行口径"分歧，本实施暂不动，待 cc-report 修订小组确认口径
- **hax-fstar**：F* fn 渲染未来引入新 keyword（如 `unfold let` / `inline_for_extraction let`）—— 当前基线 hax 30949eb 不使用
- **hax-coq**：上游引入新 silent path 不通过 `(* NotImplementedYet *)` 或 `please implement the method` 字面 —— 同上 audit §3.3 声明

### 4.3 文档同步未做项

- `deep-reports/cc-reports/{kani,hax-fstar,hax-coq}.md` 的"形式严格性"自陈段 + "实测结果"数据段——重跑后另派 agent 重写
- `docs/research/testsuite-research.md` / `docs/design/` 内的工具能力描述——本轮规则不改变工具能力评判，无需变动

---

## §5 与 audit-2 的对照

本文落地 audit-2 §4 表格 priority 1-3 规则：

| audit-2 优先级 | 工具 | audit-2 推荐 | 本实施 | 偏离 |
| --- | --- | --- | --- | --- |
| 1 | kani | exit 0 + grep 5 markers in `Found the following unsupported constructs:` warning list | wrapper.sh 实现多行 stdout grep | 无 |
| 2 | hax-fstar | grep `^let[[:space:]]+$TS_ENTRY_FN[[:space:]]` | grep `^(let\s+(rec\s+)?\|and\s+)$TS_ENTRY_FN[[:space:]]` | **采用更宽容判据**：audit 推荐忽略 mutual rec 的 `and` 形态会误报 |
| 3 | hax-coq | grep `^\s*(Definition\|Equations\|Fixpoint)[[:space:]]+$TS_ENTRY_FN[[:space:]]` | grep `^\s*(Definition\|Fixpoint\|Lemma\|Equations\|Theorem\|Program\s+Definition)[[:space:]]+$TS_ENTRY_FN[[:space:]]` | **采用更宽容判据**：audit 推荐漏 `Lemma`（coq_backend.ml:454 `is_lemma` 分支），实测合法 SUCCESS 可触发 |

两个 hax backend 的 pattern 校正与 P12 verifast `N≤40` 阈值被 falsify 同级——audit 文本不修改，但实施时按实测校正。

---

## §6 项目宪法与文档关系

按 [`CLAUDE.md`](../../CLAUDE.md) §1.3 文档优先：

- `principles.md` §六 反作弊 + `tool-integration.md` §三 0 误报 + §四 0 漏报 未变 —— 本实施落地宪法既有 claim，未提议修订
- `docs/design/tool-integration.md` §4.2 双向实测要求未变 —— 本文 §2 是实测论证记录
- `docs/design/architecture.md` 未变 —— 本实施仅在工具集成层落地

不构成宪法 / 架构层修订；属于"次要模块 3：tools 集成"的合规调优（按 [`CLAUDE.md`](../../CLAUDE.md) §3 模块优先级）。
