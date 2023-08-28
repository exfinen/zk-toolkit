use crate::building_block::{
  curves::bls12_381::{
    g1_point::G1Point,
    g2_point::G2Point,
    fq1::Fq1,
    fq12::Fq12,
    rational_function::RationalFunction,
  },
  to_biguint::ToBigUint,
};

pub struct WeilPairing {
  l_bits: Vec<bool>,
}

impl WeilPairing {
  pub fn new(l: &Fq1) -> Self {
    let mut l_bits: Vec<bool> = vec![];

    for i in 0..l.bits() {
      l_bits.push(l.bit(i));
    }
    l_bits.reverse();

    WeilPairing { l_bits }
  }

  #[allow(non_snake_case)]
  pub fn run_miller_algorithm(&self, p: &G1Point, q: &G2Point) -> Fq12 {
    let mut f = Fq12::from(&1u8 as &dyn ToBigUint);
    let mut V = p.clone();

    for bit in &self.l_bits {
      V = &V + &V;
      {
        let v2 = &(&V + &V);
        let g_num = RationalFunction::new(&V, &V);
        let g_deno = RationalFunction::new(v2, &-v2);
        f = &(&f * &f) * g_num.eval_at(q) * g_deno.eval_at(q).inv();
      }

      if *bit {
        V = &V + p;
        {
          let v_plus_p = &(&V + p);
          let g_num = RationalFunction::new(&V, p);
          let g_deno = RationalFunction::new(v_plus_p, &(-v_plus_p));
          f = f * g_num.eval_at(q) * g_deno.eval_at(q).inv();
        }
      }
    }
    f
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use num_bigint::BigUint;

  #[test]
  fn do_it() {
    let l = BigUint::parse_bytes(b"d201000000010000", 16).unwrap();  // l-torsion points

    let l_fq1 = Fq1::from_to_biguint(&l);
    let wp = WeilPairing::new(&l_fq1);

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


