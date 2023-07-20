use crate::building_block::{
  elliptic_curve::{
    affine_point::AffinePoint,
    new_affine_point::NewAffinePoint,
  },
  zero::Zero, additive_identity::AdditiveIdentity,
};
use std::ops::{BitAnd, ShrAssign};

pub trait EllipticCurvePointOps<P, E>
  where
    P: Zero<P> + AdditiveIdentity + Clone,
{
  type Adder;

  fn add(&self, p1: &P, p2: &P) -> P;

  fn vector_add(&self, ps: &[&P]) -> P {
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

  fn scalar_mul(&self, pt: &P, multiplier: &E) -> P {
    let mut n = multiplier.clone();
    let mut res = self.get_zero(&pt.x.f);
    let mut pt_pow_n = pt.clone();
    let one = multiplier.f.elem(&1u8);

    while !n.is_zero() {
      if n.clone().bitand(&one).is_one() {
        res = self.add(&res, &pt_pow_n);
      }
      pt_pow_n = self.add(&pt_pow_n, &pt_pow_n);
      n.shr_assign(1usize);
    }
    res
  }

  fn inv(&self, p: &P) -> P {
    P::new(&p.x, &p.y.inv())
  }
}
