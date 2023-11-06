// Implementation of protocol 2 described on page 5 in https://eprint.iacr.org/2013/279.pdf

use crate::{
  building_block::{
    curves::bls12_381::{
      g1_point::G1Point,
      g2_point::G2Point,
    },
    field::{
      polynomial::{
        DivResult,
        Polynomial,
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
    pinocchio::{
      crs::CRS,
      pinocchio_proof::PinocchioProof,
      witness::Witness,
    },
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

    let (ek, vk) = (&crs.ek, &crs.vk);
    let delta_v = &self.f.rand_elem(true);
    let delta_y = &self.f.rand_elem(true);

    // adjust v(s) and y(s) for zero-knowledge, w(s) is left untouched
    // due to current constraints in the pairing function implementation.
    // This needs fixing. TODO: Address the issue with w(s) in the pairing
    // function.
    //
    // to achieve zero-knowledge for v(s) and w(s), we add random multiples
    // of t(s) to them. because of this randomization, h(s) is adjusted to
    // h(s) + d_v * w(s) - d_y.
    //
    // randomizing v(s) and y(s) in v(s) * w(s) - y(s), we get:
    // (v(s) + d_v * t(s)) * w - (y(s) + d_y * t(s)) 
    //
    // factoring out t(s), we get the adjusted h(s):
    // = v(s) * w(s)        + d_v * t(s) * w(s) - y(s) - d_y * t(s)
    // = v(s) * w(s) - y(s) + d_v * t(s) * w(s) - d_y * t(s)
    // = t(s) * (h(s) + d_v * w(s) - d_y)

    let mut v_mid_s = &vk.t * delta_v;  // randomize v
    let mut g1_w_mid_s = G1Point::zero();
    let mut g2_w_mid_s = G2Point::zero();
    let mut y_mid_s = &vk.t * delta_y;  // randomize y
    let mut alpha_v_mid_s = &vk.alpha_v_t * delta_v;
    let mut alpha_w_mid_s = G1Point::zero();
    let mut alpha_y_mid_s = &vk.alpha_y_t * delta_y;
    let mut beta_vwy_mid_s = &vk.beta_t * delta_v + &vk.beta_t * delta_y;
  
    for i in 0..witness_mid.size_in_usize() {
      let w = &witness_mid[&i];

      v_mid_s += &ek.vk_mid[i] * w;
      g1_w_mid_s += &ek.g1_wk_mid[i] * w;
      g2_w_mid_s += &ek.g2_wk_mid[i] * w;
      y_mid_s += &ek.yk_mid[i] * w;

      alpha_v_mid_s += &ek.alpha_vk_mid[i] * w;
      alpha_w_mid_s += &ek.alpha_wk_mid[i] * w;
      alpha_y_mid_s += &ek.alpha_yk_mid[i] * w;
      beta_vwy_mid_s += &ek.beta_vwy_k_mid[i] * w; 
    }

    let adj_h_s = {
      let h = match self.p.divide_by(&self.t) {
        DivResult::Quotient(q) => q,
        _ => panic!("p should be divisible by t"),
      };
      let h_s = h.eval_with_g2_hidings(&ek.si);

      let witness_io = &self.witness.io();
      let mut w_s = g2_w_mid_s.clone();
      for i in 0..crs.vk.wk_io.len() {
        w_s += &crs.vk.wk_io[i] * &witness_io[&i];
      }
      h_s + w_s * delta_v + -(&crs.vk.one_g2 * delta_y)
    };

    PinocchioProof {
      v_mid_s,
      g1_w_mid_s,
      g2_w_mid_s,
      y_mid_s,
      h_s: adj_h_s,
      alpha_v_mid_s,
      alpha_w_mid_s,
      alpha_y_mid_s,
      beta_vwy_mid_s,
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

