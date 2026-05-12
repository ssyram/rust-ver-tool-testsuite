# v6 漏报 counter-challenge

> R6-cc：基于 c 路报告 `audit-v6-c-false-negative-2026-05-12.md`，对 4 个漏报候选做 disprove-first counter。
>
> 数据：`runs/run-1778560393-59119/`。只观察、不修改。

---

## 1. 候选逐条 counter

### 候选 1：aeneas-coq/fstar/lean × `aeneas-limit/mutually-recursive-traits/trigger_mutually_recursive_traits`

**c 路理由**：aeneas stdout 含三条 Warn：
- `Aeneas cannot handle such types today, and the generated code will likely be incorrect`
- `Mutually recursive trait declarations are not supported; ... model will not type-check`
- `Mutually recursive trait implementations are not supported; ... model will not type-check`

aeneas exit 0；wrapper SUCCESS。建议 wrapper grep Warn 字面。

**counter 论证**：

1. **论点 1：Warn vs Error 是 aeneas 设计选择，wrapper 是否应该 override？** 反驳：宪法 §六 Oracle 责任明确"工具自陈我没全干完必须被尊重"。`Aeneas cannot handle` + `model will not type-check` 是工具最强等级的自陈——明示前端产物形式不可用。这不是模糊提示，是确定性陈述。

2. **论点 2：是否属"求解层假设"边界（类似 kani 单线程语义）？** 反驳：aeneas 是翻译工具，无求解层。`will not type-check` 直接指向产物本身的形式正确性——属前端 partial，非求解层假设。

3. **论点 3：aeneas-limit/ corpus 本就是"已知限制反例"，oracle 抓不抓不影响数据解释？** 反驳：corpus 分类是分桶不是免责。`aeneas-limit/` 前缀的设计目的恰是"暴露 silent partial"——若 oracle 不抓，corpus 失去工程价值。

**精神性陈述反查**：
- **aeneas-coq README** 第 36 行："形式严格性 — 0 漏报...所有 unsupported 都通过 craise push error_list；aeneas 无 silent emit-stub 路径" + **第 38 行"漏报盲点：无"**——精神性陈述**硬硬硬**，无任何回旋空间
- **实测推翻**：aeneas mutually-recursive trait 走 **Warn 通道**而非 craise/error_list，产物落盘 + exit 0——直接破 README "0 漏报 / 无盲点"自陈
- **宪法 §六 Oracle 不冤枉**："SUCCESS 必须是真 SUCCESS——不允许任何 partial / silent skip / 半翻译"——本案符合 partial 定义

**判定：候选成立 ✓**

**fix 建议**：
- aeneas wrapper（aeneas-coq / fstar / lean / hol4 同一模板）追加 stdout grep：
  ```bash
  if [[ $AENEAS_EXIT -eq 0 ]] && grep -qE "model will not type-check|generated code will likely be incorrect" aeneas_stdout.log; then
      echo "[aeneas-oracle] FAIL: aeneas exit 0 but Warn channel declares product unusable" >&2
      exit 1
  fi
  ```
- README 第 36-38 行"形式可证 0 漏报 / 漏报盲点：无"必须修订——aeneas 同时有 craise + Warn 两条通路自陈 partial，README 漏抄 Warn 通路。修订为"双通路：craise→exit 1 + Warn 通道→wrapper grep"。

---

### 候选 2：aeneas-coq / aeneas-fstar × `impl-trait/return-iter/impl_trait_iter`

**c 路理由**：与候选 1 同 Warn：`core::iter::traits::accum::Sum` + `Iterator` 互递归 → `model will not type-check`。

**counter 论证**：

1. **论点 1：是 aeneas 对 core trait 的标准保守翻译，类似"外部 fn 不强制 contract"（creusot 合法设计）？** 反驳：creusot 外部 fn 是**有意未提供 contract**（求解层假设），属求解层。aeneas 这里是**对当前 crate 引用的 core trait 走 mutually-recursive 路径**——是 aeneas 自身翻译逻辑对 `impl Iterator` 返回的处理 partial。前端 partial。
2. **论点 2：corpus 选材问题（不应在通用 corpus 出现）？** 反驳：`impl-trait/return-iter` 是普通 impl Trait 用法，不属 corner case。aeneas 对此普通 Rust 模式产物不可类型检——属真 partial。

**精神性陈述反查**：与候选 1 同（README 第 36-38 行）。aeneas-fstar README 同结构。

**判定：候选成立 ✓**（与候选 1 共享同一 fix）

---

