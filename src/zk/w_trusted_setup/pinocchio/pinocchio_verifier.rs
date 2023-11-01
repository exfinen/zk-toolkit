use crate::{
  building_block::{
    curves::bls12_381::{
      g1_point::G1Point,
      g2_point::G2Point,
      pairing::Pairing,
    },
    zero::Zero,
  },
  zk::w_trusted_setup::pinocchio::{
    crs::CRS,
    pinocchio_proof::PinocchioProof,
    sparse_vec::SparseVec,
  },
};

pub struct PinocchioVerifier {
  pairing: Pairing,
}

impl PinocchioVerifier {
  pub fn new() -> Self {
    let pairing = Pairing::new();

    PinocchioVerifier {
      pairing,
    }
  }

  pub fn verify(
    &self,
    proof: &PinocchioProof,
    crs: &CRS,
    witness_io: &SparseVec,
  ) -> bool {
    println!("--> Verifying Pinnochio proof...");
    let e = |a: &G1Point, b: &G2Point| self.pairing.tate(a, b);

    let (p, vk) = (&proof, &crs.vk); 

    // KC of v * w * y
    {
      let vwd_mid_s = &p.v_mid_s + &p.g1_w_mid_s + &p.y_mid_s;
      let lhs = e(&p.beta_vwy_mid_s, &vk.g_gamma);
      let rhs = e(&vwd_mid_s, &vk.g_beta_gamma);
      if lhs != rhs { return false; }
    }

    // KC of v, w and y
    {
      let lhs = e(&p.alpha_v_mid_s, &vk.one_g2);
      let rhs = e(&p.v_mid_s, &vk.g_alpha_v); 
      if lhs != rhs { return false; }
    }
    {
      let lhs = e(&p.alpha_w_mid_s, &vk.one_g2);
      let rhs = e(&p.g1_w_mid_s, &vk.g2_alpha_w); 
      if lhs != rhs { return false; }
    }
    {
      let lhs = e(&p.alpha_y_mid_s, &vk.one_g2);
      let rhs = e(&p.y_mid_s, &vk.g2_alpha_y); 
      if lhs != rhs { return false; }
    }
     
    // QAP divisibility check
    {
      let mut v_io: G1Point = G1Point::zero();
      let mut w_io: G2Point = G2Point::zero();
      let mut y_io: G1Point = G1Point::zero();

      for i in 0..witness_io.size_in_usize() {
        let w = &witness_io[&i];
        v_io = v_io + &vk.g_v_v_k_io[i] * w;
        w_io = w_io + &vk.g_w_w_k_io[i] * w;
        y_io = y_io + &vk.g_y_y_k_io[i] * w;
      }

      let v_s = &v_io + &p.v_mid_s;
      let w_s = &w_io + &p.g2_w_mid_s;
      let y_s = &y_io + &p.y_mid_s;

      let lhs = e(&v_s, &w_s) ;
      let rhs = e(&vk.g_y_t, &p.h_s) * e(&y_s, &vk.one_g2);

      lhs == rhs
    }
  }
}

