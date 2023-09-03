use crate::building_block::{
  curves::bls12_381::{
    g1_point::G1Point,
    g2_point::G2Point,
    fq12::Fq12,
    rational_function::RationalFunction,
  },
  to_biguint::ToBigUint,
};

pub struct WeilPairing {
  l_bits: Vec<bool>,
}

impl WeilPairing {
  pub fn new() -> Self {
    let mut l = u64::from_str_radix("d201000000010000", 16).unwrap();  // l-torsion points
    let mut l_bits: Vec<bool> = vec![];
    
    while l > 0 {
      let b = l & 1;
      l_bits.push(b != 0);
      l = l >> 1
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
        let v2 = &(&V + &V);
        let g_num = RationalFunction::new(&V, &V);
        let g_deno = RationalFunction::new(v2, &-v2);
        f = &(&f * &f) * g_num.eval_at(q) * g_deno.eval_at(q).inv();
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

    // test e(p1 + p2, q) = e(p1, q) e(p2, q)
    let lhs = wp.run_miller_algorithm(&p1_p2, &q);
    let rhs1 = wp.run_miller_algorithm(&p1, &q);
    let rhs2 = wp.run_miller_algorithm(&p2, &q);

    let rhs = rhs1 * rhs2;
    assert!(lhs == rhs);
  }
}


