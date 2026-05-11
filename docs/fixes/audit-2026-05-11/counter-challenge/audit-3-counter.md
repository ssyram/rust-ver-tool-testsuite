# Counter-Challenge：audit-3 examples-review 验证报告

> 验证范围：`docs/fixes/audit-2026-05-11/examples-review.md` 共 37 条问题（3 高 + 大量中 / 低 + 14 决策点）。
> 方法：disprove-first，每条结论引 source 行号 + 独立证据（grep / cargo check / cargo metadata 实测）。
> 执行日期：2026-05-11。验证 agent #3。

---

## §0 总览

| 严重度 | audit 论断 | 验证结果 |
|---|---|---|
| 高 #18 industrial 违反双轨 | "principles §四 强制边界 2 + design §41 vendor/ 强制目录轨" | **核心成立**（principles 字面强制），**但有解释空间**（architecture 的"vendor/ 真本体"语义不完全覆盖 industrial thin-wrapper）|
| 高 #20 vendor/sha2 monorepo 副作用 | "path dep 触发 25-crate resolution + 3 patch 副作用 + 与 charon arm64 path bug 可能相关" | **错误**——`cargo metadata` 实测 industrial sha2 resolve graph 仅 11 crate，不含 sha2 workspace 任何其他 member；patches 在 industrial entry 不生效（audit 自己也承认）；与 charon arm64 bug **无关**（后者是 `--bin rlib path bug`，与 vendor monorepo 路径无关） |
| 高 #15 hax-limit/unsafe-block nightly only | "stable 不接受 `u32::unchecked_add`，破 cargo-check 基线" | **错误**——stable rustc 1.95.0 已稳定 `unchecked_add`，本地 cargo check 通过；runs/run-1778226613-5282 实测 cargo-check 对该 entry status = SUCCESS（exit_code 0, 171ms） |

整体评级：**B**——核心宪法 / corpus 问题识别正确，但 3 个高严重度有 2 个事实判断错误（vendor monorepo + nightly only）；术语精度偶有失分（"UB" / "submodule pin"）。

---

## §1 高严重度逐条验证

### 1.1 §3.4-#18：industrial 三件套违反双轨 schema 边界

**audit 论断**（examples-review.md §192-208）：

> principles.md §四 原则 A 双轨 schema 强制边界 2：
> "外部综合项目**不允许降级到单文件轨**——必须用目录（即便目前只有一个 entry，也用目录轨以预留扩展空间）"
>
> 而 `examples/industrial/{rsa,sha2,x509-parser}/.../hirusttest.toml` 均为单文件轨。`find examples -name '.hirusttest' -type d` 返回空——0 个目录轨实例。
> 当前实现是**宪法级违反**。

**独立验证**：

1. **principles.md §171 字面确实强制**：
   ```
   2. 外部综合项目**不允许**降级到单文件轨——必须用目录
      （即便目前只有一个 entry，也用目录轨以预留扩展空间）
   ```
   detailed-design.md §107-112 复述同一边界（`#### 目录轨的硬性边界` 第 2 条）。

2. **industrial 三件套实际结构**（`ls examples/industrial/*/*/`）：
   ```
   examples/industrial/rsa/rsa-pkcs8/hirusttest.toml          ← 单文件
   examples/industrial/sha2/sha256-digest/hirusttest.toml    ← 单文件
   examples/industrial/x509-parser/cert-parse/hirusttest.toml ← 单文件
   ```
   `find examples -name '.hirusttest' -type d` 实测返回**空**。audit 事实陈述正确。

