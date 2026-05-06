/// Iterator chain: range → filter → map → collect.
/// Exercises lazy iterator chains, closure capture, FromIterator.
pub fn iter_chain_collect() {
    let v: Vec<i32> = (0i32..10)
        .filter(|x| *x % 2 == 0)
        .map(|x| x * 2)
        .collect();
    let _ = v.len();
}
