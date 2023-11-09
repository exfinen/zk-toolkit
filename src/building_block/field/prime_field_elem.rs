use crate::building_block::{
  field::{
    prime_field::PrimeField,
    prime_field_elems::PrimeFieldElems,
  },
  to_biguint::ToBigUint,
};
use num_bigint::{BigUint, BigInt, ToBigInt};
use num_traits::{Zero as NumTraitZero, One, ToPrimitive};
use std::{
  cmp::{PartialOrd, Ord, Ordering},
  fmt,
  ops::{
    Add,
    Sub,
    Mul,
    Div,
    Neg,
    Rem,
    BitAnd,
    ShrAssign,
    Deref,
    AddAssign,
    MulAssign,
  },
  sync::Arc,
};
use bitvec::{
  prelude::Lsb0,
  view::BitView,
};

#[derive(Clone, Hash)]
pub struct PrimeFieldElem {
  pub f: Arc<PrimeField>,
  pub e: BigUint,
}

impl PrimeFieldElem {
  pub fn zero(&self) -> Self {  // not using Zero trait since it requires self
    PrimeFieldElem {
      f: self.f.clone(),
      e: BigUint::from(0u8),
    }
  }

  pub fn is_zero(&self) -> bool {
    BigUint::zero() == self.e
  }

  pub fn to_usize(&self) -> usize {
    (&self.e).try_into().unwrap()
  }
}

impl fmt::Debug for PrimeFieldElem {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{:?}", self.e)
  }
}

impl ToBigUint for PrimeFieldElem {
  fn to_biguint(&self) -> BigUint {
    self.e.clone()
  }
}

impl PartialEq for PrimeFieldElem {
  fn eq(&self, other: &PrimeFieldElem) -> bool {
    self.f == other.f && self.e == other.e
  }
}

impl Eq for PrimeFieldElem {}

impl Ord for PrimeFieldElem {
  fn cmp(&self, other: &PrimeFieldElem) -> Ordering {
    self.e.cmp(&other.e)
  }
}

impl PartialOrd for PrimeFieldElem {
  fn partial_cmp(&self, other: &PrimeFieldElem) -> Option<Ordering> {
    self.e.partial_cmp(&other.e)
  }
}

impl Deref for PrimeFieldElem {
  type Target = BigUint;

  fn deref(&self) -> &Self::Target {
    &self.e
  }
}

macro_rules! impl_add {
  ($rhs: ty, $target: ty) => {
    impl<'a> Add<$rhs> for $target {
      type Output = PrimeFieldElem;

      fn add(self, rhs: $rhs) -> Self::Output {
        self.plus(&rhs.to_biguint())
      }
    }
  };
}
impl_add!(u8, PrimeFieldElem);
impl_add!(u32, PrimeFieldElem);
impl_add!(PrimeFieldElem, &PrimeFieldElem);
impl_add!(&PrimeFieldElem, &PrimeFieldElem);
impl_add!(&PrimeFieldElem, PrimeFieldElem);
impl_add!(PrimeFieldElem, PrimeFieldElem);
impl_add!(&dyn ToBigUint, PrimeFieldElem);
impl_add!(BigUint, PrimeFieldElem);

impl AddAssign<PrimeFieldElem> for PrimeFieldElem {
  fn add_assign(&mut self, rhs: PrimeFieldElem) {
    *self = &*self + &rhs
  }
}

macro_rules! impl_sub {
  ($rhs: ty, $target: ty) => {
    impl<'a> Sub<$rhs> for $target {
      type Output = PrimeFieldElem;

      fn sub(self, rhs: $rhs) -> Self::Output {
        self.minus(&rhs.to_biguint())
      }
    }
  };
}
impl_sub!(u8, PrimeFieldElem);
impl_sub!(u32, PrimeFieldElem);
impl_sub!(PrimeFieldElem, &PrimeFieldElem);
impl_sub!(&PrimeFieldElem, &PrimeFieldElem);
impl_sub!(&PrimeFieldElem, PrimeFieldElem);
impl_sub!(PrimeFieldElem, PrimeFieldElem);
impl_sub!(&dyn ToBigUint, PrimeFieldElem);
impl_sub!(BigUint, PrimeFieldElem);

macro_rules! impl_mul {
  ($rhs: ty, $target: ty) => {
    impl<'a> Mul<$rhs> for $target {
      type Output = PrimeFieldElem;

      fn mul(self, rhs: $rhs) -> Self::Output {
        self.times(&rhs.to_biguint())
      }
    }
  };
}
impl_mul!(u8, PrimeFieldElem);
impl_mul!(u8, &PrimeFieldElem);
impl_mul!(u32, PrimeFieldElem);
impl_mul!(PrimeFieldElem, &PrimeFieldElem);
impl_mul!(&PrimeFieldElem, &PrimeFieldElem);
impl_mul!(&PrimeFieldElem, PrimeFieldElem);
impl_mul!(PrimeFieldElem, PrimeFieldElem);
impl_mul!(&dyn ToBigUint, PrimeFieldElem);
impl_mul!(BigUint, PrimeFieldElem);
impl_mul!(&BigUint, PrimeFieldElem);

