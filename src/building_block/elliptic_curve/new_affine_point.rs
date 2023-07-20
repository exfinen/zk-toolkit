use crate::building_block::{
  additive_identity::AdditiveIdentity,
  elliptic_curve::affine_point::AffinePoint,
  zero::Zero,
};

pub trait NewAffinePoint<P, E>
  where
    E: Zero<E> + AdditiveIdentity,
    P: AffinePoint<P, E> + Zero<P> + AdditiveIdentity,
{
  fn new(x: &P::E, y: &P::E) -> Self;
}
