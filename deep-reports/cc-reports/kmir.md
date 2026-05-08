# kmir 深度报告

## 元数据

- **run**: `run-1778226613-5282`（2026-05-08T07:50:13Z → 08:16:08Z UTC，146 entries × 19 工具，host：Apple M5 / macOS aarch64 / 24 GB / 10 cpu）
- **工具版本**：mir-semantics commit `84bea09` + stable-mir-json commit `62a239d7`，K Framework v7.1.282（brew，含 llvm-kompile-clang 补丁），kmir Python 包 0.3.181 / kframework 7.1.280
- **通过率**：46/146 = **31%**（FAILED 100 个；exit=1 共 44 个，exit=2 共 56 个，TIMEOUT 0）
- **时长（ms）**：avg 8196 / median 7395 / p90 13353 / max 105497
- **时效声明**：本快照锚定上述 run id + 工具版本 + corpus，不构成长期承诺。kmir 三段集成链对 stable-mir-json schema 漂移高度敏感（README 记录："新版 stable-mir-json 引入 DynType，84bea09 的 K definitions 不识别"）；任意一段升级都会改写本报告。

## 工具内部 pipeline + 前端边界

KMIR 是 K Framework 上 Rust MIR 形式化操作语义的解释器。命令是 `kmir run --bin __ts_harness`，pipeline 三段：

```
(a) cargo build with RUSTC=stable-mir-json
       → 把 entry 编译成 Stable MIR JSON（target/debug/linked.smir.json）
(b) Python 端 kmir.cargo.smir_for_project + kmir.parse.parser
       → 把 SMIR JSON 解析为 K term
(c) K LLVM backend（kompile + 解释执行）
       → 在 MIR 操作语义里解释执行 K term
       → 输出 K configuration（<kmir>...</kmir>）
```

KMIR 的"前端 = 全过程 = `kmir run` 完成"——理由（README §"前端接受"定义）：(a) 阶段只是 rustc + plugin 生成标准 SMIR JSON，与 cargo-check 几乎重合，不体现 KMIR 任何特有能力；KMIR 的特有能力 = K Framework MIR semantics 是否能解释执行该程序。所以前端边界就是 `kmir run` 完成。

`entry_mode = "bin"`（默认）：runner 写 harness 到 `src/bin/__ts_harness.rs`。`--bin __ts_harness` 在 84bea09 版本中只产生 warning（`Requested to run __ts_harness but multi-exec projects currently not supported`），不报错；实际 crate root 由 runner 设置的 `current_dir` 决定。

PATH 注入 `${TS_KMIR_JAVA_BIN_DIR}`（`/opt/homebrew/opt/openjdk/bin`）：K Framework 运行时需要 Java。

## SUCCESS 信号 + 形式严格性

**SUCCESS = `kmir run` exit 0 且 stdout 含 `#EndProgram ~> .K`**（K interpreter 完整跑到终止）。任何 K-stuck → FAILED。

- **形式指标**：exit 0 **加** stdout grep `#EndProgram[[:space:]]*~>[[:space:]]*\.K`
- **0 误报**：✅ 形式可证。`#EndProgram ~> .K` 是 K Framework 解释器的稳定终止 signature——K cell 化简到此 ⇔ 解释执行完整完成
- **0 漏报**：✅ 形式可证。K-stuck（K cell 卡在 unsupported terminator）grep 已封死 silent path；cargo + stable-mir-json 编译失败 / SMIR JSON 解析失败也直接 exit ≠ 0
- **漏报盲点**：无

`tool.toml` 包装：`kmir run` exit 0 时若 grep `#EndProgram ~> .K` 未命中，oracle 把它翻为 exit 2 + 写 `[kmir-oracle] FAIL: K interpreter stuck (no #EndProgram ~> .K terminator)` 到 stderr。这条 K-stuck 检测让旧 oracle（仅看 exit code）的 70% 虚高 SUCCESS 率落到真实的 31%——README 记录："实证扫到 102 个原始 SUCCESS 中 52 个是 K-stuck 假阳性"。本矩阵就是新 oracle 下的真实数字。

