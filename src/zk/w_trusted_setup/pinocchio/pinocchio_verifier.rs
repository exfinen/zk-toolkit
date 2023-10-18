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

    // e(E(βv_mid(s)) =? e(v_mid(s),E(βvγ)) 
    // e(E(βw_mid(s)) =? e(w_mid(s),E(βwγ)) 
    // e(E(βy_mid(s)) =? e(y_mid(s),E(βyγ))


    true
  }
}


