# Oracle 漏报封堵规则实施 + 反误报论证（2026-05-08）

> 落地 [`docs/fixes/oracle-leak-audit-2026-05-08.md`](./oracle-leak-audit-2026-05-08.md) §4 推荐封堵规则。本文是正式实施记录 + 反误报论证。
>
> 按 [`docs/design/tool-integration.md`](../design/tool-integration.md) §4.2 双向实测要求：每条新规则必须同时验证（i）已知 silent path → 命中（防漏报）；（ii）合法 SUCCESS → 不命中（反误报）。
>
> 实施未跑全 matrix；重跑由主进程负责。本文不改 `cc-report` 数据段 / `internal-roundup`（等重跑后另派 agent 重写）。

---

## §1 实施清单

| 工具 | 改动文件 | 规则摘要 | 风险等级 |
| --- | --- | --- | --- |
| **verifast** | `tools/verifast/tool.toml`<br>`tools/verifast/verifast-strict-wrapper.sh`（新建）<br>`tools/verifast/oracle-validation/{spec_less_baseline.rs, spec_bearing_add_one.rs, README.md}`（新建）<br>`tools/verifast/README.md` | 新规则：exit 0 后用 `-verbose 1` 输出 grep 用户源文件路径 `src/lib.rs(`；0 命中 → 重写为 exit 2 + stderr 诊断 | 极高（封堵 116 SUCCESS 全空过） |
| **prusti** | `tools/prusti/tool.toml`<br>`tools/prusti/prusti-strict-wrapper.sh`（新建）<br>`tools/prusti/README.md` | 新规则：cargo-prusti exit 0 后必有 ≥ 1 个 `target/verify/log/viper_program/*.vpr`；0 个则重写为 exit 1 | 中（README §55 写过的语义首次落地） |
| **rocq-of-rust** | `tools/rocq-of-rust/tool.toml`<br>`tools/rocq-of-rust/README.md` | 第 6 道门：`grep -rE '^[[:space:]]*Definition[[:space:]]+${TS_ENTRY_FN}[[:space:]]' rocq_translation` 必须命中；不命中 → FAILED | 中（封堵 entry_fn silent skip 类） |

总改动：3 个 tool.toml + 2 个新 wrapper + 3 份 README + 2 份 micro-test + 1 份 oracle-validation README。

---

## §2 反误报论证（按 tool-integration.md §4.2）

### 2.1 verifast

#### 新规则文本

`tools/verifast/verifast-strict-wrapper.sh`：用 `-verbose 1` 包装原 verifast 调用；exit 0 后 grep 输出中匹配 `src/lib.rs(` 的行数。0 → 重写 exit 2 + 诊断。

```bash
"$VERIFAST_BIN" -verbose 1 -target macOS -shared -skip_specless_fns \
    -ignore_unwind_paths -disable_overflow_check \
    -read_options_from_source_file src/lib.rs > out 2>&1
rc=$?; cat out
[[ $rc -eq 0 ]] || exit "$rc"
user_lines=$(grep -cE 'src/lib\.rs\(' out)
if [[ "$user_lines" -eq 0 ]]; then
    echo "[verifast-oracle] FAIL: vacuous pass — symex executed 0 statements in src/lib.rs" >&2
    exit 2
fi
exit 0
```

#### 防漏报论证（已知 silent path → 命中）

证据 1：corpus 现状全空过 116/116 SUCCESS（详 audit §3.1）。所有 entry 均为 spec-less，`-skip_specless_fns` 导致 verifast 只 verify 自己的 prelude（`<install>/bin/rust/rust_belt/lem_aux.rsspec`），verbose 输出中**不出现** `src/lib.rs(` —— 实测：

```
$ verifast -verbose 1 ... src/lib.rs   # spec_less_baseline.rs as src/lib.rs
0.026 s: lem_aux.rsspec(3,5-13): Verifying function 'u32_sync'
...   (only prelude lines)
0 errors found (37 statements verified)
$ grep -c 'src/lib\.rs(' <output>   # 0
```

→ 规则命中（exit 2 重写）。

证据 2：审计推荐的"N ≤ 40"阈值原本不充分。实测最小 spec-bearing 实例（`fn foo //@ req true; //@ ens result == 42; { 42 }`）N=39，落入 spec-less 区间 {37..40} 内 —— 阈值会误报。**verbose grep 替换 N 阈值是更稳健的判据**（解释见 §2.1 反误报）。

#### 反误报论证（合法 SUCCESS → 不命中）

构造样例：[`tools/verifast/oracle-validation/spec_bearing_add_one.rs`](../../tools/verifast/oracle-validation/spec_bearing_add_one.rs)（`fn foo //@ req true; //@ ens result == 42; { 42 }`，spec 形式 lifted from verifast 26.01 自带 `tests/rust/preprocessor_test_crlf_bom.rs`）。

