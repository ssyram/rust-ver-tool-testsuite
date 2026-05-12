---
name: tool-cc-report-rewrite
description: 单工具 cc-report 重写到当前 v6 baseline；按"我们导致" vs "工具不支持"分桶失败；产出修订建议
---

# tool-cc-report-rewrite

## 目的

重写 `deep-reports/cc-reports/<tool>.md` 一份工具 cc-report 到当前 v6 baseline。cc-report 不是 audit 报告——它是"该工具在本测试集上的能力评估 + 失败分桶 + 漏报盲点诚实声明"快照，每次 v 升级（v4→v5→v6）需重新生成。

输出**两件事**：

1. **重写后的 cc-report**（覆盖旧 v4 / v5 数据）
2. **修订建议清单**（仅"我们导致"失败需要修，"工具不支持"不修）

## 输入

调用本 skill 时必须提供：

- `<tool>`：工具目录名（如 `aeneas-lean`、`kani`、`prusti`）
- 隐含上下文：
  - 当前 v6 baseline: `runs/run-1778560393-59119/`
  - 旧 cc-report: `deep-reports/cc-reports/<tool>.md`（参考结构，重写不复用旧数据）
  - 工具自陈：`tools/<tool>/README.md` + `tools/<tool>/tool.toml` + `tools/<tool>/*-wrapper.sh`（若有）
  - 宪法 spec gate: `docs/design/principles.md`（v8 双根本问题 + UNKNOWN 严格语义）

## Spec gate（必读）

按 `/hoare-audit` Step 0 精神，rewrite 前必须读：

1. **`docs/design/principles.md`** — 全 140 行
   - §一 双根本问题（不公平 + 不公信 / 本地性 / 社区惯例 / 最大善意）
   - §四 原则 A / B / C
   - §六 Oracle 责任 + UNKNOWN 严格语义两类
2. **`docs/design/tool-integration.md`** — §四.5"我们 wrapper vs 官方 wrapper 失败归因"判据
3. **`tools/<tool>/README.md`** — 该工具自陈的形式严格性 / 0 误报 / 0 漏报 / 漏报盲点声明
4. **`tools/<tool>/tool.toml`** + 该工具 wrapper（如有）— 实际 oracle 行为

## 工作流

### Phase 1: 实测数据采集

```python
import json
with open("runs/run-1778560393-59119/results.json") as f:
    d = json.load(f)
rows = [r for r in d["results"] if r["tool"] == "<tool>"]
# 计算 S/F/U/total / 通过率 / 时长分布
```

或对应 Bash：

```bash
python3 -c "
import json
from collections import Counter
with open('runs/run-1778560393-59119/results.json') as f:
    d = json.load(f)
rows = [r for r in d['results'] if r['tool'] == '<tool>']
print(f'n={len(rows)}')
c = Counter(r['status'] for r in rows)
print(f'S/F/U: {dict(c)}')
durs = sorted(r['duration_ms'] for r in rows)
n = len(durs)
print(f'avg={sum(durs)//n} p50={durs[n//2]} p90={durs[(n*9)//10]} max={durs[-1]}')
"
```

工具版本字符串从 `d["tools"]` 取该工具条目的 `version` 字段。

### Phase 2: 失败分桶

把所有 FAILED entries 按 stderr / stdout 共性分桶：

```bash
RUN=runs/run-1778560393-59119
ls $RUN/raw/<tool>/*.exit | while read f; do
  if ! grep -q "Some(0)" "$f"; then
    base=${f%.exit}
    echo "=== $(basename $base) ==="
    head -3 "$base.stderr" 2>/dev/null
  fi
done | less
```

或抽样阅读几个 stderr 找模式，再 grep 全 161 看分布。

### Phase 3: 按"归因来源"分类每桶

按 `principles.md` §六 + `tool-integration.md` §四.5，把每个失败桶归一类：

| 归因 | 定义 | 处理 |
|---|---|---|
| **工具不支持** | 工具明示自陈 reject（如 prusti `[Prusti: unsupported feature]`）或工具自选 toolchain 拒绝 stable feature 或工具 pipeline 设计不读 deps 等"工具能力边界"| 不修。FAILED 站得住 |
| **官方 wrapper / driver crash** | 工具自带 wrapper（如 kmir `cargo.py`）/ 官方 driver panic | 不修。工具的锅，FAILED 站得住 |
| **我们 wrapper bug** | 项目维护的脚本（`prusti-strict-wrapper.sh` / `aeneas-*-wrapper.sh` / `rocq-of-rust-wrapper.sh` 等）IO 错 / 解析错 / shell 语法错 | **修**。属 §六 (b) 类 UNKNOWN，应升 UNKNOWN 而非 FAILED |
| **我们 corpus / 配置 bug** | hirusttest.toml / harness.rs.tera 错 / 我们引入的 vendored crate 触发 lint | **修**（或 oracle 加 (b) 子类规则） |
| **环境损坏** | binary 路径无效 / JVM crash / `/tmp` 清理 | **修**（治源 + 视情况加 (a) 类 UNKNOWN 规则）|
| **漏报候选** | wrapper 判 SUCCESS 但 stderr / stdout 含 partial 自陈 marker | **修**——加 wrapper grep gate 或 README 漏报盲点声明 |

