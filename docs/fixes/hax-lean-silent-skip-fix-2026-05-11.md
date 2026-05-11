# hax-lean silent skip-item gate 对齐 hax-fstar/coq（2026-05-11，P17 D1）

> 实施 entry_fn 存在性 gate，封堵 `rust-engine/src/backends/lean.rs:1521`
> `ItemKind::RustModule | ItemKind::Use { .. } => nil!()` 的 silent-skip-item 路径，
> 让 hax-lean 与 hax-fstar / hax-coq 在 §3.5 audit-1 / reports H-8 上达到对等。

## §1 问题起源

第二轮 oracle 漏报审计（`docs/fixes/oracle-leak-audit-2-2026-05-11.md` §4.4）已指出：
hax-lean 当前 oracle 在 `tools/hax-lean/tool.toml` 仅有：

1. `cargo hax exit 0` 必要条件（抓 `FromEngine::Diagnostic` 路径）
2. 产物 grep `(:=|pure|mk|,)\s*sorry\b|\bsorry\s*[,)\]]` 抓 silent sentinel sorry
   （`lean.rs:1287` PatKind::Error / `lean.rs:2163` error_node 路径）

但 audit-2 §4.4 表 entry `4 hax-lean 低（已封堵主路径）`，未要求强制 entry_fn gate；
仅以"可选增强"列出推荐 pattern `^[[:space:]]*(def|theorem)[[:space:]]+$TS_ENTRY_FN[[:space:]]`。

P13-A 在 hax-fstar / hax-coq 落地 entry_fn gate 时**已对 hax-lean 同型号路径漏审**：
`oracle-leak-rules-implementation-2-2026-05-11.md` 表 §2 行 31-32 只列 hax-fstar / hax-coq，
没列 hax-lean。reports H-8 + audit-1 §3.5 / docs-fixes H-4 在 fstar/coq 各抓 1-2 个
silent skip-item case，hax-lean 同型号路径未对齐。

用户 P17 批准：把 hax-lean 拉到与 hax-fstar / hax-coq 同等的"silent-skip-item 已封堵"
状态。

## §2 hax-lean 源码 silent-skip-item 路径定位

### 2.1 OCaml engine 端：无 silent-skip-item path

`backends/lean/lean_backend.ml`（共 132 行）仅是 `Backend.Make` + 一系列 `Phases.*` 串接
（lean_backend.ml:95-128），**无 backend-specific item 渲染逻辑**。所有 item 写出由
rust-engine Rust 端完成（详见 §2.2）。这与 hax-fstar / hax-coq 不同——后者在 OCaml 端
`fstar_backend.ml:1771` / `coq_backend.ml:588` 各有 silent-skip-item 路径。hax-lean 的
OCaml 端无对等路径。

### 2.2 rust-engine Rust 端：lean.rs:1521 Use/RustModule path

`/private/tmp/hax/rust-engine/src/backends/lean.rs`（共 2172 行）是真正的 Lean printer。
逐 ItemKind 检索（`grep -nE 'ItemKind::|nil!\(\)|emit_error!'`）找到唯一一条 silent
path：

```rust
// lean.rs:1521
ItemKind::RustModule | ItemKind::Use { .. } => nil!(),
```

`nil!()` 在 hax 的 pretty-doc 系统是**空 document**（与 hax-fstar `Use _ -> []` 同语义）。
该路径：
- 对 `ItemKind::Use`（顶级 `use foo;` 声明）和 `ItemKind::RustModule`（Rust module 边界）
  直接输出空 document
- **不调用** `emit_diagnostic` → cargo hax exit 0
- **不写**任何文本 → 产物中既无定义也无 marker

所有其他 silent-like path 经审查都不构成 silent-skip-item：
- `lean.rs:1287` `PatKind::Error(_) => text!("sorry")`：写 sorry 标记 → 现 oracle sorry-grep 抓
- `lean.rs:2163` `error_node => text!("sorry")`：同上
- `lean.rs:1523` `ItemKind::NotImplementedYet => emit_error!(issue 1706, ...)`：`emit_error!` →
  `disambiguated_todo!` → `todo_document!` → `emit_diagnostic(Unimplemented)` → cargo hax
  exit ≠ 0 → 现 exit-code gate 抓
- `lean.rs:1958` `ItemKind::Alias { .. } => emit_error!(issue 1658, ...)`：同 NotImplementedYet
- `lean.rs:1963` `ItemKind::Error(e) => docs![e]`：写 error 节点（非 silent）

合法 fn item 必经 `ItemKind::Fn`（lean.rs:1453-1505）渲染为 `def <name>` 或 `opaque <name>`
（lean.rs:1472：`if opaque { "opaque" } else { "def" }`）。`ResugaredItemKind::Constant`
（lean.rs:1912）走 `def <name>`，`ResugaredItemKind::RecursiveFn`（lean.rs:1936-1955）重用
`ItemKind::Fn` 路径加 `partial_fixpoint` 后缀。**这是单一入口**——合法 fn 必有 `def` 或
`opaque` 关键字 + entry_fn 名字。

