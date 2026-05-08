/// Generic Associated Type — lifetime-parametrised `type Item<'a>`.
pub fn gat_lending() {
    trait LendIter {
        type Item<'a> where Self: 'a;
        fn lend<'a>(&'a mut self) -> Option<Self::Item<'a>>;
    }
    struct Slicer { data: Vec<i32>, idx: usize }
    impl LendIter for Slicer {
        type Item<'a> = &'a i32 where Self: 'a;
        fn lend<'a>(&'a mut self) -> Option<&'a i32> {
            if self.idx < self.data.len() {
                let r = &self.data[self.idx];
                self.idx += 1;
                Some(r)
            } else {
                None
            }
        }
    }
    let mut s = Slicer { data: vec![10, 20, 30], idx: 0 };
    while let Some(_) = s.lend() {}
}
