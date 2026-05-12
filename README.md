# rust-ver-tool-testsuite

Rust 验证工具的**特性覆盖广度**筛选测绘框架。

- 一个例子 = 一个独立 cargo lib，含零参 `pub fn` 入口。
- 一个工具 = `tools/<name>/{tool.toml + harness.rs.tera}`（必要时含 wrapper.sh）。
- 矩阵 = 工具 × 入口。runner 不判 pass/fail——只记 exit code（SUCCESS / FAILED / UNKNOWN）+ 原始 stdout/stderr，由人解读。

**License**：Dual-licensed under Apache-2.0 OR MIT，详 [`LICENSE`](LICENSE)。
Vendored crates (`vendor/`) 保留各自上游 license。

---

## 初次 clone

```sh
git clone <repo>
cd rust-ver-tool-testsuite
git submodule update --init --recursive   # 必须！industrial/ 部分 entries 依赖 vendor/rsa /sha2 /x509-parser
cp .env.example .env                       # 按本机实际编辑路径
```

---

## 文档结构（必读，所有工作以此为标准）

**一切以 [`docs/design/`](docs/design/) 为中心**——任何更新都**优先维护文档**，再做代码 / 配置 / 实测的修改。文档与下游冲突时改下游不改文档（除非显式讨论修订上游）。

```
docs/design/
├── principles.md      ← 【宪法】绝对精神，未经显式讨论不可篡改；所有下游 100% 满足
├── architecture.md    ← 在宪法约束下的核心架构设计；index 入口
└── detailed-design.md ← 函数级细化、schema、配置示例
docs/publish/
├── publish-readiness.md  ← 学术发表 audit 方法学 + 当前 readiness checklist + re-audit triggers (质量保证 L4 层)
├── paper.md              ← ISSTA-style paper draft (中文)
├── glossary.md           ← 项目自创术语 → 学术 register 对照
└── tool-citations.md     ← 20 工具上游论文引用
```

**质量保证四层** (详 `docs/publish/publish-readiness.md` §7)：

- L1 宪法 / 架构 / 实施 (`docs/design/` + `runner/` + `tools/`)
- L2 per-tool 实测 audit (`deep-reports/cc-reports/`)
- L3 法律层 audit (`docs/audit/v6-law-*`)
- L4 publish-readiness audit (`docs/publish/publish-readiness.md`)

每层按宪法 §八 c+cc disprove-first 协议跑。

**`principles.md` 是项目宪法**——根本问题意识、项目目标、模块定位与优先级、三大派生原则、宪法级硬指标、外围原则。任何讨论与争议以此文为准。

**`architecture.md` 在不违反宪法的前提下为核心架构**——若发现架构与宪法冲突，改架构不改宪法（除非显式讨论修订宪法）。

下文（"怎么跑 / 加新 example / 加新工具"等）是用户视角的入口指引，是宪法的应用层，**不替代宪法**。

---

## 怎么跑

### 全矩阵

```sh
cargo run --release --manifest-path runner/Cargo.toml
```

不带参数即跑 `examples/` × `tools/` 全集，所有 `(tool, example, entry)` 三元组并发执行。

### 子集筛选

```sh
# 只跑某些工具
runner --tool kani --tool charon-poly

# 只跑 entry-id 匹配某 glob
runner --entry 'closure-adv/**' --entry 'enum/**'

# 组合（增量调试某工具在某 entry 的行为）
runner --tool prusti --entry '**/btreemap_basic'
```

### 重生成 report.md（不重跑工具）

```sh
runner report <runs/run-id>/
```

读已有 `results.json` 重生成 `report.md`。Schema 演化时无需重跑工具。

### CLI 选项

