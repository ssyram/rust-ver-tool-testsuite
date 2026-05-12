# 特性覆盖度报告 — 19 工具 × 146 entries（修复 harness 后）

**run 标识**：`run-1778155001-26161`
**起止**：2026-05-07T11:56:41Z – 12:19:29Z (UTC)，22m 48s wall
**Host**：host / macos / aarch64 / Apple M5 / 24576 MB / 10 cores
**并发**：10
**原始数据**：`runs/run-1778155001-26161/`（results.json + 每 task raw stderr/stdout）

旧两份报告保留：
- `feature-coverage-2026-05-07.md`（早期 8 工具版）
- `feature-coverage-2026-05-07-19tools.md`（19 工具版，但**测试系统本身 harness 与配置存在多处缺陷未修**——见本报告 §一"修复了什么"）

本报告基于的运行用了**修复后**的 testsuite 系统，是目前最接近"工具自身真实接受面"的实测。

## 一、修复了什么

通过派 sub-agent 扫全 19 份 cc-reports 的 FAILED 根因，识别出 6 类被误归到工具失败、但实际是**测试系统自身缺陷**的问题。本次 run 在跑前修复了以下 4 类（剩 2 类是工具的 input mode 真实边界——保留为工具事实陈述）：

1. **harness `{{ target_crate_name }}` 未做 hyphen→underscore 标准化**（runner exec.rs）。所有 `*-limit` 类目下含 hyphen 的 crate 名（如 `bool-bitwise-op`）拼到 Rust path 位置时被 rustc 当减法解析。修复后 cargo-check / kani / miri / prusti / kmir 等读 harness 的工具的 SUCCESS 率显著回升（cargo-check 从 67% → 100%，kani 67% → 98%，miri 67% → 97%，prusti 67% → 93%，kmir 45% → 69%）。
2. **`.env` 变量被工具误读为 config flag**（runner exec.rs spawn child）。Prusti 把所有 `PRUSTI_*` envvar 解读为它自己的 `PRUSTI_<flag>=value` 配置，runner 引入的 `PRUSTI_RUSTC=...` 让 prusti 启动 panic。修复：所有 `.env` 变量统一加 `TS_` 前缀；runner spawn child 前 `env_remove` 所有 `TS_*` envvar，让它们仅对 runner 可见。同时 `${TS_*}` 在 runner 内部用于 tool.toml 的 `${VAR}` 展开。
3. **lib-mode harness 调 `__ts_inner::<entry_fn>()` 不传参 vs 带参 entry_fn**。11 个 limit entry 的 `pub fn` 带参（如 `add_via_asm(a: u64, b: u64)`），harness 调用形式 `__ts_inner::add_via_asm()` 触发 E0061。修复：在每个违约 entry 的 lib.rs 末尾加零参 wrapper（`trigger_<原 fn 名>`），testsuite.toml 改用 wrapper 名。原带参 fn 保留——它仍是被测的 Rust 特性载体，只不过通过 wrapper 触发。
4. **`runner discover` 限制 `walkdir::max_depth(2)`**。`industrial/sha2/sha256-digest/` 三级路径被跳过，industrial 6 个 entry 在前两次报告里 0 task。修复：去掉深度限制 + 加 `.no-testsuite` 屏蔽 marker（一目录内有此文件则跳过自身 + 整棵子树）；feature 仍是一级目录，dir 用相对路径 `/` 拼接（如 `industrial/sha2/sha256-digest`）。本次 run industrial 6 entry × 19 工具 = 114 task 首次进矩阵。

未修复（属工具 input mode 真实边界，本测试如实记录）：

5. **多个工具单文件直调不读 Cargo.toml**（rocq-of-rust / verus / verifast / soteria）。dep-heavy entry（`bigint/*` + `deps-complex/*` + `industrial/*`）在 import resolution 阶段失败。这是工具 input 模式特性，按"能力靠观测"原则不预跳过。
6. **多个工具命令行未透传 entry manifest 的 `edition`**。`hax-limit/let-chains` entry 的 `Cargo.toml` 写 `edition = "2024"`，工具命令行 hard-code 旧 edition 时被拒。同样作为工具集成层的实测事实记录。

## 二、整体数字

| 状态 | 数 | 占比 |
|---|---:|---:|
| SUCCESS | 2107 | **75%** |
| FAILED  | 667  | 24% |
| UNKNOWN | 0    | 0% |
| TIMEOUT | 0    | 0% |

