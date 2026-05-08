// Aeneas limitation: bitwise operators (&, |, ^) applied to `bool` operands
// are not handled by Aeneas's binary-operation translation.
//
// Rust allows `a & b` on booleans (non-short-circuit AND); in LLBC this becomes
// a `BinOp` with operand type `bool`.  Aeneas's extractor only recognises
// bitwise binops for integer types, so it errors with:
//   "[Error] Invalid inputs for binop"
//
// Source: https://github.com/AeneasVerif/aeneas/issues/965

pub fn bitwise_and_bool(a: bool, b: bool) -> bool {
    a & b
}

pub fn bitwise_or_bool(a: bool, b: bool) -> bool {
    a | b
}

pub fn bitwise_xor_bool(a: bool, b: bool) -> bool {
    a ^ b
}

pub fn trigger_bool_bitwise_op() {
    let _ = bitwise_and_bool(true, false);
    let _ = bitwise_or_bool(false, true);
    let _ = bitwise_xor_bool(true, true);
}
