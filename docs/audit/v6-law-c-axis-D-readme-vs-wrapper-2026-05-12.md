# Axis D: 工具 README 自陈 vs wrapper 实际行为

审查日期：2026-05-12
审查范围：20 个工具（aeneas × 4 / cargo-check / charon × 2 / creusot / hax × 3 / kani / kmir / miri / prusti / ror / ror-typecheck / soteria / verifast / verus）
方法：逐工具对照 `README.md` 自陈 § "SUCCESS 信号 / partial 暴露机制 / 形式严格性 / 漏报盲点" 与 `tool.toml` + `*-wrapper.sh` 实际抓的信号。

---

## 1. 总览

- 候选数：4（其中 2 真不一致 / 2 stale-comment 文案）
- 涉及工具：rocq-of-rust / rocq-of-rust-typecheck / prusti / verifast
- 系统性问题：READE 与 wrapper "门数 / 信号列表" 漂移——P28 已加 D3.1 stderr "is not yet supported" gate（ror × 2）但 README "6 道门" / "9 道门" 表述未同步更新；prusti / verifast 的 README 描述 stderr marker 联合判定但 wrapper 实际只查 exit + 产物（非真不一致，因 marker 都已映射到 exit ≠ 0）。
- aeneas × 4 / ror × 2 的 P28 / P30 修订全部 wrapper 端落实，且 README 漏报盲点段已对应更新——本次 audit 仅在 README "SUCCESS 门列表" 段发现遗漏更新。
- kani / cargo-check / charon × 2 / creusot / hax × 3 / kmir / miri / soteria / verus 的 README 与 wrapper / tool.toml 在本次审查里一致。

---

## 2. 候选清单

### 候选 D-1：rocq-of-rust / README "6 道门" 与 wrapper 7 道门不一致

**README 自陈**（`tools/rocq-of-rust/README.md`）：
- L43：「rocq-of-rust 的 SUCCESS 必须满足 **6 道门**」
- L47-50：枚举 gate 1-6（exit 0 / 产物存在 / 0-byte / >200 字节 / failure marker / entry_fn Definition）
- 漏报盲点段（L64-67）未列 "is not yet supported" stderr 通道

**wrapper 实际**（`tools/rocq-of-rust/rocq-of-rust-wrapper.sh`）：
- L128-140：**Gate 7**（注释里自命名）`grep -q "is not yet supported" rocq_stderr.log` → exit 1
- L121-149：每次 attempt 都先跑 gate 7，再跑 gate 1-6 AND-reduce
- wrapper 文件头注释（L50-58）也只列 6 道门，与代码不符

**不一致**：
- README "6 道门" + wrapper 实际 7 道门
- README 漏报盲点段没声明 stderr silent partial 通道（D3.1）
- wrapper 内 gate 7 注释明指 D3.1（`docs/fixes/decisions-2026-05-11.md`），文档同步遗漏

**反状态排除**：
- R1 ✓ 现象明确：README 字面 vs wrapper 字面差一道门
- R2 ✗ 这不是 design contract——gate 7 是 P28 加的封堵，不是 deliberate omission
- R4 ✗ 无 fail-fast guard 上游消化此漏抄
- R11 ✗ 不是单纯文案——README 的 "6 道门" 是 0 漏报论证基础，文档与实施门数不一致影响"漏报盲点：实测验证 0 漏报"的可审性
- R12 反驳点：作者可能反驳"gate 7 与 gate 5 语义重合（都是 silent partial 信号），不必单列"——但 wrapper 自己就单列了 gate 7，README 应同步

**建议**：README §"SUCCESS 信号" 加 gate 7 条目；§漏报盲点段加 stderr "is not yet supported" 通道说明（与 aeneas × 4 §"Warn 通道封堵" 平行）。

---

### 候选 D-2：rocq-of-rust-typecheck / README "9 道门" 与 wrapper 10 道门不一致

**README 自陈**（`tools/rocq-of-rust-typecheck/README.md`）：
- L47：「SUCCESS 信号（严格反映档 1 边界）：满足 **9 道门**」
- L49-62：枚举 gate 1-6（继承档 0）+ gate 7-9（coqc exit / .vo / stderr Error）
- L43：「Stage 1 与 `tools/rocq-of-rust` 完全等价（同 sysroot / 同 binary / **同 6 道 grep**）」
- L136：「**Stage 1 与 `tools/rocq-of-rust/tool.toml` 等价（同 6 道 grep）**」

**wrapper 实际**（`tools/rocq-of-rust-typecheck/rocq-of-rust-typecheck-wrapper.sh`）：
- L116-126：Stage 1 后立即跑 "is not yet supported" gate（注释里写明"tier-0 wrapper 已抓 ... tier-1 README 第 43 / 136 行硬声明 tier-1 ⊆ tier-0 + Stage 1 与 tier-0 完全等价 6 道 grep——v6 cc-route 漏报审查发现 tier-1 漏抄此 gate，破声明。补此 gate 恢复 tier-1 ⊆ tier-0 不变式"）
- 即实际 10 道门（7 + 3），不是 9 道

**不一致**：
- README "9 道门" + wrapper 实际 10 道门
- README "Stage 1 与 ror tier-0 等价 6 道 grep" 也已不准——tier-0 wrapper 自身已 7 道
- wrapper 注释自承"破声明"——明确指出 README 漏抄

**反状态排除**：
- R1 ✓ wrapper 自己注释里写了"tier-1 README 第 43 / 136 行硬声明 ... v6 cc-route 漏报审查发现 tier-1 漏抄此 gate"——这是 wrapper 作者自己承认的不一致
- R2 ✗ 同 D-1
- R4 ✗ 不存在外部消化机制
- R11 ✗ 影响"档 1 ⊆ 档 0" 不变式的可审性
- R12 反驳点：作者可能反驳"档 0 拿到 7 道门后，档 1 = 档 0 + 3，所以总 10 道；README 仍写 9 道是同步遗漏"——这正是本候选的诉求（建议同步）

