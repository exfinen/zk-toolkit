use crate::{
  building_block::curves::bls12_381::pairing::Pairing,
  zk::w_trusted_setup::pinocchio::{
    crs::CRS,
    pinocchio_proof::PinocchioProof,
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
  ) -> bool {
    let e = |a, b| self.pairing.tate(a, b);

    // e(E(αh(s)),E(1)) =? e(E(h(s)),E(α))
    if e(&proof.alpha_h, &crs.vk.one) != e(&proof.h, &crs.vk.e_alpha) {
      return false;
    }

    // e(E(βv v_mid(s), E(γ)) =? e(v_mid(s),E(βvγ)) 
   
    // e(E(βw w_mid(s)), E(γ)) =? e(w_mid(s),E(βwγ)) 
 
    // e(E(βy y_mid(s)), E(γ)) =? e(y_mid(s),E(βyγ))

    // v_e = E(v_0(s) + E(v_i/o(s)) + E(v_mid(s))
    // w_e = E(w_0(s) + E(w_i/o(s)) + E(w_mid(s))
    // y_e = E(y_0(s) + E(y_i/o(s)) + E(y_mid(s))
    // e(v_e, w_e)/e(y_e, E(1)) ?= e(E(h(s)), E(t(s)))
    
    true
  }
}


