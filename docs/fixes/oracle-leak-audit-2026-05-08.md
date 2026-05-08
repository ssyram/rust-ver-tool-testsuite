# Oracle 漏报审计 2026-05-08

> 审计 19 个工具的 oracle 漏报情况，识别"工具因短路机制让我们 entry 绕过真处理逻辑、但 oracle 判 SUCCESS"的虚高 SUCCESS。
>
> 数据基于 `runs/run-1778226613-5282/results.json`（146 entries × 19 tools，UTC 2026-05-08T07:50:13Z – 08:16:08Z）。
> 本报告**不修改任何 tool.toml / cc-report / 实施代码**，只产出审计结论与封堵规则建议。

---

## §1 TL;DR

**verifast 是单点最大漏报源，其他工具基本可控，但有 3 个工具 README 自陈"形式可证 0 漏报"实际有缺口**：

1. **verifast 79.5% 全是 vacuous pass**——116/116 SUCCESS 全部 ≤ 40 statements verified（其中 115 个为 37/38/39 baseline，仅 1 个 40），整 corpus 0 个 `//@ req/ens/inv/pred` 注解 + `-skip_specless_fns` 三件套，意味着**没有任何 SUCCESS 让 verifast 的符号执行触及我们的 entry 函数体**。子毫秒中位响应（207ms / max 1479ms）是不调 cargo + 不跑 prover 的物理证据。**虚高占比 ≈ 100%**。
2. **rocq-of-rust 82.9% 部分虚高**——121/121 SUCCESS 全部 < 500ms（98 个 < 200ms），oracle 仅看产物 ≥ 200 字节 + 5 个 marker 不命中，**完全 skip item 类（top_level.rs:349-390 直接 vec![]）+ 上游可能新增 fallback 路径**是 README 已承认的漏报盲点。**虚高占比 5-15%（待实测）**。
3. **hax 三个 backend 部分虚高**——hax-fstar/coq/lean 的 silent path grep 是"实测验证 0 漏报，不可形式证明"，README 已老实承认"hax engine 完全 skip item"是漏报盲点。**估计虚高占比 0-5%（实测 0 现象，但理论窗口存在）**。
4. **prusti 38.4% 较稳健但 tool.toml 缺产物存在性 check**——README §55 自陈 SUCCESS = "exit 0 且 .vpr 至少一个文件存在"，但当前 tool.toml 只看 exit code 0，缺产物检查；NO_VERIFY=false + DUMP_VIPER + PRINT_HASH 三件套强制 encoder 真跑，使 exit 0 必然伴随 .vpr 存在，**实际虚高占比 0%（理论窗口存在）**。
5. **其余 14 个工具（cargo-check / kani / miri / kmir / verus / soteria / creusot / charon×2 / aeneas×4）均无空过隐患**——cargo-check / kani / miri / charon 的 0 漏报是真形式可证；aeneas 4 个 backend 实测所有 unsupported 都通过 `craise` 走 exit ≠ 0；soteria exit 1/2/3 完整覆盖三类 partial；kmir 已用 `#EndProgram ~> .K` grep 抓 K-stuck（46/146 SUCCESS 全含此终止符号）。
6. **如果实施 verifast oracle 强化，verifast 数字将从 79.5% (116) 降到 0% (0)**——它在我们的 spec-less corpus 上**不存在任何真 SUCCESS 路径**。这是本次审计最重大的实施后果。
7. **优先级 top 3**：(1) verifast 加 vacuous-pass detection；(2) rocq-of-rust 加 entry function 翻译存在性 check；(3) prusti 补 .vpr 产物存在性 check（README 已写但未实施）。

---

## §2 审计方法

### 2.1 数据源

| 来源 | 用途 |
| --- | --- |
| `tools/<name>/tool.toml` | 当前 oracle 定义（command / success / fail 信号） |
| `tools/<name>/README.md` | 自陈的 0 漏报状态 |
| `tools/<name>/harness.rs.tera` | 入口注入是否会让 entry 函数被工具触及 |
| `runs/run-1778226613-5282/results.json` | 146×19 矩阵的 status / duration_ms / exit_code |
| `runs/run-1778226613-5282/raw/<tool>/<entry>.{stdout,stderr,exit}` | 物理证据（stdout 末尾 signature / stderr partial marker） |
| `deep-reports/cc-reports/*.md` | 每工具的 §"形式严格性"段当前自陈 |
| `deep-reports/internal-roundup-2026-05-08.md` | 已发现的 verifast 空过证据起点 |

