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
    io_inputs: &SparseVec,
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
 
    println!("----> Checking if e(v_e, w_e)/e(y_e, E(1)) ?= e(E(h*t(s)), E(1))...");
    let f = &io_inputs.f;

    let mut v_e = &crs.vk.v_0 + &proof.v_mid;
    for i in 0..crs.vk.vi_io.len() {
      let w = &io_inputs[&f.elem(&i)];
      let p = &crs.vk.vi_io[i];
      v_e = v_e + p * w;
    }

    let mut w_e = &crs.vk.w_0 + &proof.w_mid_e2;
    for i in 0..crs.vk.wi_io.len() {
      let w = &io_inputs[&f.elem(&i)];
      let p = &crs.vk.wi_io[i];
      w_e = w_e + p * w;
    }

    let mut y_e = &crs.vk.y_0 + &proof.y_mid;
    for i in 0..crs.vk.yi_io.len() {
      let w = &io_inputs[&f.elem(&i)];
      let p = &crs.vk.yi_io[i];
      y_e = y_e + p * w;
    }

    let lhs = e(&v_e, &w_e) * e(&y_e, &crs.vk.one).inv();
    let rhs = e(&proof.ht, &crs.vk.one);

    lhs == rhs
  }
}


