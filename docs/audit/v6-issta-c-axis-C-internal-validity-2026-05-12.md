# Axis C: Internal Validity (ISSTA 2026 reviewer 视角)

> Reviewer: 独立 challenger（跨 session AI），ISSTA 2026 PC 角色
> 框架: Wohlin et al. 2012 *Experimentation in Software Engineering* Ch.8.6 (Internal validity threats)
> 目标: commit `ebe6858` v6 final 测量结果的因果归属
> 协议: disprove-first；每条按反状态 R1/R2/R4/R8 自我排除一次后才入清单
> 范围: examples/161 entries × tools/20 × runner（discover/exec）× principles §四/§五/§六

---

## 1. 总览

- 候选数：**6 强 / 3 弱 / 4 自身排除**
- Wohlin Ch.8.6 涉及类别：Selection、Instrumentation、Mortality、Maturation
- 重点结论：runner 形式定义（双方不可侵入）已实现得很干净；真正可被 reviewer 攻击的 internal validity 风险集中在 **harness instrumentation 模板的工具间不对称** + **Oracle 切点选择（kani --only-codegen / verus --no-verify / bug-detect 仅 MIRI/soteria）的语义裁决权**

---

## 2. 候选清单

### 候选 1：harness.rs.tera 模板工具间不对称 → Instrumentation bias【强】

**事实**（grep 全 20 harness 模板）：
- cargo-check / kmir / charon-* / aeneas-* / hax-* / rocq-of-rust / kani：harness 仅 `fn main() { let _ = <crate>::<entry>(args); }`（或 `#[kani::proof]` 包一层），是最薄 instrumentation
- **verus**：`use vstd::prelude::*; verus! { mod __ts_inner; ... #[verifier::external] fn __ts_invoke() }`——把 user code 整个塞进 `verus! {}` 块
- **soteria**：`mod __ts_inner; pub use __ts_inner::*; fn main() { ... }`（entry_mode=lib）
- **prusti**：薄 harness（`fn main()` 调用 entry）+ tool.toml 注入 `PRUSTI_NO_VERIFY=false / PRUSTI_DUMP_VIPER_PROGRAM=true / PRUSTI_PRINT_HASH=true`

**问题**（Wohlin §8.6.2 Instrumentation）：模板差异是 instrumentation 改变受试者（user code）暴露给工具的形态。verus harness 强制 `use vstd::prelude::*` + `verus! {}` 块——这等价于把 user code rewrite 进 verus DSL。reviewer 会问：「verus 在 41 features 上的接受率，到底测的是 user code 还是 verus 把 user code 包进 verus! 后的形态？」

**对照精神**：
- principles.md §四 A 形式定义：「example 的 cargo 行为字节级一致」——这是 example **磁盘字面**层面的承诺，工作副本的 lib.rs 已经被 harness 改写
- architecture.md §一「A 位阶澄清」：A 已经从"天真 A"降级为「原始磁盘字面零修改 + 隔离副本上声明式工具特定填充」——但 verus harness 用 `entry_mode=lib` 把 src/lib.rs 整个替换、把原文件 inline 进 `verus! {}`，工具间这种"包进 DSL"的处理只对 verus 做了；reviewer 视角看：**形式定义满足，但工具间 instrumentation 强度不对称**

**反状态排除**：
- R2 排除：principles + architecture 明示承诺仅在「原始磁盘字面 + cargo 行为」层面，工作副本上的 harness 差异是 **deliberate design**（principle C 的字面：异质性归 tool.toml/tera 配置）——但 **reviewer 不被 deliberate design 约束**；reviewer 问的是测量结果归因，不是"你设计意图是否清晰"
- R1 适用：现象（模板差异）确实存在 + 缺陷（reviewer 可质疑 verus 的高接受率是 instrumentation 效应）成立