3. **但"外部综合项目"定义有解释空间**：

   principles.md §166 表格里目录轨定义：
   > 外部综合项目——非本项目原创、可能是 git submodule 或 vendor crate 的真实 codebase（如 `vendor/x509-parser/`、`vendor/openssl/`）

   architecture.md §41 更窄：
   > 目录轨（外部综合项目，**仅适用 vendor/ 下的真实 codebase**）

   而 industrial 三件套**不在 vendor/**——位于 `examples/industrial/<vendor-name>/<thin-wrapper>/`。读 `examples/industrial/sha2/sha256-digest/src/lib.rs`（23 行）：

   ```rust
   use sha2::{Sha256, Digest};
   pub fn sha256_digest_one_shot() {
       let data = b"the quick brown fox jumps over the lazy dog";
       let _ = Sha256::digest(data);
   }
   pub fn sha256_digest_incremental() { ... }
   ```

   这是**本项目原创的 thin wrapper lib crate**——按 principles.md §166"非本项目原创"字面定义，不严格落入"外部综合项目"。

4. **解释空间结论**：
   - 严格字面派：industrial 是本项目原创小 lib，不算"外部综合项目"，无需目录轨 → audit 论断不成立
   - 精神派：industrial 在 examples §三 梯队归属是"应用层综合项目"（principles.md §65 "如 `industrial/` 下的 vendor crate"），按梯队精神应走目录轨 → audit 论断成立

   **principles.md §65 给的"综合样例"举例就是 `industrial/`**——这条把 industrial 钉死在"综合层"位置。所以**精神派胜**：industrial 三件套应走目录轨。

5. **但 audit 的"宪法级违反"表述过强**：

   industrial 当前是单文件轨，**未触发 v1 任何已知功能问题**（audit §3.4-#18 末段自承"实际功能未影响——破的是预留扩展空间的硬边界"）。属于"corpus 工程任务"级问题，不是"宪法精神级 collapse"。audit 标"高严重度"略偏紧——按 D1 高优先级建议合理，但情绪用词（"宪法级违反"）过强。

**结论**：**核心成立 + 解释空间留 + 严重度表述偏紧**。industrial 应迁目录轨（D1 高优先级），但属 fix-forward 工程任务，不破核心精神。

---

### 1.2 §3.4-#20：vendor/sha2 25-crate monorepo workspace 拉整 resolve

**audit 论断**（examples-review.md §230-265）：

> Cargo 走 path dep `vendor/sha2/sha2/Cargo.toml`，向上找到 `vendor/sha2/Cargo.toml` 的 `[workspace]`——会拉**全部 25 个 crate** 到 dependency resolution。
> ...
> 1. **cargo invocation 时间膨胀**（每次 build/check 都做 25-crate resolve）
> 2. **`hax-cli` 这类对工作目录敏感的工具可能误把 25 crate 当 input**
> 3. **未来 sha2 monorepo 加新 crate / patch 改变会 silent 改变测试行为**
>
> ...已知 charon-mono `0019699` commit 提到过 arm64 path bug——可能与此相关。

**独立验证（disprove-first）**：

1. **vendor/sha2 确实是 25-crate workspace**：`head -40 vendor/sha2/Cargo.toml` 显示 `members = [25 个 crate 名]` + 3 个 `[patch.crates-io]`（sha1 / sha3 / whirlpool）。audit 事实陈述正确。

2. **但 path dep 不会拉整 workspace**——cargo 实测：

   ```bash
   cd examples/industrial/sha2/sha256-digest
   cargo metadata --format-version=1 | jq '.packages | length'
   # 输出: 11
   ```

   完整 11 个 package 列表：
   ```
   block-buffer / cfg-if / const-oid / cpufeatures / crypto-common /
   digest / hybrid-array / libc / sha2 / sha256_digest / typenum
   ```

   **没有任何 sha2 workspace 的其他 24 个 member（ascon-hash256 / bash-hash / blake2 / md5 ...）在 resolve graph 中**——只有真正被 `use sha2` 引入的 sha2 + 其传递依赖。

3. **为什么 audit 推论错了**？

   cargo 行为：当 path dep 指向一个 workspace member crate，cargo **只把那个 member 当单 crate**加进 dependent 的 resolve graph——workspace `[members]` 字段**不会自动**把所有 member 都拉进来。workspace 的作用是"统一 lockfile / 统一编译目录 / 统一 patch"，而**不是**"声明依赖关系"。

   下游 crate（如 sha256_digest）的依赖关系**只看 `[dependencies]` + path 指向的目标 crate 自己的 `[dependencies]`**——不读 workspace root 的 `[members]`。

4. **patches 副作用**——audit 自己承认（§257-258）："这些 patches **仅在 workspace 内部生效**——但因为隔离副本不是 workspace root，patches 不会传到 industrial crate。"

   这条 audit 已经否定了 patches 对 industrial 的实际影响。但 audit 接下来仍写"25-crate workspace resolve 仍然发生"——上一段刚自我否定，下一段又反过来推断"性能膨胀 / hax-cli 误读 / silent 改变"。**自相矛盾**。

5. **与 charon arm64 path bug 关联**——commit 4bdf08f message：
   > P8: Charon integration with --lib workaround for arm64 path bug
   > - both use 'charon cargo -- --lib --target aarch64-apple-darwin'
   > - **--lib mode bypasses charon's --bin rlib path bug** on Apple Silicon

   是 charon 自身的 **`--bin rlib 路径处理 bug`**——与 vendor/sha2 monorepo / workspace resolve **无任何关联**。audit 写"可能与此相关"是无证据的联想。

**结论**：**事实陈述错误**——cargo metadata 实测仅 11 crate；patches 不生效（audit 自己承认）；charon arm64 bug 无关。"高严重度 / D2 高优先级"评级**不成立**，应降级到"低 / 注释级提醒"（可选优化：vendor/sha2 隔离副本路径策略可考虑改 isolation flag，但不属功能问题）。

---

### 1.3 §3.3-#15：hax-limit/unsafe-block 用 nightly-only `u32::unchecked_add`

**audit 论断**（examples-review.md §153-159）：

> `pub fn hax_limit_unsafe_block() -> u32 { let a = 100u32; let b = 200u32; unsafe { a.unchecked_add(b) } }`——entry 体含 `unsafe` 块，触发 hax HAX0000 UnsafeBlock。
>
> 注意：`u32::unchecked_add` 是 **nightly-only intrinsic**——stable Rust 不可调用。这意味着 `cargo check` 在 stable toolchain 上**会 fail**——破坏 cargo-check 基线（cargo-check 是 framework 的 baseline tool）。

**独立验证（disprove-first）**：

1. **源码确实用 `unchecked_add`**——`examples/hax-limit/unsafe-block/src/lib.rs:16`：
   ```rust
   unsafe { a.unchecked_add(b) }
   ```
   audit 引用正确。

2. **active toolchain 是 stable**：
   ```
   $ rustup show active-toolchain
   stable-aarch64-apple-darwin (default)
   $ rustc --version
   rustc 1.95.0 (59807616e 2026-04-14)
   ```
   项目根目录无 `rust-toolchain` / `rust-toolchain.toml` pin（`find -maxdepth 3 -name "rust-toolchain*"` 仅命中 `.tmp/creusot-src/rust-toolchain`，那是 creusot 内部）——runner 用系统默认 stable。

3. **实测 stable 是否接受 `unchecked_add`**：

   独立最小 repro（`/tmp/test-unchecked-add/`）：
   ```rust
   pub fn test() -> u32 {
       let a = 100u32;
       let b = 200u32;
       unsafe { a.unchecked_add(b) }
   }
   ```
   `cargo check` 输出：
   ```
   Checking test-uca v0.1.0 (/private/tmp/test-unchecked-add)
   Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.20s
   ```
   **stable 1.95.0 完全接受**——`unchecked_add` 已稳定化（Rust 标准库 unchecked_* family 在 1.79+ stable 后陆续稳定）。

4. **runs 实测验证**——`run-1778226613-5282/results.json` 对 `hax-limit/unsafe-block/hax_limit_unsafe_block` × `cargo-check`：
   ```json
   { "status": "SUCCESS", "exit_code": 0, "duration_ms": 171,
     "raw_stderr": "raw/cargo-check/hax-limit__unsafe-block__hax_limit_unsafe_block.stderr" }
   ```
   stderr 内容：
   ```
   Checking hax-limit-unsafe-block v0.1.0 (...)
   Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.13s
   ```
   **cargo-check 基线在该 entry 实测 SUCCESS**，不存在 audit 推测的"破基线"现象。

**结论**：**事实判断错误**——audit 关于 "`unchecked_add` 是 nightly-only intrinsic" 的认知**过时**（stable 已稳定）。D3 决策点"将 `unchecked_add` 替换为 stable unsafe 操作"**没有必要性**——cargo-check 基线未破。

附注：audit 的认知盲点可能源于历史信息——`unchecked_add` 历史上确实是 nightly-only（pre-1.79）。但 2026-05 时点的 stable 1.95.0 早已稳定。**审查时应实测当前 toolchain 而非依赖历史知识**——这是 audit-3 方法论失分点。

---

## §2 决策点抽样验证

### 2.1 §3.5-#24：abs/my_abs 与 struct-norm/manhattan_of 用 `-x` 对 i32::MIN 有理论"UB"

**audit 论断**（examples-review.md §297-304）：

> `abs/my_abs`：`if x < 0 { -x }`——对 `i32::MIN` 是 **overflow UB（debug build panic / release silent wrap）**。
> `struct-norm/manhattan_of` → `manhattan`：同上对 self.x / self.y < 0 取负。

**独立验证**：

1. **源码确实用 `-x`**：
   - `examples/runnable/abs/src/lib.rs:5`：`if x < 0 { -x } else { x }`
   - `examples/runnable/struct-norm/src/lib.rs:10-11`：`if self.x < 0 { -self.x } else { self.x }`

   inputs 实测：
   - `abs/hirusttest.toml:4`：`inputs = [[0], [7], [-7], [42], [-100]]`
   - 不含 `i32::MIN` (-2147483648)
   - audit 关于"inputs 不触发"陈述正确。

2. **关键术语错误：`-x` 对 i32::MIN 不是 UB**——在 safe Rust 中 `-x` 是定义良好操作：

   实测（`/tmp/test_neg2.rs`）：
   ```rust
   fn my_abs(x: i32) -> i32 { if x < 0 { -x } else { x } }
   fn main() {
       let x: i32 = std::env::args().count() as i32 * -2147483648;
       println!("my_abs: {}", my_abs(x));  // 输入 i32::MIN
   }
   ```

   - Release（`rustc -O`）输出：`my_abs: -2147483648`（**wrap, 无 panic, 无 UB**）
   - Debug（`rustc`）输出：`thread 'main' panicked: attempt to negate with overflow`（**panic, 不是 UB**）

   Rust 语义：safe Rust 中算术 overflow 是**定义良好行为**——debug 模式 panic（受 `overflow-checks` 控制），release 模式 two's-complement wrap。**UB** 仅在 `unsafe { unchecked_neg / unchecked_* }` 系列发生（audit 在 §3.3-#15 的 `unchecked_add` 才是真 UB primitive）。

3. **audit 标"理论 UB"语义错**——应改为"理论 overflow panic / wrap"。但 audit 的"corpus 缺陷"标注本身（"未来扩展 inputs 时一不小心触发"）合理——只是不应叫 UB。

**结论**：corpus 工程问题成立（D4 决策点合理），**但术语精度失分**——"UB"是 audit 自己用错的概念。建议改为 "理论 overflow"。

---

### 2.2 §3.4-#19：vendor submodule 缺 commit/branch pin

**audit 论断**（examples-review.md §210-228）：

> `.gitmodules` 无 `branch` / `tag` 字段——submodule 实际 commit 隐式锁在 `git ls-tree HEAD vendor/<name>` 的 SHA 上。这功能上是锁定的，但 `principles.md` §三 时效性原则要求"锁定 commit hash"——industrial 三件套的 vendor 锁定信号弱。

**独立验证**：

1. **`.gitmodules` 实际内容**：
   ```
   [submodule "vendor/sha2"]
       path = vendor/sha2
       url = https://github.com/RustCrypto/hashes.git
   ```
   确实无 `branch`。

2. **但"pin 信号弱"是 git 行为误解**——`git submodule status` 实测：
   ```
    85f03b5...  vendor/rsa         (v0.6.0-149-g85f03b5)
    ffe0939...  vendor/sha2        (sha2-v0.11.0)
    6f4a732...  vendor/x509-parser (x509-parser-0.16.0)
   ```

   git submodule 的 **superproject 默认行为是 pin 到具体 SHA**——superproject 的 tree object 把每个 submodule 路径绑定到具体 commit hash，clone superproject 时 git submodule update 拿到的就是那个 SHA。`.gitmodules` 的 `branch` 字段**仅影响** `git submodule update --remote`（主动跟踪 branch HEAD 的可选行为），**不影响默认 pin**。

3. **principles.md §76 实际要求**："每工具的 README 锁定 commit hash / brew tap version / nightly toolchain pin"——这条是**工具集成层**的要求（次要模块 3，`tools/<name>/README.md`），不是 examples 的 vendor submodule 层。

**结论**：audit 论断**部分错误**——git submodule 已是硬 pin（不依赖 `.gitmodules` 的 branch 字段）；principles.md §76 的"commit hash 锁"原本就指 tools README，不是 vendor `.gitmodules`。D9 决策点"加 branch / tag 字段"是**可选优化**（便于 `git submodule update --remote` 跟踪上游 tag），不是合规要求。

---

### 2.3 §3.6-#36：detailed-design.md 未明示 `target_crate_name` 的 sanitize 行为

**audit 论断**（examples-review.md §371-384）：

> `detailed-design.md §一` Tera 变量表里说 `target_crate_name`：`来自样例 Cargo.toml 的 [package].name`——**没明示 sanitize 行为**。
> 但 `runner/src/exec.rs:98`：`let crate_ident = example.crate_name.replace('-', "_");`
> 实际注入的是 `crate_name.replace('-', "_")`。**文档/实现 mismatch**。

**独立验证**：

1. **runner/src/exec.rs:94-99 实际代码 + 注释**：
   ```rust
   // cargo's standard convention: a package name like "bool-bitwise-op"
   // becomes the Rust crate identifier "bool_bitwise_op". `target_crate_name`
   // is consumed in Rust path position by every harness template, so we
   // must hand over the identifier form, not the cargo manifest name.
   let crate_ident = example.crate_name.replace('-', "_");
   ctx.insert("target_crate_name", &crate_ident);
   ```

   **关键事实**：`replace('-', '_')` 不是任意 sanitize hack——是 **cargo 标准约定**。任何 Rust 开发者都知道 `cargo new my-crate` 创建的 lib 在 `use` 语句里写 `use my_crate`。这是 cargo 内置规则，不是本项目自创。

2. **detailed-design.md 是否需要单独说明**？

   - 严格派：模板作者第一次写 `{{ target_crate_name }}` 不知道是 ident 形态时确实可能踩坑——补一句注释无害。
   - 简洁派：cargo 约定是 Rust 工具链公共知识，文档不需要重复（同理 detailed-design.md 也没说 `Cargo.toml` 是 TOML 格式）。

   两种判断都有道理，但 audit 标"中严重度 / 决策点 D10"略偏紧——这是**文档润色级**，不是"实现与文档语义偏离"。

**结论**：audit 论断**事实陈述正确**但**严重度评级偏高**——属润色级。runner 的 `replace('-', '_')` 是 cargo 约定显式实现，注释里已说明。

---

### 2.4 §3.6-#34：entry_fn `nop_via_asm` 跨 feature 重复

**audit 论断**（examples-review.md §362-365）：

> `charon-limit/inline-asm/nop_via_asm` 和 `creusot-limit/inline-asm-basic/nop_via_asm`。ID 三段唯一，sanitize 后 slug 唯一——**不破任何原则**。但 entry_fn 跨 entry 重复在 corpus 内部跟踪时增加心智负担。

**独立验证**：

```bash
grep "entries = " examples/charon-limit/inline-asm/hirusttest.toml \
                  examples/creusot-limit/inline-asm-basic/hirusttest.toml
# charon-limit/inline-asm:        entries = ["nop_via_asm"]
# creusot-limit/inline-asm-basic: entries = ["nop_via_asm"]
```

事实陈述正确。决策点级别 / 不破原则——OK。

---

### 2.5 §3.3-#16 + §3.6-#35：命名风格分裂 kebab vs underscore

**audit 论断**：~37 kebab vs ~120 underscore。

**独立验证**：

```bash
$ find examples -name "Cargo.toml" | xargs grep -l '^name = ".*-.*"' | wc -l
45
$ find examples -name "Cargo.toml" -exec grep -l '^name = ".*_.*"' {} \; | wc -l
112
```

实测 **45 kebab vs 112 underscore**（audit 写 37/120 是粗略估计，量级一致）。kebab 集中在 creusot-limit / prusti-limit / hax-limit / miri-limit 等 limit 类目。事实陈述正确——**风格分裂确存在**。

但 runner exec.rs:98 已统一处理（`replace('-', '_')`）——工程上无破坏。属命名约定决策点。

---

## §3 跨 entry 共性问题抽样

### 3.1 §3.6-#32：双轨 schema 在 corpus 层 zero 实例

audit §342-346：
> `find examples -name '.hirusttest' -type d` 返回空。意味着 framework 的"双轨"是**理论上的**——只有单文件轨被实际使用。

**独立验证**：实测同 audit。`find examples -name '.hirusttest' -type d` 返回空。

**派生评价**：
- audit §457 结论："双轨 schema 在 corpus 层从未被实例化...framework 实现了双轨 discover 逻辑（runner/src/discover.rs:104-167），但 corpus 上没人用——这条信号说明设计可能 over-engineering 了，或者 corpus 没消化设计意图"
- 这是**洞察性发现**——确实指向设计与实操脱节。但 audit 自己给的两条解释里"或者 corpus 没消化"对应 #18 industrial 应迁目录轨——一致。

**结论**：洞察成立。

---

### 3.2 §3.6-#31：runner discover 未做 schema 类型校验

audit §336-340：
> v1 只读 inputs/expected 两字段，未做严格类型检查——下游 wrapper（hax-lean-eval）自筛时才检查。
>
> 潜在 footgun：用户写一个 `pub fn foo(a: Vec<i32>) -> i32`，hirusttest.toml 加 `[runnable.foo] inputs = [[[1,2,3]]] expected = [6]`——runner discover 不报错，到 hax-lean-eval wrapper 时才 fail。

**独立验证**：未读 runner/src/discover.rs，但按 audit §80 引用 detailed-design.md "Schema 向后兼容性：`runner/src/discover.rs` 的 `HirusttestToml` 结构未设 `deny_unknown_fields`"——consistent 描述。属合理的 v1 砍项（discover 不做强校验，由下游消费者校验）。

**派生评价**：audit 标"中"——属未来 v2 增强方向，不破当前原则。OK 但严重度偏紧。

---

## §4 §3.4-#21 / #22 余下中低问题快速核

### 4.1 §3.4-#21 industrial Cargo.toml 用冗余 `[lib] path = "src/lib.rs"`

实测 `examples/industrial/sha2/sha256-digest/Cargo.toml:7`：
```toml
[lib]
path = "src/lib.rs"
```
冗余但无害——cargo 默认就是 `src/lib.rs`。**事实正确，决策点 D14 合理**——清理 vs 保留二选。

### 4.2 §3.4-#22 industrial 用 OsRng / unwrap silent 路径

实测 `examples/industrial/rsa/rsa-pkcs8/src/lib.rs:27, 33-37`：audit 引用正确。OsRng 在受限测试环境的潜在 fail 是真实可重现性风险——但属低严重度（industrial 三件套本就是模拟真实 codebase，audit 自承"不破原则"）。

---

## §5 audit-3 §3.3-#12 / §3.5-#23 正向陈述（合规结论）核验

### 5.1 §3.3-#12 所有 limit 类目 zero verifier 属性

audit grep：
```
grep -rE '^[[:space:]]*#\[kani::|^[[:space:]]*#\[creusot::|^[[:space:]]*verus!|^[[:space:]]*#\[verifier::|^[[:space:]]*#\[prusti::' examples/
```
audit 称"零命中"。

**未重跑此 grep**（可信赖 audit-3 自己 grep），但与 audit-1 / audit-2 同类 grep 跨验证一致——原则 A "plain Rust" 在 corpus 层确实 100% 合规。

### 5.2 §3.5-#23 runnable corpus 全 15 entry 合规禁止集

audit grep 列了 5 类禁止集，全 0 命中。**未重跑**，但抽样核验：

`examples/runnable/abs/src/lib.rs`、`fib/src/lib.rs`、`fact/src/lib.rs`——确实 plain Rust，无 collection / unsafe / IO / float。一致。

---

## §6 总评

### 验证条目分布

| 严重度 | 验证条目数 |
|---|---|
| 高 #15 / #18 / #20 | 3（核心要求） |
| 决策点抽样 D4 / D9 / D10 / D14 | 4 |
| 共性问题 #31 / #32 / #34 / #35 / #36 / #21 / #22 | 7 |
| **合计** | **14 条独立核验** |

### 3 高严重度真伪

| # | audit 评级 | 验证结论 |
|---|---|---|
| §3.4-#18 industrial 违反双轨 | **高 + D1 高优** | **核心成立**（精神派），但表述偏紧（"宪法级违反"过强）；严格字面派可挑战（industrial 非 vendor 真本体） |
| §3.4-#20 sha2 monorepo 25-crate resolve | **高 + D2 高优** | **错误**——`cargo metadata` 实测仅 11 crate，patches 不生效（audit 自承），charon arm64 bug 无关；应降低到注释级或撤回 |
| §3.3-#15 hax-limit unchecked_add nightly-only | **中 + D3 中优** | **错误**——stable rustc 1.95.0 已稳定 `unchecked_add`；runs 实测 cargo-check SUCCESS；D3 无修复必要 |

### audit-3 整体质量评级：**B**

**优点**：
- 方法论扎实：grep 全 corpus 验证原则 A 合规 / 禁止集合规
- 洞察性发现：双轨 schema corpus zero 实例（§3.6-#32）— 指出设计与实操脱节
- 决策点 / 非决策点判别清晰（§4 二分表）
- 大量细节问题（命名风格分裂 / 跨 entry 同源 / async fn schema 边角）扫描全面

**失分**：
1. **3 高严重度有 2 个事实判断错（#20 monorepo / #15 nightly-only）**——这是审查方法学失分：审查者依赖知识 / 推论，未做现场实测验证。
2. **术语精度偶失**："UB"误用（§3.5-#24 应叫"overflow"）/ "submodule pin 信号弱"误解 git 行为
3. **严重度评级偶偏紧**：industrial 违反双轨虽该修，但"宪法级违反"过强；D10 文档 sanitize 是润色级标"中"偏高
4. **D2 / D3 整段建议基于错误事实**——若实施会做无意义改动（D2 加 isolation hint / D3 替换 `unchecked_add` 为 stable unsafe）

**核心问题**（§3.4-#18 industrial 双轨）成立——保留 D1 作为高优先级 fix-forward 任务即可；其余 D2 / D3 应根据本 counter 撤回或大幅降级。

### 反馈给 audit-3 的方法学改进建议

- 涉及"该语法/API 不在 stable"等版本性判断时，**必须 `cargo check` 现场实测**——不能依赖训练时知识
- 涉及 cargo dep resolution 行为时，**必须 `cargo metadata` 现场实测**——不能依赖 Cargo.toml 字面推断
- "UB" / "pin" / "intrinsic" 等术语应区分严格语义与口语用法
