use crate::building_block::{
  field::{
    field_elem_ops::Inverse,
    field::Field,
  },
  zero::Zero, additive_identity::AdditiveIdentity,
};
use std::ops::{BitAnd, ShrAssign};

pub trait EllipticCurvePointOps<P, E, F>
  where
    F: Field<F>,
    P: Zero<P> + AdditiveIdentity<P> + AdditiveIdentity<E> + Clone + Inverse,
{
  type Adder;

  fn add(f: &F, p1: &P, p2: &P) -> P {
    Self::Adder::add(f, p1, p2);
  }

  fn vector_add(f: &F, ps: &[&P]) -> P {
    if ps.len() == 0 {
      panic!("cannot get the sum of empty slice");
    } else if ps.len() == 1 {
      ps[0].clone()
    } else {
      let sum = ps[0].clone();
      for p in &ps[1..] {
        Self::Adder::add(f, &sum, p);
      }
      sum
    }
  }

  fn scalar_mul(f: &F, pt: &P, multiplier: &E) -> P {
    let mut n = multiplier.clone();
    let mut res = P::get_zero(pt);
    let mut pt_pow_n = pt.clone();
    let one = multiplier.f.elem(&1u8);

    while !n.is_zero() {
      if n.clone().bitand(&one).is_one() {
        res = Self::Adder::add(f, &res, &pt_pow_n);
      }
      pt_pow_n = Self::Adder::add(f, &pt_pow_n, &pt_pow_n);
      n.shr_assign(1usize);
    }
    res
  }

  fn inv(p: &P) -> P {
    P::new(&p.x, &p.y.inv())
  }
}
