# Axis A: Reproducibility (ISSTA / ACM Badges)

> **审查范围**：v6 final 项目（hash `ebe6858`）—— design docs / runner / 20 tools 集成 / 161 examples / `run-1778560393-59119` baseline / cc-reports × 20 / v6 报告。
> **角度**：ISSTA 2026 reviewer 评 empirical study + benchmark suite paper 的 reproducibility，按 ACM Artifact Evaluation Badges 三级。disprove-first。

---

## ACM Available（最低）

### Findings

- **[Major] 项目缺 LICENSE 文件 → ACM Available 直接 blocking**
  - 证据：`ls LICENSE COPYING LICENSE.md` 均 not found；`Cargo.toml` / `runner/Cargo.toml` 无 `license` 字段；`README.md` 无 license 段。
  - 后果：ACM Available 要求 license 明示 + 可被他人下载使用。GitHub repo 不带 license = 默认 all rights reserved，第三方不可以复制 / 修改 / 发表衍生。整套 corpus + framework 在法律上不可被其他研究者引用复现。
  - 修复：补 `LICENSE` 文件（Apache-2.0 / MIT 推荐学术 benchmark）+ 在 `runner/Cargo.toml` 加 `license = "..."` 字段 + README 顶部声明。

- **[Major] 无 Zenodo / archive DOI 锚定 → 不满足 Available 长期可获取性要求**
  - 证据：`grep -rE "DOI|zenodo|10\\.[0-9]" README.md docs/` 无命中。
  - 后果：ACM Available 要求 archive in a public archive (Zenodo / FigShare / Software Heritage)。仅 GitHub commit `ebe6858` 不足——GitHub repo 可删 / 改 / force-push。审查锚定的"v6 final"实际不可永久回溯。
  - 修复：发布 Zenodo release（含 commit + DOI），README 顶部贴 DOI badge。

- **[Minor] 三个 vendor/ submodule 的版本未锁到 commit hash**
  - 证据：`.gitmodules` 仅给 `url`，未指定 branch 或 ref；submodule HEAD 由 superproject's tree 锁定但用户必须执行 `git submodule update --init --recursive` 才能拿到对应 commit。README 提到"vendor/x509-parser 是 git submodule，不动"但**整个 README + docs 无任何处提到 `git submodule update --init --recursive` 命令**。
  - 后果：新人 `git clone` 后跑 runner，`examples/industrial/x509-parser/cert-parse/` 的 path dep `../../../../vendor/x509-parser/` 是空目录 → cargo build 报 missing `Cargo.toml` → 三个 industrial entries 全 FAILED（虚假 FAILED，复现不出 baseline 的 SUCCESS/FAILED 比例）。
  - 修复：README"怎么跑"段第一步加 `git clone --recurse-submodules` 或 `git submodule update --init --recursive`。

---

## ACM Functional

### Findings

- **[Major] `.env.example` 与实际 tool.toml 之间存在 env var 漂移，新人按 `.env.example` 配会有缺项**
  - 证据：
    - `tools/rocq-of-rust-typecheck/tool.toml:28` 引用 `TS_ROCQ_OF_RUST_BIN`；`.env.example` 全文未声明。
    - `.env.example` 声明 `TS_HAX_LEAN_PRELUDE_DIR`；20 个 `tools/*/tool.toml` 与所有 wrapper 均不引用（孤儿声明，设计文档 P16-impl-B 说"启用时"用——但当前未启用即不在配置里）。
  - 后果：新人按 `.env.example` 填好后跑 runner，rocq-of-rust-typecheck 因缺 env 走 default 路径，可能影响结果但用户不可见。Reusable 也受影响。
  - 修复：把 `TS_ROCQ_OF_RUST_BIN` 写进 `.env.example`；把 `TS_HAX_LEAN_PRELUDE_DIR` 移除或标 `# RESERVED, not currently used`。

