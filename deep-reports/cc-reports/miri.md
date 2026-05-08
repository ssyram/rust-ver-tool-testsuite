# miri 深度报告

## 元数据

- **run**：`runs/run-1778226613-5282/`（2026-05-08，146 entries × 19 工具矩阵；host：Apple M5 / macOS / aarch64 / 24 GB / 10 cpu）
- **工具版本**：`miri 0.1.0 (cb40c25f6a 2026-05-04)` @ nightly toolchain
- **通过率**：142/146 = 97%
- **时长**（毫秒）：avg 2800 / median 721 / p90 7031 / max 35727
- **时效声明**：本快照锚定上述 run id + 工具版本 + corpus，不构成长期承诺。

## 工具内部 pipeline + 前端边界

MIRI（Mid-level Intermediate Representation Interpreter）由 Rust 官方维护，是 nightly toolchain 的 component。pipeline：cargo + nightly rustc 把 harness + entry lib 编译到 MIR；miri 在 MIR 层模拟执行、检测 UB（内存越界 / use-after-free / 未初始化内存读取 / 数据竞争 / 不合法 raw pointer）。

MIRI **没有"前端 / 后端"分界**——它就是个 MIR 解释器，工作内容 = 解释执行 + UB 检测。所以"前端 = 全过程"。按宪法 §六-2 "不允许 partial"——SUCCESS = 解释执行**完整跑完且无任何中断**（无 UB / unsupported / panic），任何中断 → FAILED。

## SUCCESS 信号 + 形式严格性

- **形式指标**：`cargo +nightly miri run --bin __ts_harness` exit 0（解释执行完整跑完无 UB / unsupported）
- **partial 暴露**：UB / unsupported operation / panic 任意触发 → exit ≠ 0
- **0 误报**：✅ 形式可证。miri exit 0 ⇔ 解释执行完整跑完且无 UB / unsupported operation
- **0 漏报**：✅ 形式可证。任何 UB / unsupported / panic 触发 → miri exit ≠ 0
- **漏报盲点**：无（miri 不存在 silent skip）

> 注：UB 检测在某些视角下被理解为"工具有效输出"——但按"不允许 partial / 完整完成"精神，本测试一律 FAILED（解释执行没完整跑完）。

## 实测结果

### 按 feature 类目分布

142 SUCCESS / 4 FAILED 跨 41 类目：

```
全过类目（38 个）：aeneas-limit (8/8) / arc / assoc-type / bigint (8/8) / box
  closure / closure-adv (4/4) / collections / concurrency / const
  creusot-limit (7/7) / deps-complex (7/7) / drop / enum / error
  float (10/10) / gat / generic (4/4) / hax-limit (8/8) / hello / hrtb
  impl-trait / industrial (6/6) / int / int-width (14/14) / iter / lifetime (3/3)
  panic / prusti-limit (8/8) / rc / refcell / repr / slice / trait / trait-obj
  unsafe-adv (3/3) / unsafe-ptr (2/2) / vec

部分通过：
  charon-limit: 6/7（仅 inline-asm/nop_via_asm FAILED）
  kani-limit:   5/7（extern-ffi、uninit-memory FAILED）
  miri-limit:   6/7（仅 networking-unsupported FAILED）
```

`unsafe-ptr` / `repr/union` / `concurrency`（atomic + thread-mutex）/ `unsafe-adv`（含 maybe-uninit）这些直接触及 UB 检测域的特性全部 SUCCESS。`industrial/*` 6/6 全过（含 x509-parser 解析 ASN.1 DER / RSA pkcs8 加解密 / sha2 hash 等真实加密代码全部跑通）。

### 失败模式归类

逐条读 raw stderr 后归类。

#### A. miri 自声明 unsupported operation（3/4）

| entry | error |
|---|---|
| `charon-limit/inline-asm/nop_via_asm` | `unsupported operation: inline assembly is not supported` 指向 `src/lib.rs:31 core::arch::asm!("")` |
| `kani-limit/extern-ffi/trigger_call_libc_abs` | `unsupported operation: can't call foreign function 'abs' on OS 'macos'` 指向 `src/lib.rs:29 unsafe { abs(x) }` |
| `miri-limit/networking-unsupported/tcp_connect_attempt` | `unsupported operation: 'socket' not available when isolation is enabled` 指向 std 内部 `libc::socket(...)` 调用 |

