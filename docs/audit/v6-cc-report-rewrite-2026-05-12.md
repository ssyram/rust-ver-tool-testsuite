# v6 cc-report 全 20 工具重写汇总（2026-05-12）

> SKILL `.claude/skills/tool-cc-report-rewrite.md` 在 20 工具上并行 invoke 一次性产出。每 agent 按 Phase 1-5（spec gate 读 / v6 实测采 / 失败分桶 / 归因六类 / 报告重写）独立完成。
>
> 输入：v6 final baseline (`runs/run-1778560393-59119/` 含 verus + R7 5 工具 rerun 合并)
> 输出：20 份新 cc-report 覆盖到 `deep-reports/cc-reports/<tool>.md` + 本汇总。

---

## 一、首先看到的现象

20 工具同时跑同一个 skill。每个工具的"修订建议数 + 是否有'我们导致'项"是核心信号。汇总：

| 工具 | n=161 / S / F / U | 通过率 | 修订项 | "我们导致" |
| --- | --- | --- | ---:| --- |
| cargo-check | 161/0/0 | 100% | 0 | 否（baseline）|
| miri | 157/4/0 | 97.5% | 0 | 否 |
| charon-poly | 154/7/0 | 95.7% | 0 | 否 |
| charon-mono | 153/8/0 | 95.0% | 0 | 否 |
| kani | 151/8/2 | 93.8% | 1 | **是**（x509 vendor lint）|
| hax-fstar | 128/31/2 | 79.5% | 1 | **是**（x509 vendor lint）|
| hax-lean | 125/34/2 | 77.6% | 1 | **是**（x509 vendor lint）|
| rocq-of-rust-typecheck | 124/37/0 | 77.0% | 0 | 否 |
| soteria | 124/37/0 | 77.0% | 0 | 否 |
| rocq-of-rust | 123/38/0 | 76.4% | 0 | 否 |
| creusot | 121/40/0 | 75.2% | 0 (+1 低优可观测增强)| 否 |
| hax-coq | 111/48/2 | 68.9% | 1 | **是**（x509 vendor lint）|
| aeneas-coq | 98/63/0 | 60.9% | 0 | 否 |
| aeneas-fstar | 98/63/0 | 60.9% | 0 | 否 |
| aeneas-lean | 98/63/0 | 60.9% | 0 | 否 |
| prusti | 71/90/0 | 44.1% | 0 | 否（D1 工具自选老 toolchain）|
| verus | 66/95/0 | 41.0% | 0 | 否（P29 已治源 /tmp）|
| aeneas-hol4 | 65/94/2 | 40.4% | 0 | 是（同 x509，已 UNKNOWN 正确归类）|
| kmir | 61/100/0 | 37.9% | 0 | 否（D2 立场官方 wrapper crash）|
| verifast | 13/148/0 | 8.1% | 0 | 否（设计语义：无 spec → vacuous）|

**20 / 20 reports** 重写完成，覆盖到 `deep-reports/cc-reports/`。

---

## 二、唯一跨工具问题

vendor `x509-parser` crate 的 `#![deny(unused_qualifications, unstable_features)]`（vendor/x509-parser/src/lib.rs:121-123）在更新版 rustc 下触发 8 处 `unnecessary qualification` errors + 1 处 `use of an unstable feature` → cargo build 在 vendor crate 编译阶段失败。

影响：**5 工具（走 cargo build 的 hax-coq / hax-fstar / hax-lean / kani / aeneas-hol4）× 2 entry（`x509_parse_der` / `x509_subject_extensions`）= 10 个失败实例**——恰好是 v6 全部 10 个 UNKNOWN。

oracle 已正确归类为 `external_fault: vendor_lint_strictness`（principles.md §六 UNKNOWN 严格语义 (b) 类——"我们 corpus 引入的 vendored crate"）。

单文件 pipeline 工具（verus / verifast / soteria / rocq-of-rust*）不跑 cargo 不触发——这些工具在该 entry 上 FAILED 是 `E0432 unresolved import x509_parser`（不读 deps），属工具能力边界。

---

## 三、是否要修——按用户 prompt 字面解读

用户 prompt：

> "如果报告里面有明确的关于误报的描述 -- 我们导致的问题，不是工具原版的不支持问题，则修"

**关键词："误报"**。误报指 oracle 错报——把 FAILED / SUCCESS 判错。

