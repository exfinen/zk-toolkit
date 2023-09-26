use crate::building_block::{
  curves::bls12_381::{
    g1_point::G1Point,
    g2_point::G2Point,
    fq12::Fq12,
    params::Params as P,
    rational_function::RationalFunction,
  },
  to_biguint::ToBigUint,
};
use num_bigint::BigUint;
use num_traits::Zero;

pub struct Pairing {
  l_bits: Vec<bool>,
}

macro_rules! impl_miller_algorithm {
  ($p1: ty, $p2: ty, $func: ident, $new: ident, $eval_at: ident) => {
    impl Pairing {
      #[allow(non_snake_case)]
      pub fn $func(&self, p: &$p1, q: &$p2) -> Fq12 {
        let mut f = Fq12::from(&1u8 as &dyn ToBigUint);
        let mut V = p.clone();

        for bit in &self.l_bits {
          {
            let v2 = &(&V + &V);

            let g_num = RationalFunction::$new(&V, &V);
            let g_deno = RationalFunction::$new(v2, &-v2);

            f = &(&f * &f) * g_num.$eval_at(q) * g_deno.$eval_at(q).inv();
          }
          V = &V + &V;

          if *bit {
            {
              let v_plus_p = &(&V + p);
              let g_num = RationalFunction::$new(&V, p);
              let g_deno = RationalFunction::$new(v_plus_p, &(-v_plus_p));
              f = f * g_num.$eval_at(q) * g_deno.$eval_at(q).inv();
            }
            V = &V + p;
          }
        }
        f
      }
    }
  };
}
impl_miller_algorithm!(G1Point, G2Point, calc_g1_g2, new_g1, eval_with_g2);
impl_miller_algorithm!(G2Point, G1Point, calc_g2_g1, new_g2, eval_with_g1);

impl Pairing {
  pub fn new() -> Self {
    // TODO explain why subtracting 1
    let mut l = P::subgroup().order_ref() - &BigUint::from(1u8);
    let mut l_bits: Vec<bool> = vec![];
    let one = BigUint::from(1u8);
    
    while !l.is_zero() {
      let b = &l & &one;
      l_bits.push(!b.is_zero());
      l = l >> 1u32;
    }
    l_bits.reverse();
    l_bits.remove(0);  // drop msb 1 

    Pairing { l_bits }
  }

  pub fn weil(&self, p: &G1Point, q: &G2Point) -> Fq12 {
    let num = self.calc_g1_g2(p, q);
    let deno = self.calc_g2_g1(q, p);

    num * deno.inv()
  }

  pub fn tate(&self, p: &G1Point, q: &G2Point) -> Fq12 {
    let intmed = self.calc_g1_g2(p, q);

    // apply final exponentiation
    let one = BigUint::from(1u8);
    let exp = 
      (P::base_prime_field().order_ref().pow(P::embedding_degree()) - one)
      / P::subgroup().order_ref();
    let exp = Fq12::from(&exp as &dyn ToBigUint);
    intmed * exp
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn test_pairing(pairing: &Pairing, p: &G1Point, q: &G2Point) -> bool {
    let p2 = p + p;

    // test e(p + p2, q) = e(p, q) e(p2, q)

    // println!("Calculating e(p + p2, q)...");
    let lhs = pairing.weil(&(p + &p2), q);

    // println!("Calculating e(p, q)...");
    let rhs1 = pairing.weil(p, q);

    // println!("Calculating e(p2, q)...");
    let rhs2 = pairing.weil(&p2, q);

    let rhs = rhs1 * rhs2;

    // println!("lhs = {:?}", &lhs);
    // println!("rhs = {:?}", &rhs);

    lhs == rhs
  }

  #[test]
  fn test_weil_pairing_with_generators() {
    let wp = Pairing::new();
    let p = G1Point::g();
    let q = G2Point::g();
    let res = test_pairing(&wp, &p, &q);
    assert!(res);
  }

  #[test]
  fn test_weil_pairing_with_random_points() {
    let mut errors = 0;
    let num_tests = 1;

    for i in 0..num_tests {
      println!("iteration {}", i);
      let wp = Pairing::new();
      let p = G1Point::get_random_point();
      let q = G2Point::get_random_point();
      let res = test_pairing(&wp, &p, &q);
      if res == false {
        println!("----> iteration {} failed!", i);
        errors += 1;
      }
    }
    println!("{} tests failed!", errors);
  }
}

