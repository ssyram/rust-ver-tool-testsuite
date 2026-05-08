/// `union` — overlapping fields, requires unsafe to read.
pub fn repr_union() {
    #[repr(C)]
    union Reg {
        i: u32,
        f: f32,
    }
    let r = Reg { i: 0x40490FDB };
    let _bits = unsafe { r.i };
    let _f = unsafe { r.f };
}
