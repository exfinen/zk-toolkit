use crate::building_block::field::Field;
use crate::snarks::{
  term::Term,
  gate::Gate,
  constraint::Constraint,
};
use std::collections::HashMap;

use super::sparse_vec::SparseVec;

pub struct R1CSTmpl<'a> {
  pub f: &'a Field,
  pub constraints: Vec<Constraint>,
  pub witness: Vec<Term>,
  pub indices: HashMap<Term, usize>,
}

impl<'a> R1CSTmpl<'a> {
  fn new(f: &'a Field) -> Self {
    let mut tmpl = R1CSTmpl {
      f,
      constraints: vec![],
      witness: vec![],
      indices: HashMap::<Term, usize>::new(),
    };
    tmpl.add_term(&Term::One);
    tmpl
  }

  fn add_term(&mut self, t: &Term) {
    match t {
      Term::Sum(a, b) => {
        self.add_term(&a);
        self.add_term(&b);
      },
      Term::Num(_) => {},  // num should not be included to witness
      t => {
        if self.indices.contains_key(t) { return };
        self.witness.push(t.clone());
        self.indices.insert(t.clone(), self.indices.len());
      }
    }
  }

  fn gate_to_vec(&mut self, f: &Field, vec: &mut SparseVec, term: &Term) {
    match term {
      Term::Sum(a, b) => {
        self.gate_to_vec(f, vec, &a);
        self.gate_to_vec(f, vec, &b);
      },
      Term::Num(n) => {
        vec.set(0, n.clone());  // Num is represented by multiplying the number by 1
      },
      x => {
        let index = self.indices.get(&x).unwrap();
        vec.set(*index, f.elem(&1u8));
      },
    }
  }