### 2.2 漏报判定线索（优先级降序）

1. **stdout signature 同 baseline**（强证据）：N 个 SUCCESS 报告同一固定 signature（如 verifast `37 statements verified`），且该 signature 不依赖于用户代码体——必然空过。
2. **耗时分布双峰 / 子毫秒地板**（强证据）：SUCCESS 集合在 `< 200ms`、`< 500ms` 桶有显著占比，与"工具完整跑完 cargo + 翻译/求解"应有的 1-10s 量级不匹配——必然空过或单文件 fast-path。
3. **CLI flag 显式 skip**（强证据）：tool.toml 含 `-skip_specless_fns` / `--no-encode` / `--trusted` 等让某些条件下绕过处理逻辑的 flag——配合 corpus 没满足的条件 = 必然空过。
4. **tool.toml 缺产物存在性 check**（中证据）：README 自陈"产物存在 = 真处理"，但 tool.toml 实施只看 exit code——理论窗口存在，需双向实测。
5. **README 自陈"⚠️ 不可形式证明"**（弱证据）：实测 0 漏报，但 grep / 产物 marker 不能形式排除上游引入新 silent path。

### 2.3 风险大类分组

按"漏报机制类型"分组，而非按工具种类：

| 大类 | 机制 | 含工具 |
| --- | --- | --- |
| A. Spec-less skip | tool 自带 `-skip_specless_fns` 或类似 flag，无 spec 即跳过 | **verifast** |
| B. 单文件解析-only fast-path | 不读 Cargo.toml，rustc_interface 单文件解析 + 翻译，但产物可能不含 entry | **rocq-of-rust** |
| C. Silent product fallback | engine 内部对 unsupported 直接 emit `sorry` / `failure ((` / `NotImplementedYet`，不发 Diagnostic | **hax-fstar / hax-coq / hax-lean**（已部分封堵，仍有理论窗口） |
| D. 产物存在性 check 未实施 | README 写"exit 0 + 产物存在"，tool.toml 只看 exit code | **prusti**（理论窗口） |
| E. 单一 exit code | exit 0 = SUCCESS，工具自陈所有 unsupported 走 exit ≠ 0 | cargo-check / kani / miri / charon×2 / aeneas×4 / verus / creusot / soteria |
| F. exit + product grep（已实施） | exit 0 必要 + grep 终止符号 / silent marker 充分 | kmir / hax×3 / rocq-of-rust |

A、B、C 是本审计的重点；E、F 是已可证 / 已封堵的。

---

## §3 逐工具评估（19 个，按风险等级排序）

### 高风险 — A 类

#### 3.1 verifast 【高风险，确凿】

**当前 oracle**（`tools/verifast/tool.toml`）：

```toml
command = ["${TS_VERIFAST_BIN}", "-target", "macOS", "-shared",
  "-skip_specless_fns", "-ignore_unwind_paths",
  "-disable_overflow_check", "-read_options_from_source_file",
  "src/lib.rs"]
```

判定仅看 exit code（runner 默认 success() == exit 0）。

**短路机制**：`-skip_specless_fns` 让 verifast 跳过没 `//@ req` / `//@ ens` / `//@ inv` / `//@ pred` 注解的函数。

**corpus 实际命中**：`grep -rE '//@\s*(req|ens|inv|pred)' examples/` = **0 命中**。所有 entry 都是 spec-less。

**物理证据**：
- 116 SUCCESS 中 stdout 第一行分布：`104 个 "37 statements verified"` + `8 个 "38 statements verified"` + `3 个 "39 statements verified"` + `1 个 "40 statements verified"`（trait/cyclic-bound/cyclic_bound_use）。**115/116 是 baseline 之一**。
- 时长：avg 316ms / median 207ms / p10 179ms / p90 569ms / max 1479ms。**43/116 SUCCESS < 200ms，112/116 < 1000ms**。普通 cargo build 在 macOS arm64 上至少 1-2s，verifast 不调 cargo——但完整 rustc-verifast 解析 + IR 构造也至少需要数百毫秒。子毫秒中位响应只可能是"接受源码 + 立即跳过所有 fn"。
- 单 entry sample：`hello/basic-hello/hello` exit 0，stdout 仅一行 `0 errors found (37 statements verified) (target: arm64-apple-macosx (LP64))`，stderr 空。

