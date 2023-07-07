use std::ops::{Add, Sub, Mul};
use crate::building_block::bls12_381::additional_ops::AdditionalOps;
use crate::building_block::bls12_381::fq1::Fq1;
use std::fmt;

#[derive(Debug, Clone)]
pub struct Fq2 {
  pub u1: Fq1,
  pub u0: Fq1,
}

impl Fq2 {
    pub fn new(u1: &Fq1, u0: &Fq1) -> Self {
        Fq2 { u1: u1.clone(), u0: u0.clone() }
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

impl fmt::Display for Fq2 {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{{ u1: {}, u0: {} }}", self.u1, self.u0)
  }
}

#[cfg(test)]
mod tests {
  use crate::building_block::bls12_381::fq_test_helper::get_fq1_values;
  use super::*;

  fn to_str_tuple(x: &Fq2) -> (String, String) {
    (x.u1.n.to_string(), x.u0.n.to_string())
  }

  #[test]
  fn test_add() {
    let (a1, b1, c1, d1) = get_fq1_values();
    let a2 = Fq2::new(&a1, &b1);
    let b2 = Fq2::new(&c1, &d1);
    let sum = a2 + b2;
    let (u1, u0) = to_str_tuple(&sum);
    assert_eq!(u1, "4002409555221667393417789825735904156556882819939007885332058136124031650490837864442687629129015664037894272559777");
    assert_eq!(u0, "4002409555221667393417789825735904156556882819939007885332058136124031650490837864442687629129015664037894272559773");
  }

  #[test]
  fn test_sub() {
    let (a1, b1, c1, d1) = get_fq1_values();
    let a2 = Fq2::new(&a1, &b1);
    let b2 = Fq2::new(&c1, &d1);
    let sum = a2 - b2;
    let (u1, u0) = to_str_tuple(&sum);
    assert_eq!(u1, "4");
    assert_eq!(u0, "4");
  }

  #[test]
  fn test_mul() {
    let (a1, b1, c1, d1) = get_fq1_values();
    let a2 = Fq2::new(&a1, &b1);
    let b2 = Fq2::new(&c1, &d1);
    let sum = a2 * b2;
    let (u1, u0) = to_str_tuple(&sum);
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
      let (u1, u0) = to_str_tuple(&x);
      assert_eq!(u1, "3178384058558382930067068391025570947853995180539800379528399108098495722448606539410369587837747733206563098797478");
      assert_eq!(u0, "2707512346179363236723798999762523400023773672311681804783451092083903763567331496534759278528451772731516713790444");
    }
    {
      let x = Fq2::inv(&b2);
      let (u1, u0) = to_str_tuple(&x);
      assert_eq!(u1, "1816478182754449047781919997833987271052739125972318963343011000240906672145841800016296693220091724447967400623288");
      assert_eq!(u0, "523392018759756505293095592596233620472823137992024108081884525493142600448801874580966843809178971451109251027049");
    }
  }

  #[test]
  fn test_reduce() {
    let (a1, b1, c1, d1) = get_fq1_values();
    let a2 = Fq2::new(&a1, &b1);
    let b2 = Fq2::new(&c1, &d1);
    let x = Fq2::apply_reduce_rule(&(&a2 * &b2));
    let (u1, u0) = to_str_tuple(&x);
    assert_eq!(u1, "86");
    assert_eq!(u0, "4002409555221667393417789825735904156556882819939007885332058136124031650490837864442687629129015664037894272559749");
  }
}