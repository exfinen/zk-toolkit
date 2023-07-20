use crate::building_block::{
  additive_identity::AdditiveIdentity,
  elliptic_curve::{
    affine_point::AffinePoint,
    curve_equation::CurveEquation,
    elliptic_curve_point_ops::EllipticCurvePointOps,
    new_affine_point::NewAffinePoint,
  },
  field::{
    field::Field,
    field_elem_ops::Inverse,
  },
  zero::Zero,
};
use std::ops::{Add, Mul, Sub, Div};

pub trait Curve<Op, Eq, P, E, F>
where
  F: Field<F>,
  E: Zero<E> + AdditiveIdentity<E> + Add<E> + Sub<E> + Mul<E> + Div<E>,
  P: Zero<P> + Inverse + NewAffinePoint<P, E> + AffinePoint<P, E> + AdditiveIdentity<P> + AdditiveIdentity<E> + Clone + Add<P>,
  Op: EllipticCurvePointOps<P, E, F>,
  Eq: CurveEquation<P>,
{
  fn g(&self) -> P;
  fn group(&self) -> F;
  fn ops(&self) -> Box<Op>;
  fn equation(&self) -> Box<Eq>;
}
