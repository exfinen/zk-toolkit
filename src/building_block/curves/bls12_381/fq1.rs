use crate::building_block::{
  curves::bls12_381::{
    fq2::Fq2,
    reduce::Reduce,
  },
  field::{
    prime_field::PrimeField,
    prime_field_elem::PrimeFieldElem,
  },
  to_biguint::ToBigUint,
};
use num_bigint::BigUint;
use std::{
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
  pub fn from_u8_slice(buf: &[u8]) -> Self {
    let f = Fq1::base_field();
    let n = BigUint::parse_bytes(buf, 16).unwrap();
    PrimeFieldElem::new(&f, &n)
  }

  // not using From trait since that would implment the trait to PrimeFieldElem
  pub fn from_to_biguint(n: &dyn ToBigUint) -> Self {
    Fq1::new(&Fq1::base_field(), &n.to_biguint())
  }

  // not using Into trait since that would implment the trait to PrimeFieldElem
  pub fn into_fq2(&self) -> Fq2 {
    Fq2::new(&Fq1::fq1_zero(), self)
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
