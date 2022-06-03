use num_bigint::BigUint;

pub trait ToBigUint {
  fn to_biguint(&self) -> BigUint;
}

macro_rules! impl_to_biguint_for {
  ($name: ty) => {
    impl ToBigUint for $name {
      fn to_biguint(&self) -> BigUint {
        BigUint::from(*self)
      }
    }
  };
}

impl_to_biguint_for!(u8);
impl_to_biguint_for!(u16);
impl_to_biguint_for!(u32);
impl_to_biguint_for!(u64);
impl_to_biguint_for!(u128);
