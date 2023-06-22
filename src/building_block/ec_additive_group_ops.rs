use crate::building_block::ec_point::EcPoint;
use crate::building_block::field::Field;
use num_bigint::BigUint;
use num_traits::identities::{Zero, One};
use std::ops::{BitAnd, ShrAssign};

pub trait EcAdditiveGroupOps {
  fn add(&self, p1: &EcPoint, p2: &EcPoint) -> EcPoint;

  fn inv(&self, p: &EcPoint) -> EcPoint;

  fn vector_add(&self, ps: &[&EcPoint]) -> EcPoint {
    if ps.len() == 0 {
      panic!("cannot get the sum of empty slice");
    } else if ps.len() == 1 {
      ps[0].clone()
    } else {
      let sum = ps[0].clone();
      for p in &ps[1..] {
        self.add(&sum, p);
      }
      sum
    }
  }

  fn get_zero(&self, f: &Field) -> EcPoint;

  fn is_zero(&self, p: &EcPoint) -> bool;

  fn scalar_mul(&self, pt: &EcPoint, multiplier: &BigUint) -> EcPoint {
    let mut n = multiplier.clone();
    let mut res = self.get_zero(&pt.x.f);
    let mut pt_pow_n = pt.clone();
    let one = BigUint::one();

    while !n.is_zero() {
      if n.clone().bitand(&one).is_one() {
        res = self.add(&res, &pt_pow_n);
      }
      pt_pow_n = self.add(&pt_pow_n, &pt_pow_n);
      n.shr_assign(1usize);
    }
    res
  }
}