| flag | 默认 | 含义 |
|---|---|---|
| `--examples <dir>` | `examples` | 例子根目录 |
| `--tools <dir>` | `tools` | 工具配置根目录 |
| `--runs <dir>` | `runs` | 输出根目录 |
| `--parallel <N>` | CPU 核数 | 并发上限 |
| `--tool <NAME>` | (空 = 全部) | 只跑指定工具，可重复 |
| `--entry <GLOB>` | (空 = 全部) | 只跑匹配 ID 的 entry，可重复 |

---

## 看结果

每次运行生成 `runs/run-<unix-ts>-<pid>/`：

```
runs/run-1778060885-78869/
├── report.md            # 头部 metadata + 按 feature 分组的工具 × 入口 矩阵
├── results.json         # 机器可读版（含 host info / 工具版本 / 时间戳 + 每 task 详情）
└── raw/<tool>/
    ├── <feature>__<dir>__<entry>.stdout
    ├── <feature>__<dir>__<entry>.stderr
    └── <feature>__<dir>__<entry>.exit
```

排查某 FAILED 单元：`cat raw/<tool>/<feature>__<dir>__<entry>.stderr`。

`results.json` 顶部含 host 戳（hostname / cpu_brand / mem / num_cpus）+ ISO 8601 起止时间 + 每工具 `version_command` 输出——按"工具非静态原则"裸数据自描述，跨时段对比时版本上下文不丢失。

### 结果分类

- **SUCCESS** — 子进程跑完 + exit 0
- **FAILED** — 子进程跑完 + exit ≠ 0、被信号杀、或超时 SIGKILL
- **UNKNOWN** — runner 自身故障（cp / 渲染 / spawn 失败等），**不归工具责任**

---

## 加一个新 example

最小三件套：`examples/<feature>/<dir>/{Cargo.toml, src/lib.rs, hirusttest.toml}`。

```toml
# examples/myfeat/basic/Cargo.toml
[package]
name = "myfeat_basic"
version = "0.1.0"
edition = "2021"

[lib]
path = "src/lib.rs"
```

```rust
// examples/myfeat/basic/src/lib.rs
pub fn myfeat_basic_entry() {
    let _ = 1 + 1;   // plain Rust, zero verifier-specific markers.
}
```

```toml
# examples/myfeat/basic/hirusttest.toml
entries = ["myfeat_basic_entry"]
```

约束：

- 入口 = **零参 `pub fn`**（任意返回类型）
- 一 entry = 一独立 lib crate（**禁止跨 entry 共享 helper**——共享出问题会让多个 entry 同 fail，丢失工具间分化信号）
- 例子源码 **plain Rust**，零 verifier 标记（`#[kani::proof]` / `#[ensures]` / `verus! {...}` 等都不允许）
- `hirusttest.toml` 可选 `target_path = "<rel>"`：多 crate 样例时指向被测 crate
- **形式定义**：加 `hirusttest.toml` 前后 example 自身行为必须**字节级 100% 一致**（任意 `cargo build/check/run/test` 输出不变）—— `hirusttest.toml` 是 cargo / rustc 不读的项目自有 schema，所以加它不影响 example 行为。**禁止**通过 `[package.metadata.hirusttest]` 等内嵌进 Cargo.toml 的方案（cargo 会读，违反形式定义）。详见 [`docs/design/principles.md`](docs/design/principles.md) §四 A 的形式定义。

### 加一个外部综合项目（如 vendor/x509-parser、vendor/openssl）

外部综合项目（非本项目原创、可能是 git submodule 或 vendor crate 的真实 codebase）必须用**目录轨**：`<example_dir>/.hirusttest/config.toml`。`config.toml` 字段与单文件轨的 `hirusttest.toml` 同（`entries` + `target_path`），并预留扩展字段（per-entry stub 注入、per-tool 配置覆盖等，渐进实施）。

```
vendor/x509-parser/                  ← 原项目（git submodule，不动）
├── Cargo.toml                       （原项目自带）
├── src/                             （原项目自带）
└── .hirusttest/
    ├── config.toml                  ← 必有，entries + target_path 等
    └── <可选辅助文件>               ← per-entry verifier stub 等（渐进实施）
```

