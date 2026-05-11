# VeriFast

Separation-logic verifier for C and Rust, using symbolic execution and an SMT prover.

## 简介

VeriFast 由 KU Leuven 出品，通过分离逻辑注解（`//@ req/ens`）对 C 和 Rust 做形式化内存安全证明。内核为符号执行 + 内置 Z3v4.5/Redux prover，不依赖 Cargo，直接读取单个 `.rs` 文件。版本 26.01 提供原生 arm64 prebuilt binary，无需编译。

GitHub: <https://github.com/verifast/verifast>

## 本测试集中的"前端接受"定义

本测试集筛选 Rust **前端特性覆盖广度**——把每个工具测到"工具自带后端验证器/求解器之前"为止。

VeriFast 是 separation-logic 符号执行验证器，pipeline：rustc-verifast (内置 rustc driver) → MIR → VeriFast IR → 符号执行 + 路径前置/后置约束生成 → SMT (Z3v4.5 / Redux)。

按宪法 §六-2 不允许 partial（"SUCCESS = 工具完整完成它的工作单元，不允许任何 partial / silent skip / 半翻译"）："工具完成它自己的工作" = VeriFast **完整跑完符号执行且无中断**。具体：

- **算 SUCCESS**：exit 0 = 验证通过（在 `-skip_specless_fns` 下，无 spec 即无可证伪项 → 自动 exit 0）
- **算 FAILED**：任何中断
  - exit 1 = 验证失败（symex 在某条路径上证明失败）—— 符号执行被 verify-err 中断 = 没完整跑完
  - rustc-verifast 在 IR 构造阶段拒收（语言层不支持的构造，如 closure types、async fn 等）

> 注意 `-skip_specless_fns` 的语义：plain Rust 样例 exit 0 = "VeriFast 接受源码进入 IR，没有 spec 可证伪"，**不等于"验证通过"**。
>
> 注：verify err 在某些视角下可理解为"工具有效输出"，但按宪法精神（不允许 partial = 必须完整跑完）一律 FAILED。oracle（runner 按 exit code 判定）与宪法一致。

- **产物**：`-emit_sexpr` / `-emit_rocq` / `-dump_ast` 可让 VeriFast IR 落盘（含 `(declare-function <fn-name> :type-parameters () :return-type ... :body ... :precondition ... :postcondition ...)`）—— 但 `-emit_sexpr` 在某些 case（如 lifetime 复杂 entry）internal emitter 会 panic，落地不稳，目前未启用

### 本 corpus 上 VeriFast 实际工作量（2026-05-08 起）

**corpus 现状**：本测试集所有 entry **零** `//@ req/ens/inv/pred` spec 注解（grep `//@\s*(req|ens|inv|pred)` 全 corpus 0 命中）。配合 tool.toml 三件 flag（`-skip_specless_fns` + `-disable_overflow_check` + `-ignore_unwind_paths`），实际效果是：

- `-skip_specless_fns` 让所有用户函数被跳过（无 spec 即不 verify）
- `-disable_overflow_check` 关掉算术溢出隐式断言
- `-ignore_unwind_paths` 关掉 panic 路径

**2026-05-08 oracle 调整前**：在仅 exit 0 的 oracle 下，所有 spec-less entry 报 SUCCESS（116/116），但 stdout 仅一行 baseline `0 errors found (37–40 statements verified)`，其中 N 来自 verifast 自身 prelude，**与用户代码无关**。

**2026-05-08 oracle 调整后**：[`verifast-strict-wrapper.sh`](verifast-strict-wrapper.sh) 用 `-verbose 1` 检测 symex 是否触及 user file，spec-less entry 在新 oracle 下被识别为 vacuous pass → FAILED。**预期 SUCCESS 数从 116 (79.5%) 降到 0–2 (0–1.4%)**——这是合理的，因为本 corpus 在当前 corpus 设计上就不应该让 verifast 有非 trivial 真 SUCCESS。

如果未来要让 verifast 真做 verification，需要给 entry 加 spec 注解（不符合"entry 自包含 + 不为单工具加 boilerplate"原则），或换工具（如 hax / kani）评估 verifier-deep 部分。

### SUCCESS 信号（严格反映前端特性支持范围）

为了严格反映前端特性支持范围（不允许 partial），**SUCCESS = verifast 通过 wrapper 的双重检查**：

1. verifast exit 0
2. **`-verbose 1` 输出中至少 1 行包含用户源文件路径锚 `src/lib.rs(`** —— 即 symex 至少在用户代码上执行了 1 条 statement

任何门未满足 → FAILED。

第二道门由 [`tools/verifast/verifast-strict-wrapper.sh`](verifast-strict-wrapper.sh) 强制；2026-05-08 之前的 oracle 仅用 exit 0 单道门，会让 spec-less corpus 上的 vacuous pass 全部错判 SUCCESS（详见 [`docs/fixes/oracle-leak-audit-2026-05-08.md`](../../docs/fixes/oracle-leak-audit-2026-05-08.md) §3.1）。

