// Implementation of protocol 2 described on page 5 in https://eprint.iacr.org/2013/279.pdf

use crate::{
  building_block::mcl::{
    mcl_fr::MclFr,
    mcl_g1::MclG1,
    mcl_g2::MclG2,
    mcl_sparse_vec::MclSparseVec,
    pairing::Pairing,
  },
  zk::w_trusted_setup::groth16::mcl_based::{
    crs::CRS,
    proof::Proof,
  },
};
use num_traits::Zero;

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
    stmt_wires: &MclSparseVec,
  ) -> bool {
    let e = |a: &MclG1, b: &MclG2| self.pairing.e(a, b);

    println!("--> Verifying Groth16 proof...");
    let lhs = e(&proof.A, &proof.B);

    let mut sum_term = MclG1::zero();
    for i in 0..stmt_wires.size.to_usize() {
      let ai = &stmt_wires[&MclFr::from(i)];
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

