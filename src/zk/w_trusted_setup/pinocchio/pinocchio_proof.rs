use crate::building_block::curves::bls12_381::{
  g1_point::G1Point,
  g2_point::G2Point,
};

pub struct PinocchioProof {
  pub v_mid: G1Point,
  pub w_mid_e1: G1Point,
  pub w_mid_e2: G2Point,
  pub y_mid: G1Point,
  pub beta_v_mid: G1Point,
  pub beta_w_mid_e1: G1Point,
  pub beta_y_mid: G1Point,
  pub h: G1Point,
  pub alpha_h: G1Point,
  pub ht: G1Point,
}