实测（2026-05-08，verifast 26.01 binary）：

```
$ cd /tmp/vf-test-spec   # spec_bearing_add_one.rs copied to src/lib.rs
$ VERIFAST_BIN=... bash verifast-strict-wrapper.sh
... 113 lines of verbose output ...
0 errors found (39 statements verified) (target: arm64-apple-macosx (LP64))
$ echo $?
0   ← wrapper accepts ✓
```

stdout 包含 10 行 `src/lib.rs(LINE,COL-LINE,COL):` 形式的 verbose 输出（`Verifying function 'foo'` / `Executing statement` / `Leak check.` / etc.），grep 命中 → 规则不触发 → 透传 exit 0 ✓。

理论补充：spec-bearing fn 必有至少一个 `//@ req` 才能逃过 `-skip_specless_fns`；一旦未跳过，verifast 必走 prototype-implementation-check + symex 路径，每条都打 `<source-path>(LINE,COL):` 标签到 verbose 输出。**有 spec → 必有 user-file mention**，由 verifast 设计强制保证；规则 reject 条件（0 mention）对真 SUCCESS 不可达。

#### 端到端 runner 验证（2026-05-08）

通过 runner 跑 `hello/basic-hello/hello`（spec-less corpus 典型样例）：runner 报告 **3861 ms FAILED, exit=2**，stderr 含完整 `[verifast-oracle] FAIL: vacuous pass — symex executed 0 statements in src/lib.rs` 诊断，stdout 末行仍是 `0 errors found (37 statements verified) (target: arm64-apple-macosx (LP64))`。耗时 3861ms 是 verbose 模式下 verifast 跑完 prelude verify + 流式 trace 的物理量级（vs 旧 oracle 207ms median），与"完整跑过 prelude"语义一致。规则按预期触发。

#### 信心等级

**high**。判据基于 verifast 自身的 verbose tracing（每条 symex 步骤都带源文件 tag），不是脆弱的字符串模式；双向实测 + 设计论证均通过。

#### 阈值/路径校准

- 路径锚点 `src/lib.rs(`：runner 始终把工作副本的 lib 放在 `src/lib.rs`，verifast 命令行就用这个相对路径，verbose 也以此路径打 tag。如 runner 改 layout，wrapper 需同步改锚点。
- Verbose volume：本 corpus 单 entry ~110 行 verbose 输出，体积可控，不会冲爆 raw stdout 文件。

### 2.2 prusti

#### 新规则文本

`tools/prusti/prusti-strict-wrapper.sh`：跑 `arch -x86_64 cargo-prusti`，exit 0 后必须 `find target/verify/log/viper_program -name '*.vpr' | wc -l ≥ 1`；0 个 → 重写 exit 1 + 诊断。

```bash
arch -x86_64 "$CARGO_PRUSTI"
rc=$?
[[ $rc -ne 0 ]] && exit "$rc"
vpr_count=$(find target/verify/log/viper_program -name '*.vpr' 2>/dev/null | wc -l | tr -d ' ')
if [[ "$vpr_count" -eq 0 ]]; then
    echo "[prusti-oracle] FAIL: cargo-prusti exited 0 but produced 0 .vpr files" >&2
    exit 1
fi
exit 0
```

#### 防漏报论证（已知 silent path → 命中）

理论 silent path：未来 toolchain 升级或 prusti commit 演进可能让 encoder 走 fast-path 跳过 lower（不写 .vpr）但仍 exit 0。`PRUSTI_DUMP_VIPER_PROGRAM=true` 在 commit `a0681ee` 是 unconditional dump after encode；但**没有形式保证未来 commit 不引入 fast-path**。

实测验证：当前实施在 run-1778226613-5282 的 56 个 SUCCESS 上 0 漏报现象（所有 SUCCESS 都 ≥ 13s 真跑 encoder），但**理论窗口存在**——README §"形式严格性"承认这点（"NEW config 下 encoder 真跑"是 commit-locked claim）。规则把这个 claim 从 README 文字提升为可执行 oracle。

→ 已知 silent path 类型（commit drift / config 失效）→ 命中条件（vpr_count==0）。

#### 反误报论证（合法 SUCCESS → 不命中）

理论分析：在 commit `a0681ee` + 当前 NEW config（`PRUSTI_NO_VERIFY=false` + `PRUSTI_DUMP_VIPER_PROGRAM=true` + `PRUSTI_PRINT_HASH=true`）下：

- `PRUSTI_NO_VERIFY=false` → 进入 `verify(env, def_spec)` 的 `Encoder::process_encoding_queue` 全程
- `PRUSTI_DUMP_VIPER_PROGRAM=true` → encoder 完成后无条件写 `.vpr` 到 `target/verify/log/viper_program/`（`prusti-server/src/process_verification.rs`）
- `PRUSTI_PRINT_HASH=true` → dump 完成后才 `return Success`，绕过 Silicon

