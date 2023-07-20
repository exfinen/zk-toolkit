use crate::building_block::{
  additive_identity::AdditiveIdentity,
  elliptic_curve::weierstrass::curves::bls12_381::reduce::Reduce,
  field::{
    field_elem::NewFieldElem, prime_field_elem::PrimeFieldElem,
  },
  to_biguint::ToBigUint,
};
use num_bigint::BigUint;
use once_cell::sync::Lazy;
use std::fmt;

pub struct Fq1<E> {
  order: E,
}

impl<E> NewFieldElem<E> for Fq1<E> where E: NewFieldElem<E> {
  fn elem(n: &dyn ToBigUint) -> E {
    E::elem(n)
  }
}

impl<E> AdditiveIdentity<Fq1<E>> for Fq1<E> {
  fn get_additive_identity(&self) -> E {
    E::elem(0u8)
  }
}

pub static FIELD_ORDER: Lazy<BigUint> = Lazy::new(|| {
  BigUint::parse_bytes(b"1a0111ea397fe69a4b1ba7b6434bacd764774b84f38512bf6730d2a0f6b0f6241eabfffeb153ffffb9feffffffffaaab", 16).unwrap()
});

impl Fq1<PrimeFieldElem> {
  pub fn new() -> Self {
    Fq1 {
      order: FIELD_ORDER.clone(),
    }
  }
}

impl Reduce for Fq1<PrimeFieldElem> {
  fn reduce(n: &Self) -> Self {
    n.clone()
  }
}

impl fmt::Display for Fq1<PrimeFieldElem> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.n)
  }
}