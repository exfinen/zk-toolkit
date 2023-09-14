use std::{
  convert::From,
  fmt,
  ops::{Add, Sub, Mul, Neg},
};
use crate::building_block::{
  curves::bls12_381::{
    reduce::Reduce,
    fq1::Fq1,
  },
  to_biguint::ToBigUint,
  zero::Zero,
};

#[derive(Debug, Clone)]
pub struct Fq2 {
  pub u1: Fq1,
  pub u0: Fq1,
}

impl Fq2 {
  pub fn new(u1: &Fq1, u0: &Fq1) -> Self {
      Fq2 { u1: u1.clone(), u0: u0.clone() }
  }

  pub fn inv(&self) -> Self {
    let factor = &(&self.u1 * &self.u1 + &self.u0 * &self.u0).inv();
    Self {
      u1: self.u1.negate() * factor,
      u0: &self.u0 * factor,
    }
  }

  pub fn sq(&self) -> Self {
    self * self
  }
}

impl Zero<Fq2> for Fq2 {
  fn is_zero(&self) -> bool {
    self.u0.is_zero() && self.u1.is_zero()
  }

  fn zero() -> Self {
    Self {
      u1: Fq1::fq1_zero(),
      u0: Fq1::fq1_zero(),
    }
  }
}

impl Reduce for Fq2 {
  fn reduce(&self) -> Self {
    Self {
      u1: &self.u1 + &self.u0,
      u0: &self.u0 - &self.u1,
    }
  }
}

impl PartialEq for Fq2 {
  fn eq(&self, other: &Self) -> bool {
    self.u1 == other.u1 && self.u0 == other.u0
  }
}

impl Eq for Fq2 {}

impl From<&dyn ToBigUint> for Fq2 {
  fn from(n: &dyn ToBigUint) -> Self {
    let u0 = Fq1::from_to_biguint(n);
    Fq2::new(&Fq1::fq1_zero(), &u0)
  }
}

impl fmt::Display for Fq2 {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}\n{}", self.u0, self.u1)
  }
}

macro_rules! impl_neg {
  ($target: ty) => {
    impl Neg for $target {
      type Output = Fq2;

      fn neg(self) -> Self::Output {
          Fq2::zero() - self
      }
    }
  }
}
impl_neg!(Fq2);
impl_neg!(&Fq2);

macro_rules! impl_add {
  ($rhs: ty, $target: ty) => {
    impl Add<$rhs> for $target {
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
    impl Sub<$rhs> for $target {
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
    impl Mul<$rhs> for $target {
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
  use crate::building_block::curves::bls12_381::fq_test_helper::{
    get_fq1_values,
    get_fq2_values,
  };
  use super::*;

  fn to_strs(x: &Fq2) -> [String; 2] {
    [x.u1.e.to_string(), x.u0.e.to_string()]
  }

  #[test]
  fn test_add() {
    let (a1, b1, c1, d1) = get_fq1_values();
    let a2 = Fq2::new(&a1, &b1);
    let b2 = Fq2::new(&c1, &d1);
    let sum = a2 + b2;
    let [u1, u0] = to_strs(&sum);
    assert_eq!(u1, "4002409555221667393417789825735904156556882819939007885332058136124031650490837864442687629129015664037894272559777");
    assert_eq!(u0, "4002409555221667393417789825735904156556882819939007885332058136124031650490837864442687629129015664037894272559773");
  }

  #[test]
  fn test_sub() {
    let (a1, b1, c1, d1) = get_fq1_values();
    let a2 = Fq2::new(&a1, &b1);
    let b2 = Fq2::new(&c1, &d1);
    let sum = a2 - b2;
    let [u1, u0] = to_strs(&sum);
    assert_eq!(u1, "4");
    assert_eq!(u0, "4");
  }

  #[test]
  fn test_mul() {
    let (a1, b1, c1, d1) = get_fq1_values();
    let a2 = Fq2::new(&a1, &b1);
    let b2 = Fq2::new(&c1, &d1);
    let sum = a2 * b2;
    let [u1, u0] = to_strs(&sum);
    assert_eq!(u1, "62");
    assert_eq!(u0, "24");
  }

  #[test]
  fn test_inv() {
    let (a1, b1, c1, d1) = get_fq1_values();
    let a2 = Fq2::new(&a1, &b1);
    let b2 = Fq2::new(&c1, &d1);
    {
      let x = Fq2::inv(&a2);
      let [u1, u0] = to_strs(&x);
      assert_eq!(u1, "3178384058558382930067068391025570947853995180539800379528399108098495722448606539410369587837747733206563098797478");
      assert_eq!(u0, "2707512346179363236723798999762523400023773672311681804783451092083903763567331496534759278528451772731516713790444");
    }
    {
      let x = Fq2::inv(&b2);
      let [u1, u0] = to_strs(&x);
      assert_eq!(u1, "1816478182754449047781919997833987271052739125972318963343011000240906672145841800016296693220091724447967400623288");
      assert_eq!(u0, "523392018759756505293095592596233620472823137992024108081884525493142600448801874580966843809178971451109251027049");
    }
  }

  #[test]
  fn test_reduce() {
    let (a1, b1, c1, d1) = get_fq1_values();
    let a2 = Fq2::new(&a1, &b1);
    let b2 = Fq2::new(&c1, &d1);
    let x = Fq2::reduce(&(&a2 * &b2));
    let [u1, u0] = to_strs(&x);
    assert_eq!(u1, "86");
    assert_eq!(u0, "4002409555221667393417789825735904156556882819939007885332058136124031650490837864442687629129015664037894272559749");
  }

  #[test]
  fn test_neg() {
    let (a1, b1, c1, d1) = &get_fq2_values();
    assert_eq!(-a1 + a1, Fq2::zero());
    assert_eq!(-b1 + b1, Fq2::zero());
    assert_eq!(-c1 + c1, Fq2::zero());
    assert_eq!(-d1 + d1, Fq2::zero());
  }
}