impl MulAssign<PrimeFieldElem> for PrimeFieldElem {
  fn mul_assign(&mut self, rhs: PrimeFieldElem) {
    *self = &*self * &rhs;
  }
}

macro_rules! impl_div {
  ($rhs: ty, $target: ty) => {
    impl<'a> Div<$rhs> for $target {
      type Output = PrimeFieldElem;

      fn div(self, rhs: $rhs) -> Self::Output {
        self.divide_by(&rhs.to_biguint())
      }
    }
  };
}
impl_div!(u8, PrimeFieldElem);
impl_div!(u32, PrimeFieldElem);
impl_div!(PrimeFieldElem, &PrimeFieldElem);
impl_div!(&PrimeFieldElem, &PrimeFieldElem);
impl_div!(&PrimeFieldElem, PrimeFieldElem);
impl_div!(PrimeFieldElem, PrimeFieldElem);
impl_div!(&dyn ToBigUint, PrimeFieldElem);
impl_div!(BigUint, PrimeFieldElem);

macro_rules! impl_rem {
  ($rhs: ty, $target: ty) => {
    impl<'a> Rem<$rhs> for $target {
      type Output = PrimeFieldElem;

      fn rem(self, rhs: $rhs) -> Self::Output {
        self.plus(&rhs.to_biguint())
      }
    }
  };
}
impl_rem!(u8, PrimeFieldElem);
impl_rem!(u8, &PrimeFieldElem);
impl_rem!(u32, PrimeFieldElem);
impl_rem!(PrimeFieldElem, &PrimeFieldElem);
impl_rem!(&PrimeFieldElem, &PrimeFieldElem);
impl_rem!(&PrimeFieldElem, PrimeFieldElem);
impl_rem!(PrimeFieldElem, PrimeFieldElem);
impl_rem!(&dyn ToBigUint, PrimeFieldElem);
impl_rem!(BigUint, PrimeFieldElem);

macro_rules! impl_bit_and {
  ($rhs: ty, $target: ty) => {
    impl<'a> BitAnd<$rhs> for $target {
      type Output = PrimeFieldElem;

      fn bitand(self, rhs: $rhs) -> Self::Output {
        let res = &self.e & rhs.e.clone();
        PrimeFieldElem {
          f: self.f.clone(),
          e: res,
        }
      }
    }
  }
}
impl_bit_and!(PrimeFieldElem, &PrimeFieldElem);
impl_bit_and!(&PrimeFieldElem, &PrimeFieldElem);
impl_bit_and!(&PrimeFieldElem, PrimeFieldElem);
impl_bit_and!(PrimeFieldElem, PrimeFieldElem);

macro_rules! impl_shr_assign {
  ($rhs: ty, $target: ty) => {
    impl ShrAssign<$rhs> for $target {
      fn shr_assign(&mut self, rhs: $rhs) {
        let n = rhs.to_u64().unwrap();
        self.e >>= n;
      }
    }
  }
}
impl_shr_assign!(BigUint, PrimeFieldElem);
// impl_shr_assign!(BigUint, &PrimeFieldElem);
impl_shr_assign!(&BigUint, PrimeFieldElem);
// impl_shr_assign!(&BigUint, &PrimeFieldElem);

impl Neg for PrimeFieldElem {
  type Output = Self;

  fn neg(self) -> Self::Output {
    self.negate()
  }
}

impl<'a> Neg for &'a PrimeFieldElem {
  type Output = PrimeFieldElem;

  fn neg(self) -> Self::Output {
    self.negate()
  }
}

impl PrimeFieldElem {
  pub fn new(f: &Arc<PrimeField>, e: &impl ToBigUint) -> Self {
    let e = e.to_biguint();
    let f = f.clone();
    if e.ge(f.order_ref()) {
      let e = e.rem(f.order_ref());
      PrimeFieldElem { f, e }
    } else {
      PrimeFieldElem { f, e }
    }
  }

  pub fn inc(&mut self) -> () {
    self.e = self.plus(&1u8).e;
  }

  pub fn plus(&self, rhs: &impl ToBigUint) -> PrimeFieldElem {
    let rhs = rhs.to_biguint() % self.f.order_ref();
    let mut e = self.e.clone();
    e += &rhs;
    if &e >= self.f.order_ref() {
      e -= self.f.order_ref();
    }
    PrimeFieldElem { f: self.f.clone(), e }
  }

  pub fn minus(&self, rhs: &impl ToBigUint) -> PrimeFieldElem {
    let rhs = rhs.to_biguint() % self.f.order_ref();
    let f = self.f.clone();
    if self.e < rhs {
      let diff = &rhs - &self.e;
      let e = self.f.order_ref() - diff;
      PrimeFieldElem { f, e }
    } else {
      let mut e = self.e.clone();
      e -= &rhs;
      PrimeFieldElem { f, e }
    }
  }