**形式严格性现状**（cc-report verifast §0 漏报）：自陈"✅ 形式可证 0 漏报"，但 **README §55 同时承认**："SUCCESS 实际语义降级为 vacuous pass，**不**等于 verification 实际发生。这不是 oracle 的漏报，是 corpus 设计与工具特性使然"。

**审计判断**：cc-report 把"oracle 漏报"狭义定义为"符号执行真跑了但 oracle 错判 SUCCESS"，把 vacuous pass 归为"语义降级"而非"漏报"。但**项目宪法 §6 / `tool-integration.md` §4.2 反作弊原则**明确：测试目标是"工具完整处理我们的 entry 代码"——`-skip_specless_fns` 让 entry 完全没经过 verify 阶段，本质就是漏报。**这是 README 自陈与项目原则口径不一致的明确点名**。

**风险等级**：**高**。SUCCESS 数 116 中 ≥ 115 是空过（39 statements verified 的 trait-obj 两条也只比 baseline 多 2 statements，可能仍是 vacuous——需要对比 prelude 的 statement 数才能精确分母）。**估计虚高占比 ≈ 100%**。

---

### 中风险 — B 类

#### 3.2 rocq-of-rust 【中风险，待实测】

**当前 oracle**（`tools/rocq-of-rust/tool.toml`，5 道门）：

```sh
mkdir -p rocq_translation && rocq-of-rust translate --path src/lib.rs --output-path rocq_translation
# 后置 5 道 grep guard：
#   1. exit 0
#   2. find -name '*.v' -print -quit  非空（至少有 1 个 .v）
#   3. find -name '*.v' -size 0  数 == 0（无空文件）
#   4. find -name '*.v' -size +200c  数 ≥ 1（至少 1 个 > 200 字节）
#   5. ! grep -rE '\(\* (Error |Unexpected |Please report!|thir failed to compile|Unimplemented )' rocq_translation
```

**短路机制**：rocq-of-rust 设计上对所有 unsupported 走 `cargo warning` + 在产物里 emit `(* Unimplemented ... *)` 注释，**不通过 exit code 表达 partial**（永远 exit 0）。oracle 必须靠产物 marker grep。

**已知盲点**（README §"漏报盲点"）：
1. 上游引入新 silent fallback 路径不带已知 5 个 markers（实测 0 现象）。
2. 完全 skip item 类（`use` / `extern crate` / `macro_rules!` 在 `top_level.rs:349-390` 直接 `vec![]`）—— README 论证这些是合理 skip。

**物理证据 / 可疑信号**：
- 121 SUCCESS 全部 < 500ms（中位 129ms / max 465ms）。**98/121 < 200ms**。这是"单文件 rustc_interface 解析 + 翻译" 的合理量级（不读 Cargo.toml，不需 cargo build）。
- 最快 SUCCESS：`miri-limit/ffi-unshimmed-extern/call_unshimmed_foreign_fn` 96ms（含 `extern "C" { fn getpid() -> i32 }` ForeignMod item）。 **rocq-of-rust 是否对 ForeignMod 走 silent skip？** 这不在 README 列举的 use/extern crate/macro_rules! 三类内，**潜在漏报点**（需要双向实测验证）。
- 工业三件套 0/6（不读 Cargo.toml 死在 unresolved import）—— 这个不是漏报，是单文件模式天然死。

**形式严格性现状**：README 自陈"⚠️ 实测验证 0 漏报但不可形式证明"。

**风险等级**：**中**。**估计虚高占比 5-15%**（需要遍历 121 SUCCESS 的产物，确认每个 entry function 真有对应 `Definition` / `Module` 项）。

---

### 中风险 — C 类（已部分封堵但有理论窗口）

#### 3.3 hax-fstar 【中风险】

**当前 oracle**（`tools/hax-fstar/tool.toml`）：

```sh
cargo +nightly-2025-11-08 hax -C --lib ';' into fstar
# exit 0 后追加 grep：
grep -rE 'Rust_primitives\.Hax\.failure|failure \(\(' proofs/fstar/extraction/ 2>/dev/null
```

