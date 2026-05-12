# cargo-check — 特性支持评估报告（v6 final post-P35 baseline）

## 元数据

- **数据源**：`runs/run-1778560393-59119/`（2026-05-12 v6 final，合并 verus rerun + R7 5-tool rerun，post-P35）
- **工具配置**：`tools/cargo-check/`（`tool.toml` + `README.md`，无 wrapper）
- **工具版本**：`cargo 1.95.0 (f2d3ce0bd 2026-03-21)`（机器默认 stable toolchain）
- **本工具实测**：n=161 / SUCCESS=161 / FAILED=0 / UNKNOWN=0，通过率 **100%**
- **时长分布**：avg 1655ms / median 194ms / p90 5339ms / max 17905ms
- **宪法 baseline**：`principles.md` v8（P27 修宪后 / P31 法律传导后 / P35 累积派生应用）
- **时效声明**：本快照锚定上述 run id + 工具版本 + corpus，不构成长期承诺。

## 矩阵角色：baseline，不是被评测的"工具"

cargo-check 在 20 工具矩阵中的定位与其他 19 个验证工具不同，按 P35 累积派生重申：

- **不测验证能力**：rustc 前端完成 parse / macro expand / name resolution / type check / borrow check / MIR build 即 exit 0，**不进入 codegen / 求解 / 翻译**。本行不参与"工具特性覆盖率"维度的比较。
- **作为 corpus 合法性的健全性基准**：任一 entry 若在某验证工具上 FAILED，先看 cargo-check 是否同样 FAILED——若是，则问题在样例自身的 Rust 合法性，与工具能力无关；若不是，则 FAILED 反映工具能力差异。
- **100% 通过的实际意义**（本次 v6）：从反面验证 corpus 整体落在 stable Rust 接受面之内。换言之，本 run 中其他 19 个工具的 `161 - SUCCESS` 数 = FAILED 数，**全部归因到工具能力差异**，不存在"corpus 自身写了非法 Rust 导致 verifier 拒"的混淆来源。

这条性质是 corpus 维护的合法性兜底，不是 cargo-check 工具能力的对比性结论。本报告刻意不与其他 19 工具做通过率排名。

## pipeline + 前端边界

```
cargo check --bin __ts_harness
  ↓
rustc 前端：parse → macro expand → name resolution → type check → borrow check → MIR build
  ↓
exit 0（不进入 codegen / LLVM IR / 链接）
```

- **harness 形态**：`fn main() { <crate>::<entry>(); }` 标准 bin 入口（runner 注入到 `__ts_harness` bin target）
- **测量边界**：rustc 前端全过程。cargo-check 没有"后端"概念，所以"前端 = 全过程"
- **项目维护 wrapper**：无。`tool.toml` 直接调 `cargo check --bin __ts_harness`，无 shell 包装
- **归因路径单一**：按 P31 / `tool-integration.md` §四.5，任何 exit ≠ 0 都直接来自 rustc 本身——既不存在"我们 wrapper bug → UNKNOWN (b)"通路，也不存在"官方 wrapper 锅 → FAILED"歧义。失败归因在本工具上是退化的

## SUCCESS 信号 + 形式严格性

按宪法 §六 双通路 partial 暴露：

- **主信号通路**：`cargo check --bin __ts_harness` exit 0
- **wrapper 补抓通路**：无（无项目维护 wrapper）

形式严格性 0 误报 / 0 漏报状态（按 `tool-integration.md` §三 / §四.1）：

- **0 误报**：形式可证。rustc 单一 exit 信号，exit 0 ⇔ type / borrow check 全通过。这是 rustc 自身实现保证——任何 check 失败路径都升 `rustc_errors::Diagnostic` 并最终影响 exit code
- **0 漏报**：形式可证。任何 check 失败 → rustc exit ≠ 0。rustc 不存在 silent skip 路径（rustc 没有"我跳过这部分检查也算成功"的模式——与 verifier 工具如 hax / aeneas 完全不同）
- **漏报盲点**：无（与所有其他工具不同；cargo-check 是矩阵中唯一可宣告"0 误报 / 0 漏报均形式可证"的项）

**注**：此处"形式可证"指可溯源到 rustc 源码层的单一信号通路 + 无 silent skip 设计；非项目自陈，rustc 的 error 模型是 Rust 官方的设计约束。这条强声明在本工具上站得住，是因为 cargo-check 的 oracle 与 rustc exit code 完全重合——不存在 P30 在其他工具上推翻"0 漏报"的"silent skip / partial 自陈"路径。

## 失败分桶（按 P31 §四.5 归因分类）

**本次 0 个 FAILED**——无可分桶项。

- 无"工具不支持"桶（cargo-check 不存在自陈 reject 概念——它接受所有合法 stable Rust）
- 无"官方 wrapper / driver crash"桶（无官方 wrapper）
- 无"我们 wrapper bug"桶（无项目 wrapper）
- 无"我们 corpus / 配置 bug"桶（corpus 在 stable Rust 接受面内）
- 无"环境损坏"桶
- 无"漏报候选"桶

## 漏报盲点（诚实声明）

无。理由见上节"0 漏报：形式可证"。

cargo-check 与项目 wrapper-based 工具（aeneas / hax / kmir / rocq-of-rust / verifast / prusti / verus 等）的关键差异：项目 wrapper 添加的 partial 暴露 gate 都是为了补 grep silent skip——cargo-check 没有这层需求，因此也不需要列"已封堵盲点" / "未封堵盲点"两栏。

## v5.1 → v6 ΔS 解释

- v5.1 baseline：146 entries × 100% = 146 SUCCESS（`runs/run-1778226613-5282/`，2026-05-08）
- v6 baseline：161 entries × 100% = 161 SUCCESS（本次 v6 final post-P35）
- **ΔS = +15**：来自 v5.1 → v6 间 corpus 扩张（新增 `runnable/` 类目 15 entries）。通过率不变（仍 100%）
- 无任何 v5.1 SUCCESS 在 v6 退化为 FAILED——corpus 扩张未引入 Rust 不合法样例
- 无任何 v5.1 FAILED 在 v6 转 SUCCESS（v5.1 本就 0 FAILED）

## 时长尾端观察

| ms | entry |
|---|---|
| 17905 | `deps-complex/chrono-serde/chrono_serde` |
| 16966 | `industrial/x509-parser/cert-parse/x509_parse_der` |
| 16575 | `industrial/x509-parser/cert-parse/x509_subject_extensions` |
| 15072 | `industrial/rsa/rsa-pkcs8/rsa_pubkey_from_pkcs8` |
| 12728 | `deps-complex/collections-serde/collections_serde` |
| 11624 | `deps-complex/bigint-serde/bigint_serde` |
| 11501 | `deps-complex/trait-serde-generic/trait_serde_generic` |
| 9716  | `bigint/num-complex-ops/num_complex_ops` |

长尾全部是 dep-heavy entry——cargo 走 crates.io index 同步 + serde / chrono / num-bigint / nom / asn1-rs 等传递依赖编译。这些时间反映 cargo build 流水线开销，与 rustc 前端能力无关。median 仅 194ms 印证了"非依赖样例的前端检查在毫秒级"。

## 修订建议清单（仅"我们导致"失败）

**无需修订**——所有 161 entry SUCCESS，无 FAILED / UNKNOWN，无"我们导致"项。

cargo-check 作为 baseline 的角色在 v6 final post-P35 baseline 上仍保持完整：100% 通过 → corpus 在 stable Rust 接受面内 → 其他工具 FAILED 数全部归因到工具能力差异，不混入 corpus 合法性噪声。
