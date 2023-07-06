use std::ops::{Add, Sub, Mul};
use crate::building_block::bls12_381::additional_ops::AdditionalOps;
use crate::building_block::bls12_381::fq1::Fq1;

#[derive(Debug, Clone)]
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

#[cfg(test)]
mod tests {
  use crate::building_block::bls12_381::setup::BASE_FIELD;

use super::*;

  /*
let a1 = Fq1 3
let b1 = Fq1 5
let c1 = Fq1 7
a1 * b1
b1 * c1

let a2 = Fq2 a1 b1
let b2 = Fq2 b1 c1
a2 * b2
-- testing inv a2*b2
inv a2 * b2
   */
  #[test]
  fn test_inv() {
    let f = &BASE_FIELD;
    let a1 = Fq1::new(f, &f.elem(&3u8));
    let b1 = Fq1::new(f, &f.elem(&5u8));
    let c1 = Fq1::new(f, &f.elem(&7u8));

    let a2 = Fq2::new(a1.clone(), b1.clone());
    let b2 = Fq2::new(b1.clone(), c1.clone());

    let x = Fq2::inv(&(a2 * b2));
    println!("inv={:?}", x);
  }
}