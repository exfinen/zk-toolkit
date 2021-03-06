use std::rc::Rc;
use std::ops;
use num_bigint::{BigUint, BigInt, ToBigInt};
use num_traits::{Zero, One};
use core::ops::Rem;
use bitvec::prelude::*;
use rand::RngCore;
use crate::to_biguint::ToBigUint;
use crate::random_number::RandomNumber;
use std::ops::{Range, Index, RangeFrom, RangeTo, Deref};

///////////////
// FieldElem

#[derive(Debug, Clone)]
pub struct FieldElem {
  pub f: Field,
  pub n: BigUint,
}

impl Deref for FieldElem {
  type Target = BigUint;

  fn deref(&self) -> &Self::Target {
    &self.n
  }
}


impl PartialEq for FieldElem {
  fn eq(&self, other: &Self) -> bool {
    self.f == other.f && self.n == other.n
  }
}

impl Eq for FieldElem {}

impl ops::Add<&dyn ToBigUint> for FieldElem {
  type Output = Self;

  fn add(self, rhs: &dyn ToBigUint) -> Self::Output {
    self.plus(&rhs.to_biguint())
  }
}

impl<'a> ops::Add<&dyn ToBigUint> for &'a FieldElem {
  type Output = FieldElem;

  fn add(self, rhs: &dyn ToBigUint) -> Self::Output {
    self.plus(&rhs.to_biguint())
  }
}

impl ops::Sub<&dyn ToBigUint> for FieldElem {
  type Output = Self;

  fn sub(self, rhs: &dyn ToBigUint) -> Self::Output {
    self.minus(&rhs.to_biguint())
  }
}

impl<'a> ops::Sub<&dyn ToBigUint> for &'a FieldElem {
  type Output = FieldElem;

  fn sub(self, rhs: &dyn ToBigUint) -> Self::Output {
    self.minus(&rhs.to_biguint())
  }
}

impl ops::Mul<&dyn ToBigUint> for FieldElem {
  type Output = Self;

  fn mul(self, rhs: &dyn ToBigUint) -> Self::Output {
    self.times(&rhs.to_biguint())
  }
}

impl<'a> ops::Mul<&dyn ToBigUint> for &'a FieldElem {
  type Output = FieldElem;

  fn mul(self, rhs: &dyn ToBigUint) -> Self::Output {
    self.times(&rhs.to_biguint())
  }
}

impl ops::Div<&dyn ToBigUint> for FieldElem {
  type Output = Self;

  fn div(self, rhs: &dyn ToBigUint) -> Self::Output {
    self.divide_by(&rhs.to_biguint())
  }
}

impl<'a> ops::Div<&dyn ToBigUint> for &'a FieldElem {
  type Output = FieldElem;

  fn div(self, rhs: &dyn ToBigUint) -> Self::Output {
    self.divide_by(&rhs.to_biguint())
  }
}

impl ops::Neg for FieldElem {
  type Output = Self;

  fn neg(self) -> Self::Output {
    self.negate()
  }
}

impl<'a> ops::Neg for &'a FieldElem {
  type Output = FieldElem;

  fn neg(self) -> Self::Output {
    self.negate()
  }
}

impl FieldElem {
  pub fn new(f: &Field, n: &impl ToBigUint) -> Self {
    let n = n.to_biguint();
    let f = f.clone();
    if n.ge(&f.order) {
      let n = n.rem(&(*f.order));
      FieldElem { f, n }
    } else {
      FieldElem { f, n }
    }
  }

  pub fn plus(&self, rhs: &impl ToBigUint) -> FieldElem {
    let mut n = self.n.clone();
    n += &rhs.to_biguint();
    if n >= *self.f.order {
      n -= &(*self.f.order);
    }
    FieldElem { f: self.f.clone(), n }
  }

  pub fn minus(&self, rhs: &impl ToBigUint) -> FieldElem {
    let rhs = rhs.to_biguint();
    let f = self.f.clone();
    if self.n < rhs {
      let diff = &rhs - &self.n;
      let n = &(*self.f.order) - diff;
      FieldElem { f, n }
    } else {
      let mut n = self.n.clone();
      n -= &rhs;
      FieldElem { f, n }
    }
  }

  pub fn times(&self, rhs: &impl ToBigUint) -> FieldElem {
    let mut n = self.n.clone();
    n *= &rhs.to_biguint();
    n %= &(*self.f.order);
    if n < BigUint::zero() {
      n += &(*self.f.order);
    }
    FieldElem { f: self.f.clone(), n }
  }