**强制边界**：
- 简单单特性测试（`examples/<feature>/<dir>/`）**不允许**升级到目录轨
- 外部综合项目**必须**用目录轨（即便目前只有一个 entry，预留扩展空间）
- 同一目录禁止 `hirusttest.toml` 与 `.hirusttest/` 同时存在（runner discover 报错）
- `.hirusttest/` 整目录加不加，对原项目的 `cargo build/check/run/test` 字节级一致——满足形式定义

详见 [`docs/design/principles.md`](docs/design/principles.md) §四 A 双轨 schema。

加完直接重跑 runner 即被自动发现。

---

## 加一个新工具

### 硬指标（必须满足）

加入新工具**必须满足核心精神**——配置 `tool.toml` 让工具完整跑完它**自身设计的前端**（直到自带后端验证器 / 求解器之前），且不允许任何 partial / silent skip。

具体硬指标：

1. **明确该工具的前端 / 后端边界**：把工具的内部 pipeline 阶段图画出，标出"工具自带后端"开始的位置，`tool.toml` 必须停在该位置之前。前端边界由工具内部设计决定，**没有统一标准**——翻译类、编码 + SMT 类、解释执行类、符号执行类、编译基线各自不同。

2. **`tool.toml` 命令必须严格停在前端边界**：不进入工具自带的下游求解器/分析器；同时让前端**完整跑完**（不允许工具自陈"我没全干完"的 partial 路径被 oracle 接受为 SUCCESS）。

3. **SUCCESS 信号必须严格**：明确什么 exit code + 什么产物条件 = "前端完整接受 + 无 silent partial"。如果工具有 silent path（如 hax-lean 的 `text!("sorry")` 不发 Diagnostic），oracle 必须用产物 grep / stderr 检测补抓——目标是 0 误报、0 漏报。

4. **README 必须写明**：
   - 工具内部 pipeline 阶段图（哪几段，工具自带后端在哪一段）
   - 前端 / 后端边界落在哪个阶段
   - `tool.toml` 用什么 flag 实现该切割
   - SUCCESS 信号 + partial 暴露机制 + 已知盲点（按 `tools/cargo-check/README.md` 的"### SUCCESS 信号（严格反映前端特性支持范围）"模板）

**反作弊推论**：dry-run flag 必须让工具**真的拿样例代码喂自己的前端**——不能绕路给 stock rustc 走 cargo-check 等价路径（典型反例：Verus 的 `mod __ts_inner` 必须放进 `verus! {}` 块内）。

**Tools 端绝不为框架适配**（principles.md §四 A 5 层精细化层级 4）：`tools/<name>/` 目录下的所有文件（`tool.toml`、`harness.rs.tera`、wrapper.sh）是**集成者描述工具自身行为**的桥接信号——**不是工具自身的修改**。tool 本身（cargo-kani / cargo-prusti / cargo-creusot 等）作为黑盒，绝不为框架做任何代码或行为上的适配。

### 最小文件结构

最小两件套：`tools/<name>/{tool.toml, harness.rs.tera}`。

```toml
# tools/mytool/tool.toml
command          = ["cargo", "mytool", "--bin", "__ts_harness"]
timeout_secs     = 300                                 # 可选，默认 300
extra_cargo_deps = ['mytool-helper = "1.0"']           # 可选，默认 []
entry_mode       = "lib"                               # 可选，默认 "bin"
version_command  = ["cargo", "mytool", "--version"]    # 可选；若有则 metadata 段含输出
```

```rust
// tools/mytool/harness.rs.tera
fn main() {
    {{ target_crate_name }}::{{ entry_fn }}();
}
```

模板变量恰好两个：`{{ target_crate_name }}` + `{{ entry_fn }}`。

退出码 0 → SUCCESS，非 0 → FAILED。

字段 / harness 写法约定详见 [`docs/design/detailed-design.md`](docs/design/detailed-design.md)。

