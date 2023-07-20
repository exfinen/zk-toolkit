use crate::building_block::{
  additive_identity::AdditiveIdentity,
  elliptic_curve::affine_point::AffinePoint,
  zero::Zero,
};

pub trait NewAffinePoint<P, E>
  where
    E: Zero<E> + AdditiveIdentity<E>,
    P: AffinePoint<P, E> + Zero<P> + AdditiveIdentity<E>,
{
  fn new(x: &P::E, y: &P::E) -> Self;
}
