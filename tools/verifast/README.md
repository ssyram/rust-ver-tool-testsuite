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

### 重要：本 corpus 上 VeriFast 实际工作量

**实测发现**：本测试集所有 entry **零** `//@ req/ens/inv/pred` spec 注解（grep `//@\s*(req|ens|inv|pred)` 全 corpus 0 命中）。配合 tool.toml 三件 flag（`-skip_specless_fns` + `-disable_overflow_check` + `-ignore_unwind_paths`），实际效果是：

- `-skip_specless_fns` 让所有用户函数被跳过（无 spec 即不 verify）
- `-disable_overflow_check` 关掉算术溢出隐式断言
- `-ignore_unwind_paths` 关掉 panic 路径

**结果**：116 个 SUCCESS 中 **104 个**报告 `37 statements verified`（同一常数 baseline，来自 verifast 自身 prelude / harness 的 IR 统计），剩 12 个为 38–40（差 0–3 条）。**verifast 的 verification 功能在本 corpus 上事实上被关闭**，SUCCESS 实际含义降级为：

> rustc-verifast 接受源码进入 IR + 没有 spec 可证伪

而非"symex 在用户代码上完成"。本 corpus 0 个真 verify-err entry。

这与项目"覆盖度只看是否在册接受、不区分深浅"的原则不矛盾——前端 IR 接受是真发生了。但读 verifast 的 SUCCESS 数字时务必留意：**SUCCESS ≠ verification 实际发生**。

如果未来要让 verifast 真做 verification，需要给 entry 加 spec 注解（不符合"entry 自包含 + 不为单工具加 boilerplate"原则），或不加 `-skip_specless_fns`（plain Rust 会因溢出 / panic 路径触发误判）。

### SUCCESS 信号（严格反映前端特性支持范围）

为了严格反映前端特性支持范围（不允许 partial），**SUCCESS = verifast exit 0**。任何 partial / IR 拒收 → FAILED。

- **partial 暴露机制**：rustc-verifast 任何 IR 构造失败（async / closure / 浮点 / const-generic 等）→ exit ≠ 0；symex verify err → exit 1
- **形式严格性 — 0 误报（不冤枉能力）**：✅ 形式可证。verifast exit 0 ⇔ 前端 IR 构造完成（在 spec-less 下：无可证伪项）
- **形式严格性 — 0 漏报（不高估能力）**：✅ 形式可证。rustc-verifast 任何 IR 构造失败 → exit ≠ 0；symex verify err → exit 1
- **重要注（语义降级，不是漏报）**：当前 corpus **0 个** spec 注解 + tool.toml `-skip_specless_fns + -disable_overflow_check + -ignore_unwind_paths` 三件套 → SUCCESS 实际语义降级为 **vacuous pass**（rustc-verifast 接受源码进入 IR + 无 spec 可证伪），**不**等于 verification 实际发生。这不是 oracle 的漏报，是 corpus 设计与工具特性使然——读者在引用 verifast 支持率时务必留意

## 安装

上游：<https://github.com/verifast/verifast>

本测试基线：26.01（released 2026-01-21）。

按上游文档自行安装（官方提供 macOS arm64 prebuilt tarball）。装好后把 `verifast` 可执行文件路径填到 `.env` 的 `TS_VERIFAST_BIN`。本项目不提供安装脚本或步骤教程，避免在工具版本变迁后误导。

## 本框架配置

参见 `tool.toml`。关键参数：

- **command**：直接调用 `verifast` binary，目标 `src/lib.rs`；不经过 `cargo verifast`（后者在 lib+bin 双 target 时报错）。
- `-target macOS`：指定 LP64/arm64-apple-macosx 平台，决定指针与整数宽度。
- `-shared`：跳过"必须有 main 函数"检查（lib crate 用）。
- `-skip_specless_fns`：跳过无 `//@ req/ens` 注解的函数，使 plain Rust 样例 exit 0（工具看不到 spec → 无可证伪项）。
- `-ignore_unwind_paths`：忽略 panic/unwind 路径（覆盖度筛选 run 可接受；sound 条件为 `-C panic=abort`）。
- `-disable_overflow_check`：关闭算术溢出检查（无 range precondition 时 plain 代码会误报）。
- `-read_options_from_source_file`：允许文件首行 `// verifast_options{...}` 覆盖上述默认值。
- **entry_mode**：未设置（默认 `bin`）；harness 写入 `src/bin/__ts_harness.rs`，但 verifast 只读 `src/lib.rs`，harness 被忽略。
- **harness**：空的 `fn main() {}`——仅满足框架契约，verifast 不通过 cargo 链接跨 crate，无法解析跨 crate 调用。

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

plain Rust 样例在 `-skip_specless_fns` 下全部预期 SUCCESS（工具静默通过）。带正确 `//@ req/ens` 注解的样例进入 SMT 验证；带错误 spec 的样例预期 FAILED（exit 1）。
