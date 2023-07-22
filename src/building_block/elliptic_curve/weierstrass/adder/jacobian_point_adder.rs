use crate::building_block::{
  additive_identity::AdditiveIdentity,
  elliptic_curve::{
    curve::Curve,
    affine_point::AffinePoint,
    jacobian_point::JacobianPoint,
    weierstrass::adder::point_adder::PointAdder,
  },
  field::field_elem_ops::Inverse,
  zero::Zero,
};
use std::{
  cmp::PartialEq,
  ops::{Add, Sub, Mul, Div},
};

#[derive(Clone)]
pub struct JacobianPointAdder();

impl<P, C, E, F> PointAdder<P, E, F, C> for JacobianPointAdder
  where
    E:Zero<E> + AdditiveIdentity<E> + PartialEq<E> + Add<E> + Sub<E> + Mul<E> + Div<E>,
    C: Curve<P, E, F> + Clone,
    P: AffinePoint<Element=E> + Add<P> + Zero<P> + AdditiveIdentity<P> + Inverse + Clone,
{
  fn add(curve: &Box<C>, p1: &P, p2: &P) -> P {
    if p1.is_zero() && p2.is_zero() {  // zero + zero is zero
      p1.clone()
    } else if p1.is_zero() {  // adding p2 to zero is p2
      p2.clone()
    } else if p2.is_zero() {  // adding p1 to zero is p1
      p1.clone()
    } else if p1.x() == p2.x() && p1.y() != p2.y() {  // if line through p1 and p2 is vertical line
      curve.g().get_additive_identity()
    } else if p1.x() == p2.x() && p1.y() == p2.y() {  // if adding the same point
      // special case: if y == 0, the tangent line is vertical
      if p1.y.is_zero() || p2.y.is_zero() {
        return curve.g().get_additive_identity();
      }

      // formula described in: http://hyperelliptic.org/EFD/g1p/auto-shortw-jacobian-0.html#doubling-dbl-2009-l
      // w/ unnecessary computation removed
      let jp: JacobianPoint<C> = p1.into();

      let a = &jp.x.sq();
      let b = &jp.y.sq();
      let c = &b.sq();
      let d = &(((jp.x + b).sq() - a - c) * 2u8);
      let e = &(a * 3u8);
      let e_sq = &e.sq();
      let x3 = e_sq - (d * 2u8);
      let y3 = e * (d - &x3) - (c * 8u8);
      let z3 = jp.y * 2u8;

      let jp2 = JacobianPoint {
        curve: curve.clone(),
        x: x3,
        y: y3,
        z: z3,
      };
      jp2.into()

    } else {  // when line through p1 and p2 is non-vertical line
      let jp1: JacobianPoint = p1.into();
      let jp2: JacobianPoint = p2.into();

      // formula described in: https://hyperelliptic.org/EFD/g1p/auto-shortw-jacobian-3.html#addition-add-2007-bl
      // w/ unnecessary computation removed
      let h = jp2.x - &jp1.x;
      let i = (&h * 2).sq();
      let j = &h * &i;
      let r = (jp2.y - &jp1.y) * 2u8;
      let v = jp1.x * &i;
      let x3 = (r.sq() - &j) - (&v * 2u8);
      let y3 = r * (v - &x3) - (jp1.y * (j * 2u8));
      let z3 = h * 2u8;

      let jp3 = JacobianPoint {
        curve: curve.clone(),
        x: x3,
        y: y3,
        z: z3,
      };
      jp3.into()
    }
  }
}