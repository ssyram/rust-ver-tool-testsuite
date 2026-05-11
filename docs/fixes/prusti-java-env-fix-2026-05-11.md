# Prusti Java/Viper 环境修复记录（2026-05-11）

## 1. 现象

matrix `run-1778492036-50081` 中 prusti **0/161 全 FAILED**。所有 entry 的 stderr 均为同一 panic：

```
thread 'rustc' panicked at viper/src/ast_factory/ast_type.rs:130:73:
called `Result::unwrap()` on an `Err` value: JavaException
```

历史对比：P12 (`run-1778238662-69805`) 时 prusti **56/146 SUCCESS**。中间没有改 `.env` / `tool.toml` / wrapper，因此判定为**本机环境出问题**而非工具行为变化。

## 2. 诊断

按用户给出的诊断步骤逐项执行：

| 检查 | 结果 |
|---|---|
| `java --version` (PATH) | **Unable to locate a Java Runtime**（系统未装 system-wide JDK） |
| `$JAVA_HOME` | 空 |
| `/Library/Java/JavaVirtualMachines/` | 空目录 |
| `~/Library/Java/JavaVirtualMachines/` | 不存在 |
| `.env` 中 `TS_PRUSTI_JAVA_HOME` | `/tmp/ts-tools-install/jdk-x64/jdk-17.0.19+10/Contents/Home` |
| 上述 JDK 路径 `java -version` | **OK** — Temurin 17.0.19+10 x86_64（路径仍在，可执行） |

因此 `Unable to locate a Java Runtime` 并非 prusti 的实际问题——prusti 不走系统 java，走 `.env` 配的 x86_64 Temurin 17。

更细的诊断：跑单 entry 看 stderr 内容——prusti 启动后 JVM **被成功 spawn**，panic 出现在 `viper::ast_factory::AstFactory::backend_bv32_type` 调用一个 Java 方法时抛 `JavaException`。说明 JVM 在，但其加载的 Viper 类**缺失/不完整**。

进一步检查 `/tmp/ts-tools-install/prusti/viper_tools/` 内容：

```
viper_tools/
├── backends/         <- 空目录（应有 silicon.jar / carbon.jar）
├── boogie/Binaries/  <- 空目录（应有 Boogie.dll 等）
├── resources/sfx/    <- 空目录（应有音效资源）
└── z3/bin/z3         <- 仍在（单 binary）
```

而 `prusti/cargo-prusti`、`prusti-rustc`、`prusti-driver`、`libprusti_contracts.rlib` 仍在。

**根因**：macOS 周期性清理 `/tmp` 中老旧文件。z3 binary、cargo-prusti、prusti-rustc 等"新"些没被清，但 `viper_tools/backends/*.jar`、`boogie/Binaries/*` 等**子目录里的所有内容**被清掉，**留下空目录壳**。JVM 起来后调 `AstFactory.backend_bv32_type()` 走到 Viper 类→ 没有 jar → `ClassNotFoundException` → JNI 转 `JavaException` → unwrap panic。

因此本质是 **prusti 安装文件被 `/tmp` 清理破坏**，与 JDK 本身无关。

## 3. 修复路径

走的是用户列出的 **A+B 之外的第四条**：**重装 prusti release + 把 JDK 也迁到稳定位置**。理由：

- A（改 `.env` JDK 路径）：JDK 路径其实是好的，单改 `.env` 没用
- B（brew install openjdk@17）：brew openjdk@17 是 **arm64**，而 prusti v0.2.2 release 是 x86_64-only，需要 x86_64 JDK
- C（手动下 JDK）：JDK 已经有了，不缺
- **实际所需**：重新下载 prusti release，恢复被 `/tmp` 清掉的 `viper_tools/backends/*.jar` 等

### 3.1 下载并安装 prusti release 到稳定位置

```bash
mkdir -p ~/.local/share/ts-tools/prusti
cd ~/.local/share/ts-tools/prusti
curl -sL -o prusti-release-macos.zip \
  "https://github.com/viperproject/prusti-dev/releases/download/v-2023-08-22-1715/prusti-release-macos.zip"
unzip -q prusti-release-macos.zip
chmod +x cargo-prusti prusti-rustc prusti-driver prusti-server prusti-server-driver viper_tools/z3/bin/z3
xattr -dr com.apple.quarantine ~/.local/share/ts-tools/prusti
```

注：release tag `v-2023-08-22-1715` 对应 `tools/prusti/README.md` 锚定的 commit `a0681ee` 同日 build。

