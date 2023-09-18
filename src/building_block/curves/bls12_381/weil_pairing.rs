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

impl WeilPairing {
  pub fn new() -> Self {
    let mut l = BigUint::parse_bytes(b"73EDA753299D7D483339D80809A1D80553BDA402FFFE5BFEFFFFFFFF00000000", 16).unwrap();  // l-torsion points
    let mut l_bits: Vec<bool> = vec![];
    let one = BigUint::from(1u8);
    
    while !l.is_zero() {
      let b = &l & &one;
      l_bits.push(!b.is_zero());
      l = l >> 1u32;
    }
    l_bits.reverse();
    l_bits.remove(0);  // drop msb 1 

    for b in &l_bits {
      print!("{}", if *b { 1 } else { 0 });
    }
    println!("");

    WeilPairing { l_bits }
  }

  #[allow(non_snake_case)]
  pub fn run_miller_algorithm(&self, p: &G1Point, q: &G2Point) -> Fq12 {
    let mut f = Fq12::from(&1u8 as &dyn ToBigUint);
    let mut V = p.clone();

    for bit in &self.l_bits {
      {
        println!("G1 point before dbl:\n{:?}\n", &V);
        let v2 = &(&V + &V);
        println!("G1 point after dbl:\n{:?}\n", &v2);

        let g_num = RationalFunction::new(&V, &V);
        let g_deno = RationalFunction::new(v2, &-v2);

        // this is not working!!!
        println!("Fp12 element before dbl:\n{}\n", &f);
        f = &(&f * &f) * g_num.eval_at(q) * g_deno.eval_at(q).inv();
        println!("Fp12 element after dbl:\n{}\n", &f);
      }
      V = &V + &V;

      if *bit {
        {
          let v_plus_p = &(&V + p);
          let g_num = RationalFunction::new(&V, p);
          let g_deno = RationalFunction::new(v_plus_p, &(-v_plus_p));
          f = f * g_num.eval_at(q) * g_deno.eval_at(q).inv();
        }
        V = &V + p;
      }
    }
    f
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn do_it() {
    let wp = WeilPairing::new();

    let p1 = G1Point::g();
    let p2 = &p1 + &p1;
    let p1_p2 = &p1 + &p2;
    let q = G2Point::g();

    let _lhs = wp.run_miller_algorithm(&p1, &q);
 
    // test e(p1 + p2, q) = e(p1, q) e(p2, q)
    // let _lhs = wp.run_miller_algorithm(&p1_p2, &q);
    // let rhs1 = wp.run_miller_algorithm(&p1, &q);
    // let rhs2 = wp.run_miller_algorithm(&p2, &q);

    // let rhs = rhs1 * rhs2;
    // assert!(lhs == rhs);
  }
}


