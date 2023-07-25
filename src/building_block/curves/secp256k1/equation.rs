use crate::building_block::{
  curves::secp256k1::affine_point::AffinePoint,
  field::prime_field_elem::PrimeFieldElem,
  zero::Zero,
};

// Y^2 + a_1XY + a_3Y = X^3 + a_2X^2 + a_4X + a_6
#[derive(Debug, Clone)]
pub struct Equation {
  pub a1: PrimeFieldElem,
  pub a2: PrimeFieldElem,
  pub a3: PrimeFieldElem,
  pub a4: PrimeFieldElem,
  pub a6: PrimeFieldElem,
}

impl Equation {
  pub fn new(
    a1: PrimeFieldElem,
    a2: PrimeFieldElem,
    a3: PrimeFieldElem,
    a4: PrimeFieldElem,
    a6: PrimeFieldElem,
  ) -> Self {
    Equation { a1, a2, a3, a4, a6 }
  }

  pub fn is_rational_point(&self, p: &AffinePoint) -> bool {
    if p.is_zero() {
      false
    } else {
      // check if Y^2 + a_1XY + a_3Y = X^3 + a_2X^2 + a_4X + a_6 holds
      let lhs =
        &p.y * &p.y
        + &self.a1 * &p.x * &p.y
        + &self.a3 * &p.y
        ;
      let rhs =
        &p.x * &p.x * &p.x
        + &self.a2 * &p.x * &p.x
        + &self.a4 * &p.x
        + &self.a6
        ;
      lhs == rhs
    }
  }
}
