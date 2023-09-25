use crate::building_block::{
  curves::bls12_381::{
    fq2::Fq2,
    params::Params as P,
    reduce::Reduce,
  },
  field::prime_field_elem::PrimeFieldElem,
  to_biguint::ToBigUint,
};
use num_bigint::BigUint;
use std::fmt;

pub type Fq1 = PrimeFieldElem;

impl Fq1 {
  pub fn is_fq1_zero(&self) -> bool {
    self.is_zero()
  }

  pub fn fq1_zero() -> Fq1 {
    let f = P::base_prime_field();
    PrimeFieldElem::new(&f, &BigUint::from(0u8))
  }

  // not using From trait since that would implment the trait to PrimeFieldElem
  pub fn from_u8_slice(buf: &[u8]) -> Self {
    let f = P::base_prime_field();
    let n = BigUint::parse_bytes(buf, 16).unwrap();
    PrimeFieldElem::new(&f, &n)
  }

  // not using From trait since that would implment the trait to PrimeFieldElem
  pub fn from_to_biguint(n: &dyn ToBigUint) -> Self {
    Fq1::new(&P::base_prime_field(), &n.to_biguint())
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
    let mut s = self.to_str_radix(16).to_uppercase();
    if s.len() < 96 {
      s = "0".repeat(96 - s.len()) + &s;
    }
    write!(f, "{} {} {} {} {} {}", 
      &s[0..16],
      &s[16..32],
      &s[32..48],
      &s[48..64],
      &s[64..80],
      &s[80..96],
    )
  }
}