  // calculate w/ binary method
  pub fn pow(&self, rhs: &impl ToBigUint) -> FieldElem {
    let rhs = rhs.to_biguint();
    let rhs_le_bytes = rhs.to_bytes_le();

    let mut sum = BigUint::one();
    let mut bit_value = self.n.clone();
    let rhs_in_bits = rhs_le_bytes.view_bits::<Lsb0>();

    for bit in rhs_in_bits {
      if bit == true {
        sum *= &bit_value;
      }
      bit_value = (&bit_value * &bit_value) % &*self.f.order;
      sum %= &(*self.f.order);
    }

    FieldElem { f: self.f.clone(), n: sum }
  }

  pub fn sq(&self) -> FieldElem {
    let mut n = self.n.clone();
    n *= &self.n;
    n %= &(*self.f.order);
    FieldElem { f: self.f.clone(), n }
  }

  // based on extended Euclidean algorithm
  pub fn safe_inv(&self) -> Result<FieldElem, String> {
    if self.n == BigUint::zero() {
      return Err("Cannot find inverse of zero".to_string());
    }
    let order = self.f.order.to_bigint().unwrap();
    let v = self.n.to_bigint().unwrap();
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

      // do next calculattion based on new and previous equations
      r0 = r1;
      r1 = r2;
      x0 = x1;
      y0 = y1;
      x1 = x2;
      y1 = y2;
    }

    // if the result is not field element, convert it to field element
    let mut new_v = x0;
    if new_v < zero.clone() {
      while new_v < zero.clone() {
        new_v += order.clone();
      }
    } else {
      if new_v >= order {
        new_v %= order;
      }
    }
    Ok(FieldElem { f: self.f.clone(), n: new_v.to_biguint().unwrap() })
  }

  pub fn inv(&self) -> FieldElem {
    self.safe_inv().unwrap()
  }

  pub fn safe_div(&self, other: &impl ToBigUint) -> Result<FieldElem, String> {
    let inv = self.f.elem(&other.to_biguint()).safe_inv()?;
    Ok(self.times(&inv))
  }

  pub fn divide_by(&self, other: &impl ToBigUint) -> FieldElem {
    self.safe_div(other).unwrap()
  }

  pub fn negate(&self) -> FieldElem {
    let f = self.f.clone();
    if self.n == BigUint::zero() {
      FieldElem { f, n: self.n.clone() }
    } else {
      let mut n = (*self.f.order).clone();
      n -= &self.n;
      FieldElem { f, n }
    }
  }
}

#[derive(Debug, Clone)]
pub struct Field {
  pub order: Rc<BigUint>,
}

impl ToBigUint for BigUint {
  fn to_biguint(&self) -> BigUint {
    self.clone()
  }
}

impl ToBigUint for FieldElem {
  fn to_biguint(&self) -> BigUint {
    self.n.clone()
  }
}

impl Field {
  pub fn new(order: &impl ToBigUint) -> Self {
    Field {
      order: Rc::new(order.to_biguint()),
    }
  } 

  pub fn elem(&self, x: &impl ToBigUint) -> FieldElem {
    FieldElem::new(self, x)
  }

  pub fn repeated_elem(&self, x: &impl ToBigUint, count: usize) -> FieldElems {
    let xs = (0..count).map(|_| FieldElem::new(self, x)).collect::<Vec<FieldElem>>();
    FieldElems(xs)
  }

  pub fn first_n_powers_of_x(&self, x: &impl ToBigUint, n: usize) -> FieldElems {
    let mut vec: Vec<FieldElem> = vec![];
    let mut curr = self.elem(&1u8);
    for _ in 0..n {
      vec.push(curr.clone());
      curr = curr * x;
    }
    FieldElems(vec)
  }

  // returns FieldElem in range [1, field_order-1]
  pub fn rand_elem(&self) -> FieldElem {
    let buf_size = (self.order.bits() as f64 / 8f64).ceil() as usize;
    let mut buf = vec![0u8; buf_size];
    loop {
      let mut rand = RandomNumber::new();
      rand.gen.fill_bytes(&mut buf);
      let x = FieldElem::new(&self, &BigUint::from_bytes_be(&buf));
      if x.n != BigUint::zero() { 
        return x;
      }
    }
  }

  pub fn rand_elems(&self, n: usize) -> FieldElems {
    let xs = (0..n).map(|_| self.rand_elem()).collect::<Vec<FieldElem>>();
    FieldElems(xs)
  }

}

impl PartialEq for Field {
  fn eq(&self, other: &Self) -> bool {
    self.order == other.order
  }
}

impl Eq for Field {}

/////////////////
// FieldElems

pub struct FieldElems(pub Vec<FieldElem>);

