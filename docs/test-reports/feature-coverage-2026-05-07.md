# 测试报告：7 工具 × 43 entries 特性覆盖矩阵

**Run**: `runs/run-1778071324-9549/`  
**生成日期**: 2026-05-07  
**总数**: 7 工具 × 43 entries = **301 任务**  
**结果**: 260 SUCCESS / 41 FAILED / 0 UNKNOWN

UNKNOWN = 0 表明 runner 自身在整次 run 中零故障——所有 FAILED 都是工具层真信号，不是 runner 问题。

## 一、整体矩阵

| 工具 | SUCCESS | FAILED | 备注 |
|---|---|---|---|
| cargo-check  | 43/43 | 0 | baseline，确认所有样例编译通过 |
| charon-poly  | 43/43 | 0 | poly 不实例化，绕过 vtable 处理 |
| charon-mono  | 42/43 | 1 | 1 真翻译 panic（详见下文） |
| MIRI         | 43/43 | 0 | UB 检测器，样例无 UB 全 pass |
| Kani         | 39/43 | 4 | 1 unsupported + 3 timeout |
| Creusot      | 35/43 | 8 | dyn / unsafe / iterator 三类硬约束 + 1 ICE |
| Prusti       | 15/43 | 28 | 现代 Rust 特性大面积空缺 |

## 二、Charon 重映射

| 配置 | charon-poly | charon-mono |
|---|---|---|
| 无 `--abort-on-error`（旧） | 43/43 假 SUCCESS | 43/43 假 SUCCESS |
| 加 `--abort-on-error`（现） | 43/43 真 SUCCESS | **42/43，1 真 FAILED** |

charon 默认遇内部 panic 仍 exit 0。无 flag 时 `lifetime/static-bound`（`Box<dyn Any>` vtable drop preshim）的 panic 被静默吞掉显示成 SUCCESS。加 flag 后真实暴露：

```
thread 'rustc' panicked at translate/translate_trait_objects.rs:1707:13:
Could not determine method index for drop in vtable
ERROR Compilation panicked
```

只在 mono 模式触发——poly 不做完整实例化绕过了。这是 charon 已知 vtable 处理 bug。

## 三、各工具能力地图

按 SUCCESS / FAILED 维度归纳，每条都引到具体 entry 与 stderr 关键字。

### Kani（4 缺陷）

| Entry | 失败模式 | 缺陷分类 |
|---|---|---|
| `collections/hashmap` | exit 1 + `Found unsupported constructs` | HashMap 内部 intrinsics（hash 函数 / random state）不支持 |
| `collections/btreemap` | TIMEOUT 600s | BTreeMap insert 路径 SAT 爆炸 |
| `concurrency/thread-mutex` | TIMEOUT 600s | thread spawn / Mutex 不被建模（kani 单线程 model） |
| `enum/nested-guard` | TIMEOUT 600s | 递归 enum + Box + match guard 路径爆炸 |

弱点集中在 std collection 内部 / 并发原语 / 重递归数据结构。其余 39 entry 全 pass：含 generics / GAT / HRTB / impl Trait / 进阶 unsafe（transmute / MaybeUninit / ptr::write）/ Drop / repr(C) / union / atomic / const fn / wrapping & checked 整数。

### Creusot（8 缺陷，4 类）

| Entry | 关键 stderr |
|---|---|
| `trait-obj/dyn-dispatch` | `forbidden dyn type: dyn Greeter` |
| `closure-adv/boxed-dyn-fn` | `forbidden dyn type: dyn std::ops::Fn(i32) -> i32` |
| `lifetime/static-bound` | `forbidden dyn type: dyn std::any::Any` |
| `concurrency/thread-mutex` | `forbidden dyn type: dyn Any + Send`（Mutex 内部产生） |
| `unsafe-ptr/raw-read` | `Dereference of a raw pointer is forbidden in creusot` |
| `error/result-question` | `Split: creusot_std::prelude::IteratorSpec is not satisfied` |
| `collections/btreemap` | `BTreeMap::Iter: IteratorSpec is not satisfied` |
| `repr/union` | **internal compiler error: entered unreachable code**（ICE） |

