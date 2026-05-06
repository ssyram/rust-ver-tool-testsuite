/// Push two ints, pop them. Exercises Vec allocation, push/pop semantics, Drop.
pub fn push_pop_seq() {
    let mut v: Vec<i32> = Vec::new();
    v.push(1);
    v.push(2);
    let _ = v.pop();
    let _ = v.pop();
}
