use num_bigint::BigUint;
use std::rc::Rc;
use crate::field_elem::FieldElem;

#[derive(Debug, Clone)]
pub struct Field {
  pub order: Rc<BigUint>,
}

pub trait ToBigUint {
  fn to_biguint(&self) -> BigUint;
}

impl ToBigUint for BigUint {
  fn to_biguint(&self) -> BigUint {
    self.clone()
  }
}

macro_rules! impl_to_biguint_for {
  ($name: ty) => {
    impl ToBigUint for $name {
      fn to_biguint(&self) -> BigUint {
        BigUint::from(*self)
      }
    }
  };
}
impl_to_biguint_for!(u8);
impl_to_biguint_for!(u16);
impl_to_biguint_for!(u32);
impl_to_biguint_for!(u64);
impl_to_biguint_for!(u128);

impl Field {
  pub fn new(order: BigUint) -> Self {
    Field {
      order: Rc::new(order),
    }
  } 

  pub fn elem(&self, n: &impl ToBigUint) -> FieldElem {
    FieldElem::new(self.clone(), n.to_biguint())
  }
}

impl PartialEq for Field {
  fn eq(&self, other: &Self) -> bool {
    self.order == other.order
  }
}

impl Eq for Field {}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn new_elem_from_biguint() {
    let f = Field::new(BigUint::from(11u32));
    let a = f.elem(&7u8);
    assert_eq!(a.n, BigUint::from(7u32));
  }
  
  #[test]
  fn new_elem_from_u8() {
    let f = Field::new(BigUint::from(11u32));
    let a = f.elem(&7u8);
    assert_eq!(a.n, BigUint::from(7u32));
  }
}