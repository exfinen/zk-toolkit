use crate::{
  impl_mul,
  building_block::{
    field::{
      prime_field::PrimeField,
      prime_field_elem::PrimeFieldElem,
    },
    zero::Zero,
  },
};
use std::{
  fmt,
  ops::{Add, Sub, Mul},
  sync::Arc,
};
use once_cell::sync::Lazy;
use num_bigint::BigUint;
use num_traits::Zero as NumTraitsZero;

#[derive(Debug, PartialEq)]
pub enum Parity {
  Even,
  Odd,
}

#[derive(Clone)]
pub enum AffinePoint {
  AtInfinity,
  Rational { x: PrimeFieldElem, y: PrimeFieldElem },
}

impl fmt::Debug for AffinePoint {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      AffinePoint::AtInfinity => write!(f, "{{ Point at infinity }}"),
      AffinePoint::Rational { x, y } => write!(f, "{{ x: {:?}, y: {:?} }}", x, y),
    }
  }
}

static BASE_FIELD: Lazy<Arc<PrimeField>> = Lazy::new(|| {
  // order of base field q: 2^255 - 19
  let two = BigUint::from(2u8);
  let q = two.pow(255u32).sub(19u8);
  Arc::new(PrimeField::new(&q))
});

static CURVE_GROUP: Lazy<Arc<PrimeField>> = Lazy::new(|| {
  // order of base point l: 2^252 + 27742317777372353535851937790883648493
  let two = BigUint::from(2u8);
  let l = two.pow(252u32).add(27742317777372353535851937790883648493u128);
  Arc::new(PrimeField::new(&l))
});

impl AffinePoint {
  pub fn new(x: &PrimeFieldElem, y: &PrimeFieldElem) -> Self {
    AffinePoint::Rational { x: x.clone(), y: y.clone() }
  }

  pub fn base_field() -> Arc<PrimeField> {
    BASE_FIELD.clone()
  }

  pub fn curve_group() -> Arc<PrimeField> {
    CURVE_GROUP.clone()
  }

  // base point (+x, 4/5)
  #[allow(non_snake_case)]
  pub fn B() -> AffinePoint {
    let f = Self::base_field();
    let B_y = f.elem(&4u8) / 5u8;
    let B_x = Self::recover_x(&Self::d(), &B_y, Parity::Even);  // get positive x
    AffinePoint::new(&B_x, &B_y)
  }

  fn get_parity(e: &PrimeFieldElem) -> Parity {
    if (&e.e % 2u8).is_zero() { Parity::Even } else { Parity::Odd }
  }

  // d is passed to allow new() to call this function. ideally d should be replaced by &self
  #[allow(non_snake_case)]
  pub fn recover_x(d: &PrimeFieldElem, y: &PrimeFieldElem, x_parity: Parity) -> PrimeFieldElem {
    let f = &d.f;
    let q = &d.f.order;

    // xx = x^2 = (y^2 - 1) / (1 + d*y^2)
    let xx = (y.sq() - 1u8) / ((d * y.sq()) + 1u8);

    // calculate the square root of xx assuming a^((q-1)/4) = 1 mod q
    let mut x = (&xx).pow(&((q + &3u8) / &8u8));

    // if that doesn't match, calculate the square root of xx again
    // assuming a^((q-1)/4) = -1 mod q
    if &x.sq() != &xx {
      let I = f.elem(&2u8).pow(&((q - &1u8) / &4u8));
      x = &x * &I;
    }
    let root_parity = Self::get_parity(&x);
    if root_parity != x_parity {
      x = -&x;
    }
    x
  }

  pub fn d() -> PrimeFieldElem {
    let f = Self::base_field();

    // d = -121665 / 121666
    let d = -f.elem(&121665u32) / 121666u32;
    d
  }
}

impl_mul!(PrimeFieldElem, AffinePoint);

