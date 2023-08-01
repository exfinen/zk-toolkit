use crate::building_block::{
  curves::bls12_381::reduce::Reduce,
  field::{
    prime_field::PrimeField,
    prime_field_elem::PrimeFieldElem,
  },
};
use num_bigint::BigUint;
use std::{
  fmt,
  rc::Rc,
};

pub type Fq1 = PrimeFieldElem;

impl Fq1 {
  pub fn base_field() -> PrimeField {
    let q = BigUint::parse_bytes(b"1a0111ea397fe69a4b1ba7b6434bacd764774b84f38512bf6730d2a0f6b0f6241eabfffeb153ffffb9feffffffffaaab", 16).unwrap();
    PrimeField::new(&q)
  }

  pub fn is_fq1_zero(&self) -> bool {
    self.is_zero()
  }

  pub fn fq1_zero() -> Fq1 {
    let f = Rc::new(Fq1::base_field());
    PrimeFieldElem::new(&f, &BigUint::from(0u8))
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
