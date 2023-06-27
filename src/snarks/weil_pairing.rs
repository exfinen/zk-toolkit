#![allow(non_snake_case)]

use crate::building_block::{
  ec_point::EcPoint,
  field::{Field, FieldElem},
  to_biguint::ToBigUint,
  weierstrass_eq::WeierstrassEq,
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
    //_E: WeierstrassEq,
  ) -> Self {
    let k = WeilPairing::find_k(f_q, r);
    let q_to_k = f_q.order.pow(k);

    // F_qk contains mu_r of r-th roots of unity
    let f_qk = Field::new(&q_to_k);

    // compute E[m]; the set of m-torsion points of E


    WeilPairing { f_qk }
  }

  pub fn find_k(f_q: &Field, r: &FieldElem) -> u32 {
    let zero = &BigUint::Zero();
    let one = &BigUint::One();
    let mut k = 1u32;
    let mut f_q_pow = *f_q.order;
    loop {
      if (f_q_pow - one) % r.n == zero { break; }
      f_q_pow = f_q_pow * f_q.order;
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
