//! Miri limitation: networking syscalls have no shims and are not supported.
//!
//! Source: https://github.com/rust-lang/miri/blob/master/README.md
//! "Miri runs the program as a platform-independent interpreter, so the
//! program has no access to most platform-specific APIs or FFI. A few APIs
//! have been implemented (such as printing to stdout, accessing environment
//! variables, and basic file system access) but most have not: for example,
//! Miri currently does not support networking."
//!
//! When code calls `std::net::TcpStream::connect`, Miri reaches the underlying
//! `connect(2)` / `bind(2)` syscall shim that is absent, and reports:
//!   error: unsupported operation: can't call foreign function: bind
//!   (or `connect`)
//!
//! Triggered aspect: constructing a `TcpStream` causes Miri to invoke the
//! `connect` system-call path for which no interpreter shim exists.

use std::net::TcpStream;

/// Attempts a TCP connection to localhost on an ephemeral port.
///
/// Under Miri this immediately hits an unsupported-operation error because
/// the `connect` syscall is not shimmed.  Under a real runtime the call will
/// fail with a "connection refused" OS error (no server is listening), which
/// is fine — the important observable fact is that the code compiles and the
/// networking path is exercised.
pub fn tcp_connect_attempt() -> bool {
    TcpStream::connect("127.0.0.1:19999").is_ok()
}