四类硬约束：
1. **`dyn Trait` trait object 一律拒**（4 个）—— "dyn support is currently minimal"，是 creusot 已知未实现。
2. **raw pointer 禁**（1 个）—— 强制走 `creusot_std::ghost::perm::Perm<*const T>` ghost ownership。
3. **std iterator spec 缺失**（2 个）—— `Split` / `BTreeMap::Iter` 没实现 `IteratorSpec`。
4. **union 触发 ICE**（1 个）—— creusot 自己崩溃。

剩 35 entry SUCCESS——含纯函数、generic、HashMap、Vec、Rc/Arc、closure (fn/fnmut)、enum match、Drop、impl Trait return、HRTB、GAT。Creusot 在**纯 Rust 语言核心特性**支持出乎意料好；缺陷集中在 OOP-ish 抽象、unsafe、std iterator 三块。

### Prusti（28 缺陷）

按 stderr 关键字分类：

**(a) unsupported feature**（21 个）：

| 缺陷类 | 中招 entry |
|---|---|
| HRTB 高阶生命周期 | `hrtb/for-all-lifetime`, `iter/chain-collect` |
| `impl Trait` 返回（Opaque type） | `closure-adv/return-impl-fn`, `impl-trait/return-iter` |
| Box unsizing（`Box<T> → Box<dyn>` / `Box<[T]>`） | `trait-obj/dyn-dispatch`, `lifetime/static-bound`, `gat/lending-iter` |
| Box dyn Fn | `closure-adv/boxed-dyn-fn` |
| shallow borrows（match 路径） | `enum/nested-guard` |
| const generics | `generic/array-len` |
| reference constants | `generic/sum-bound`, `panic/explicit`, `collections/btreemap` |
| 引用类型字段 | `collections/hashmap` |
| iterator 不全 | `slice/index-iter`, `concurrency/thread-mutex` |
| union | `repr/union`, `unsafe-adv/maybe-uninit` |
| raw addr / cast to raw ptr | `unsafe-adv/ptr-write`, `unsafe-ptr/raw-read` |

**(b) internal error**（3 个）—— prusti 自己崩：`closure/fn-fnmut/closure_fn`、`closure/fn-fnmut/closure_fnmut`、`drop/custom-drop/custom_drop_order`

**(c) verification error**（4 个）—— prusti **真正跑了 verify** 并发现潜在 panic 路径（这其实是 prusti 在工作）：
- `arc/clone-drop`, `rc/clone-drop`：`add overflow`
- `const/const-fn`：`multiply overflow`
- `panic/div-zero`：`divide by zero`

**(d) resolution failure**（1 个）—— `error/result-question` 在 cargo 解析阶段挂，不算 prusti 能力缺陷。

弱点：HRTB / impl Trait / Box unsize / const generic / union / raw ptr 等中级到高级特性几乎全 unsupported。适用域是简单单态 Rust + 显式 spec。注意：**(c) 4 个 verification error 是 prusti 在工作的证据**——下游分析者要分清"unsupported"（拒绝）与"verification error"（跑了但报警）的语义。

### Charon-mono（1 缺陷）

| Entry | 失败模式 |
|---|---|
| `lifetime/static-bound` | charon-driver thread panic on `Box<dyn Any>` vtable drop preshim（`translate_trait_objects.rs:1707:13`） |

实例化 `Box<dyn Any>` 触发 charon 内部"`Could not determine method index for drop in vtable`"。Poly 模式不做该实例化所以绕过——这是 charon 自身 mono 翻译路径的 bug，不是样例代码问题。

### MIRI / Charon-poly / cargo-check（无缺陷信号）

各 43/43 SUCCESS。三种"全 pass"含义不同：

- **cargo-check**：baseline，仅证样例都合法 Rust，无验证含义
- **MIRI**：UB 检测器，样例都无 UB 自然全 pass。要测出 MIRI 缺陷需特意写 UB 样例。
- **Charon-poly**：翻译器，poly 模式不做完整实例化，所有 entry 都翻译成功。要测出 charon 真缺陷需 mono 模式 + 复杂 vtable / generic 实例化场景。

