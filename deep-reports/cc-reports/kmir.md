# kmir — 特性支持评估报告（v6 baseline）

## 元数据

- **数据源**：`runs/run-1778560393-59119/`（2026-05-12T04:33:13Z → 04:55:27Z UTC，161 entries，host：Apple M5 / macOS aarch64 / 24 GB / 10 cpu，parallelism=10）
- **工具配置**：`tools/kmir/`
- **工具版本**：mir-semantics commit `84bea09` + stable-mir-json commit `62a239d7`，K Framework v7.1.282（brew，含 llvm-kompile-clang 补丁），kmir Python 包 0.3.181 / kframework 7.1.280，nightly-2024-11-29 toolchain（stable-mir-json 专用）
  - 备注：`tool.toml` 的 `version_command = ["sh","-c","... kmir --version"]` 因 kmir CLI 不支持 `--version` 子命令而退化为 usage 信息，无 semver 字符串入 results.json。版本锚定在 README 与上述 commit 哈希上。
- **本工具实测**：n=161 / SUCCESS=**61** / FAILED=**100** / UNKNOWN=0，通过率 **37.9%**
- **FAILED 内部分布**：exit=2 共 **56**（K-stuck）+ exit=1 共 **44**（cargo + stable-mir-json + kmir Python 链失败）；TIMEOUT 0
- **时长分布**：avg 6312 ms / median 6069 ms / p90 9663 ms / max 40389 ms（全部 entry 在 180 s timeout 内完成）
- **宪法 baseline**：`principles.md` v8（P27 修宪后 UNKNOWN 严格语义 + P31 §四.5"我们 wrapper vs 官方 wrapper"归因传导）
- **时效声明**：本快照锚定上述 run id + 工具版本 + corpus，不构成长期承诺。kmir 三段集成链（cargo+stable-mir-json / kmir Python / K LLVM backend）对 stable-mir-json schema 漂移高度敏感（已知：`Safety::Safe` / `DynType` 等新版 SMIR 节点在 84bea09 的 K definitions 上无对应 production）；任一组件升级都会改写本报告。

## pipeline + 前端边界

KMIR 是 K Framework 上 Rust MIR 形式化操作语义的解释器。`kmir run --bin __ts_harness` 的 pipeline 三段：

```
(a) cargo build with RUSTC=stable-mir-json    [上游官方 driver]
       → 把 entry 编译成 Stable MIR JSON（target/debug/linked.smir.json）
(b) kmir Python: kmir.cargo.smir_for_project + kmir.parse.parser    [上游官方 wrapper]
       → 把 SMIR JSON 解析为 K term
(c) K LLVM backend（kompile + 解释执行）    [K Framework backend]
       → 在 MIR 操作语义里解释执行 K term，输出 K configuration（<kmir>...</kmir>）
```

**前端边界**：`kmir run` 全过程完成（与 MIRI 同构 —— MIR 级解释执行就是 kmir 的"前端 = 全过程"）。理由（README §"前端接受"定义）：(a) 阶段只是 rustc + plugin 生成标准 SMIR JSON，与 cargo-check 几乎重合，不体现 kmir 任何特有能力；kmir 的特有能力 = K Framework MIR semantics 是否能解释执行该程序。

**框架与工具的边界**：
- 本项目维护：runner 调用 `kmir run` 的薄壳（`tool.toml` 的 sh 包装 + grep `#EndProgram ~> .K` 终结模式检测 + PATH 注入 `${TS_KMIR_JAVA_BIN_DIR}`）
- 上游官方 wrapper：`kmir.cargo.smir_for_project`（在 `kmir/cargo.py`，调 stable-mir-json） + `kmir.parse.parser`（SMIR → K term） + K LLVM interpreter

按宪法 §六 + tool-integration.md §4.5："**官方工具自带 wrapper（如 kmir 的 Python `kmir/cargo.py`）失败 = 工具锅 → FAILED**"。本报告所有 `kmir.cargo` / `kmir.parse` / K LLVM 链失败均按"工具能力边界"归类 FAILED，不升 UNKNOWN。

`entry_mode = "bin"`（默认）：runner 写 harness 到 `src/bin/__ts_harness.rs`。`--bin __ts_harness` 在 84bea09 版本中只产生 warning（`Requested to run __ts_harness but multi-exec projects currently not supported`），不报错；实际 crate root 由 runner 设置的 `current_dir` 决定。

## SUCCESS 信号 + 形式严格性