- **partial 暴露机制**：rustc-verifast 任何 IR 构造失败（async / closure / 浮点 / const-generic 等）→ exit ≠ 0；symex verify err → exit 1；vacuous pass（symex 未触及 user file）→ wrapper 重写为 exit 2
- **形式严格性 — 0 误报（不冤枉能力）**：✅ 形式可证。verifast exit 0 + verbose 输出含 user-file 行 ⇔ symex 在用户代码上执行了至少 1 条 statement（每条 verbose 行就是 verifast 自己的 tracing 标签）
- **形式严格性 — 0 漏报（不高估能力）**：✅ 实测 + 设计论证。`-skip_specless_fns` 跳过 user fn 时 verifast 不会发任何带用户源文件路径的 verbose 行（仅 prelude 行）—— 0 漏报由 verifast 设计强制；rustc-verifast 任何 IR 构造失败仍走 exit ≠ 0 老路径
- **反误报双向实测**：[`oracle-validation/`](oracle-validation/) 子目录含 spec-less + spec-bearing 两个 micro-test，验证规则在两个方向上都正确触发 / 不触发；详见该目录 README

### vacuous-pass 历史口径修订（重要）

2026-05-08 之前 README 把"`-skip_specless_fns` 让 spec-less entry 退化为 vacuous pass"称为"语义降级，不是漏报"。审计（[`docs/fixes/oracle-leak-audit-2026-05-08.md`](../../docs/fixes/oracle-leak-audit-2026-05-08.md) §3.1 / §7.1）按项目宪法 §6 反作弊原则口径校正：**SUCCESS = 工具完整完成它的工作单元**——`-skip_specless_fns` 让 entry 完全没经过 verify 阶段，本质就是 silent skip，符合 partial 定义，应封堵。

新 wrapper 落地这次口径校正。**当前实施下，SUCCESS = symex 真跑过用户代码**，与项目宪法对齐。

## 安装

上游：<https://github.com/verifast/verifast>

本测试基线：26.01（released 2026-01-21）。

按上游文档自行安装（官方提供 macOS arm64 prebuilt tarball）。装好后把 `verifast` 可执行文件路径填到 `.env` 的 `TS_VERIFAST_BIN`。本项目不提供安装脚本或步骤教程，避免在工具版本变迁后误导。

## 本框架配置

参见 `tool.toml`。关键参数：

- **command**：通过 [`verifast-strict-wrapper.sh`](verifast-strict-wrapper.sh) 调用 verifast binary（不直接走 cargo verifast——后者在 lib+bin 双 target 时报错）。Wrapper 用 `-verbose 1` 包装并实施 vacuous-pass 检测（详见 §SUCCESS 信号 + wrapper 文件头注释）。
- 给 verifast 的实际 flag（在 wrapper 内）：
  - `-target macOS`：指定 LP64/arm64-apple-macosx 平台，决定指针与整数宽度。
  - `-shared`：跳过"必须有 main 函数"检查（lib crate 用）。
  - `-skip_specless_fns`：跳过无 `//@ req/ens` 注解的函数。Wrapper 在 spec-less corpus 上把这个的 silent skip 行为翻译为 FAILED（见 wrapper 头注）。
  - `-ignore_unwind_paths`：忽略 panic/unwind 路径（覆盖度筛选 run 可接受；sound 条件为 `-C panic=abort`）。
  - `-disable_overflow_check`：关闭算术溢出检查（无 range precondition 时 plain 代码会误报）。
  - `-read_options_from_source_file`：允许文件首行 `// verifast_options{...}` 覆盖上述默认值。
  - `-verbose 1`（wrapper 加）：让 symex 打 per-statement 的 source-path 标签到 stdout，给 vacuous-pass 检测提供 grep 锚。
- **entry_mode**：未设置（默认 `bin`）；harness 写入 `src/bin/__ts_harness.rs`，但 verifast 只读 `src/lib.rs`，harness 被忽略。
- **harness**：空的 `fn main() {}`——仅满足框架契约，verifast 不通过 cargo 链接跨 crate，无法解析跨 crate 调用。
- **VERIFAST_BIN env 重导出**：`tool.toml` 用 `env VERIFAST_BIN=${TS_VERIFAST_BIN}` 重导出（runner 在 spawn child 前 strip 所有 `TS_*`）。Wrapper 读 `VERIFAST_BIN`。

## 已知限制 / 坑

| 限制 | 说明 |
|------|------|
| 单文件模式 | verifast 直接处理 `src/lib.rs`，不支持 `mod foo;` 引用外部文件模块 |
| 无跨 crate 调用 | 不经 cargo 链接，harness 中的 `crate::fn()` 调用无法解析 |
| `-skip_specless_fns` 语义 | plain Rust exit 0 = "无 spec 可证伪"，非"代码通过验证" |
| panic 路径 | `-ignore_unwind_paths` 关闭后不验证 panic 行为 |
| 溢出检查 | `-disable_overflow_check` 后不验证整数溢出 |
| 别名模型 | VeriFast 不完全验证 Rust mutable reference aliasing 规则 |

## 关联 sub-tests

本工具未派生限制集 agent，无 `examples/verifast-limit/`。

**P12 修订前**：plain Rust 样例在 `-skip_specless_fns` 下全部预期 SUCCESS（工具静默通过）。

**P12 修订后（2026-05-08，当前生效）**：plain Rust 样例在 `verifast-strict-wrapper.sh` 的双重检查（exit 0 + `-verbose 1` symex 触及 user file）下预期 FAILED——`-skip_specless_fns` 让所有用户函数被跳过，是 vacuous pass 而非"真验证用户代码"。详见 §SUCCESS 信号 + vacuous-pass 历史口径修订段。带正确 `//@ req/ens` 注解的样例进入 SMT 验证；带错误 spec 的样例预期 FAILED（exit 1）。
