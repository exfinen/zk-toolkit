#![allow(non_snake_case)]

/*
use crate::building_block::{
  field::{Field, FieldElem},
  to_biguint::ToBigUint,
  elliptic_curve::{
    ec_point::EcPoint,
  },
};
use num_bigint::BigUint;
use num_traits::{One, Zero};

#[derive(Clone)]
pub struct WeilPairing {
  pub f_qk: Field, // extention field of q^k
}

impl WeilPairing {
  pub fn new(
    f_q: &Field,    // base field F_q
    r: &FieldElem,  // prime s.t. r | n (#E(F_q))
    E: WeierstrassEq,
    curve_order: &FieldElem,
    rational_point_on_E: &EcPoint,  // can be any point on E
  ) -> Self {
    let k = WeilPairing::find_embedding_degree(f_q, r);
    let q_to_k = f_q.order.pow(k);

    // extension fielf F_qk contains mu_r of r-th roots of unity
    let f_qk = Field::new(&q_to_k);

    // compute E[r]; the set of r-torsion points of E
    let _r_torsion_points = WeilPairing::calc_r_torsion_points(
      &f_qk,
      &curve_order,
      &r,
      &E,
      rational_point_on_E,
    );

    WeilPairing { f_qk }
  }

  pub fn calc_r_torsion_points(
    f_qk: &Field,  // extension field of order q^k
    curve_order: &FieldElem,
    r: &FieldElem,
    _E: &WeierstrassEq,
    rational_point_on_E: &EcPoint,
  ) -> Vec<EcPoint> {
    if (&curve_order.n % &r.n).is_zero() == false {
      panic!("the curve order is not divisible by r");
    }
    // you can find a generator point for the subgroup of order n
    // by taking any point P of order m on the curve, and computing
    // the point Q = (m/n)P. This point Q will have order n.
    //
    // nQ = n(m/n)P = mP = 0
    let ops = Secp256k1JacobianAddOps::new(f_qk);
    let cofactor = curve_order / r;
    let g: EcPoint = ops.scalar_mul(rational_point_on_E, &cofactor);

    let mut torsion_points = vec![];
    let mut p = g.clone();
    while {
      torsion_points.push(p.clone());
      p = ops.add(&p, &g);
      p != g
    } {}

    torsion_points
  }

  pub fn find_embedding_degree(f_q: &Field, r: &FieldElem) -> u32 {
    let zero: BigUint = Zero::zero();
    let one: BigUint = One::one();
    let mut k = 1u32;
    let q = &*f_q.order.clone();
    let mut f_q_pow = q.clone();
    loop {
      if (&f_q_pow - &one) % &r.n == zero { break; }
      f_q_pow = f_q_pow * q;
      k += 1;
    }
    k
  }

  pub fn get_torsion_points(_n: &impl ToBigUint) -> Vec<EcPoint> {
    vec!{}
  }
}

// #[cfg(test)]
// mod tests {
//   use super::*;

//   #[test]
//   fn test_foo() {
//     // let curve = WeierstrassEq::secp256k1();
//     // let ops = JacobianAddOps::new();
//     let q = 2u8.pow(7);
//     let k = 4;
//     let pairing = WeilPairing::new(q, k);

//     WeilPairing::get_torsion_points(n)
//   }
// }

 */