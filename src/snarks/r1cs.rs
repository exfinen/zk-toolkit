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
  pub solution_vec: SparseVec,
}

impl R1CS {
  fn build_solution_vec(
    f: &Field,
    tmpl: &R1CSTmpl,
    var_assignments: &HashMap<Term, FieldElem>,
  ) -> Result<SparseVec, String> {
    // generate SparseVec from the witness
    let mut solution_vec = SparseVec::new(tmpl.witness.len());

    for (i, term) in tmpl.witness.iter().enumerate() {
      match term {
        Term::One => {
          solution_vec.set(i, f.elem(&1u8));
        },
        Term::Sum(_a, _b) => { assert!(false, "Sum should not be included"); }
        term => {
          match var_assignments.get(term) {
            Some(v) => solution_vec.set(i, v.clone()),
            None => {
              return Err(format!("Witness for '{:?}' is missing", term));
            }
          }
        },
      }
    }
    Ok(solution_vec)
  }

  // evaluate all constraints and confirm they all hold
  pub fn validate(&self) -> Result<(), String> {
    for constraint in &self.constraints {
      let a = &(&constraint.a * &self.solution_vec).sum();
      let b = &(&constraint.b * &self.solution_vec).sum();
      let c = &(&constraint.c * &self.solution_vec).sum();

      if &(a * b) != c {
        return Err(format!("Constraint a ({:?}) * b ({:?}) = c ({:?}) doesn't hold", a, b, c));
      }
    }
    Ok(())
  }

  pub fn new(f: &Field, tmpl: &R1CSTmpl, var_assignments: &HashMap<Term, FieldElem>) -> Result<R1CS, String> {
    let solution_vec = R1CS::build_solution_vec(&f, tmpl, var_assignments)?;
    let r1cs = R1CS {
      constraints: tmpl.constraints.clone(),
      solution_vec,
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
    for gate in gates {
      println!("{:?}", gate);
    }
    let tmpl = &R1CSTmpl::from_gates(f, gates);

    let mut vars = HashMap::<Term, FieldElem>::new();
    vars.insert(Term::Var("x".to_string()), f.elem(&3u8));
    vars.insert(Term::Var("y".to_string()), f.elem(&2u8));
    vars.insert(Term::TmpVar(1), f.elem(&8u8));
    vars.insert(Term::TmpVar(2), f.elem(&11u8));
    vars.insert(Term::Out, eq.rhs);

    let r1cs = R1CS::new(f, tmpl, &vars).unwrap();
    r1cs.validate().unwrap();
  }
}