19 工具 × 146 entries = 2774 task。0 个 task 命中 timeout（说明 timeout 配置足够），0 runner-internal 故障。

**对比上次（前 harness fix 的 19tools 报告 run-1778148197-53283）**：

| run | tasks | SUCCESS | rate | 主要差异 |
|---|---:|---:|---:|---|
| 旧（harness bug 未修） | 2660 | 1826 | 68% | 缺 industrial；`*-limit` 类目大量 hyphen 误判 |
| 新（fix 后） | 2774 | 2107 | **75%** | +114 industrial task，+281 SUCCESS（多数来自 hyphen / TS_ / wrapper 修复 + kmir PATH 修复） |

## 三、工具维度

按 SUCCESS rate 排序。时间字段仅作环境上下文，不作工具评分。

| tool | n | S | F | rate | avg(ms) | p50 | p90 | max |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| **cargo-check** | 146 | 146 | 0 | **100%** | 2619 | 227 | 5522 | 52912 |
| **kani**        | 146 | 144 | 2 | **98%**  | 4165 | 857 | 13047 | 53245 |
| **miri**        | 146 | 142 | 4 | **97%**  | 3473 | 1015 | 10351 | 33428 |
| charon-poly     | 146 | 139 | 7 | 95%  | 3333 | 387 | 13617 | 39439 |
| charon-mono     | 146 | 138 | 8 | 94%  | 2285 | 375 | 8588 | 26841 |
| **prusti**      | 146 | 137 | 9 | **93%** | 3829 | 979 | 10943 | 49811 |
| hax-lean        | 146 | 130 | 16 | 89% | 3877 | 1727 | 8864 | 40009 |
| rocq-of-rust    | 146 | 122 | 24 | 83% | 99 | 87 | 125 | 492 |
| verifast        | 146 | 116 | 30 | 79% | 211 | 200 | 284 | 801 |
| hax-fstar       | 146 | 115 | 31 | 78% | 3834 | 1395 | 8284 | 43337 |
| soteria         | 146 | 109 | 37 | 74% | 1883 | 1050 | 4266 | 12644 |
| creusot         | 146 | 106 | 40 | 72% | 35358 | 36584 | 42579 | 49267 |
| kmir            | 146 | 102 | 44 | 69% | 7213 | 6520 | 10506 | 67798 |
| hax-coq         | 146 | 98 | 48 | 67% | 3765 | 1698 | 10338 | 28351 |
| aeneas-lean     | 146 | 87 | 59 | 59% | 4370 | 1853 | 9060 | 50447 |
| aeneas-fstar    | 146 | 87 | 59 | 59% | 4044 | 1759 | 9045 | 33923 |
| aeneas-coq      | 146 | 87 | 59 | 59% | 4007 | 1564 | 10136 | 42440 |
| verus           | 146 | 51 | 95 | 34% | 482  | 498 | 717 | 1084 |
| aeneas-hol4     | 146 | 51 | 95 | 34% | 4010 | 1708 | 9283 | 26907 |

加粗的四个工具是 fix 后变化最显著的——hyphen normalize + TS_ envvar strip 修复让它们的真实接受面浮现。

aeneas 4 backend 中 hol4 显著低于其他三 (34% vs 59%)，且 fstar/coq/lean 三者 S 数完全相同（87/146）——揭示 aeneas pipeline 中 charon 阶段 + aeneas mid-end 阶段的失败是 4 个 backend 共享，hol4 额外有 backend pretty-print 阶段的独立失败。

## 四、特性维度

```
                         tasks   S   F  rate
hello                       19  19   0  100%
int                         38  38   0  100%
vec                         19  19   0  100%
panic                       38  37   1   97%
int-width                  266 254  12   95%
generic                     76  72   4   94%
drop / const                19  18   1   94%
arc / impl-trait / rc / slice 19 17   2  ~89%
prusti-limit               152 133  19   87%
enum / box                  38  32   6   84%
refcell                     19  16   3   84%
miri-limit                 133 108  25   81%
hax-limit                  152 120  32   78%
unsafe-adv / closure        ...                ~78%
aeneas-limit               152 120  32   78%
creusot-limit              133 101  32   75%
closure-adv / repr / kani-limit / iter / hrtb  ~73%
concurrency / collections / float ~70%
trait-obj / error           38  ~26 ~12  ~68%
bigint                     152 100  52   65%
charon-limit               133  86  47   64%
lifetime                    57  35  22   61%
gat                         19  11   8   57%
unsafe-ptr                  38  20  18   52%
deps-complex               133  65  68   48%
trait                       19   9  10   47%
industrial                 114  48  66   42%   ← 首次跑
```