**短路机制 / 已知 silent path**：hax engine 在 unsupported 上有两条路径：
1. emit `FromEngine::Diagnostic` → cargo hax exit 1（官方信号，已抓）
2. silent path：fstar backend 直接 emit `Rust_primitives.Hax.failure` literal——cargo hax exit 0 但产物含 marker（grep 抓）

**物理证据**：avg 3285ms / median 1356ms / max 27070ms。SUCCESS 时长合理（cargo hax 真跑 cargo build + engine）。无可疑短路。

**形式严格性现状**：README 自陈"⚠️ 实测验证 0 漏报但不可形式证明"。已知漏报盲点：(a) hax engine 完全 skip item（实测 0 现象）；(b) 上游引入新 silent path 的可能。

**风险等级**：**中**（实测 0 现象，但理论窗口存在）。

#### 3.4 hax-coq 【中风险】

同 hax-fstar，grep `failure \(\(|please implement the method`。**额外注意**：`(* NotImplementedYet *)` 是 boilerplate header（每 .v 文件都有），oracle 已正确不抓。avg 3736ms / median 1938ms。

**风险等级**：**中**（同 hax-fstar）。

#### 3.5 hax-lean 【中风险】

grep 模式最复杂——`awk` 切 `--` 行注释，再 grep `(:=|pure|mk|,)\s*sorry\b|\bsorry\s*[,)\]]` 抓 term 位置 sorry，避免 binder 位置 `let sorry :` 误命中。avg 3870ms / median 1869ms。**现 oracle 已经做了 README §"形式严格性"承诺的最大封堵**。

**风险等级**：**中**（同 hax-fstar，silent path 集中在 `lean.rs:1287/2163` PatKind::Error / error_node）。

---

### 中风险 — D 类（产物存在性 check 未实施）

#### 3.6 prusti 【中风险，理论窗口】

**当前 oracle**（`tools/prusti/tool.toml`）：仅 exit code 0，**未检 .vpr 产物存在性**。

**已实施的封堵**：
- `PRUSTI_NO_VERIFY=false` → 进入 `verify(env, def_spec)` 路径，触发 `Encoder::process_encoding_queue`
- `PRUSTI_DUMP_VIPER_PROGRAM=true` → encoder 完成后写 `.vpr` 到 `target/verify/log/viper_program/`
- `PRUSTI_PRINT_HASH=true` → 在 dump 之后、`new_viper_verifier()` 之前 return Success，Silicon/Z3 永不启动

**README §55 自陈**："**前端接受**：exit code `0` 且 `target/verify/log/viper_program/*.vpr` 至少一个文件存在"。**但 tool.toml 当前 command 没 grep .vpr**——理论上若 encoder 在某 entry 上有 silent fast-path（如所有 fn 都 trusted），会出现 exit 0 + 0 个 .vpr。

**物理证据**：avg 19907ms / median 18802ms / min 13097ms。**全部 SUCCESS 都 ≥ 13 秒**——这与 cargo build + JVM bootstrap + encoder 真跑的量级一致。**56 SUCCESS 没有任何子毫秒响应**——物理上 prusti 真在跑 encoder。

**形式严格性现状**：README 自陈"✅ 形式可证 0 漏报，无漏报盲点（NEW config 下 encoder 真跑）"。

**风险等级**：**中**（实施完整封堵，README 已写完整规则但 tool.toml 未跟进；实测 0 漏报，但 NEW config 一旦因 toolchain 升级失效，缺产物 check 会让漏报无法察觉）。

---

### 低风险 — E 类（exit code 单一信号已足够）

#### 3.7 cargo-check 【无风险】

`cargo check --bin __ts_harness`。rustc 单一信号，exit 0 ⇔ type/borrow check 全过。**0 漏报形式可证，无 short-circuit**。

#### 3.8 kani 【低风险】

`cargo kani --only-codegen --bin __ts_harness`。harness 含 `#[kani::proof]` 注入，确保 codegen 触及 entry。

**README §32 自陈"⚠️ 部分可证"**：`Found the following unsupported constructs: caller_location / foreign function ...` 是 SAT 阶段才有意义的 warning，不影响 codegen 完成 / exit 0。**实测 corpus 上 SUCCESS entry 无此 warning**。

avg 3581ms / median 944ms / min 465ms / max 40023ms。无可疑信号。

