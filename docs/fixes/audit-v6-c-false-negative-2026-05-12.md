# v6 漏报候选审查（c 路独立审查）

> R6-c：基于 v6 matrix（`runs/run-1778560393-59119/`）的 0 漏报独立审查。
>
> **只观察、不修改、不 commit**。漏报候选留待 cc 阶段 counter-challenge 后由用户决策。
>
> 数据：2026-05-12 run，20 工具 × 161 entries = 3220 results，SUCCESS=2209 / FAILED=897 / UNKNOWN=8。
>
> 漏报定义（按宪法 §六 + tool-integration.md §四）："oracle 判 SUCCESS 但工具自陈 partial / silent skip / 半翻译"。

---

## 1. 总览

- **强候选数（建议加 oracle）**：3 个独立 entry × 工具维度共 **8 个 SUCCESS 实例**
- **系统性模式**：
  1. **aeneas mutually-recursive trait silent partial**：aeneas 自陈"the model will not type-check"——产物形式不可用，aeneas exit 仍 0（2 个 entries × 3-4 backends = 7 SUCCESS）
  2. **aeneas-lean builtin model missing field**：aeneas-lean 自陈"could not find the information for item 'map' / 'sum'. The model defined in the Lean library seems to be missing"——core trait method silent drop（1 SUCCESS）
  3. **rocq-of-rust-typecheck 漏继承 ror tier-0 D3.1 gate**：tier-0 ror 已通过 `grep "is not yet supported"` 抓 silent `Pattern::Wild`，tier-1 wrapper 没继承该 grep（1 SUCCESS）
- **涉及工具**：aeneas-coq / aeneas-fstar / aeneas-lean / rocq-of-rust-typecheck
- **排除（合法 warning，README 已 D3.4 / D3.5 / D3.6 文档化）**：kani concurrency 7 SUCCESS / soteria intrinsic 4 SUCCESS / verus derived auto-spec 1 SUCCESS
- **v5 → v6 修复确认**：P27 已封堵 ror inline-asm / aeneas charon stage type-error / aeneas charon inline-asm（实测在 v6 中均 FAILED）

---

## 2. 候选清单

### 候选 1：aeneas-coq / aeneas-limit/mutually-recursive-traits/trigger_mutually_recursive_traits

**stdout 关键**（去 ANSI 后）：

```
[Warn ] Found an associated type in a trait declaration; trait associated types are usually
        lifted to become parameters of the trait definition, but this can fail with
        mutually-recursive traits as well as GATs. Aeneas cannot handle such types today,
        and the generated code will likely be incorrect.
[Warn ] Mutually recursive trait declarations are not supported; the following group of
        mutually recursive traits is going to be extracted but their model will not type-check:
        'mutually_recursive_traits::__ts_inner::Trait1', ...
        'mutually_recursive_traits::__ts_inner::Trait2', ...
[Warn ] Mutually recursive trait implementations are not supported; the following group of
        mutually recursive impls is going to be extracted but their model will not type-check:
```

aeneas exit 0；wrapper SUCCESS。

**漏报推断**：aeneas 明确自陈三条："Aeneas cannot handle such types today / generated code will likely be incorrect"、"not supported ... model will not type-check"。这是工具最强等级的自陈"我没全干完"——按宪法 §六-2 必须被尊重。产物落盘但 aeneas 自己说不可类型检。

**反向论证**：是否是合法 warning？
- 非求解层假设——aeneas 是前端翻译工具，"model will not type-check" 直接指向产物形式不可用，是前端 partial
- 非边界声明——aeneas README 自陈"形式可证 0 漏报，aeneas exit 0 ⇔ Errors.error_list 空"。但这里 Errors.error_list 为空（用的是 Warn 通道，不是 Error 通道）—— aeneas Warn / Error 之间的分类与"完整完成"精神不一致
- 非样例本身就该被翻译——`aeneas-limit/` 前缀正是"已知 aeneas 限制"的 corpus 分类，但 corpus 分类目的是"展示 silent partial 的反例"——oracle 应该抓住而非放过

**推荐处理**：aeneas wrapper grep aeneas stdout 含 `"model will not type-check"` 或 `"generated code will likely be incorrect"` → FAILED。受影响 entries：aeneas-coq / fstar / lean × `aeneas-limit/mutually-recursive-traits/...` + `impl-trait/return-iter/impl_trait_iter`（候选 2）。

---

### 候选 2：aeneas-coq / aeneas-fstar / impl-trait/return-iter/impl_trait_iter

**stdout 关键**（aeneas-coq）：

```
[Warn ] Mutually recursive trait declarations are not supported; the following group of
        mutually recursive traits is going to be extracted but their model will not type-check:
        'core::iter::traits::accum::Sum', source: '/rustc/library/core/src/iter/traits/accum.rs', ...
        'core::iter::traits::iterator::Iterator', source: '/rustc/library/core/src/iter/traits/iterator.rs', ...
```

aeneas-fstar 同。aeneas-lean 在此 entry 触发**不同** warning（见候选 3）。