`hello` / `int` / `vec` 三个特性 100% SUCCESS——所有 19 工具都接受；matrix 上没有任何 entry 在 19 工具上 0 SUCCESS。

`*-limit` 类目（工具自声明限制集）现在的接受率 73-87%——fix 前是 42-61%，差距全部来自 testsuite-side bug，**不是工具能力**。

## 五、industrial — 首次跑工业代码

6 个 entry × 19 工具 = 114 task / 48 SUCCESS（42%）。每个 entry 都是 8/19 SUCCESS——非常齐整。

| entry | SUCCESS 工具 |
|---|---|
| `rsa/rsa-pkcs8/rsa_pkcs1v15_encrypt` | cargo-check, charon-mono, charon-poly, hax-coq, hax-fstar, hax-lean, kani, miri |
| `rsa/rsa-pkcs8/rsa_pubkey_from_pkcs8` | 同上 |
| `sha2/sha256-digest/sha256_digest_incremental` | 同上 |
| `sha2/sha256-digest/sha256_digest_one_shot` | 同上 |
| `x509-parser/cert-parse/x509_parse_der` | aeneas-coq, aeneas-fstar, aeneas-lean, **cargo-check, charon-mono, charon-poly, creusot, miri** |
| `x509-parser/cert-parse/x509_subject_extensions` | 同上 |

**rsa / sha2 vs x509-parser** 在 8 个 SUCCESS 工具上有显著分化：

- rsa / sha2：cargo-check / charon × 2 / hax × 3 / kani / miri 通过；aeneas 系列全 fail（不通过 charon→aeneas 流水线的 LLBC mid-end 阶段）；creusot 全 fail。
- x509-parser：cargo-check / charon × 2 / **aeneas-coq/fstar/lean** / creusot / miri 通过；hax 全 fail；kani 全 fail。

x509-parser 用 nom parser-combinator 重 lifetime—对 hax 接受面破。rsa / sha2 用 trait-heavy generic-bound 设计—对 aeneas 流水线某阶段破，但被 hax 接受。这两个工业项目落在不同工具的接受面交集，是矩阵观察到的工业级代码"工具特性维度差异"。

aeneas-hol4 / verifast / verus / kmir / prusti / soteria / rocq-of-rust 在所有 6 个 industrial entry 上**全 FAILED**——它们要么单文件不读 Cargo.toml（verifast / verus / soteria / rocq-of-rust），要么是 hol4 backend 在 industrial 复杂依赖链上的边界 / kmir stable-mir-json 在重 dep 项目上的 JSON 解析问题 / prusti 锁的 nightly-2023-08-15 cargo 不识别 industrial 依赖的某些较新 manifest 字段。

## 六、跨工具特性边界（53 个 limit entry 展开）

每个 limit entry 测一个具体 Rust 特性。下面按主题分组列出，每个给"工具自声明限制内容 + 跨工具实测分布"。entry 命名沿用 `<source-tool-limit>/<dir>/<entry_fn>`。

详细分组与主题文字与上一份 19tools 报告（`feature-coverage-2026-05-07-19tools.md` §五）一致——本次 fix 后的数据让这些主题的工具支持率分布更准确（特别是被 hyphen / TS_ envvar 误判过的工具）。新数据下的关键变化：

- **async fn / coroutine**（`charon-limit/async-fn` + `kani-limit/async-await`）：cargo-check + miri 现在两 entry 都 SUCCESS（之前 hyphen 误判被算 FAILED），新分布揭示 12 个工具拒收 async（aeneas × 4 / charon × 2 / creusot / hax-coq / hax-fstar / rocq-of-rust / soteria / verifast）。
- **Inline ASM**（4 entry × 19 = 76 task）：fix 前看上去 cargo-check 拒收 asm，实际是 hyphen-name 拒。fix 后 cargo-check 4/4 全 SUCCESS——Rust 标准编译器接受 asm。真实的拒 asm 工具是 charon × 2 / creusot / hax-coq / hax-fstar / kani（only-codegen 阶段拒）/ kmir（K 没有汇编模型）/ miri / soteria / verus。
- **`&mut` 复杂用法**（hax 5 entries + prusti 3 + aeneas 2）：hax-coq / hax-fstar 在多 entry FAILED 与 hax 自声明限制吻合；hax-lean 普遍接受这些（hax-lean 整体 89% 是 hax 系列最高）。
- **闭包高阶**：cargo-check 现在 5/5 全 SUCCESS（hyphen 修复）；prusti 在 closures-unsupported 自陈一致 FAILED 但其他工具普遍接受。
- **trait object / dyn**：creusot-limit/dyn-trait-forbidden 在 5/19 工具 SUCCESS——creusot 自身在该 entry **FAILED**（与它自声明的 dyn 限制一致）。
- **panic 策略 / unwinding**：kani-limit/stack-unwinding 上 kani 自身 FAILED（与 README 一致）；charon 双 mode + hax-fstar + rocq-of-rust + verifast 接受。

