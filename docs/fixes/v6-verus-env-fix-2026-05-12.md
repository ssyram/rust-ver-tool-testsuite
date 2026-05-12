# v6 verus 环境修复（2026-05-12）

## 现象

v6 重跑首次运行 verus 工具 161/161 entries 全 FAILED（exit 128），stderr 仅一行：

```
error: did not find a valid verusroot
```

## 根因

`TS_VERUS_BIN` 当时指向 `/tmp/ts-tools-install/verus-download/verus-arm64-macos/verus`。macOS 自动每周清理 `/tmp/` 中超过 3 天未访问的文件。verus binary 自身因被偶尔 access 而保留，但其依赖的辅助文件（`verus-root`、`.vstd-fingerprint`、`cargo-verus`、`version.json`、`version.txt`、`macos_allow_gatekeeper.sh`、`vstd.vir` 等）被清掉。

verus 启动时通过查找同目录的 `verus-root` 文件来确定其 sysroot；该文件缺失导致 "did not find a valid verusroot" 直接退出。

## 修复

把 verus 重装到 macOS 不会自动清理的位置：

```
~/.local/share/ts-tools/verus/verus-arm64-macos/
```

下载链接：
<https://github.com/verus-lang/verus/releases/download/release/0.2026.05.03.8b81855/verus-0.2026.05.03.8b81855-arm64-macos.zip>

`.env` 更新：

```diff
-TS_VERUS_BIN=/tmp/ts-tools-install/verus-download/verus-arm64-macos/verus
+TS_VERUS_BIN=${TS_TOOLS_BASE}/verus/verus-arm64-macos/verus
```

## 历史回声

此问题是 P22 prusti viper_tools 问题的重演——同样是 `/tmp/` 自动清理把工具支持文件吃掉，工具本体保留但跑不起来。prusti 当时也是从 `/tmp/` 迁移到 `~/.local/share/ts-tools/prusti/`。

## 项目影响

- v6 verus 数据废，重跑独立 run（`runs/run-1778561896-25488/`），合并入 v6 `runs/run-1778560393-59119/results.json`
- v6 verus 最终：66 SUCCESS / 95 FAILED / 0 UNKNOWN（与 v5.1 一致：66/73/22 — UNKNOWN 数差是 DP-4 严格化影响）

## 教训

凡是依赖 `/tmp/` 路径的 TS_*_BIN 都有 /tmp 自动清理风险。建议把所有工具安装迁移到 `~/.local/share/ts-tools/`。本次只迁了 verus + prusti，其他（charon / aeneas / soteria）当前 OK 但仍属定时炸弹。

未来 oracle 改进方向：加 runner 启动时的 binary 可执行性预检（`<TS_*_BIN> --version` 跑一次，失败则 fail-fast）。本次未做，因为：

1. 本次问题已通过迁移彻底治源（不再依赖 /tmp）
2. 加预检属 runner 增强，按宪法 §三 应在核心模块（runner）中实施，需独立 PR + design 讨论
3. 与本次 P27 修宪 + Oracle 严格化不属同一 commit
