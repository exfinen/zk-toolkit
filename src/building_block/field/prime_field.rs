use crate::building_block::{
  additive_identity::AdditiveIdentity,
  field::{
    prime_field_elem::PrimeFieldElem,
    prime_field_elems::PrimeFieldElems,
  },
  random_number::RandomNumber,
  to_bigint::ToBigInt as ToBigIntType,
  to_biguint::ToBigUint,
};
use num_bigint::{BigUint, BigInt, Sign};
use num_traits::Zero;
use rand::RngCore;

#[derive(Debug, Clone, Hash)]
pub struct PrimeField {
  pub order: BigUint,
}

impl AdditiveIdentity for PrimeField {
  fn get_additive_identity(&self) -> Self {
    PrimeFieldElem {
      f: self.clone(),
      e: BigUint::from(0u8),
    }
  }
}

impl PrimeField {
  pub fn new(order: &impl ToBigUint) -> Self {
    PrimeField {
      order: order.to_biguint(),
    }
  }

  pub fn elem(&self, x: &impl ToBigUint) -> PrimeFieldElem {
    PrimeFieldElem::new(self, x)
  }

  pub fn elem_from_signed(&self, x: &impl ToBigIntType) -> PrimeFieldElem {
    let n = x.to_bigint();
    if n.sign() == Sign::Minus {
      let order = &BigInt::from_biguint(Sign::Plus, self.order.clone());
      let mut n = -n;
      n = n % order;
      n = order - n;
      let n = n.to_biguint().unwrap();
      PrimeFieldElem::new(self, &n)
    } else {
      let n = BigInt::from(n).to_biguint().unwrap();
      PrimeFieldElem::new(self, &n)
    }
  }

  pub fn repeated_elem(&self, x: &impl ToBigUint, count: usize) -> PrimeFieldElems {
    let xs = (0..count).map(|_| PrimeFieldElem::new(self, x)).collect::<Vec<PrimeFieldElem>>();
    PrimeFieldElems(xs)
  }

  pub fn first_n_powers_of_x(&self, x: &impl ToBigUint, n: usize) -> PrimeFieldElems {
    let mut vec: Vec<PrimeFieldElem> = vec![];
    let mut curr = self.elem(&1u8);
    for _ in 0..n {
      vec.push(curr.clone());
      curr = curr * x.to_biguint();
    }
    PrimeFieldElems(vec)
  }

  // returns FieldElem in range [1, field_order-1]
  pub fn rand_elem(&self, exclude_zero: bool) -> PrimeFieldElem {
    let buf_size = (self.order.bits() as f64 / 8f64).ceil() as usize;
    let mut buf = vec![0u8; buf_size];
    loop {
      let mut rand = RandomNumber::new();
      rand.gen.fill_bytes(&mut buf);
      let x = PrimeFieldElem::new(&self, &BigUint::from_bytes_be(&buf));
      if !exclude_zero || x.e != BigUint::zero() {
        return x;
      }
    }
  }

  pub fn rand_elems(&self, n: usize, exclude_zero: bool) -> PrimeFieldElems {
    let xs = (0..n).map(|_| self.rand_elem(exclude_zero)).collect::<Vec<PrimeFieldElem>>();
    PrimeFieldElems(xs)
  }

}

impl PartialEq for PrimeField {
  fn eq(&self, other: &Self) -> bool {
    self.order == other.order
  }
}

impl Eq for PrimeField {}

/////////////
// tests

#[cfg(test)]
mod tests {
  use super::*;
  use num_traits::ToPrimitive;

  #[test]
  fn new_below_order() {
    let f = PrimeField::new(&11u8);
    let a = PrimeFieldElem::new(&f, &7u8);
    assert_eq!(a.e, BigUint::from(7u8));
  }

  #[test]
  fn new_above_order() {
    let f = PrimeField::new(&11u8);
    let a = PrimeFieldElem::new(&f, &13u8);
    assert_eq!(a.e, BigUint::from(2u32));
  }

  #[test]
  fn inc_below_order() {
    let f = PrimeField::new(&11u8);
    let mut a = PrimeFieldElem::new(&f, &1u8);
    a.inc();
    assert_eq!(a, f.elem(&2u8));
  }