---

## 精神宪法（必读）

> 全部精神原则归一在 [`docs/design/principles.md`](docs/design/principles.md)——**所有架构、实现、配置都必须 100% 满足**。本节是用户视角的摘要，详读看宪法。

**项目目标**：构建一个**工具特性覆盖率测试框架**，通过样例驱动方式系统测量比较不同 Rust 工具的特性覆盖。**项目目标是"框架"，不是任何具体工具的测试报告**。

**模块分层**：
- **核心模块**（first-class，长期承诺）：测试运行与结果分析框架（`runner/`）+ 样例库（`examples/`）
- **次要模块**（non-first-class，应用展示）：市面工具样例（`tools/` + 测试报告），是核心模块的应用展示，**不作为核心目标**

**核心精神**：工具在**后端求解理想情况下**能支持的 Rust 特性范围——假设工具后端无限完美，仅测前端能不能接受这段 Rust。**不评估**：后端求解能力、翻译产物语义忠实度、验证证明难度。

**诚实测试范围**（次要模块 3 的子原则）：
- **以形式指标为最终解释** —— 各工具因设计不同取不同形式指标（exit code / 产物 grep / stderr grep / 多门组合），承认这些指标定义的诚实边界
- **上限保证（不冤枉能力）** —— oracle 0 误报第一；测试支持率是真实能力的**严格上限**，工具实际能力 ≤ 测得支持率
- **下限诚实（不高估能力）** —— 尽量无漏报；某些工具（hax × 3 / rocq-of-rust 等 grep-based）无法形式证明 0 漏报，每个工具 README 都明确声明形式严格性 + 漏报盲点
- **不允许 partial** —— SUCCESS = 工具完整完成它的工作单元；即便 partial 产物落盘也是 FAILED

**承认时效性**：测试报告锚定具体时间 + 工具版本，不构成项目对工具能力的长期承诺。