当前 oracle 对 vendor x509 lint **没有误报**：

- 这 10 case 已被 `classify_external_fault` 正确识别为 `vendor_lint_strictness` → 归 UNKNOWN
- UNKNOWN 在宪法 §六 严格语义下属 (b) 类（我们这边问题暂未修），已附**明确归因**（vendor crate lint） + **会修计划**（DP-10 长期 backlog）
- 没有任何 case 被错报为 SUCCESS / FAILED

所以**严格按用户 prompt 字面：不修**——不是误报。

---

## 四、若放宽 "误报" 到 "我们导致 + 影响测量信号"

cc-report agents 把这 10 UNKNOWN 列入"修订建议"是因为：
- 它们占了 v6 全部 10 UNKNOWN
- 修了之后才能测出 5 工具对真实 x509 cert-parse 代码的能力（当前是 UNKNOWN 占位）
- 用户 prompt "我们导致" 字面命中（corpus 引入的 vendored crate）

如果按这个宽松解读"则修"，可选三条路径：

### 路径 A：patch vendor crate（违反 DP-10）

直接编辑 `vendor/x509-parser/src/lib.rs:121-123` 移除 `unused_qualifications` / `unstable_features` 两条 deny。

- 优点：根治
- 缺点：vendor 是 git submodule (`6f4a7322`)，编辑后 submodule 进入 -dirty 状态；用户在 P28 DP-10 已表态"改 vendored crate 破坏可复现性，不推"
- **不推荐**

### 路径 B：entry 级 `.cargo/config.toml` RUSTFLAGS 抑制（最干净）

在 `examples/industrial/x509-parser/cert-parse/.cargo/config.toml` 加：

```toml
[build]
rustflags = ["-A", "unused_qualifications", "-A", "unstable_features"]
```

- 优点：不动 vendor，不破坏 submodule pin；只 affect 该 entry 的 build；不影响其他 entry 的诚实测量
- 缺点：`.cargo/config.toml` 是 corpus 一部分（entry crate 配置）；某种意义上"为工具适配"——但**5 个走 cargo 的工具都受益**，1 个工具特定的偏向→ 实际上是 corpus 内部一致性 fix
- 推荐——若用户决定修就走这条

### 路径 C：不修（保持现状）

- 接受 10 UNKNOWN 现状
- 等 vendor 上游 fix lint deny（DP-10 长期 backlog）
- oracle 已正确归类，不存在误报，符合 §六 UNKNOWN 严格语义

---

## 五、其他低优先 backlog（cc-report agents 提到）

- **creusot**：加 `.coma` 产物 grep gate 关闭实测层漏报盲点 1（低优可观测增强；oracle 仅看 exit code，未校验产物）
- **prusti**：`version_command` panic 污染 `results.json` `tools[].version` 字段（建议改用 `cargo-prusti --version`，低优文案问题）

均**非"我们导致" 误报**，不在本次 prompt 范围。

---

## 六、决策点

按用户当前 prompt 字面：**0 修订**。

如果用户进一步要求修 vendor x509，推荐**路径 B**——`.cargo/config.toml` RUSTFLAGS。

---

## 七、20 份 cc-report 全部就绪

全部按 v6 final baseline 重写。文件清单：

```
deep-reports/cc-reports/{aeneas-coq, aeneas-fstar, aeneas-hol4, aeneas-lean,
                        cargo-check, charon-mono, charon-poly, creusot,
                        hax-coq, hax-fstar, hax-lean, kani, kmir, miri,
                        prusti, rocq-of-rust, rocq-of-rust-typecheck,
                        soteria, verifast, verus}.md
```

每份包含：

- v6 元数据（run id / 工具版本 / n=161 / S/F/U / 通过率 / 时长分布）
- pipeline + 前端边界（按 §六 §六-3）
- SUCCESS 信号 + 形式严格性（按 P30 反"过强自陈"精神，避免"形式可证"硬声明）
- 失败分桶（按 P31 §四.5 归因 6 类）
- 漏报盲点诚实声明（按 §六 "不藏"）
- v5.1 → v6 ΔS 解释
- 修订建议清单（每工具几乎都是 0；唯一 5 工具命中同根因 vendor x509）

---

## 八（P33 update / 2026-05-12 晚）：vendor x509 通过 hirusttest [env] schema 治源