**风险等级**：**低**（SAT 阶段 warning 路径理论存在，但 SAT 不在本测试范围）。

#### 3.9 miri 【无风险】

`cargo +nightly miri run --bin __ts_harness`。harness 调 entry，miri 解释执行；UB/unsupported → exit ≠ 0。**0 漏报形式可证，无 silent skip**。

#### 3.10 verus 【低风险】

`verus --no-verify --log vir --crate-type=lib src/lib.rs`。harness `mod __ts_inner` 在 `verus! {}` 块**内**，确保 inner 真经过 verus 前端（不退化为 stock rustc 透传）。

**潜在质疑**：verus 在 `charon-limit/async-fn/async_forty_two` 上 SUCCESS（exit 0 + warning unused future）。看上去像空过——但 verus README §15 明示"`--no-verify` 同时切 AIR + Z3，VIR 是最深前端表示"，verus 接受 async fn 进入 VIR 是它的真实边界，不是空过。

avg 700ms / median 637ms / min 421ms / max 1518ms / 6 个 < 500ms。**无任何子毫秒响应**——verus 真在跑（VIR 构造 + lifetime check）。

**风险等级**：**低**（harness 反作弊已确保 inner 在 `verus! {}` 块内；VIR 构造是真前端边界）。

#### 3.11 creusot 【低风险】

`cargo-creusot`（无 subcommand 即只翻译，不调 why3）。**0 漏报形式可证**：creusot 用 `crash_and_error / span_err / span_fatal` 把所有 unsupported 升级为 rustc error → exit 101，无 silent path。

avg 39363ms / median 38613ms / min 18885ms。**所有 SUCCESS ≥ 18 秒**——cargo build + creusot-rustc 替换 RUSTC 跑全 deps tree，物理上没有空过窗口。

**潜在隐忧（非漏报）**：tool.toml 没检查 `verif/<crate>_rlib/*.coma` 产物存在性（README §24 提议"可补强"）。但因为 creusot 用 rustc-error 显式拒绝路径覆盖，缺这个 check 不构成漏报。

**风险等级**：**低**。

#### 3.12 charon-mono 【无风险】

`charon cargo --monomorphize --abort-on-error --print-llbc -- --lib --target aarch64-apple-darwin`。

`--abort-on-error` 让任何 `register_error!` 立即翻 exit ≠ 0。**0 漏报形式可证**。avg 2394ms / median 350ms。

#### 3.13 charon-poly 【无风险】

同 charon-mono（去掉 `--monomorphize`）。

#### 3.14 aeneas-coq / 3.15 aeneas-fstar / 3.16 aeneas-lean / 3.17 aeneas-hol4 【低风险】

两段 pipeline（charon → .llbc → aeneas → product）。wrapper 仅看 aeneas 的 exit code。

**0 漏报形式可证**：aeneas 的 `Errors.error_list` + `craise` 把所有 unsupported 推 exit ≠ 0。aeneas-hol4 的 `extract_trait_decl Option.get None` panic 也是 exit 2，不是 silent。

aeneas-coq avg 3048ms / median 1467ms / min 801ms。无可疑信号。

**风险等级**：**低**（4 个 backend 同源 mid-end，`craise` 路径覆盖完整）。

---

### 低风险 — F 类（exit + product grep 已实施）

#### 3.18 kmir 【低风险】

`kmir run` 在 K-stuck 时 CLI 仍 exit 0。已用 `grep '#EndProgram[[:space:]]*~>[[:space:]]*\.K'` 抓终止符号——实测 46/146 SUCCESS 全部含此 signature（grep -l 验证 46 个匹配）。

**0 漏报现状**：sound（exit 0 + 终止符号 ⇔ K interpreter 真跑完）。

#### 3.19 soteria 【无风险】

exit 0 = SUCCESS，exit 1/2/3 三档完整覆盖 bug detect / symex crash / Charon crash。avg 1726ms / median 1120ms。**0 漏报形式可证**。

---

## §4 推荐封堵规则（核心）

### 4.1 verifast — 必须封堵

**问题诊断**：当前 oracle exit 0 → SUCCESS 在 spec-less corpus 上语义退化为 vacuous pass。**根因不在 oracle，在 tool.toml 的 -skip_specless_fns + corpus 0 spec 注解组合**。有两种方向：

