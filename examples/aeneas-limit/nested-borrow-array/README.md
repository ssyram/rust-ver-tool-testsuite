# nested-borrow-array

**Limitation:** Aeneas cannot handle nested borrows that arise when references are
stored in an array and then indexed back out into a lifetime-bearing return type.

## Trigger

```rust
pub struct H<T>(pub T);
impl<T> H<T> {
    pub fn find_max(&self) -> Option<&T> {
        let mut max: Option<&T> = None;
        let arr = [&self.0];          // array-of-references
        for i in 0..1usize {
            max = Some(arr[i]);       // nested borrow: &T under arr's borrow
        }
        max
    }
}
```

## Aeneas error

```
[Error] Found a case of unsupported nested borrows
```

## Why

Aeneas represents borrows symbolically.  When a `&T` is stored inside `[&T; N]`
the resulting LLBC has a projection through an array that itself lives behind a
borrow, creating a borrow-under-borrow ("nested borrow") structure that the
symbolic interpreter does not yet support.

## Source

<https://github.com/AeneasVerif/aeneas/issues/929>
