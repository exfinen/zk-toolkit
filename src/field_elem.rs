use crate::field::Field;
use num_bigint::{BigUint, ToBigInt};

#[derive(Clone, Debug)]
pub struct FieldElem<'a> {
  pub f: &'a Field,
  pub v: BigUint,
}

impl <'a> PartialEq for FieldElem<'a> {
  fn eq(&self, other: &Self) -> bool {
    self.f == other.f && self.v == other.v
  }
}

impl <'a> Eq for FieldElem<'a> {}

impl <'a> FieldElem<'a> {
  pub fn new(f: &'a Field, v: BigUint) -> Self {
    if v >= f.order {
      let v = v % &f.order;
      FieldElem { f, v }
    } else {
      FieldElem { f, v }
    }
  }

  pub fn add(&self, other: &FieldElem<'a>) -> FieldElem<'a> {
    let mut v = self.v.clone();
    v += &other.v;
    if v >= self.f.order {
      v -= &self.f.order;
    }
    FieldElem { f: self.f, v }
  }

  pub fn sub(&self, other: &FieldElem<'a>) -> FieldElem<'a> {
    if self.v < other.v {
      let diff = other.v.clone() - &self.v;
      let v = self.f.order.clone() - diff;
      FieldElem { f: self.f, v }
    } else {
      let mut v = self.v.clone();
      v -= &other.v;
      FieldElem { f: self.f, v }
    }
  }

  pub fn mul(&self, other: &FieldElem<'a>) -> FieldElem<'a> {
    let mut v = self.v.clone();
    v *= &other.v;
    v %= &self.f.order;
    if v < self.f.zero {
      v += &self.f.order;
    }
    FieldElem { f: self.f, v }
  }

  pub fn mul_u32(&self, other_u32: u32) -> FieldElem<'a> {
    let other_fe = self.f.element(BigUint::from(other_u32));
    self.mul(&other_fe)
  }

  pub fn pow_u32(&self, other_u32: u32) -> FieldElem<'a> {
    let mut v = self.v.clone();
    let num_multiply = other_u32 - 1;
    for _ in 0..num_multiply {
      v *= &self.v;
      v %= &self.f.order;
    }
    FieldElem { f: self.f, v }
  }

  // based on extended Euclidean algorithm
  pub fn inv(&self) -> Result<FieldElem<'a>, String> {
    if self.v == self.f.zero {
      return Err("Cannot find inverse of zero".to_string());
    }
    let order = self.f.order.to_bigint().unwrap();
    let v = self.v.to_bigint().unwrap();
    let zero = self.f.zero.to_bigint().unwrap();
    let one = self.f.one.to_bigint().unwrap();

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
      let q = r0.clone() / r1.clone();
      let r2 = r0.clone() % r1.clone();
      // this produces the same result as above r2 using mod
      //let r2 = x2.clone() * order.clone() + y2.clone() * v.clone();
      let x2 = x0 - x1.clone() * q.clone();
      let y2 = y0 - y1.clone() * q.clone();

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
    Ok(FieldElem { f: self.f, v: new_v.to_biguint().unwrap() })
  }