- **[Major] tool README 的"安装"段统一逃避到上游，无可重现 install 脚本**
  - 证据：`tools/prusti/README.md` "安装" 段："按上游文档自行安装（macOS arm64 上需要 Rosetta + x86_64 toolchain + x86_64 JDK，参考上游 release notes 与项目 wiki）。**本项目不提供安装脚本或步骤教程，避免在工具版本变迁后误导。**" 同 pattern 复现于 cargo-check / kani / verus / kmir / verifast / soteria / aeneas × 4 / hax × 3 / rocq-of-rust × 2 / creusot / charon × 2 / miri = 全部 20 个。
  - 后果：ACM Functional 要求"complete install instructions"。"自行参考上游"是反 reproducibility 的——上游文档/路径随时可漂，commit pin 在上游 README 找不到本地配置语境。审查 reviewer 无法在 4 小时内装齐 20 个工具中的任一难装项（prusti / kmir / soteria 至少需要 OCaml opam switch / K Framework / Rosetta + 老 toolchain 组合）。
  - 修复：每个工具 README 末加可执行 "Install Recipe"（具体命令 + commit hash + 已知 platform 限制），即便会随时间过期——这是 reproducibility 的核心义务，不是过期成本的借口。

- **[Major] baseline run 的 `results.json` `tools[].command` 内嵌绝对路径 `/Users/ssyram/...` + `/tmp/ts-tools-install/...`**
  - 证据：`runs/run-1778560393-59119/results.json` 多处 `"/Users/ssyram/workspace/rust-ver/rust-ver-tool-testsuite/tools/..."` 与 `/tmp/ts-tools-install/charon/bin/charon` 与 `/Users/ssyram/.opam/default/bin/hax-engine` 等。
  - 后果：他人无法直接比对 command 数组、也无法在没有这些路径的机器上"重放"该 run。command 字段本应是机读重放语义但内嵌私 host 路径，纯纪念性。
  - 修复：在 `results.json` 落盘前做 path normalize（替换为 `${TS_PROJECT_ROOT}` / `${TS_CHARON_BIN}` 等 placeholder）；或额外存原始 `tool.toml` 内容（含 placeholder）+ `expanded_command`（含本机绝对路径）双字段。

- **[Minor] README 提"残留 verifier 子进程"用 `./scripts/kill-stragglers.sh`，但未说何时该跑**
  - 证据：README §"Troubleshooting"。`scripts/kill-stragglers.sh` 存在。但跑全 161×20 矩阵跑 1334s 期间可能有何残留无定量描述；reviewer 跑半小时 partial 不知该不该跑。
  - 修复：补可观察的征兆描述（如 `ps -ef | grep -E "(cbmc|kompile|...)"` 不空即应跑）。

- **[Minor] `cargo build -p runner` clean checkout 实测可成功** ✓
  - 证据：本审查跑 `cargo build -p runner --message-format=short` exit 0，`Finished dev profile`。runner 自身 ≈ 1.3s build。这是唯一 Functional 级的硬正面。

- **[Minor] 缺一个 reviewer-friendly "minimal repro" demo**
  - 证据：README "怎么跑" 直接讲全矩阵。但 reviewer 装齐 20 工具前的快速 sanity check（如 `runner --tool cargo-check --entry 'hello/**'`）的入门示例没出现。
  - 修复：在 README 加 "5-minute smoke test"：仅 cargo-check（zero 外部依赖）跑 1 个 hello entry。

---

## ACM Reusable（最严）

### Findings

- **[Major] runner 没暴露 `results.json` JSON schema 定义文件**
  - 证据：`detailed-design.md:517-650` 给的是"JSON example"格式，而非可机读的 schema（JSON Schema / OpenAPI / 至少 `serde` struct doc）。第三方工具消费 `results.json` 只能照搬示例字段名 + 看 runner src/report.rs 反推。
  - 后果：ACM Reusable 强调 "data schema documented"。reviewer 想做 longitudinal study（拼接多 run 做时序对比）必须自己 reverse engineer schema。
  - 修复：发布 `results.schema.json`（JSON Schema draft-07/2020-12）或在 `detailed-design.md` 补"字段表 + 类型 + 取值集"完整 schema 段。

