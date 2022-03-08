use num_bigint::BigUint;
use crate::field_elem::FieldElem;

#[derive(Debug)]
pub struct Field {
  pub order: BigUint,
  pub zero: BigUint,
  pub one: BigUint,
}

impl Field {
  pub fn new(order: BigUint) -> Self {
    Field {
      order,
      zero: BigUint::from(0u32),
      one: BigUint::from(1u32),
    }
  }

  pub fn element(&self, v: BigUint) -> FieldElem {
    FieldElem::new(self, v)
  }
}

impl PartialEq for Field {
  fn eq(&self, other: &Self) -> bool {
    self.order == other.order
  }
}

impl Eq for Field {}
