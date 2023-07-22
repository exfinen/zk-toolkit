use crate::building_block::{
  additive_identity::AdditiveIdentity,
  elliptic_curve::{
    affine_point::AffinePoint,
    curve::Curve,
  },
  field::field_elem_ops::Inverse,
  zero::Zero,
};
use std::ops::{Add, Sub, Mul, Div};

pub trait PointAdder<P, E, F, C>
  where
    E:Zero<E> + AdditiveIdentity<E> + Add<E> + Sub<E> + Mul<E> + Div<E>,
    C: Curve<P, E, F>,
    P: AffinePoint<Element=E> + Add<P> + Zero<P> + AdditiveIdentity<P> + Inverse + Clone,
{
  fn add(curve: &Box<C>, p1: &P, p2: &P) -> P;
}
