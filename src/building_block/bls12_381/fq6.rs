use std::ops::{Add, Sub, Mul};
use crate::building_block::bls12_381::additional_ops::AdditionalOps;
use crate::building_block::bls12_381::fq2::Fq2;

pub struct Fq6 {
  v2: Fq2,
  v1: Fq2,
  v0: Fq2,
}

impl AdditionalOps for Fq6 {
  fn apply_reduce_rule(n: &Self) -> Self {
    Self {
      v2: n.v1,
      v1: n.v0,
      v0: Self::apply_reduce_rule(n.v2),
    }
  }

  fn inv(n: &Self) -> Self {
    let t0 = n.v0 * n.v0 - Self::apply_reduce_rule(n.v1 * n.v2);
    let t1 = Self::apply_reduce_rule(n.v2 * n.v2) - n.v0 * n.v1;
    let t2 = n.v1 * n.v1 - n.v0 * n.v2;
    let factor = Self::inv (
      n.v0 * t0
      + Self::apply_reduce_rule(n.v2 * t1)
      + Self::apply_reduce_rule(n.v1 * t2)
    );
    Self {
      v2: t2 * factor,
      v1: t1 * factor,
      v0: t0 * factor,
    }
  }
}

impl Fq6 {
  pub fn new(v2: Fq2, v1: Fq2, v0: Fq2) -> Self {
    Fq6 { v2, v1, v0 }
  }
}


impl Add<Fq6> for Fq6 {
  type Output = Fq6;

  fn add(self, rhs: Fq6) -> Self::Output {
    Fq6 {
      v2: self.v2 + rhs.v2,
      v1: self.v1 + rhs.v1,
      v0: self.v0 + rhs.v0,
    }
  }
}

impl Sub<Fq6> for Fq6 {
  type Output = Fq6;

  fn sub(self, rhs: Fq6) -> Self::Output {
    Fq6 {
      v2: self.v2 - rhs.v2,
      v1: self.v1 - rhs.v1,
      v0: self.v0 - rhs.v0,
    }
  }
}

impl Mul<Fq6> for Fq6 where Fq6: AdditionalOps {
  type Output = Fq6;

  fn mul(self, rhs: Fq6) -> Self::Output {
    let t0 = self.v0 * rhs.v0;
    let t1 = self.v0 * rhs.v1 + self.v1 * rhs.v0;
    let t2 = self.v0 * rhs.v2 + self.v1 * rhs.v1 + self.v2 * rhs.v0;
    let t3 = Self::apply_reduce_rule(self.v1 * rhs.v2 + self.v2 * rhs.v1);
    let t4 = Self::apply_reduce_rule(self.v2 * rhs.v2);
    Fq6 {
      v2: t2,
      v1: t1 + t4,
      v0: t0 + t3,
    }
  }
}
