use crate::building_block::curves::bls12_381::{
  g1_point::G1Point,
  g2_point::G2Point,
};

pub struct PinocchioProof {
  pub v_mid_s: G1Point,
  pub w_mid_s: G2Point,
  pub y_mid_s: G1Point,

  pub h_s: G2Point,

  pub alpha_v_mid_s: G1Point,
  pub alpha_w_mid_s: G1Point,
  pub alpha_y_mid_s: G1Point,

  pub beta_vwy_mid_s: G1Point,
}