**漏报推断**：与候选 1 同类——`impl Trait` 返回 Iterator 触发 `Sum / Iterator` 互递归识别，aeneas 自陈 model not type-check。entry 本意是测"impl Trait 返回 iterator 能否被翻译"——产物不可类型检意味着翻译不可用。

**反向论证**：与候选 1 同。

**推荐处理**：与候选 1 同。同一 grep 一起覆盖。

---

### 候选 3：aeneas-lean / impl-trait/return-iter/impl_trait_iter

**stdout 关键**：

```
[Warn ] When retrieving the builtin information for trait decl
        'core::iter::traits::iterator::Iterator', could not find the information for
        item 'map'. The model defined in the Lean library seems to be missing the
        corresponding field.
[Warn ] When retrieving the builtin information for trait decl
        'core::iter::traits::iterator::Iterator', could not find the information for
        item 'sum'. ... seems to be missing the corresponding field.
```

aeneas exit 0；wrapper SUCCESS。

**漏报推断**：aeneas-lean 自陈"Lean library 的 Iterator 模型缺 `map` / `sum` 字段"——即 entry 用到的 `Iterator::map` / `Iterator::sum` 的 builtin info 在 aeneas-lean prelude 中找不到对应字段，产物中这些 method call 会指向缺失符号。属于前端**builtin 模型不完整**导致 silent translation gap——按"工具自陈我没全干完"必须被尊重。

**反向论证**：是否是合法 warning？
- 非求解层——aeneas-lean 是翻译工具，没有"求解层"
- 非外部 fn 不强制 contract（creusot 的合法设计）——这里是 aeneas-lean 自身 builtin model 缺失，影响产物完整性
- 可能是 "aeneas-lean prelude 演进未跟上 core trait" 的边界——但仍属"工具自陈漏"

**推荐处理**：aeneas-lean wrapper grep stdout 含 `"could not find the information for item"` 或 `"model defined in the Lean library seems to be missing"` → FAILED。或合并到候选 1 的统一 grep `\bWarn \].*not supported\|model will not type-check\|seems to be missing` 多 pattern OR。

---

### 候选 4：rocq-of-rust-typecheck / unsafe-ptr/raw-ptr-const/raw_ptr_const_match

**stderr 关键**（每个 attempt 都重复）：

```
warning: This kind of constant in patterns is not yet supported.
 --> src/lib.rs:9:9
  |
9 |         DISGUISED_INT => 1,
  |         ^^^^^^^^^^^^^

warning: 1 warning emitted
```

**漏报推断**：rocq-of-rust（tier-0）已通过 P27/D3.1 加 `grep "is not yet supported"` 抓住该 silent `Pattern::Wild` 替换路径——v6 中 ror tier-0 在此 entry 已 **FAILED**。但 ror-typecheck（tier-1）wrapper **没继承**这道 grep；tier-1 只复用 ror tier-0 的 gate 1-6（exit / 产物 / failure marker / entry_fn 存在性）+ 新加 coqc gate 7-9。tier-1 README 声明"任一 entry 在本工具 SUCCESS ⇒ 在 tools/rocq-of-rust SUCCESS（档 1 ≤ 档 0）"——v6 实测推翻：此 entry tier-0 FAILED 但 tier-1 SUCCESS。

**反向论证**：是否是合法 warning？
- ror-typecheck README "0 漏报" 自陈"coqc 对任何 typecheck 失败必 exit ≠ 0"——但缺失的不是 coqc 端漏报，是 ror translate 端的 silent skip（已被 ror tier-0 D3.1 抓，tier-1 漏抄）
- README "Admitted 一通到底" 边界**不**适用——这里是 ror silently 把 `DISGUISED_INT => 1` 替换为 wildcard `_`（产物语义改变），不是 Admitted 占位
- 这是**wrapper 实现漏抄**，不是设计意图

**推荐处理**：ror-typecheck wrapper stage 1 同步追加 ror tier-0 D3.1 的 `grep "is not yet supported"` gate（在 wrapper 第 113-121 行 translate 之后、第 124+ gate 检查之前插入）。或更彻底：tier-1 wrapper 改为先调用 tier-0 wrapper，全部 gate 1-6 复用，再做 coqc——避免未来 tier-0 加 gate tier-1 再漏抄。

---

## 3. 排除候选（合法 warning，README 已 D3.4 / D3.5 / D3.6 文档化）

| # | 工具 | entry 数 | warning 关键 | 排除理由 |
|---|---|---|---|---|
| E1 | kani | 7 SUCCESS | `Kani currently does not support concurrency. The following constructs will be treated as sequential operations` | README D3.4：BMC 单线程语义约束（求解层假设，前端完成）；非前端 partial |
| E2 | soteria | 2 SUCCESS | `An atomic intrinsic was encountered; it will be executed as sequential code` | README D3.5：符号执行单线程语义约束（求解层假设） |
| E3 | soteria | 2 SUCCESS | `A complex floating point intrinsic was encountered; it will be executed with a significant over-approximation` | README D3.5：soundness-preserving abstraction（非 silent skip） |
| E4 | verus | 1 SUCCESS | `autoderive Clone impl does not take the form Verus expects; continuing, but without adding a specification for the derived Clone impl` | README D3.6：spec gen 是 VIR → SMT 中间步骤（非前端） |

