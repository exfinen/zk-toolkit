use crate::{
  building_block::{
    curves::bls12_381::{
      g1_point::G1Point,
      g2_point::G2Point,
    },
    field::{
      prime_field::PrimeField,
      prime_field_elem::PrimeFieldElem,
    },
    zero::Zero,
  },
  zk::w_trusted_setup::pinocchio::{
    crs::CRS,
    equation_parser::EquationParser,
    gate::Gate,
    qap::QAP,
    polynomial::{
      DivResult,
      Polynomial,
    },
    pinocchio_proof::PinocchioProof,
    r1cs::R1CS,
    r1cs_tmpl::R1CSTmpl,
    term::Term,
    witness::Witness,
  },
};
use std::collections::HashMap;

pub struct PinocchioProver {
  pub f: PrimeField,
  pub max_degree: usize,
  pub num_constraints: usize,
  pub witness: Witness,
  pub p: Polynomial,
  pub t: Polynomial,
  pub vi: Vec<Polynomial>,
  pub wi: Vec<Polynomial>,
  pub yi: Vec<Polynomial>,
}

impl PinocchioProver {
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
    let p = qap.build_p(&r1cs.witness);

    let max_degree: usize = {
      let xs = vec![
        &qap.vi[..],
        &qap.wi[..],
        &qap.yi[..],
        &vec![p.clone(), t.clone()],
      ].concat();
      let n: PrimeFieldElem = xs.iter().map(|x| x.degree()).max().unwrap();
      let n: usize = n.e.try_into().unwrap();
      n + 1
    };

    let witness = Witness::new(&r1cs.witness.clone(), &tmpl.mid_beg);
    let num_constraints = tmpl.constraints.len();

    PinocchioProver {
      f: f.clone(),
      max_degree,
      num_constraints,
      witness,
      p,
      t,
      vi: qap.vi.clone(),
      wi: qap.wi.clone(),
      yi: qap.yi.clone(),
    }
  }

  pub fn prove(&self, crs: &CRS) -> PinocchioProof {
    println!("--> Generating proof...");
    let witness_mid = &self.witness.mid();

    let ek = &crs.ek;

    let mut g_v_v_mid_s = G1Point::zero();
    let mut g1_w_w_mid_s = G1Point::zero();
    let mut g2_w_w_mid_s = G2Point::zero();
    let mut g_y_y_mid_s = G1Point::zero();
    let mut g_v_alpha_v_mid_s = G1Point::zero();
    let mut g_w_alpha_w_mid_s = G1Point::zero();
    let mut g_y_alpha_y_mid_s = G1Point::zero();
    let mut g_beta_vwy_mid_s = G1Point::zero();
  
    for i in 0..witness_mid.size_in_usize() {
      let w = &witness_mid[&i];

      g_v_v_mid_s = &g_v_v_mid_s + &ek.g_v_v_k_mid[i] * w;
      g1_w_w_mid_s = &g1_w_w_mid_s + &ek.g1_w_w_k_mid[i] * w;
      g2_w_w_mid_s = &g2_w_w_mid_s + &ek.g2_w_w_k_mid[i] * w;
      g_y_y_mid_s = &g_y_y_mid_s + &ek.g_y_y_k_mid[i] * w;

      g_v_alpha_v_mid_s = &g_v_alpha_v_mid_s + &ek.g_v_alpha_v_k_mid[i] * w;
      g_w_alpha_w_mid_s = &g_w_alpha_w_mid_s + &ek.g_w_alpha_w_k_mid[i] * w;
      g_y_alpha_y_mid_s = &g_y_alpha_y_mid_s + &ek.g_y_alpha_y_k_mid[i] * w;

      g_beta_vwy_mid_s = &g_beta_vwy_mid_s + &ek.g_vwy_beta_vwy_k_mid[i] * w; 
    }

    let g_h_s = {
      let h = match self.p.divide_by(&self.t) {
        DivResult::Quotient(q) => q,
        _ => panic!("p should be divisible by t"),
      };
      h.eval_with_g2_hidings(&ek.g2_si)
    };

    PinocchioProof {
      g_v_v_mid_s,
      g1_w_w_mid_s,
      g2_w_w_mid_s,
      g_y_y_mid_s,
      g_h_s,
      g_v_alpha_v_mid_s,
      g_w_alpha_w_mid_s,
      g_y_alpha_y_mid_s,
      g_beta_vwy_mid_s,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::zk::w_trusted_setup::pinocchio::pinocchio_verifier::PinocchioVerifier;

  #[test]
  fn test_generate_proof_and_verify() {
    let f = &G1Point::curve_group();

    let expr = "(x * x * x) + x + 5 == 35";
    println!("Expr: {}\n", expr);
    let eq = EquationParser::parse(f, expr).unwrap();

    let witness_map = {
      use crate::zk::w_trusted_setup::pinocchio::term::Term::*;
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
    let prover = &PinocchioProver::new(f, expr, &witness_map);
    let verifier = &PinocchioVerifier::new();
    let crs = CRS::new(f, prover);

    let proof = prover.prove(&crs);
    let result = verifier.verify(
      &proof,
      &crs,
      &prover.witness.io(),
    );

    assert!(result);
  }
}

