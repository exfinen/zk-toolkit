use mcl_rust::*;
use std::{
  cmp::Ordering,
  convert::From,
  fmt,
  ops::{Add, Sub, Mul, Neg},
  hash::{Hash, Hasher},
};
use num_traits::Zero;

#[derive(Debug, Clone)]
pub struct MclFr {
  pub v: Fr,
}

impl MclFr {
  pub fn new() -> Self {
    let v = Fr::zero();
    MclFr::from(&v)
  }

  pub fn inv(&self) -> Self {
    let mut v = Fr::zero();
    Fr::inv(&mut v, &self.v);
    MclFr::from(&v)
  }

  pub fn sq(&self) -> Self {
    let mut v = Fr::zero();
    Fr::sqr(&mut v, &self.v);
    MclFr::from(&v)
  }

  pub fn rand(exclude_zero: bool) -> Self {
    let mut v = Fr::zero();
    while {
      Fr::set_by_csprng(&mut v);
      v.is_zero() && exclude_zero
    } {}
    MclFr::from(&v)
  }

  pub fn inc(&self) -> Self {
    let v = &self.v + &Fr::from_int(1);
    MclFr::from(&v)
  }
}

impl Zero for MclFr {
  fn is_zero(&self) -> bool {
    self.v.is_zero()
  }

  fn zero() -> Self {
    MclFr { v: Fr::zero() }
  }
}

impl From<i32> for MclFr {
  fn from(value: i32) -> Self {
    let v = Fr::from_int(value);
    MclFr { v }
  }
}

impl From<usize> for MclFr {
  fn from(value: usize) -> Self {
    let value: i32 = value as i32;
    let v = Fr::from_int(value);
    MclFr { v }
  }
}

impl From<&Fr> for MclFr {
  fn from(v: &Fr) -> Self {
    MclFr { v: v.clone() }
  }
}

impl PartialOrd for MclFr {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    let r = &self.v.cmp(&other.v);
    if r.is_zero() {
      Some(Ordering::Equal)
    } else if r < &0 {
      Some(Ordering::Less)
    } else {
      Some(Ordering::Greater)
    }
  }
}

impl PartialEq for MclFr {
  fn eq(&self, other: &Self) -> bool {
    self.v == other.v
  }
}

impl Hash for MclFr {
  fn hash<H: Hasher>(&self, state: &mut H) {
    let mut buf: Vec<u8> = vec![];
    self.v.set_hash_of(&mut buf);
    buf.hash(state);
  }
}

impl Eq for MclFr {}

impl fmt::Display for MclFr {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.v.get_str(16))
  }
}

macro_rules! impl_neg {
  ($target: ty) => {
    impl Neg for $target {
      type Output = MclFr;

      fn neg(self) -> Self::Output {
        let mut v = Fr::zero();
        Fr::neg(&mut v, &self.v);
        MclFr { v }
      }
    }
  }
}
impl_neg!(MclFr);
impl_neg!(&MclFr);

macro_rules! impl_add {
  ($rhs: ty, $target: ty) => {
    impl Add<$rhs> for $target {
      type Output = MclFr;

      fn add(self, rhs: $rhs) -> Self::Output {
        let mut v = Fr::zero();
        Fr::add(&mut v, &self.v, &rhs.v);
        MclFr { v }
      }
    }
  };
}
impl_add!(MclFr, MclFr);
impl_add!(&MclFr, MclFr);
impl_add!(MclFr, &MclFr);
impl_add!(&MclFr, &MclFr);

macro_rules! impl_sub {
  ($rhs: ty, $target: ty) => {
    impl Sub<$rhs> for $target {
      type Output = MclFr;

      fn sub(self, rhs: $rhs) -> Self::Output {
        let mut v = Fr::zero();
        Fr::sub(&mut v, &self.v, &rhs.v);
        MclFr { v }
      }
    }
  };
}
impl_sub!(MclFr, MclFr);
impl_sub!(&MclFr, MclFr);
impl_sub!(MclFr, &MclFr);
impl_sub!(&MclFr, &MclFr);

macro_rules! impl_mul {
  ($rhs: ty, $target: ty) => {
    impl Mul<$rhs> for $target {
      type Output = MclFr;

      fn mul(self, rhs: $rhs) -> Self::Output {
        let mut v = Fr::zero();
        Fr::mul(&mut v, &self.v, &rhs.v);
        MclFr { v }
      }
    }
  };
}
impl_mul!(MclFr, MclFr);
impl_mul!(&MclFr, MclFr);
impl_mul!(MclFr, &MclFr);
impl_mul!(&MclFr, &MclFr);

#[cfg(test)]
mod tests {
  use super::*;
  use crate::building_block::curves::mcl::mcl_initializer::MclInitializer;

  #[test]
  fn test_add() {
    MclInitializer::init();

    let n3 = MclFr::from(3i32);
    let n9 = MclFr::from(9i32);
    let exp = MclFr::from(12i32);
    let act = n3 + n9;
    assert_eq!(exp, act);
  }

  #[test]
  fn test_sub() {
    MclInitializer::init();

    let n9 = MclFr::from(9i32);
    let n3 = MclFr::from(3i32);
    let exp = MclFr::from(6i32);
    let act = n9 - n3;
    assert_eq!(exp, act);
  }

  #[test]
  fn test_mul() {
    MclInitializer::init();

    let n3 = MclFr::from(3i32);
    let n9 = MclFr::from(9i32);
    let exp = MclFr::from(27i32);
    let act = n3 * n9;
    assert_eq!(exp, act);
  }


  #[test]
  fn test_inv() {
    MclInitializer::init();

    let n1 = MclFr::from(1i32);
    let n9 = MclFr::from(9i32);
    let inv9 = n9.inv();

    assert_eq!(n9 * inv9, n1);
  }

  #[test]
  fn test_neg() {
    MclInitializer::init();

    let n9 = &MclFr::from(9i32);
    assert_eq!(n9 + -n9, MclFr::zero());
  }
}
