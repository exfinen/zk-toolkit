use crate::building_block::{
  additive_identity::AdditiveIdentity,
  elliptic_curve::weierstrass::curves::bls12_381::fq1::Fq1,
  field::field_elem_ops::Inverse,
  zero::Zero,
};
use std::ops::Add;

#[derive(Clone)]
pub struct G1Point {
  pub curve: Box<Fq1>,
  pub x: Fq1,
  pub y: Fq1,
}

impl Inverse for G1Point {
  fn inv(&self) -> Self {
    G1Point {
      curve: self.curve.clone(),
      x: self.x.clone(),
      y: self.y.inc(),
    }
  }
}

impl Zero<G1Point> for G1Point {
  fn get_zero(f: &G1Point) -> G1Point {
    G1Point {
      curve: f.curve.clone(),
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

impl AdditiveIdentity<G1Point> for G1Point {
  fn get_additive_identity(&self) -> Self {
    G1Point {
      curve: self.curve.clone(),
      x: self.x.get_additive_identity(),
      y: self.x.get_additive_identity(),
    }
  }
}
