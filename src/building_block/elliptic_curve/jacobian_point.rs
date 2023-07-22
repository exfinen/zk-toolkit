use crate::building_block::{
  elliptic_curve::{
    ec_point::EcPoint,
    curve::Curve,
  },
  field::{
    prime_field::PrimeField,
    prime_field_elem::PrimeFieldElem,
  },
  zero::Zero,
};

use super::weierstrass::curves::secp256k1::Secp256k1;

#[derive(Debug, Clone)]
pub struct JacobianPoint<C>
where
  C: Curve<EcPoint, PrimeFieldElem, PrimeField>,
{
  pub curve: Box<C>,
  pub x: PrimeFieldElem,
  pub y: PrimeFieldElem,
  pub z: PrimeFieldElem,
}

impl From<EcPoint> for JacobianPoint<Secp256k1> {
  fn from(pt: EcPoint) -> Self {
    if pt.is_zero() {
      panic!("Cannot convert point at infinity to Jacobian point")  // TODO fix this
    } else {
      JacobianPoint {
        curve: pt.curve(),
        x: pt.x.clone(),
        y: pt.y.clone(),
        z: pt.x.f.elem(&1u8),
      }
    }
  }
}