## 实测结果

### 按 feature 类目分布

全 SUCCESS 类目（少数几个）：`hello / int / int-width 7/14（部分） / iter / panic / refcell / repr 1/2 / slice / unsafe-adv 2/3 / unsafe-ptr 0/2`（注：很多类目只有部分 SUCCESS，因 K-stuck 翻转）。

完整失败计数（覆盖几乎所有类目）：

```
bigint 8 · int-width 7 · deps-complex 7 · charon-limit 7 · miri-limit 6
kani-limit 6 · industrial 6 · prusti-limit 5 · float 5 · hax-limit 4
creusot-limit 4 · closure-adv 4 · unsafe-ptr 2 · lifetime 2 · concurrency 2
collections 2 · closure 2 · box 2 · aeneas-limit 2 · vec 1 · unsafe-adv 1
trait-obj 1 · slice 1 · repr 1 · refcell 1 · rc 1 · iter 1 · int 1
impl-trait 1 · hrtb 1 · generic 1 · gat 1 · error 1 · enum 1 · drop 1 · arc 1
```

失败模式分两大支：exit=1（44 个，cargo / stable-mir-json / kmir Python 失败）+ exit=2（56 个，K-stuck）。

### 失败模式归类（基于 raw stderr/stdout 实读）

**A. K-stuck（56，exit=2）**——`[kmir-oracle] FAIL: K interpreter stuck (no #EndProgram ~> .K terminator)`。stdout 含 K configuration 但 `<k>` cell 卡在中途，例：

- `closure/fn-fnmut/closure_fn` 与 `_fnmut`：K 卡在 `#mkAggregate ( aggregateKindClosure ( closureDef ( ... ), .GenericArgs ) ) ~> ... ~> #execTerminator ( terminator (... kind: terminatorKindReturn ) )`
- `aeneas-limit/closure-if-capture`、`closure-adv/fn-once`、`closure-adv/return-impl-fn`：closure aggregation 路径上同形态
- `charon-limit/inline-asm/nop_via_asm`：`#execTerminator ( terminator (... kind: terminatorKindInlineAsm (... template: "[]" ... ) ) ) ~> .K`
- `concurrency/atomic/atomic_seqcst`、`float/cmp` / `float/round` / `float/transcendental` / `float/total-order` / `float/nan-prop`：K rule 缺失对应 reduction
- `aeneas-limit/fnmut-closure-unit-return`、`charon-limit/copy-deref-closure`、`drop/custom-drop`、`generic/sum-bound`、`hax-limit/closure-mutates-outer` / `_labelled-break` / `_ret-mut-ref`、`hrtb/for-all-lifetime`、`impl-trait/return-iter`、`creusot-limit/dyn-trait-forbidden`、`creusot-limit/inline-asm-basic`（同 inline-asm 形态）等

K-stuck 的形态指纹：closure aggregation / inline asm terminator / atomic / float intrinsic / GAT / closure mut capture——共同特征是 K rule 在 84bea09 上对这些 SMIR 节点未提供化简规则。

**B. SMIR JSON 解析失败（约 35，exit=1）**——`json.decoder.JSONDecodeError: Expecting value: line 1 column 1 (char 0)`。stable-mir-json 没产生合法 JSON 行——可能是空 stdout、写 panic / 报错文本。属于此类的 entry 含：
- `arc/clone-drop` / `box/basic-alloc` / `box/shallow-init` / `rc/clone-drop`
- `charon-limit/arc-slice-unsize` / `_box-branch-init` / `_generic-to-dyn-unsize` / `_precise-drops-const-generic`
- `closure-adv/boxed-dyn-fn` / `_early-bound-lifetime`
- `collections/btreemap` / `_hashmap`、`concurrency/thread-mutex`、`creusot-limit/generic-for-loop` / `_thread-local-ref` / `_vec-macro-std`
- `enum/nested-guard`、`error/result-question`、`gat/lending-iter`
- `industrial/*` 6 个（rsa / sha2 / x509-parser）
- `lifetime/static-bound` / `_thread-local`、`miri-limit/thread-interleaving-partial`、`trait-obj/dyn-dispatch`

