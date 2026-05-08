/// `thread_local!` macro — TLS storage. Charon's known-failure;
/// also stresses other verifiers' TLS / static-init handling.
thread_local!(static FOO: u32 = 0);

pub fn thread_local_read() {
    let v = FOO.with(|x| *x);
    let _ = v;
}
