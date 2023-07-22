use crate::building_block::{
  additive_identity::AdditiveIdentity,
  elliptic_curve::weierstrass::weierstrass_eq::WeierstrassEq,
  field:: field_elem_ops::Inverse,
  zero::Zero,
};
use std::ops::{Add, Mul, Sub, Div};
use num_bigint::BigUint;

pub trait Curve<P, E, F>
where
  E: Zero<E> + AdditiveIdentity<E> + Add<E> + Sub<E> + Mul<E> + Div<E>,
  P: Zero<P> + Inverse + AdditiveIdentity<P> + Clone + Add<P>,
{
  fn f(&self) -> F;    // base prime field
  fn f_n(&self) -> F;  // field of order n for convenience
  fn g(&self) -> P;    // generator point
  fn n(&self) -> BigUint;    // order of g
  fn eq(&self) -> Box<WeierstrassEq<E>>;
  fn point_at_infinity(&self) -> P;
}
