use mcl_rust::*;
use std::{
  convert::From,
  fmt,
  ops::{Add,
    Sub,
    Mul,
    Neg,
  },
};
use num_traits::Zero;

#[derive(Debug, Clone)]
pub struct MclGT {
  pub v: GT,
}

impl MclGT {
  pub fn new() -> Self {
    let v = GT::zero();
    MclGT::from(&v)
  }

  pub fn inv(&self) -> Self {
    let mut v = GT::zero();
    GT::inv(&mut v, &self.v);
    MclGT::from(&v)
  }

  pub fn sq(&self) -> Self {
    let mut v = GT::zero();
    GT::sqr(&mut v, &self.v);
    MclGT::from(&v)
  }
}

impl Zero for MclGT {
  fn is_zero(&self) -> bool {
    self.v.is_zero()
  }

  fn zero() -> Self {
    MclGT::from(&GT::zero())
  }
}

impl From<i32> for MclGT {
  fn from(value: i32) -> Self {
    let v = GT::from_int(value);
    MclGT { v }
  }
}

impl From<&GT> for MclGT {
  fn from(v: &GT) -> Self {
    MclGT { v: v.clone() }
  }
}

impl PartialEq for MclGT {
  fn eq(&self, other: &Self) -> bool {
    self.v == other.v
  }
}

impl Eq for MclGT {}

impl fmt::Display for MclGT {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.v.get_str(16))
  }
}

macro_rules! impl_neg {
  ($target: ty) => {
    impl Neg for $target {
      type Output = MclGT;

      fn neg(self) -> Self::Output {
        let mut v = GT::zero();
        GT::neg(&mut v, &self.v);
        MclGT::from(&v)
      }
    }
  }
}
impl_neg!(MclGT);
impl_neg!(&MclGT);

macro_rules! impl_add {
  ($rhs: ty, $target: ty) => {
    impl Add<$rhs> for $target {
      type Output = MclGT;

      fn add(self, rhs: $rhs) -> Self::Output {
        let mut v = GT::zero();
        GT::add(&mut v, &self.v, &rhs.v);
        MclGT::from(&v)
      }
    }
  };
}
impl_add!(MclGT, MclGT);
impl_add!(&MclGT, MclGT);
impl_add!(MclGT, &MclGT);
impl_add!(&MclGT, &MclGT);

macro_rules! impl_sub {
  ($rhs: ty, $target: ty) => {
    impl Sub<$rhs> for $target {
      type Output = MclGT;

      fn sub(self, rhs: $rhs) -> Self::Output {
        let mut v = GT::zero();
        GT::sub(&mut v, &self.v, &rhs.v);
        MclGT::from(&v)
      }
    }
  };
}
impl_sub!(MclGT, MclGT);
impl_sub!(&MclGT, MclGT);
impl_sub!(MclGT, &MclGT);
impl_sub!(&MclGT, &MclGT);

macro_rules! impl_mul {
  ($rhs: ty, $target: ty) => {
    impl Mul<$rhs> for $target {
      type Output = MclGT;

      fn mul(self, rhs: $rhs) -> Self::Output {
        let mut v = GT::zero();
        GT::mul(&mut v, &self.v, &rhs.v);
        MclGT { v }
      }
    }
  };
}
impl_mul!(MclGT, MclGT);
impl_mul!(&MclGT, MclGT);
impl_mul!(MclGT, &MclGT);
impl_mul!(&MclGT, &MclGT);

#[cfg(test)]
mod tests {
  use super::*;
  use crate::building_block::mcl::mcl_initializer::MclInitializer;

  #[test]
  fn test_add() {
    MclInitializer::init();

    let n3 = MclGT::from(3i32);
    let n9 = MclGT::from(9i32);
    let exp = MclGT::from(12i32);
    let act = n3 + n9;
    assert_eq!(exp, act);
  }

  #[test]
  fn test_sub() {
    MclInitializer::init();

    let n9 = MclGT::from(9i32);
    let n3 = MclGT::from(3i32);
    let exp = MclGT::from(6i32);
    let act = n9 - n3;
    assert_eq!(exp, act);
  }

  #[test]
  fn test_mul() {
    MclInitializer::init();

    let n3 = MclGT::from(3i32);
    let n9 = MclGT::from(9i32);
    let exp = MclGT::from(27i32);
    let act = n3 * n9;
    assert_eq!(exp, act);
  }

  // #[test]
  // fn test_inv() {
  //   MclInitializer::init();
  // 
  //   let n1 = MclGT::from(1i32);
  //   let n9 = MclGT::from(9i32);
  //   let inv9 = n9.inv();
  // 
  //   assert_eq!(n9 * inv9, n1);
  // }

  #[test]
  fn test_neg() {
    MclInitializer::init();

    let n9 = &MclGT::from(9i32);
    assert_eq!(n9 + -n9, MclGT::zero());
  }
}
