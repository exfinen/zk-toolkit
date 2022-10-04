use crate::building_block::field::{Field, FieldElem};
use crate::snarks::constraint::Constraint;
use crate::snarks::sparse_vec::SparseVec;
use crate::snarks::r1cs_tmpl::{Term, R1CSTmpl};
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
              return Err(format!("Witness for {:?} is missing", term));
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

  pub fn new(f: Field, tmpl: &R1CSTmpl, var_assignments: &HashMap<Term, FieldElem>) -> Result<R1CS, String> {
    let solution_vec = R1CS::build_solution_vec(&f, tmpl, var_assignments)?;
    let r1cs = R1CS {
      constraints: tmpl.constraints.clone(),
      solution_vec,
    };
    Ok(r1cs)
  }
}