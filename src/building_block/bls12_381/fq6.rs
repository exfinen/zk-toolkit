use std::ops::{Add, Sub, Mul, Neg};
use crate::building_block::bls12_381::additional_ops::AdditionalOps;
use crate::building_block::bls12_381::fq2::Fq2;

#[derive(Debug, Clone)]
pub struct Fq6 {
  v2: Fq2,
  v1: Fq2,
  v0: Fq2,
}

impl AdditionalOps for Fq6 {
  fn apply_reduce_rule(n: &Self) -> Self {
    Self {
      v2: n.v1.clone(),
      v1: n.v0.clone(),
      v0: Fq2::apply_reduce_rule(&n.v2),
    }
  }

  fn inv(n: &Self) -> Self {
    let t0 = &n.v0 * &n.v0 - Fq2::apply_reduce_rule(&(&n.v1 * &n.v2));
    let t1 = Fq2::apply_reduce_rule(&(&n.v2 * &n.v2)) - &n.v0 * &n.v1;
    let t2 = &n.v1 * &n.v1 - &n.v0 * &n.v2;
    let factor = Fq2::inv(&(
      &n.v0 * &t0
      + Fq2::apply_reduce_rule(&(&n.v2 * &t1))
      + Fq2::apply_reduce_rule(&(&n.v1 * &t2))
    ));
    Self {
      v2: &t2 * &factor,
      v1: &t1 * &factor,
      v0: &t0 * &factor,
    }
  }

  fn zero() -> Self {
      Self {
        v2: Fq2::zero(),
        v1: Fq2::zero(),
        v0: Fq2::zero(),
      }
  }
}

impl Fq6 {
  pub fn new(v2: Fq2, v1: Fq2, v0: Fq2) -> Self {
    Fq6 { v2, v1, v0 }
  }
}

impl Neg for Fq6 {
  type Output = Fq6;

  fn neg(self) -> Self::Output {
    Self::zero() - self
  }
}

macro_rules! impl_add {
  ($rhs: ty, $target: ty) => {
    impl<'a> Add<$rhs> for $target {
      type Output = Fq6;

      fn add(self, rhs: $rhs) -> Self::Output {
        Fq6 {
          v2: &self.v2 + &rhs.v2,
          v1: &self.v1 + &rhs.v1,
          v0: &self.v0 + &rhs.v0,
        }
      }
    }
  };
}
impl_add!(Fq6, Fq6);
impl_add!(Fq6, &Fq6);
impl_add!(&Fq6, Fq6);
impl_add!(&Fq6, &Fq6);

macro_rules! impl_sub {
  ($rhs: ty, $target: ty) => {
    impl<'a> Sub<$rhs> for $target {
      type Output = Fq6;

      fn sub(self, rhs: $rhs) -> Self::Output {
        Fq6 {
          v2: &self.v2 - &rhs.v2,
          v1: &self.v1 - &rhs.v1,
          v0: &self.v0 - &rhs.v0,
        }
      }
    }
  };
}
impl_sub!(Fq6, Fq6);
impl_sub!(Fq6, &Fq6);
impl_sub!(&Fq6, Fq6);
impl_sub!(&Fq6, &Fq6);

macro_rules! impl_mul {
  ($rhs: ty, $target: ty) => {
    impl<'a> Mul<$rhs> for $target {
      type Output = Fq6;

      fn mul(self, rhs: $rhs) -> Self::Output {
        let t0 = &self.v0 * &rhs.v0;
        let t1 = &self.v0 * &rhs.v1 + &self.v1 * &rhs.v0;
        let t2 = &self.v0 * &rhs.v2 + &self.v1 * &rhs.v1 + &self.v2 * &rhs.v0;
        let t3 = Fq2::apply_reduce_rule(&(&self.v1 * &rhs.v2 + &self.v2 * &rhs.v1));
        let t4 = Fq2::apply_reduce_rule(&(&self.v2 * &rhs.v2));
        Fq6 {
          v2: t2,
          v1: t1 + t4,
          v0: t0 + t3,
        }
      }
    }
  };
}
impl_mul!(Fq6, Fq6);
impl_mul!(Fq6, &Fq6);
impl_mul!(&Fq6, Fq6);
impl_mul!(&Fq6, &Fq6);