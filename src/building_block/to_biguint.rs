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
impl_to_biguint_for!(usize);

impl ToBigUint for bool {
  fn to_biguint(&self) -> BigUint {
    let x = if *self { 1u8 } else { 0u8 };
    BigUint::from(x)
  }
}

impl ToBigUint for BigUint {
  fn to_biguint(&self) -> BigUint {
    self.clone()
  }
}