**建议处理**（不修宪法，补 threats to validity 章节）：测试报告必须明示「verus harness 把 user code wrap 进 `verus! {}`，对照工具 cargo-check / charon-* 无此 wrapper；如 verus 接受率高于 cargo-check，不可直接归因为 verus 前端更强」。可在 `tools/verus/README.md` 与 cc-report 阐述。

---

### 候选 2：Oracle 切点选择的工具不对称 → Selection bias on response variable【强】

**事实**:
- kani: `--only-codegen` 切到 GotoC codegen 后停（不调 CBMC）
- verus: `--no-verify` + `--log vir` 切到 VIR 构造后停（不调 Z3，AIR/Z3 同时被切——tool.toml 注明无独立 AIR-only flag）
- prusti: `PRUSTI_NO_VERIFY=false + DUMP_VIPER_PROGRAM=true + PRINT_HASH=true` 切到 Viper 程序 dump 后停（不调 Silicon/Z3）
- charon: 翻译到 LLBC 后停（charon 本无 SAT 后端）
- creusot: 完整 pipeline（900s timeout）
- aeneas / hax: 4 backend extraction（Coq/F*/HOL4/Lean）

**问题**（Wohlin §8.6.1 Selection + §8.6.6 Confounding constructs）：「前端 / 后端」切点对每个工具语义不同。kani 的 `--only-codegen` 是官方 supported flag，verus 的 `--no-verify` 也是；但 **prusti 的"前端"是用 PRUSTI_NO_VERIFY=false 反向操作 + dump 强制**——这是社区惯例外的姿势组合（虽然 docs/fixes 引用了源码确认）。reviewer 会问：「verus 的 VIR ≈ prusti 的 Viper ≈ kani 的 GotoC ≈ charon 的 LLBC——这种"前端深度对齐"的等价类如何独立校验？」

**对照精神**：
- principles.md §六「前端测量（深度切割）」+「测必须命中工具自身前端，而非 rustc 等代理前端」——精神是「不退化到 rustc parse」，**没明示「不同工具的前端深度等价」**
- principles.md §四 B 等距推论：「不区分翻译深浅」——但这条说的是 **接受率视角不加权浅深**，**不**保证切点选得对工具公平

**反状态排除**：
- R2 部分排除：principles §六 明示「前端测量」是 deliberate；但 **切点的具体选择**（如 prusti 用反向 flag）属于 detailed-design / tools README 层面工程实现，未在宪法 deliberate 化
- R1 适用：切点选择本质上是 measurement instrument 选择，工具间不对称 = Wohlin selection on instrument bias

**建议处理**：cc-report 必须列「每工具切点 + 切点是否社区惯例 + 切点是否官方文档化」表格，作为 internal validity threats to validity 子章节。可在 tools/<name>/README.md 增 "front-end cut justification" 段。

---

### 候选 3：bug-detect = SUCCESS（P35）只对 MIRI + soteria 触发 → 工具间 reward 不对称【强】

**事实**:
- P35（2026-05-12）：MIRI 与 soteria 的 wrapper 把"工具检测到 entry 的 UB / real bug"重写为 SUCCESS（exit 0）
- 其他 17 个工具：检测到代码"有问题"一律 FAILED（如 prusti 报 spec 违反 / kani 报 verification failure / creusot 报 SMT unsat / aeneas 报 well-formedness fail）
- 派生原则在 principles.md §四 B「测必要条件 / 不问语义对错」——bug-detect 属于「工具产物形态」

**问题**（Wohlin §8.6.6 Confounding constructs + §8.6.1 differential reward）：
1. 「bug-detect = SUCCESS」对 dynamic analysis（MIRI = 解释执行 + soteria = symbolic execution）天然成立——它们设计目标就是 bug-detect
2. 对 deductive verifiers（prusti / verus / creusot / kani 的 SAT 层）：它们的 bug-detect 模式（如 prusti 检测出 spec 违反）reviewer 视角等价 SUCCESS——但项目把这些归 FAILED
3. **不对称 reward 直接污染通过率**：MIRI 在 examples/error/ 等 entry 上若触发 UB 报告 → SUCCESS；同样 entry 给 kani 触发 verification failure → FAILED

