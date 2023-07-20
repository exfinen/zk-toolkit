use crate::building_block::{
  additive_identity::AdditiveIdentity,
  elliptic_curve::{
    affine_point::AffinePoint,
    curve_equation::CurveEquation,
    elliptic_curve_point_ops::{
      EllipticCurveField,
      EllipticCurvePointAdd,
      ElllipticCurvePointInv,
    },
    new_affine_point::NewAffinePoint,
  },
  zero::Zero,
};
use std::ops::{Add, Mul, Sub, Div};

pub trait Curve<Op, Eq, P, E, F>
where
  E: Zero<E> + AdditiveIdentity + Add<E> + Sub<E> + Mul<E> + Div<E>,
  P: Zero<P> + NewAffinePoint<P, E> + AffinePoint<P, E> + AdditiveIdentity + Clone + Add<P>,
  Op: EllipticCurveField<F> + EllipticCurvePointAdd<P, E> + ElllipticCurvePointInv<P, E, F>,
  Eq: CurveEquation<P>,
{
  fn g(&self) -> P;
  fn group(&self) -> F;
  fn ops(&self) -> Box<Op>;
  fn equation(&self) -> Box<Eq>;
}
