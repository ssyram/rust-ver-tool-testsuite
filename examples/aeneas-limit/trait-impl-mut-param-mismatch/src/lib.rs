// Aeneas limitation: when a generic trait is instantiated with a mutable
// reference (`&mut T`) as a type argument, the extracted Lean model of the
// trait and the extracted model of the impl disagree on the return type,
// producing invalid Lean code that does not type-check.
//
// The trait definition emits `Result Unit` for the method, but the concrete
// impl instantiated at `&mut u8` emits `Result Std.U8`, because Aeneas
// generates a "backward" (loan-giving) return value for mutable references
// in impls but not in the abstract trait dictionary.
//
// Source: https://github.com/AeneasVerif/aeneas/issues/961

pub trait Process<Arg> {
    fn run(arg: Arg);
}

pub struct Worker;

impl Process<&mut u8> for Worker {
    fn run(arg: &mut u8) {
        *arg += 1;
    }
}

pub fn use_process(_w: Worker, x: &mut u8) {
    Worker::run(x);
}

pub fn trigger_trait_impl_mut_param_mismatch() {
    let mut val: u8 = 0;
    use_process(Worker, &mut val);
}
