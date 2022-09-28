use crate::building_block::field::{Field, FieldElem};
use crate::snarks::equation_parser::Equation;
use std::collections::HashMap;
use std::cmp::{PartialEq, Eq};

use super::config::SignalId;

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
  pub a: Vec<Vec<Term>>,
  pub b: Vec<Vec<Term>>,
  pub c: Vec<Vec<Term>>,
  pub w_tmpl: Vec<Term>,
  pub w_index: HashMap<Term, usize>,
}

impl R1CS {
  pub fn build_template(gates: &[Gate]) -> R1CS {
    let mut w_index = HashMap::<Term, usize>::new();
    let mut w_tmpl: Vec<Term> = vec![Term::One];

    // TODO
    // 1. don't add term if already in w
    // 2. handle Sum properly
    for gate in gates {
      w_tmpl.push(gate.a.clone());
      w_index.insert(gate.a.clone(), gates.len());

      w_tmpl.push(gate.b.clone());
      w_index.insert(gate.b.clone(), gates.len());

      w_tmpl.push(gate.c.clone());
      w_index.insert(gate.c.clone(), gates.len());
    }

    R1CS {
      a: vec![vec![]],
      b: vec![vec![]],
      c: vec![vec![]],
      w_tmpl,
      w_index,
    }
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
    let input = "3 * x + 4 = 10";
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
    let input = "3 * x + 4 = 10";
    println!("Equation: {}", input);
    let (_, eq) = Parser::parse(f, input).unwrap();

    let gates = &Gate::build(f, &eq);
    for gate in gates {
      println!("{:?}", gate);
    }

    let r1cs = R1CS::build_template(gates);

    for (i, term) in r1cs.w_tmpl.iter().enumerate() {
      println!("{}: {:?}", i, term);
    }
  }
}