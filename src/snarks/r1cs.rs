use crate::building_block::field::{Field, FieldElem};
use crate::snarks::equation_parser::Equation;
use std::collections::HashMap;
use std::cmp::{PartialEq, Eq};

use super::config::SignalId;
use super::matrix::SparseVec;

#[derive(Clone, Hash, PartialEq, Eq)]
pub enum Term {
  Num(FieldElem),
  One,
  Out,
  Sum(Box<Term>, Box<Term>),  // Sum will only not contain Out and Sum itself
  TmpVar(SignalId),
  Var(String),
}

impl std::fmt::Debug for Term {
  fn fmt(&self, _f: &mut std::fmt::Formatter) -> std::fmt::Result {
      match self {
        Term::Num(n) => print!("{:?}", n.n),
        Term::One => print!("1"),
        Term::Out => print!("out"),
        Term::Sum(a, b) => print!("({:?} + {:?})", a, b),
        Term::TmpVar(n) => print!("t{}", n),
        Term::Var(s) => print!("{}", s),
      };
      Ok(())
  }
}

pub struct R1CS {
  pub abcs: Vec<(SparseVec, SparseVec, SparseVec)>,
  pub w_tmpl: Vec<Term>,
  pub w_index: HashMap<Term, usize>,
}

impl R1CS {
  fn new() -> Self {
    let mut r1cs = R1CS {
      abcs: vec![],
      w_tmpl: vec![],
      w_index: HashMap::<Term, usize>::new(),
    };
    r1cs.update_index(&Term::One);
    r1cs
  }

  fn update_index(&mut self, t: &Term) {
    match t {
      Term::Sum(a, b) => {
        self.update_index(&a);
        self.update_index(&b);
      },
      t => {
        if self.w_index.contains_key(t) { return };
        self.w_tmpl.push(t.clone());
        self.w_index.insert(t.clone(), self.w_index.len());
      }
    }
  }

  fn gate_to_vec(r1cs: &R1CS, f: &Field, vec: &mut SparseVec, term: &Term) {
    match term {
      Term::Sum(a, b) => {
        R1CS::gate_to_vec(r1cs, f, vec, &a);
        R1CS::gate_to_vec(r1cs, f, vec, &b);
      },
      x => {
        let index = r1cs.w_index.get(&x).unwrap();
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

  pub fn from_gates(f: &Field, gates: &[Gate]) -> R1CS {
    let mut r1cs = R1CS::new();

    // build witness vector
    for gate in gates {
      r1cs.update_index(&gate.a);
      r1cs.update_index(&gate.b);
      r1cs.update_index(&gate.c);
    }

    let vec_size = gates.len();

    for gate in gates {
      let mut a_vec = SparseVec::new(vec_size);
      R1CS::gate_to_vec(&r1cs, f, &mut a_vec, &gate.a);

      let mut b_vec = SparseVec::new(vec_size);
      R1CS::gate_to_vec(&r1cs, f, &mut b_vec, &gate.b);

      let mut c_vec = SparseVec::new(vec_size);
      R1CS::gate_to_vec(&r1cs, f, &mut c_vec, &gate.c);

      let abc = (a_vec, b_vec, c_vec);
      r1cs.abcs.push(abc)
    }

    r1cs
  }
}

pub struct Gate {
  pub a: Term,
  pub b: Term,
  pub c: Term,
}

impl std::fmt::Debug for Gate {
  fn fmt(&self, _f: &mut std::fmt::Formatter) -> std::fmt::Result {
      print!("{:?} = {:?} * {:?}", self.c, self.a, self.b);
      Ok(())
  }
}

impl Gate {
  // traverse the Equation tree generating statement at each Add/Mul node
  fn traverse_and_build(
    f: &Field, eq: &Equation, gates: &mut Vec<Gate>
  ) -> Term {
    match eq {
      Equation::Num(n) => Term::Num(n.clone()),
      Equation::Var(s) => Term::Var(s.clone()),

      Equation::Add(signal_id, left, right) => {
        let a = Gate::traverse_and_build(f, left, gates);
        let b = Gate::traverse_and_build(f, right, gates);
        let c = Term::TmpVar(*signal_id);
        // a + b = c
        // -> (a + b) * 1 = c
        let sum = Term::Sum(Box::new(a), Box::new(b));
        gates.push(Gate { a: sum, b: Term::One, c: c.clone() });
        c
      },
      Equation::Mul(signal_id, left, right) => {
        let a = Gate::traverse_and_build(f, left, gates);
        let b = Gate::traverse_and_build(f, right, gates);
        let c = Term::TmpVar(*signal_id);
        gates.push(Gate { a, b, c: c.clone() });
        c
      },
    }
  }

  pub fn build(f: &Field, eq: &Equation) -> Vec<Gate> {
    let mut gates: Vec<Gate> = vec![];
    let root = Gate::traverse_and_build(f, eq, &mut gates);

    let out_gate = Gate { a: root, b: Term::One, c: Term::Out };
    gates.push(out_gate);
    gates
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::snarks::equation_parser::Parser;

  #[test]
  fn test_gate_build() {
    let f = &Field::new(&3911u16);
    let input = "3 * x + 4";
    println!("Equation: {}", input);
    let (_, eq) = Parser::parse(f, input).unwrap();

    let gates = Gate::build(f, &eq);

    for gate in gates {
      println!("{:?}", gate);
    }
  }

  #[test]
  fn test_r1cs_build_template() {
    let f = &Field::new(&3911u16);
    let input = "3 * x + 4";
    let (_, eq) = Parser::parse(f, input).unwrap();

    let gates = &Gate::build(f, &eq);
    let r1cs = R1CS::from_gates(f, gates);

    // expected w: [1, 3, x, t1, 4, t2, out]
    let h = r1cs.w_index;
    let w = [
      Term::One,
      Term::Num(f.elem(&3u8)),
      Term::Var("x".to_string()),
      Term::TmpVar(1),
      Term::Num(f.elem(&4u8)),
      Term::TmpVar(2),
      Term::Out,
    ];
    assert_eq!(h.len(), w.len());

    for (i, term) in w.iter().enumerate() {
      assert_eq!(h.get(&term).unwrap(), &i);
    }
  }

  fn term_to_str(r1cs: &R1CS, vec: &SparseVec) -> String {
    let indices = vec.indices();
    let s = indices.iter().map(|i| {
      match &r1cs.w_tmpl[*i] {
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
    let input = "3 * x + 4";
    let (_, eq) = Parser::parse(f, input).unwrap();

    let gates = &Gate::build(f, &eq);
    let r1cs = R1CS::from_gates(f, gates);

    let mut res = vec![];
    for abc in &r1cs.abcs {
      let a = term_to_str(&r1cs, &abc.0);
      let b = term_to_str(&r1cs, &abc.1);
      let c = term_to_str(&r1cs, &abc.2);
      res.push((a, b, c));
    }

    assert_eq!(res.len(), 3);
    assert_eq!(res[0], ("3".to_string(), "x".to_string(), "t1".to_string()));
    assert_eq!(res[1], ("4 + t1".to_string(), "1".to_string(), "t2".to_string()));  // not "t1 + 4" due to the term order in w_index
    assert_eq!(res[2], ("t2".to_string(), "1".to_string(), "out".to_string()));
  }
 }