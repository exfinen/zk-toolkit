use crate::building_block::{
  secp256k1::{
    affine_point::AffinePoint,
    secp256k1::Secp256k1,
  },
  field::prime_field_elem::PrimeFieldElem,
  zero::Zero,
};
use std::rc::Rc;

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

impl From<AffinePoint> for JacobianPoint {
  fn from(p: AffinePoint) -> Self {
    if p.is_zero() {
      panic!("Cannot convert point at infinity to Jacobian point");
    } else {
      JacobianPoint::new(&p.curve, &p.x, &p.y, &p.x.f.elem(&1u8))
    }
  }
}
