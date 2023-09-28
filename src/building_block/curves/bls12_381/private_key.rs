use crate::building_block::curves::bls12_381::params::Params as P;
use num_bigint::{
  BigUint,
  RandBigInt,
};
use num_traits::One;

pub struct PrivateKey {
  pub value: BigUint,
}

impl PrivateKey {
  pub fn new() -> Self {
    let mut rng = rand::thread_rng();

    // integer between 1 and r-1 where r is the subgroup order
    let value = rng.gen_biguint_range(
      &BigUint::one(),
      P::subgroup().order_ref()
    );

    Self { value }
  }
}

