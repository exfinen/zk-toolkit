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
      t => {
        if self.indices.contains_key(t) { return };
        self.witness.push(t.clone());
        self.indices.insert(t.clone(), self.indices.len());
      }
    }
  }

  fn gate_to_vec(tmpl: &R1CSTmpl, f: &Field, vec: &mut SparseVec, term: &Term) {
    match term {
      Term::Sum(a, b) => {
        R1CSTmpl::gate_to_vec(tmpl, f, vec, &a);
        R1CSTmpl::gate_to_vec(tmpl, f, vec, &b);
      },
      x => {
        let index = tmpl.indices.get(&x).unwrap();
        match &x {
          Term::Num(n) => {
            vec.set(*index, n.clone());
          },
          _ => {
            vec.set(*index, f.elem(&1u8));
          },
        };
      },
    }
  }

  pub fn from_gates(f: &'a Field, gates: &[Gate]) -> R1CSTmpl<'a> {
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
      R1CSTmpl::gate_to_vec(&tmpl, f, &mut a, &gate.a);

      let mut b = SparseVec::new(vec_size);
      R1CSTmpl::gate_to_vec(&tmpl, f, &mut b, &gate.b);

      let mut c = SparseVec::new(vec_size);
      R1CSTmpl::gate_to_vec(&tmpl, f, &mut c, &gate.c);

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
  fn test_gate_build_add() {
    let f = &Field::new(&3911u16);
    let input = "x + 4 == 9";
    let eq = Parser::parse(f, input).unwrap();
    let gates = &Gate::build(f, &eq);
    assert_eq!(gates.len(), 2);

    // t1 = (x + 4) * 1
    assert_eq!(gates[0].a, Term::Sum(Box::new(Term::Var("x".to_string())), Box::new(Term::Num(f.elem(&4u8)))));
    assert_eq!(gates[0].b, Term::One);
    assert_eq!(gates[0].c, Term::TmpVar(1));

    // out = t1 * 1
    assert_eq!(gates[1].a, Term::TmpVar(1));
    assert_eq!(gates[1].b, Term::One);
    assert_eq!(gates[1].c, Term::Out);
  }

  #[test]
  fn test_gate_build_sub() {
    let f = &Field::new(&3911u16);
    let input = "x - 4 == 9";
    let eq = Parser::parse(f, input).unwrap();
    let gates = &Gate::build(f, &eq);
    assert_eq!(gates.len(), 2);

    // t1 = (x + 4) * 1
    assert_eq!(gates[0].a, Term::Sum(Box::new(Term::Num(f.elem(&4u8))), Box::new(Term::TmpVar(1))));
    assert_eq!(gates[0].b, Term::One);
    assert_eq!(gates[0].c, Term::Var("x".to_string()));

    // out = t1 * 1
    assert_eq!(gates[1].a, Term::TmpVar(1));
    assert_eq!(gates[1].b, Term::One);
    assert_eq!(gates[1].c, Term::Out);
  }

  #[test]
  fn test_gate_build_mul() {
    let f = &Field::new(&3911u16);
    let input = "x * 4 == 9";
    let eq = Parser::parse(f, input).unwrap();
    let gates = &Gate::build(f, &eq);
    assert_eq!(gates.len(), 2);

    // x = (4 + t1) * 1
    assert_eq!(gates[0].a, Term::Var("x".to_string()));
    assert_eq!(gates[0].b, Term::Num(f.elem(&4u8)));
    assert_eq!(gates[0].c, Term::TmpVar(1));

    // out = t1 * 1
    assert_eq!(gates[1].a, Term::TmpVar(1));
    assert_eq!(gates[1].c, Term::Out);
  }

  #[test]
  fn test_gate_build_div() {
    let f = &Field::new(&3911u16);
    let input = "x / 4 == 2";
    let eq = Parser::parse(f, input).unwrap();
    let gates = &Gate::build(f, &eq);
    assert_eq!(gates.len(), 2);

    // x = 4 * t1
    assert_eq!(gates[0].a, Term::Num(f.elem(&4u8)));
    assert_eq!(gates[0].b, Term::TmpVar(1));
    assert_eq!(gates[0].c, Term::Var("x".to_string()));

    // out = t1 * 1
    assert_eq!(gates[1].a, Term::TmpVar(1));
    assert_eq!(gates[1].c, Term::Out);
  }

  #[test]
  fn test_gate_build_combined() {
    let f = &Field::new(&3911u16);
    let input = "(3 * x + 4) / 2 == 11";
    println!("Equation: {}", input);
    let eq = Parser::parse(f, input).unwrap();
    let gates = &Gate::build(f, &eq);
    assert_eq!(gates.len(), 4);

    // t1 = 3 * x
    assert_eq!(gates[0].a, Term::Num(f.elem(&3u8)));
    assert_eq!(gates[0].b, Term::Var("x".to_string()));
    assert_eq!(gates[0].c, Term::TmpVar(1));

    // t2 = (t1 + 4) * 1
    assert_eq!(gates[1].a, Term::Sum(Box::new(Term::TmpVar(1)), Box::new(Term::Num(f.elem(&4u8)))));
    assert_eq!(gates[1].b, Term::One);
    assert_eq!(gates[1].c, Term::TmpVar(2));

    // t2 = 2 * t3
    assert_eq!(gates[2].a, Term::Num(f.elem(&2u8)));
    assert_eq!(gates[2].b, Term::TmpVar(3));
    assert_eq!(gates[2].c, Term::TmpVar(2));

    // out = t3 * 1
    assert_eq!(gates[3].a, Term::TmpVar(3));
    assert_eq!(gates[3].b, Term::One);
    assert_eq!(gates[3].c, Term::Out);
  }

  #[test]
  fn test_r1cs_from_gates() {
    let f = &Field::new(&3911u16);
    let input = "(3 * x + 4) / 2 == 11";
    let eq = Parser::parse(f, input).unwrap();

    let gates = &Gate::build(f, &eq);
    for g in gates {
      println!("{:?}", g);
    }
  }

  #[test]
  fn test_r1cs_build_template() {
    let f = &Field::new(&3911u16);
    let input = "(3 * x + 4) / 2 == 11";
    let eq = Parser::parse(f, input).unwrap();

    let gates = &Gate::build(f, &eq);
    let r1cs = R1CSTmpl::from_gates(f, gates);

    let h = r1cs.indices;
    let w = [
      Term::One,
      Term::Num(f.elem(&3u8)),
      Term::Var("x".to_string()),
      Term::TmpVar(1),
      Term::Num(f.elem(&4u8)),
      Term::TmpVar(2),
      Term::Num(f.elem(&2u8)),
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
        Term::Num(n) => n.to_string(),
        Term::Var(s) => s.clone(),
        Term::TmpVar(i) => format!("t{}", i),
        Term::One => "1".to_string(),
        Term::Out => "out".to_string(),
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
    assert_eq!(res[1], ("t1 + 4".to_string(), "1".to_string(), "t2".to_string()));
    assert_eq!(res[2], ("t2".to_string(), "1".to_string(), "out".to_string()));
  }
 }