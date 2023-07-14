use super::super::ec_point::EcPoint;
use crate::building_block::{
  elliptic_curve::curve_equation::CurveEquation,
  field::{Field, FieldElem},
};
use num_bigint::BigUint;

// Y^2 + a_1XY + a_3Y = X^3 + a_2X^2 + a_4X + a_6
#[derive(Clone)]
pub struct WeierstrassEq {
  pub f: Field,
  pub a1: FieldElem,
  pub a2: FieldElem,
  pub a3: FieldElem,
  pub a4: FieldElem,  // a originally
  pub a6: FieldElem,  // b originally
}

impl WeierstrassEq {
  pub fn new(
    f: &Field,
    a1: BigUint,
    a2: BigUint,
    a3: BigUint,
    a4: BigUint,
    a6: BigUint,
  ) -> Self {
    let a1 = FieldElem::new(f, &a1);
    let a2 = FieldElem::new(f, &a2);
    let a3 = FieldElem::new(f, &a3);
    let a4 = FieldElem::new(f, &a4);
    let a6 = FieldElem::new(f, &a6);

    WeierstrassEq { f: f.clone(), a1, a2, a3, a4, a6 }
  }
}

impl CurveEquation for WeierstrassEq {
  fn is_rational_point(&self, pt: &EcPoint) -> bool {
    if pt.is_inf {
      false
    } else {
      // check if Y^2 + a_1XY + a_3Y = X^3 + a_2X^2 + a_4X + a_6 holds
      let lhs =
        &pt.y * &pt.y
        + &self.a1 * &pt.x * &pt.y
        + &self.a3 * &pt.y
        ;
      let rhs =
        &pt.x * &pt.x * &pt.x
        + &self.a2 * &pt.x * &pt.x
        + &self.a4 * &pt.x
        + &self.a6
        ;
      lhs == rhs
    }
  }
}
