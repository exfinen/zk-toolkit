use std::ops::{Add, Sub, Mul};
use crate::building_block::bls12_381::additional_ops::AdditionalOps;
use crate::building_block::bls12_381::fq6::Fq6;

#[derive(Debug, Clone)]
pub struct Fq12 {
  w1: Fq6,
  w0: Fq6,
}

impl Fq12 {
  pub fn new(w1: &Fq6, w0: &Fq6) -> Self {
      Fq12 {
        w1: w1.clone(),
        w0: w0.clone(),
      }
  }
}

impl AdditionalOps for Fq12 {
  fn apply_reduce_rule(_n: &Self) -> Self {
    panic!("Not implemented");
  }

  fn inv(n: &Self) -> Self {
    let factor = Fq6::inv(&(
      &n.w0 * &n.w0
      - Fq6::apply_reduce_rule(&(&n.w1 * &n.w1))
    ));
    Self {
      w0: -n.w1.clone() * &factor,
      w1: &n.w0 * &factor,
    }
  }

  fn zero() -> Self {
      Self {
        w1: Fq6::zero(),
        w0: Fq6::zero(),
      }
  }
}

macro_rules! impl_add {
  ($rhs: ty, $target: ty) => {
    impl<'a> Add<$rhs> for $target {
      type Output = Fq12;

      fn add(self, rhs: $rhs) -> Self::Output {
        Fq12 {
          w1: &self.w1 + &rhs.w1,
          w0: &self.w0 + &rhs.w0,
        }
      }
    }
  };
}
impl_add!(Fq12, Fq12);
impl_add!(Fq12, &Fq12);
impl_add!(&Fq12, Fq12);
impl_add!(&Fq12, &Fq12);

macro_rules! impl_sub {
  ($rhs: ty, $target: ty) => {
    impl<'a> Sub<$rhs> for $target {
      type Output = Fq12;

      fn sub(self, rhs: $rhs) -> Self::Output {
        Fq12 {
          w1: &self.w1 - &rhs.w1,
          w0: &self.w0 - &rhs.w0,
        }
      }
    }
  };
}
impl_sub!(Fq12, Fq12);
impl_sub!(Fq12, &Fq12);
impl_sub!(&Fq12, Fq12);
impl_sub!(&Fq12, &Fq12);

macro_rules! impl_mul {
  ($rhs: ty, $target: ty) => {
    impl<'a> Mul<$rhs> for $target {
      type Output = Fq12;

      fn mul(self, rhs: $rhs) -> Self::Output {
        Fq12 {
          w1: &self.w1 * &rhs.w0 + &self.w0 * &rhs.w1,
          w0: &self.w0 * &rhs.w0 + Fq6::apply_reduce_rule(&(&self.w1 * &rhs.w1))
        }
      }
    }
  };
}
impl_mul!(Fq12, Fq12);
impl_mul!(Fq12, &Fq12);
impl_mul!(&Fq12, Fq12);
impl_mul!(&Fq12, &Fq12);