  pub fn times(&self, rhs: &impl ToBigUint) -> PrimeFieldElem {
    let rhs = rhs.to_biguint() % self.f.order_ref();
    let mut e = self.e.clone();
    e *= &rhs.to_biguint();
    e %= self.f.order_ref();
    PrimeFieldElem { f: self.f.clone(), e }
  }

  // calculate w/ binary method
  pub fn pow(&self, rhs: &impl ToBigUint) -> PrimeFieldElem {
    let rhs = rhs.to_biguint();
    let rhs_le_bytes = rhs.to_bytes_le();

    let mut sum = BigUint::one();
    let mut bit_value = self.e.clone();
    let rhs_in_bits = rhs_le_bytes.view_bits::<Lsb0>();

    for bit in rhs_in_bits {
      if bit == true {
        sum *= &bit_value;
      }
      bit_value = (&bit_value * &bit_value) % self.f.order_ref();
      sum %= self.f.order_ref();
    }

    PrimeFieldElem { f: self.f.clone(), e: sum }
  }

  pub fn sq(&self) -> PrimeFieldElem {
    let mut e = self.e.clone();
    e *= &self.e;
    e %= self.f.order_ref();
    PrimeFieldElem { f: self.f.clone(), e }
  }

  pub fn cube(&self) -> PrimeFieldElem {
    let mut e = self.e.clone();
    e *= &self.e;
    e %= self.f.order_ref();
    e *= &self.e;
    e %= self.f.order_ref();
    PrimeFieldElem { f: self.f.clone(), e }
  }

  pub fn pow_seq(&self, n: &impl ToBigUint) -> PrimeFieldElems {
    let zero = &BigUint::from(0u8);
    let one = &BigUint::from(1u8);
    let n = n.to_biguint();

    let mut i = zero.clone();
    let mut xs = vec![];
    let mut x = self.f.elem(&1u8);

    while &i < &n {
      xs.push(x.clone());
      x = x * &self.e;
      i += one;
    }
    PrimeFieldElems::new(&xs)
  }

  pub fn repeat(&self, n: &impl ToBigUint) -> PrimeFieldElems {
    let zero = &BigUint::from(0u8);
    let one = &BigUint::from(1u8);
    let n = n.to_biguint();

    let mut i = zero.clone();
    let mut xs = vec![];

    while &i < &n {
      xs.push(self.clone());
      i += one;
    }
    PrimeFieldElems::new(&xs)
  }

  // based on extended Euclidean algorithm
  pub fn safe_inv(&self) -> Result<PrimeFieldElem, String> {
    if self.e == BigUint::zero() {
      return Err("Cannot find inverse of zero".to_string());
    }
    let order = self.f.order_ref().to_bigint().unwrap();
    let v = self.e.to_bigint().unwrap();
    let zero = BigInt::zero();
    let one = BigInt::one();

    // x0*a + y0*b = a
    // x1*a + y1*b = b
    let mut r0 = v.clone();  // initially equals to a
    let mut r1 = order.clone();  // initially equals to b
    let mut x0 = one.clone();
    let mut y0 = zero.clone();
    let mut x1 = zero.clone();
    let mut y1 = one.clone();

    while r1 != zero {
      // a mod b
      // = a - q*b
      // = (x0*a + y0*b) - q*(x1*a + y1*b)
      // = x0*a - q*x1*a + y0*b - q*y1*b
      // = (x0 - x1*q)*a + (y0 - y1*q)*b
      // = r
      let q = &r0 / &r1;
      let r2 = &r0 % &r1;
      // this produces the same result as above r2 using mod
      //let r2 = x2 * order + y2 * v;
      let x2 = &x0 - &x1 * &q;
      let y2 = &y0 - &y1 * &q;

      // do next calculation based on new and previous equations
      r0 = r1;
      r1 = r2;
      x0 = x1;
      y0 = y1;
      x1 = x2;
      y1 = y2;
    }

    // if the result is not a field element, convert it to a field element
    let mut new_v = x0;
    if new_v < zero.clone() {
      while new_v < zero.clone() {
        new_v += &order;
      }
    } else {
      if &new_v >= &order {
        new_v %= order;
      }
    }
    Ok(PrimeFieldElem { f: self.f.clone(), e: new_v.to_biguint().unwrap() })
  }

  pub fn inv(&self) -> PrimeFieldElem {
    self.safe_inv().unwrap()
  }

  pub fn safe_div(&self, rhs: &impl ToBigUint) -> Result<PrimeFieldElem, String> {
    let rhs = rhs.to_biguint() % self.f.order_ref();
    let inv = self.f.elem(&rhs.to_biguint()).safe_inv()?;
    Ok(self.times(&inv))
  }

  pub fn divide_by(&self, rhs: &impl ToBigUint) -> PrimeFieldElem {
    self.safe_div(rhs).unwrap()
  }

