use crate::building_block::{
  curves::bls12_381::{
    fq12::Fq12,
    g1_point::G1Point,
    g2_point::G2Point,
  },
  zero::Zero,
};

pub struct RationalFunction {
  numerator: u32,
  denominator: u32,
}

impl RationalFunction {
  pub fn new(_a: &G1Point, _b: &G1Point) -> Self {
    RationalFunction {
      numerator: 0,
      denominator: 0,
    }
  }

  pub fn eval_at(&self, _p: &G2Point) -> Fq12 {
    Fq12::zero()
  }
}