均经"宪法 §六-3 前端测量原则"原则解释为合法——SUCCESS 不抓。

---

## 4. 系统性二阶模式（非漏报，仅记录）

### 4.1 aeneas "Found an unknown type declaration with region parameters"

跨 9-10 个 SUCCESS entries 出现：

```
[Warn ] Found an unknown type declaration with region parameters: as we can not know
        whether the regions are used in mutable borrows or not the extracted code may be incorrect.
```

**判定**：aeneas 自陈"extracted code may be incorrect"。但这是 aeneas 对**外部 type**（如 `core::fmt::Formatter`）的标准 region-conservative 警告，与 creusot 对 external fn 不强制 contract 同类——属"对当前 crate 外部依赖的保守翻译边界"。涉及 SUCCESS entries 包括 `bigint/bigint-arith` / `collections/hashmap` / `panic/explicit` 等基础样例，若 oracle 抓将导致大面积 FAILED—— **不建议加 grep**。

**README 补完建议**：aeneas-* README "漏报盲点"应补"对当前 crate 外部 type（如 core::fmt::Formatter）region-conservative warning，按外部依赖边界精神保持 SUCCESS（与 creusot external fn 同类）"。

### 4.2 ror-typecheck 与 ror tier-0 的等价声明 vs 实测漂移

README 第 12-16 行声明 "Gates 1-6 mirror tools/rocq-of-rust"——v5 时点等价，v6 ror tier-0 加 D3.1 后两者**不再等价**。建议改为运行时直接调用 ror tier-0 wrapper，或在 ror tier-0 wrapper 加 gate 时同步更新 ror-typecheck wrapper（写入 CHANGES）。

---

## 5. 总结

- **强候选（建议加 oracle）**：3 个独立 entry × 工具维度共 **8 SUCCESS 实例**
  1. aeneas-coq / aeneas-fstar / aeneas-lean × `aeneas-limit/mutually-recursive-traits/...` （3）
  2. aeneas-coq / aeneas-fstar × `impl-trait/return-iter/impl_trait_iter` （2）
  3. aeneas-lean × `impl-trait/return-iter/impl_trait_iter`（不同 warning 通道，1）
  4. rocq-of-rust-typecheck × `unsafe-ptr/raw-ptr-const/raw_ptr_const_match`（1）
  - 合计实例数：3+2+1+1 = 7（注：候选 1+2 可共用同一 grep；候选 3 是 aeneas-lean 独立 warning）
- **README 补完候选**：1（aeneas-* "extracted code may be incorrect" 对外部 type 的 region warning，已知盲点补诚实声明）
- **排除**：4 类（合计 12 SUCCESS 实例）—— kani concurrency 7 / soteria atomic 2 / soteria over-approx 2 / verus derived Clone 1。均经 D3.4 / D3.5 / D3.6 文档化，oracle 不抓符合宪法精神

## 6. cc 阶段反质询要点

1. **候选 1+2 aeneas "model will not type-check"**：
   - 反质询：aeneas Warn vs Error 通道分类是 aeneas 设计选择，"Warn 不升 exit" 是 aeneas 团队判断。oracle 应尊重还是 override？
   - 答辩：宪法 §六-2 "工具自陈我没全干完必须被尊重"——`will not type-check` 是最强等级的自陈，比 `not supported` 更直接。oracle 应抓
   - 决策点：grep 该字面会让 5-7 个 SUCCESS 转 FAILED，影响 aeneas 通过率

2. **候选 3 aeneas-lean "model ... missing"**：
   - 反质询：是否合并到候选 1+2 同一 grep？
   - 答辩：可以——`could not find the information for item` 是 aeneas-lean 独有，应单独 pattern；但都属"前端 builtin 模型不全"语义
   - 决策点：单 pattern OR 还是分两条 grep

3. **候选 4 ror-typecheck**：
   - 反质询：是否 wrapper bug（漏抄）而非 oracle 漏报？
   - 答辩：本质是 wrapper 漏抄 → 表现为 oracle 漏报。修复 = wrapper 加 grep 或调用 tier-0 wrapper
   - 决策点：直接 sync grep 还是改架构调用 tier-0

## 7. 限制声明

1. **work/ 已清空**：本审仅基于 stdout/stderr 层；产物级 grep 不可独立重验
2. **N=7 attempts**：ror-typecheck stderr 含 7 次 attempts 合并；本审看到的是合并 stderr
3. **样例 corpus 偏置**：理论 silent path 在 v6 corpus 0 现象不代表理论不存在
4. **kani / soteria / verus 排除基于 README 自陈**：若用户决定按宪法"完整完成"严格解读重新拉回，候选数+12

---

> 报告创建时间：2026-05-12
>
> 审查者：独立 c 路 agent
>
> 数据：`runs/run-1778560393-59119/`
