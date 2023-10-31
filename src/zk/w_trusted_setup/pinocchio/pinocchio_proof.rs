use crate::building_block::curves::bls12_381::{
  g1_point::G1Point,
  g2_point::G2Point,
};

pub struct PinocchioProof {
  pub g_v_v_mid_s: G1Point,
  pub g1_w_w_mid_s: G1Point,
  pub g2_w_w_mid_s: G2Point,
  pub g_y_y_mid_s: G1Point,

  pub g_h_s: G2Point,

  pub g_v_alpha_v_mid_s: G1Point,
  pub g_w_alpha_w_mid_s: G1Point,
  pub g_y_alpha_y_mid_s: G1Point,

  pub g_beta_vwy_mid_s: G1Point,
}

