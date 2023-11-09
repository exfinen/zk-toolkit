use crate::building_block::{
  curves::bls12_381::{
    g1_point::G1Point,
    g2_point::G2Point,
    gt_point::GTPoint,
    fq12::Fq12,
    params::Params as P,
    rational_function::RationalFunction,
  },
  to_biguint::ToBigUint,
};
use num_bigint::BigUint;
use num_traits::Zero as NumTraitsZero;

#[derive(Clone)]
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
    // TODO find out the reason for subtracting 1
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

  pub fn weil(&self, p1: &G1Point, p2: &G2Point) -> GTPoint {
    println!("Started Weil pairing");
    println!("Running Miller loop G1-G2...");

    let num = self.calc_g1_g2(p1, p2);
    println!("Running Miller loop G2-G1...");
    let deno = self.calc_g2_g1(p2, p1);
    let e = num * deno.inv();
    GTPoint::new(&e)
  }

  pub fn tate(&self, p1: &G1Point, p2: &G2Point) -> GTPoint {
    println!("Started Tate pairing");
    println!("Running Miller loop G1-G2...");

    let intmed = self.calc_g1_g2(&p1, &p2);

    // apply final exponentiation
    println!("Applying final exponentiation...");
    let one = BigUint::from(1u8);
    let q_to_12 = P::base_prime_field().order_ref().pow(P::embedding_degree());
    let r = P::subgroup().order();
    let exp = (q_to_12 - one) / r;
    let e = intmed.pow(&exp);
    GTPoint::new(&e)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn test(
    pairing: &Pairing,
    pair: &dyn Fn(&Pairing, &G1Point, &G2Point) -> GTPoint,
    p1: &G1Point,
    p2: &G2Point,
  ) -> bool {
    let ten_p1s = p1 * P::subgroup().elem(&10u8);

    // test e(p1 + ten_p1s, p2) = e(p1, p2) e(ten_p1s, p2)
    let lhs = pair(pairing, &(p1 + &ten_p1s), p2);
    let rhs1 = pair(pairing, p1, p2);
    let rhs2 = pair(pairing, &ten_p1s, p2);

    let rhs = rhs1 * rhs2;

    lhs == rhs
  }

  fn test_with_generators(
    pair: &dyn Fn(&Pairing, &G1Point, &G2Point) -> GTPoint,
  ) {
    let pairing = &Pairing::new();
    let p1 = G1Point::g();
    let p2 = G2Point::g();
    let res = test(pairing, pair, &p1, &p2);
    assert!(res);
  }

  fn test_with_random_points(
    pair: &dyn Fn(&Pairing, &G1Point, &G2Point) -> GTPoint,
  ) {
    let mut errors = 0;
    let num_tests = 1;

    for _ in 0..num_tests {
      let pairing = &Pairing::new();
      let p1 = G1Point::get_random_point();
      let p2 = G2Point::get_random_point();
      let res = test(pairing, pair, &p1, &p2);
      if res == false {
        errors += 1;
      }
    }
    assert!(errors == 0);
  }

  fn test_plus_to_mul(
    pair: &dyn Fn(&Pairing, &G1Point, &G2Point) -> GTPoint,
  ) {
    let pairing = &Pairing::new();
    let one = &G2Point::g();

    let p = &(G1Point::g() + G1Point::g());

    let lhs = {
      let p_plus_p = p + p;
      pair(pairing, &p_plus_p, one)
    };

    let rhs = {
      let a = &pair(pairing, &p, one); 
      a * a
    };
    assert!(lhs == rhs);
  }

  #[test]
  fn test_weil_pairing_with_generators() {
    test_with_generators(&Pairing::weil);
  }

  #[test]
  fn test_weil_pairing_with_random_points() {
    test_with_random_points(&Pairing::weil);
  }

  #[test]
  fn test_tate_pairing_with_generators() {
    test_with_generators(&Pairing::tate);
  }

  #[test]
  fn test_tate_pairing_with_random_points() {
    test_with_random_points(&Pairing::tate);
  }

  #[test]
  fn test_tate_pairing_with_test_plus_to_mul() {
    test_plus_to_mul(&Pairing::tate);
  }

  #[test]
  fn test_signature_verification() {
    let pairing = &Pairing::new();
    let g1 = &G1Point::g();
    let sk = &P::subgroup().elem(&2u8);
    let pk = &(g1 * sk);

    let m = &b"hamburg steak".to_vec();
    let hash_m = &G2Point::hash_to_g2point(m);

    // e(pk, H(m)) = e(g1*sk, H(m)) = e(g1, sk*H(m))
    let lhs = pairing.tate(pk, hash_m);
    let rhs = pairing.tate(g1, &(hash_m * sk));

    assert!(lhs == rhs);
  }
}

