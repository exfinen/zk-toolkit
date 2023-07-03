use std::ops::{Add, Sub, Mul};
use crate::building_block::bls12_381::additional_ops::AdditionalOps;
use crate::building_block::field::FieldElem;

type Fq1 = FieldElem;

impl AdditionalOps for Fq1 {
  fn apply_reduce_rule(n: &Self) -> Self {
    n.clone()
  }

  fn inv(n: &Self) -> Self {
      n.inv()
  }
}
