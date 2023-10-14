use crate::{
  building_block::field::{prime_field::PrimeField, prime_field_elem::PrimeFieldElem},
  zk::w_trusted_setup::pinocchio::{
    crs::CRS,
    equation_parser::EquationParser,
    gate::Gate,
    qap::QAP,
    polynomial::Polynomial,
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

pub struct PinocchioProver {
  pub max_degree: usize,
  pub mid_beg: usize,
  pub vi: Vec<Polynomial>,
  pub wi: Vec<Polynomial>,
  pub yi: Vec<Polynomial>,
  pub t: Polynomial,
}

impl PinocchioProver {
  pub fn new(
    f: &PrimeField,
    expr: &str,
    witness: &HashMap<Term, PrimeFieldElem>,
    mid_beg: &PrimeFieldElem,

  ) -> Self {
    let eq = EquationParser::parse(f, expr).unwrap();

    let gates = &Gate::build(f, &eq);
    let tmpl = &R1CSTmpl::from_gates(f, gates);

    let r1cs = R1CS::from_tmpl(f, tmpl, &witness).unwrap();
    r1cs.validate().unwrap();

    let qap = QAP::build(f, &r1cs );

    let t = QAP::build_t(f, &tmpl.constraints.len());

    let max_degree = {
      let vi = qap.vi.iter().map(|x| x.degree()).max().unwrap();
      let wi = qap.wi.iter().map(|x| x.degree()).max().unwrap();
      let yi = qap.yi.iter().map(|x| x.degree()).max().unwrap();
      cmp::max(cmp::max(vi, wi), yi)
    };

    PinocchioProver {
      max_degree: (&max_degree.e).try_into().unwrap(),
      mid_beg: (&mid_beg.e).try_into().unwrap(),
      vi: qap.vi.clone(),
      wi: qap.wi.clone(),
      yi: qap.yi.clone(),
      t,
    }
  }

  pub fn prove(&self, _crs: CRS) -> PinocchioProof {
    PinocchioProof()
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
        (Term::var("x"), f.elem(&3u8)),
        (TmpVar(1), f.elem(&9u8)),
        (TmpVar(2), f.elem(&27u8)),
        (TmpVar(3), f.elem(&8u8)),
        (TmpVar(4), f.elem(&35u8)),
        (Out, eq.rhs),
      ])
    };
    let mid_beg = f.elem(&1u8);

    let prover = &PinocchioProver::new(f, expr, &witness, &mid_beg);

    let crs = CRS::new(f, prover);

    let _proof = prover.prove(crs);
  }
}











