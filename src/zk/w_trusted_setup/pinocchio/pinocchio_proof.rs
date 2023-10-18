use crate::building_block::curves::bls12_381::g1_point::G1Point;

pub struct PinocchioProof {
  pub v_mid: G1Point,
  pub w_mid: G1Point,
  pub y_mid: G1Point,
  pub beta_v_mid: G1Point,
  pub beta_w_mid: G1Point,
  pub beta_y_mid: G1Point,
  pub h: G1Point,
  pub alpha_h: G1Point,
}