  pub fn negate(&self) -> PrimeFieldElem {
    let f = self.f.clone();
    if self.e == BigUint::zero() {
      PrimeFieldElem { f, e: self.e.clone() }
    } else {
      let mut e = self.f.order();
      e -= &self.e;
      PrimeFieldElem { f, e }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use num_traits::ToPrimitive;

  #[test]
  fn new_below_order() {
    let f = Arc::new(PrimeField::new(&11u8));
    let a = PrimeFieldElem::new(&f, &7u8);
    assert_eq!(a.e, BigUint::from(7u8));
  }

  #[test]
  fn new_above_order() {
    let f = Arc::new(PrimeField::new(&11u8));
    let a = PrimeFieldElem::new(&f, &13u8);
    assert_eq!(a.e, BigUint::from(2u32));
  }

  #[test]
  fn inc_below_order() {
    let f = Arc::new(PrimeField::new(&11u8));
    let mut a = PrimeFieldElem::new(&f, &1u8);
    a.inc();
    assert_eq!(a, f.elem(&2u8));
  }

  #[test]
  fn inc_above_order() {
    let f = Arc::new(PrimeField::new(&11u8));
    let mut a = PrimeFieldElem::new(&f, &10u8);
    a.inc();
    assert_eq!(a, f.elem(&0u8));
  }

  #[test]
  fn add_eq_order_result() {
    let f = Arc::new(PrimeField::new(&11u8));
    let a = PrimeFieldElem::new(&f, &9u8);
    let b = PrimeFieldElem::new(&f, &2u8);
    let c = a + &b;
    assert_eq!(c.e, BigUint::from(0u8));
  }

  #[test]
  fn add_below_order_result() {
    let f = Arc::new(PrimeField::new(&11u8));
    let a = PrimeFieldElem::new(&f, &9u8);
    let b = PrimeFieldElem::new(&f, &1u8);
    let c = a + &b;
    assert_eq!(c.e, BigUint::from(10u8));
  }

  #[test]
  fn add_above_order_result() {
    let f = Arc::new(PrimeField::new(&11u8));
    let a = PrimeFieldElem::new(&f, &9u8);
    let b = PrimeFieldElem::new(&f, &3u8);
    let c = a + b;
    assert_eq!(c.e, BigUint::from(1u8));
  }

  #[test]
  fn plus_above_order_times_2_result() {
    let f = Arc::new(PrimeField::new(&11u8));
    let a = PrimeFieldElem::new(&f, &3u8);
    let c = a.plus(&24u8);
    assert_eq!(c.e, BigUint::from(5u8));
  }

  #[test]
  fn sub_smaller_val() {
    let f = Arc::new(PrimeField::new(&11u8));
    let a = PrimeFieldElem::new(&f, &9u8);
    let b = PrimeFieldElem::new(&f, &2u8);
    let c = a - b;
    assert_eq!(c.e, BigUint::from(7u8));
  }

  #[test]
  fn sub_eq_val() {
    let f = Arc::new(PrimeField::new(&11u8));
    let a = PrimeFieldElem::new(&f, &9u8);
    let b = PrimeFieldElem::new(&f, &9u8);
    let c = a - b;
    assert_eq!(c.e, BigUint::zero());
  }

  #[test]
  fn sub_larger_val() {
    let f = Arc::new(PrimeField::new(&11u8));
    let a = PrimeFieldElem::new(&f, &9u8);
    let b = PrimeFieldElem::new(&f, &10u8);
    let c = a - b;
    assert_eq!(c.e, BigUint::from(10u8));
  }

  #[test]
  fn minus_order_times_2() {
    let f = Arc::new(PrimeField::new(&11u8));
    let a = PrimeFieldElem::new(&f, &3u8);
    let c = a.minus(&24u8);
    assert_eq!(c.e, BigUint::from(1u8));
  }

  #[test]
  fn mul_below_order_result() {
    let f = Arc::new(PrimeField::new(&11u8));
    let a = PrimeFieldElem::new(&f, &2u8);
    let b = PrimeFieldElem::new(&f, &5u8);
    let c = a * b;
    assert_eq!(c.e, BigUint::from(10u8));
  }

  #[test]
  fn mul_eq_order_result() {
    let f = Arc::new(PrimeField::new(&11u8));
    let a = PrimeFieldElem::new(&f, &1u8);
    let b = PrimeFieldElem::new(&f, &11u8);
    let c = a * b;
    assert_eq!(c.e, BigUint::from(0u32));
  }

  #[test]
  fn mul_above_order_result() {
    let f = Arc::new(PrimeField::new(&11u8));
    let a = PrimeFieldElem::new(&f, &3u8);
    let b = PrimeFieldElem::new(&f, &9u8);
    let c = a * b;
    assert_eq!(c.e, BigUint::from(5u8));
  }

  #[test]
  fn mul_u32_below_order_result() {
    let f = Arc::new(PrimeField::new(&11u8));
    let a = PrimeFieldElem::new(&f, &2u8);
    let b = a * 5u8;
    assert_eq!(b.e, BigUint::from(10u8));
  }

  #[test]
  fn test_mul_large_number() {
    let order = BigUint::parse_bytes(b"115792089237316195423570985008687907852837564279074904382605163141518161494337", 10).unwrap();
    let f = Arc::new(PrimeField::new(&order));

    let lhs_e = BigUint::from(1234u32);
    let lhs = PrimeFieldElem::new(&f, &lhs_e);

    let rhs_e = BigUint::parse_bytes(b"63954422509139660694275478881573291931659433822585593108077818434106113196321", 10).unwrap();
    let rhs = PrimeFieldElem::new(&f, &rhs_e);

    let exp_e = BigUint::parse_bytes(b"65344605666012213284100148944976995885360063020612010813911848313075706616617", 10).unwrap();
    let exp = PrimeFieldElem::new(&f, &exp_e);

    let act = lhs * rhs;
    println!("act={:?}", &act);
    assert_eq!(act, exp);
  }

  struct InvTestCase {
    order: u32,
    n: u32,
    exp: u32,
  }

  #[test]
  fn inv_small_primes() -> Result<(), String> {
    let test_cases = [
      // order 97
      InvTestCase { order: 97u32, n: 1u32, exp: 1u32 },
      InvTestCase { order: 97u32, n: 2u32, exp: 49u32 },
      InvTestCase { order: 97u32, n: 3u32, exp: 65u32 },
      InvTestCase { order: 97u32, n: 4u32, exp: 73u32 },
      InvTestCase { order: 97u32, n: 5u32, exp: 39u32 },
      InvTestCase { order: 97u32, n: 6u32, exp: 81u32 },
      InvTestCase { order: 97u32, n: 7u32, exp: 14u32 },
      InvTestCase { order: 97u32, n: 8u32, exp: 85u32 },
      InvTestCase { order: 97u32, n: 9u32, exp: 54u32 },
      InvTestCase { order: 97u32, n: 10u32, exp: 68u32 },
      InvTestCase { order: 97u32, n: 11u32, exp: 53u32 },
      InvTestCase { order: 97u32, n: 12u32, exp: 89u32 },
      InvTestCase { order: 97u32, n: 13u32, exp: 15u32 },
      InvTestCase { order: 97u32, n: 14u32, exp: 7u32 },
      InvTestCase { order: 97u32, n: 15u32, exp: 13u32 },
      InvTestCase { order: 97u32, n: 16u32, exp: 91u32 },
      InvTestCase { order: 97u32, n: 17u32, exp: 40u32 },
      InvTestCase { order: 97u32, n: 18u32, exp: 27u32 },
      InvTestCase { order: 97u32, n: 19u32, exp: 46u32 },
      InvTestCase { order: 97u32, n: 20u32, exp: 34u32 },
      InvTestCase { order: 97u32, n: 21u32, exp: 37u32 },
      InvTestCase { order: 97u32, n: 22u32, exp: 75u32 },
      InvTestCase { order: 97u32, n: 23u32, exp: 38u32 },
      InvTestCase { order: 97u32, n: 24u32, exp: 93u32 },
      InvTestCase { order: 97u32, n: 25u32, exp: 66u32 },
      InvTestCase { order: 97u32, n: 26u32, exp: 56u32 },
      InvTestCase { order: 97u32, n: 27u32, exp: 18u32 },
      InvTestCase { order: 97u32, n: 28u32, exp: 52u32 },
      InvTestCase { order: 97u32, n: 29u32, exp: 87u32 },
      InvTestCase { order: 97u32, n: 30u32, exp: 55u32 },
      InvTestCase { order: 97u32, n: 31u32, exp: 72u32 },
      InvTestCase { order: 97u32, n: 32u32, exp: 94u32 },
      InvTestCase { order: 97u32, n: 33u32, exp: 50u32 },
      InvTestCase { order: 97u32, n: 34u32, exp: 20u32 },
      InvTestCase { order: 97u32, n: 35u32, exp: 61u32 },
      InvTestCase { order: 97u32, n: 36u32, exp: 62u32 },
      InvTestCase { order: 97u32, n: 37u32, exp: 21u32 },
      InvTestCase { order: 97u32, n: 38u32, exp: 23u32 },
      InvTestCase { order: 97u32, n: 39u32, exp: 5u32 },
      InvTestCase { order: 97u32, n: 40u32, exp: 17u32 },
      InvTestCase { order: 97u32, n: 41u32, exp: 71u32 },
      InvTestCase { order: 97u32, n: 42u32, exp: 67u32 },
      InvTestCase { order: 97u32, n: 43u32, exp: 88u32 },
      InvTestCase { order: 97u32, n: 44u32, exp: 86u32 },
      InvTestCase { order: 97u32, n: 45u32, exp: 69u32 },
      InvTestCase { order: 97u32, n: 46u32, exp: 19u32 },
      InvTestCase { order: 97u32, n: 47u32, exp: 64u32 },
      InvTestCase { order: 97u32, n: 48u32, exp: 95u32 },
      InvTestCase { order: 97u32, n: 49u32, exp: 2u32 },
      InvTestCase { order: 97u32, n: 50u32, exp: 33u32 },
      InvTestCase { order: 97u32, n: 51u32, exp: 78u32 },
      InvTestCase { order: 97u32, n: 52u32, exp: 28u32 },
      InvTestCase { order: 97u32, n: 53u32, exp: 11u32 },
      InvTestCase { order: 97u32, n: 54u32, exp: 9u32 },
      InvTestCase { order: 97u32, n: 55u32, exp: 30u32 },
      InvTestCase { order: 97u32, n: 56u32, exp: 26u32 },
      InvTestCase { order: 97u32, n: 57u32, exp: 80u32 },
      InvTestCase { order: 97u32, n: 58u32, exp: 92u32 },
      InvTestCase { order: 97u32, n: 59u32, exp: 74u32 },
      InvTestCase { order: 97u32, n: 60u32, exp: 76u32 },
      InvTestCase { order: 97u32, n: 61u32, exp: 35u32 },
      InvTestCase { order: 97u32, n: 62u32, exp: 36u32 },
      InvTestCase { order: 97u32, n: 63u32, exp: 77u32 },
      InvTestCase { order: 97u32, n: 64u32, exp: 47u32 },
      InvTestCase { order: 97u32, n: 65u32, exp: 3u32 },
      InvTestCase { order: 97u32, n: 66u32, exp: 25u32 },
      InvTestCase { order: 97u32, n: 67u32, exp: 42u32 },
      InvTestCase { order: 97u32, n: 68u32, exp: 10u32 },
      InvTestCase { order: 97u32, n: 69u32, exp: 45u32 },
      InvTestCase { order: 97u32, n: 70u32, exp: 79u32 },
      InvTestCase { order: 97u32, n: 71u32, exp: 41u32 },
      InvTestCase { order: 97u32, n: 72u32, exp: 31u32 },
      InvTestCase { order: 97u32, n: 73u32, exp: 4u32 },
      InvTestCase { order: 97u32, n: 74u32, exp: 59u32 },
      InvTestCase { order: 97u32, n: 75u32, exp: 22u32 },
      InvTestCase { order: 97u32, n: 76u32, exp: 60u32 },
      InvTestCase { order: 97u32, n: 77u32, exp: 63u32 },
      InvTestCase { order: 97u32, n: 78u32, exp: 51u32 },
      InvTestCase { order: 97u32, n: 79u32, exp: 70u32 },
      InvTestCase { order: 97u32, n: 80u32, exp: 57u32 },
      InvTestCase { order: 97u32, n: 81u32, exp: 6u32 },
      InvTestCase { order: 97u32, n: 82u32, exp: 84u32 },
      InvTestCase { order: 97u32, n: 83u32, exp: 90u32 },
      InvTestCase { order: 97u32, n: 84u32, exp: 82u32 },
      InvTestCase { order: 97u32, n: 85u32, exp: 8u32 },
      InvTestCase { order: 97u32, n: 86u32, exp: 44u32 },
      InvTestCase { order: 97u32, n: 87u32, exp: 29u32 },
      InvTestCase { order: 97u32, n: 88u32, exp: 43u32 },
      InvTestCase { order: 97u32, n: 89u32, exp: 12u32 },
      InvTestCase { order: 97u32, n: 90u32, exp: 83u32 },
      InvTestCase { order: 97u32, n: 91u32, exp: 16u32 },
      InvTestCase { order: 97u32, n: 92u32, exp: 58u32 },
      InvTestCase { order: 97u32, n: 93u32, exp: 24u32 },
      InvTestCase { order: 97u32, n: 94u32, exp: 32u32 },
      InvTestCase { order: 97u32, n: 95u32, exp: 48u32 },
      InvTestCase { order: 97u32, n: 96u32, exp: 96u32 },

      // order 53
      InvTestCase { order: 53u32, n: 1u32, exp: 1u32 },
      InvTestCase { order: 53u32, n: 2u32, exp: 27u32 },
      InvTestCase { order: 53u32, n: 3u32, exp: 18u32 },
      InvTestCase { order: 53u32, n: 4u32, exp: 40u32 },
      InvTestCase { order: 53u32, n: 5u32, exp: 32u32 },
      InvTestCase { order: 53u32, n: 6u32, exp: 9u32 },
      InvTestCase { order: 53u32, n: 7u32, exp: 38u32 },
      InvTestCase { order: 53u32, n: 8u32, exp: 20u32 },
      InvTestCase { order: 53u32, n: 9u32, exp: 6u32 },
      InvTestCase { order: 53u32, n: 10u32, exp: 16u32 },
      InvTestCase { order: 53u32, n: 11u32, exp: 29u32 },
      InvTestCase { order: 53u32, n: 12u32, exp: 31u32 },
      InvTestCase { order: 53u32, n: 13u32, exp: 49u32 },
      InvTestCase { order: 53u32, n: 14u32, exp: 19u32 },
      InvTestCase { order: 53u32, n: 15u32, exp: 46u32 },
      InvTestCase { order: 53u32, n: 16u32, exp: 10u32 },
      InvTestCase { order: 53u32, n: 17u32, exp: 25u32 },
      InvTestCase { order: 53u32, n: 18u32, exp: 3u32 },
      InvTestCase { order: 53u32, n: 19u32, exp: 14u32 },
      InvTestCase { order: 53u32, n: 20u32, exp: 8u32 },
      InvTestCase { order: 53u32, n: 21u32, exp: 48u32 },
      InvTestCase { order: 53u32, n: 22u32, exp: 41u32 },
      InvTestCase { order: 53u32, n: 23u32, exp: 30u32 },
      InvTestCase { order: 53u32, n: 24u32, exp: 42u32 },
      InvTestCase { order: 53u32, n: 25u32, exp: 17u32 },
      InvTestCase { order: 53u32, n: 26u32, exp: 51u32 },
      InvTestCase { order: 53u32, n: 27u32, exp: 2u32 },
      InvTestCase { order: 53u32, n: 28u32, exp: 36u32 },
      InvTestCase { order: 53u32, n: 29u32, exp: 11u32 },
      InvTestCase { order: 53u32, n: 30u32, exp: 23u32 },
      InvTestCase { order: 53u32, n: 31u32, exp: 12u32 },
      InvTestCase { order: 53u32, n: 32u32, exp: 5u32 },
      InvTestCase { order: 53u32, n: 33u32, exp: 45u32 },
      InvTestCase { order: 53u32, n: 34u32, exp: 39u32 },
      InvTestCase { order: 53u32, n: 35u32, exp: 50u32 },
      InvTestCase { order: 53u32, n: 36u32, exp: 28u32 },
      InvTestCase { order: 53u32, n: 37u32, exp: 43u32 },
      InvTestCase { order: 53u32, n: 38u32, exp: 7u32 },
      InvTestCase { order: 53u32, n: 39u32, exp: 34u32 },
      InvTestCase { order: 53u32, n: 40u32, exp: 4u32 },
      InvTestCase { order: 53u32, n: 41u32, exp: 22u32 },
      InvTestCase { order: 53u32, n: 42u32, exp: 24u32 },
      InvTestCase { order: 53u32, n: 43u32, exp: 37u32 },
      InvTestCase { order: 53u32, n: 44u32, exp: 47u32 },
      InvTestCase { order: 53u32, n: 45u32, exp: 33u32 },
      InvTestCase { order: 53u32, n: 46u32, exp: 15u32 },
      InvTestCase { order: 53u32, n: 47u32, exp: 44u32 },
      InvTestCase { order: 53u32, n: 48u32, exp: 21u32 },
      InvTestCase { order: 53u32, n: 49u32, exp: 13u32 },
      InvTestCase { order: 53u32, n: 50u32, exp: 35u32 },
      InvTestCase { order: 53u32, n: 51u32, exp: 26u32 },
      InvTestCase { order: 53u32, n: 52u32, exp: 52u32 },

      // order 11
      InvTestCase { order: 11u32, n: 1u32, exp: 1u32 },
      InvTestCase { order: 11u32, n: 2u32, exp: 6u32 },
      InvTestCase { order: 11u32, n: 3u32, exp: 4u32 },
      InvTestCase { order: 11u32, n: 4u32, exp: 3u32 },
      InvTestCase { order: 11u32, n: 5u32, exp: 9u32 },
      InvTestCase { order: 11u32, n: 6u32, exp: 2u32 },
      InvTestCase { order: 11u32, n: 7u32, exp: 8u32 },
      InvTestCase { order: 11u32, n: 8u32, exp: 7u32 },
      InvTestCase { order: 11u32, n: 9u32, exp: 5u32 },
      InvTestCase { order: 11u32, n: 10u32, exp: 10u32 },
    ];

    for x in test_cases {
      let f = Arc::new(PrimeField::new(&x.order));
      let a = PrimeFieldElem::new(&f, &x.n);
      let inv = a.safe_inv()?;
      assert_eq!(inv.e, BigUint::from(x.exp));
    }
    Ok(())
  }

  #[test]
  fn div() {
    let f = Arc::new(PrimeField::new(&11u8));
    let a = PrimeFieldElem::new(&f, &4u8);
    let b = PrimeFieldElem::new(&f, &2u8);
    let c = a.safe_div(&b).unwrap();
    assert_eq!(c.e, BigUint::from(2u32));
  }

  #[test]
  fn inv_secp256k1() -> Result<(), String> {
    let p = BigUint::parse_bytes(b"FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F", 16).unwrap();
    let f = Arc::new(PrimeField::new(&p));
    let a = PrimeFieldElem::new(&f, &1112121212121u64);

    let exp = BigUint::parse_bytes(b"52624297956533532283067125375510330718705195823487497799082320305224600546911", 10).unwrap();
    let inv = a.safe_inv()?;
    assert_eq!(exp, inv.e);
    Ok(())
  }

  #[test]
  fn neg() {
    let f = Arc::new(PrimeField::new(&11u8));
    let a = PrimeFieldElem::new(&f, &5u8);
    assert_eq!(a.negate().e, BigUint::from(6u8));

    let neg_a = a.clone() + &a.negate();
    assert_eq!(neg_a.e, BigUint::from(0u32));
  }

  #[test]
  fn cube() {
    {
      // when value is less than field order
      let f = Arc::new(PrimeField::new(&100u8));
      let a = PrimeFieldElem::new(&f, &3u8);
      let b = a.cube();
      assert_eq!(b.e.to_u8().unwrap(), 27);
    }
    {
      // when value is larger than field order
      let f = Arc::new(PrimeField::new(&11u8));
      let a = PrimeFieldElem::new(&f, &3u8);
      let b = a.cube();
      assert_eq!(b.e.to_u8().unwrap(), 5);
    }
  }

  #[test]
  fn pow_seq() {
    {
      // when value is less than field order
      let f = Arc::new(PrimeField::new(&100u8));
      let a = PrimeFieldElem::new(&f, &3u8);
      let xs = a.pow_seq(&4u8);
      assert_eq!(xs.len(), 4);
      assert_eq!(xs[0].e.to_u8().unwrap(), 1);
      assert_eq!(xs[1].e.to_u8().unwrap(), 3);
      assert_eq!(xs[2].e.to_u8().unwrap(), 9);
      assert_eq!(xs[3].e.to_u8().unwrap(), 27);
    }
    {
      // when value is larger than field order
      let f = Arc::new(PrimeField::new(&11u8));
      let a = PrimeFieldElem::new(&f, &3u8);
      let xs = a.pow_seq(&4u8);
      assert_eq!(xs.len(), 4);
      assert_eq!(xs[0].e.to_u8().unwrap(), 1);
      assert_eq!(xs[1].e.to_u8().unwrap(), 3);
      assert_eq!(xs[2].e.to_u8().unwrap(), 9);
      assert_eq!(xs[3].e.to_u8().unwrap(), 5);
    }
  }

  #[test]
  fn repeat() {
    let f = Arc::new(PrimeField::new(&11u8));
    let a = PrimeFieldElem::new(&f, &5u8);
    let xs = a.repeat(&3u8);
    assert_eq!(xs.len(), 3);
    assert_eq!(xs[0].e.to_u8().unwrap(), 5);
    assert_eq!(xs[1].e.to_u8().unwrap(), 5);
    assert_eq!(xs[2].e.to_u8().unwrap(), 5);
  }

  #[test]
  fn pow_various_combinations() {
    let f = Arc::new(PrimeField::new(&100000000u128));
    let test_cases = [
      (2u128, 0u128, 1u128),
      (2u128, 1u128, 2u128),
      (2u128, 2u128, 4u128),
      (2u128, 3u128, 8u128),
      (3u128, 0u128, 1u128),
      (3u128, 1u128, 3u128),
      (3u128, 2u128, 9u128),
      (3u128, 3u128, 27u128),
      (17u128, 7u128, 10338673u128),
    ];

    for t in test_cases {
      let base = PrimeFieldElem::new(&f, &t.0);
      let exponent = BigUint::from(t.1);
      let expected = BigUint::from(t.2);
      assert_eq!(base.pow(&exponent).e, BigUint::from(expected));
    }
  }

  #[test]
  fn pow_below_order() {
    let f = Arc::new(PrimeField::new(&11u8));
    let a = PrimeFieldElem::new(&f, &2u8);
    assert_eq!(a.pow(&3u8).e, BigUint::from(8u8));
  }

  #[test]
  fn pow_above_order() {
    let f = Arc::new(PrimeField::new(&11u8));
    let a = PrimeFieldElem::new(&f, &2u8);
    assert_eq!(a.pow(&4u8).e, BigUint::from(5u8));
  }

  #[test]
  fn sq_below_order() {
    let f = Arc::new(PrimeField::new(&11u8));
    let a = PrimeFieldElem::new(&f, &2u8);
    assert_eq!(a.sq().e, BigUint::from(4u8));
  }

  #[test]
  fn sq_above_order() {
    let f = Arc::new(PrimeField::new(&11u8));
    let a = PrimeFieldElem::new(&f, &4u8);
    assert_eq!(a.sq().e, BigUint::from(5u8));
  }
  #[test]
  fn new_elem_from_biguint() {
    let f = PrimeField::new(&11u8);
    let a = f.elem(&7u8);
    assert_eq!(a.e, BigUint::from(7u8));
  }

  #[test]
  fn new_elem_from_u8() {
    let f = PrimeField::new(&11u8);
    let a = f.elem(&7u8);
    assert_eq!(a.e, BigUint::from(7u8));
  }

  #[test]
  fn new_elem_from_pos_signed_int() {
    let f = PrimeField::new(&11u8);
    let a = f.elem_from_signed(&7);
    assert_eq!(a.e, BigUint::from(7u8));
  }

  #[test]
  fn new_elem_from_neg_signed_int() {
    let f = PrimeField::new(&11u8);
    let a = f.elem_from_signed(&-7);
    assert_eq!(a.e, BigUint::from(4u8));
  }
}
