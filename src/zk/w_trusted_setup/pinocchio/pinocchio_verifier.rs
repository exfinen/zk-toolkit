use crate::{
  building_block::curves::bls12_381::pairing::Pairing,
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
    let e = |a, b| self.pairing.tate(a, b);

    // println!("----> Checking if e(E(αh(s)),E(1)) =? e(E(h(s)),E(α))...");
    // if e(&proof.alpha_h, &crs.vk.one) != e(&proof.h, &crs.vk.e_alpha) {
    //   return false;
    // }
    // 
    // println!("----> Checking if e(E(βv v_mid(s), E(γ)) =? e(v_mid(s),E(βvγ))..."); 
    // if e(&proof.beta_v_mid, &crs.vk.e_gamma) != e(&proof.v_mid, &crs.vk.beta_v_gamma) {
    //   return false;
    // }
    // 
    // println!("----> Checking if e(E(βw w_mid(s)), E(γ)) =? e(w_mid(s),E(βwγ))..."); 
    // if e(&proof.beta_w_mid_e1, &crs.vk.e_gamma) != e(&proof.w_mid_e1, &crs.vk.beta_w_gamma) {
    //   return false;
    // }
    // 
    // println!("----> Checking if e(E(βy y_mid(s)), E(γ)) =? e(y_mid(s),E(βyγ))...");
    // if e(&proof.beta_y_mid, &crs.vk.e_gamma) != e(&proof.y_mid, &crs.vk.beta_y_gamma) {
    //   return false;
    // }
 
    println!("----> Checking if e(v_e, w_e)/e(y_e, E(1)) ?= e(E(h(s)), E(t(s)))...");
    let f = &witness_io.f;

    let mut v_e = proof.v_mid.clone();
    for i in 0..crs.vk.vi_io.len() {
      let w = &witness_io[&f.elem(&i)];
      let p = &crs.vk.vi_io[i];
      v_e = v_e + p * w;
    }

    let mut w_e = proof.w_mid_e2.clone();
    for i in 0..crs.vk.wi_io.len() {
      let w = &witness_io[&f.elem(&i)];
      let p = &crs.vk.wi_io[i];
      w_e = w_e + p * w;
    }

    let mut w_e_e1 = proof.w_mid_e1.clone();
    for i in 0..crs.vk.wi_io.len() {
      let w = &witness_io[&f.elem(&i)];
      let p = &crs.vk.wi_io_e1[i];
      w_e_e1 = w_e_e1 + p * w;
    }

    let mut y_e = proof.y_mid.clone();
    for i in 0..crs.vk.yi_io.len() {
      let w = &witness_io[&f.elem(&i)];
      let p = &crs.vk.yi_io[i];
      y_e = y_e + p * w;
    }

    // hz = h(s) + v(s) + w(s) + t(s) - 1
    let hz = &proof.h + &v_e + &w_e_e1 + &crs.vk.t_e1 + &crs.vk.one_e1.inv();

    let lhs = e(&v_e, &w_e) * e(&y_e, &crs.vk.one).inv();
    let rhs = e(&hz, &crs.vk.t);

    lhs == rhs
  }
}


