// Creusot limitation: coercing a named function item to a bare function pointer
// produces a MIR PointerCoercion(ReifyFnPointer) cast, which Creusot cannot
// translate and emits "Unsupported cast: ... PointerCoercion(ReifyFnPointer, Implicit)".
//
// Source: Issue #1728 (Function pointers) — minimal reproduction given in the issue
//         creusot/src/translation/function/statement.rs – Unsupported pointer cast branch
//
// cargo check: passes (function pointer coercion is standard Rust)
// Creusot: crashes with "Unsupported cast: PointerCoercion(ReifyFnPointer, Implicit)"

fn add_one(x: u32) -> u32 {
    x + 1
}

pub fn get_fn_ptr() -> fn(u32) -> u32 {
    add_one
}