**方向 A：去掉 -skip_specless_fns**（不推荐——会让所有 plain Rust 在溢出/panic 路径误判 FAILED，引入大量误报）。

**方向 B：在 oracle 端加 vacuous-pass detection**（推荐）：

```toml
# tools/verifast/tool.toml — 第一层硬规则（基于实测可上线）
command = [
  "sh", "-c",
  "${TS_VERIFAST_BIN} -target macOS -shared -skip_specless_fns -ignore_unwind_paths -disable_overflow_check -read_options_from_source_file src/lib.rs > /tmp/vf-out-$$ 2>&1; rc=$?; cat /tmp/vf-out-$$; if [ $rc -eq 0 ]; then if grep -qE '^0 errors found \\((3[7-9]|40) statements verified\\)' /tmp/vf-out-$$; then echo '[verifast-oracle] FAIL: vacuous pass — only prelude statements (≤ 40) verified, no user code reached symex' >&2; rc=2; fi; fi; rm -f /tmp/vf-out-$$; exit $rc"
]
```

- **防漏报场景**：所有 spec-less entry → 命中 baseline 区间 ≤ 40 → FAILED（正确：没经过真处理）
- **反误报场景**：未来引入 `//@ req/ens` 注解的 entry → statement count > 40 → SUCCESS（因为真 verified 用户函数 statements）—— 但**当前 corpus 不存在此场景**，需要加一个带 spec 的合法 entry 作为正向验证样例（待实测）
- **信心等级**：**high**（115/116 baseline + 0 spec 注解事实充分支撑）
- **数字阈值** 40 的依据：当前 corpus 最大 SUCCESS 是 cyclic_bound_use 的 40 statements，仍是 spec-less，所以 ≤ 40 的全部归 FAILED 是 sound 的；阈值需要在加 spec entry 后重新校准

**第二层 - 待实测验证软规则**：

```toml
# 子毫秒响应判空过（待实测）
fail_when.elapsed_ms_lt = 100   # verifast 真符号执行小 entry 至少需 100ms
```

- **防漏报场景**：未来若 verifast 再加新 short-circuit 让某些 entry 0ms 退出 → 命中
- **反误报场景**：合法 SUCCESS 通常 ≥ 200ms（参考 verifast 全 corpus min=139ms）—— **但 verifast min 已到 139ms，留 100ms 安全裕度**
- **信心等级**：**low**（不能形式排除 verifast 在某些极简 entry 上真实 < 100ms）

### 4.2 rocq-of-rust — 中等封堵

**问题诊断**：oracle 5 道 grep guard 已较严格，但**没检 entry function 是否真有翻译产物**。`call_unshimmed_foreign_fn` 96ms SUCCESS 是 ForeignMod item 的真实可疑信号——需要双向实测。

**第一层硬规则**：

```toml
# 在 5 道 grep guard 后追加：grep entry function 在产物里
# entry_fn 名字由 runner 注入（例如 "call_unshimmed_foreign_fn"）
# 注意：tool.toml 不支持 Tera，需要把 entry function 名透过环境变量注入或在 runner 阶段渲染
# 如果当前 tool.toml 框架不支持，则该项作为 runner 层的 oracle 增强提案
```

- **状态**：**待实测验证**（需要先确认 runner 是否能向 tool.toml command 注入 entry_fn 名字；当前 tool.toml 用 `{{ entry_fn }}` 在 harness.rs.tera 里渲染，但不在 tool.toml 内）
- **信心等级**：**medium**（如能实施会闭环掉 ForeignMod silent skip 风险）

**第二层软规则**：

```toml
# 子毫秒响应作为 ForeignMod / 完全 skip 类的额外信号（待实测）
# rocq-of-rust 当前 min 90ms — 阈值难定
```

- **信心等级**：**low**（rocq-of-rust 整体就是单文件 fast-path，子毫秒不是空过物理证据）

### 4.3 hax 三个 backend — 已基本封堵，软强化

**问题诊断**：silent path grep 已实施且 README 经过实测验证。剩余风险是"hax engine 完全 skip item"的理论盲点。

**第二层软规则**（待实测）：

```toml
# 同 rocq-of-rust：grep entry function 是否在产物里
# (待 runner 支持 entry_fn 注入到 tool.toml command)
```

- **信心等级**：**low**（hax engine skip item 实测 0 现象）