### 2.3 fn 重写为 Use kind 的理论窗口

实际 fn item 不会被 hax phases 重写为 Use kind（Use 是 Rust `use foo;` 顶级 use，不是 fn）。
但与 hax-fstar / hax-coq 的判断完全一致——**理论窗口存在**：

- hax engine phases 未来可能引入新重写让某 fn 在某条件下被消失（dead-code 优化 / inline
  消解后没有调用点 / mutual rec bundle 处理 bug 等）
- 上游 PR 新增 ItemKind variant 走 silent path

实测 0 现象、纯理论窗口封堵（与 hax-fstar `fstar_backend.ml:1771` / hax-coq
`coq_backend.ml:588` 的封堵动机相同）。

## §3 oracle gate 设计

### 3.1 grep pattern 选择

`tools/hax-lean/tool.toml` 在原 sorry-grep 之后追加 gate：

```sh
elif [ -n "$TS_ENTRY_FN" ] && \
     ! grep -rqE "^(def|opaque)[[:space:]]+$TS_ENTRY_FN([[:space:]]|\()" \
         proofs/lean/extraction/ 2>/dev/null; then
    echo "[hax-lean-oracle] FAIL: entry_fn '$TS_ENTRY_FN' missing from .lean products (silent skip — lean.rs:1521 Use/RustModule path; see docs/fixes/hax-lean-silent-skip-fix-2026-05-11.md)" >&2
    rc=1
fi
```

### 3.2 pattern 各成分论证

| 成分 | 选择 | 理由 |
|---|---|---|
| 关键字集 | `def\|opaque` | lean.rs:1472 是唯一 fn-item 入口，二选一；ResugaredItemKind::Constant / RecursiveFn 同走 `def`（lean.rs:1919/1944-1955） |
| 锚定 | `^` | hax-lean 在 `namespace <crate>` 内对 fn item 输出顶格无缩进（基于实测 basic_hello.lean / mutual_recursion.lean） |
| entry_fn 边界 | `([[:space:]]\|\()` | 实测产物形态 `def hello (` —— entry_fn 后必有空白（接 generics / `:`）或 `(`（接 params）；`\b` 不行因 `_`/数字 边界不稳 |
| 变量形式 | `$TS_ENTRY_FN` 无 `{}` | runner expand_env 仅展开 `${...}` 形式；`$TS_ENTRY_FN` 透传 sh，sh 在 child env 中拿到 runner 注入的值（P12 / P13 教训） |
| 不容忍 | `theorem` / `partial def` / `lemma` / `axiom` | lean.rs 全文 grep 仅 `def`/`opaque`/`abbrev`/`instance`——`abbrev` 是 TyAlias，`instance` 是 Impl block，非 fn；其它关键字未出现 |

### 3.3 与 audit-2 §4.4 推荐 pattern 的差异

audit-2 §4.4 推荐：`^[[:space:]]*(def|theorem)[[:space:]]+$TS_ENTRY_FN[[:space:]]`

本实施修正：

1. **去掉 `theorem`**：lean.rs 全文实测无 `theorem` 关键字渲染分支——audit 推荐的防御性扩展无源码支持，反而扩大潜在误报面
2. **加 `opaque`**：lean.rs:1472 `if opaque { "opaque" } else { "def" }`——opaque fn 也是合法翻译，audit 推荐 pattern 漏抓
3. **`^` 而非 `^[[:space:]]*`**：实测产物中 `def` 顶格无缩进（namespace 内不缩进）；放开 `[[:space:]]*` 不必要（同 audit P13-A §2.3 对 hax-coq `^\s*` 的扩展是因为 coq 嵌套 Module 可能缩进，hax-lean 实测不嵌套缩进）
4. **entry_fn 边界改 `([[:space:]]|\()`**：hax-lean 在 entry_fn 后可能立即接 `(`（如 `def hello (_ : Tuple0)`）—— audit 推荐 `[[:space:]]` 不漏；本实施加 `\(` 容忍是冗余防御，不改变命中行为

## §4 反误报双向实测

### 4.1 防漏报实测（silent skip case → FAILED）

无法在当前 hax-engine 上构造真正命中 `lean.rs:1521` 的 fn item case（fn 不会被 phases
重写为 Use kind）。改用**手动注入** `TS_ENTRY_FN=nonexistent_fn` 模拟 silent skip：

```bash
# 用 hello/basic-hello 的产物（含 def hello），entry_fn 设为 nonexistent_fn
cd /tmp/p17-hax-lean-test  # 含 proofs/lean/extraction/basic_hello.lean
TS_ENTRY_FN="nonexistent_fn" 
grep -rqE "^(def|opaque)[[:space:]]+$TS_ENTRY_FN([[:space:]]|\()" proofs/lean/extraction/
# → exit 1（MISS）→ oracle FAIL with entry_fn missing message ✓
```