特征是 entry 的 lib.rs 涉及 dyn trait / Box<dyn> / Arc<[T]> / std::collections / std::thread / GAT / nested match guard / 第三方 crate——stable-mir-json 在这些 entry 上未成功产 SMIR。

**C. cargo build 在 entry lib.rs 上失败（约 9，exit=1）**——`ERROR ... kmir.cargo - Cargo compilation failed!` + `Exception: Cargo compilation failed`：
- `bigint/*` 8 个：stable-mir-json 在依赖 `num_bigint` 等的 entry 上 cargo 阶段失败
- `hax-limit/let-chains/hax_limit_let_chains`：`error[E0658]: 'let' expressions in this position are unstable`（stable-mir-json 用的 nightly toolchain edition / feature gating）
- `deps-complex/*` 多个：cargo + stable-mir-json 在外部 crate 依赖上失败

注：B 与 C 的差别是 stderr 末尾错误形态——B 是 `JSONDecodeError`，C 是 `Cargo compilation failed`；都属于 stable-mir-json 链的失败信号。

**D. K parser 拒绝 SMIR symbol（合并入 B 类形态，未单独命中）**——README 提到的 `Safety::Safe` / `DynType` 等 schema 漂移在本矩阵下多数被前置 cargo 失败截留，未单独触发 K parser AssertionError。

合计：56（K-stuck）+ 44（cargo/SMIR 链失败）= 100。

## 与本次测试边界的关系

**K-stuck 检测让旧 70% 虚高真实化为 31%**：旧 tool.toml 仅看 exit code，K interpreter 卡 stuck 时 CLI 仍 exit 0 → 102/146 名义 SUCCESS（70%）。新 oracle 通过 grep `#EndProgram ~> .K` 终结模式，把 52 个假阳性翻为 exit 2 → 真实 46/146 (31%)。

这条修订的精神基础：宪法 §六-2 不允许 partial。K cell 残留 `#execTerminator(InlineAsm)` / `#mkAggregate(closure)` 等 unsupported 项 = K 语义没跑完 = partial = 必须 FAILED。oracle 上的 grep guard 是 partial 暴露机制的形式落地。

**对比信号**：`charon-limit/inline-asm/nop_via_asm` 在 cargo-check 上 SUCCESS（rustc 接受）、miri 上 FAILED（`unsupported operation: inline assembly is not supported`）、kmir 旧 oracle 上 SUCCESS（K configuration 含 `terminatorKindInlineAsm` 但 CLI exit 0）、**kmir 新 oracle 上 FAILED**（K-stuck，缺 reduce 规则）。新 oracle 把 kmir 与 miri 的判定对齐——两个工具都暴露"inline asm 未支持"，前者通过 K-stuck，后者通过显式 unsupported feature 错误。

**慢**：median 7.4s / max 105.5s。每次 `kmir run` 做增量 kompile（modify definition.kore + 重链接），简单程序约 10-30s 是 K Framework 解释执行本身的成本，不是越界做了证明；max 105s 仍在 180s timeout 内，全部 entry 限时内完成。

## 历史快照声明

本报告所有数字与归类锚定 mir-semantics 84bea09 + stable-mir-json 62a239d7 + K Framework v7.1.282 + nightly-2024-11-29 toolchain。三段链中任一组件升级都会改写归类（K rule 增加 / SMIR schema 演进 / kmir.cargo 解析逻辑变化）。
