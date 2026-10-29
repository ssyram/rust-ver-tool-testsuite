trait X<const A: usize> {
    fn reverse(x: [i32; A]) -> [i32; A];
}

trait Y<const A: usize>: X<A> {
    fn foo();
}

struct Wrapper {
    val: [i32; 3]
}

impl X<3> for Wrapper {
    fn reverse(x: [i32; 3]) -> [i32; 3] {
        [x[2], x[1], x[0]]
    }
}

impl Y<3> for Wrapper {
    fn foo() {}
}
