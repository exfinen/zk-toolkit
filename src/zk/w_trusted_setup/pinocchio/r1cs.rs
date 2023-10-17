use crate::building_block::field::{
  prime_field::PrimeField,
  prime_field_elem::PrimeFieldElem,
};
use crate::zk::w_trusted_setup::pinocchio::{
  constraint::Constraint,
  r1cs_tmpl::R1CSTmpl,
  sparse_vec::SparseVec,
  sparse_matrix::SparseMatrix,
  term::Term,
};
use std::collections::HashMap;

#[derive(Clone)]
pub struct R1CS {
  pub constraints: Vec<Constraint>,
  pub witness: SparseVec,
  pub mid_beg: PrimeFieldElem,
}

// matrix representing a constraint whose
// row is the multiples of each witness value i.e.
// a  = [a1, a2, ...]
// *
// b  = [b1, b2, ...]
// ||
// c  = [c1, c2, ...]
pub struct ConstraintMatrices {
  pub a: SparseMatrix,
  pub b: SparseMatrix,
  pub c: SparseMatrix,
}

impl R1CS {
  // build the witness vector that is in the order expected by the prover
  // while validating the witness vector with the witness_instance
  fn build_witness_vec(
    f: &PrimeField,
    tmpl: &R1CSTmpl,
    witness_instance: &HashMap<Term, PrimeFieldElem>,
  ) -> Result<(SparseVec, PrimeFieldElem), String> {

    // build witness vector with values assigned
    let mut witness = SparseVec::new(f, &tmpl.witness.len());

    let mut i = f.elem(&0u8);

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

      if &(a * b) != c {
        return Err(format!("Constraint a ({:?}) * b ({:?}) = c ({:?}) doesn't hold", a, b, c));
      }
    }
    Ok(())
  }

  pub fn from_tmpl(
    f: &PrimeField,
    tmpl: &R1CSTmpl,
    witness: &HashMap<Term, PrimeFieldElem>,
  ) -> Result<R1CS, String> {
    let (witness, mid_beg) = R1CS::build_witness_vec(&f, tmpl, witness)?;
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
      a: SparseMatrix::from(&a),
      b: SparseMatrix::from(&b),
      c: SparseMatrix::from(&c),
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
      a: SparseMatrix::from(&a),
      b: SparseMatrix::from(&b),
      c: SparseMatrix::from(&c),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::zk::w_trusted_setup::pinocchio::{
    equation_parser::EquationParser,
    gate::Gate,
    r1cs_tmpl::R1CSTmpl,
    term::Term,
  };

  #[test]
  fn test_validate() {
    let f = &PrimeField::new(&3911u16);
    let input = "(x + 2) + 4 * y == 21";
    let eq = EquationParser::parse(f, input).unwrap();

    let gates = &Gate::build(f, &eq);
    let tmpl = &R1CSTmpl::from_gates(f, gates);

    let witness = HashMap::<Term, PrimeFieldElem>::from([
      (Term::One, f.elem(&1u8)),
      (Term::Var("x".to_string()), f.elem(&3u8)),
      (Term::Var("y".to_string()), f.elem(&4u8)),
      (Term::Out, eq.rhs),
      (Term::TmpVar(1), f.elem(&5u8)),
      (Term::TmpVar(2), f.elem(&16u8)),
      (Term::TmpVar(3), f.elem(&21u8)),
    ]);
    let r1cs = R1CS::from_tmpl(f, tmpl, &witness).unwrap();
    r1cs.validate().unwrap();
  }
}
