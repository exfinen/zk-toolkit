use crate::building_block::{
  secp256k1::affine_point::AffinePoint,
  field::{
    field_elem_ops::Inverse,
    prime_field::PrimeField,
    prime_field_elem::PrimeFieldElem,
  },
};
use std::ops::{Add, Sub, Mul, Div};

#[derive(Debug, Clone)]
pub struct JacobianPoint {
  pub x: PrimeFieldElem,
  pub y: PrimeFieldElem,
  pub z: PrimeFieldElem,
}

impl From<AffinePoint> for JacobianPoint {
  fn from(p: AffinePoint) -> Self {
    if p.is_zero() {
      panic!("Cannot convert point at infinity to Jacobian point");
    } else {
      JacobianPoint {
          x: p.x.clone(),
          y: p.y.clone(),
          z: p.x.f.elem(&1u8),
      }
    }
  }
}