### Phase 4: cc-report 重写

按下面骨架重写 `deep-reports/cc-reports/<tool>.md`（保留旧结构 / 更新数据 / 加 P31 时代的语义）：

```markdown
# <tool> — 特性支持评估报告（v6 baseline）

## 元数据

- **数据源**：`runs/run-1778560393-59119/`（2026-05-12 v6 final，合并 verus rerun + R7 5-tool rerun）
- **工具配置**：`tools/<tool>/`
- **工具版本**：（从 results.json tools 数组取 version 字段）
- **本工具实测**：n=161 / SUCCESS=<S> / FAILED=<F> / UNKNOWN=<U>，通过率 **<pct>%**
- **时长分布**：avg <a>ms / median <m>ms / p90 <p90>ms / max <max>ms
- **宪法 baseline**：`principles.md` v8 (P27 修宪后 / P31 法律传导后)
- **时效声明**：本快照锚定上述 run id + 工具版本 + corpus，不构成长期承诺。

## pipeline + 前端边界

（复用或更新旧 cc-report 该节；如 P27 后改动则同步描述。需明确：哪部分是工具本身、哪部分是项目维护的 wrapper、前端测量边界在哪里）

## SUCCESS 信号 + 形式严格性

按宪法 §六 双通路 partial 暴露：

- **主信号通路**：（exit code / 产物 grep / stderr marker）
- **wrapper 补抓通路**（若有）：（项目 wrapper 加的 gate）

形式严格性 0 误报 / 0 漏报状态：（按 P27 后宪法严格语义评估，避免"形式可证 / 漏报盲点：无"类过强声明，除非真有源码层证明）

## 失败分桶（按 P31 §四.5 归因分类）

### 桶 1：<工具不支持类型 X>（<N> case）

代表 entry：`<feature>/<dir>/<entry>`

stderr / stdout 特征：

```
<片段>
```

**归因**：工具不支持（如 inline asm / async fn / specific MIR construct）
**处理**：不修。本地性原则下 FAILED 站得住，工具开发者不能驳回。

### 桶 2：<我们导致类型 Y>（<N> case）

stderr / stdout 特征：（片段）

**归因**：我们的 wrapper bug / corpus 引入的 lint / 环境损坏 ...
**处理**：**修**。建议方案：（具体 fix 描述——加 oracle 规则 / 改 wrapper / 改 hirusttest schema / 等）

（继续列剩余桶）

## 漏报盲点（诚实声明）

- 已通过 wrapper gate 封堵：（列）
- 仍存在的盲点：（列 + 触发条件 + 修复 backlog）

## v5.1 → v6 ΔS 解释

（如该工具有 ΔS ≠ 0，逐项解释来源；如 ΔS = 0 写"v6 与 v5.1 通过率同"）

## 修订建议清单（仅"我们导致"失败）

| # | 桶 | 涉及 case | 修复方案 | 优先级 |
|---|---|---|---|---|
| 1 | （桶 2 的描述）| <N> | （具体 patch）| 高/中/低 |

如无任何"我们导致"失败：明确写"无需修订，所有 FAILED 均为工具能力边界"。
```

### Phase 5: 输出

写 `deep-reports/cc-reports/<tool>.md`（覆盖旧版本）。
回总结：

- 通过率 N/161（vs v5.1）
- 失败分桶概要
- **修订建议数 + 大类**（核心信号——下游汇总用）
- 该工具是否有"我们导致"的 fix 项

## 反模式

- **直接套用旧 v4/v5 cc-report 文本**：旧 cc-report 数据 + 分类逻辑都是 P27 前的。必须完全重测重分类
- **"工具不支持" + "我们导致" 混桶**：失去归因能力。每桶单一归因
- **过强自陈**："形式可证 0 漏报" 这种声明已被 P30 v6 cc audit 在多个工具上推翻——除非有完整源码层证明（如 cargo-check / miri），否则说"实测 + wrapper 双通路封堵"+ 列具体盲点
- **修"工具不支持"类**：违反宪法 §一 不公信。工具的能力边界就是它的能力边界，我们不替它修
- **不写漏报盲点**：违反宪法 §六 "不藏"
