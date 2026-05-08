/// u32 bitwise operations: &, |, ^, !, <<, >>.
/// Also includes count_ones, count_zeros, leading_zeros, trailing_zeros which
/// expose whether a verifier models bit-level structure or only arithmetic.
pub fn int_width_bit_ops_u32() {
    let a: u32 = 0b1010_1010_u32;
    let b: u32 = 0b1100_1100_u32;

    let _and: u32 = a & b;   // 0b1000_1000
    let _or:  u32 = a | b;   // 0b1110_1110
    let _xor: u32 = a ^ b;   // 0b0110_0110
    let _not: u32 = !a;       // 0b...0101_0101 (complement all 32 bits)

    // Shifts (safe amounts)
    let _shl: u32 = a << 4;  // 0b1010_1010_0000
    let _shr: u32 = a >> 2;  // 0b0010_1010

    // Rotate
    let _rot_l: u32 = a.rotate_left(4);
    let _rot_r: u32 = a.rotate_right(4);

    // Bit counting — requires verifier to model Hamming weight
    let _ones: u32  = a.count_ones();   // 4
    let _zeros: u32 = a.count_zeros();  // 28
    let _lz: u32    = a.leading_zeros();
    let _tz: u32    = a.trailing_zeros(); // 1

    // Mask and test a specific bit
    let _bit3: bool = (a & (1_u32 << 3)) != 0; // bit 3 of 0b1010_1010 = 1
}