**建议**：README §47 改为 "10 道门"；L43 + L136 把"6 道 grep"改为"7 道 grep"；§漏报盲点段补一条 stderr "is not yet supported" silent partial 通道说明，与 ror tier-0 保持平行。

---

### 候选 D-3：prusti / README "wrapper 联合实施 stderr marker" 表述与 wrapper 不一致（文案）

**README 自陈**（`tools/prusti/README.md` L55-62）：
- L56-60：「前端拒绝：exit code ≠ 0，且 stderr 含下列 marker 之一：`[Prusti: unsupported feature]` / `[Prusti: internal error]` / `thread 'rustc' panicked at prusti-interface/src/environment/mir_storage.rs`」
- L62：「两个条件由 `prusti-strict-wrapper.sh` 联合实施（2026-05-08 起）」

**wrapper 实际**（`tools/prusti/prusti-strict-wrapper.sh` L48-87）：
- 只检查 `arch -x86_64 "$CARGO_PRUSTI"` 的 exit code 与 `.vpr` 文件数量
- **不 grep stderr 的任何 marker**

**不一致**：
- README 字面 "两个条件由 wrapper 联合实施" 隐含 wrapper 实际抓 stderr markers，但 wrapper 完全不抓
- 实际原因：3 个 marker 都对应 exit ≠ 0（Prusti 设计上的不变量），所以 wrapper 只看 exit 已经能覆盖 stderr marker 的语义——这是 design contract（R2 适用）

**反状态排除**：
- R1 ✗→部分：现象是文案误导，不影响 oracle 正确性
- R2 ✓ 真实情况是 Prusti 设计契约——任何 stderr marker 必伴随 exit ≠ 0，wrapper 只查 exit 是 sound 的
- R4 ✓ Prusti 自身的 marker emit 路径与 exit ≠ 0 是绑定的
- R11 ✓ 这是文案问题（"联合实施"措辞不严谨），不是实现 bug
- R12 反驳点：「Prusti 的 3 个 marker 都强制 exit ≠ 0，wrapper 只看 exit 等价覆盖，README "联合实施" 措辞可解读为 oracle 信号集合而非 wrapper 实际代码路径」——可接受，但 README 应避免歧义

**建议**（低优先级 / 文案级）：L62 改"两个条件 oracle 上等价联合实施（marker 必伴 exit ≠ 0，wrapper 通过 exit code 单道门已覆盖）"或类似说法，避免读者期望 wrapper 内有 grep 实现。

---

### 候选 D-4：verifast / tool.toml comment 与 wrapper 实际信号不一致（文案）

**tool.toml comment**（`tools/verifast/tool.toml` L7-13）：
- 「post-checks for the vacuous-pass signature `"0 errors found (N statements verified)"` with N ≤ 40」

**wrapper 实际**（`tools/verifast/verifast-strict-wrapper.sh` L105-114）：
- 实际 grep `src/lib.rs(` 锚的存在性（user-file verbose line count），**不**用 N ≤ 40 阈值
- wrapper 自己的文件头注释（L19-23）也明确说 N alone 不够，已改用 src/lib.rs 锚

**不一致**：
- `tool.toml` L7-13 的 comment 还停留在旧 oracle "N ≤ 40 threshold"，wrapper 早已升级到 src/lib.rs 锚 grep
- README 与 wrapper 实际一致（L48-52 写"src/lib.rs( 锚"）；问题只在 tool.toml comment 没同步

**反状态排除**：
- R1 ✓ comment 字面 N ≤ 40 vs wrapper 实际 src/lib.rs 锚
- R2 ✗ 这不是 design contract——comment 是描述实施，已 stale
- R4 ✗
- R11 ✓ 仅 tool.toml comment 文案 stale，不影响 oracle 行为
- R12 反驳点：「tool.toml comment 是历史记录，wrapper 才是事实——读者应优先看 wrapper」——可接受，但 stale comment 误导

**建议**（低优先级 / 文案级）：tool.toml 头部 comment 同步成 src/lib.rs 锚的描述（已在 wrapper 头部充分阐述，tool.toml 只需简化引用 wrapper.sh）。

---

## 3. 总结

四个候选中 **D-1（ror）+ D-2（ror-typecheck）是真不一致**，wrapper 比 README 多一道 stderr "is not yet supported" gate；这是 P28 补 D3.1 时只改了 wrapper、没同步 README "门数表"导致的。wrapper 自己的注释（D-2 L116-126）已经显式承认"破声明"，建议优先补 README。

**D-3（prusti）+ D-4（verifast tool.toml comment）是文案级表述歧义** —— oracle 正确性不受影响。

未发现：
- 形式严格性 0 误报 / 0 漏报声明 vs 实际信号集合的根本矛盾（aeneas × 4 / kani / kmir / miri / cargo-check / charon × 2 / creusot / hax × 3 / soteria / verus 一致）
- partial 暴露机制 vs wrapper grep 信号集合的根本矛盾（aeneas × 4 已全部 4-pattern 对齐；kani 5 marker 完全对齐；hax × 3 silent-skip-item gate 在 tool.toml 内嵌实施，与 README 一致）
- timeout_secs / version_command 异常（未深查，本次 audit 不在重点）
- 缺 README / 缺 wrapper 文件——20 个工具齐全

**修复优先级**：D-1 + D-2 同批改（同源问题，3-5 行 README diff），D-3 + D-4 可在下次 README pass 顺手清理。
