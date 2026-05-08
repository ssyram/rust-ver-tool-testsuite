# kani 深度报告

## 元数据

- **run**：`runs/run-1778226613-5282/`（2026-05-08，146 entries × 19 工具矩阵；host：Apple M5 / macOS / aarch64 / 24 GB / 10 cpu）
- **工具版本**：`cargo-kani 0.67.0`
- **通过率**：144/146 = 98%
- **时长**（毫秒）：avg 3871 / median 1108 / p90 9952 / max 40023
- **时效声明**：本快照锚定上述 run id + 工具版本 + corpus，不构成长期承诺。

## 工具内部 pipeline + 前端边界

Kani 由 AWS 出品，基于 CBMC 的 Rust bounded model checker。pipeline 是：cargo-kani 把 rustc 替换为 `kani-compiler`（rustc plugin）；kani-compiler 接管 rustc 前端后把 MIR 翻译为 GotoC IR，写出 goto-binary（`*.symtab.out`）+ 元数据；最后 `cargo kani` 把 goto-binary 喂给 CBMC 做 SAT/SMT 求解判 reachability / assertion violation。

本测试用 `--only-codegen` 把测试边界精确切在 codegen 完成之处——**前端**（kani-compiler 的 MIR → GotoC + 类型 / 模型构造）算本工具范围；**后端**（CBMC 求解）不算。让 kani 跑完整 SAT 求解会失公（其他工具都在前端层停下，kani 却因求解超时频繁 FAILED）。

## SUCCESS 信号 + 形式严格性

- **形式指标**：`cargo kani --only-codegen --bin __ts_harness` exit 0
- **partial 暴露**：kani-compiler 在 codegen 阶段任何失败 → exit ≠ 0
- **0 误报**：✅ 形式可证。kani exit 0 ⇔ codegen 完成无错误
- **0 漏报**：⚠️ 实测——`Found the following unsupported constructs: caller_location / foreign function ...` 在 stderr 是 warning 而非 error，对 SAT 求解阶段才有意义；理论上 corpus 可能触发"warning + codegen 完成 → exit 0"的边角情形。本次 corpus 上 SUCCESS entry 未观察到此 warning
- **漏报盲点**：codegen 完成 + warning 但 SAT 阶段才会触发问题的 entry（本次未观察到）

## 实测结果

### 按 feature 类目分布

144 SUCCESS / 2 FAILED 跨 41 feature 类目，FAILED 集中在 1 个类目：

```
全过类目（40 个）：aeneas-limit (8/8) / arc / assoc-type / bigint (8/8) / box
  charon-limit (7/7) / closure / closure-adv (4/4) / collections / concurrency
  const / creusot-limit (7/7) / deps-complex (7/7) / drop / enum / error
  float (10/10) / gat / generic (4/4) / hax-limit (8/8) / hello / hrtb / impl-trait
  int / int-width (14/14) / iter / kani-limit (7/7) / lifetime (3/3)
  miri-limit (7/7) / panic / prusti-limit (8/8) / rc / refcell / repr
  slice / trait / trait-obj / unsafe-adv / unsafe-ptr / vec

部分通过：
  industrial: 4/6（仅 x509-parser 两个 entry FAILED）
```

注意 `kani-limit/` 7/7 全 SUCCESS——这些 entry 故意触发 Kani 自声明的"不支持"特性（inline-assembly / async-await / float-overapprox / thread-interleaving-partial / extern-ffi / uninit-memory / simd-bitmask-large-vector），在 `--only-codegen` 路径上 codegen 阶段都通过；它们的"不支持"性质本来要在 CBMC 求解阶段才显现，本测试不到那一步。`bigint/*` 8/8 + `deps-complex/*` 7/7 全 SUCCESS——cargo-kani 走标准 cargo 流程，外部 crate（num-bigint / chrono / serde / nom 等）都进 codegen pipeline。

### 失败模式归类

逐条读 raw stdout（kani 把诊断都打到 stdout）后归类。

