use std::hash::{Hash, Hasher};
use twox_hash::XxHash3_128;

pub struct Xxh3Hasher128(XxHash3_128);

impl Default for Xxh3Hasher128 {
  fn default() -> Self {
    Self(XxHash3_128::new())
    // or Xxh3::with_seed(seed) for domain separation
  }
}

impl Hasher for Xxh3Hasher128 {
  fn write(&mut self, bytes: &[u8]) {
    self.0.write(bytes); // stream bytes, no allocation
  }
  // Hasher requires a u64 result; return the 64-bit XXH3 if you want,
  // or the low 64 bits of the 128-bit digest.
  fn finish(&self) -> u64 {
    // digest* usually consumes; clone the small state to compute without mutating
    self.0.finish_128() as u64
  }
}

impl Xxh3Hasher128 {
  pub fn finish_u128(self) -> u128 {
    // consume the state to produce the 128-bit digest
    self.0.finish_128()
  }
}

// Helper for any T: Hash
pub fn one_shot_128<T: Hash>(value: &T) -> u128 {
  let mut h = Xxh3Hasher128::default();
  value.hash(&mut h);
  h.finish_u128()
}

// Helper for any T: Hash
pub fn one_shot_64<T: Hash>(value: &T) -> u64 {
  let mut h = Xxh3Hasher128::default();
  value.hash(&mut h);
  h.finish()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn hashes_strings() {
    let a = one_shot_128(&"hello");
    let b = one_shot_128(&"hello");
    let c = one_shot_128(&"world");
    assert_eq!(a, b);
    assert_ne!(a, c);
  }

  #[test]
  fn hashes_structs() {
    #[derive(Hash)]
    struct S { x: u32, y: String }
    let h1 = one_shot_128(&S { x: 1, y: "a".into() });
    let h2 = one_shot_128(&S { x: 1, y: "a".into() });
    assert_eq!(h1, h2);
  }
}
