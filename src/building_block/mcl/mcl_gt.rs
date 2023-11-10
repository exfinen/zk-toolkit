use mcl_rust::*;
use std::{
  convert::From,
  fmt,
  ops::Mul,
};

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

  pub fn g() -> Self {
    MclGT::from(1)
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
  fn test_mul() {
    MclInitializer::init();

    let n3 = MclGT::from(3i32);
    let n9 = MclGT::from(9i32);
    let exp = MclGT::from(27i32);
    let act = n3 * n9;
    assert_eq!(exp, act);
  }

  #[test]
  fn test_inv() {
    MclInitializer::init();

    let n1 = MclGT::from(1i32);
    let n9 = MclGT::from(9i32);
    let inv9 = n9.inv();

    assert_eq!(n9 * inv9, n1);
  }
}