  pub fn div(&self, other: &FieldElem<'a>) -> Result<FieldElem<'a>, String> {
    let inv = other.inv()?;
    Ok(self.mul(&inv))
  }

  pub fn neg(&self) -> FieldElem<'a> {
    if self.v == self.f.zero {
      FieldElem { f: self.f, v: self.v.clone() }
    } else {
      let mut v = self.f.order.clone();
      v -= &self.v;
      FieldElem { f: self.f, v }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_add_eq_order_result() {
    let f = Field::new(BigUint::from(11u32));
    let a = f.element(BigUint::from(9u32));
    let b = f.element(BigUint::from(2u32));
    let c = a.add(&b);
    assert_eq!(c.v, BigUint::from(0u32));
  }

  #[test]
  fn test_add_below_order_result() {
    let f = Field::new(BigUint::from(11u32));
    let a = f.element(BigUint::from(9u32));
    let b = f.element(BigUint::from(1u32));
    let c = a.add(&b);
    assert_eq!(c.v, BigUint::from(10u32));
  }

  #[test]
  fn test_add_above_order_result() {
    let f = Field::new(BigUint::from(11u32));
    let a = f.element(BigUint::from(9u32));
    let b = f.element(BigUint::from(3u32));
    let c = a.add(&b);
    assert_eq!(c.v, BigUint::from(1u32));
  }

  #[test]
  fn test_sub_smaller_val() {
    let f = Field::new(BigUint::from(11u32));
    let a = f.element(BigUint::from(9u32));
    let b = f.element(BigUint::from(2u32));
    let c = a.sub(&b);
    assert_eq!(c.v, BigUint::from(7u32));
  }

  #[test]
  fn test_sub_eq_val() {
    let f = Field::new(BigUint::from(11u32));
    let a = f.element(BigUint::from(9u32));
    let b = f.element(BigUint::from(9u32));
    let c = a.sub(&b);
    assert_eq!(c.v, f.zero);
  }

  #[test]
  fn test_sub_larger_val() {
    let f = Field::new(BigUint::from(11u32));
    let a = f.element(BigUint::from(9u32));
    let b = f.element(BigUint::from(10u32));
    let c = a.sub(&b);
    assert_eq!(c.v, BigUint::from(10u32));
  }

  #[test]
  fn test_mul_below_order_result() {
    let f = Field::new(BigUint::from(11u32));
    let a = f.element(BigUint::from(2u32));
    let b = f.element(BigUint::from(5u32));
    let c = a.mul(&b);
    assert_eq!(c.v, BigUint::from(10u32));
  }

  #[test]
  fn test_mul_eq_order_result() {
    let f = Field::new(BigUint::from(11u32));
    let a = f.element(BigUint::from(1u32));
    let b = f.element(BigUint::from(11u32));
    let c = a.mul(&b);
    assert_eq!(c.v, BigUint::from(0u32));
  }

  #[test]
  fn test_mul_above_order_result() {
    let f = Field::new(BigUint::from(11u32));
    let a = f.element(BigUint::from(3u32));
    let b = f.element(BigUint::from(9u32));
    let c = a.mul(&b);
    assert_eq!(c.v, BigUint::from(5u32));
  }

  #[test]
  fn test_mul_u32_below_order_result() {
    let f = Field::new(BigUint::from(11u32));
    let a = f.element(BigUint::from(2u32));
    let b = a.mul_u32(5);
    assert_eq!(b.v, BigUint::from(10u32));
  }

  struct InvTestCase {
    order: u32,
    v: u32,
    exp: u32,
  }

  #[test]
  fn test_inv_small_primes() -> Result<(), String> {
    let test_cases = [
      // order 97
      InvTestCase { order: 97u32, v: 1u32, exp: 1u32 },
      InvTestCase { order: 97u32, v: 2u32, exp: 49u32 },
      InvTestCase { order: 97u32, v: 3u32, exp: 65u32 },
      InvTestCase { order: 97u32, v: 4u32, exp: 73u32 },
      InvTestCase { order: 97u32, v: 5u32, exp: 39u32 },
      InvTestCase { order: 97u32, v: 6u32, exp: 81u32 },
      InvTestCase { order: 97u32, v: 7u32, exp: 14u32 },
      InvTestCase { order: 97u32, v: 8u32, exp: 85u32 },
      InvTestCase { order: 97u32, v: 9u32, exp: 54u32 },
      InvTestCase { order: 97u32, v: 10u32, exp: 68u32 },
      InvTestCase { order: 97u32, v: 11u32, exp: 53u32 },
      InvTestCase { order: 97u32, v: 12u32, exp: 89u32 },
      InvTestCase { order: 97u32, v: 13u32, exp: 15u32 },
      InvTestCase { order: 97u32, v: 14u32, exp: 7u32 },
      InvTestCase { order: 97u32, v: 15u32, exp: 13u32 },
      InvTestCase { order: 97u32, v: 16u32, exp: 91u32 },
      InvTestCase { order: 97u32, v: 17u32, exp: 40u32 },
      InvTestCase { order: 97u32, v: 18u32, exp: 27u32 },
      InvTestCase { order: 97u32, v: 19u32, exp: 46u32 },
      InvTestCase { order: 97u32, v: 20u32, exp: 34u32 },
      InvTestCase { order: 97u32, v: 21u32, exp: 37u32 },
      InvTestCase { order: 97u32, v: 22u32, exp: 75u32 },
      InvTestCase { order: 97u32, v: 23u32, exp: 38u32 },
      InvTestCase { order: 97u32, v: 24u32, exp: 93u32 },
      InvTestCase { order: 97u32, v: 25u32, exp: 66u32 },
      InvTestCase { order: 97u32, v: 26u32, exp: 56u32 },
      InvTestCase { order: 97u32, v: 27u32, exp: 18u32 },
      InvTestCase { order: 97u32, v: 28u32, exp: 52u32 },
      InvTestCase { order: 97u32, v: 29u32, exp: 87u32 },
      InvTestCase { order: 97u32, v: 30u32, exp: 55u32 },
      InvTestCase { order: 97u32, v: 31u32, exp: 72u32 },
      InvTestCase { order: 97u32, v: 32u32, exp: 94u32 },
      InvTestCase { order: 97u32, v: 33u32, exp: 50u32 },
      InvTestCase { order: 97u32, v: 34u32, exp: 20u32 },
      InvTestCase { order: 97u32, v: 35u32, exp: 61u32 },
      InvTestCase { order: 97u32, v: 36u32, exp: 62u32 },
      InvTestCase { order: 97u32, v: 37u32, exp: 21u32 },
      InvTestCase { order: 97u32, v: 38u32, exp: 23u32 },
      InvTestCase { order: 97u32, v: 39u32, exp: 5u32 },
      InvTestCase { order: 97u32, v: 40u32, exp: 17u32 },
      InvTestCase { order: 97u32, v: 41u32, exp: 71u32 },
      InvTestCase { order: 97u32, v: 42u32, exp: 67u32 },
      InvTestCase { order: 97u32, v: 43u32, exp: 88u32 },
      InvTestCase { order: 97u32, v: 44u32, exp: 86u32 },
      InvTestCase { order: 97u32, v: 45u32, exp: 69u32 },
      InvTestCase { order: 97u32, v: 46u32, exp: 19u32 },
      InvTestCase { order: 97u32, v: 47u32, exp: 64u32 },
      InvTestCase { order: 97u32, v: 48u32, exp: 95u32 },
      InvTestCase { order: 97u32, v: 49u32, exp: 2u32 },
      InvTestCase { order: 97u32, v: 50u32, exp: 33u32 },
      InvTestCase { order: 97u32, v: 51u32, exp: 78u32 },
      InvTestCase { order: 97u32, v: 52u32, exp: 28u32 },
      InvTestCase { order: 97u32, v: 53u32, exp: 11u32 },
      InvTestCase { order: 97u32, v: 54u32, exp: 9u32 },
      InvTestCase { order: 97u32, v: 55u32, exp: 30u32 },
      InvTestCase { order: 97u32, v: 56u32, exp: 26u32 },
      InvTestCase { order: 97u32, v: 57u32, exp: 80u32 },
      InvTestCase { order: 97u32, v: 58u32, exp: 92u32 },
      InvTestCase { order: 97u32, v: 59u32, exp: 74u32 },
      InvTestCase { order: 97u32, v: 60u32, exp: 76u32 },
      InvTestCase { order: 97u32, v: 61u32, exp: 35u32 },
      InvTestCase { order: 97u32, v: 62u32, exp: 36u32 },
      InvTestCase { order: 97u32, v: 63u32, exp: 77u32 },
      InvTestCase { order: 97u32, v: 64u32, exp: 47u32 },
      InvTestCase { order: 97u32, v: 65u32, exp: 3u32 },
      InvTestCase { order: 97u32, v: 66u32, exp: 25u32 },
      InvTestCase { order: 97u32, v: 67u32, exp: 42u32 },
      InvTestCase { order: 97u32, v: 68u32, exp: 10u32 },
      InvTestCase { order: 97u32, v: 69u32, exp: 45u32 },
      InvTestCase { order: 97u32, v: 70u32, exp: 79u32 },
      InvTestCase { order: 97u32, v: 71u32, exp: 41u32 },
      InvTestCase { order: 97u32, v: 72u32, exp: 31u32 },
      InvTestCase { order: 97u32, v: 73u32, exp: 4u32 },
      InvTestCase { order: 97u32, v: 74u32, exp: 59u32 },
      InvTestCase { order: 97u32, v: 75u32, exp: 22u32 },
      InvTestCase { order: 97u32, v: 76u32, exp: 60u32 },
      InvTestCase { order: 97u32, v: 77u32, exp: 63u32 },
      InvTestCase { order: 97u32, v: 78u32, exp: 51u32 },
      InvTestCase { order: 97u32, v: 79u32, exp: 70u32 },
      InvTestCase { order: 97u32, v: 80u32, exp: 57u32 },
      InvTestCase { order: 97u32, v: 81u32, exp: 6u32 },
      InvTestCase { order: 97u32, v: 82u32, exp: 84u32 },
      InvTestCase { order: 97u32, v: 83u32, exp: 90u32 },
      InvTestCase { order: 97u32, v: 84u32, exp: 82u32 },
      InvTestCase { order: 97u32, v: 85u32, exp: 8u32 },
      InvTestCase { order: 97u32, v: 86u32, exp: 44u32 },
      InvTestCase { order: 97u32, v: 87u32, exp: 29u32 },
      InvTestCase { order: 97u32, v: 88u32, exp: 43u32 },
      InvTestCase { order: 97u32, v: 89u32, exp: 12u32 },
      InvTestCase { order: 97u32, v: 90u32, exp: 83u32 },
      InvTestCase { order: 97u32, v: 91u32, exp: 16u32 },
      InvTestCase { order: 97u32, v: 92u32, exp: 58u32 },
      InvTestCase { order: 97u32, v: 93u32, exp: 24u32 },
      InvTestCase { order: 97u32, v: 94u32, exp: 32u32 },
      InvTestCase { order: 97u32, v: 95u32, exp: 48u32 },
      InvTestCase { order: 97u32, v: 96u32, exp: 96u32 },

      // order 53
      InvTestCase { order: 53u32, v: 1u32, exp: 1u32 },
      InvTestCase { order: 53u32, v: 2u32, exp: 27u32 },
      InvTestCase { order: 53u32, v: 3u32, exp: 18u32 },
      InvTestCase { order: 53u32, v: 4u32, exp: 40u32 },
      InvTestCase { order: 53u32, v: 5u32, exp: 32u32 },
      InvTestCase { order: 53u32, v: 6u32, exp: 9u32 },
      InvTestCase { order: 53u32, v: 7u32, exp: 38u32 },
      InvTestCase { order: 53u32, v: 8u32, exp: 20u32 },
      InvTestCase { order: 53u32, v: 9u32, exp: 6u32 },
      InvTestCase { order: 53u32, v: 10u32, exp: 16u32 },
      InvTestCase { order: 53u32, v: 11u32, exp: 29u32 },
      InvTestCase { order: 53u32, v: 12u32, exp: 31u32 },
      InvTestCase { order: 53u32, v: 13u32, exp: 49u32 },
      InvTestCase { order: 53u32, v: 14u32, exp: 19u32 },
      InvTestCase { order: 53u32, v: 15u32, exp: 46u32 },
      InvTestCase { order: 53u32, v: 16u32, exp: 10u32 },
      InvTestCase { order: 53u32, v: 17u32, exp: 25u32 },
      InvTestCase { order: 53u32, v: 18u32, exp: 3u32 },
      InvTestCase { order: 53u32, v: 19u32, exp: 14u32 },
      InvTestCase { order: 53u32, v: 20u32, exp: 8u32 },
      InvTestCase { order: 53u32, v: 21u32, exp: 48u32 },
      InvTestCase { order: 53u32, v: 22u32, exp: 41u32 },
      InvTestCase { order: 53u32, v: 23u32, exp: 30u32 },
      InvTestCase { order: 53u32, v: 24u32, exp: 42u32 },
      InvTestCase { order: 53u32, v: 25u32, exp: 17u32 },
      InvTestCase { order: 53u32, v: 26u32, exp: 51u32 },
      InvTestCase { order: 53u32, v: 27u32, exp: 2u32 },
      InvTestCase { order: 53u32, v: 28u32, exp: 36u32 },
      InvTestCase { order: 53u32, v: 29u32, exp: 11u32 },
      InvTestCase { order: 53u32, v: 30u32, exp: 23u32 },
      InvTestCase { order: 53u32, v: 31u32, exp: 12u32 },
      InvTestCase { order: 53u32, v: 32u32, exp: 5u32 },
      InvTestCase { order: 53u32, v: 33u32, exp: 45u32 },
      InvTestCase { order: 53u32, v: 34u32, exp: 39u32 },
      InvTestCase { order: 53u32, v: 35u32, exp: 50u32 },
      InvTestCase { order: 53u32, v: 36u32, exp: 28u32 },
      InvTestCase { order: 53u32, v: 37u32, exp: 43u32 },
      InvTestCase { order: 53u32, v: 38u32, exp: 7u32 },
      InvTestCase { order: 53u32, v: 39u32, exp: 34u32 },
      InvTestCase { order: 53u32, v: 40u32, exp: 4u32 },
      InvTestCase { order: 53u32, v: 41u32, exp: 22u32 },
      InvTestCase { order: 53u32, v: 42u32, exp: 24u32 },
      InvTestCase { order: 53u32, v: 43u32, exp: 37u32 },
      InvTestCase { order: 53u32, v: 44u32, exp: 47u32 },
      InvTestCase { order: 53u32, v: 45u32, exp: 33u32 },
      InvTestCase { order: 53u32, v: 46u32, exp: 15u32 },
      InvTestCase { order: 53u32, v: 47u32, exp: 44u32 },
      InvTestCase { order: 53u32, v: 48u32, exp: 21u32 },
      InvTestCase { order: 53u32, v: 49u32, exp: 13u32 },
      InvTestCase { order: 53u32, v: 50u32, exp: 35u32 },
      InvTestCase { order: 53u32, v: 51u32, exp: 26u32 },
      InvTestCase { order: 53u32, v: 52u32, exp: 52u32 },

      // order 11
      InvTestCase { order: 11u32, v: 1u32, exp: 1u32 },
      InvTestCase { order: 11u32, v: 2u32, exp: 6u32 },
      InvTestCase { order: 11u32, v: 3u32, exp: 4u32 },
      InvTestCase { order: 11u32, v: 4u32, exp: 3u32 },
      InvTestCase { order: 11u32, v: 5u32, exp: 9u32 },
      InvTestCase { order: 11u32, v: 6u32, exp: 2u32 },
      InvTestCase { order: 11u32, v: 7u32, exp: 8u32 },
      InvTestCase { order: 11u32, v: 8u32, exp: 7u32 },
      InvTestCase { order: 11u32, v: 9u32, exp: 5u32 },
      InvTestCase { order: 11u32, v: 10u32, exp: 10u32 },
    ];

    for x in test_cases {
      let f = Field::new(BigUint::from(x.order));
      let a = f.element(BigUint::from(x.v));
      let inv = a.inv()?;
      assert_eq!(inv.v, BigUint::from(x.exp));
    }
    Ok(())
  }

  #[test]
  fn test_div() {
    let f = Field::new(BigUint::from(11u32));
    let a = f.element(BigUint::from(4u32));
    let b = f.element(BigUint::from(2u32));
    let c = a.div(&b).unwrap();
    assert_eq!(c.v, BigUint::from(2u32));
  }

  #[test]
  fn test_inv_secp256k1() -> Result<(), String> {
    let p = BigUint::parse_bytes(b"FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F", 16).unwrap();
    let f = Field::new(p);
    let a = f.element(BigUint::from(1112121212121u64));

    let exp = BigUint::parse_bytes(b"52624297956533532283067125375510330718705195823487497799082320305224600546911", 10).unwrap();
    let inv = a.inv()?;
    assert_eq!(exp, inv.v);
    Ok(())
  }

  #[test]
  fn test_neg() {
    let f = Field::new(BigUint::from(11u32));
    let a = f.element(BigUint::from(5u32));
    assert_eq!(a.neg().v, BigUint::from(6u32));

    let neg_a = a.add(&a.neg());
    assert_eq!(neg_a.v, BigUint::from(0u32));
  }

  #[test]
  fn test_pow_u32_below_order() {
    let f = Field::new(BigUint::from(11u32));
    let a = f.element(BigUint::from(2u32));
    assert_eq!(a.pow_u32(3u32).v, BigUint::from(8u32));
  }

  #[test]
  fn test_pow_u32_above_order() {
    let f = Field::new(BigUint::from(11u32));
    let a = f.element(BigUint::from(2u32));
    assert_eq!(a.pow_u32(4u32).v, BigUint::from(5u32));
  }
}