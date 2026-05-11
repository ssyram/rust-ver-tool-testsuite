# hax-lean-eval Corpus Baseline (P16-impl-A 产出)

> **产出阶段**：P16-impl-A（corpus 扩展 + schema 落地，本阶段交付物）
> **后续阶段**：P16-impl-B（实施 `tools/hax-lean-eval/` 工具入口）将以本文档为反误报双向实测的"已知 SUCCESS 一致性"基准
> **设计稿溯源**：[`../design/hax-lean-consistency-design-2026-05-11.md`](../design/hax-lean-consistency-design-2026-05-11.md)
> **schema 锚点**：[`../design/detailed-design.md`](../design/detailed-design.md) §一·单文件轨·`[runnable.<entry_fn>]` 扩展段

---

## §1 新增 15 个 runnable entry 清单

每条 entry 满足 design §1.4 纯内部化判定准则（参数 / 返回类型 ∈ { `i*` / `u*` / `bool` }；函数体仅用 hax-lean prelude 已支持的符号：基础算术 / 比较 / 控制流 / match / 自递归 / 自定义 ADT）。

| # | entry id（feature/dir/fn） | fn 签名 | 测的特性 | inputs[0] | expected[0] | 全 input → expected |
|---|---|---|---|---|---|---|
| 1 | `runnable/add-two/add_two` | `add_two(i32, i32) -> i32` | pure i32 算术 | `[3, 4]` | `7` | `[3,4]→7 / [10,-7]→3 / [0,0]→0` |
| 2 | `runnable/fact/fact` | `fact(i32) -> i32` | 自递归 + if-else | `[0]` | `1` | `[0]→1 / [1]→1 / [5]→120 / [6]→720` |
| 3 | `runnable/fib/fib` | `fib(u32) -> u32` | 自递归 + match over 数值字面 | `[0]` | `0` | `[0]→0 / [1]→1 / [5]→5 / [10]→55` |
| 4 | `runnable/gcd/gcd` | `gcd(u32, u32) -> u32` | 自递归 + 余数 | `[12, 18]` | `6` | `[12,18]→6 / [100,75]→25 / [17,5]→1 / [0,7]→7` |
| 5 | `runnable/abs/my_abs` | `my_abs(i32) -> i32` | if-else 单分支 + 一元 `-` | `[0]` | `0` | `[0]→0 / [7]→7 / [-7]→7 / [42]→42 / [-100]→100` |
| 6 | `runnable/max3/max3` | `max3(i32, i32, i32) -> i32` | 多 if 嵌套 | `[1, 2, 3]` | `3` | `[1,2,3]→3 / [3,2,1]→3 / [-5,0,-1]→0 / [7,7,7]→7` |
| 7 | `runnable/power/pow_n` | `pow_n(i32, u32) -> i32` | 自递归 + 乘法 | `[2, 0]` | `1` | `[2,0]→1 / [2,8]→256 / [3,4]→81 / [-2,3]→-8 / [5,1]→5` |
| 8 | `runnable/bool-ops/and_or_not` | `and_or_not(bool, bool, bool) -> bool` | `&& \|\| !` 复合 | `[true, true, true]` | `true` | 全 4 组见 hirusttest.toml |
| 9 | `runnable/saturating/sat_add_u8` | `sat_add_u8(u8, u8) -> u8` | 手实现 saturating add（无 builtin） | `[0, 0]` | `0` | `[0,0]→0 / [100,50]→150 / [200,100]→255 / [255,1]→255 / [250,250]→255` |
| 10 | `runnable/digit-sum/digit_sum` | `digit_sum(u32) -> u32` | 自递归 + mod / div | `[0]` | `0` | `[0]→0 / [7]→7 / [123]→6 / [9999]→36 / [1000000]→1` |
| 11 | `runnable/parity/is_even` | `is_even(i32) -> bool` | mod 2 + bool 返回 | `[0]` | `true` | `[0]→true / [1]→false / [2]→true / [-3]→false / [-4]→true / [100]→true` |
| 12 | `runnable/enum-classify/classify_sign` | `classify_sign(i32) -> i32` | 自定义 enum `Sign` + match | `[0]` | `0` | `[0]→0 / [7]→1 / [-3]→-1 / [100]→1 / [-1]→-1` |
| 13 | `runnable/struct-norm/manhattan_of` | `manhattan_of(i32, i32) -> i32` | 自定义 struct `Point` + impl method | `[0, 0]` | `0` | `[0,0]→0 / [3,4]→7 / [-3,4]→7 / [-5,-7]→12 / [10,-2]→12` |
| 14 | `runnable/add-u32/add_u32` | `add_u32(u32, u32) -> u32` | pure u32 算术 | `[0, 0]` | `0` | `[0,0]→0 / [1,2]→3 / [100,200]→300 / [1000,23456]→24456` |
| 15 | `runnable/sub-clamped/sub_clamped` | `sub_clamped(i32, i32) -> i32` | if 边界 clamp（无 checked 内置） | `[10, 3]` | `7` | `[10,3]→7 / [3,10]→0 / [0,0]→0 / [5,5]→0 / [100,1]→99` |

**Rust 端 expected 值已通过 rustc 编译实跑验证**——每条 entry 的全部 inputs 都跑过 `println!("{}", entry_fn(args))`，输出与 hirusttest.toml `expected` 字段字面一致。

---

## §2 P16-impl-B 反误报双向实测起点

