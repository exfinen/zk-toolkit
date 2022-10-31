use crate::building_block::field::{Field, FieldElem};
use crate::snarks::{
  constraint::Constraint,
  r1cs_tmpl::R1CSTmpl,
  sparse_vec::SparseVec,
  term::Term,
};
use std::collections::HashMap;

pub struct R1CS {
  pub constraints: Vec<Constraint>,
  pub witness: SparseVec,
}

impl R1CS {
  fn build_witness_vec(
    f: &Field,
    tmpl: &R1CSTmpl,
    term_values: &HashMap<Term, FieldElem>,
  ) -> Result<SparseVec, String> {
    // generate SparseVec from the witness
    let mut witness = SparseVec::new(f, &tmpl.witness.len());

    let add = |i: &usize, term: &Term, witness: &mut SparseVec| -> Result<(), String> {
      match term_values.get(term) {
        Some(v) => {
          witness.set(i, v);
          Ok(())
        },
        None => Err(format!("Witness for '{:?}' is missing", term)),
      }
    };

    for (i, term) in tmpl.witness.iter().enumerate() {
      match term {
        Term::One => {
          witness.set(&i, &1u8);
        },
        Term::Sum(_a, _b) => { assert!(false, "Sum shouldn't have been included"); }
        Term::Num(n) => {
          witness.set(&i, n);
        },
        Term::Var(_) => if let Err(err) = add(&i, term, &mut witness) { return Err(err) },
        Term::TmpVar(_) => if let Err(err) = add(&i, term, &mut witness) { return Err(err) },
        Term::Out => if let Err(err) = add(&i, term, &mut witness) { return Err(err) },
      }
    }
    Ok(witness)
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

  pub fn from_tmpl(f: &Field, tmpl: &R1CSTmpl, witness: &HashMap<Term, FieldElem>) -> Result<R1CS, String> {
    let witness = R1CS::build_witness_vec(&f, tmpl, witness)?;
    let r1cs = R1CS {
      constraints: tmpl.constraints.clone(),
      witness,
    };
    Ok(r1cs)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::snarks::{
    equation_parser::Parser,
    gate::Gate,
    r1cs_tmpl::R1CSTmpl,
    term::Term,
  };

  #[test]
  fn test_validate() {
    let f = &Field::new(&3911u16);
    let input = "x + 4 * y == 11";
    let eq = Parser::parse(f, input).unwrap();

    let gates = &Gate::build(f, &eq);
    let tmpl = &R1CSTmpl::from_gates(f, gates);

    let witness = HashMap::<Term, FieldElem>::from([
      (Term::Var("x".to_string()), f.elem(&3u8)),
      (Term::Var("y".to_string()), f.elem(&2u8)),
      (Term::TmpVar(1), f.elem(&8u8)),
      (Term::TmpVar(2), f.elem(&11u8)),
      (Term::Out, eq.rhs),
    ]);
    let r1cs = R1CS::from_tmpl(f, tmpl, &witness).unwrap();
    r1cs.validate().unwrap();
  }
}