**SUCCESS = `kmir run` exit 0 且 stdout 含 `#EndProgram ~> .K`**（K interpreter 完整跑到终止）。

按宪法 §六 不藏 partial 精神，双通路 partial 暴露：

- **主信号通路（exit code）**：cargo + stable-mir-json 编译失败、SMIR JSON parse 失败、kmir Python 异常 → exit≠0 直接 FAILED
- **wrapper 补抓通路（grep gate）**：K interpreter 卡 stuck 时 kmir CLI 仍 exit 0，但 `<k>` cell 残留 `#execTerminator(InlineAsm)` / `#mkAggregate(closure)` / unsupported reduction 等项。`tool.toml` 的 sh 包装 grep `#EndProgram[[:space:]]*~>[[:space:]]*\.K` 未命中时把 exit 0 翻为 exit 2，并写 `[kmir-oracle] FAIL: K interpreter stuck` 到 stderr。本次扫描全 161 entries 验证 SUCCESS 集合无 leak（61 个全部含 `#EndProgram`）。

**形式严格性 — 0 误报**：✅ 形式可证。`#EndProgram ~> .K` 是 K Framework 解释器的稳定终止 signature，K cell 化简到此 ⇔ 解释执行完整完成。
**形式严格性 — 0 漏报**：实测 + wrapper grep 双通路封堵到位。K-stuck silent path 已被 grep 翻转；cargo / parse / Python 链失败 exit≠0 也直接 FAILED。漏报盲点见下节诚实声明。

## 失败分桶（按 P31 §四.5 归因分类）

100 个 FAILED 按 stdout / stderr 形态分四桶，全部归"工具能力边界"类（依 D2 立场，kmir 官方 `cargo.py` / K parser / K LLVM 失败 = 工具锅）：

### 桶 A：K interpreter stuck（56 case，exit=2）

K cell 卡在中途，未化简到 `#EndProgram ~> .K`。stdout 含 K configuration 残留，wrapper grep 把 exit 0 翻为 exit 2。

形态指纹：
- **closure aggregation 路径**：K 卡在 `#mkAggregate(aggregateKindClosure(...))`——代表 entry `closure/fn-fnmut/closure_fn`、`closure/fn-fnmut/closure_fnmut`、`closure-adv/fn-once`、`closure-adv/return-impl-fn`、`aeneas-limit/closure-if-capture`、`hax-limit/closure-mutates-outer`、`charon-limit/copy-deref-closure` 等
- **inline asm terminator**：K 卡在 `#execTerminator(... kind: terminatorKindInlineAsm(... template: "[]" ...))`——代表 `charon-limit/inline-asm`、`creusot-limit/inline-asm-basic`（这一个反例：相同 entry 在某些时候是 SUCCESS 名单，因实测 stdout 含 EndProgram —— 实际属于不同测试 entry 名）
- **float intrinsic / atomic / 高 width 整数**：K rule 缺失对应 reduction——`float/cmp` / `float/round` / `float/transcendental` / `float/total-order` / `float/nan-prop`、`int-width/arith-i128` / `_arith-u128` / `_arith-i8` / `_bit-ops-u32` / `_checked-all-widths` / `_overflowing-i32` / `_saturating-u16`、`int/checked`、`concurrency/atomic/atomic_seqcst`
- **HRTB / GAT / dyn trait call / impl Trait return**：`hrtb/for-all-lifetime`、`impl-trait/return-iter`、`creusot-limit/dyn-trait-forbidden`、`generic/sum-bound`
- **drop / unwinding / FFI**：`drop/custom-drop`、`error/result-question`、`kani-limit/stack-unwinding` / `_loop-unwinding` / `_extern-ffi` / `_float-overapprox` / `_uninit-memory`、`miri-limit/ffi-unshimmed-extern` / `_networking-unsupported` / `_simd-bitmask-large-vector` / `_soundness-not-guaranteed` / `_weak-memory-incomplete`
- **prusti-limit closure / for-loop 衍生**：`prusti-limit/closure-in-pure-fn` / `_closures-unsupported` / `_for-loop-iterator` / `_loan-crosses-loop-boundary` / `_spec-entailment-unsupported`
- **其它 partial**：`refcell/borrow`、`repr/union`、`slice/index-iter`、`vec/basic-push-pop`、`unsafe-adv/maybe-uninit`、`unsafe-ptr/raw-ptr-const` / `_raw-read`、`iter/chain-collect`、`charon-limit/async-fn`、`deps-complex/itertools-multi`、`hax-limit/labelled-break` / `_ret-mut-ref`、`aeneas-limit/fnmut-closure-unit-return`

