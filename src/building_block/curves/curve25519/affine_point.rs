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
  rc::Rc,
};
use num_bigint::BigUint;
use num_traits::Zero as NumTraitsZero;

#[derive(Debug, PartialEq)]
pub enum Parity {
  Even,
  Odd,
}

#[derive(Clone)]
pub struct AffinePoint {
  pub x: PrimeFieldElem,
  pub y: PrimeFieldElem,
}

impl fmt::Debug for AffinePoint {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{{ x: {:?}, y: {:?} }}", &self.x, &self.y)
  }
}

impl AffinePoint {
  pub fn new(x: &PrimeFieldElem, y: &PrimeFieldElem) -> Self {
    Self {
      x: x.clone(),
      y: y.clone(),
    }
  }

  pub fn base_field() -> PrimeField {
    let two = BigUint::from(2u8);

    // order of base field q: 2^255 - 19
    let q = two.pow(255u32).sub(19u8);
    PrimeField::new(&q)
  }

  pub fn curve_group() -> PrimeField {
    let two = BigUint::from(2u8);

    // order of base point l: 2^252 + 27742317777372353535851937790883648493
    let l = two.pow(252u32).add(27742317777372353535851937790883648493u128);
    PrimeField::new(&l)
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
        let one = AffinePoint::base_field().elem(&1u8);
        let x1y2 = &self.x * &rhs.y;
        let x2y1 = &rhs.x * &self.y;
        let x1x2y1y2 = &x1y2 * &x2y1;
        let y1y2 = &self.y * &rhs.y;
        let x1x2 = &self.x * &rhs.x;
        let x = (x1y2 + x2y1) / (&one + (&AffinePoint::d() * &x1x2y1y2));
        let y = (y1y2 + x1x2) / (&one - (&AffinePoint::d() * x1x2y1y2));

        AffinePoint {
          x: x,
          y: y,
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
    if self.is_zero() != rhs.is_zero() { // false if one is zero and the other is non-zero
      false
    } else if self.is_zero() {  // true if both are zero
      true
    } else {  // otherwise check if coordinates are the same
      self.x == rhs.x && self.y == rhs.y
    }
  }
}

impl Eq for AffinePoint {}

impl Zero<AffinePoint> for AffinePoint {
  fn zero() -> AffinePoint {
      let f = Rc::new(AffinePoint::base_field());
      AffinePoint::new(
        &PrimeFieldElem { f: f.clone(), e: BigUint::from(0u8) },
        &PrimeFieldElem { f: f.clone(), e: BigUint::from(1u8) },
      )
  }

  fn is_zero(&self) -> bool {
    self == &AffinePoint::zero()
  }
}


#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn adding_zero_test() {
    let zero = &AffinePoint::zero();
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