impl<'a> Index<usize> for FieldElems {
  type Output = FieldElem;

  fn index(&self, index: usize) -> &Self::Output {
    let x = &self.0[index];
    x
  }
}

impl<'a> FieldElems {
  pub fn new(xs: &'a [FieldElem]) -> Self {
    FieldElems(xs.to_vec())
  }

  pub fn sum(&self) -> FieldElem {
    assert!(self.0.len() > 0);
    let xs = &self.0;
    xs.iter().fold(xs[0].f.elem(&0u8), |acc, x| {
      acc + x
    })
  }

  pub fn from(&self, range: RangeFrom<usize>) -> FieldElems {
    let xs = &self.0[range.start..self.0.len()];
    FieldElems(xs.to_vec())
  }

  pub fn to(&self, range: RangeTo<usize>) -> FieldElems {
    let xs = &self.0[0..range.end];
    FieldElems(xs.to_vec())
  }
}

impl PartialEq for FieldElems {
  fn eq(&self, other: &Self) -> bool {
    if self.len() != other.len() {
      false
    } else {
      self.iter().zip(other.iter()).fold(true, |acc, (l, r)| {
        acc && l.f == r.f && l.n == r.n
      })
    }
  }
}

impl Eq for FieldElems {}

impl Deref for FieldElems {
  type Target = [FieldElem];

  fn deref(&self) -> &Self::Target {
    &self.0[..]
  }
}

impl<'a> ops::Add<&'a[FieldElem]> for &FieldElems {
  type Output = FieldElems;

  fn add(self, rhs: &'a[FieldElem]) -> Self::Output {
    assert!(self.len() > 0 && self.len() == rhs.len());

    let xs: Vec<FieldElem> = self.iter().zip(rhs.iter()).map(|(a, b)| {
      a + b 
    }).collect();
    FieldElems(xs)
  }
}

impl<'a> ops::Sub<&FieldElems> for &FieldElems {
  type Output = FieldElems;

  fn sub(self, rhs: &FieldElems) -> Self::Output {
    assert!(self.len() > 0 && self.len() == rhs.len());

    let xs: Vec<FieldElem> = self.iter().zip(rhs.iter()).map(|(l, r)| {
      l - r 
    }).collect();
    FieldElems(xs)
  }
}

// returns Hadamard product
impl<'a> ops::Mul<&FieldElems> for &FieldElems {
  type Output = FieldElems;

  fn mul(self, rhs: &FieldElems) -> Self::Output {
    assert!(self.len() > 0 && self.len() == rhs.len());

    let xs = self.iter().zip(rhs.iter()).map(|(l, r)| {
      l * r
    }).collect();
    FieldElems(xs)
  }
}

// multiply rhs (scalar) to each element
impl<'a> ops::Mul<&FieldElem> for &FieldElems {
  type Output = FieldElems;

  fn mul(self, rhs: &FieldElem) -> Self::Output {
    assert!(self.len() > 0);

    let xs: Vec<FieldElem> = self.iter().map(|fe| {
      fe * rhs  
    }).collect();

    FieldElems(xs)
  }
}

