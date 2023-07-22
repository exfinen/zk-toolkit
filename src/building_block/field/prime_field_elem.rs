use crate::building_block::{
  additive_identity::AdditiveIdentity,
  field::{
    field_elem_ops::Inverse,
    prime_field::PrimeField,
    prime_field_elems::PrimeFieldElems,
  },
  to_biguint::ToBigUint,
  zero::Zero,
};
use num_bigint::BigUint;
use std::{
  cmp::{PartialOrd, Ord, Ordering},
  ops,
  ops::{Deref, Rem},
};
use num_traits::{One, Zero as NumTraitsZero};
use bitvec::{
  prelude::Lsb0,
  view::BitView,
};

#[derive(Debug, Clone, Hash)]
pub struct PrimeFieldElem {
  pub f: PrimeField,
  pub e: BigUint,
}

impl Inverse for PrimeFieldElem {
  fn inv(&self) -> Self {
    self.safe_inv().unwrap()
  }
}

impl AdditiveIdentity<PrimeFieldElem> for PrimeFieldElem {
  fn get_additive_identity(&self) -> PrimeFieldElem {
    PrimeFieldElem {
      f: self.f.clone(),
      e: BigUint::from(0u8)
    }
  }
}

impl ToBigUint for PrimeFieldElem {
  fn to_biguint(&self) -> BigUint {
    self.e.clone()
  }
}

impl Zero<PrimeFieldElem> for PrimeFieldElem {
  fn get_zero(t: &PrimeFieldElem) -> Self {
    PrimeFieldElem {
      f: t.f.clone(),
      e: BigUint::from(0u8),
    }
  }

  fn is_zero(&self) -> bool {
    BigUint::zero() == self.e
  }
}

impl PartialEq for PrimeFieldElem {
  fn eq(&self, other: &Self) -> bool {
    self.f == other.f && self.e == other.e
  }
}

impl Eq for PrimeFieldElem {}

impl Ord for PrimeFieldElem {
  fn cmp(&self, other: &Self) -> Ordering {
    self.e.cmp(&other.e)
  }
}

impl PartialOrd for PrimeFieldElem {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
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
    impl<'a> ops::Add<$rhs> for $target {
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

macro_rules! impl_sub {
  ($rhs: ty, $target: ty) => {
    impl<'a> ops::Sub<$rhs> for $target {
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
    impl<'a> ops::Mul<$rhs> for $target {
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

macro_rules! impl_div {
  ($rhs: ty, $target: ty) => {
    impl<'a> ops::Div<$rhs> for $target {
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

impl ops::Neg for PrimeFieldElem {
  type Output = Self;

  fn neg(self) -> Self::Output {
    self.negate()
  }
}

impl<'a> ops::Neg for &'a PrimeFieldElem {
  type Output = PrimeFieldElem;

  fn neg(self) -> Self::Output {
    self.negate()
  }
}

impl PrimeFieldElem {
  pub fn new(f: &PrimeField, e: &impl ToBigUint) -> Self {
    let e = e.to_biguint();
    let f = f.clone();
    if e.ge(&f.order) {
      let e = e.rem(&f.order);
      PrimeFieldElem { f, e }
    } else {
      PrimeFieldElem { f, e }
    }
  }

  pub fn inc(&mut self) -> () {
    self.e = self.plus(&1u8).e;
  }

  pub fn plus(&self, rhs: &impl ToBigUint) -> PrimeFieldElem {
    let rhs = rhs.to_biguint() % &self.f.order;
    let mut e = self.e.clone();
    e += &rhs;
    if e >= self.f.order {
      e -= &self.f.order;
    }
    PrimeFieldElem { f: self.f.clone(), e }
  }

  pub fn minus(&self, rhs: &impl ToBigUint) -> PrimeFieldElem {
    let rhs = rhs.to_biguint() % &self.f.order;
    let f = self.f.clone();
    if self.e < rhs {
      let diff = &rhs - &self.e;
      let e = &self.f.order - diff;
      PrimeFieldElem { f, e }
    } else {
      let mut e = self.e.clone();
      e -= &rhs;
      PrimeFieldElem { f, e }
    }
  }

  pub fn times(&self, rhs: &impl ToBigUint) -> PrimeFieldElem {
    let rhs = rhs.to_biguint() % &self.f.order;
    let mut e = self.e.clone();
    e *= &rhs.to_biguint();
    e %= &self.f.order;
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
      bit_value = (&bit_value * &bit_value) % &self.f.order;
      sum %= &self.f.order;
    }

    PrimeFieldElem { f: self.f.clone(), e: sum }
  }

  pub fn sq(&self) -> PrimeFieldElem {
    let mut e = self.e.clone();
    e *= &self.e;
    e %= &self.f.order;
    PrimeFieldElem { f: self.f.clone(), e }
  }

  pub fn cube(&self) -> PrimeFieldElem {
    let mut e = self.e.clone();
    e *= &self.e;
    e %= &self.f.order;
    e *= &self.e;
    e %= &self.f.order;
    PrimeFieldElem { f: self.f.clone(), e }
  }

  pub fn pow_seq(&self, e: usize) -> PrimeFieldElems {
    let mut xs = vec![];
    let mut x = self.f.elem(&1u8);

    for _ in 0..e {
      xs.push(x.clone());
      x = x * &self.e;
    }
    PrimeFieldElems::new(&xs)
  }

  pub fn repeat(&self, n: usize) -> PrimeFieldElems {
    let mut xs = vec![];
    for _ in 0..n {
      xs.push(self.clone());
    }
    return PrimeFieldElems::new(&xs);
  }

  // based on extended Euclidean algorithm
  pub fn safe_inv(&self) -> Result<PrimeFieldElem, String> {
    if self.e == BigUint::zero() {
      return Err("Cannot find inverse of zero".to_string());
    }
    // x0*a + y0*b = a
    // x1*a + y1*b = b
    let mut r0 = self.e.clone();  // initially equals to a
    let mut r1 = self.f.order.clone();  // initially equals to b
    let mut x0 = BigUint::one();
    let mut y0 = BigUint::zero();
    let mut x1 = BigUint::zero();
    let mut y1 = BigUint::one();

    while r1 != BigUint::zero() {
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
    if new_v < BigUint::zero() {
      while new_v < BigUint::zero() {
        new_v += &self.f.order;
      }
    } else {
      if &new_v >= &self.f.order {
        new_v %= &self.f.order;
      }
    }
    Ok(PrimeFieldElem { f: self.f.clone(), e: new_v.to_biguint() })
  }

  pub fn inv(&self) -> PrimeFieldElem {
    self.safe_inv().unwrap()
  }

  pub fn safe_div(&self, rhs: &impl ToBigUint) -> Result<PrimeFieldElem, String> {
    let rhs = rhs.to_biguint() % &self.f.order;
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
      let mut e = self.f.order.clone();
      e -= &self.e;
      PrimeFieldElem { f, e }
    }
  }
}