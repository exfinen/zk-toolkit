use crate::building_block::{
  additive_identity::AdditiveIdentity,
  elliptic_curve::{
    affine_point::AffinePoint,
    weierstrass::curves::bls12_381::fq1::Fq1,
  },
  field::{
    field_elem_ops::Inverse,
    prime_field::PrimeField,
  },
  zero::Zero,
};
use std::ops::Add;

use super::bls12_381_g1::BLS12_381_G1;

#[derive(Clone)]
pub struct G1Point {
  pub curve: Box<BLS12_381_G1>,
  pub x: Fq1,
  pub y: Fq1,
}

impl AffinePoint for G1Point {
  type Element = Fq1;

  fn x(&self) -> Self::Element {
    self.x.clone()
  }

  fn y(&self) -> Self::Element {
    self.y.clone()
  }
}

impl Inverse for G1Point {
  fn inv(&self) -> Self {
    G1Point {
      curve: self.curve.clone(),
      x: self.x.clone(),
      y: self.y.inv(),
    }
  }
}

impl Zero<G1Point> for G1Point {
  fn get_zero(f: &G1Point) -> G1Point {
    G1Point {
      curve: f.curve.clone(),
      x: Fq1::get_zero(&f.x()),
      y: Fq1::get_zero(&f.x()),
    }
  }

  fn is_zero(&self) -> bool {
     self.x.is_zero() && self.y.is_zero()
  }
}

impl Add<G1Point> for G1Point {
  type Output = G1Point;

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
