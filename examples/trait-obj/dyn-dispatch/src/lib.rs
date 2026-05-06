/// Dynamic dispatch through `Box<dyn Trait>`.
/// Exercises trait objects, vtable, dynamic dispatch.
trait Greeter {
    fn greet(&self) -> i32;
}

struct A;
impl Greeter for A {
    fn greet(&self) -> i32 {
        1
    }
}

struct B;
impl Greeter for B {
    fn greet(&self) -> i32 {
        2
    }
}

pub fn dyn_dispatch() {
    let g1: Box<dyn Greeter> = Box::new(A);
    let g2: Box<dyn Greeter> = Box::new(B);
    let _ = g1.greet() + g2.greet();
}
