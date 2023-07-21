use crate::building_block::{
  additive_identity::AdditiveIdentity,
  field:: field_elem_ops::Inverse,
  zero::Zero,

};
use std::ops::{Add, Mul, Sub, Div};

pub trait Curve<P, E, F>
where
  E: Zero<E> + AdditiveIdentity<E> + Add<E> + Sub<E> + Mul<E> + Div<E>,
  P: Zero<P> + Inverse + AdditiveIdentity<P> + AdditiveIdentity<E> + Clone + Add<P>,
{
  fn get_field(&self) -> F;
  fn g(&self) -> P;
}