/////////////
// tests

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn new_below_order() {
    let f = Field::new(&11u32);
    let a = FieldElem::new(&f, &7u32);
    assert_eq!(a.n, BigUint::from(7u32));
  }

  #[test]
  fn new_above_order() {
    let f = Field::new(&11u32);
    let a = FieldElem::new(&f, &13u32);
    assert_eq!(a.n, BigUint::from(2u32));
  }

  #[test]
  fn add_eq_order_result() {
    let f = Field::new(&11u32);
    let a = FieldElem::new(&f, &9u32);
    let b = FieldElem::new(&f, &2u32);
    let c = a + &b;
    assert_eq!(c.n, BigUint::from(0u32));
  }

  #[test]
  fn add_below_order_result() {
    let f = Field::new(&11u32);
    let a = FieldElem::new(&f, &9u32);
    let b = FieldElem::new(&f, &1u32);
    let c = a + &b;
    assert_eq!(c.n, BigUint::from(10u32));
  }

  #[test]
  fn add_above_order_result() {
    let f = Field::new(&11u32);
    let a = FieldElem::new(&f, &9u32);
    let b = FieldElem::new(&f, &3u32);
    let c = a + &b;
    assert_eq!(c.n, BigUint::from(1u32));
  }

  #[test]
  fn sub_smaller_val() {
    let f = Field::new(&11u32);
    let a = FieldElem::new(&f, &9u32);
    let b = FieldElem::new(&f, &2u32);
    let c = a - &b;
    assert_eq!(c.n, BigUint::from(7u32));
  }

  #[test]
  fn sub_eq_val() {
    let f = Field::new(&11u32);
    let a = FieldElem::new(&f, &9u32);
    let b = FieldElem::new(&f, &9u32);
    let c = a - &b;
    assert_eq!(c.n, BigUint::zero());
  }

  #[test]
  fn sub_larger_val() {
    let f = Field::new(&11u32);
    let a = FieldElem::new(&f, &9u32);
    let b = FieldElem::new(&f, &10u32);
    let c = a - &b;
    assert_eq!(c.n, BigUint::from(10u32));
  }

  #[test]
  fn mul_below_order_result() {
    let f = Field::new(&11u32);
    let a = FieldElem::new(&f, &2u32);
    let b = FieldElem::new(&f, &5u32);
    let c = a * &b;
    assert_eq!(c.n, BigUint::from(10u32));
  }

  #[test]
  fn mul_eq_order_result() {
    let f = Field::new(&11u32);
    let a = FieldElem::new(&f, &1u32);
    let b = FieldElem::new(&f, &11u32);
    let c = a * &b;
    assert_eq!(c.n, BigUint::from(0u32));
  }

  #[test]
  fn mul_above_order_result() {
    let f = Field::new(&11u32);
    let a = FieldElem::new(&f, &3u32);
    let b = FieldElem::new(&f, &9u32);
    let c = a * &b;
    assert_eq!(c.n, BigUint::from(5u32));
  }

  #[test]
  fn mul_u32_below_order_result() {
    let f = Field::new(&11u8);
    let a = FieldElem::new(&f, &2u8);
    let b = a * &5u8;
    assert_eq!(b.n, BigUint::from(10u8));
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
      let f = Field::new(&x.order);
      let a = FieldElem::new(&f, &x.n);
      let inv = a.safe_inv()?;
      assert_eq!(inv.n, BigUint::from(x.exp));
    }
    Ok(())
  }

  #[test]
  fn div() {
    let f = Field::new(&11u32);
    let a = FieldElem::new(&f, &4u32);
    let b = FieldElem::new(&f, &2u32);
    let c = a.safe_div(&b).unwrap();
    assert_eq!(c.n, BigUint::from(2u32));
  }

  #[test]
  fn inv_secp256k1() -> Result<(), String> {
    let p = BigUint::parse_bytes(b"FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F", 16).unwrap();
    let f = Field::new(&p);
    let a = FieldElem::new(&f, &1112121212121u64);

    let exp = BigUint::parse_bytes(b"52624297956533532283067125375510330718705195823487497799082320305224600546911", 10).unwrap();
    let inv = a.safe_inv()?;
    assert_eq!(exp, inv.n);
    Ok(())
  }

  #[test]
  fn neg() {
    let f = Field::new(&11u32);
    let a = FieldElem::new(&f, &5u32);
    assert_eq!(a.negate().n, BigUint::from(6u32));

    let neg_a = a.clone() + &a.negate();
    assert_eq!(neg_a.n, BigUint::from(0u32));
  }

  #[test]
  fn pow_various_combinations() {
    let f = Field::new(&100000000u128);
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
      let base = FieldElem::new(&f, &t.0);
      let exponent = BigUint::from(t.1);
      let expected = BigUint::from(t.2);
      assert_eq!(base.pow(&exponent).n, BigUint::from(expected));
    }
  }

  #[test]
  fn pow_below_order() {
    let f = Field::new(&11u32);
    let a = FieldElem::new(&f, &2u32);
    assert_eq!(a.pow(&3u32).n, BigUint::from(8u32));
  }

  #[test]
  fn pow_above_order() {
    let f = Field::new(&11u32);
    let a = FieldElem::new(&f, &2u32);
    assert_eq!(a.pow(&4u32).n, BigUint::from(5u32));
  }

  #[test]
  fn sq_below_order() {
    let f = Field::new(&11u32);
    let a = FieldElem::new(&f, &2u32);
    assert_eq!(a.sq().n, BigUint::from(4u32));
  }

  #[test]
  fn sq_above_order() {
    let f = Field::new(&11u32);
    let a = FieldElem::new(&f, &4u32);
    assert_eq!(a.sq().n, BigUint::from(5u32));
  }
  #[test]
  fn new_elem_from_biguint() {
    let f = Field::new(&11u32);
    let a = f.elem(&7u8);
    assert_eq!(a.n, BigUint::from(7u32));
  }
  
  #[test]
  fn new_elem_from_u8() {
    let f = Field::new(&11u32);
    let a = f.elem(&7u8);
    assert_eq!(a.n, BigUint::from(7u32));
  }
}