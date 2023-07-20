use crate::building_block::{
  elliptic_curve::{
    affine_point::AffinePoint,
    curve_equation::CurveEquation,
  },
  field::{
    prime_field::PrimeField,
    prime_field_elem::PrimeFieldElem,
  },
  zero::Zero,
};

// Y^2 + a_1XY + a_3Y = X^3 + a_2X^2 + a_4X + a_6
#[derive(Clone)]
pub struct WeierstrassEq<F, E> {
  pub f: F,
  pub a1: E,
  pub a2: E,
  pub a3: E,
  pub a4: E,  // a originally
  pub a6: E,  // b originally
}

impl WeierstrassEq<PrimeField, PrimeFieldElem> {
  pub fn new(
    f: &PrimeField,
    a1: &PrimeFieldElem,
    a2: &PrimeFieldElem,
    a3: &PrimeFieldElem,
    a4: &PrimeFieldElem,
    a6: &PrimeFieldElem,
  ) -> Self {
    WeierstrassEq {
      f: f.clone(),
      a1: a1.clone(),
      a2: a2.clone(),
      a3: a3.clone(),
      a4: a4.clone(),
      a6: a6.clone(),
    }
  }
}

impl<F, E, P> CurveEquation<P> for WeierstrassEq<F, E>
  where P: AffinePoint<P, E> + Zero<P>
{
  fn is_rational_point(&self, pt: &P) -> bool {
    if pt.is_inf {
      false
    } else {
      // check if Y^2 + a_1XY + a_3Y = X^3 + a_2X^2 + a_4X + a_6 holds
      let lhs =
        pt.y() * pt.y()
        + self.a1 * pt.x() * pt.y()
        + self.a3 * pt.y()
        ;
      let rhs =
        pt.x() * pt.x() * pt.x()
        + self.a2 * pt.x() * pt.x()
        + self.a4 * pt.x()
        + self.a6
        ;
      lhs == rhs
    }
  }
}