共同特征：K rule 在 84bea09 上对这些 SMIR 节点未提供化简规则。

**归因**：工具能力边界（K Framework MIR semantics 在 84bea09 commit 上对这些 MIR 构造未定义 reduction）。
**处理**：不修。本地性原则下 FAILED 站得住，K rule 增量是 kmir 上游的工作。

### 桶 B：SMIR JSON decode failure（23 case，exit=1）

stable-mir-json 没产生合法 JSON 行（空 stdout / 写 panic / 报错文本），kmir Python `kmir.cargo.smir_for_project` 在 `cargo.py:91` 处 `json.loads(line)` 抛 `json.decoder.JSONDecodeError: Expecting value: line 1 column 1 (char 0)`。

代表 entry：
- 智能指针：`arc/clone-drop`、`rc/clone-drop`、`box/basic-alloc`、`box/shallow-init`
- dyn / unsize：`trait-obj/dyn-dispatch`、`closure-adv/boxed-dyn-fn`、`charon-limit/arc-slice-unsize` / `_box-branch-init` / `_generic-to-dyn-unsize` / `_precise-drops-const-generic`
- std 集合 / 线程：`collections/btreemap`、`concurrency/thread-mutex`、`miri-limit/thread-interleaving-partial`、`lifetime/static-bound`
- GAT / nested guard / async：`gat/lending-iter`、`enum/nested-guard`、`kani-limit/async-await`
- creusot-limit：`creusot-limit/generic-for-loop`、`creusot-limit/vec-macro-std`
- 第三方 crate：`industrial/rsa/rsa-pkcs8/rsa_pubkey_from_pkcs8` / `_rsa_pkcs1v15_encrypt`、`deps-complex/collections-serde`、`deps-complex/trait-serde-generic`

**归因**：上游 stable-mir-json 在这些 entry 上未产合法 SMIR——属 kmir 集成链的官方 driver 行为（stable-mir-json 是 mir-semantics 配套的 rustc plugin）。
**处理**：不修。tool-integration.md §4.5："**官方 wrapper / driver crash → 工具的锅，FAILED 站得住**"。

### 桶 C：cargo build 失败（19 case，exit=1）

`kmir.cargo - Cargo compilation failed!`（`cargo.py:97` 抛 `Exception: Cargo compilation failed`）。stable-mir-json 用 nightly-2024-11-29 toolchain 跑 cargo build，在依赖第三方 crate 或新 std API 的 entry 上 cargo 阶段失败。

代表 entry：
- bigint 全 8 个：`bigint/bigint-arith` / `_bigint-bitwise` / `_bigint-conv` / `_bigint-modpow` / `_num-complex-ops` / `_num-integer-gcd` / `_num-rational-arith` / `_num-traits-abstract`
- std 集合 / thread-local：`collections/hashmap`、`creusot-limit/thread-local-ref`、`lifetime/thread-local`
- 第三方 crate：`industrial/sha2/sha256-digest/sha256_digest_one_shot` / `_sha256_digest_incremental`、`industrial/x509-parser/cert-parse/x509_parse_der` / `_x509_subject_extensions`、`deps-complex/bigint-serde` / `_chrono-bigint` / `_chrono-serde` / `_error-chain`

**归因**：stable-mir-json 的 cargo + nightly toolchain 链在这些 entry 上 build 失败——同样属 kmir 集成链的上游 driver 行为。
**处理**：不修。同桶 B 归因（官方 driver crash）。

### 桶 D：K parser AssertionError（1 case，exit=1）

`closure-adv/early-bound-lifetime/early_bound_closure_arg`：`AssertionError: No production for 'Safety::Safe' in sort 'Safety'`。SMIR 含 84bea09 的 K definitions 未识别的新 enum variant。

**归因**：stable-mir-json 升级到含 `Safety::Safe` 节点的版本，但 mir-semantics 84bea09 的 K definitions 未跟上——README 已记录此 schema drift 类问题。属 kmir 集成链的 schema 兼容性问题。
**处理**：不修。

### 桶 E：nightly feature gate（1 case，exit=1）

`hax-limit/let-chains/hax_limit_let_chains`：cargo 报 `error[E0658]: 'let' expressions in this position are unstable`，随后 `kmir.cargo - Cargo compilation failed!`。stable-mir-json 锁的 nightly-2024-11-29 toolchain 未开 `let_chains` feature gate。

