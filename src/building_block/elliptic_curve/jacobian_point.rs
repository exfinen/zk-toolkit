use crate::building_block::{
  additive_identity::AdditiveIdentity,
  elliptic_curve::affine_point::AffinePoint,
  field::field_elem::NewFieldElem,
  zero::Zero,
};

#[derive(Debug, Clone)]
pub struct JacobianPoint<E> {
  pub x: E,
  pub y: E,
  pub z: E,
}

impl<P, E> From<P> for JacobianPoint<E>
  where
    E: Zero<E> + AdditiveIdentity<E>,
    P: AffinePoint<P, E> + Zero<P> + AdditiveIdentity<E>
{
  fn from(pt: P) -> Self {
    if pt.is_zero() {
      panic!("Cannot convert inf to Jacobian point")
    } else {
      JacobianPoint {
        x: pt.x.clone(),
        y: pt.y.clone(),
        z: pt.x.f.elem(&1u8),
      }
    }
  }
}
