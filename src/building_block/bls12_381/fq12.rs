use std::ops::{Add, Sub, Mul};
use crate::building_block::bls12_381::additional_ops::AdditionalOps;
use crate::building_block::bls12_381::fq6::Fq6;

pub struct Fq12 {
  w1: Fq6,
  w0: Fq6,
}

impl Fq12 {
  pub fn new(w1: Fq6, w0: Fq6) -> Self {
      Fq12 { w1, w0 }
  }
}

impl AdditionalOps for Fq12 {
  fn apply_reduce_rule(n: &Self) -> Self {
    panic!("Not implemented");
  }

  fn inv(n: &Self) -> Self {
    let factor = Self::inv(
      n.w0 * n.w0 - Self
    )
    Self {
    }
  }
}

impl Add<Fq12> for Fq12 {
  type Output = Fq12;

  fn add(self, rhs: Fq12) -> Self::Output {
    Fq1(self.0 + rhs.0)
  }
}

impl Sub<Fq12> for Fq12 {
  type Output = Fq12;

  fn sub(self, rhs: Fq12) -> Self::Output {
  }
}

impl Mul<Fq12> for Fq12 {
  type Output = Fq12;

  fn mul(self, rhs: Fq12) -> Self::Output {
  }
}