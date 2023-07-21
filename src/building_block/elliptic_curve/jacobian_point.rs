use crate::building_block::{
  additive_identity::AdditiveIdentity,
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

#[derive(Debug, Clone)]
pub struct JacobianPoint {
  pub curve: Box<dyn Curve<EcPoint, PrimeFieldElem, PrimeField>>,
  pub x: PrimeFieldElem,
  pub y: PrimeFieldElem,
  pub z: PrimeFieldElem,
}

impl<P, E> From<P> for JacobianPoint {
  fn from(pt: EcPoint) -> Self {
    if pt.is_zero() {
      panic!("Cannot convert inf to Jacobian point")
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