### 候选 3：aeneas-lean × `impl-trait/return-iter/impl_trait_iter`

**c 路理由**：stdout 含：
- `could not find the information for item 'map'. The model defined in the Lean library seems to be missing the corresponding field`
- `could not find the information for item 'sum'. ... seems to be missing the corresponding field`

aeneas exit 0；wrapper SUCCESS。

**counter 论证**：

1. **论点 1：这是 aeneas-lean prelude / builtin model 不全，不是 aeneas 自身能力？** 反驳：aeneas-lean 是 aeneas 项目的一个 backend，prelude/builtin model 是**该 backend 的组成部分**。Iterator::map / sum 是 stable Rust 核心 API，aeneas-lean builtin model 不含等价定义——产物中 `Iterator::map` 调用指向缺失符号，下游 Lean 编译必然挂。前端产物不可用。
2. **论点 2：是否属"对 prelude 演进未跟上"的运营盲点（非翻译能力盲点）？** 反驳：宪法 §六"工具自陈我没全干完必须被尊重"。aeneas-lean 自己说 `seems to be missing`——自陈即足够，不区分运营 vs 翻译盲点。
3. **论点 3：`seems to be missing` 是软陈述（"似乎"），非硬陈述？** 反驳：实质 = aeneas-lean 在产物里 emit 了一个引用缺失符号的 method call。落到产物层是硬效果（Lean 编译挂）。aeneas 用 `seems` 是措辞习惯，不是不确定性。

**精神性陈述反查**：
- **aeneas-lean README** 第 44 行："形式严格性 — 0 漏报...aeneas 不存在 silent emit-stub-but-exit-0 路径" + **第 46 行"漏报盲点：无（依赖 aeneas 上游正确实现 craise）"**
- **实测推翻**：本 entry 即 `silent emit-stub-but-exit-0`——builtin 缺失走 Warn 通道，绕过 craise，exit 0。README "无盲点 / 不存在 silent" 被推翻。

**判定：候选成立 ✓**

**fix 建议**：aeneas-lean wrapper grep stdout：
```bash
if [[ $AENEAS_EXIT -eq 0 ]] && grep -qE "could not find the information for item|seems to be missing the corresponding field" aeneas_stdout.log; then
    exit 1
fi
```
可与候选 1+2 合并为多 pattern OR：`(model will not type-check|generated code will likely be incorrect|seems to be missing the corresponding field|could not find the information for item)`。

aeneas-lean README "漏报盲点：无"修订为"已知盲点：builtin model 不全时走 Warn 通道，wrapper 加 grep 补抓"。

---

### 候选 4：rocq-of-rust-typecheck × `unsafe-ptr/raw-ptr-const/raw_ptr_const_match`

**c 路理由**：tier-0 ror 已通过 D3.1 加 `grep "is not yet supported"` 抓 silent `Pattern::Wild` → 在本 entry FAILED；tier-1 wrapper 漏抄该 grep → SUCCESS。

**counter 论证**：

1. **论点 1：tier-1 是否设计上不必与 tier-0 一致？** 反驳：**ror-typecheck README 第 43 行**明示"Stage 1 与 `tools/rocq-of-rust` **完全等价**（同 sysroot / 同 binary / **同 6 道 grep**）"，**第 136 行**"任一 entry 在本工具 SUCCESS ⇒ 在 `tools/rocq-of-rust` SUCCESS（档 1 ≤ 档 0）"。设计声明硬约束 tier-1 ⊆ tier-0；实测推翻该不等式。
2. **论点 2：tier-0 的 D3.1 grep 是 v6 新加，tier-1 来不及同步——是时间差非设计违反？** 反驳：tier-1 README 已声明"等价 6 道 grep"——若 tier-0 加 gate，tier-1 README 也应更新（或采用 wrapper 调 tier-0 的架构）。当前是 wrapper 漏抄 + README 声明陈旧，两边都需修。
3. **论点 3：tier-1 的 coqc gate 是否足以兜底（coqc typecheck 失败必 exit ≠ 0）？** 反驳：本案是 ror translate 把 `DISGUISED_INT => 1` silently 替换为 `_` wildcard——产物在 Rocq 端 **typecheck 通过**（wildcard match 合法）但**语义改变**。coqc gate 抓不到语义改变；只能靠 stage 1 的 stderr grep。tier-1 README 第 88-90 行"0 漏报...本工具档 1 边界明确只保证 typecheck 通过，不保证语义正确"——但 silent translation 把"测什么"换了，与 README 自陈的边界不符（README 边界是"我翻译完了再 typecheck"，不是"我翻译时 silently 改变产物语义"）。