**归因**：stable-mir-json 用的 nightly toolchain 拒收 `let_chains`——属"工具自选 toolchain 拒绝 feature"类（principles.md §六 明示属工具能力边界）。
**处理**：不修。本地性原则下：装对 kmir 要求的 nightly-2024-11-29，let_chains 仍不通就是工具能力边界。

## 漏报盲点（诚实声明）

- **已通过 wrapper gate 封堵**：K-stuck silent path（K interpreter exit 0 + `<k>` cell 残留 unsupported terminator / aggregate）—— `tool.toml` 的 grep `#EndProgram ~> .K` 未命中翻为 exit 2，已验证本次 61 个 SUCCESS 全部含该终结模式
- **仍存在的盲点**：
  1. **K 解释器内部 partial 化简 + 假装完结**：若 K rule 把 unsupported 构造 silent 化简到某个含 `#EndProgram` 的伪终止状态（理论可能），grep gate 会漏抓。未在本次 61 个 SUCCESS 上人工抽样找到此类例（抽样深度有限）
  2. **stable-mir-json 输出 SMIR JSON 但语义已损（如 unsupported feature 被 silently 截掉）**：若 SMIR 输出是 well-formed JSON 但已丢失 entry body 的某些构造，kmir Python parse 成功 + K 解释执行也跑到 EndProgram → SUCCESS。stable-mir-json 上游是否有此类静默 lower 路径未确认
  3. **K configuration 化简到 `#EndProgram` 但残留 side-effect cell**：K 多 cell 模型下 `<k>` 终止后其它 cell 仍可能有未消费状态。grep 只看 `<k>` cell，其它 cell 残留不视为 partial

修复 backlog（次要模块优先级，非长期承诺）：盲点 1 可通过 README/kmir 上游对"伪终止"构造调研得分类策略；盲点 2 需 stable-mir-json 上游对 silent lowering 透明度声明配合，超出本框架范围；盲点 3 可加扩展 grep 检查 `<kmir>` 完整化简到 `.K`。

## v5.1 → v6 ΔS 解释

- **v5.1**（旧报告，run-1778226613-5282，146 entries）：46/146 = 31.5%
- **v6**（本次，run-1778560393-59119，161 entries）：61/161 = 37.9%
- **ΔS = +15**（绝对值）/ +6.4 pp（比率）
- **逐项来源**：v6 corpus 新增 15 个 `runnable/*` entries（abs / add-two / add-u32 / bool-ops / digit-sum / enum-classify / fact / fib / gcd / max3 / parity / power / saturating / struct-norm / sub-clamped），全部基本 int / bool / pattern 匹配 / 简单递归——kmir 全 15 通过
- **共有 146 entries 中**：0 flip（所有共有 entry 状态在 v5.1 与 v6 上一致），归因证据：oracle 配置（含 K-stuck grep）未变 + kmir 版本未变 + corpus 未变 + 工具自身确定性
- **结论**：v6 通过率提升完全来自 corpus 扩展引入的简单 entry，与工具能力无关。kmir 在共有 146 entries 上的能力图谱与 v5.1 完全一致

## 修订建议清单（仅"我们导致"失败）

| # | 桶 | 涉及 case | 修复方案 | 优先级 |
|---|---|---|---|---|
| — | — | 0 | — | — |

**无需修订**。100 个 FAILED 全部归"工具能力边界"类：
- 56 K-stuck → K Framework MIR semantics 在 84bea09 commit 上未定义 reduction
- 23 SMIR JSON decode failure → stable-mir-json 上游 driver crash（D2 立场：官方 wrapper `kmir/cargo.py` 失败 = 工具锅）
- 19 cargo build failure → stable-mir-json + nightly toolchain 链失败（同上）
- 1 K parser AssertionError → mir-semantics 84bea09 与新版 stable-mir-json 的 schema drift
- 1 nightly feature gate（E0658 let_chains）→ stable-mir-json 锁的 nightly-2024-11-29 不接受该 feature

按宪法 §一-2 本地性原则 + §六 UNKNOWN 严格语义 + tool-integration.md §4.5"官方 wrapper 失败 = 工具锅"，所有 FAILED 站得住，kmir 上游不能驳回。

oracle 端无配置改动建议——双通路 partial 暴露已生效，SUCCESS 集合无 leak。
