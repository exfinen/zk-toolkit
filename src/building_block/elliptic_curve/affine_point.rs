use crate::building_block::{
  additive_identity::AdditiveIdentity,
  zero::Zero,
};

pub trait AffinePoint<P, E>
  where
    E: Zero<E> + AdditiveIdentity,
    P: Zero<P> + AdditiveIdentity,
{
  type E;

  fn x(&self) -> E;
  fn y(&self) -> E;
}
