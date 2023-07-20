use std::{
  ops::{Add, Sub, Mul},
  fmt,
};
use crate::building_block::{
  additive_identity::AdditiveIdentity,
  elliptic_curve::weierstrass::curves::bls12_381::{
    reduce::Reduce,
    fq1::Fq1,
  },
  field::{field_elem_ops::Inverse, prime_field_elem::PrimeFieldElem},
};

#[derive(Debug, Clone)]
pub struct Fq2<E> {
  pub u1: Fq1<E>,
  pub u0: Fq1<E>,
}

impl<E> Fq2<E> {
    pub fn new(u1: &Fq1<E>, u0: &Fq1<E>) -> Self {
        Fq2 { u1: u1.clone(), u0: u0.clone() }
    }
}

impl<E> AdditiveIdentity<E> for Fq2<E> {
  fn get_additive_identity() -> E {
      Self {
        u1: Fq1::zero(),
        u0: Fq1::zero(),
      }
  }
}

impl<E> Inverse<E> for Fq2<E> {
  fn inv(&self) -> E {
    let factor = &(self.u1 * self.u1 + self.u0 * self.u0).inv();
    Self {
      u1: self.u1.negate() * factor,
      u0: self.u0 * factor,
    }
  }
}

impl<E> Reduce for Fq2<E> {
  fn reduce(&self) -> Self {
    Self {
      u1: self.u1 + self.u0,
      u0: self.u0 - self.u1,
    }
  }
}

macro_rules! impl_add {
  ($rhs: ty, $target: ty) => {
    impl<E> Add<$rhs> for $target {
      type Output = Fq2<E>;

      fn add(self, rhs: $rhs) -> Self::Output {
        Fq2 {
          u1: &self.u1 + &rhs.u1,
          u0: &self.u0 + &rhs.u0,
        }
      }
    }
  };
}
impl_add!(Fq2<PrimeFieldElem>, Fq2<PrimeFieldElem>);
impl_add!(Fq2<PrimeFieldElem>, &Fq2<PrimeFieldElem>);
impl_add!(&Fq2<PrimeFieldElem>, Fq2<PrimeFieldElem>);
impl_add!(&Fq2<PrimeFieldElem>, &Fq2<PrimeFieldElem>);

macro_rules! impl_sub {
  ($rhs: ty, $target: ty) => {
    impl<E> Sub<$rhs> for $target {
      type Output = Fq2<E>;

      fn sub(self, rhs: $rhs) -> Self::Output {
        Fq2 {
          u1: &self.u1 - &rhs.u1,
          u0: &self.u0 - &rhs.u0,
        }
      }
    }
  };
}
impl_sub!(Fq2<PrimeFieldElem>, Fq2<PrimeFieldElem>);
impl_sub!(Fq2<PrimeFieldElem>, &Fq2<PrimeFieldElem>);
impl_sub!(&Fq2<PrimeFieldElem>, Fq2<PrimeFieldElem>);
impl_sub!(&Fq2<PrimeFieldElem>, &Fq2<PrimeFieldElem>);

macro_rules! impl_mul {
  ($rhs: ty, $target: ty) => {
    impl<E> Mul<$rhs> for $target {
      type Output = Fq2<E>;

      fn mul(self, rhs: $rhs) -> Self::Output {
        Fq2 {
          u1: &self.u1 * &rhs.u0 + &self.u0 * &rhs.u1,
          u0: &self.u0 * &rhs.u0 - &self.u1 * &rhs.u1,
        }
      }
    }
  };
}
impl_mul!(Fq2<PrimeFieldElem>, Fq2<PrimeFieldElem>);
impl_mul!(Fq2<PrimeFieldElem>, &Fq2<PrimeFieldElem>);
impl_mul!(&Fq2<PrimeFieldElem>, Fq2<PrimeFieldElem>);
impl_mul!(&Fq2<PrimeFieldElem>, &Fq2<PrimeFieldElem>);

impl<E> fmt::Display for Fq2<E> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{{ u1: {}, u0: {} }}", self.u1, self.u0)
  }
}

#[cfg(test)]
mod tests {
  use crate::building_block::elliptic_curve::weierstrass::curves::bls12_381::fq_test_helper::get_fq1_values;
  use super::*;

  fn to_strs(x: &Fq2<PrimeFieldElem>) -> [String; 2] {
    [x.u1.n.to_string(), x.u0.n.to_string()]
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
}
