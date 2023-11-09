// Implementation of protocol 2 described on page 5 in https://eprint.iacr.org/2013/279.pdf

use crate::{
  building_block::{
    curves::bls12_381::{
      g1_point::G1Point,
      g2_point::G2Point,
      pairing::Pairing,
    },
    zero::Zero,
    field::sparse_vec::SparseVec,
  },
  zk::w_trusted_setup::groth16::zktoolkit_based::{
    crs::CRS,
    proof::Proof,
  },
};

pub struct Verifier {
  pairing: Pairing,
}

impl Verifier {
  pub fn new(pairing: &Pairing) -> Self {
    Verifier {
      pairing: pairing.clone(),
    }
  }

  pub fn verify(
    &self,
    proof: &Proof,
    crs: &CRS,
    stmt_wires: &SparseVec,
  ) -> bool {
    let e = |a: &G1Point, b: &G2Point| self.pairing.tate(a, b);

    println!("--> Verifying Groth16 proof...");
    let lhs = e(&proof.A, &proof.B);

    let mut sum_term = G1Point::zero();
    for i in 0..stmt_wires.size_in_usize() {
      let ai = &stmt_wires[&i];
      sum_term += &crs.g1.uvw_stmt[i] * ai;
    }

    let rhs =
      &crs.gt.alpha_beta
      * e(&sum_term, &crs.g2.gamma)
      * e(&proof.C, &crs.g2.delta) 
      ;

    lhs == rhs
  }
}

