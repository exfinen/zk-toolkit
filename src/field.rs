use num_bigint::BigUint;
use std::rc::Rc;

#[derive(Debug)]
pub struct Field {
  pub order: BigUint,
}

impl Field {
  pub fn new(order: BigUint) -> Rc<Self> {
    Rc::new(Field {
      order,
    })
  }
}

impl PartialEq for Field {
  fn eq(&self, other: &Self) -> bool {
    self.order == other.order
  }
}

impl Eq for Field {}
