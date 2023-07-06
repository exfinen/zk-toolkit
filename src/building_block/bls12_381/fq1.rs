use crate::building_block::{
  bls12_381::{
    additional_ops::AdditionalOps,
    setup::BASE_FIELD,
  },
  field::FieldElem,
};
use once_cell::sync::Lazy;

pub type Fq1 = FieldElem;

impl AdditionalOps for Fq1 {
  fn apply_reduce_rule(n: &Self) -> Self {
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
