use crate::{
  building_block::{
    curves::bls12_381::g1_point::G1Point,
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
    witness::Witness,
  },
};
use std::{
  collections::HashMap,
  cmp,
};

pub struct PinocchioProver {
  pub f: PrimeField,
  pub max_degree: usize,  // TODO use PrimeFieldElem or BigUint
  pub mid_beg: usize,
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

    let witness = Witness::new(&r1cs.witness.clone(), &tmpl.mid_beg);

    let num_constraints = tmpl.constraints.len();

    PinocchioProver {
      f: f.clone(),
      max_degree: (&max_degree.e).try_into().unwrap(),
      mid_beg: (&tmpl.mid_beg.e).try_into().unwrap(),
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
    let witness_mid = &self.witness.mid();

    let calculate = |points: &Vec<G1Point>| {
      let mut sum = G1Point::zero();
      for i in 0..points.len() {
        sum = sum + &(&points[i] * &witness_mid[&self.f.elem(&i)]);
      }
      sum
    };

    let v = calculate(&crs.h_vi_mid);
    let beta_v = calculate(&crs.h_beta_vi_mid);

    let w = calculate(&crs.h_wi_mid);
    let beta_w = calculate(&crs.h_beta_wi_mid);

    let y = calculate(&crs.h_yi_mid);
    let beta_y = calculate(&crs.h_beta_yi_mid);

    let h = match self.p.divide_by(&self.t) {
      DivResult::Quotient(h) => h,
      DivResult::QuotientRemainder(_) => panic!("p must be divisible by t"),
    };

    let h_hiding = h.eval_with_g1_hidings(&crs.h_si);
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
  use super::*;

  #[test]
  fn test1() {
    let f = &PrimeField::new(&3911u16);

    let expr = "(x * x * x) + x + 5 == 35";
    let eq = EquationParser::parse(f, expr).unwrap();

    /*
      x = 3
      t1 = x(3) * x(3) = 9
      t2 = t1(9) * x(3) = 27
      t3 = x(3) + 5 = 8
      t4 = t2(27) + t2(8) = 35
      out = t4
    */
    let witness = {
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
    let prover = &PinocchioProver::new(f, expr, &witness);

    let crs = CRS::new(f, prover);

    let _proof = prover.prove(&crs);
  }
}











