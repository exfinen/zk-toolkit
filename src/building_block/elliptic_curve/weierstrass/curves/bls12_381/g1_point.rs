use crate::building_block::{
  additive_identity::AdditiveIdentity,
  elliptic_curve::{
    affine_point::AffinePoint,
    weierstrass::curves::bls12_381::fq1::Fq1,
    new_affine_point::NewAffinePoint,
  },
  zero::Zero,
};
use std::ops::Add;

#[derive(Clone)]
pub struct G1Point {
  pub x: Fq1,
  pub y: Fq1,
}

impl Zero<G1Point> for G1Point {
  fn get_zero(_f: &G1Point) -> G1Point {
    G1Point {
      x: Fq1::zero(),
      y: Fq1::zero(),
    }
  }

  fn is_zero(&self) -> bool {
     self.x.is_zero() && self.y.is_zero()
  }
}

impl Add<G1Point> for G1Point {
  fn add(self, rhs: G1Point) -> Self::Output {

  }
}

impl AdditiveIdentity for G1Point {
  fn get_additive_identity(&self) -> Self {
    G1Point {
      x: self.x.get_additive_identity(),
      y: self.x.get_additive_identity(),
    }
  }
}

impl AffinePoint<G1Point, Fq1> for G1Point {
  fn x(&self) -> Fq1 {
    self.x
  }

  fn y(&self) -> Fq1 {
    self.y
  }
}

impl NewAffinePoint<G1Point, Fq1> for G1Point {
  fn new(x: &Fq1, y: &Fq1) -> Self {
    G1Point {
      x: x.clone(),
      y: y.clone(),
    }
  }
}