验证 `viper_tools/backends/viperserver.jar` 存在（被 `/tmp` 清掉的就是这个 jar）。

### 3.2 把 x86_64 JDK 也迁出 `/tmp`

`/tmp/ts-tools-install/jdk-x64/` 这次还在，但下次 `/tmp` 清理迟早再炸一次。一并迁移：

```bash
cp -R /tmp/ts-tools-install/jdk-x64 ~/.local/share/ts-tools/
```

### 3.3 更新 `.env`

```diff
-TS_PRUSTI_RUSTC=/tmp/ts-tools-install/prusti/prusti-rustc
-TS_CARGO_PRUSTI=/tmp/ts-tools-install/prusti/cargo-prusti
+TS_PRUSTI_RUSTC=/Users/ssyram/.local/share/ts-tools/prusti/prusti-rustc
+TS_CARGO_PRUSTI=/Users/ssyram/.local/share/ts-tools/prusti/cargo-prusti
 TS_PRUSTI_RUST_TOOLCHAIN_DIR=/Users/ssyram/.rustup/toolchains/nightly-2023-08-15-x86_64-apple-darwin
-TS_PRUSTI_JAVA_HOME=/tmp/ts-tools-install/jdk-x64/jdk-17.0.19+10/Contents/Home
+TS_PRUSTI_JAVA_HOME=/Users/ssyram/.local/share/ts-tools/jdk-x64/jdk-17.0.19+10/Contents/Home
```

## 4. 实测验证

### 4.1 单 entry

```bash
set -a && source .env && set +a
target/release/runner --tool prusti --entry 'hello/basic-hello/hello'
```

输出：

```
[SUCCESS] prusti hello/basic-hello/hello (7635ms)
Total: 1 succeeded / 0 failed / 0 unknown / 1 total
```

修复前是 `[FAILED] (1188ms, exit=101)` + `JavaException` panic；修复后 7.6 s 正常 SUCCESS（其中 JVM bootstrap + encoder + .vpr dump 走完整）。

### 4.2 全 matrix

```bash
target/release/runner --tool prusti
```

输出：

```
Total: 56 succeeded / 105 failed / 0 unknown / 161 total
run_dir: runs/run-1778494055-14621
```

| 项 | P12 baseline (`run-1778238662-69805`) | 修复后 (`run-1778494055-14621`) |
|---|---|---|
| SUCCESS | 56 / 146 | **56 / 161** |
| FAILED  | 90 / 146 | 105 / 161 |
| Δ entry 数 | — | +15（P12 后新增的样例） |

确认 zero JavaException：

```bash
grep -lE "JavaException" runs/run-1778494055-14621/raw/prusti/*.stderr | wc -l
# -> 0
```

修复后的 105 FAILED 全部是 prusti encoder 真实拒绝（unsupported feature / internal error / ICE），符合 prusti 前端边界定义；不再有"环境错"冒充工具失败。

## 5. 是否需要 oracle 层 UNKNOWN/FAILED 归档

按用户指示——是。本次修复后症状消失，但凡是"工具内部抛 JVM panic / 找不到依赖二进制"这类**环境故障**，runner 目前都会归类为 FAILED，这与宪法的 **0 误报**硬指标存在缝隙：环境故障被记为"工具拒绝该特性"，会把工具能力评低。

议题归档：建议在 `docs/fixes/oracle-environment-fault-2026-05-11.md` 单独立题，覆盖：

1. 哪些 panic / exit code pattern 应映射 UNKNOWN 而非 FAILED（如 `JavaException` / `dyld: Library not loaded` / `command not found`）
2. UNKNOWN 在 results.json + report.md 的呈现
3. tool.toml 是否暴露 `unknown_stderr_patterns`，由 runner 在 oracle 层统一识别

**本任务不实施，仅归档**——本任务的范围严格限定在"修 Java/viper 环境恢复 prusti 工作"。

## 6. 长期保护

- `~/.local/share/ts-tools/` 不在 macOS `/tmp` 清理范围，prusti 装在这里下次不会因为 `/tmp` 周期清理再坏
- 后续如果用户全清 `~/.local/share/ts-tools/`，按上述 §3.1-§3.3 步骤重装即可，无需重新调整 wrapper / tool.toml

## 7. 影响范围

- 修改：`.env`（路径迁出 `/tmp`）
- 不动：`.env.example`、`tool.toml`、`prusti-strict-wrapper.sh`、`docs/design/*`、其他工具
- 不 commit（用户指示）
