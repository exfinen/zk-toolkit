use crate::building_block::{
  field::{
    field_elem_ops::Inverse,
    prime_field_elem::PrimeFieldElem,
  },
  secp256k1::jacobian_point::JacobianPoint,
  zero::Zero,
};
use std::ops::{Add, Sub};

#[derive(Clone)]
pub struct AffinePoint {
  pub x: PrimeFieldElem,
  pub y: PrimeFieldElem,
  pub is_inf: bool,
}

impl AffinePoint {
  pub fn new(x: &PrimeFieldElem, y: &PrimeFieldElem) -> Self {
    AffinePoint {
      x: x.clone(),
      y: y.clone(),
    }
  }
}

impl From<JacobianPoint> for EcPoint {
  fn from(pt: JacobianPoint<Secp256k1>) -> Self {
    if pt.z.is_zero() {
      panic!("z is not expected to be zero");
    } else {
      let z2 = pt.z.sq();
      let z3 = &z2 * &pt.z;
      let x = &pt.x / z2;
      let y = &pt.y / z3;
      EcPoint {
        curve: pt.curve,
        x,
        y,
        is_inf: false,
      }
    }
  }
}

impl Zero<AffinePoint> for AffinePoint {
  fn get_zero() -> Self {
    EcPoint {
      x: PrimeFieldElem::zero(),
      y: PrimeFieldElem::zero(),
      is_inf: true,
    }
  }

  fn is_zero(&self) -> bool {
    self.is_inf
  }
}

impl Add for AffinePoint {
  type Output = AffinePoint;

  fn add(self, rhs: Self) -> Self::Output {
    if self.is_zero() && rhs.is_zero() {  // inf + inf is inf
      self.clone()
    } else if self.is_zero() {  // adding p2 to inf is p2
      rhs.clone()
    } else if rhs.is_zero() {  // adding p1 to inf is p1
      self.clone()
    } else if self.x == rhs.x && self.y != rhs.y {  // if line through p1 and p2 is vertical line
      self.zero()
    } else if self.x == rhs.x && self.y == rhs.y {  // if adding the same point
      // special case: if y == 0, the tangent line is vertical
      if self.x.is_zero() || self.y.is_zero() {
        return self.zero()
      }
      // differentiate y^2 = x^3 + Ax + B w/ implicit differentiation
      // d/dx(y^2) = d/dx(x^3 + Ax + B)
      // 2y dy/dx = 3x^2 + A
      // dy/dx = (3x^2 + A) / 2y
      //
      // dy/dx is the slope m of the tangent line at the point
      // m = (3x^2 + A) / 2y
      let m1 = self.x.sq() * 3u8;
      let m2 = &self.y() * 2u8;
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
      let p3x = m.sq() - (&self.x * 2u8);

      // then get the y-coordinate by substituting x in (1) w/ x3 to get y3
      // y3 = m(x3 − p1.x) + p1.y
      //
      // reflecting y3 across the x-axis results in the addition result y-coordinate
      // result.y = -1 * y3 = m(p1.x - x3) - p1.y
      let p3y_neg = m * (&self.x - &p3x) - &self.y;

      AffilePoint::new(&p3x, &p3y_neg)

    } else {  // when line through p1 and p2 is non-vertical line
      // slope m of the line that intersects the curve at p1 and p2:
      // p2.y - p1.y = m(p2.x - p1.x)
      // m(p2.x - p1.x) = p2.y - p1.y
      // m = (p2.y - p1.y) / (p2.x - p1.x)
      let m = (&rhs.y - &rhs.y) / (&rhs.x - &self.x);

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
      let p3x = m.sq() - &self.x - &rhs.x;

      // using (1), find the y-coordinate of the 3rd intersecting point and p3x obtained above
      // y = m(x − p1.x) + p1.y
      // p3.y = m(p3.x − p1.x) + p1.y
      let p3y = m * (&p3x - &self.x) + &self.y;

      // then (p3.x, -p3.y) is the result of adding p1 and p2
      AffinePoint::new(&p3x, &-p3y)
    }
  }
}

impl Inverse for EcPoint {
  fn inv(&self) -> Self {
    if self.is_inf {
      panic!("Cannot calculate the inverse of zero");
    }
    EcPoint {
      curve: self.curve,
      x: self.x.clone(),
      y: self.y.inv(),
      is_inf: false,
    }
  }
}

impl PartialEq for EcPoint {
  fn eq(&self, other: &Self) -> bool {
    if self.is_inf != other.is_inf {
      false
    } else if self.is_inf {  // both is_inf's are true
      true
    } else {  // both is_inf's are false
      self.x == other.x && self.y == other.y
    }
  }
}

impl Eq for EcPoint {}

// impl<Op, E> From<EcPointWithOps<Op, E>> for EcPoint<E>
//   where Op: EllipticCurveField + EllipticCurvePointAdd<EcPoint<E>, E> + ElllipticCurvePointInv<EcPoint<E>, E> {

//   fn from(x: EcPointWithOps<Op>) -> EcPoint<E> {
//     x.0.1
//   }
// }

impl AdditiveIdentity<EcPoint> for EcPoint {
  fn get_additive_identity(&self) -> EcPoint {
    EcPoint {
      curve: self.curve,
      x: PrimeFieldElem::get_zero(&self.x),
      y: PrimeFieldElem::get_zero(&self.x),
      is_inf: true,
    }
  }
}

  // pub fn safe_new(x: &E, y: &E) -> Result<Self, String> where E: Clone {
  //   if x.f != y.f {
  //     return Err("Orders of field elements differ".to_string());
  //   }
  //   Ok(EcPoint::new(x, y))
  // }