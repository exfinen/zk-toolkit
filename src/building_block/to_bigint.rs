use num_bigint::BigInt;

pub trait ToBigInt {
  fn to_bigint(&self) -> BigInt;
}

macro_rules! impl_to_bigint_for {
  ($name: ty) => {
    impl ToBigInt for $name {
      fn to_bigint(&self) -> BigInt {
        BigInt::from(*self)
      }
    }
  };
}

impl_to_bigint_for!(i8);
impl_to_bigint_for!(i16);
impl_to_bigint_for!(i32);
impl_to_bigint_for!(i64);
impl_to_bigint_for!(i128);
impl_to_bigint_for!(isize);

impl ToBigInt for BigInt {
  fn to_bigint(&self) -> BigInt {
    self.clone()
  }
}