  pub fn from_gates(f: &'a Field, gates: &[Gate]) -> Self {
    let mut tmpl = R1CSTmpl::new(f);

    // build witness vector
    for gate in gates {
      tmpl.add_term(&gate.a);
      tmpl.add_term(&gate.b);
      tmpl.add_term(&gate.c);
    }

    let vec_size = tmpl.witness.len();

    // create a, b anc c vectors for each gate
    for gate in gates {
      let mut a = SparseVec::new(vec_size);
      tmpl.gate_to_vec(f, &mut a, &gate.a);

      let mut b = SparseVec::new(vec_size);
      tmpl.gate_to_vec(f, &mut b, &gate.b);

      let mut c = SparseVec::new(vec_size);
      tmpl.gate_to_vec(f, &mut c, &gate.c);

      let constraint = Constraint { a, b, c };
      tmpl.constraints.push(constraint)
    }
    tmpl
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::snarks::equation_parser::Parser;

  #[test]
  fn test_get_to_vec() {
    let f = &Field::new(&3911u16);
    let terms = vec![
      Term::Out,
      Term::Var("x".to_string()),
      Term::TmpVar(2),
    ];
    for term in terms {
      let mut tmpl = R1CSTmpl::new(f);
      let mut sv = SparseVec::new(2);
      tmpl.add_term(&term);
      tmpl.gate_to_vec(f, &mut sv, &term);
      let indices = sv.indices().to_vec();

      // should be stored at index 1 in witness vector
      assert_eq!(indices[0], 1);
      // and the multiplier should be 1
      assert_eq!(sv.get(&1), f.elem(&1u8));
    }
    {
      // test Num term
      let mut tmpl = R1CSTmpl::new(f);
      let mut sv = SparseVec::new(1);
      let n = f.elem(&4u8);
      let term = Term::Num(n.clone());
      tmpl.gate_to_vec(f, &mut sv, &term);
      let indices = sv.indices().to_vec();

      // term should map to index 0 of witness that stores One term
      assert_eq!(indices[0], 0);
      assert_eq!(sv.get(&0), n);
    }
    {
      // test Sum term
      let mut tmpl = R1CSTmpl::new(f);
      let mut sv = SparseVec::new(3);
      let y = Term::Var("y".to_string());
      let z = Term::Var("z".to_string());
      let term = Term::Sum(Box::new(y.clone()), Box::new(z.clone()));
      tmpl.add_term(&term);
      tmpl.gate_to_vec(f, &mut sv, &term);
      let mut indices = sv.indices().to_vec();
      indices.sort();

      // y and z should be stored at index 1 and 2 of witness vector respectively
      assert_eq!(indices[0], 1);
      assert_eq!(indices[1], 2);

      // and both of the multipliers should be 1
      assert_eq!(sv.get(&1), f.elem(&1u8));
      assert_eq!(sv.get(&2), f.elem(&1u8));
    }
  }

  #[test]
  fn test_add_term() {
    let f = &Field::new(&3911u16);
    let mut tmpl = R1CSTmpl::new(f);
    assert_eq!(tmpl.indices.len(), 1);

    // initially witness contains only One term
    assert_eq!(tmpl.indices.get(&Term::One).unwrap(), &0);
    assert_eq!(tmpl.witness.len(), 1);
    assert_eq!(tmpl.witness[0], Term::One);

    // Num term should not be added to witness
    tmpl.add_term(&Term::Num(f.elem(&9u8)));
    assert_eq!(tmpl.indices.len(), 1);
    assert_eq!(tmpl.witness.len(), 1);

    // One term should not be added twice
    tmpl.add_term(&Term::One);
    assert_eq!(tmpl.indices.len(), 1);
    assert_eq!(tmpl.witness.len(), 1);

    // Var term should be added
    let x = Term::Var("x".to_string());
    tmpl.add_term(&x);
    assert_eq!(tmpl.witness.len(), 2);
    assert_eq!(tmpl.indices.get(&x).unwrap(), &1);
    assert_eq!(tmpl.witness[1], x);

    // same Var term should not be added twice
    tmpl.add_term(&x);
    assert_eq!(tmpl.indices.len(), 2);
    assert_eq!(tmpl.witness.len(), 2);

    // TmpVar term should be added
    let x = Term::TmpVar(1);
    tmpl.add_term(&x);
    assert_eq!(tmpl.indices.len(), 3);
    assert_eq!(tmpl.indices.get(&x).unwrap(), &2);
    assert_eq!(tmpl.witness.len(), 3);
    assert_eq!(tmpl.witness[2], x);

    // same TmpVar term should not be added twice
    tmpl.add_term(&x);
    assert_eq!(tmpl.indices.len(), 3);
    assert_eq!(tmpl.witness.len(), 3);

    // Out term should be added
    let x = Term::Out;
    tmpl.add_term(&x);
    assert_eq!(tmpl.indices.len(), 4);
    assert_eq!(tmpl.indices.get(&x).unwrap(), &3);
    assert_eq!(tmpl.witness.len(), 4);
    assert_eq!(tmpl.witness[3], x);

    // Out term should not be added twice
    tmpl.add_term(&x);
    assert_eq!(tmpl.indices.len(), 4);
    assert_eq!(tmpl.witness.len(), 4);

    // Sum term should be added
    let y = Term::Var("y".to_string());
    let z = Term::Var("z".to_string());
    let sum = Term::Sum(Box::new(y.clone()), Box::new(z.clone()));
    tmpl.add_term(&sum);
    assert_eq!(tmpl.indices.len(), 6);
    assert_eq!(tmpl.indices.get(&y).unwrap(), &4);
    assert_eq!(tmpl.indices.get(&z).unwrap(), &5);
    assert_eq!(tmpl.witness.len(), 6);
    assert_eq!(tmpl.witness[4], y);
    assert_eq!(tmpl.witness[5], z);
  }

  #[test]
  fn test_build_from_gates() {
    let f = &Field::new(&3911u16);
    let input = "(3 * x + 4) / 2 == 11";
    let eq = Parser::parse(f, input).unwrap();

    let gates = &Gate::build(f, &eq);
    for g in gates {
      println!("{:?}", g);
    }
  }

  #[test]
  fn test_bulding_witness() {
    let f = &Field::new(&3911u16);
    let input = "(3 * x + 4) / 2 == 11";
    let eq = Parser::parse(f, input).unwrap();

    let gates = &Gate::build(f, &eq);
    let r1cs = R1CSTmpl::from_gates(f, gates);

    let h = r1cs.indices;
    let w = [
      Term::One,
      Term::Var("x".to_string()),
      Term::TmpVar(1),
      Term::TmpVar(2),
      Term::TmpVar(3),
      Term::Out,
    ];
    assert_eq!(h.len(), w.len());

    for (i, term) in w.iter().enumerate() {
      assert_eq!(h.get(&term).unwrap(), &i);
    }
  }

  fn term_to_str(tmpl: &R1CSTmpl, vec: &SparseVec) -> String {
    let mut indices = vec.indices().to_vec();
    indices.sort();  // sort to make indices order deterministic
    let s = indices.iter().map(|i| {
      match &tmpl.witness[*i] {
        Term::Var(s) => s.clone(),
        Term::TmpVar(i) => format!("t{}", i),
        Term::One => format!("{:?}", &vec.get(i).n),
        Term::Out => "out".to_string(),
        // not handling Term::Sum since it's not used in tests
        _ => "?".to_string(),
      }
    }).collect::<Vec<String>>().join(" + ");
    format!("{}", s)
  }

  #[test]
  fn test_r1cs_build_a_b_c_matrix() {
    let f = &Field::new(&3911u16);
    let input = "3 * x + 4 == 11";
    let eq = Parser::parse(f, input).unwrap();

    let gates = &Gate::build(f, &eq);
    let tmpl = R1CSTmpl::from_gates(f, gates);

    let mut res = vec![];
    for constraint in &tmpl.constraints {
      let a = term_to_str(&tmpl, &constraint.a);
      let b = term_to_str(&tmpl, &constraint.b);
      let c = term_to_str(&tmpl, &constraint.c);
      res.push((a, b, c));
    }

    assert_eq!(res.len(), 3);
    assert_eq!(res[0], ("3".to_string(), "x".to_string(), "t1".to_string()));
    assert_eq!(res[1], ("4 + t1".to_string(), "1".to_string(), "t2".to_string()));
    assert_eq!(res[2], ("t2".to_string(), "1".to_string(), "out".to_string()));
  }
 }