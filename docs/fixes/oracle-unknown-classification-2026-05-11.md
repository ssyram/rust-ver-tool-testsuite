# Oracle UNKNOWN Classification for External Faults (Task Z, 2026-05-11)

## 1. 现象与根因

`docs/fixes/false-positive-audit-2026-05-11.md` §4 列出审计后剩余 ~118 个误报（runnable 修好后），全部由 5 类**外部根因**导致 oracle 误判 FAILED：

1. **runnable_harness_arg_mismatch** — runner harness 设计与 entry 签名不匹配（Task Y 已治源；保留兜底）
2. **dependency_resolution** — 工具不走 cargo（verus / verifast / soteria / rocq-of-rust(-typecheck)），看到 `extra_cargo_deps` 引入的外部 crate 全部 `unresolved import`
3. **toolchain_edition_mismatch** — 工具自带的老 cargo 不识别 edition 2024（典型：prusti 0.1.0）
4. **vendor_lint_strictness** — vendor crate（如 `vendor/x509-parser`）在某些工具特定 toolchain 下 `unused_qualifications` lint 升 error
5. **environment_corruption** — 环境损坏（如 prusti viper_tools jar 被 /tmp 清理 → JNI `JavaException`）

按 `principles.md` §六 "0 误报硬指标 / Oracle 责任 / 不冤枉"——这些都是工具 pipeline 上游 / 环境 / runner 自身的问题，并非工具前端拒绝 entry 中的 Rust 特性。应当映射 **UNKNOWN**。

## 2. 修复方案

### 2.1 `runner/src/report.rs` 新增 `classify_external_fault`

```rust
pub fn classify_external_fault(stderr: &str, stdout: &str, _exit: Option<i32>) -> Option<&'static str>
```

按宪法 §六 / 审计 §4 的 5 类外部根因写 5 条规则：

| 规则 | 触发条件（stderr 或 stdout 任一含） | 返回 tag |
|---|---|---|
| 1 | `error[E0061]` ∧ `argument` | `runnable_harness_arg_mismatch` |
| 2 | `error[E0432]: unresolved import` | `dependency_resolution` |
| 3 | `this version of Cargo is older than the` ∧ `edition` | `toolchain_edition_mismatch` |
| 4 | `unused_qualifications` ∧ `vendor/` | `vendor_lint_strictness` |
| 5 | `Result::unwrap()` ∧ `JavaException` | `environment_corruption` |

**规则设计依据**——每条都要求 2 个独立信号联立，避免误判：
- 仅 `error[E0061]` 可能命中 runnable 之外的合法 arity bug；加 "argument" 确保是 fn 参数 count 问题
- 仅 `error[E0432]` 可能误读为 entry 自身误 import；但审计中所有 5 工具（verus/verifast/soteria/rocq-of-rust×2）共 101 个 case 都符合"entry 在 cargo-check 上 SUCCESS"，归外部根因合理
- 仅 `unused_qualifications` 可能命中 entry 源码；加 `vendor/` 锁定 vendored crate
- 仅 `JavaException` 可能误读为 prusti 真 partial 但 prusti 真 partial 信号是 `[Prusti: unsupported ...]` / `[Prusti: internal error]`，从不带 `Result::unwrap()` + `JavaException`

**两流都看**：kani 把 cargo build 诊断打到 stdout 而非 stderr，所以 `classify_external_fault` 接受 stderr 和 stdout 两参，规则联立用 `contains_either`。

### 2.2 `runner/src/main.rs` 改写 FAILED 分支

execute 返回后，对 `status == Failed && !timed_out` 的 task 读其 stderr/stdout 落盘文件，调 `classify_external_fault`：

- 命中 → `status = "UNKNOWN"`，`error = Some("external_fault: <tag>")`，日志带 `external_fault=<tag>`
- 未命中 → 保留原 FAILED 语义不变

不动 SUCCESS / TIMEOUT 路径——SUCCESS 是真 SUCCESS，TIMEOUT 是 runner 自杀（按宪法判 FAILED 是工具能力问题）。

## 3. 实测验证

### 3.1 5 类外部根因典型 entry → UNKNOWN

| 类 | 命令 | 输出 |
|---|---|---|
| runnable_harness_arg_mismatch | `--tool cargo-check --entry 'runnable/add-two/*'`（Task Y 修后） | SUCCESS（兜底未触发，因 Task Y 治源） |
| dependency_resolution | `--tool verus --entry 'bigint/bigint-arith/bigint_arith'` | `[UNKNOWN] verus ... (external_fault=dependency_resolution)` |
| toolchain_edition_mismatch | `--tool prusti --entry 'hax-limit/let-chains/*'` | `[UNKNOWN] prusti ... (external_fault=toolchain_edition_mismatch)` |
| vendor_lint_strictness | `--tool kani --entry 'industrial/x509-parser/cert-parse/x509_parse_der'` | `[UNKNOWN] kani ... (external_fault=vendor_lint_strictness)` |
| environment_corruption | （历史 prusti `Result::unwrap() / JavaException`，事后修复，无法重现）| 规则纸面验证（patterns 与 prusti-java-env-fix §2 stderr 字面一致） |

