/// Recursive enum + match with guard clause.
pub fn nested_match_guard() {
    enum Tree {
        Leaf(i32),
        Node(Box<Tree>, Box<Tree>),
    }
    fn tree_sum(t: &Tree) -> i32 {
        match t {
            Tree::Leaf(v) if *v >= 0 => *v,
            Tree::Leaf(_) => 0,
            Tree::Node(l, r) => tree_sum(l) + tree_sum(r),
        }
    }
    let t = Tree::Node(
        Box::new(Tree::Leaf(3)),
        Box::new(Tree::Node(
            Box::new(Tree::Leaf(-1)),
            Box::new(Tree::Leaf(7)),
        )),
    );
    let _ = tree_sum(&t);
}
