// Aeneas limitation: a closure that both (a) captures a variable from the
// enclosing scope and (b) contains an if-then-else expression causes Aeneas
// to fail with "[Error] Unimplemented" when generating the `call` impl for
// the closure.
//
// The combination of captured state and conditional branching inside the
// closure body is the triggering pattern; either factor alone is handled.
//
// Source: https://github.com/AeneasVerif/aeneas/issues/924

pub fn make_closure_with_capture_and_branch(a: u64) {
    let _c = || {
        if true { a } else { a }
    };
}

pub fn trigger_closure_if_capture() {
    make_closure_with_capture_and_branch(42);
}
