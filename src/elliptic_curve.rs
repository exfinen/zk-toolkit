use num_bigint::BigUint;
use crate::ec_point::EcPoint;
use num_traits::identities::{Zero, One};
use std::ops::{BitAnd, ShrAssign};

pub trait AddOps {
  fn get_zero_point(&self) -> EcPoint; 

  fn add(&self, p1: &EcPoint, p2: &EcPoint) -> EcPoint;

  fn scalar_mul(&self, pt: &EcPoint, multiplier: &BigUint) -> EcPoint {
    let mut n = multiplier.clone();
    let mut res = self.get_zero_point();
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

pub trait EllipticCurve {
  fn g(&self) -> EcPoint;
  fn n(&self) -> BigUint;
  fn is_on_curve(&self, pt: &EcPoint) -> bool;
}