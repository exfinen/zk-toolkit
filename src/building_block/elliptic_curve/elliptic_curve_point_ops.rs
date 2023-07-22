use crate::building_block::{
  additive_identity::AdditiveIdentity,
  elliptic_curve::{
    affine_point::AffinePoint,
    curve::Curve,
    weierstrass::adder::point_adder::PointAdder,
  },
  field::field_elem_ops::Inverse,
  zero::Zero,
};
use std::ops::{Add, Sub, Mul, Div};

pub trait EllipticCurvePointOps<P, E, F, C>
  where
    C: Curve<P, E, F>,
    E: Zero<E> + Add<E> + Sub<E> + Mul<E> + Div<E> + AdditiveIdentity<E> + Clone,
    P: AffinePoint<Element=E> + Zero<P> + Add<P> + AdditiveIdentity<P> + Clone + Inverse,
{
  type Adder: PointAdder<P, E, F, C>;

  fn add(curve: &C, p1: &P, p2: &P) -> P {
    Self::Adder::add(curve, p1, p2)
  }

  fn vector_add(curve: &C, ps: &[&P]) -> P {
    if ps.len() == 0 {
      panic!("cannot get the sum of empty slice");
    } else if ps.len() == 1 {
      ps[0].clone()
    } else {
      let sum = ps[0].clone();
      for p in &ps[1..] {
        Self::Adder::add(curve, &sum, p);
      }
      sum
    }
  }

  fn scalar_mul(curve: &C, pt: &P, multiplier: &E) -> P {
    let mut n = multiplier.clone();
    let mut res = P::get_zero(pt);
    let mut pt_pow_n = pt.clone();
    let one = multiplier.f.elem(&1u8);

    while !n.is_zero() {
      if n.clone().bitand(&one).is_one() {
        res = Self::Adder::add(curve, &res, &pt_pow_n);
      }
      pt_pow_n = Self::Adder::add(curve, &pt_pow_n, &pt_pow_n);
      n.shr_assign(1usize);
    }
    res
  }

  fn inv(&self, p: &P) -> P {
    P::new(&p.x, &p.y.inv())
  }
}
