// vendor: git submodule RustCrypto/hashes @ ffe093984c004769747e998f77da8ff7c0e7a765 (tag sha2-v0.11.0)
// path dep: vendor/sha2/sha2 (sha2 subcrate inside hashes monorepo)
use sha2::{Sha256, Digest};

/// One-shot SHA-256 digest via the `Digest::digest` convenience method.
/// Tests that verifiers can model the generic Digest trait dispatch used
/// throughout the RustCrypto ecosystem.
pub fn sha256_digest_one_shot() {
    let data = b"the quick brown fox jumps over the lazy dog";
    let _ = Sha256::digest(data);
}

/// Incremental SHA-256: update in two chunks, then finalize.
/// Exercises the stateful hasher path and `GenericArray` output type.
pub fn sha256_digest_incremental() {
    let mut hasher = Sha256::new();
    hasher.update(b"the quick brown fox ");
    hasher.update(b"jumps over the lazy dog");
    let _ = hasher.finalize();
}
