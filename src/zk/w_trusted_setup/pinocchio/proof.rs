// Implementation of protocol 2 described on page 5 in https://eprint.iacr.org/2013/279.pdf

use crate::building_block::curves::bls12_381::{
  g1_point::G1Point,
  g2_point::G2Point,
};

pub struct Proof {
  pub v_mid_s: G1Point,
  pub g1_w_mid_s: G1Point,
  pub g2_w_mid_s: G2Point,
  pub y_mid_s: G1Point,
  pub h_s: G2Point,
  pub alpha_v_mid_s: G1Point,
  pub alpha_w_mid_s: G1Point,
  pub alpha_y_mid_s: G1Point,
  pub beta_vwy_mid_s: G1Point,
}

