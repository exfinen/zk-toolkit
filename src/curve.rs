use num_bigint::BigUint;
use crate::ec_point::EcPoint;

pub trait Curve {
  fn g(&self) -> EcPoint;
  fn n(&self) -> BigUint;
  fn add(&self, p1: &EcPoint, p2: &EcPoint) -> EcPoint;
  fn scalar_mul(&self, pt: &EcPoint, multiplier: &BigUint) -> EcPoint;
  fn is_on_curve(&self, pt: &EcPoint) -> bool;
}