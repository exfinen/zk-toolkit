use crate::building_block::mcl::{
  mcl_fr::MclFr,
  mcl_sparse_vec::MclSparseVec,
  mcl_sparse_matrix::MclSparseMatrix,
  qap::{
    constraint::Constraint,
    r1cs_tmpl::R1CSTmpl,
    term::Term,
  },
};
use std::collections::HashMap;
use num_traits::Zero;

#[derive(Clone)]
pub struct R1CS {
  pub constraints: Vec<Constraint>,
  pub witness: MclSparseVec,
  pub mid_beg: MclFr,
}

// matrix representing a constraint whose
// row is the multiples of each witness value i.e.
// a  = [a1, a2, ...]
// *
// b  = [b1, b2, ...]
// ||
// c  = [c1, c2, ...]
#[derive(Debug)]
pub struct ConstraintMatrices {
  pub a: MclSparseMatrix,
  pub b: MclSparseMatrix,
  pub c: MclSparseMatrix,
}

impl R1CS {
  // build the witness vector that is in the order expected by the prover
  // while validating the witness vector with the witness_instance
  fn build_witness_vec(
    tmpl: &R1CSTmpl,
    witness_instance: &HashMap<Term, MclFr>,
  ) -> Result<(MclSparseVec, MclFr), String> {

    // build witness vector with values assigned
    let mut witness = MclSparseVec::new(&MclFr::from(tmpl.witness.len()));

    let mut i = MclFr::zero();

    for term in tmpl.witness.iter() {
      if !witness_instance.contains_key(&term) {
        return Err(format!("'{:?}' is missing in witness_instance", term));
      }
      witness[&i] = witness_instance.get(&term).unwrap().clone();
      i.inc();
    }

    Ok((witness, tmpl.mid_beg.clone()))
  }

  // evaluate all constraints and confirm they all hold
  pub fn validate(&self) -> Result<(), String> {
    for constraint in &self.constraints {
      let a = &(&constraint.a * &self.witness).sum();
      let b = &(&constraint.b * &self.witness).sum();
      let c = &(&constraint.c * &self.witness).sum();

      println!("r1cs: {:?} * {:?} = {:?}", &a, &b, &c);
      if &(a * b) != c {
        return Err(format!("Constraint a ({:?}) * b ({:?}) = c ({:?}) doesn't hold", a, b, c));
      }
    }
    println!("");
    Ok(())
  }

  pub fn from_tmpl(
    tmpl: &R1CSTmpl,
    witness: &HashMap<Term, MclFr>,
  ) -> Result<R1CS, String> {
    let (witness, mid_beg) = R1CS::build_witness_vec(tmpl, witness)?;
    let r1cs = R1CS {
      constraints: tmpl.constraints.clone(),
      witness,
      mid_beg,
    };
    Ok(r1cs)
  }

  pub fn to_constraint_by_witness_matrices(&self) -> ConstraintMatrices {
    let mut a = vec![];
    let mut b = vec![];
    let mut c = vec![];

    for constraint in &self.constraints {
      a.push(&constraint.a * &self.witness);
      b.push(&constraint.b * &self.witness);
      c.push(&constraint.c * &self.witness);
    }

    ConstraintMatrices {
      a: MclSparseMatrix::from(&a),
      b: MclSparseMatrix::from(&b),
      c: MclSparseMatrix::from(&c),
    }
  }

  pub fn to_constraint_matrices(&self) -> ConstraintMatrices {
    let mut a = vec![];
    let mut b = vec![];
    let mut c = vec![];

    for constraint in &self.constraints {
      a.push(constraint.a.clone());
      b.push(constraint.b.clone());
      c.push(constraint.c.clone());
    }

    ConstraintMatrices {
      a: MclSparseMatrix::from(&a),
      b: MclSparseMatrix::from(&b),
      c: MclSparseMatrix::from(&c),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::building_block::mcl::{
    mcl_initializer::MclInitializer,
    qap::{
      equation_parser::EquationParser,
      gate::Gate,
      r1cs_tmpl::R1CSTmpl,
      term::Term,
    },
  };

  #[test]
  fn test_validate() {
    MclInitializer::init();
    let input = "(x + 2) + 4 * y == 21";
    let eq = EquationParser::parse(input).unwrap();

    let gates = &Gate::build(&eq);
    let tmpl = &R1CSTmpl::new(gates);

    let witness = HashMap::<Term, MclFr>::from([
      (Term::One, MclFr::from(1)),
      (Term::Var("x".to_string()), MclFr::from(3)),
      (Term::Var("y".to_string()), MclFr::from(4)),
      (Term::Out, eq.rhs),
      (Term::TmpVar(1), MclFr::from(5)),
      (Term::TmpVar(2), MclFr::from(16)),
      (Term::TmpVar(3), MclFr::from(21)),
    ]);
    let r1cs = R1CS::from_tmpl(tmpl, &witness).unwrap();
    r1cs.validate().unwrap();
  }
}
