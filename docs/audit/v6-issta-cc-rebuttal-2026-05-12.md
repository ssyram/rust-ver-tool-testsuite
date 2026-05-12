# v6 ISSTA reviewer C / E findings — cc rebuttal（2026-05-12）

> 6 路 c challenger 跑完后，按宪法 §八 disprove-first 协议，对 reviewer C (Internal Validity) 和 reviewer E (Conclusion Validity) 的部分 finding 用宪法 + 工具 nature 反驳，作为 cc 路记录。其他 axis（A / B / D / F）的 finding 大多 valid，直接进入修订（P38 + P39）。

---

## 一、Reviewer E "Multi-run validation 缺失" — 反驳 ✗

**c finding 原文**：

> 单次 run（仅 ror N=7 是 oracle 内反非确定性，非 run-to-run 重复）；其他 SMT/BMC 工具 timeout 边界未估计 flip rate。

**反驳论证**：

reviewer E 把 "empirical study 默认要 multi-run" 当作 universal truth 套用——但**没问被测对象的 nature**。

1. **20 个工具都是确定性程序**：cargo / charon / aeneas / kani / verus / prusti / creusot / verifast / miri / kmir / soteria —— 给同样 input + 同样 environment + 同样 toolchain，输出**必定 byte-identical**。没有 LLM，没有 RNG，没有非确定性 scheduling。
2. **timeout flip 不会发生**：项目设计成切到前端层（`--only-codegen` / `--no-verify` / `PRUSTI_NO_VERIFY=false+PRINT_HASH` / 等），**不进入 BMC/SMT 求解层** —— 没有"求解超时随机化"风险。
3. **唯一已知非确定性**：`rocq-of-rust` 翻译路径（`thread_local!` 类 entry）——项目**已经用 N=7 attempts AND-reduce 设计上处理**（详见 `tools/rocq-of-rust/rocq-of-rust-wrapper.sh` header + `docs/fixes/ror-gate6-fix-2026-05-11.md`）。
4. **run-to-run 重复不会有任何信息增益**：对确定性程序重复执行 = 重新计算相同结果 = 纯浪费算力。

**判定**：reviewer E finding **不成立**（误把 empirical study 通用方法学套用到确定性 build/translation pipeline）。

**注**：reviewer E 的 minor finding（Wilson CI / McNemar）成立——通过率的统计精度提示是合理学术补强，会在 P39 落地。

---

## 二、Reviewer C "Bug detect = SUCCESS 跨工具非对称" — 反驳 ✗

**c finding 原文**：

> 候选 3：bug-detect=SUCCESS 仅 MIRI/soteria 触发；其他演绎验证器（kani/verus/prusti/creusot）项目设计成跑前端层，永远不进入 bug detect 路径——这是公平的吗？

**反驳论证**：

reviewer C 看到 surface-level 不对称就跳到"不公平"——但**没读宪法 §六 前端测量 + architecture §一 P35 派生**的完整精神。

**核心事实**：

1. **MIRI 是 abstract interpreter** —— 工作 nature 是把 Rust 当解释执行的代码完整跑一遍。**没有"前端 vs 求解层"切点可切**。给一段代码 → 跑完无 issue ∨ 跑完 + 检测出 UB → 都是 MIRI 按设计意图工作。**bug-detect 不是它的"额外能力"，是它的有效输出形态之一**。
2. **soteria 是 symbolic execution** —— 同理，符号执行整段走完是其本性。
3. **verifier-style 工具（kani / verus / prusti / creusot）**按宪法 §六 前端测量 **deliberate 切到前端层**——这是项目主动让它们停在更前面（避免 BMC/SMT 求解时间和资源消耗，让所有工具在 feature-coverage layer 可比）。**它们本来就不在 bug-detect 路径**，不是被歧视。
4. **verifast** 跑完整 verification 但需要用户 spec 注解；本 corpus 0 个 entry 有 `//@` 注解，verifast 不进入真 verification 路径——也不进入 bug-detect 路径。

**每个工具按它能给出的有效输出形态评 SUCCESS——对称**。

**修宪行动**（P38 已落地，作为反驳的固化）：

- `principles.md` §六"前端测量"段加"工具输出形态对称性"明示
- `architecture.md` §一 "bug detect 归 SUCCESS" 段加"对称性论证（防 reviewer 误读）"——明示三类工具的输出 nature + 切点选择的 deliberate design

**判定**：reviewer C finding **不成立**（surface-level reading 没识别工具 nature 异同）。

**注**：reviewer C 的 candidate 4（`[env] RUSTFLAGS=--cap-lints=warn` instrumentation）+ candidate 5（`extra_cargo_deps` creusot）+ candidate 6（跳过 `Cargo.lock`）成立，会在 P39 加 Threats to Validity 章节诚实声明。

---

## 三、其他 axis (A / B / D / F) findings 直接进入修订

c 路其他 axis findings 大多 valid，反驳空间小，**直接进入修订**而非走完整 cc：

- **Axis A 可重现性**：LICENSE / submodule init / .env.example / results.json 路径 / archive DOI ——P38 工程修
- **Axis B Construct Validity**：强措辞同步降级（charon×2 + cargo-check）/ "工具菜"口语整理 / "前端测量"边界声明——P38 + P39
- **Axis D External Validity**：§3 主表重设计（按集团排序 + 测量边界列 + wrapper 状态列）——P39
- **Axis F 术语 + 引用**：cite list / glossary / Wilson CI / paper draft——P39

详 P38/P39 commit。

---

## 四、c+cc 协议实证（再次）

本次 audit 6 c agents 抛 30+ Major findings；cc 反驳 C + E 两个 reviewer 各 1 个核心 finding 不成立（precision 损失约 7%），其余 valid 进入修订。

c+cc 协议在跨学科评审（empirical study 教科书方法 vs 项目 deliberate design）尤其重要——单走 c 会过度套用方法学到 conceptually 不匹配的场景。