### 4.4 prusti — 补 .vpr 产物存在性检查

**问题诊断**：README §55 已写明 SUCCESS 信号 = "exit 0 + .vpr 至少一个存在"，但 tool.toml 未实施。

**第一层硬规则**：

```toml
# tools/prusti/tool.toml — 在 cargo-prusti 后追加产物 check
command = [
  "env",
  # ... existing env ...
  "${TS_PROJECT_ROOT}/tools/prusti/prusti-wrapper.sh"   # 新增 wrapper
]
```

`prusti-wrapper.sh`：

```bash
#!/usr/bin/env bash
set -euo pipefail
arch -x86_64 ${TS_CARGO_PRUSTI}
rc=$?
if [[ $rc -eq 0 ]]; then
    vpr_count=$(find target/verify/log/viper_program -name '*.vpr' 2>/dev/null | wc -l | tr -d ' ')
    if [[ $vpr_count -eq 0 ]]; then
        echo "[prusti-oracle] FAIL: cargo-prusti exit 0 but no .vpr produced (encoder fast-path?)" >&2
        rc=1
    fi
fi
exit $rc
```

- **防漏报场景**：未来若 prusti 因 toolchain 升级出现 encoder silent fast-path → 命中
- **反误报场景**：当前 56 SUCCESS 全部 ≥ 13 秒，确认 encoder 真跑 = 必有 .vpr —— 不会误命中
- **信心等级**：**high**（README §55 已经做完了形式论证，只需把它落到 oracle）

### 4.5 其他 14 工具

无需新增封堵规则。cargo-check / kani / miri / charon×2 / aeneas×4 / verus / creusot / soteria / kmir 的 oracle 当前都满足"exit code（+ 必要时 grep）= SUCCESS 信号充分"——形式可证或实测验证。

---

## §5 重跑 + 报告重写计划

### 5.1 实施推荐规则后预估数字变化

| 工具 | 当前 SUCCESS | 实施后估计 | 变化 |
| --- | --- | --- | --- |
| **verifast** | 116 (79.5%) | **0-2 (0-1.4%)** | **-79pp** |
| rocq-of-rust | 121 (82.9%) | 100-115 (68-79%) | -4 ~ -15pp（待实测） |
| hax-fstar | 115 (78.8%) | 113-115 (77-79%) | -0 ~ -2pp（实测 0 现象） |
| hax-coq | 98 (67.1%) | 96-98 (66-67%) | -0 ~ -2pp |
| hax-lean | 110 (75.3%) | 108-110 (74-75%) | -0 ~ -2pp |
| prusti | 56 (38.4%) | 56 (38.4%) | 0pp（实施只是补防御，预期 0 移动） |
| 其他 14 个 | 不变 | 不变 | 0 |

**关键变化点**：verifast **从矩阵第 6 强（79.5%）跌到垫底（0-1%）**——这是合理的，因为它在 spec-less corpus 上本就不应该有非 trivial SUCCESS。

### 5.2 cc-report 需要重写的关键段

1. **`tools/verifast/cc-report.md`**（同 deep-reports/cc-reports/verifast.md）：
   - §0 漏报段："✅ 形式可证" → "❌ 在 spec-less corpus 上 oracle 当前漏报率 ≈ 100%；新 oracle (statements verified ≤ 40 → FAILED) 已封堵"
   - §对应性表 / §测试结果：vacuous pass 116 → 真 SUCCESS 0-2
   - §"重要注（语义降级，不是漏报）" 段需要修订为 "经审计，按项目宪法 §6 反作弊原则，vacuous pass 即漏报；oracle 已封堵"

2. **`deep-reports/internal-roundup-2026-05-08.md`**：
   - 暴论 1 升级："verifast 79.5% 是空过率" → "verifast 79.5% 已被新 oracle 封堵到 0-1%"
   - §3D（model-check / symex 组）verifast 116 → 0-2，需要重排序
   - §6 时长表 verifast avg 316ms 注解需更新

3. **`tools/rocq-of-rust/cc-report.md`**（如实施 entry_fn grep）：
   - §0 漏报盲点段补充"ForeignMod silent skip 路径已闭环"

4. **`tools/prusti/cc-report.md`**（如实施 .vpr check）：
   - §0 漏报段从"无（NEW config 下 encoder 真跑）" 升级为 "无 + .vpr 产物存在性 hard check"

