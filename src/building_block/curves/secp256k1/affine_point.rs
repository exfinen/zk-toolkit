use crate::{
  impl_mul,
  building_block::{
    field::prime_field_elem::PrimeFieldElem,
    curves::secp256k1::{
      jacobian_point::JacobianPoint,
      secp256k1::Secp256k1,
    },
    zero::Zero,
  },
};
use std::{
  fmt,
  ops::{Add, Mul},
  rc::Rc,
};

#[derive(Clone)]
pub struct AffinePoint {
  pub curve: Rc<Secp256k1>,
  pub x: PrimeFieldElem,
  pub y: PrimeFieldElem,
  pub is_inf: bool,
}

impl fmt::Debug for AffinePoint {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{{ x: {:?}, y: {:?}, is_inf: {:?} }}", &self.x, &self.y, self.is_inf)
  }
}

impl AffinePoint {
  pub fn new(curve: &Rc<Secp256k1>, x: &PrimeFieldElem, y: &PrimeFieldElem) -> Self {
    AffinePoint {
      curve: curve.clone(),
      x: x.clone(),
      y: y.clone(),
      is_inf: false,
    }
  }

  pub fn rand_point(&self, exclude_zero: bool) -> Self {
    let g = &self.curve.g();
    loop {
      let multiplier = self.curve.f_n.rand_elem(exclude_zero);
      let p = g * multiplier;
      if !exclude_zero || !p.is_zero() { return p; }
    }
  }

  pub fn inv(&self) -> Self {
    if self.is_inf {
      panic!("Cannot calculate the inverse of zero");
    }
    AffinePoint::new(
      &self.curve,
      &self.x,
      &self.y.inv(),
    )
  }
}

impl From<JacobianPoint> for AffinePoint {
  fn from(p: JacobianPoint) -> Self {
    if p.z.is_zero() {
      panic!("z is not expected to be zero");
    } else {
      let z2 = p.z.sq();
      let z3 = &z2 * &p.z;
      let x = &p.x / z2;
      let y = &p.y / z3;
      AffinePoint::new(
        &p.curve,
        &x,
        &y,
      )
    }
  }
}

impl Zero<AffinePoint> for AffinePoint {
  fn zero(&self) -> Self {
    AffinePoint {
      curve: self.curve.clone(),
      x: self.curve.f.elem(&0u8),
      y: self.curve.f.elem(&0u8),
      is_inf: true,
    }
  }

  fn is_zero(&self) -> bool {
    self.is_inf
  }
}

impl_mul!(PrimeFieldElem, AffinePoint);

macro_rules! impl_add {
  ($rhs: ty, $target: ty) => {
    impl Add<$rhs> for $target {
      type Output = AffinePoint;

      fn add(self, rhs: $rhs) -> Self::Output {
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
          let m2 = &self.y * 2u8;
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

          AffinePoint::new(&self.curve, &p3x, &p3y_neg)

        } else {  // when line through p1 and p2 is non-vertical line
          // slope m of the line that intersects the curve at p1 and p2:
          // p2.y - p1.y = m(p2.x - p1.x)
          // m(p2.x - p1.x) = p2.y - p1.y
          // m = (p2.y - p1.y) / (p2.x - p1.x)
          let m = (&rhs.y - &self.y) / (&rhs.x - &self.x);

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
          AffinePoint::new(&self.curve, &p3x, &-p3y)
        }
      }
    }
  }
}
impl_add!(AffinePoint, AffinePoint);
impl_add!(AffinePoint, &AffinePoint);
impl_add!(&AffinePoint, AffinePoint);
impl_add!(&AffinePoint, &AffinePoint);

impl PartialEq for AffinePoint {
  fn eq(&self, other: &Self) -> bool {
    if self.is_inf != other.is_inf {  // false if one is zero and the other is non-zero
      false
    } else if self.is_inf {  // true if both are zero
      true
    } else {  // otherwise check if coordinates are the same
      self.x == other.x && self.y == other.y
    }
  }
}

impl Eq for AffinePoint {}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn scalar_mul() {
    let curve = Rc::new(Secp256k1::new());
    let g = &curve.g();

    {
      let act = g * curve.f.elem(&1u8);
      assert_eq!(&act, g);
    }
    {
      let act = g * curve.f.elem(&2u8);
      let exp = g + g;
      assert_eq!(act, exp);
    }
    {
      let act = g * curve.f.elem(&3u8);
      let exp = g + g + g;
      assert_eq!(act, exp);
    }
  }
}