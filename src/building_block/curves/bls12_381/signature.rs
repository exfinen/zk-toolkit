use crate::building_block::curves::bls12_381::{
  g1_point::G1Point,
  g2_point::G2Point,
  pairing::Pairing,
  private_key::PrivateKey,
};

pub struct Signer {
  g1: G1Point,
  pairing: Pairing,
}

impl Signer {
  pub fn new() -> Self {
    let g1 = G1Point::g();
    let pairing = Pairing::new();

    Signer {
      g1,
      pairing,
    }
  }

  // G1Point generator * sk
  pub fn gen_public_key(&self, sk: &PrivateKey) -> G1Point {
    &self.g1 * sk
  }

  pub fn sign(&self, m: &Vec<u8>, sk: &PrivateKey) -> G2Point {
    let hash_m = &G2Point::hash_to_g2point(m);
    hash_m * sk
  }

  pub fn verify(&self, m: &Vec<u8>, sig: &G2Point, pk: &G1Point) -> bool {
    let hash_m = &G2Point::hash_to_g2point(m);
    let lhs = self.pairing.tate(&self.g1, sig); 
    let rhs = self.pairing.tate(pk, hash_m); 
    lhs == rhs
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test() {
    let signer = Signer::new();

    let sk = &PrivateKey::new();
    let pk = &signer.gen_public_key(sk);

    let m = &b"chili crab".to_vec();
    let sig = &signer.sign(m, &sk);

    let is_valid_sig = signer.verify(m, sig, pk);
    assert!(is_valid_sig);
  }
}

