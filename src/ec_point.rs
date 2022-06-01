use crate::field::{Field, FieldElem};
use num_bigint::BigUint;
use num_traits::identities::{Zero, One};

#[derive(Debug, Clone)]
pub struct EcPoint {
  pub x: FieldElem,
  pub y: FieldElem,
  pub is_inf: bool,
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

impl EcPoint {
  pub fn inf() -> Self {
    EcPoint {
      x: FieldElem::new(Field::new(BigUint::one()), BigUint::zero()),
      y: FieldElem::new(Field::new(BigUint::one()), BigUint::zero()),
      is_inf: true,
    }
  }

  pub fn new(x: FieldElem, y: FieldElem) -> Result<Self, String> {
    if x.f != y.f {
      return Err("Orders of field elements differ".to_string());
    }
    Ok(EcPoint { x, y, is_inf: false })
  }
}