#### A. crate `#![deny(unstable_features, unused_qualifications)]` 与 kani 注入的 `#![feature(register_tool)]` 冲突（2/2）

| entry | 触发 |
|---|---|
| `industrial/x509-parser/cert-parse/x509_parse_der` | x509-parser 的 `lib.rs:122` 设 `#![deny(unstable_features, unused_qualifications, ...)]` |
| `industrial/x509-parser/cert-parse/x509_subject_extensions` | 同一 vendor crate |

stdout 节选：

```
error: unnecessary qualification
   --> /.../vendor/x509-parser/src/cri_attributes.rs:31:41
note: the lint level is defined here
   --> /.../vendor/x509-parser/src/lib.rs:123:31
123 |         unused_import_braces, unused_qualifications)]

error: use of an unstable feature
   --> <crate attribute>:1:12
  1 | #![feature(register_tool)]
note: the lint level is defined here
   --> /.../vendor/x509-parser/src/lib.rs:122:9
122 |         unstable_features,

error: Failed to execute cargo (exit status: 101). Found 8 compilation errors.
```

诊断指向 vendor crate 自身的 `lib.rs`：crate 用 `#![deny(unstable_features)]`，但 kani 在编译该 crate 时注入了 `#![feature(register_tool)]`（用于 `#[kani::proof]` 的 `register_tool(kanitool)`）；同时 kani 的 lint 配置让 `unused_qualifications` 命中 8 处，被 `#![deny(unused_qualifications)]` 升为 error。两类共 8 errors → cargo exit 101。

对比同 entry 在 cargo-check 上的行为：cargo-check 也会触发 `unused_qualifications` 的 lint，但只是 warning（不在 deny 升级路径里——stable rustc 不需要注入 `register_tool`），所以 cargo-check exit 0。换言之，**这 2 个 FAILED 是 kani-compiler 注入 `register_tool` feature × vendor crate `deny(unstable_features)` 的交互**——既不在样例的 Rust 合法性层面（cargo-check SUCCESS），也不在 GotoC codegen 能力层面（错误发生在 lint 检查阶段，未到 MIR 翻译）。

### 时长尾端观察

`max=40s` 出现在 `deps-complex/error-chain/error_chain` SUCCESS。`deps-complex/*` 与 `industrial/*` 类下耗时 25–40s 一段——cargo build 拉传递依赖 + kani-compiler 替换 rustc 后的 codegen 时间。CBMC 求解被 `--only-codegen` 跳过，整体耗时主要落在 cargo + GotoC codegen。timeout 设 120s 在本矩阵从未触发。

## 与本次测试边界的关系

- **测试切割点**：本测试不调用 CBMC、不评 SAT/SMT 求解器在 GotoC 上的验证结果。SUCCESS 仅蕴含"kani-compiler 完成 MIR → GotoC codegen"。下游 reachability / assertion 验证不在测量范围。
- **已知 corpus 偏向**：`kani-limit/*` 7 个 entry 故意触发 Kani 自声明的"不支持"特性，期望在 Kani 上 FAILED；但这些"不支持"在 SAT 求解阶段（如 inline-asm / extern-ffi 在 CBMC 内是 trap branch、async/await 是 unsupported summary）才表现，在 `--only-codegen` 路径上 codegen 全过、对应 stderr warning（如 `Found the following unsupported constructs: foreign function`）但 exit 0。corpus 包含"后端边界"样例但本测试只切前端边界。
- **本次未触达**：kani 的 GotoC codegen 因 MIR 节点 unsupported 而拒收的案例。2 个 FAILED 都在 lint 层面（kani-compiler 编译流水线的 rustc 阶段），未进入 MIR → GotoC 翻译路径。

## 历史快照声明

本报告是 2026-05-08 运行 `runs/run-1778226613-5282` 的实测快照；锚定 `cargo-kani 0.67.0` × 当前 corpus（146 entries，含 6 个 industrial vendor crate entries）。kani 升级、kani-compiler 修复 lint 注入、x509-parser 上游放宽 `deny(unstable_features)` 等任一变化后均需重测。