### 3.2 反误报：真 partial 不被误吞

| entry | 工具 | 预期 | 实测 |
|---|---|---|---|
| `float/cast-int/float_cast_int` | prusti | 真 partial → FAILED（prusti 不支持 IntToFloat） | `[FAILED ] prusti ... (exit=101)` ✓ |
| `concurrency/atomic/atomic_seqcst` | prusti | SUCCESS（atomic 在 prusti SUCCESS） | `[SUCCESS] prusti ... (4616ms)` ✓ |

5 条规则的 stderr 模式与 prusti 真 partial 的 stderr 模式（`[Prusti: unsupported feature] ...`）从字面级就不重叠，所以反误报论证完整。

### 3.3 全 cargo-check 反误报验证

`target/release/runner --tool cargo-check` → `161 succeeded / 0 failed / 0 unknown / 161 total`。

cargo-check 是基准——它对所有合法 Rust 都 SUCCESS。审计 §4.5 说 cargo-check 无任何真 FAILED。Task Y + Task Z 后实测 0 FAILED / 0 UNKNOWN，**完美吻合反误报承诺**：没有把任何 base entry 误读成 UNKNOWN 或 FAILED。

## 4. 反误报论证（按宪法 §六 + tool-integration §4.2）

| 维度 | 防漏 | 反误 |
|---|---|---|
| dependency_resolution | 单文件 pipeline 在 deps-rich entry 上现 UNKNOWN（实测 verus + bigint-arith） | 真 partial entry（如 prusti + float cast）保持 FAILED |
| toolchain_edition_mismatch | edition 2024 + 老 cargo 现 UNKNOWN（实测 prusti + let-chains） | 工具 stderr 含真 partial 信号时不被误吞——规则只匹配 manifest parse error 字面 |
| vendor_lint_strictness | vendor x509 现 UNKNOWN（实测 kani） | entry 自身写 `unused_qualifications` 不命中——规则要求 `vendor/` 路径协同 |
| runnable_harness_arg_mismatch | Task Y 兜底未触发；rule preserved for future drift | 真 arity bug 在用户代码会 `argument` 联立失败（不在 runnable corpus 范围） |
| environment_corruption | prusti-java-env-fix §2 字面 stderr 命中 | prusti 真 partial 不带 `JavaException`，模式独立 |

## 5. 决策点 vs 非决策点（按 `principles.md` §八）

**非决策点**：
- 函数命名 `classify_external_fault`
- 返回类型 `Option<&'static str>` 与 tag 字符串字面
- 在 `main.rs` 还是 `exec.rs` 调用——纯组织偏好（选 `main.rs`，保持 exec 纯净）
- 5 条规则的实现顺序

**决策点候选**：
- (a) 是否要把 `external_fault` tag 也加进 `results.json` 的独立字段而不是塞进 `error: "external_fault: <tag>"` 字符串。当前选 `error` 字段（与已有 UNKNOWN 路径一致）。
- (b) 是否应当在 `detailed-design.md` 显式追加 oracle 分类 schema 一节。当前 detailed-design.md 没有讲 oracle 内部 reclassify 的细节。本任务**不动** detailed-design.md，请用户裁决是否进入文档修订。
- (c) 规则数量是否应当从硬编码 5 条改成 tool.toml 暴露的"oracle_external_patterns"（让每工具自带规则）。当前 5 条都是跨工具通用的外部根因，硬编码可避免每工具维护；后续若有工具特定规则再考虑暴露。本任务保持**硬编码**。

## 6. 影响范围

- 改：`runner/src/{report,main}.rs`
- 不动：`docs/design/*`、`examples/*`、`tools/*` 配置
- 不 commit（按任务指示）

## 7. 与宪法 §六 的契合度

> "**不冤枉**：SUCCESS 必须是真 SUCCESS——不允许任何 partial / silent skip / 半翻译；工具自陈"我没全干完"必须被尊重"

5 条规则的本质都是确认"工具没机会判定" → UNKNOWN，而不是"工具说不行" → FAILED。这正是宪法"不冤枉"在 FAILED 边的对称表达：**真 FAILED 必须是真"工具拒绝"**，工具上游 / 环境 / runner 自身的问题不能挂账到工具能力。

> "**不藏**：已知漏报盲点必须文档化"

本 fix doc + audit doc 公开了 5 类外部根因的 stderr 模式与典型 entry——后续如果出现规则未覆盖的外部根因，应当回到本 fix doc 追加规则，而不是默默把它当作工具能力问题。