**精神性陈述反查**：
- README 第 43 行 "Stage 1 完全等价 6 道 grep" 硬声明
- README 第 136 行 "任一 entry 在本工具 SUCCESS ⇒ 在 tools/rocq-of-rust SUCCESS" 硬不等式
- 实测：tier-1 SUCCESS + tier-0 FAILED → 不等式被破

**判定：候选成立 ✓**

**fix 建议**（二选一）：
- (A) **同步追加 grep**：tier-1 wrapper 在 translate 后、coqc gate 前插入与 tier-0 相同的 `grep "is not yet supported"`（最少改动）
- (B) **架构重构**：tier-1 wrapper 改为先调用 tier-0 wrapper（继承所有 gate），通过后再做 coqc。永久消除两边漂移风险。
- 推荐 B；若 B 工程成本高则 A，并在 README CHANGES 写"tier-0 加 gate 时必须同步 tier-1"。

---

## 2. 总结

| 类别 | 数量 | 候选编号 |
|---|---|---|
| **成立** | 4 | 候选 1 / 2 / 3 / 4 |
| **不成立** | 0 | — |
| **部分成立** | 0 | — |

**全部 4 个候选 counter-challenge 后仍站得住**。理由共性：

1. 候选 1+2+3：aeneas 同时有 craise + Warn 两条 partial 通路；aeneas-coq/lean README 自陈"0 漏报 / 无盲点"漏抄 Warn 通道。这是宪法 §六 "Oracle 不冤枉 / 自陈被尊重 / 不藏盲点" 的硬性 violation
2. 候选 4：tier-1 README 自我硬约束 "tier-1 ⊆ tier-0"，实测违反——wrapper 实现漏抄而非设计意图

**操作建议清单**（按工程成本排序）：

1. **aeneas wrapper 多 pattern grep**（同时覆盖候选 1+2+3）：
   - 文件：`tools/aeneas-coq/aeneas-coq-wrapper.sh`、`tools/aeneas-fstar/aeneas-fstar-wrapper.sh`、`tools/aeneas-lean/aeneas-lean-wrapper.sh`、`tools/aeneas-hol4/aeneas-hol4-wrapper.sh`
   - pattern：`(model will not type-check|generated code will likely be incorrect|seems to be missing the corresponding field|could not find the information for item)`
   - 位置：aeneas exit 0 检查之后、`exit 0` 之前

2. **aeneas README 修订**（候选 1+2+3 配套）：
   - 文件：`tools/aeneas-coq/README.md`、`tools/aeneas-fstar/README.md`、`tools/aeneas-lean/README.md`、`tools/aeneas-hol4/README.md`
   - "形式严格性 — 0 漏报"段落补充 Warn 通道双通路说明
   - "漏报盲点：无" 修订为列举已知 Warn 通道字面 + wrapper grep 兜底说明

3. **ror-typecheck wrapper 同步**（候选 4）：
   - 文件：`tools/rocq-of-rust-typecheck/rocq-of-rust-typecheck-wrapper.sh`
   - 推荐 (B) 直接调用 tier-0 wrapper；fallback (A) 同步 `grep "is not yet supported"`
   - README 第 43 行 + 第 136 行声明配套校验或修订

4. **复测**：fix 后重跑 v6 corpus，预期：
   - aeneas-coq: -1 (mutually-recursive) -1 (return-iter) = SUCCESS 数 -2
   - aeneas-fstar: 同上 -2
   - aeneas-lean: -1 (mutually-recursive) -1 (return-iter aeneas-lean 独有 warning) = -2
   - aeneas-hol4: 如有同 warning 也 -X（c 路报告中 hol4 未单独列出，需核）
   - ror-typecheck: -1
   - 总计：8 SUCCESS → FAILED（与 c 路报告"7-8 实例"吻合）

---

## 3. 限制声明

1. work/ 已清空；本审仅基于 c 路引用 + raw stdout/stderr 复核（已直接读 `runs/run-1778560393-59119/raw/<tool>/<entry>.stdout/exit` 确认证据真实存在）
2. aeneas-hol4 在 c 路报告未独立列出但属同一 backend 模板，建议 fix 时一并应用
3. ror tier-0 wrapper grep 行号引用基于 v6 当前 HEAD；如再演进 tier-0，需同步本报告

---

> 报告创建时间：2026-05-12
>
> 审查者：独立 cc 路 counter-challenge agent
>
> 上游：`docs/fixes/audit-v6-c-false-negative-2026-05-12.md`
