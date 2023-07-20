use crate::building_block::{
  additive_identity::AdditiveIdentity,
  zero::Zero,
};

pub trait AffinePoint<P, E>
  where
    E: Zero<E> + AdditiveIdentity<E>,
    P: Zero<P> + AdditiveIdentity<P> + AdditiveIdentity<E>,
{
  type E;

  fn x(&self) -> E;
  fn y(&self) -> E;
}
