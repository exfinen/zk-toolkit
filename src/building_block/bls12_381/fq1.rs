use crate::building_block::{
  bls12_381::{
    additional_ops::AdditionalOps,
    setup::BASE_FIELD,
  },
  field::FieldElem,
};
use once_cell::sync::Lazy;
use std::fmt;

pub type Fq1 = FieldElem;

impl AdditionalOps for Fq1 {
  fn reduce(n: &Self) -> Self {
    n.clone()
  }

  fn inv(n: &Self) -> Self {
    n.inv()
  }

  fn zero() -> Self {
    static ZERO: Lazy<FieldElem> = Lazy::new(|| {
      let f = BASE_FIELD.clone();
      f.elem(&0u8)
    });
    ZERO.clone()
  }
}

impl fmt::Display for Fq1 {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.n)
  }
}