macro_rules! impl_add {
  ($rhs: ty, $target: ty) => {
    impl Add<$rhs> for $target {
      type Output = AffinePoint;

      // Edwards Addition Law
      // (x1,y1) + (x2,y2) = ((x1y2 + x2y1) / (1 + d x1x2 y1y2), (y1y2 + x1x2) / (1 - d x1x2 y1y2))
      fn add(self, rhs: $rhs) -> Self::Output {
        match (self, rhs) {
          (AffinePoint::AtInfinity, AffinePoint::AtInfinity) => AffinePoint::AtInfinity,
          (AffinePoint::AtInfinity, p) => p.clone(),
          (p, AffinePoint::AtInfinity) => p.clone(),
          (AffinePoint::Rational { x: x1, y: y1 }, AffinePoint::Rational { x: x2, y: y2 }) => {
            let one = AffinePoint::base_field().elem(&1u8);
            let x1y2 = x1.clone() * y2.clone();
            let x2y1 = x2.clone() * y1.clone();
            let x1x2y1y2 = &x1y2 * &x2y1;
            let y1y2 = y1 * y2;
            let x1x2 = x1 * x2;
            let x = (x1y2 + x2y1) / (&one + (&AffinePoint::d() * &x1x2y1y2));
            let y = (y1y2 + x1x2) / (&one - (&AffinePoint::d() * x1x2y1y2));

            AffinePoint::Rational { x, y }
          },
        }
      }
    }
  }
}
impl_add!(AffinePoint, AffinePoint);
impl_add!(AffinePoint, &AffinePoint);
impl_add!(&AffinePoint, AffinePoint);
impl_add!(&AffinePoint, &AffinePoint);

impl PartialEq for AffinePoint {
  fn eq(&self, rhs: &Self) -> bool {
    match (self, rhs) {
      (AffinePoint::AtInfinity, AffinePoint::AtInfinity) => true,
      (_, AffinePoint::AtInfinity) => false,
      (AffinePoint::AtInfinity, _) => false,
      (AffinePoint::Rational { x: x1, y: y1 }, AffinePoint::Rational { x: x2, y: y2 }) => {
        x1 == x2 && y1 == y2
      },
    }
  }
}

impl Eq for AffinePoint {}

impl Zero<AffinePoint> for AffinePoint {
  fn zero() -> AffinePoint {
      let f = AffinePoint::base_field();
      AffinePoint::new(
        &PrimeFieldElem { f: f.clone(), e: BigUint::from(0u8) },
        &PrimeFieldElem { f: f.clone(), e: BigUint::from(1u8) },
      )
  }

  fn is_zero(&self) -> bool {
    if let AffinePoint::AtInfinity = self { true } else { false }
  }
}


#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  #[allow(non_snake_case)]
  fn add_same_points() {
    let B = &AffinePoint::B();
    let B2 = B + B;
    match B2 {
      AffinePoint::AtInfinity => panic!("expected rational point, but got point at infinity"),
      AffinePoint::Rational { x: _x, y: _y } => {
      },
    }
  }

  #[test]
  fn adding_zero_test() {
    let zero = &AffinePoint::AtInfinity;
    #[allow(non_snake_case)]
    let B = &AffinePoint::B();
    {
      let pt = zero + zero;
      assert!(pt.is_zero());
    }
    {
      let pt = B + zero;
      assert!(&pt == B);
    }
    {
      let pt = zero + B;
      assert!(&pt == B);
    }
    {
      let pt = B + B;
      assert!(pt.is_zero() == false);
    }
  }

  #[test]
  #[allow(non_snake_case)]
  fn scalar_mul() {
    let B = &AffinePoint::B();
    let f = &AffinePoint::curve_group();
    {
      let act = B * f.elem(&1u8);
      assert_eq!(&act, B);
    }
    {
      let act = B * f.elem(&2u8);
      let exp = B + B;
      assert_eq!(act, exp);
    }
    {
      let act = B * f.elem(&3u8);
      let exp = B + B + B;
      assert_eq!(act, exp);
    }
  }
}
