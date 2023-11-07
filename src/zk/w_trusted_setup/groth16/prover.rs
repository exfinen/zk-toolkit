use crate::{
  building_block::curves::mcl::{
    mcl_fr::MclFr,
    mcl_g1::MclG1,
    mcl_g2::MclG2,
    pairing::Pairing,
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
    expr: &str,
    witness_map: &HashMap<Term, MclFr>,
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
    let wires = Wires::new(&r1cs.witness.clone(), &l);
    let n = tmpl.constraints.len();

    Prover {
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
    let r = &MclFr::rand(true);
    let s = &MclFr::rand(true);

    let (A, B, B_g1) = {
      let mut sum_term_A = MclG1::zero();
      let mut sum_term_B = MclG2::zero();
      let mut sum_term_B_g1 = MclG1::zero();

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
      let mut sum = MclG1::zero();

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
    building_block::curves::mcl::pairing::Pairing,
    zk::w_trusted_setup::groth16::verifier::Verifier,
  };

  #[test]
  fn test_generate_proof_and_verify() {
    let expr = "(x * x * x) + x + 5 == 35";
    println!("Expr: {}\n", expr);
    let eq = EquationParser::parse(expr).unwrap();

    let witness_map = {
      use crate::zk::w_trusted_setup::qap::term::Term::*;
      HashMap::<Term, MclFr>::from([
        (Term::One, MclFr::from(1)),
        (Term::var("x"), MclFr::from(3)),
        (TmpVar(1), MclFr::from(9)),
        (TmpVar(2), MclFr::from(27)),
        (TmpVar(3), MclFr::from(8)),
        (TmpVar(4), MclFr::from(35)),
        (Out, eq.rhs),
      ])
    };
    let prover = &Prover::new(expr, &witness_map);
    let pairing = &Pairing;
    let verifier = &Verifier::new(pairing);
    let crs = CRS::new(prover, pairing);

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