任何 exit 0 路径必经过 dump 站点 → 必有 ≥ 1 .vpr。
规则 reject 条件（exit 0 + 0 .vpr）在当前 config 下**不可达**。

实测证据：run-1778226613-5282 下 56 个 SUCCESS 全部 wall ≥ 13s（avg 19907ms / median 18802ms / min 13097ms）—— JVM bootstrap + encoder 确实跑了；按 README §"对比"表的旧 vs 新 config 时长对比，新 config 比旧多 ~1.2s 是 encoder + JVM bootstrap，与"真跑 encoder"语义一致。

#### 信心等级

**high**。论证基于 commit-pinned 源码语义（`a0681ee`），双向都通过：reject 条件构造性不可达；防漏报覆盖未来 drift 的窗口。

#### 主进程实测遗留

prusti 二进制本地存在但 JVM bootstrap 较重（~13s/entry × 146 = ~30 min × 配置时间），**未在 agent 内单跑端到端 wrapper 验证**；主进程重跑 matrix 时即可一并验证（预期 56 SUCCESS 数字不变，行为等同当前实施）。

### 2.3 rocq-of-rust

#### 新规则文本

`tools/rocq-of-rust/tool.toml` 第 6 道门：在已有 5 道 grep guard 后追加：

```sh
elif [ -n "$TS_ENTRY_FN" ] && \
     ! grep -rqE "^[[:space:]]*Definition[[:space:]]+$TS_ENTRY_FN[[:space:]]" rocq_translation; then
    echo "[rocq-oracle] FAIL: entry_fn '$TS_ENTRY_FN' missing from .v products" >&2
    rc=1
fi
```

`TS_ENTRY_FN` 由 runner 在 `runner/src/exec.rs:178` 强制注入到 child env（不被 TS_* strip 清理）。允许 `^[[:space:]]*Definition` 匹配嵌套模块内的 fn（防止用户把 fn 写在 `mod inner { ... }` 内被拒）。

**runner expand_env 陷阱**：tool.toml 内必须用 `$TS_ENTRY_FN`（无 `{}`）而非 `${TS_ENTRY_FN}`。runner 的 `expand_env` 把 `${VAR}` 形式在 TOML parse 时展开为运行时 env var；但 `TS_ENTRY_FN` 在 runner 启动时不存在（仅在 spawn child 前才注入），所以 `${TS_ENTRY_FN}` 会被展开为空串，让 grep regex 失效。`$TS_ENTRY_FN` 形式对 runner 不展开（只匹配 `${...}`），交给 sh 执行时的 shell 展开。

#### 防漏报论证（已知 silent path → 命中）

实测 2026-05-08：

| corpus 模拟 | 结果 |
| --- | --- |
| `pub fn my_entry_fn() -> u32 { 42 }`（合法） | 产物含 `Definition my_entry_fn ...`，gate 6 通过 ✓ |
| `pub use std::option::Option as my_use_alias;` 作 entry | 产物为 0-byte .v，gate 3 已拦（不到 gate 6） ✓ |
| `macro_rules! my_entry_fn { ... }` 作 entry | 产物为 0-byte .v，gate 3 已拦 ✓ |
| `extern crate alloc;` + `pub fn other_fn` 但 entry=`alloc` | 产物含 `Definition other_fn` 但**无** `Definition alloc`，gate 6 命中 → FAILED ✓ |
| `mod inner { pub fn my_entry_fn() }` 嵌套 fn | 产物含 `  Definition my_entry_fn ...`（缩进），gate 6 通过 ✓（用 `^[[:space:]]*Definition` 而非 `^Definition`） |

→ ForeignMod / extern crate / use 这类 audit 标记的 silent skip 路径全部被前 5 道门或 gate 6 闭环。

#### 反误报论证（合法 SUCCESS → 不命中）

`TS_ENTRY_FN` 为空字符串时 grep 整个跳过（`[ -n "$TS_ENTRY_FN" ]` 短路）—— 不构成 false positive 路径。

合法 entry 必为 `examples/<feature>/<dir>/hirusttest.toml` 中 `entries = [...]` 列出的 fn 名，必为 `src/lib.rs` 顶层或嵌套模块内的 `pub fn`（按 corpus 约定）。rocq-of-rust 对每个 fn item 都生成 `Definition <name>` —— gate 6 grep 命中。规则 reject 条件（fn 名找不到 `Definition`）只在 fn item 被 silently skipped 时成立，与"合法翻译完成"互斥。

实测样例：上表 5 个用例覆盖（fn / use / macro / extern crate / nested mod），合法 fn 全部不命中规则。

