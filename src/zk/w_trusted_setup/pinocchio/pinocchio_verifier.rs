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
    public_io_inputs: &SparseVec,
  ) -> bool {
    let e = |a, b| self.pairing.tate(a, b);

    // e(E(αh(s)),E(1)) =? e(E(h(s)),E(α))
    // if e(&proof.alpha_h, &crs.vk.one) != e(&proof.h, &crs.vk.e_alpha) {
    //   return false;
    // }

    // e(E(βv v_mid(s), E(γ)) =? e(v_mid(s),E(βvγ)) 
    // if e(&proof.beta_v_mid, &crs.vk.e_gamma) != e(&proof.v_mid, &crs.vk.beta_v_gamma) {
    //   return false;
    // }
   
    // e(E(βw w_mid(s)), E(γ)) =? e(w_mid(s),E(βwγ)) 
    // if e(&proof.beta_w_mid, &crs.vk.e_gamma) != e(&proof.w_mid, &crs.vk.beta_w_gamma) {
    //   return false;
    // }
 
    // e(E(βy y_mid(s)), E(γ)) =? e(y_mid(s),E(βyγ))
    // if e(&proof.beta_y_mid, &crs.vk.e_gamma) != e(&proof.y_mid, &crs.vk.beta_y_gamma) {
    //   return false;
    // }

    // v_e = E(v_0(s) + E(v_i/o(s)) + E(v_mid(s))
    // w_e = E(w_0(s) + E(w_i/o(s)) + E(w_mid(s))
    // y_e = E(y_0(s) + E(y_i/o(s)) + E(y_mid(s))
    // e(v_e, w_e)/e(y_e, E(1)) ?= e(E(h(s)), E(t(s)))

    let f = &public_io_inputs.f;

    println!("pub input: {}", public_io_inputs.size);
    println!("io: {}", crs.vk.vi_io.len());

    let mut v_e = crs.vk.v_0.clone();
    for i in 0..crs.vk.vi_io.len() {
      let w = &public_io_inputs[&f.elem(&i)];
      let vi = &crs.vk.vi_io[i];
      v_e = v_e + vi * w;
    }
    
    let mut w_e = crs.vk.w_0.clone();
    for i in 0..crs.vk.wi_io.len() {
      let w = &public_io_inputs[&f.elem(&i)];
      let wi = &crs.vk.wi_io[i];
      w_e = w_e + wi * w;
    }

    let mut y_e = crs.vk.y_0.clone();
    for i in 0..crs.vk.yi_io.len() {
      let w = &public_io_inputs[&f.elem(&i)];
      let yi = &crs.vk.yi_io[i];
      y_e = y_e + yi * w;
    }

    let lhs = e(&v_e, &w_e) - e(&y_e, &crs.vk.one);
    let rhs = e(&proof.h, &crs.vk.t);

    lhs == rhs
  }
}


