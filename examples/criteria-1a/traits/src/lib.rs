trait Trait<T> {
    fn foo(value: T) -> i32 {
        1
    }
}

struct Example;

impl Trait<&i32> for Example {
    fn foo(value: &i32) -> i32 {
        *value
    }
}

struct PureExample;

impl Trait<&i32> for PureExample {
    fn foo(value: &i32) -> i32 {
        *value
    }
}

fn main() {
    let x = 5;
    let y = 6;
}

