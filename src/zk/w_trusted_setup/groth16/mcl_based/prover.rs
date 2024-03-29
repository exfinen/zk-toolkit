use crate::{
  building_block::mcl::{
    mcl_fr::MclFr,
    mcl_g1::MclG1,
    mcl_g2::MclG2,
    polynomial::{
      DivResult,
      Polynomial,
    },
    qap::{
      equation_parser::EquationParser,
      gate::Gate,
      qap::QAP,
      r1cs::R1CS,
      r1cs_tmpl::R1CSTmpl,
      term::Term,
    },
  },
  zk::w_trusted_setup::groth16::mcl_based::{
    crs::CRS,
    proof::Proof,
    wires::Wires,
  },
};
use num_traits::Zero;
use std::collections::HashMap;

pub struct Prover {
  pub n: MclFr,  // # of constraints 
  pub l: MclFr,  // end index of statement variables
  pub m: MclFr,  // end index of statement + witness variables
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
    let eq = EquationParser::parse(expr).unwrap();

    let gates = &Gate::build(&eq);
    let tmpl = &R1CSTmpl::new(gates);

    let r1cs = R1CS::from_tmpl(tmpl, &witness_map).unwrap();
    r1cs.validate().unwrap();

    let qap = QAP::build(&r1cs);

    let t = QAP::build_t(&MclFr::from(tmpl.constraints.len()));
    let h = {
      let p = qap.build_p(&r1cs.witness);
      match p.divide_by(&t) {
        DivResult::Quotient(q) => q,
        _ => panic!("p should be divisible by t"),
      }
    };

    let l = &tmpl.mid_beg - MclFr::from(1);
    let m = MclFr::from(tmpl.witness.len() - 1);
    let wires = Wires::new(&r1cs.witness.clone(), &l);
    let n = MclFr::from(tmpl.constraints.len());

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

      for i in 0..=self.m.to_usize() {
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

      let wit_beg = self.l.to_usize() + 1;
      for i in wit_beg..=self.m.to_usize() {
        let ai = &self.wires[i];
        sum += &crs.g1.uvw_wit[i - wit_beg] * ai;
      }

      let ht_by_delta = self.h.eval_with_g1_hidings(&crs.g1.xt_by_delta);

      sum 
      + &ht_by_delta
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
    building_block::mcl::{
      pairing::Pairing,
      mcl_initializer::MclInitializer,
    },
    zk::w_trusted_setup::groth16::mcl_based::verifier::Verifier,
  };

  #[test]
  fn test_generate_proof_and_verify() {
    MclInitializer::init();

    let expr = "(x * x * x) + x + 5 == 35";
    println!("Expr: {}\n", expr);
    let eq = EquationParser::parse(expr).unwrap();

    let witness_map = {
      use crate::building_block::mcl::qap::term::Term::*;
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

