#![allow(non_snake_case)]

use crate::building_block::{
  ec_point::EcPoint,
  field::Field,
  to_biguint::ToBigUint,
  weierstrass_eq::WeierstrassEq,
};
use num_bigint::BigUint;

#[derive(Clone)]
pub struct WeilPairing {
  pub f_qk: Field,   // extention field of q^k
}

impl WeilPairing {
  pub fn new(
    q: BigUint,  // base field order
    k: u32,      // embedding degree
    _E: WeierstrassEq,
    _n: BigUint,  // order of E
  ) -> Self {
    let q_to_k = q.pow(k);
    let f_qk = Field::new(&q_to_k);
    WeilPairing { f_qk }
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
