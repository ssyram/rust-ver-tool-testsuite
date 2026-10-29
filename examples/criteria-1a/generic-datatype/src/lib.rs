struct Foo<'a, T, const X: usize> {
    val: &'a [T; X]
}

trait Sum {
    fn sum(&self) -> isize;
}

impl<'a> Sum for Foo<'a, isize, 3> {
    fn sum(&self) -> isize {
        self.val[0] + self.val[1] + self.val[2]
    }
}
