use crate::building_block::curves::bls12_381::g1_point::G1Point;

pub struct PinocchioProof {
  pub v: G1Point,
  pub w: G1Point,
  pub y: G1Point,
  pub beta_v: G1Point,
  pub beta_w: G1Point,
  pub beta_y: G1Point,
  pub h: G1Point,
  pub h_alpha: G1Point,
}

