// Hax limitation: a closure (FnMut) that mutates bindings from an outer
// scope is not supported.
//
// Hax error: HAX0006 (ClosureMutatesParentBindings)
// "The bindings [...] cannot be mutated here: they don't belong to the
//  closure scope, and this is not allowed."
//
// Source: hax-types/src/diagnostics/mod.rs, Kind::ClosureMutatesParentBindings = 6
// Issue: https://github.com/hacspec/hax/issues/1060
//   "`FnMut` closures are not supported yet"
//   labels = [bug, engine, keep-open, unsupported-rust]
//
// In the issue: `self.k.take().map(|k| { self.y = 5; })` fails because
// the closure mutates `self.y`, a binding outside the closure's own scope.

pub fn closure_mutates_outer() -> u32 {
    let mut acc = 0u32;
    let v = [1u32, 2, 3];
    for x in v {
        // This FnMut closure captures and mutates `acc` from the enclosing scope.
        let _: () = (|| { acc += x; })();
    }
    acc
}
