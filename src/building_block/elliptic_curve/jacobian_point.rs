use crate::building_block::{
  additive_identity::AdditiveIdentity,
  elliptic_curve::{
    affine_point::AffinePoint,
    ec_point::EcPoint,
    curve::Curve,
  },
  field::{
    field_elem_ops::Inverse,
    prime_field::PrimeField,
    prime_field_elem::PrimeFieldElem,
  },
  zero::Zero,
};
use std::ops::{Add, Sub, Mul, Div};

use super::weierstrass::curves::secp256k1::Secp256k1;

#[derive(Debug, Clone)]
pub struct JacobianPoint<C, E, P, F>
  where
    E:Zero<E> + AdditiveIdentity<E> + PartialEq<E> + Add<E> + Sub<E> + Mul<E> + Div<E>,
    C: Curve<P, E, F> + Clone,
    P: AffinePoint<Element=E> + Add<P> + Zero<P> + AdditiveIdentity<P> + Inverse + Clone,
{
  pub curve: Box<C>,
  pub x: E,
  pub y: E,
  pub z: E,
}

impl From<EcPoint> for JacobianPoint<Secp256k1, PrimeFieldElem, >> {
  fn from(pt: EcPoint) -> Self {
    if pt.is_zero() {
      panic!("Cannot convert point at infinity to Jacobian point")  // TODO fix this
    } else {
      JacobianPoint {
        curve: pt.curve.clone(),
        x: pt.x.clone(),
        y: pt.y.clone(),
        z: pt.x.f.elem(&1u8),
      }
    }
  }
}
