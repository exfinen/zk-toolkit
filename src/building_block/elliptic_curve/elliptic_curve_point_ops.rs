use crate::building_block::{
  elliptic_curve::{
    curve::Curve,
    ec_point::EcPoint,
    weierstrass::adder::point_adder::PointAdder,
  },
  field::{
    field_elem_ops::Inverse,
    prime_field::PrimeField,
    prime_field_elem::PrimeFieldElem,
  },
  zero::Zero, additive_identity::AdditiveIdentity,
};
use std::ops::Add;

pub trait EllipticCurvePointOps<P, E, F, C>
  where
    C: Curve<P, E, F>,
    E: Clone,
    P: Zero<P> + Add<P> + AdditiveIdentity<P> + Clone + Inverse,
{
  type Adder: PointAdder<P, C>;

  fn add(&self, p1: &P, p2: &P) -> P {
    Self::Adder::add(self, p1, p2)
  }

  fn vector_add(&self, ps: &[&P]) -> P {
    if ps.len() == 0 {
      panic!("cannot get the sum of empty slice");
    } else if ps.len() == 1 {
      ps[0].clone()
    } else {
      let sum = ps[0].clone();
      for p in &ps[1..] {
        Self::Adder::add(self, &sum, p);
      }
      sum
    }
  }

  fn scalar_mul(&self, pt: &P, multiplier: &E) -> P {
    let mut n = multiplier.clone();
    let mut res = P::get_zero(pt);
    let mut pt_pow_n = pt.clone();
    let one = multiplier.f.elem(&1u8);

    while !n.is_zero() {
      if n.clone().bitand(&one).is_one() {
        res = Self::Adder::add(self, &res, &pt_pow_n);
      }
      pt_pow_n = Self::Adder::add(self, &pt_pow_n, &pt_pow_n);
      n.shr_assign(1usize);
    }
    res
  }

  fn inv(p: &P) -> P {
    P::new(&p.x, &p.y.inv())
  }
}
