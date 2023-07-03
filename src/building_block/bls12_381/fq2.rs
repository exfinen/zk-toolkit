use std::ops::{Add, Sub, Mul};
use crate::building_block::bls12_381::additional_ops::AdditionalOps;
use crate::building_block::bls12_381::fq1::Fq1;

pub struct Fq2 {
    u1: Fq1,
    u0: Fq1,
}

impl Fq2 {
    pub fn new(u1: Fq1, u0: Fq1) -> Self {
        Fq2 { u1, u0 }
    }
}

impl AdditionalOps for Fq2 {
  fn apply_reduce_rule(n: &Self) -> Self {
    Self {
      u1: n.u1 + n.u0,
      u0: n.u0 - n.u1,
    }
  }

  fn inv(n: &Self) -> Self {
    let factor = inv(
      n.u1 * n.u1 + n.u0 * n.u0
    );
    Self {
      u1: n.u1.negate() * factor,
      u0: n.u0 * factor,
    }
  }
}

impl Add<Fq2> for Fq2 {
  type Output = Fq2;

  fn add(self, rhs: Fq2) -> Self::Output {
    Fq2 {
      u1: self.u1 + rhs.u1,
      u0: self.u0 + rhs.u0,
    }
  }
}

impl Sub<Fq2> for Fq2 {
  type Output = Fq2;

  fn sub(self, rhs: Fq2) -> Self::Output {
    Fq2 {
      u1: self.u1 - rhs.u1,
      u0: self.u0 - rhs.u0,
    }
  }
}

impl Mul<Fq2> for Fq2 {
  type Output = Fq2;

  fn mul(self, rhs: Fq2) -> Self::Output {
    Fq2 {
      u1: self.u1 * rhs.u0 + self.u0 * rhs.u1,
      u0: self.u0 * rhs.u0 - self.u1 * rhs.u1,
    }
  }
}