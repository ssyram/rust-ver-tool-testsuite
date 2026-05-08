/// `std::collections::HashMap` insert / get / contains.
pub fn hashmap_basic() {
    use std::collections::HashMap;
    let mut m: HashMap<String, i32> = HashMap::new();
    m.insert("a".to_string(), 1);
    m.insert("b".to_string(), 2);
    let _ = m.get("a").copied();
    let _ = m.contains_key("b");
    let _ = m.len();
}
