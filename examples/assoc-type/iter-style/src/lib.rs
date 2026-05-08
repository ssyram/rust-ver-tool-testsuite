/// Trait with associated type (Iterator-like).
pub fn assoc_type_iter() {
    trait MyIter {
        type Item;
        fn next_item(&mut self) -> Option<Self::Item>;
    }
    struct Counter { n: u32, max: u32 }
    impl MyIter for Counter {
        type Item = u32;
        fn next_item(&mut self) -> Option<u32> {
            if self.n < self.max {
                self.n += 1;
                Some(self.n)
            } else {
                None
            }
        }
    }
    let mut c = Counter { n: 0, max: 3 };
    while let Some(_) = c.next_item() {}
}