  #[test]
  fn inc_above_order() {
    let f = PrimeField::new(&11u8);
    let mut a = PrimeFieldElem::new(&f, &10u8);
    a.inc();
    assert_eq!(a, f.elem(&0u8));
  }

  #[test]
  fn add_eq_order_result() {
    let f = PrimeField::new(&11u8);
    let a = PrimeFieldElem::new(&f, &9u8);
    let b = PrimeFieldElem::new(&f, &2u8);
    let c = a + &b;
    assert_eq!(c.e, BigUint::from(0u8));
  }

  #[test]
  fn add_below_order_result() {
    let f = PrimeField::new(&11u8);
    let a = PrimeFieldElem::new(&f, &9u8);
    let b = PrimeFieldElem::new(&f, &1u8);
    let c = a + &b;
    assert_eq!(c.e, BigUint::from(10u8));
  }

  #[test]
  fn add_above_order_result() {
    let f = PrimeField::new(&11u8);
    let a = PrimeFieldElem::new(&f, &9u8);
    let b = PrimeFieldElem::new(&f, &3u8);
    let c = a + b;
    assert_eq!(c.e, BigUint::from(1u8));
  }

  #[test]
  fn plus_above_order_times_2_result() {
    let f = PrimeField::new(&11u8);
    let a = PrimeFieldElem::new(&f, &3u8);
    let c = a.plus(&24u8);
    assert_eq!(c.e, BigUint::from(5u8));
  }

  #[test]
  fn sub_smaller_val() {
    let f = PrimeField::new(&11u8);
    let a = PrimeFieldElem::new(&f, &9u8);
    let b = PrimeFieldElem::new(&f, &2u8);
    let c = a - b;
    assert_eq!(c.e, BigUint::from(7u8));
  }

  #[test]
  fn sub_eq_val() {
    let f = PrimeField::new(&11u8);
    let a = PrimeFieldElem::new(&f, &9u8);
    let b = PrimeFieldElem::new(&f, &9u8);
    let c = a - b;
    assert_eq!(c.e, BigUint::zero());
  }

  #[test]
  fn sub_larger_val() {
    let f = PrimeField::new(&11u8);
    let a = PrimeFieldElem::new(&f, &9u8);
    let b = PrimeFieldElem::new(&f, &10u8);
    let c = a - b;
    assert_eq!(c.e, BigUint::from(10u8));
  }

  #[test]
  fn minus_order_times_2() {
    let f = PrimeField::new(&11u8);
    let a = PrimeFieldElem::new(&f, &3u8);
    let c = a.minus(&24u8);
    assert_eq!(c.e, BigUint::from(1u8));
  }

  #[test]
  fn mul_below_order_result() {
    let f = PrimeField::new(&11u8);
    let a = PrimeFieldElem::new(&f, &2u8);
    let b = PrimeFieldElem::new(&f, &5u8);
    let c = a * b;
    assert_eq!(c.e, BigUint::from(10u8));
  }

  #[test]
  fn mul_eq_order_result() {
    let f = PrimeField::new(&11u8);
    let a = PrimeFieldElem::new(&f, &1u8);
    let b = PrimeFieldElem::new(&f, &11u8);
    let c = a * b;
    assert_eq!(c.e, BigUint::from(0u32));
  }

  #[test]
  fn mul_above_order_result() {
    let f = PrimeField::new(&11u8);
    let a = PrimeFieldElem::new(&f, &3u8);
    let b = PrimeFieldElem::new(&f, &9u8);
    let c = a * b;
    assert_eq!(c.e, BigUint::from(5u8));
  }

