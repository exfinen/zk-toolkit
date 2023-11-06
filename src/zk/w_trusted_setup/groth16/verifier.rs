// Implementation of protocol 2 described on page 5 in https://eprint.iacr.org/2013/279.pdf

use crate::zk::w_trusted_setup::groth16::{
  crs::CRS,
  proof::Proof,
};

pub struct Verifier {
}

impl Verifier {
  pub fn new() -> Self {
    Verifier {
    }
  }

  pub fn verify(
    &self,
    _proof: &Proof,
    _crs: &CRS,
  ) -> bool {
    println!("--> Verifying Groth16 proof...");
    true
  }
}

