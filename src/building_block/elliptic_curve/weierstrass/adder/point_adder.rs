use std::ops::Add;
use crate::building_block::{
  elliptic_curve::{
    curve::Curve,
    ec_point::EcPoint,
  },
  field::{
    prime_field::PrimeField,
    prime_field_elem::PrimeFieldElem,
  },
};

pub trait PointAdder<P, C>
  where
    P: Add<P>,
    C: Curve<EcPoint, PrimeFieldElem, PrimeField>,
{
  fn add(curve: &C, p1: &P, p2: &P) -> P;
}