  #[test]
  fn mul_u32_below_order_result() {
    let f = PrimeField::new(&11u8);
    let a = PrimeFieldElem::new(&f, &2u8);
    let b = a * 5u8;
    assert_eq!(b.e, BigUint::from(10u8));
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
      let f = PrimeField::new(&x.order);
      let a = PrimeFieldElem::new(&f, &x.n);
      let inv = a.safe_inv()?;
      assert_eq!(inv.e, BigUint::from(x.exp));
    }
    Ok(())
  }

  #[test]
  fn div() {
    let f = PrimeField::new(&11u8);
    let a = PrimeFieldElem::new(&f, &4u8);
    let b = PrimeFieldElem::new(&f, &2u8);
    let c = a.safe_div(&b).unwrap();
    assert_eq!(c.e, BigUint::from(2u32));
  }

  #[test]
  fn inv_secp256k1() -> Result<(), String> {
    let p = BigUint::parse_bytes(b"FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F", 16).unwrap();
    let f = PrimeField::new(&p);
    let a = PrimeFieldElem::new(&f, &1112121212121u64);

    let exp = BigUint::parse_bytes(b"52624297956533532283067125375510330718705195823487497799082320305224600546911", 10).unwrap();
    let inv = a.safe_inv()?;
    assert_eq!(exp, inv.e);
    Ok(())
  }

  #[test]
  fn neg() {
    let f = PrimeField::new(&11u8);
    let a = PrimeFieldElem::new(&f, &5u8);
    assert_eq!(a.negate().e, BigUint::from(6u8));

    let neg_a = a.clone() + &a.negate();
    assert_eq!(neg_a.e, BigUint::from(0u32));
  }

  #[test]
  fn cube() {
    {
      // when value is less than field order
      let f = PrimeField::new(&100u8);
      let a = PrimeFieldElem::new(&f, &3u8);
      let b = a.cube();
      assert_eq!(b.e.to_u8().unwrap(), 27);
    }
    {
      // when value is larger than field order
      let f = PrimeField::new(&11u8);
      let a = PrimeFieldElem::new(&f, &3u8);
      let b = a.cube();
      assert_eq!(b.e.to_u8().unwrap(), 5);
    }
  }

  #[test]
  fn pow_seq() {
    {
      // when value is less than field order
      let f = PrimeField::new(&100u8);
      let a = PrimeFieldElem::new(&f, &3u8);
      let xs = a.pow_seq(4);
      assert_eq!(xs.len(), 4);
      assert_eq!(xs[0].e.to_u8().unwrap(), 1);
      assert_eq!(xs[1].e.to_u8().unwrap(), 3);
      assert_eq!(xs[2].e.to_u8().unwrap(), 9);
      assert_eq!(xs[3].e.to_u8().unwrap(), 27);
    }
    {
      // when value is larger than field order
      let f = PrimeField::new(&11u8);
      let a = PrimeFieldElem::new(&f, &3u8);
      let xs = a.pow_seq(4);
      assert_eq!(xs.len(), 4);
      assert_eq!(xs[0].e.to_u8().unwrap(), 1);
      assert_eq!(xs[1].e.to_u8().unwrap(), 3);
      assert_eq!(xs[2].e.to_u8().unwrap(), 9);
      assert_eq!(xs[3].e.to_u8().unwrap(), 5);
    }
  }

  #[test]
  fn repeat() {
    let f = PrimeField::new(&11u8);
    let a = PrimeFieldElem::new(&f, &5u8);
    let xs = a.repeat(3);
    assert_eq!(xs.len(), 3);
    assert_eq!(xs[0].e.to_u8().unwrap(), 5);
    assert_eq!(xs[1].e.to_u8().unwrap(), 5);
    assert_eq!(xs[2].e.to_u8().unwrap(), 5);
  }

  #[test]
  fn pow_various_combinations() {
    let f = PrimeField::new(&100000000u128);
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
    let f = PrimeField::new(&11u8);
    let a = PrimeFieldElem::new(&f, &2u8);
    assert_eq!(a.pow(&3u8).e, BigUint::from(8u8));
  }

  #[test]
  fn pow_above_order() {
    let f = PrimeField::new(&11u8);
    let a = PrimeFieldElem::new(&f, &2u8);
    assert_eq!(a.pow(&4u8).e, BigUint::from(5u8));
  }

  #[test]
  fn sq_below_order() {
    let f = PrimeField::new(&11u8);
    let a = PrimeFieldElem::new(&f, &2u8);
    assert_eq!(a.sq().e, BigUint::from(4u8));
  }

  #[test]
  fn sq_above_order() {
    let f = PrimeField::new(&11u8);
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
