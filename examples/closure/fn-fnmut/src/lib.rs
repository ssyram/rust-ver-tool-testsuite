/// Fn closure capturing immutable env.
pub fn closure_fn() {
    let x = 10i32;
    let add_x = |y: i32| x + y;
    let _ = add_x(5);
}

/// FnMut closure mutating captured state across multiple calls.
pub fn closure_fnmut() {
    let mut count = 0i32;
    let mut incr = || {
        count += 1;
    };
    incr();
    incr();
    let _ = count;
}
