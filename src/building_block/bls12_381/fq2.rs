use std::ops::{Add, Sub, Mul};
use crate::building_block::bls12_381::additional_ops::AdditionalOps;
use crate::building_block::bls12_381::fq1::Fq1;

#[derive(Clone)]
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
      u1: &n.u1 + &n.u0,
      u0: &n.u0 - &n.u1,
    }
  }

  fn inv(n: &Self) -> Self {
    let factor = &(&n.u1 * &n.u1 + &n.u0 * &n.u0).inv();
    Self {
      u1: n.u1.negate() * factor,
      u0: &n.u0 * factor,
    }
  }

  fn zero() -> Self {
      Self {
        u1: Fq1::zero(),
        u0: Fq1::zero(),
      }
  }
}

macro_rules! impl_add {
  ($rhs: ty, $target: ty) => {
    impl<'a> Add<$rhs> for $target {
      type Output = Fq2;

      fn add(self, rhs: $rhs) -> Self::Output {
        Fq2 {
          u1: &self.u1 + &rhs.u1,
          u0: &self.u0 + &rhs.u0,
        }
      }
    }
  };
}
impl_add!(Fq2, Fq2);
impl_add!(Fq2, &Fq2);
impl_add!(&Fq2, Fq2);
impl_add!(&Fq2, &Fq2);

macro_rules! impl_sub {
  ($rhs: ty, $target: ty) => {
    impl<'a> Sub<$rhs> for $target {
      type Output = Fq2;

      fn sub(self, rhs: $rhs) -> Self::Output {
        Fq2 {
          u1: &self.u1 - &rhs.u1,
          u0: &self.u0 - &rhs.u0,
        }
      }
    }
  };
}
impl_sub!(Fq2, Fq2);
impl_sub!(Fq2, &Fq2);
impl_sub!(&Fq2, Fq2);
impl_sub!(&Fq2, &Fq2);

macro_rules! impl_mul {
  ($rhs: ty, $target: ty) => {
    impl<'a> Mul<$rhs> for $target {
      type Output = Fq2;

      fn mul(self, rhs: $rhs) -> Self::Output {
        Fq2 {
          u1: &self.u1 * &rhs.u0 + &self.u0 * &rhs.u1,
          u0: &self.u0 * &rhs.u0 - &self.u1 * &rhs.u1,
        }
      }
    }
  };
}
impl_mul!(Fq2, Fq2);
impl_mul!(Fq2, &Fq2);
impl_mul!(&Fq2, Fq2);
impl_mul!(&Fq2, &Fq2);
