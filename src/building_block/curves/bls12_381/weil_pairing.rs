use crate::building_block::curves::bls12_381::{
    g1_point::G1Point,
    g2_point::G2Point,
    fq1::Fq1,
    fq12::Fq12,
    rational_function::RationalFunction,
  };

  pub struct WeilPairing {
    l_bits: Vec<bool>,
  }

  impl WeilPairing {
    pub fn new(l: Fq1) -> Self {
      let mut l_bits: Vec<bool> = vec![];

      for i in 0..l.bits() {
        l_bits.push(l.bit(i));
      }
      l_bits.reverse();

      WeilPairing { l_bits }
    }

    pub fn compute(&self, p: &G1Point, q: &G2Point) -> Fq12 {
      let one = Fq1::from_to_biguint(&1u8);
      let mut f = Fq12::from(&one);
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