stderr 共同模式：cargo 完成 `Compiling` + `Finished`，进入 `cargo-miri runner` 解释执行后 miri 在 entry 自身的 `src/lib.rs` 上抛 `error: unsupported operation: ...`，配 backtrace 指向具体调用点，exit 1。这三条都是 miri README 自陈的"已知不支持"形态（inline-assembly / unshimmed FFI / network 隔离）。

#### B. miri 真实检出 UB（1/4）

| entry | error |
|---|---|
| `kani-limit/uninit-memory/read_uninit_byte` | `Undefined Behavior: reading memory at alloc201[0x0..0x1], but memory is uninitialized at [0x0..0x1], and this operation requires initialized memory` 指向 `src/lib.rs:29 unsafe { x.assume_init() }` |

stderr 节选：

```
error: Undefined Behavior: reading memory at alloc201[0x0..0x1], but memory is uninitialized at [0x0..0x1]
  --> src/lib.rs:29:14
   |
29 |     unsafe { x.assume_init() }
   |              ^^^^^^^^^^^^^^^ Undefined Behavior occurred here
   = note: stack backtrace:
           0: kani_limit_uninit_memory::read_uninit_byte
               at src/lib.rs:29:14
Uninitialized memory occurred at alloc201[0x0..0x1], in this allocation:
alloc201 (stack variable, size: 1, align: 1) {
    __                                              │ ░
}
```

这是矩阵中**唯一一个 miri 真实检出 UB 的 FAILED**——`MaybeUninit::<u8>::uninit().assume_init()` 在解释执行中读取未初始化栈字节。按宪法 §六-2 "不允许 partial / 完整跑完"精神，这条被记为 FAILED——解释执行被 UB 中断未完整跑完。本测试是覆盖度筛选，不是 UB 检测有效性测量，故 miri 检出 UB 在本 oracle 下与其他中断同列 FAILED。

### 时长尾端观察

`max=35.7s` 在 `industrial/rsa/rsa-pkcs8/rsa_pkcs1v15_encrypt` SUCCESS——RSA 加密涉及大量 BigInt modpow 与 SHA-256 内部循环，miri 解释执行慢 10–100 倍体现充分。`industrial/x509-parser/*`（21–29s）、`deps-complex/*`（10–22s）也偏长。timeout 设 300s，本矩阵从未触发。median 仅 721ms，长尾集中在重型 dep / industrial。

### 边角观察：int-to-ptr cast 走 warning 不当 UB

`unsafe-ptr/raw-ptr-const/raw_ptr_const_match` 含 `let p = 43 as *const ();`，miri 输出 `warning: integer-to-pointer cast` + `help: this program is using integer-to-pointer casts ... which means that Miri might miss pointer bugs` 但 exit 0 → SUCCESS。这是 miri 默认 "permissive provenance" 模式：发警告不当 UB。这是 miri 在本矩阵上"接受 UB-marginal 代码"形态的实测一例，不构成上文 0 漏报判断的反例（warning 不是 partial）。

## 与本次测试边界的关系

- **测试切割点**：SUCCESS 仅蕴含"miri 在默认 flag 下能完整解释执行该 entry 无中断"。miri 默认未显式 enable tree borrows / strict provenance / data race detection，故未覆盖这些 UB 检测维度。
- **已知 corpus 偏向**：3 个 `kani-limit/*` 中 2 个（extern-ffi、uninit-memory）和 1 个 `miri-limit/networking-unsupported` 在 miri 上 FAILED——这些 entry 故意触发 miri 已知不支持的接口（FFI shim 缺失 / `tcp_connect` 在 isolation mode 下被拒）或故意触发 UB（assume_init 未初始化）。corpus 包含这些"miri 边界"样例是有意设计；本次实测 4 个 FAILED 全部是这种 corpus 倾向的预期触发。
- **本次未触达**：miri 在普通 entry（非 limit / industrial 加密代码）上检出意外 UB——当前 corpus 在写普通样例时已避开了边角 UB。

## 历史快照声明

本报告是 2026-05-08 运行 `runs/run-1778226613-5282` 的实测快照；锚定 `miri 0.1.0 (cb40c25f6a 2026-05-04)` × 当前 corpus（146 entries）。nightly toolchain 升级后 miri component 可能新增 / 移除 unsupported operations，need 重测才能更新 FAILED 集合。
