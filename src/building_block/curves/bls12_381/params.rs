use crate::building_block::field::prime_field::PrimeField;
use num_bigint::BigUint;
use std::sync::Arc;
use once_cell::sync::Lazy;

pub struct Params();

static BASE_PRIME_FIELD: Lazy<Arc<PrimeField>> = Lazy::new(|| {
  let q = BigUint::parse_bytes(b"1a0111ea397fe69a4b1ba7b6434bacd764774b84f38512bf6730d2a0f6b0f6241eabfffeb153ffffb9feffffffffaaab", 16).unwrap();
  Arc::new(PrimeField::new(&q))
});

static SUBGROUP: Lazy<Arc<PrimeField>> = Lazy::new(|| {
  let r = BigUint::parse_bytes(b"73eda753299d7d483339d80809a1d80553bda402fffe5bfeffffffff00000001", 16).unwrap();
  Arc::new(PrimeField::new(&r))
});

impl Params {
  pub fn base_prime_field() -> Arc<PrimeField> {
    BASE_PRIME_FIELD.clone()
  }

  pub fn subgroup() -> Arc<PrimeField> {
    SUBGROUP.clone()
  }

  pub fn embedding_degree() -> u32 {
    12u32
  }
}

