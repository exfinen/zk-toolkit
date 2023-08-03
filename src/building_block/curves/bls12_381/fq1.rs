use crate::building_block::{
  curves::bls12_381::reduce::Reduce,
  field::{
    prime_field::PrimeField,
    prime_field_elem::PrimeFieldElem,
  },
};
use num_bigint::BigUint;
use std::{
  convert::From,
  fmt,
  sync::Arc,
};
use once_cell::sync::Lazy;

pub type Fq1 = PrimeFieldElem;

static BASE_FIELD: Lazy<Arc<PrimeField>> = Lazy::new(|| {
  let q = BigUint::parse_bytes(b"1a0111ea397fe69a4b1ba7b6434bacd764774b84f38512bf6730d2a0f6b0f6241eabfffeb153ffffb9feffffffffaaab", 16).unwrap();
  Arc::new(PrimeField::new(&q))
});

impl Fq1 {
  pub fn base_field() -> Arc<PrimeField> {
    BASE_FIELD.clone()
  }

  pub fn is_fq1_zero(&self) -> bool {
    self.is_zero()
  }

  pub fn fq1_zero() -> Fq1 {
    let f = Fq1::base_field();
    PrimeFieldElem::new(&f, &BigUint::from(0u8))
  }

  // not using From trait since that would implment the trait to PrimeFieldElem
  pub fn from(buf: &[u8]) -> Self {
    let f = Fq1::base_field();
    let n = BigUint::parse_bytes(buf, 16).unwrap();
    PrimeFieldElem::new(&f, &n)
  }
}

impl Reduce for Fq1 {
  fn reduce(&self) -> Self {
    self.clone()
  }
}

impl fmt::Display for Fq1 {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self)
  }
}
