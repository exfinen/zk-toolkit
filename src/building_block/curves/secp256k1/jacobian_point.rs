use crate::{
  impl_jacobian_add,
  impl_mul,
  building_block::{
    curves::secp256k1::{
      affine_point::AffinePoint,
      secp256k1::Secp256k1,
    },
    field::prime_field_elem::PrimeFieldElem,
    zero::Zero,
  },
};
use std::{
  ops::{Add, Mul},
  rc::Rc,
};

#[derive(Clone)]
pub struct JacobianPoint {
  pub curve: Rc<Secp256k1>,
  pub x: PrimeFieldElem,
  pub y: PrimeFieldElem,
  pub z: PrimeFieldElem,
}

impl JacobianPoint {
  pub fn new(
    curve: &Rc<Secp256k1>,
    x: &PrimeFieldElem,
    y: &PrimeFieldElem,
    z: &PrimeFieldElem,
  ) -> Self {
    JacobianPoint {
      curve: curve.clone(),
      x: x.clone(),
      y: y.clone(),
      z: z.clone(),
    }
  }
}

// write tests
impl Zero<JacobianPoint> for JacobianPoint {
  fn is_zero(&self) -> bool {
    self.x.is_zero() && self.y.is_zero()
  }

  fn zero(&self) -> JacobianPoint {
    let one = self.curve.f.elem(&0u8);
    JacobianPoint::new(
      &self.curve,
      &self.x.zero(),
      &self.y.zero(),
      &one,
    )
  }
}

impl_jacobian_add!();

impl_mul!(PrimeFieldElem, JacobianPoint);

impl From<AffinePoint> for JacobianPoint {
  fn from(p: AffinePoint) -> Self {
    if p.is_zero() {
      panic!("Cannot convert point at infinity to Jacobian point");
    } else {
      JacobianPoint::new(&p.curve, &p.x, &p.y, &p.x.f.elem(&1u8))
    }
  }
}

impl From<&AffinePoint> for JacobianPoint {
  fn from(p: &AffinePoint) -> Self {
    if p.is_zero() {
      panic!("Cannot convert point at infinity to Jacobian point");
    } else {
      JacobianPoint::new(&p.curve, &p.x, &p.y, &p.x.f.elem(&1u8))
    }
  }
}

impl PartialEq for JacobianPoint {
  // TODO write tests
  fn eq(&self, rhs: &Self) -> bool {
    (&self.x / &self.z == &rhs.x / &rhs.z)
    && (&self.y / &self.z == &rhs.y / &rhs.y)
  }
}

impl Eq for JacobianPoint {}