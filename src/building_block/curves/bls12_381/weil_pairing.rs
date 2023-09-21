use crate::building_block::{
  curves::bls12_381::{
    g1_point::G1Point,
    g2_point::G2Point,
    fq12::Fq12,
    rational_function::RationalFunction,
  },
  to_biguint::ToBigUint,
};
use num_bigint::BigUint;
use num_traits::Zero;

pub struct WeilPairing {
  l_bits: Vec<bool>,
}

macro_rules! impl_miller_algorithm {
  ($p1: ty, $p2: ty, $func: ident, $new: ident, $eval_at: ident) => {
    impl WeilPairing {
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
impl_miller_algorithm!(G1Point, G2Point, calc_numerator, new_g1, eval_with_g2);
impl_miller_algorithm!(G2Point, G1Point, calc_denominator, new_g2, eval_with_g1);

impl WeilPairing {
  pub fn new() -> Self {
    // l is the group order of G1, G2 and G12 curves
    let mut l = BigUint::parse_bytes(b"73EDA753299D7D483339D80809A1D80553BDA402FFFE5BFEFFFFFFFF00000000", 16).unwrap();
    let mut l_bits: Vec<bool> = vec![];
    let one = BigUint::from(1u8);
    
    while !l.is_zero() {
      let b = &l & &one;
      l_bits.push(!b.is_zero());
      l = l >> 1u32;
    }
    l_bits.reverse();
    l_bits.remove(0);  // drop msb 1 

    WeilPairing { l_bits }
  }

  pub fn calculate(&self, p: &G1Point, q: &G2Point) -> Fq12 {
    let num = self.calc_numerator(p, q);
    let deno = self.calc_denominator(q, p);

    num * deno.inv()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn do_it() {
    let wp = WeilPairing::new();

    let p = G1Point::g();
    let p2 = &p + &p;
    let q = G2Point::g();

    // test e(p + p2, q) = e(p, q) e(p2, q)

    println!("Calculating e(p + p2, q)...");
    let lhs = wp.calculate(&(&p + &p2), &q);

    println!("Calculating e(p, q)...");
    let rhs1 = wp.calculate(&p, &q);

    println!("Calculating e(p2, q)...");
    let rhs2 = wp.calculate(&p2, &q);

    let rhs = rhs1 * rhs2;

    println!("lhs = {:?}", &lhs);
    println!("rhs = {:?}", &rhs);

    assert!(lhs == rhs);
  }
}