**关于 deep-reports/cc-reports/**：这些是基于具体 run 的**本地自洽 AI 分析样例**——内部失败模式归类、跨工具发现等都是某时间点的观察快照，**不构成对工具的实质评判**：(a) 不是工具本身的责任；(b) 不纳入任何工具设计与实现考量；(c) 都是外来的。详见 [`docs/design/principles.md`](docs/design/principles.md) §三-3-3 实测报告责任边界 + [`docs/design/tool-integration.md`](docs/design/tool-integration.md) §七。

---

## 测试运行环境与平台

**当前测试基线运行环境**：

- host：Apple M5 / macOS aarch64（darwin 25.4.0）
- 部分工具配置含 macOS arm64 / x86_64 硬编码路径或 target triple（详见各工具 README §"已知限制 / 平台兼容"）

**基础设施层精神**：runner 自身平台无关（纯 Rust + 标准 stdlib，无 OS-specific syscall 之外的依赖）。各工具的 platform-specific 配置封在该工具的 `tool.toml` / wrapper 内，用户可通过 `.env` / 编辑 `tool.toml` / 修改 wrapper 改造适配其他平台——基础设施不替工具屏蔽平台差异，但也不阻止用户接入。

**用户级配置**：所有 `TS_*` 前缀的环境变量都在 [`.env.example`](.env.example) 列出，用户复制到 `.env` 后按本机实际安装路径填即可。详见 `.env.example` 与各工具 README "安装" 段。

---

## 工具类别与"前端边界"

19 个工具按内部 pipeline 分五类。**本测试只测每个工具到自带后端验证器/求解器之前为止**，不进入下游求解器。每个工具的 README 都精确写出本工具的这条边界。

| 工具类 | 前端 = 测什么 | 不测的"后端" | 当前工具 |
|---|---|---|---|
| **编译基线** | rustc 类型 / 借用检查 | (无) | cargo-check |
| **解释执行类** | MIR / K MIR semantics 全程跑完；UB / unsupported 是工具有效输出 | (无 SMT 后端) | miri、kmir |
| **符号执行类** | symex 跑完；bug 检测信号是工具有效输出 | SMT 求解（Z3） | soteria、verifast |
| **翻译类（纯）** | 写出目标语言文件 | 下游 prover/checker 不调 | charon-mono / charon-poly、creusot、hax × 3、aeneas × 4、rocq-of-rust |
| **编码 + 后端 SMT 类** | 写出后端输入文件（goto-binary / .vpr / VIR） | SMT 求解（CBMC / Silicon / Z3） | kani、prusti、verus |

> creusot 在"翻译类"出现：`cargo-creusot` 默认无 subcommand 即只翻译到 `.coma`，本测试用此模式；`cargo creusot prove` subcommand 才进 SMT，不在测试范围。

### 统一精神

- **让每个工具完成它自己设计的工作单元**，不进入下游 SMT/证明求解；矩阵的横向对比因此公平
- **不允许 partial**：SUCCESS = 工具完整完成它的工作单元，不允许任何 partial / silent skip / 半翻译。即便 partial 产物落盘也是 FAILED——工具自陈"我没全干完"必须被尊重（如 aeneas 的 `Generated the partial file`，hax 的 `text!("sorry")` silent path）。被 UB / bug / verify-err 中断的解释器 / 符号执行同样按 partial 处理（中断 = 没完整跑完）→ FAILED
- **不区分翻译深浅**：浅 syntactic 搬运、深 MIR 翻译、verifier dialect 接受全部一视同仁
- **反作弊**：dry-run flag 必须让工具真的拿样例代码喂自己的前端——不能绕路给 stock rustc 走 cargo-check 等价路径
- **不预设忠实度**：翻译类工具的产物是否能被下游 prover type-check / 是否语义忠实于原 Rust，不在本测试范围

辅助脚本 `scripts/check-translation-correspondence.py` 可对翻译类工具的产物做函数级对应性测量（入口 fn + adjacency closure 在产物里有 declaration 即命中）。

---

## 当前工具集（19 个）

每条链接到该工具自己的 README（含安装步骤 + 配置说明 + 关联 limit sub-tests）。

| 工具 | 类目 | README |
|---|---|---|
| **cargo-check** | baseline（仅 cargo 编译） | [tools/cargo-check/](tools/cargo-check/README.md) |
| **kani** | model checker（前端层 `--only-codegen`） | [tools/kani/](tools/kani/README.md) |
| **miri** | Rust interpreter + UB detector | [tools/miri/](tools/miri/README.md) |
| **charon-poly** | Rust → LLBC 翻译器（多态） | [tools/charon-poly/](tools/charon-poly/README.md) |
| **charon-mono** | Rust → LLBC 翻译器（单态化） | [tools/charon-mono/](tools/charon-mono/README.md) |
| **prusti** | spec-based verifier (Viper) | [tools/prusti/](tools/prusti/README.md) |
| **creusot** | spec-based verifier (Why3) | [tools/creusot/](tools/creusot/README.md) |
| **hax-lean** | Rust → Lean 4（Hax backend） | [tools/hax-lean/](tools/hax-lean/README.md) |
| **hax-fstar** | Rust → F\* | [tools/hax-fstar/](tools/hax-fstar/README.md) |
| **hax-coq** | Rust → Coq/Rocq | [tools/hax-coq/](tools/hax-coq/README.md) |
| **aeneas-lean** | Rust → Lean 4（Aeneas backend，经 charon LLBC） | [tools/aeneas-lean/](tools/aeneas-lean/README.md) |
| **aeneas-fstar** | Rust → F\* | [tools/aeneas-fstar/](tools/aeneas-fstar/README.md) |
| **aeneas-coq** | Rust → Coq/Rocq | [tools/aeneas-coq/](tools/aeneas-coq/README.md) |
| **aeneas-hol4** | Rust → HOL4 SML | [tools/aeneas-hol4/](tools/aeneas-hol4/README.md) |
| **verifast** | spec verifier（KU Leuven） | [tools/verifast/](tools/verifast/README.md) |
| **verus** | SMT spec verifier（`verus! { }`） | [tools/verus/](tools/verus/README.md) |
| **rocq-of-rust** | Rust → Rocq（直接 THIR 翻译，Formal Land） | [tools/rocq-of-rust/](tools/rocq-of-rust/README.md) |
| **soteria** | Tree Borrows 符号执行 | [tools/soteria/](tools/soteria/README.md) |
| **kmir** | K Framework MIR operational semantics | [tools/kmir/](tools/kmir/README.md) |

每个工具有对应的 limit sub-tests（如 `examples/<tool>-limit/`），是该工具自声明的"不支持"形态——用作工具能力地图的真分化信号。

---

## Troubleshooting

### 残留 verifier 子进程（cbmc / kompile / etc 占 CPU）

runner 中断后某些工具的孙子进程可能逃出 process group。运行：

```sh
./scripts/kill-stragglers.sh
```

walk pid tree + kill process group + 杀 pid 三层 cascade，覆盖所有已集成 verifier 的进程名。

### 工具版本与平台

所有工具的安装步骤都假设 macOS Apple Silicon (Darwin / aarch64)。Linux 路径未实测。

工具的版本由各自 README 锁定（commit hash / brew tap version / nightly toolchain pin）。每次 run 的 `results.json` metadata 段记录跑时具体版本——见架构原则"工具非静态原则"。

---

## 设计要点（速览）

详见 [`docs/design/architecture.md`](docs/design/architecture.md)。要点：

1. **三原则**：(A) 例子与工具互不侵入 / (B) 测必要条件而非充分 / (C) 异质性归配置不归代码。
2. **位阶澄清**：原则 A 保护**原始磁盘的样例源码**字面零修改；隔离副本上的 manifest 注入（`extra_cargo_deps`）+ entry 入口替换（`entry_mode = "lib"`）属"中介层声明式填充"，不破 A。
3. **前端支持性观察原则**：**测工具理论上能"接受"的代码范围，不测它实际能"处理完成验证"的范围。** 本项目只筛前者：kani 用 `--only-codegen`、prusti 用 `PRUSTI_NO_VERIFY=false` + `PRUSTI_DUMP_VIPER_PROGRAM=true` + `PRUSTI_PRINT_HASH=true`（让 encoder 真跑 + Silicon 不调）、verus 用 `--no-verify`、charon 用 `--abort-on-error` 等都体现这条。**不区分翻译深浅**：浅 syntactic 搬运、深 MIR 翻译、verifier dialect 接受全部一视同仁，是覆盖度测量本性。**反作弊推论**：dry-run flag 必须让工具真的拿样例代码喂自己的前端——不能绕路给 stock rustc 走 cargo-check 等价路径（典型反例：Verus harness 里 `mod __ts_inner` 必须放 `verus! {}` 块内，详见 `tools/verus/README.md`）。
4. **工具非静态原则**：观测必须打上时间戳 + 工具版本 + host 戳；`results.json` metadata 段自描述。
5. **运行时投影**：运行原子 = 单 entry；能力靠观测，不靠声明。

---

## 已知约束

- 例子和工具都通过**目录扫描**自动发现，不维护中央注册表。
- 单 task 是**完全隔离副本**（`cp -r`，跳过 `target/` + `Cargo.lock`）——可放心并发，互不影响。
- 例子被排除在 root workspace 外（根 `Cargo.toml` 的 `exclude = ["examples", "tools", "runs", ".tmp"]`）——`cargo check` 在子目录单独操作不会牵动整个 workspace。
- 子进程在独立 process group（`process_group(0)`），timeout 时 `kill(-pgid, SIGKILL)` 杀整个 group（孙子进程一并）。
