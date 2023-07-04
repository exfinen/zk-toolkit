use crate::building_block::bls12_381::additional_ops::AdditionalOps;
use crate::building_block::field::{Field, FieldElem};
use num_bigint::BigUint;
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
      let order = BigUint::parse_bytes(b"0x1a0111ea397fe69a4b1ba7b6434bacd764774b84f38512bf6730d2a0f6b0f6241eabfffeb153ffffb9feffffffffaaab", 16).unwrap();
      let f = Field::new(&order);
      f.elem(&0u8)
    });
    ZERO.clone()
  }
}