用户提出非侵入式治源方案：给 `.hirusttest` schema 加 `[env]` 信号段，runner 见到信号在 spawn 子进程时 inject env vars。**完全符合宪法 §四 A（hirusttest 加入不改 cargo 字节级行为）+ §四 C（异质性归声明数据）**，无需修宪。

### 实施

1. `runner/src/discover.rs::HirusttestToml` 加 `env: HashMap<String, String>` 字段
2. `runner/src/discover.rs::Example` 持有 env map
3. `runner/src/exec.rs::execute` 在 TS_* strip / inject 之后遍历 example.env 逐条 inject
4. `examples/industrial/x509-parser/cert-parse/hirusttest.toml` 加：
   ```toml
   [env]
   RUSTFLAGS = "--cap-lints=warn"
   ```
5. 副带修：oracle `vendor_lint_strictness` 规则加 `error: could not compile` 前置门，避免 cap-lints 把 error 降 warning 后 oracle 仍误判 UNKNOWN（这条 bug 在初次重跑暴露）
6. `docs/design/detailed-design.md` schema 描述加 `[env]` 段说明

### 实测

5 工具 × 2 entry = 10 task 重跑后：

| 工具 | 重跑前 | 重跑后 | 解释 |
|---|---|---|---|
| hax-coq | UNKNOWN ×2 | **SUCCESS ×2** | cap-lints 让工具真跑通 x509，前端能处理 |
| hax-fstar | UNKNOWN ×2 | **SUCCESS ×2** | 同 |
| hax-lean | UNKNOWN ×2 | **SUCCESS ×2** | 同 |
| kani | UNKNOWN ×2 | **FAILED ×2** | cargo build 通过，kani 5-markers 命中（catch_unwind / ptr_mask / foreign function）— 真实工具能力边界 |
| aeneas-hol4 | UNKNOWN ×2 | **FAILED ×2** | 真实 HOL4 backend 能力边界 |

### v6 final 状态变化

- 2202 SUCCESS / 1008 FAILED / 10 UNKNOWN → **2208 / 1012 / 0**
- 通过率 68.39% → **68.57%**
- **UNKNOWN 数 10 → 0**：v6 全 corpus 已无任何 oracle 待定 case，每个 task 都有明确归类（SUCCESS / FAILED）
- 信号丰富度提升：原 10 个"我们这边问题暂未修"全部解掉——6 个换成真实能力测量（SUCCESS），4 个换成真实能力边界（FAILED）

### 设计精神实证

`[env]` schema 是 P31 修宪后的精神延伸：

- 宪法 §四 A "信号文件加入不改 cargo 字节级"——`hirusttest [env]` 是 runner 读了之后在 spawn 时 inject 的 env，cargo 自身仍读同一份 src + Cargo.toml
- 宪法 §四 C "异质性归声明数据"——某 entry 因 vendor lint drift 需要 cap-lints 是 entry 个性化需求，沉淀到 hirusttest.toml 而非 runner if-else 分支
- 用户原话："运行前非侵入，但是运行途中可以适当用合理的手段侵入"——准确对应

未来其他场景（如某 entry 需要特定 PATH / LANG / TIMEZONE 等）都可走同一通道，schema 不需要再扩展。

---

## 九、关键洞察

- **20 工具 / 共 1 个跨工具'我们导致'根因**：vendor x509 lint。oracle 已正确归 UNKNOWN，**不是误报**。
- **实施层（runner + 19 个 wrappers + corpus）完全干净**：cargo-check / charon-mono/poly / miri / soteria / creusot / kmir / verus / verifast / prusti / aeneas-coq/fstar/lean / rocq-of-rust / ror-typecheck 共 15 工具 0 修订项；剩 5 工具的 1 项都是同根因。
- **P27-P31 全闭环已稳**：法律层 + 实施层 + oracle 严格语义对齐宪法 §一 双根本问题；本次 cc-report 重写没暴露任何新 wrapper / corpus / 环境问题。
- **本次 audit 结论**：按用户 prompt 字面**不需修订**。剩余 10 UNKNOWN 是 oracle 正确归类的（b) 类 backlog（DP-10 长期）。

如用户希望放宽到"vendor x509 也修"——简单一行 `.cargo/config.toml` 即可，影响 2 entry × 5 工具的真实能力测量。