完整 entry × tool 分布矩阵见 `runs/run-1778155001-26161/report.md` 第三段（按 feature 分块的 per-entry × per-tool 表）。

## 七、运行时观察 — testsuite 系统自身行为

**runner discover** 现在按 testsuite.toml 找 entry，不限深度。`.no-testsuite` 标记文件让一个目录及其整棵子树跳过——给 vendor submodule / scratch 目录用。industrial entries 这次首次进矩阵。

**`.env` 系统**：`.env.example` 含每工具的上游 GitHub URL + 测试用 commit/版本号 + 对应 `TS_*` envvar 名；用户复制 `.env.example` → `.env` 后填本机路径，跑 runner 前 `source .env`。runner 在读 tool.toml 时对 command / version_command 数组每条字符串做 `${TS_*}` 展开（保留 array argv，不走 sh 拼接）；spawn child 前 strip 所有 `TS_*` envvar 防污染。`TS_PROJECT_ROOT` 由 runner 自动注入 = invocation cwd。

**报告自描述**：`results.json` 顶部 metadata 段含 host info / timestamps (ISO 8601 UTC + Unix secs) / 每工具版本字符串 + 完整 argv + entry_mode + extra_cargo_deps。这次 run 的 metadata 显示 cargo 1.95.0 / charon 0.1.184 / aeneas a14083a6 / creusot 0.11.0 / hax 30949eb8 / kani 0.67.0 / miri cb40c25f / verifast 26.01 / verus 0.2026.05.03.8b81855 / rocq-of-rust 0.1.0。

## 八、整体核心观察

1. **harness fix 让真实的工具接受面浮现**：cargo-check 100% / kani 98% / miri 97% / prusti 93%——这四个工具在本测试矩阵上的"接受 Rust 子集"边界比之前报告显示的宽得多。修复是一行 `replace('-', "_")` + 一行 `env_remove("TS_*")` —— 但暴露的影响达 281 task。
2. **9 个工具 ≥ 89% 接受率**：cargo-check / kani / miri / charon × 2 / prusti / hax-lean / rocq-of-rust / verifast——这些工具对本矩阵覆盖的 Rust 特性绝大多数都"在册"。
3. **3 个工具 ≤ 35%**：verus（34% / 接受 Rust 子集严格小、`verus! {}` 块外构造大量被拒）、aeneas-hol4（34% / HOL4 backend pretty-print 在 generics / closures 多构造上独立 FAILED）。kmir（69%）介于中间，主要被 stable-mir-json schema 漂移 + dep-heavy 项目 JSON 解析阻挡。
4. **`*-limit` 集 73-87% 接受率**：揭示工具自声明的限制只有部分能在不同工具上被一致拒收；多数 limit entry 在多数工具上 SUCCESS，只在声明该限制的工具上 FAILED。
5. **`industrial` 接受率 42% 是矩阵最低之一**：6 个工业项目 × 19 工具 = 114 task / 48 SUCCESS。揭示 dep-heavy + 工业级代码风格在多工具上是真实边界——8 个工具能接受其中部分组合，11 个工具全部拒收。
6. **0 timeout / 0 unknown**：当前各工具 timeout 配置足够（runtime config `tool.toml` 内 timeout_secs 范围 120-900）；没有 runner-internal 故障。

数据来源 `runs/run-1778155001-26161/results.json`；每 entry 的 doc comment 在 `examples/<feature>/<dir>/src/lib.rs`；每工具的 raw stderr 在 `runs/run-1778155001-26161/raw/<tool>/<entry_id>.stderr`。每工具的细化分析报告在 `deep-reports/cc-reports/<tool>.md`（注：这些 cc-reports 写于 fix 前的 run，FAILED 根因表里的 hyphen / wrapper / TS_ envvar 类问题在新 run 已不再出现，但工具自身的真实边界部分仍然适用）。
