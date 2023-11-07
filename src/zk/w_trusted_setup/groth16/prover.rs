use crate::{
  building_block::{
    curves::bls12_381::{
      g1_point::G1Point,
      g2_point::G2Point,
    },
    field::{
      polynomial::{
        Polynomial,
        DivResult,
      },
      prime_field::PrimeField,
      prime_field_elem::PrimeFieldElem,
    },
    zero::Zero,
  },
  zk::w_trusted_setup::{
    qap::{
      equation_parser::EquationParser,
      gate::Gate,
      qap::QAP,
      r1cs::R1CS,
      r1cs_tmpl::R1CSTmpl,
      term::Term,
    },
    groth16::{
      crs::CRS,
      proof::Proof,
      wires::Wires,
    },
  },
};
use std::collections::HashMap;

pub struct Prover {
  pub f: PrimeField,
  pub n: usize,  // # of constraints 
  pub l: usize,  // end index of statement variables
  pub m: usize,  // end index of statement + witness variables
  pub wires: Wires,
  pub h: Polynomial,
  pub t: Polynomial,
  pub ui: Vec<Polynomial>,
  pub vi: Vec<Polynomial>,
  pub wi: Vec<Polynomial>,
}

impl Prover {
  pub fn new(
    f: &PrimeField,
    expr: &str,
    witness_map: &HashMap<Term, PrimeFieldElem>,
  ) -> Self {
    let eq = EquationParser::parse(f, expr).unwrap();

    let gates = &Gate::build(f, &eq);
    let tmpl = &R1CSTmpl::new(f, gates);

    let r1cs = R1CS::from_tmpl(f, tmpl, &witness_map).unwrap();
    r1cs.validate().unwrap();

    let qap = QAP::build(f, &r1cs);

    let t = QAP::build_t(f, &tmpl.constraints.len());
    let h = {
      let p = qap.build_p(&r1cs.witness);
      match p.divide_by(&t) {
        DivResult::Quotient(q) => q,
        _ => panic!("p should be divisible by t"),
      }
    };

    let l  = {
      let wit_beg: usize = (&tmpl.mid_beg.e).try_into().unwrap();
      wit_beg - 1
    };
    let m = tmpl.witness.len() - 1;
    let wires = Wires::new(f, &r1cs.witness.clone(), &l);
    let n = tmpl.constraints.len();

    Prover {
      f: f.clone(),
      n,
      l,
      m,
      wires,
      t,
      h,
      ui: qap.vi.clone(),
      vi: qap.wi.clone(),
      wi: qap.yi.clone(),
    }
  }

  #[allow(non_snake_case)]
  pub fn prove(&self, crs: &CRS) -> Proof {
    println!("--> Generating proof...");
    let f = &self.f;

    let r = &f.rand_elem(true);
    let s = &f.rand_elem(true);

    let (A, B, B_g1) = {
      let mut sum_term_A = G1Point::zero();
      let mut sum_term_B = G2Point::zero();
      let mut sum_term_B_g1 = G1Point::zero();

      for i in 0..=self.m {
        let ai = &self.wires[i];
        let ui_prod = self.ui[i].eval_with_g1_hidings(&crs.g1.xi) * ai;
        let vi_prod = self.vi[i].eval_with_g2_hidings(&crs.g2.xi) * ai;
        let vi_prod_g1 = self.vi[i].eval_with_g1_hidings(&crs.g1.xi) * ai;

        sum_term_A += ui_prod;
        sum_term_B += vi_prod;
        sum_term_B_g1 += vi_prod_g1;
      }
      let A = &crs.g1.alpha + &sum_term_A + &crs.g1.delta * r;
      let B = &crs.g2.beta + &sum_term_B + &crs.g2.delta * s;
      let B_g1 = &crs.g1.beta + &sum_term_B_g1 + &crs.g1.delta * s;
      (A, B, B_g1)
    };

    let C = {
      let mut sum = G1Point::zero();

      let wit_beg = self.l + 1;
      for i in wit_beg..=self.m {
        let ai = &self.wires[i];
        sum += &crs.g1.uvw_wit[i - wit_beg] * ai;
      }
      sum 
      + &crs.g1.ht_by_delta
      + &A * s
      + &B_g1 * r
      + -(&crs.g1.delta * r * s)
    };

    Proof {
      A,
      B,
      C,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    building_block::curves::bls12_381::pairing::Pairing,
    zk::w_trusted_setup::groth16::verifier::Verifier,
  };

  #[test]
  fn test_generate_proof_and_verify() {
    let f = &G1Point::curve_group();

    let expr = "(x * x * x) + x + 5 == 35";
    println!("Expr: {}\n", expr);
    let eq = EquationParser::parse(f, expr).unwrap();

    let witness_map = {
      use crate::zk::w_trusted_setup::qap::term::Term::*;
      HashMap::<Term, PrimeFieldElem>::from([
        (Term::One, f.elem(&1u8)),
        (Term::var("x"), f.elem(&3u8)),
        (TmpVar(1), f.elem(&9u8)),
        (TmpVar(2), f.elem(&27u8)),
        (TmpVar(3), f.elem(&8u8)),
        (TmpVar(4), f.elem(&35u8)),
        (Out, eq.rhs),
      ])
    };
    let prover = &Prover::new(f, expr, &witness_map);
    let pairing = &Pairing::new();
    let verifier = &Verifier::new(pairing);
    let crs = CRS::new(f, prover, pairing);

    let proof = prover.prove(&crs);
    let stmt_wires = &prover.wires.statement();
    let result = verifier.verify(
      &proof,
      &crs,
      stmt_wires,
    );

    assert!(result);
  }
}