补充实测：先前审计中可疑的 96ms `call_unshimmed_foreign_fn` 案 —— 实测 rocq-of-rust **正确**翻译了该 fn 为 `Definition call_unshimmed_foreign_fn`（产物含 `Parameter getpid` + `Definition call_unshimmed_foreign_fn`）。原本怀疑 ForeignMod 走 silent skip 是**错误假设**，本实施确认；speed 96ms 只是单文件解析的天然下限，不是空过物理证据。

#### 端到端 runner 验证（2026-05-08）

通过实际 runner 跑 2 个 entry 验证：

| entry | 预期 | 实际 | 状态 |
| --- | --- | --- | --- |
| `hello/basic-hello/hello`（合法 pub fn） | SUCCESS | runner 35ms SUCCESS ✓ | 反误报通过 |
| `miri-limit/ffi-unshimmed-extern/call_unshimmed_foreign_fn`（先前可疑） | SUCCESS（fn 真翻译） | runner 57ms SUCCESS ✓ | 反误报通过；audit hypothesis 证伪 |

#### 信心等级

**medium-high**。规则机制清晰；实测覆盖 5 类典型 silent skip 类型；剩余漏报盲点：rocq-of-rust 上游引入新 silent fallback 路径（README §"漏报盲点"已声明）。

---

## §3 验证遗留

### 3.1 主进程负责的端到端验证

| 工具 | 待主进程做的事 |
| --- | --- |
| **verifast** | 重跑 matrix。预期 116 SUCCESS → 0 (–116, –79.5pp)，所有 spec-less entry 命中 vacuous-pass 规则。stdout 体积 +~110 行/entry × 146 entries（可控） |
| **prusti** | 重跑 matrix。预期 56 SUCCESS → 56（数字不动；当前所有 SUCCESS 都已 ≥ 13s 真跑 encoder，必有 .vpr） |
| **rocq-of-rust** | 重跑 matrix。预期 121 SUCCESS → 100~120（–0~–21pp，待实测；audit §5.1 估计 –4~–15pp） |

### 3.2 已知未封堵的 silent path

- **verifast**：spec-bearing entry 中 verifast 内部某条 path 上 silent skip user fn 而仍 exit 0 + 写 verbose user-file lines —— 理论窗口；当前 corpus 无 spec entry 故未现身。规则在 spec corpus 上行为待实测。
- **prusti**：encoder 内部 silent skip 单个 fn item（不影响 .vpr 总数 ≥ 1）—— 当前规则只校验 .vpr 存在，不校验 entry_fn 是否在某个 .vpr 内。如需进一步封堵需 entry_fn-level grep（runner 已支持 `TS_ENTRY_FN` 注入；本次未做以避免越界）。
- **rocq-of-rust**：上游引入新 silent fallback 路径不带 5 个 marker 之一 —— README §"漏报盲点"已声明。

---

## §4 与 docs/fixes/oracle-leak-audit-2026-05-08.md 的关系

本文落地 audit §4 表格的 priority 1 / 2 / 3 规则：

| audit 优先级 | 工具 | audit 推荐 | 本实施 | 偏离 |
| --- | --- | --- | --- | --- |
| 1 | verifast | exit 0 + grep `(3[7-9]\|40) statements verified` ≤ 40 → FAILED | exit 0 + grep `src/lib.rs(` 计数 == 0 → FAILED | **采用更稳健判据**（N 阈值在 spec-bearing 最小例 N=39 上会误报，verbose grep 双向都通过实测） |
| 2 | prusti | exit 0 + `find ...vpr` ≥ 1 否则 FAILED | 同 audit | 无 |
| 3 | rocq-of-rust | grep entry_fn（如 runner 支持注入） | runner 已支持 `TS_ENTRY_FN` 注入；落地为 gate 6 | 无；audit 中"如 runner 不支持则跳过"的兜底未触发 |

**与 audit 的口径校正**：audit §4.1 给的 `(3[7-9]|40)` 阈值在我方追加 micro-test 实测时被**证伪**（spec-bearing 最小例 N=39 落入区间），改用 verbose user-file grep 判据。本文是工程上的重要校正，不修改 audit 文本本身（audit 已 commit 入 P11）。

---

## §5 项目宪法与文档关系

按 [`CLAUDE.md`](../../CLAUDE.md) §1.3 文档优先：

- `principles.md` §6 反作弊原则未变 —— 本实施落地宪法既有 claim，未提议修订
- `docs/design/tool-integration.md` §3 / §4.2 双向实测要求未变 —— 本文 §2 是实测论证记录
- `docs/design/architecture.md` 未变 —— 本实施仅在工具集成层（§5 工具集成边界）落地

不构成宪法 / 架构层修订；属于"次要模块 3：tools 集成"的合规调优（按 [`CLAUDE.md`](../../CLAUDE.md) §3 模块优先级）。
