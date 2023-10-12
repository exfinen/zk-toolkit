use crate::{
  building_block::field::{prime_field::PrimeField, prime_field_elem::PrimeFieldElem},
  zk::w_trusted_setup::pinocchio::{
    crs::CRS,
    equation_parser::Parser,
    gate::Gate,
    qap::{
      ApplyWitness,
      QAP,
    },
    polynomial::Polynomial,
    pinocchio_proof::PinocchioProof,
    r1cs::R1CS,
    r1cs_tmpl::R1CSTmpl,
    term::Term,
  },
};
use std::collections::HashMap;

pub struct PinocchioProver {
  qap: QAP,
  t: Polynomial,
}

impl PinocchioProver {
  pub fn new(
    f: &PrimeField,
    input: &str,
    witness: &HashMap<Term, PrimeFieldElem>,

  ) -> Self {
    let eq = Parser::parse(f, input).unwrap();

    let gates = &Gate::build(f, &eq);
    let tmpl = &R1CSTmpl::from_gates(f, gates);

    let r1cs = R1CS::from_tmpl(f, tmpl, &witness).unwrap();
    r1cs.validate().unwrap();

    let qap = QAP::build(f, &r1cs, &ApplyWitness::End);
    let t = QAP::build_t(f, &tmpl.constraints.len());

    PinocchioProver { qap, t }
  }

  pub fn prove(_crs: CRS) -> PinocchioProof {
    PinocchioProof()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test1() {
    let input = "x + 2x+ 3";
    let witness = HashMap::new();

    let f = &PrimeField::new(&3911u16);
    let prover = PinocchioProver::new(f, input, witness);
    let crs = CRS::new(
      f,
      &degree,
      &mid_beg,
      &prover.qap.a_polys,
      &prover.qap.b_polys,
      &prover.qap.c_polys,
      &prover.t, 
    );

    let proof = prover.prove(crs);
  }
}