正确触发 `[hax-lean-oracle] FAIL: entry_fn 'nonexistent_fn' missing` ✓

### 4.2 反误报实测（真 SUCCESS entry → 维持 SUCCESS）

6 个真 SUCCESS entry 跑过 hax-lean，逐一验证新 pattern 命中：

| entry | entry_fn | grep 命中行 |
|---|---|---|
| `hello/basic-hello/hello` | `hello` | `def hello (_ : rust_primitives.hax.Tuple0) :` |
| `enum/data-variants/enum_match_data` | `enum_match_data` | `def enum_match_data (_ : rust_primitives.hax.Tuple0) :` |
| `arc/clone-drop/arc_clone_drop` | `arc_clone_drop` | `def arc_clone_drop (_ : rust_primitives.hax.Tuple0) :` |
| `bigint/bigint-arith/bigint_arith` | `bigint_arith` | `def bigint_arith (_ : rust_primitives.hax.Tuple0) :` |
| `bigint/bigint-bitwise/bigint_bitwise` | `bigint_bitwise` | `def bigint_bitwise (_ : rust_primitives.hax.Tuple0) :` |
| `creusot-limit/mutual-recursion/trigger_is_even` | `trigger_is_even` | `def trigger_is_even (_ : rust_primitives.hax.Tuple0) :` |

新 tool.toml 跑同 6 个 entry：6/6 SUCCESS（runs/run-1778491773-9138）—— 无 false-positive。

## §5 重跑结果

`runs/run-1778491781-11043/`（2026-05-11，hax-lean 单工具）：

- **161 entries，SUCCESS 125 / FAILED 36 / TIMEOUT 0**
- 通过率 **77.6%**（v1 110/146 = 75.3%）
- 数字差异完全由 corpus 扩张解释：+15 runnable entry 全部 SUCCESS
- 新 entry_fn gate 在当前 corpus **0 命中**（与 hax-fstar / hax-coq 一致）
- 36 FAILED 分桶（与 v1 一致）：20 silent-sorry（A 桶）+ 13 [HAX0001/2/3/8]（B/C 桶）+
  2 industrial（D 桶）+ 1 stack overflow（E 桶）= 36

### 与 hax-fstar / hax-coq 对照

| 工具 | gate 数 | silent-skip 实测 |
|---|---|---|
| hax-fstar | sorry-grep + `let/let rec/and <fn>` entry gate | 0 现象（P13-A 实施后） |
| hax-coq | failure-grep + `Definition/Fixpoint/Lemma/Equations/Theorem/Program Definition <fn>` entry gate | 0 现象（P13-A 实施后） |
| **hax-lean** | **sorry-grep + `def/opaque <fn>` entry gate（本 P17 D1）** | **0 现象** |

三工具达到形式严格性对等：sorry sentinel 路径 + silent skip-item 路径双封堵，残余风险仅
为"上游引入第三类新 silent path"的理论窗口。

## §6 派生链

```
reports H-8（hax-coq/fstar/lean 三件套对照）
  └→ audit-1 §3.5（漏审 hax-lean entry_fn gate）
       └→ docs-fixes H-4（hax-lean entry_fn gate 待补）
            └→ P17 D1（本文档实施）
                 └→ tools/hax-lean/tool.toml +1 gate
                 └→ deep-reports/cc-reports/hax-lean.md 升级 0 漏报 ⚠️ → ✅
                 └→ 重跑 runs/run-1778491781-11043 验证无 false-positive
```

audit-2 §4.4 把 hax-lean entry_fn gate 列为"可选增强"——P17 D1 把它升级为"强制"以
**对齐 hax-fstar/coq 的形式严格性表述**。0 漏报从 ⚠️ 实测验证 → ✅ 与三工具对等。

## 改动文件清单

- `tools/hax-lean/tool.toml`：加 entry_fn gate（grep `^(def|opaque)\s+$TS_ENTRY_FN`），更新注释
- `deep-reports/cc-reports/hax-lean.md`：
  - 元数据：run id v1 → vN，通过率 110/146 → 125/161
  - SUCCESS 信号：双轨 → 三轨；加 entry_fn grep 描述
  - 形式严格性：0 漏报 ⚠️ → ✅（与 hax-fstar/coq 对等）；0 误报实测 6 entry 补充
  - 失败模式归类：增加 A' silent skip-item 行（0 现象）
- `docs/fixes/hax-lean-silent-skip-fix-2026-05-11.md`（本文档）

## 参考

- `docs/fixes/oracle-leak-audit-2-2026-05-11.md` §3.2 / §3.3 / §4.4
- `docs/fixes/oracle-leak-rules-implementation-2-2026-05-11.md` §2.2 / §2.3
- `tools/hax-fstar/tool.toml` / `tools/hax-coq/tool.toml`（参照 gate 模板）
- hax engine 源码：`/private/tmp/hax/rust-engine/src/backends/lean.rs:1287/1453/1472/1521/1912/1936/2163`