**对照精神**：
- principles.md §四 B：「只问能否吃下产出预期形状」——"预期形状"在 §六 deliberate 化前不含 bug-detect 维度
- architecture.md §一「bug detect 归 SUCCESS」是 P35 在 architecture 层添加的、**未上宪法**——属于 architecture 派生
- 反 deliberate 论：P35 的派生只覆盖 MIRI + soteria 两工具的 wrapper 改造，其他工具是否应享有同等 reward 在 architecture / detailed-design 中**未给出对称性论证**

**反状态排除**：
- R2 不能完全排除：bug-detect = SUCCESS 已经 P35 deliberate 化（架构层）
- 但 R2 的范围只覆盖 MIRI/soteria，**对称性问题未被宪法 / 架构 deliberate**
- R1 + R8 适用：现象 = reward 不对称；建议 = 把对称性论证补到 architecture 或决定不补（明示其他工具不享有 reward）

**建议处理**（决策点）：要么宪法 / architecture 显式排除「deductive verifiers 的 bug-detect」（明示理由：例如 prusti 的 unsat 是 spec 缺失而非 user code bug），要么 P35 推广到 kani/prusti/verus/creusot 但补上"是否 user code bug vs spec gap"判定逻辑。当前裸 P35 给 reviewer 一个干净的攻击面。

---

### 候选 4：industrial/x509-parser 的 [env] RUSTFLAGS 注入 → Corpus 是否被 instrument 化【强】

**事实**（examples/industrial/x509-parser/cert-parse/hirusttest.toml）:
```toml
[env]
RUSTFLAGS = "--cap-lints=warn"
```
注释明示原因：vendor/x509-parser/src/lib.rs:121-123 有 `#![deny(unstable_features, unused_qualifications)]`，新版 rustc（kani / hax-* / aeneas-* pin 的更新 toolchain）触发 `unused_qualifications` lint → deny 升 error → cargo build 在工具前端看到 user code 前就 abort。

**问题**（Wohlin §8.6.4 Maturation + §8.6.2 Instrumentation）：
1. reviewer 会问：「为了让 corpus entry 在更新 toolchain 下能跑通，你修改了 entry 的 build env——这是 corpus 适配工具，违反双方不可侵入吗？」
2. 反观：单文件 pipeline 工具（verus / verifast / soteria / rocq-of-rust）在 x509-parser 上撞另一个 capability boundary（unresolved import x509_parser）——这意味着 **[env] 注入对工具子集（cargo-shelling）有效、对另一子集（单文件 pipeline）无效**，工具间 differential effect
3. 派生 principles.md §四 A 形式定义检查：A 形式承诺「example 的 cargo 行为字节级一致」——`RUSTFLAGS=--cap-lints=warn` **改 cargo 行为**了，只是改的是 vendor crate 的 lint level 而非 user code 字节

**对照精神**：
- principles.md §六 UNKNOWN 严格语义「我们 corpus 引入的 vendored crate lint」明示归 UNKNOWN——这是 deliberate
- architecture.md UNKNOWN 投影第 (b) 类「我们 corpus 引入的 vendored crate lint」——deliberate
- **但**：[env] RUSTFLAGS 注入是 P33 的"修复方式"，不是 UNKNOWN——它把本应 UNKNOWN 的 case 改成正常跑。reviewer 会问：「这是修复 internal validity 还是 corpus instrumentation？」

**反状态排除**：
- R2 部分排除：P33 deliberate 化了 [env] schema + RUSTFLAGS 注入 + 注释充分说明
- R1 适用：reviewer 视角看，corpus 文件携带 instrumentation 命令本身 = corpus 不再"纯净"；纯不纯净不是 deliberate 能消除的

