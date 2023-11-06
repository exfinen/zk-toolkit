use crate::building_block::curves::bls12_381::{
  g1_point::G1Point,
  g2_point::G2Point,
};

#[allow(non_snake_case)]
pub struct Proof {
  pub A: G1Point,
  pub B: G2Point,
  pub C: G1Point,
}

