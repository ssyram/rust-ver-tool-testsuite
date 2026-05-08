/// `#[repr(C)]` struct — fixed layout for FFI.
pub fn repr_c_struct() {
    #[repr(C)]
    struct Point { x: i32, y: i32, z: i32 }
    let p = Point { x: 1, y: 2, z: 3 };
    let _ = (p.x, p.y, p.z);
    let size = std::mem::size_of::<Point>();
    let _ = size;
}