**建议处理**：在 cc-report internal validity 章节明示「`[env] RUSTFLAGS=--cap-lints=warn` 是上游 vendor crate lint 策略与新 toolchain 漂移的桥接，不改 user code semantics；但 reviewer 应理解 corpus 含此 instrumentation」。考虑给 [env] 注入加白名单（principles 或 detailed-design 明示「[env] 只允许 lint-level / 不允许改 feature flag / cfg flag」）。

---

### 候选 5：extra_cargo_deps（creusot 注入 creusot_contracts）→ 工具 → corpus 单向污染【强】

**事实**（tools/creusot/tool.toml）:
```toml
extra_cargo_deps = ['creusot-std = "0.11.0"']
```
runner/src/exec.rs:83-87：把这行 inject 到 **工作副本** Cargo.toml [dependencies]，原始磁盘不动。

**问题**（Wohlin §8.6.2 Instrumentation）：
1. reviewer 角度：「creusot 工具配置可以把任意 dep 加到任意 entry——这是工具单向写 corpus 的 power。如果其他工具也这么做（如 hax 注入 hax-lib / prusti 注入 prusti-contracts）通过率会改变吗？」
2. 当前 20 工具中只有 creusot 用了 extra_cargo_deps（grep tools/*/tool.toml）。verus 用 entry_mode=lib + harness wrap，prusti 用 PRUSTI_* env——三种"工具特定填充"都没有上游能力等价对照
3. principles.md §四 A 形式定义「原始磁盘字面零修改 + 隔离副本上声明式工具特定填充」是 P21 在 architecture 层做的 A 位阶澄清——deliberate

**对照精神**：
- architecture.md §一 明示「extra_cargo_deps 字段 + entry_mode=lib 字段——两机制都是 tool.toml 字段，runner 读字段做事，不破原则 C」——deliberate
- 但 deliberate **的是机制**，不是「机制对工具差异化使用的内卷效应」——reviewer 会问：「为什么 prusti 不需要 prusti-contracts dep 注入而能跑？因为 prusti 的 wrapper 用 PRUSTI_NO_VERIFY=false 跳过 spec 收集——三工具采用三种隐藏机制，等价性不可独立校验」

**反状态排除**：
- R2 部分排除：机制 deliberate 化；effect 对称性未 deliberate
- R1 适用：现象 = 工具间通过率受 extra_cargo_deps 使用与否影响

**建议处理**：cc-report 必须列「工具 × {harness wrap / extra_cargo_deps / env 注入 / wrapper script} 矩阵」+「每格的语义影响范围」，作为 internal validity 章节。

---

### 候选 6：copy_dir_excluding 跳过 Cargo.lock → Prusti 妥协痕迹固化为全局策略【强】

**事实**（runner/src/exec.rs:55-62）:
```
// Exclude `target/` to avoid copying build artifacts. Exclude `Cargo.lock`
// because (a) keeping a stale lock under a tool with an older toolchain
// (e.g. prusti's pinned 2023-08-15 nightly cannot parse v4 lockfiles
// written by recent stable cargo) breaks builds, and (b) for a feature-
// coverage screening run, fresh dep resolution per task is what we want
```

**问题**（Wohlin §8.6.6 Confounding constructs）：
1. 该决定是为 prusti 老 toolchain 妥协（理由 a），但应用于**全 20 工具 + 全 161 entry**——所有工具都享受"每次 fresh dep resolve"
2. reviewer 会问：「dep resolution 跨 run 不稳定 → 同 entry 在 day 1 和 day 30 之间，crates.io 上 transitive dep 升级 → 实际 build 的 dep tree 不同 → 通过率漂移不可归因到工具能力变化」——这是 Wohlin maturation threat 的具体投影
3. 对照 industrial/ 三个 entry 含 `Cargo.lock`（git status 显示），但运行时被 copy 跳过 → corpus 锁定的 reproducibility 被运行时打破

**对照精神**：
- principles.md §五 runner「结果记录严谨全面：(工具版本, 测试环境, ISO 时间戳, 样例特征)」——但**没说"依赖图快照"**也要锚定
- principles.md §六「时空锚定」+「脱离时间 + 工具版本上下文的结论无效」——这条精神承认时间漂移，等价于承认 maturation threat 是 feature 不是 bug
- 但 reviewer 视角：精神承认 ≠ measurement 设计无 internal validity 缺陷

**反状态排除**：
- R2 排除（部分）：跳过 Cargo.lock 是 deliberate（runner 注释明示），时空锚定也是 deliberate（principles §六）
- R1 适用：reviewer 角度看，时空锚定**仅记录时间**不**冻结时间**——即测量结果在不同时间 run 的 day-to-day variance 未被控制，但 internal validity 要求 confound 被控制或被测量

**建议处理**：要么 runner 在 results.json 增「依赖图 hash / Cargo.lock 内容快照」字段（control + measure），要么 cc-report 明示 maturation threat（限制结论时效）。当前精神已承认时效短，可作为 deliberate 接受。

---

### 候选 7：41 features 选择标准不可独立校验 → Selection bias on corpus【弱】

**事实**：examples/ 下 42 个 feature 桶（含 -limit/ 7 个工具特定桶 + industrial/ + 33 个 Rust 语言特性桶）。

**问题**（Wohlin §8.6.1 Selection）：feature 桶的选择 + 桶内 entry 数（部分桶 1 个 entry，部分 8+）的差异——reviewer 会问：「你为什么测 closure-adv 而不测 async-trait？你为什么 hax-limit 8 个而 kani-limit 7 个？」

**反状态排除**：
- R2 排除：principles.md §五「样例覆盖单特性、边界（-limit/）、综合（如 industrial/）多个梯队」明示梯队 deliberate；具体选 41 features 是 deliberate 工程过程（commit history 可追）
- R8 适用：这是流程建议 ≠ blocking——可补「feature 选择标准 + 排除标准」文档到 docs/research/，不是宪法或测量缺陷

**入弱候选**：feature 选择透明度可提升，但不构成 internal validity 阻断。

---

### 候选 8：rayon parallel 与 host concurrency → Maturation / instrumentation threat【弱】

**事实**（runner/src/main.rs:143-176）：
- `--parallel` 默认 = `available_parallelism()`
- 全 (tool, entry) 任务 par_iter 并发

**问题**：并发可能影响：
1. 共享 cargo registry cache 的并发访问（罕见但可能 lock contention）
2. 共享 prusti viper_tools / kani CBMC 的并发资源竞争
3. timeout (120-900s) 在高负载下偏向 timeout，低负载下不偏向

**反状态排除**：
- R2 排除：并发是 deliberate（principles.md §五 runner 精神含「严谨全面」未排除并发；architecture.md 实现选择）
- R8 适用：可补「测量在串行 / 并发下结果差异」复现实验，不是 blocking

**入弱候选**：建议至少为 ISSTA 提交版重跑一次 `--parallel=1` 对照。

---

### 候选 9：-limit/ 桶 entries 分母平等参与 → Corpus 对工具不利【弱】

**事实**：7 个 -limit/ 桶（aeneas/charon/creusot/hax/kani/miri/prusti × 各 7-8 entries），共 52 entries 故意触发对应工具 reject。分母里平等参与计算所有 20 工具的通过率。

**问题**（reviewer 视角）：
- aeneas-limit/ 故意触发 aeneas reject → 对 aeneas 不公平降低通过率？
- 但同时这些 entry 是 aeneas-specific known limitations，**其他工具（kani / verus）跑这些 entry 可能 PASS**——这意味着 -limit/ 桶 **对被针对的工具不利、对其他工具中性或有利**

**反状态排除**：
- R2 排除：principles.md §五 明示「样例覆盖单特性、边界（-limit/）、综合（industrial/）多个梯队」——-limit/ 是 deliberate 设计的 corpus 多样性梯队
- R1 不适用：reviewer 视角的「对工具不利」预设了「通过率应该高」，这违反 principles §二 范围「工具能力客观判定不在项目范围」——通过率是测量值不是评分
- R4 排除：framework 不为 -limit/ 桶任何特殊处理，所有 entry 一视同仁，bias-free

**自身排除候选**：不入清单。-limit/ 桶被 reviewer 误读为 selection bias 的概率确实存在，cc-report 可明示「-limit/ 桶是 known limitations corpus，对应工具通过率天然偏低是 corpus 多样性 feature 不是测量 bug」预防误读。

---

### 候选 10：vendored crate 选择（rsa / sha2 / x509-parser）→ Selection bias【自身排除】

industrial/ 只选 3 个 crate。reviewer 可问选择标准。

**自身排除**：principles.md §五 industrial 是「综合」梯队代表，3 个 crate 足以代表概念，不需要枚举所有。R2 排除。

---

### 候选 11：kani-strict-wrapper 的 5-marker 列表 → Instrumentation 对工具自身不公【自身排除】

kani wrapper 用 5 个 unsupported-construct marker 判定 partial codegen；显式排除 caller_location / foreign function。

**自身排除**：tools/kani/tool.toml 注释充分说明 5-marker 的选择理由（避免在 std-using entry 上 mass false positive）；docs/fixes 文件链入档。R2 deliberate。

---

### 候选 12：runner 工作目录 .tmp/ 共享 target/ → 缓存污染【自身排除】

不成立。检查 exec.rs 已确认每次 task 独立 work_dir，且 target 不被 copy。R4 排除。

---

### 候选 13：harness entry_args 来自 runnable.<entry>.inputs 字面 → 输入选择 bias【自身排除】

15 个 runnable entries 有 inputs/expected。reviewer 可问输入选择。

**自身排除**：feature-coverage screening 测的是「工具能否吃下代码」不是「特定输入下行为」（principles §四 B），输入选择对 acceptance 信号不构成混杂。R2 排除。

---

## 3. 总结

| 候选 | 强度 | Wohlin 类别 | 决策性 |
|---|---|---|---|
| 1 verus harness wrap | 强 | Instrumentation | reviewer 攻击面，cc-report 补 |
| 2 oracle 切点不对称 | 强 | Selection + Instrumentation | reviewer 攻击面，tools README 补 |
| 3 bug-detect 仅 MIRI/soteria | 强 | Confounding + Selection | **决策点**：是否推广 P35 |
| 4 [env] RUSTFLAGS 注入 | 强 | Instrumentation + Maturation | 可考虑白名单 |
| 5 extra_cargo_deps 单向写 | 强 | Instrumentation | cc-report 补矩阵 |
| 6 跳过 Cargo.lock | 强 | Confounding (maturation) | results.json 增 dep hash 字段 |
| 7 41 features 选择透明度 | 弱 | Selection | 流程建议 |
| 8 rayon 并发 | 弱 | Maturation | 补串行复现 |
| 9 -limit/ 对工具不利 | 自排 | — | cc-report 明示防误读 |
| 10/11/12/13 | 自排 | — | — |

**关键观察**：
- runner 形式定义层（A 位阶澄清 / 双轨 schema / TS_* env / extra_cargo_deps 机制）在 architecture 层已 deliberate，无 internal validity 阻断
- 真正 reviewer 攻击面集中在 **测量姿势的工具间对称性论证**——这是 ISSTA 评审会重点质询的"为什么你这种切法 / 注入 / 重写对所有工具公平"
- 候选 3（bug-detect 不对称）是唯一**真决策点**——其他可通过 cc-report 写 threats to validity 化解
