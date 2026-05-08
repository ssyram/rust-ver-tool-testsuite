/// Cross-width casts involving 64-bit and 128-bit types.
/// i64 -> i128 (sign-extend), u64 -> u128 (zero-extend), and the reverse
/// truncating casts.  The 64->128 direction is lossless; 128->64 truncates.
/// Verifiers must correctly model 128-bit register width in the wider casts.
pub fn int_width_cast_i64_u64_i128() {
    // i64 -> i128: sign-extension, lossless
    let a: i64 = i64::MAX;
    let _w1: i128 = a as i128;        // 2^63 - 1, exact

    let b: i64 = i64::MIN;
    let _w2: i128 = b as i128;        // -2^63, exact

    let neg: i64 = -1_i64;
    let _w3: i128 = neg as i128;      // -1 (sign-extended, not 2^64-1)

    // u64 -> u128: zero-extension, lossless
    let c: u64 = u64::MAX;
    let _w4: u128 = c as u128;        // 2^64 - 1, exact
    let _w5: u128 = 1_u64 as u128;   // 1

    // i128 -> i64: truncating (loses high 64 bits)
    let big: i128 = (i64::MAX as i128) + 1_i128; // 2^63, overflows i64
    let _t1: i64 = big as i64;        // i64::MIN (0x8000...0000)

    // u128 -> u64: truncating
    let ubig: u128 = (u64::MAX as u128) + 1_u128; // 2^64
    let _t2: u64 = ubig as u64;       // 0

    // isize / usize on aarch64 are 64-bit; verify round-trip with i64
    let s: isize = isize::MAX;
    let _r: i64 = s as i64;           // same bits on 64-bit platform
    let _back: isize = (_r) as isize; // round-trip
}
