use rand_chacha::ChaChaRng;
use rand::{RngCore, SeedableRng};

pub struct RandomNumber {
  pub gen: ChaChaRng,
}

impl RandomNumber {
  pub fn new() -> Self {
    let gen = ChaChaRng::from_entropy();
    RandomNumber { gen }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_1() {
    let mut r = RandomNumber::new();
    let mut buf = [0u8; 32];
    r.gen.fill_bytes(&mut buf);
  }
}