- **[Major] "Add a new tool" 教程仅覆盖最简两件套，未覆盖实测中 20 个工具有 15 个用 wrapper + grep oracle 的复杂 case**
  - 证据：README "加一个新工具" 段示例 `command = ["cargo", "mytool", "--bin", "__ts_harness"]` + 4 行 harness。但实际 20 个 tool.toml 中：hax × 3 / aeneas × 4 / soteria / verifast / kani / prusti / kmir / rocq-of-rust × 2 / miri = 15 个用 wrapper.sh + 多 gate grep oracle（产物 grep / stderr grep / 多门 AND）。新增工具如果 silent partial 模式不显见，按 README 简易教程做出来的集成会漏报。
  - 修复：在 `docs/design/tool-integration.md`（已存在）写 "advanced integration" 段并在 README 链接；列出 silent partial detection 的最少 gate 集合 + 推荐 wrapper 模板。

- **[Minor] README"加一个外部综合项目（如 vendor/x509-parser）"段说必须用目录轨 `.hirusttest/config.toml`；但实测 `examples/industrial/x509-parser/cert-parse/` 仍用单文件 `hirusttest.toml`，且无 `.hirusttest/` 目录**
  - 证据：README L137–148 vs `examples/industrial/x509-parser/cert-parse/hirusttest.toml` 实际是单文件 + `[env]` 段。`ls examples/industrial/x509-parser/cert-parse/.hirusttest/` not found。
  - 性质：文案与实现不一致（R11 类）。reviewer 试图按 README "加外部综合项目" 的指引去做新 vendor 集成，会发现实测项目自身没按这条规则——困惑。
  - 修复：要么补真目录轨实例（移 cert-parse 到 `.hirusttest/`），要么 README 改成"单文件轨已够，目录轨为可选预留扩展"。

- **[Minor] corpus 的 entry 设计原则文档化够，但反作弊文档化偏弱**
  - 证据：README 提"反作弊：dry-run flag 必须让工具真的拿样例代码喂自己的前端"，给了 Verus mod __ts_inner 反例。但 161 entries 没单独的反作弊审计记录（如"每条 entry 通过了哪些反作弊 invariant"）。
  - 修复：可在 `docs/design/detailed-design.md` 加 corpus 健全性 invariant 清单（无 `#[kani::proof]` / 无 `verus! {}` / 无 `#[ensures]` 的 grep + 自动 CI 检查）。

- **[Minor] runner CLI 有 `runner report <run-dir>` 子命令但旧 run 重分析路径未在 README 显式宣称版本化承诺**
  - 证据：README L54-58 提到 "Schema 演化时无需重跑工具"；但未明示 schema backward-compat 承诺（如 `run_id` 字段一旦发布永不重命名）。reusable 标的 longitudinal benchmark 缺这条会受 schema drift 风险。
  - 修复：在 `detailed-design.md` 加 schema versioning policy（如 `results.json` 加 `"schema_version": "1.0"` 字段，并承诺 minor 加字段 / major 删字段）。

- **[Style] `runs/` 下有 130+ 历史 run 但 `.gitignore` 未排 → 仓库膨胀 + 旧数据混入复现实验**
  - 证据：`ls runs/ | wc -l` ≈ 130+。多数是中间调试 run。
  - 修复：把 runs 加进 `.gitignore`，只保留 v6 baseline run 单独提交到 `runs/baseline/` 或 release artifact。

---

## Summary

- **Major findings**: 6（Available 2 / Functional 3 / Reusable 2，含 1 跨级 results.json 路径）
- **Minor findings**: 7
- **Style findings**: 1

### ACM Badge assessment

- **Available** ✗ —— 缺 LICENSE + 缺 archive DOI 双 blocker
- **Functional** ✗ —— 安装步骤全部"自行参考上游"实质不可重现 + results.json 嵌绝对路径不可重放
- **Reusable** ✗ —— 缺 schema 定义文件 + 新增工具教程未覆盖真实复杂度

### 关键 blockers（按 acceptance 风险排序）

1. **LICENSE 缺失**（Available 一票否决，几分钟可修）
2. **20 个工具 README 无可重现 install recipe**（Functional 核心，工作量大）
3. **results.json 内嵌私 host 绝对路径**（Functional 二号，半天可修）
4. **archive DOI 缺失**（Available，需操作 Zenodo）
5. **JSON schema 未文档化**（Reusable，需补 schema.json）
