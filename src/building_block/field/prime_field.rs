use crate::building_block::{
  field::{
    prime_field_elem::PrimeFieldElem,
    prime_field_elems::PrimeFieldElems,
  },
  random_number::RandomNumber,
  to_bigint::ToBigInt as ToBigIntType,
  to_biguint::ToBigUint,
};
use num_bigint::{BigUint, BigInt, Sign};
use num_traits::Zero as NumTraitsZero;
use rand::RngCore;
use std::sync::Arc;

#[derive(Debug, Clone, Hash)]
pub struct PrimeField {
  order: BigUint,
}

impl PrimeField {
  pub fn new(order: &impl ToBigUint) -> Self {
    PrimeField {
      order: order.to_biguint(),
    }
  }

  pub fn order(&self) -> BigUint {
    self.order.clone()
  }

  pub fn order_ref(&self) -> &BigUint {
    &self.order
  }

  pub fn elem(&self, x: &impl ToBigUint) -> PrimeFieldElem {
    let f = Arc::new(self.clone());
    PrimeFieldElem::new(&f, x)
  }

  pub fn elem_from_signed(&self, x: &impl ToBigIntType) -> PrimeFieldElem {
    let f = Arc::new(self.clone());
    let n = x.to_bigint();
    if n.sign() == Sign::Minus {
      let order = &BigInt::from_biguint(Sign::Plus, self.order.clone());
      let mut n = -n;
      n = n % order;
      n = order - n;
      let n = n.to_biguint().unwrap();
      PrimeFieldElem::new(&f, &n)
    } else {
      let n = BigInt::from(n).to_biguint().unwrap();
      PrimeFieldElem::new(&f, &n)
    }
  }

  pub fn repeated_elem(&self, x: &impl ToBigUint, count: usize) -> PrimeFieldElems {
    let f = Arc::new(self.clone());
    let xs = (0..count).map(|_| PrimeFieldElem::new(&f, x)).collect::<Vec<PrimeFieldElem>>();
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
    let f = Arc::new(self.clone());
    loop {
      let mut rand = RandomNumber::new();
      rand.gen.fill_bytes(&mut buf);
      let x = PrimeFieldElem::new(&f, &BigUint::from_bytes_be(&buf));
      if !exclude_zero || x.e != BigUint::zero() {
        return x;
      }
    }
  }

  pub fn rand_elems(&self, n: &usize, exclude_zero: bool) -> PrimeFieldElems {
    let xs = (0..*n).map(|_| self.rand_elem(exclude_zero)).collect::<Vec<PrimeFieldElem>>();
    PrimeFieldElems(xs)
  }

  pub fn seq(&self, from: &PrimeFieldElem, to: &PrimeFieldElem, incl_last_elem: bool) -> Vec<PrimeFieldElem> {
    let mut i = from.clone();
    let mut xs = vec![];

    while &i <= to {
      if !incl_last_elem && &i == to {
        break;
      }
      xs.push(i.clone());
      i.inc();
    }
    xs
  }
}

impl PartialEq for PrimeField {
  fn eq(&self, other: &Self) -> bool {
    self.order == other.order
  }
}

impl Eq for PrimeField {}