每个 entry 都是"已知 SUCCESS 一致性"基准——`tools/hax-lean-eval/` 实施完成后跑这些 entry 应该全部 SUCCESS：

- **Stage 1**：cargo hax exit 0 + 产物无 term-position sorry（与现有 hax-lean tool 一致）
- **Stage 2**：lake env lean exit 0 + stderr 无 `declaration uses 'sorry'` / `unexpected token`
- **Stage 3**：cargo run exit 0 + stdout 含上表 `expected` 字面行
- **Stage 4**：unwrap_rust_m_ok 后 Lean 输出与 Rust 输出 + `expected` 三向一致

**任一 entry 在 P16-impl-B 实测中 FAILED 都需要排查**：

| FAILED 类目 | 可能原因 | 应对 |
|---|---|---|
| Stage 1 FAILED（cargo hax 报错或 silent partial） | hax 翻译边界——某符号未在 hax-lean prelude 实现，或 hax printer bug | 把该 entry 移到 `examples/hax-limit/` 作 limit 案例，并在本文档加注 |
| Stage 2 FAILED（lake env lean 报错） | hax 产物在 Lean 端 typecheck 失败，或上游 hax 输出格式演化导致 #eval 表达式语法不对 | 排查产物，必要时调整 wrapper 渲染规则；若是 hax 上游 bug 则报 issue |
| Stage 3 FAILED（cargo run 失败） | Rust 端 panic / 溢出 / 编译错——理论上不应发生（pure 函数 + 小输入） | 检查 inputs 是否触发 overflow（i32 算术不应越界） |
| Stage 4 FAILED（值不一致） | 翻译语义错误，或 unwrap regex 失效 | 排查 `compare.py` normalize 规则，确认 Lean `#eval` 输出格式与 hax 上游约定一致 |

---

## §3 类型支持矩阵（v1）

P16-impl-A 已实施：参数 / 返回 ∈ { `i*` / `u*` / `bool` }（matrix § detailed-design.md 单文件轨 schema 段）

P16-impl-A 未实施（v2 留项）：

- **Tuple / Array 作为参数 / 返回类型**：design §4.4 风险点 3 提示 `#eval` 对 tuple 的输出格式（`⟨3, 4⟩` vs `(3, 4)`）需 micro-test 验证 normalize 规则；v1 用"struct 在内部，返回 i*"绕过（如 #13 `struct-norm` 返回 `i32` 而非 `Point`）
- **Struct / Enum 作为参数 / 返回类型**：同上，Lean `#eval` 用 `Point.mk 3 4` 形态而 Rust `Debug` 用 `Point { x: 3, y: 4 }`，跨形态比对需 structural normalize（design `compare_mode = "structural"` 占位预留，v1 未实施）
- **浮点 (`f32` / `f64`)**：Rust `Debug` 与 Lean `#eval` 输出格式不一致；design `compare_mode = "epsilon"` 占位预留，v1 未实施
- **i*::checked_* / overflowing_***：design §1.4 禁止集——hax-lean prelude 未实现，翻译产物会含 sorry → 故意未纳入 corpus

---

## §4 schema 兼容性验证

P16-impl-A 验证（2026-05-11 实测）：

- `runner --tool cargo-check` 跑全 matrix：**146 SUCCESS / 15 FAILED / 0 UNKNOWN / 161 total**
  - 146 SUCCESS = 现有 142 个 hirusttest.toml 中的 entry 全部通过
  - 15 FAILED = 新增 runnable entry——cargo-check 在 `entry_mode = "bin"` 下 harness 调零参 fn 而真 fn 带参，编译失败属预期（见 detailed-design.md 中"ID 与真 fn 名"段）
  - 0 UNKNOWN = runner 解析 `[runnable.*]` 段无错——`serde(default)` 默认行为下未知字段被忽略，向后兼容
- 现有 142 个 hirusttest.toml **零破坏**

---

## §5 hax-lean 翻译率（本机当前实测，未达预期）

P16-impl-A 跑 `runner --tool hax-lean --entry 'runnable/*'` 实测：**0 SUCCESS / 15 FAILED**（2026-05-11 本机）。

根因：本机 `TS_HAX_ENGINE_BIN=/Users/ssyram/.opam/default/bin/hax-engine` 当前不可达——`rust-engine/src/ocaml_engine.rs:130` 内部 spawn 找不到子组件（错误：`Os { code: 2, kind: NotFound }`）。**与新增 entry 无关**——baseline `hello/basic-hello/hello` 在同一次 run 中也 FAILED。

这是已知的 hax 工具链本地环境配置问题，不属本任务（P16-impl-A：corpus）范围。P16-impl-B 实施前需先恢复 hax-engine 子组件路径——这条记入 hax-lean-eval tool README 的安装步骤。

P16-impl-B 实施后若 hax-engine 恢复可达，预计本表 15 个 entry 在 Stage 1（hax 翻译）应大部分 SUCCESS（pure 算术 / 控制流 / 自递归是 hax prelude 完全支持的）。`enum-classify` 与 `struct-norm` 因引入自定义 ADT 是 design §1.4 中标 ⚠️ 需 micro-test 的灰色项——可能是 P16-impl-B 阶段第一批失败案例。

---

## §6 后续工作

- P16-impl-B 实施 `tools/hax-lean-eval/`（次要模块 3，按 design §3 四 stage pipeline + §4 反误报论证）
- 若本表 entry 在 P16-impl-B 中 Stage 1 FAILED 数 > 30%，重新评估 design §1.4 准则的实测准确性，可能需要重写部分 entry
