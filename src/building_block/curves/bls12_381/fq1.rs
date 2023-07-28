use crate::building_block::{
  elliptic_curve::weierstrass::curves::bls12_381::reduce::Reduce,
  field::prime_field_elem::PrimeFieldElem,
};
use num_bigint::BigUint;
use once_cell::sync::Lazy;
use std::fmt;

pub type Fq1 = PrimeFieldElem;

pub static FIELD_ORDER: Lazy<BigUint> = Lazy::new(|| {
  BigUint::parse_bytes(b"1a0111ea397fe69a4b1ba7b6434bacd764774b84f38512bf6730d2a0f6b0f6241eabfffeb153ffffb9feffffffffaaab", 16).unwrap()
});

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