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
    polynomial::{Polynomial, DivResult},
    pinocchio_proof::PinocchioProof,
    r1cs::R1CS,
    r1cs_tmpl::R1CSTmpl,
    term::Term,
  },
};
use std::{
  collections::HashMap,
  cmp,
};

use super::sparse_vec::SparseVec;

pub struct PinocchioProver {
  pub f: PrimeField,
  pub max_degree: usize,  // TODO use PrimeFieldElem or BigUint
  pub mid_beg: usize,
  pub witness: SparseVec,
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
    witness: &HashMap<Term, PrimeFieldElem>,
  ) -> Self {
    let eq = EquationParser::parse(f, expr).unwrap();

    let gates = &Gate::build(f, &eq);
    let tmpl = &R1CSTmpl::from_gates(f, gates);

    let r1cs = R1CS::from_tmpl(f, tmpl, &witness).unwrap();
    r1cs.validate().unwrap();

    let qap = QAP::build(f, &r1cs);

    let t = QAP::build_t(f, &tmpl.constraints.len());
    let p = qap.build_p(&r1cs.witness);

    let max_degree = {
      let vi = qap.vi.iter().map(|x| x.degree()).max().unwrap();
      let wi = qap.wi.iter().map(|x| x.degree()).max().unwrap();
      let yi = qap.yi.iter().map(|x| x.degree()).max().unwrap();
      cmp::max(cmp::max(vi, wi), yi)
    };

    PinocchioProver {
      f: f.clone(),
      max_degree: (&max_degree.e).try_into().unwrap(),
      mid_beg: (&tmpl.mid_beg.e).try_into().unwrap(),
      witness: r1cs.witness.clone(),
      p,
      t,
      vi: qap.vi.clone(),
      wi: qap.wi.clone(),
      yi: qap.yi.clone(),
    }
  }

  fn evaluate_polys_in_g1(
    &self,
    ps: &Vec<Polynomial>,
    pows: &Vec<G1Point>,  // x^0, x^1, ...
    witness: &SparseVec,
  ) -> G1Point {
    let mut sum = G1Point::zero();

    for i in 0..ps.len() {
      let p = ps[i].eval_with_g1_hidings(pows);
      let w = &witness[&self.f.elem(&i)];
      sum = sum + (p * w);
    };
    sum
  }

  // TODO share the code with evaluate_polys_in_g1
  #[allow(dead_code)]
  fn evaluate_polys_in_g2(
    &self,
    ps: &Vec<Polynomial>,
    pows: &Vec<G2Point>,  // x^0, x^1, ...
    witness: &SparseVec,
  ) -> G2Point {
    let mut sum = G2Point::zero();

    for i in 0..ps.len() {
      let p = ps[i].eval_with_g2_hidings(pows);
      let w = &witness[&self.f.elem(&i)];
      sum = sum + (p * w);
    };
    sum
  }

  pub fn prove(&self, crs: &CRS) -> PinocchioProof {
println!("1");
    let v = self.evaluate_polys_in_g1(&self.vi, &crs.h_vi_mid, &self.witness);
    let w = self.evaluate_polys_in_g1(&self.wi, &crs.h_wi_mid, &self.witness);
    let y = self.evaluate_polys_in_g1(&self.yi, &crs.h_yi_mid, &self.witness);

println!("2");
    let beta_v = self.evaluate_polys_in_g1(&self.vi, &crs.h_beta_vi_mid, &self.witness);
    let beta_w = self.evaluate_polys_in_g1(&self.wi, &crs.h_beta_wi_mid, &self.witness);
    let beta_y = self.evaluate_polys_in_g1(&self.yi, &crs.h_beta_yi_mid, &self.witness);

println!("3");
    let h = match self.p.divide_by(&self.t) {
      DivResult::Quotient(h) => h,
      DivResult::QuotientRemainder(_) => panic!("p must be divisible by t"),
    };
println!("4");
    let h_hiding = h.eval_with_g1_hidings(&crs.h_si);
println!("5");
    let h_alpha = h.eval_with_g1_hidings(&crs.h_alpha_si);

    PinocchioProof {
      v,
      w,
      y,
      beta_v,
      beta_w,
      beta_y,
      h: h_hiding,
      h_alpha,
    }
  }
}

#[cfg(test)]
mod tests {
  //use super::*;

  #[test]
  fn test1() {
    // let f = &PrimeField::new(&3911u16);
    // 
    // let expr = "(x * x * x) + x + 5 == 35";
    // let eq = EquationParser::parse(f, expr).unwrap();
    // 
    // /*
    //   x = 3
    //   t1 = x(3) * x(3) = 9
    //   t2 = t1(9) * x(3) = 27
    //   t3 = x(3) + 5 = 8
    //   t4 = t2(27) + t2(8) = 35
    //   out = t4
    // */
    // let witness = {
    //   use crate::zk::w_trusted_setup::pinocchio::term::Term::*;
    //   HashMap::<Term, PrimeFieldElem>::from([
    //     (Term::One, f.elem(&1u8)),
    //     (Term::var("x"), f.elem(&3u8)),
    //     (TmpVar(1), f.elem(&9u8)),
    //     (TmpVar(2), f.elem(&27u8)),
    //     (TmpVar(3), f.elem(&8u8)),
    //     (TmpVar(4), f.elem(&35u8)),
    //     (Out, eq.rhs),
    //   ])
    // };
    // let prover = &PinocchioProver::new(f, expr, &witness);
    // 
    // let crs = CRS::new(f, prover);
    // 
    // let _proof = prover.prove(&crs);
  }
}











