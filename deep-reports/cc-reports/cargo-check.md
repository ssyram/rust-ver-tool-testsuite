# cargo-check 深度报告

## 元数据

- **run**：`runs/run-1778226613-5282/`（2026-05-08，146 entries × 19 工具矩阵）
- **工具版本**：`cargo 1.95.0 (f2d3ce0bd 2026-03-21)`（机器默认 stable toolchain）
- **通过率**：146/146 = 100%
- **时长**（毫秒）：avg 2124 / median 222 / p90 6913 / max 26420
- **时效声明**：本快照锚定上述 run id + 工具版本 + corpus，不构成长期承诺。

## 工具内部 pipeline + 前端边界

cargo-check 是 Cargo 内置子命令，调用 stable rustc 完成完整前端：parse → macro expand → name resolution → type check → borrow check → MIR build。**没有后端**——在 codegen / LLVM IR 之前 exit 0。本工具的"前端 = 全过程"，矩阵中作为"健全性基准"存在：若某 entry 在其他工具上 FAILED 而 cargo-check 也 FAILED，说明问题在样例本身的 Rust 合法性，不在工具能力。`tool.toml` 命令为 `cargo check --bin __ts_harness`，对 runner 注入的 bin harness 做类型检查；harness 模板 `fn main() { <crate>::<entry>(); }` 标准 bin 入口。

## SUCCESS 信号 + 形式严格性

- **形式指标**：`cargo check --bin __ts_harness` exit 0
- **partial 暴露**：rustc 自身的 error registry——任何错误 → exit ≠ 0
- **0 误报**：✅ 形式可证。rustc 单一信号，exit 0 ⇔ type / borrow check 全部通过
- **0 漏报**：✅ 形式可证。任何 check 失败 → rustc exit ≠ 0
- **漏报盲点**：无

## 实测结果

### 按 feature 类目分布

本次 146 个 entry 全部 SUCCESS，覆盖 41 个 feature 类目：

```
aeneas-limit (8/8) / arc (1/1) / assoc-type (1/1) / bigint (8/8) / box (2/2)
charon-limit (7/7) / closure (2/2) / closure-adv (4/4) / collections (2/2)
concurrency (2/2) / const (1/1) / creusot-limit (7/7) / deps-complex (7/7)
drop (1/1) / enum (2/2) / error (1/1) / float (10/10) / gat (1/1) / generic (4/4)
hax-limit (8/8) / hello (1/1) / hrtb (1/1) / impl-trait (1/1) / industrial (6/6)
int (2/2) / int-width (14/14) / iter (1/1) / kani-limit (7/7) / lifetime (3/3)
miri-limit (7/7) / panic (2/2) / prusti-limit (8/8) / rc (1/1) / refcell (1/1)
repr (2/2) / slice (1/1) / trait (1/1) / trait-obj (2/2) / unsafe-adv (3/3)
unsafe-ptr (2/2) / vec (1/1)
```

包含所有 `*-limit` 类别（aeneas / charon / creusot / hax / kani / miri / prusti），共 52 entries——这些 entry 触发的是其他工具的内部限制，不触及 stable rustc 的语言接受面。`industrial/` 6 个 entry（vendor 出来的 `x509-parser` / `rsa` / `sha2` 真实 crate）也全部 SUCCESS。

### 失败模式归类

本次 0 个 FAILED——无可归类的失败模式。

### 时长尾端观察

`industrial/x509-parser/cert-parse/x509_parse_der`（26.4s）、`deps-complex/chrono-serde/chrono_serde`（20.7s）、`industrial/x509-parser/cert-parse/x509_subject_extensions`（20.7s）、`industrial/rsa/rsa-pkcs8/*`（20s 附近）等长尾全部是 dep-heavy entry——cargo 走 crates.io index 同步 + serde / chrono / num-bigint / nom / asn1-rs 等传递依赖编译。这些时间反映 cargo build 流水线开销，不反映 rustc 前端能力。median 仅 222ms。

## 与本次测试边界的关系

- **测试切割点**：cargo-check 不执行代码、不进入 codegen、不检测运行时 UB；它的 SUCCESS 仅蕴含 rustc 前端接受。下游 verifier（kani / charon / hax 等）能否在同一 entry 上完成翻译 / 验证与本工具的判定无关。
- **作为基线的 corpus 倾向**：本次 corpus 全部 entry 都是合法 stable Rust（含 `*-limit` 类别——这些是其他工具的限制集，但语言层面合法）。corpus 设计上避免了纯语法不合法的样例；如果将来引入"故意写坏的 Rust"（如未声明类型、未导入 trait），cargo-check 会出现 FAILED 信号，那时它就会发挥"健全性基准"的过滤作用。
- **本次的实际作用**：所有 19 个工具中 cargo-check 是**唯一 0 失败的工具**——这从反面验证了 corpus 整体在 stable Rust 接受面之内，其他工具的 FAILED 都是**工具自身能力差异**而非样例 Rust 错误。

## 历史快照声明

本报告是 2026-05-08 运行 `runs/run-1778226613-5282` 的实测快照；锚定 `cargo 1.95.0 (f2d3ce0bd 2026-03-21)` stable toolchain × 当前 corpus（146 entries）。toolchain 升级或 corpus 变化（特别是引入语法不合法 entry）后需重测。
