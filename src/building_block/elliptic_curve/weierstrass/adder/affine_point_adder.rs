use crate::building_block::{
  additive_identity::AdditiveIdentity,
  elliptic_curve::{
    curve::Curve,
    affine_point::AffinePoint,
    weierstrass::adder::point_adder::PointAdder,
  },
  field::field_elem_ops::Inverse,
  zero::Zero,
};
use std::ops::{Add, Sub, Mul, Div};

pub struct AffinePointAdder();

impl<P, C, E, F> PointAdder<P, C, E, F> for AffinePointAdder
  where
    E:Zero<E> + AdditiveIdentity<E> + Add<E> + Sub<E> + Mul<E> + Div<E>,
    C: Curve<P, E, F>,
    P: AffinePoint<Element=P> + Add<P> + Zero<P> + AdditiveIdentity<P> + Inverse + Clone,
{
  fn add(curve: &C, p1: &P, p2: &P) -> P {
    if p1.is_zero() && p2.is_zero() {  // inf + inf is inf
      p1.get_zero()
    } else if p1.is_zero() {  // adding p2 to inf is p2
      p2.clone()
    } else if p2.is_zero() {  // adding p1 to inf is p1
      p1.clone()
    } else if p1.x() == p2.x() && p1.y() != p2.y() {  // if line through p1 and p2 is vertical line
      curve.g().get_additive_identity()
    } else if p1.x() == p2.x() && p1.y() == p2.y() {  // if adding the same point
      // special case: if y == 0, the tangent line is vertical
      if p1.x.is_zero() || p2.y.is_zero() {
        return curve.g().get_additive_identity()
      }
      // differentiate y^2 = x^3 + Ax + B w/ implicit differentiation
      // d/dx(y^2) = d/dx(x^3 + Ax + B)
      // 2y dy/dx = 3x^2 + A
      // dy/dx = (3x^2 + A) / 2y
      //
      // dy/dx is the slope m of the tangent line at the point
      // m = (3x^2 + A) / 2y
      let m1 = p1.x().sq() * 3u8;
      let m2 = &p1.y() * 2u8;
      let m = m1 / &m2;

      // equation of intersecting line is
      // y = m(x − p1.x) + p1.y (1)
      //
      // substitute y with (1):
      // (m(x − p1.x) + p1.y)^2 = x^3 + Ax + B
      //
      // moving LHS to RHS, we get:
      // 0 = x^3 - m^2 x^2 + ...  (2)
      //
      // with below equation:
      // (x - r)(x - s)(x - t) = x^3 + (r + s + t)x^2 + (ab + ac + bc)x − abc
      //
      // we know that the coefficient of x^2 term is:
      // r + s + t
      //
      // using (2), the coefficient of x^2 term of the intersecting line is:
      // m^2 = r + s + t
      //
      // since p1 and p2 are the same point, replace r and s w/ p1.x
      // to get the x-coordinate of the point where (1) intersects the curve
      // x3 = m^2 − 2*p1.x
      let p3x = m.sq() - (&p1.x() * 2u8);

      // then get the y-coordinate by substituting x in (1) w/ x3 to get y3
      // y3 = m(x3 − p1.x) + p1.y
      //
      // reflecting y3 across the x-axis results in the addition result y-coordinate
      // result.y = -1 * y3 = m(p1.x - x3) - p1.y
      let p3y_neg = m * (&p1.x() - &p3x) - &p1.y();
      P::new(&p3x, &p3y_neg)

    } else {  // when line through p1 and p2 is non-vertical line
      // slope m of the line that intersects the curve at p1 and p2:
      // p2.y - p1.y = m(p2.x - p1.x)
      // m(p2.x - p1.x) = p2.y - p1.y
      // m = (p2.y - p1.y) / (p2.x - p1.x)
      let m = (&p2.y() - &p1.y()) / (&p2.x() - &p1.x());

      // then the equation of the line is:
      // y = m(x − p1.x) + p1.y  (1)
      //
      // starting from a curve equation of Weierstrass form:
      // y^2 = x^3 + Ax + B
      //
      // substitute y with (1):
      // (m(x − p1.x) + p1.y)^2 = x^3 + Ax + B
      //
      // moving LHS to RHS, we get:
      // 0 = x^3 - m^2 x^2 + ...  (2)
      //
      // with below equation:
      // (x - r)(x - s)(x - t) = x^3 + (r + s + t)x^2 + (ab + ac + bc)x − abc
      //
      // we know that the coefficient of x^2 term is:
      // r + s + t
      //
      // using (2), the coefficient of x^2 term of the intersecting line is:
      // m^2 = r + s + t
      //
      // substitute r and s with the known 2 roots -p1.x and p2.x:
      // m^2 = p1.x + p2. + t
      // t = m^2 - p1.x - p2.x
      //
      // here t is the x coordinate of the p3 we're trying to find:
      // p3.x = m^2 - p1.x - p2.x
      let p3x = m.sq() - &p1.x() - &p2.x();

      // using (1), find the y-coordinate of the 3rd intersecting point and p3x obtained above
      // y = m(x − p1.x) + p1.y
      // p3.y = m(p3.x − p1.x) + p1.y
      let p3y = m * (&p3x - &p1.x()) + &p1.y();

      // then (p3.x, -p3.y) is the result of adding p1 and p2
      P::new(&p3x, &-p3y)
    }
  }
}