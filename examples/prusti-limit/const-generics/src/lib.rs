// Prusti limitation: const generics cause a panic in Prusti's borrow-checking
// fact collection.
//
// Functions or types parameterised by a const generic (`const N: usize`) cause
// `get_body_with_borrowck_facts` to panic inside Prusti, so the function body
// cannot be analysed at all. This is reported in issue #1195.
//
// Source: https://github.com/viperproject/prusti-dev/issues/1195

/// A fixed-capacity buffer backed by a const-generic array.
pub struct Buffer<const N: usize> {
    data: [u8; N],
    len: usize,
}

impl<const N: usize> Buffer<N> {
    pub fn new() -> Self {
        Buffer { data: [0u8; N], len: 0 }
    }

    /// Push one byte, returning `false` if the buffer is full.
    pub fn push(&mut self, byte: u8) -> bool {
        if self.len < N {
            self.data[self.len] = byte;
            self.len += 1;
            true
        } else {
            false
        }
    }
}

/// Return the number of elements currently stored.
pub fn buffer_len<const N: usize>(buf: &Buffer<N>) -> usize {
    buf.len
}

/// Zero-arg entry: instantiate `Buffer` at a concrete `const N`, exercise both
/// methods + the const-generic free function — touches every const-generic
/// site so Prusti's borrow-fact collection panics on this entry.
pub fn entry_const_generic_buffer() {
    let mut b: Buffer<4> = Buffer::new();
    let _ = b.push(1);
    let _ = b.push(2);
    let _ = buffer_len(&b);
}
