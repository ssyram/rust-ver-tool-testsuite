# 修正方案：`extra_cargo_deps` + `entry_mode` 引入

集成 Creusot 时遇到的两个互锁问题。两次都涉及 `tool.toml` schema 扩展并触及原则 A 的位阶解读，按 workflow §5.1 把"复现 / 根因 / 参考实现 / 修正方案"四节沉淀于此。

---

## 1. 复现

### 1.1 第一阶段：`extra_cargo_deps`

把样例集中无任何工具相关 dep 的 plain Rust crate（如 `examples/hello/basic-hello/`）通过 runner 喂给 Creusot：

```sh
$ cargo-creusot   # 在隔离副本里跑
error: creusot-std not found in dependencies
```

Creusot 在 cargo 解析阶段直接拒——它要求样例 `Cargo.toml` 的 `[dependencies]` 里**字面列出** `creusot-std`，没有这个 dep 不进入编译流程。

### 1.2 第二阶段：`entry_mode`

加完 `extra_cargo_deps` 注入后，cargo-creusot 进入编译阶段，但样例 lib 仍报错：

```
Checking creusot-std v0.11.0
Checking basic_hello v0.1.0  ← 我们样例的 lib crate
error: The `creusot_std` crate is not loaded.
  = note: Don't forget to actually use creusot_std: `use creusot_std::prelude::*;`
```

harness（`src/bin/__ts_harness.rs`）顶部已经 `use creusot_std::prelude::*;`，但 cargo-creusot 不只检查 harness——它对整个 cargo project 的所有 crate target 都跑 creusot-rustc。**样例 `src/lib.rs` 是顶级 lib target**，它没有 use 语句，creusot-rustc 直接拒。

---

## 2. 根因

### 2.1 第一阶段根因

Creusot 强制要求"被 verify 的 crate 在 `Cargo.toml` 里直接 dep `creusot-std`"。这是 cargo-creusot 入口处的硬检查，不是 creusot 验证逻辑本身的需求。

我们的样例端原则 A：**"样例不为任何工具修改、不依赖任何工具相关 crate"**。两者直接冲突。

### 2.2 第二阶段根因

cargo-creusot 是 cargo 的 wrapper：它把 `RUSTC` 环境变量替换成 `creusot-rustc`，让 cargo 编译流程中**每个 crate target** 都经过 creusot-rustc 处理。creusot-rustc 在每个 crate 的 entry（顶级 module）查找 `use creusot_std::*` 或 `extern crate creusot_std;`——找不到就报 "creusot_std crate is not loaded"。

Rust 的 use / extern crate 是 crate-local 概念。harness（bin target）的 use 不能被 lib target 看见。我们要让样例 lib 满足 creusot-rustc 的要求，必须在**样例 lib 的源码**里 import creusot_std。

但**样例源码层** 同样受原则 A 约束。

---

## 3. 参考实现思路（评估）

| 思路 | 描述 | 评估 |
|---|---|---|
| **(A) 修改样例 src/lib.rs 顶部加 `use creusot_std::*;`** | 直接破原则 A 源码层 | ✗ 拒绝——样例不为工具改 |
| **(B) 样例 Cargo.toml 直接列 `creusot-std` 为 optional dep** | 部分让步原则 A，且仍要解决 lib use | ✗ 不彻底 |
| **(C) `extra_cargo_deps` 字段：runner 在隔离副本上 inject manifest** | 原始磁盘样例零改动；副本 manifest 由 runner 声明式填充 | ✓ 解第一阶段 |
| **(D) 把 use 也注入隔离副本的 src/lib.rs 顶部** | 仍是改副本 lib 源码——单点 ad hoc，未来工具一种新需求又来一字段 | ✗ ad hoc |
| **(E) `entry_mode = "lib"`：harness 取代副本 lib，原 lib 内嵌为 mod __ts_inner** | runner 在副本上重新组织 crate 入口结构，harness 是顶级 lib，原 lib 文件原样保留只换名+内嵌；声明式机制，可被任何同类工具复用 | ✓ 解第二阶段 |

**(C) + (E) 联合**是最终方案。

---

## 4. 修正方案

### 4.1 schema 扩展

`tool.toml` 增加两个可选字段：

```toml
extra_cargo_deps = ['creusot-std = "0.11.0"']   # 可选，默认 []
entry_mode       = "lib"                         # 可选，默认 "bin"
```

字段语义见 `docs/research/testsuite-research.md` §6.2。

### 4.2 runner 流程扩展

详见 §7.2：cp 后多两个声明式步骤——

- 3b. `extra_cargo_deps` 非空 → toml AST 编辑 inject 到副本 `Cargo.toml` `[dependencies]` 表
- 5b. `entry_mode = "lib"` → 副本里 `src/lib.rs` 改名为 `src/__ts_inner.rs`，渲染好的 harness 写到新 `src/lib.rs` 当顶级 lib

### 4.3 原则 A 位阶澄清

详见 §2 / §5.8：原则 A 保护的是**原始磁盘的样例源码**字面零修改。隔离副本上的 manifest 注入与 lib 取代都属"中介层声明式填充"——runner 在副本上为工具准备的环境，不是样例端为工具让步。

承诺强度从原版"样例对所有工具完全无感"降级为"**原始磁盘字面零修改 + 副本上声明式工具特定填充**"。

### 4.4 兼容性

两个新字段都 `#[serde(default)]`：旧 `tool.toml`（cargo-check / kani / miri / charon poly+mono / prusti）零改动。

### 4.5 反例域

引入 `extra_cargo_deps` + `entry_mode = "lib"` 后，仍**无法纳入**的工具类型（§5.6 反例域）：

- 要求修改 entry 函数自身
- 不通过 cargo 子命令运行
- 要求多文件协同特定布局（同包内特定相对位置）
- 要求样例 `Cargo.toml` 含 `[features]` / `[patch]` / profile 配置 而非仅 `[dependencies]` 行——当前 schema 只支持 `[dependencies]` 表，不覆盖更复杂的 manifest 布局（未来可同范式扩展，例如加 `extra_cargo_features` 字段）

---

## 关联 commit / 文件

- 设计：`docs/research/testsuite-research.md` §2 / §4.1 / §5.5 / §5.8 / §6.2 / §6.5 / §7.2 / §10.1
- 代码：`runner/src/discover.rs`（`EntryMode` enum + `Tool` 字段），`runner/src/exec.rs`（`patch_cargo_deps` toml AST 实现 + entry_mode 分支）
- 配置：`tools/creusot/{tool.toml, harness.rs.tera}`