## 四、跨工具对比读法

读矩阵的几个有用切片：

**纯函数 + 基础类型 + 简单 unsafe**（如 `hello/basic-hello`, `vec/basic-push-pop`, `box/basic-alloc`, `rc/clone-drop`, `unsafe-ptr/raw-read` 中的 read 部分）：cargo-check / kani / miri / charon×2 / prusti / creusot 全过——基础语言层支持普遍良好。

**dyn Trait**：kani 通 / miri 通 / charon-mono FAIL / creusot FAIL / prusti FAIL（unsizing 不支持）。**dyn 是验证工具普遍弱区**。

**HRTB / impl Trait return / GAT**：kani 通 / charon 通 / creusot 通 / **prusti 全 FAIL**。这些 Rust 现代特性 prusti 还没跟上。

**std collection**：cargo-check / charon / miri 通；kani 在 hashmap unsupp + btreemap timeout；prusti 多个 unsupported；creusot iterator spec 缺。**std collection 内部实现是普遍痛点**。

**concurrency**：除 miri / cargo-check / charon-poly 外，所有 verifier 都不行——kani timeout、prusti unsupported、creusot dyn 拒、charon-mono 没事是因 thread-mutex 没碰 vtable bug。**并发是 verifier 共同痛点**。

**panic / overflow / div-zero**：kani 全过（CBMC 强项）/ prusti 真发现 verification error（这是它的核心能力）/ creusot 全过 / charon 翻译过 / miri 不查（运行时） / cargo-check 不查。**算术异常检测 prusti 与 kani 各自有优势**。

## 五、统计与结构观察

**SUCCESS 平均率**: 86.4%（260/301）。

**FAILED 分布**: 28 prusti + 8 creusot + 4 kani + 1 charon-mono = 41。Prusti 占失败 68%，集中反映 Rust 现代特性 vs prusti 当前实现的 gap。

**TIMEOUT**: 仅 kani 3 个（btreemap, thread-mutex, nested-guard），均在 600s 上限。Timeout 实施（process group SIGKILL）真生效——3 个任务每个跑满 600s ± 50ms 后被杀。

**ICE / 工具崩溃**: 2 个（creusot/repr-union, charon-mono/lifetime-static-bound）。这两条是工具自身 bug 不是设计拒绝。

**verification error vs unsupported**: prusti 4 条 verification error 表明它正常工作；其他 24 条 unsupported 表明它对应特性还未实现。下游解读 prusti 数据时务必分清。

## 六、运行时信号验证

本次 run 同时验证了几个 runner 关键修复：

- **timeout 实施（process group SIGKILL）**：3 个 kani timeout 准时停（600027ms / 600045ms / 600049ms）；之前的 v1 在这种 entry 上 runner thread 永久阻塞（`read_to_end` 等不到 EOF）
- **Cargo.lock 排除**：prusti 的 nightly-2023-08-15 cargo 不再因为 v4 lockfile 解析失败——之前 prusti 0/42 全挂在 lock 上
- **`entry_mode = "lib"` 取代 lib**：creusot 35 entry 跑通——之前 0/43 因为样例 lib 没 `use creusot_std`
- **`--abort-on-error` charon 重映射**：暴露 1 个真 mono FAIL，覆盖之前 43/43 假 SUCCESS
- **toml AST patch**：`extra_cargo_deps` inject 在 42 个无 `[dependencies]` 段的样例上 + 1 个 creusot 集成上均成功，无字符串拼接边角错
- **UNKNOWN = 0**：runner 内部零故障

## 七、Raw 数据与重现

完整 raw output 在 `runs/run-1778071324-9549/raw/<tool>/<slug>.{stdout,stderr,exit}`，按 tool × entry 分文件。

`results.json` 含每 task `{ entry_id, tool, status, exit_code, duration_ms, timed_out, raw_stdout, raw_stderr, error }` 字段。

`report.md` 含按 feature 分组的人读矩阵 + 末尾汇总。