### 5.3 internal-roundup 数字更新

按 5.1 估计，重跑后总体矩阵：

- 工具总通过率排序 top 5 不变（cargo-check / kani / miri / charon-poly / charon-mono），第 6-8 名重排序（verifast 大跌 + rocq-of-rust 小跌）
- 7 条暴论中暴论 1 需要改写
- §6 时长表 verifast 行需要更新（新 oracle 加 grep 后约 +50ms）

---

## §6 实施优先级

按 "严重性 × 改动成本" 排序：

| # | 工具 | 严重性 | 改动成本 | 备注 |
| --- | --- | --- | --- | --- |
| **1** | **verifast** | 极高（116 SUCCESS 全空过） | 低（tool.toml 加 sh -c 包装 + 1 行 grep） | 改完后 verifast 数字会从 79.5% 跌到 ≈ 0%，是本次审计最重大的实施 |
| **2** | **prusti** | 中（理论窗口，当前实测 0 漏报） | 低（新增 prusti-wrapper.sh + 改 tool.toml command 引用） | README §55 已写完整逻辑，只需落地；改完后 prusti 数字预期不变 |
| **3** | **rocq-of-rust** | 中（96ms ForeignMod 可疑 SUCCESS） | 中（需 runner 支持 entry_fn 名注入到 tool.toml command） | 需要先调研 runner 是否支持注入；若不支持，先在 deep-reports 中标"已知盲点" |
| 4 | hax-fstar / coq / lean | 低（实测 0 现象，理论窗口） | 中（同 rocq-of-rust，需要 entry_fn 注入） | 与 #3 同改动路径，可一起做 |
| 5 | 其他 14 工具 | 0 | 0 | 不实施 |

---

## §7 审计观察 / 暴论

7.1 **README 自陈"形式可证 0 漏报"与项目宪法口径不一致的工具：仅 verifast 一家**。verifast README §55 用了一个语义滑移："这不是 oracle 的漏报，是 corpus 设计与工具特性使然"——把"oracle 错判 SUCCESS"的责任推给 corpus（"应该加 spec"）和工具（"应该不要 -skip_specless_fns"）。但项目宪法 §6 的反作弊原则明确："SUCCESS = 工具完整完成它的工作单元，不允许任何 partial / silent skip / 半翻译"——`-skip_specless_fns` 让 entry 函数被显式 skip，符合 silent skip 的定义。**这是 README 与宪法之间的口径漂移，应改 README，不改宪法**。

7.2 **真"全空过"工具只有 verifast 一家——其余 18 工具的低占比 SUCCESS 可疑路径（rocq-of-rust 96ms / hax silent skip 理论窗口 / prusti 缺产物 check）实测都 0 现象**。把 verifast 与其他工具的漏报风险并列是误导——verifast 是 100% 漏报，其他工具是 0-5% 理论窗口。

7.3 **耗时分布是判别空过最强的物理证据，胜过 stdout / 产物 grep**。verifast 中位 207ms 与所有其他真跑工具差 1-2 个数量级（charon-mono median 350ms / kani 944ms / verus 637ms / aeneas-coq 1467ms）。子毫秒级响应在工业级 toolchain 上几乎不可能是真完整跑——单是 cargo + rustc loader bootstrap 通常都要 100-300ms。

7.4 **"自陈 0 漏报"与"实测 0 漏报"的鸿沟**：cargo-check / kani / miri / charon×2 / aeneas×4 / verus / creusot / soteria / verifast / prusti / kmir 共 13 个工具自陈"✅ 形式可证 0 漏报"。其中 verifast 这条是错的（语义滑移），其余 12 个验证为真。**hax×3 + rocq-of-rust 共 4 个工具自陈"⚠️ 不可形式证明"——更诚实，且实测确实 0 现象**。**形式可证不是金标准，诚实标注 grep 边界 + 实测覆盖才是**。

7.5 **本审计催生的最大原则**：今后新增工具时，不要在 README 把"oracle 信号"与"测试目标对齐"分割来谈——verifast 就是这样把"vacuous pass"挡到了 corpus 设计层面。如果一个 SUCCESS 的语义不能直接映射到"工具完整处理了我们的 entry 代码"，它就是 oracle 漏报，应在 oracle 层面封堵。
