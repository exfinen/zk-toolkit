use crate::building_block::field::prime_field_elem::PrimeFieldElem;

// Y^2 + a_1XY + a_3Y = X^3 + a_2X^2 + a_4X + a_6
#[derive(Debug, Clone)]
pub struct WeierstrassEq {
  pub a1: PrimeFieldElem,
  pub a2: PrimeFieldElem,
  pub a3: PrimeFieldElem,
  pub a4: PrimeFieldElem,
  pub a6: PrimeFieldElem,
}

impl WeierstrassEq {
  pub fn new(
    a1: &PrimeFieldElem,
    a2: &PrimeFieldElem,
    a3: &PrimeFieldElem,
    a4: &PrimeFieldElem,
    a6: &PrimeFieldElem,
  ) -> Self {
    WeierstrassEq {
      a1: a1.clone(),
      a2: a2.clone(),
      a3: a3.clone(),
      a4: a4.clone(),
      a6: a6.clone(),
    }
  }

  pub fn is_rational_point(&self, x: &PrimeFieldElem, y: &PrimeFieldElem) -> bool {
    // check if Y^2 + a_1XY + a_3Y = X^3 + a_2X^2 + a_4X + a_6 holds
    let lhs =
      y * y
      + &self.a1 * x * y
      + &self.a3 * y
      ;
    let rhs =
      x * x * x
      + &self.a2 * x * x
      + &self.a4 * x
      + &self.a6
      ;
    lhs == rhs
  }
}
