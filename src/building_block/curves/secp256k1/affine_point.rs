use crate::{
  impl_mul,
  impl_affine_add,
  building_block::{
    field::prime_field_elem::PrimeFieldElem,
    curves::secp256k1::{
      jacobian_point::JacobianPoint,
      secp256k1::Secp256k1,
    },
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
  pub curve: Rc<Secp256k1>,
  pub x: PrimeFieldElem,
  pub y: PrimeFieldElem,
  pub is_inf: bool,
}

impl fmt::Debug for AffinePoint {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{{ x: {:?}, y: {:?}, is_inf: {:?} }}", &self.x, &self.y, self.is_inf)
  }
}

impl AffinePoint {
  pub fn new(curve: &Rc<Secp256k1>, x: &PrimeFieldElem, y: &PrimeFieldElem) -> Self {
    AffinePoint {
      curve: curve.clone(),
      x: x.clone(),
      y: y.clone(),
      is_inf: false,
    }
  }

  pub fn rand_point(&self, exclude_zero: bool) -> Self {
    let g = &self.curve.g();
    loop {
      let multiplier = self.curve.f_n.rand_elem(exclude_zero);
      let p = g * multiplier;
      if !exclude_zero || !p.is_zero() { return p; }
    }
  }

  pub fn inv(&self) -> Self {
    if self.is_inf {
      panic!("Cannot calculate the inverse of zero");
    }
    AffinePoint::new(
      &self.curve,
      &self.x,
      &self.y.inv(),
    )
  }
}

impl From<JacobianPoint> for AffinePoint {
  fn from(p: JacobianPoint) -> Self {
    if p.z.is_zero() {
      panic!("z is not expected to be zero");
    } else {
      let z2 = p.z.sq();
      let z3 = &z2 * &p.z;
      let x = &p.x / z2;
      let y = &p.y / z3;
      AffinePoint::new(
        &p.curve,
        &x,
        &y,
      )
    }
  }
}

impl Zero<AffinePoint> for AffinePoint {
  fn zero(&self) -> Self {
    AffinePoint {
      curve: self.curve.clone(),
      x: self.curve.f.elem(&0u8),
      y: self.curve.f.elem(&0u8),
      is_inf: true,
    }
  }

  fn is_zero(&self) -> bool {
    self.is_inf
  }
}

impl_affine_add!(AffinePoint);
impl_mul!(PrimeFieldElem, AffinePoint);

impl PartialEq for AffinePoint {
  fn eq(&self, other: &Self) -> bool {
    if self.is_inf != other.is_inf {  // false if one is zero and the other is non-zero
      false
    } else if self.is_inf {  // true if both are zero
      true
    } else {  // otherwise check if coordinates are the same
      self.x == other.x && self.y == other.y
    }
  }
}

impl Eq for AffinePoint {}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn scalar_mul() {
    let curve = Rc::new(Secp256k1::new());
    let g = &curve.g();

    {
      let act = g * curve.f.elem(&1u8);
      assert_eq!(&act, g);
    }
    {
      let act = g * curve.f.elem(&2u8);
      let exp = g + g;
      assert_eq!(act, exp);
    }
    {
      let act = g * curve.f.elem(&3u8);
      let exp = g + g + g;
      assert_eq!(act, exp);
    }
  }
}