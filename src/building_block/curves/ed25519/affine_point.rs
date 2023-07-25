use crate::{
  impl_mul,
  building_block::{
    curves::ed25519::ed25519_sha512::Ed25519Sha512,
    field::prime_field_elem::PrimeFieldElem,
    zero::Zero,
  },
};
use std::{
  fmt,
  ops::{Add, Mul},
  rc::Rc,
};

#[derive(Clone)]
pub struct AffinePoint {
  pub curve: Rc<Ed25519Sha512>,
  pub x: PrimeFieldElem,
  pub y: PrimeFieldElem,
}

impl fmt::Debug for AffinePoint {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{{ x: {:?}, y: {:?} }}", &self.x, &self.y)
  }
}

impl AffinePoint {
  pub fn new(curve: &Rc<Ed25519Sha512>, x: &PrimeFieldElem, y: &PrimeFieldElem) -> Self {
    AffinePoint {
      curve: curve.clone(),
      x: x.clone(),
      y: y.clone(),
    }
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
        let x1y2 = &self.x * &rhs.y;
        let x2y1 = &rhs.x * &self.y;
        let x1x2y1y2 = &x1y2 * &x2y1;
        let y1y2 = &self.y * &rhs.y;
        let x1x2 = &self.x * &rhs.x;
        let x = (x1y2 + x2y1) / (&self.curve.one + (&self.curve.d * &x1x2y1y2));
        let y = (y1y2 + x1x2) / (&self.curve.one - (&self.curve.d * x1x2y1y2));

        AffinePoint {
          curve: self.curve.clone(),
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

impl Zero<AffinePoint> for AffinePoint {
  fn zero(&self) -> AffinePoint {
      AffinePoint::new(
        &self.curve.clone(),
        &self.curve.zero,
        &self.curve.one,
      )
  }

  fn is_zero(&self) -> bool {
      self.x.is_zero() && self.y == self.curve.one
  }
}

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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn adding_zero_test() {
    let curve = Ed25519Sha512::new();
    let zero = &curve.B().zero();
    #[allow(non_snake_case)]
    let B = &curve.B();
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
    let curve = Rc::new(Ed25519Sha512::new());
    let B = &curve.B();

    {
      let act = B * curve.f.elem(&1u8);
      assert_eq!(&act, B);
    }
    {
      let act = B * curve.f.elem(&2u8);
      let exp = B + B;
      assert_eq!(act, exp);
    }
    {
      let act = B * curve.f.elem(&3u8);
      let exp = B + B + B;
      assert_eq!(act, exp);
    }
  }
}
