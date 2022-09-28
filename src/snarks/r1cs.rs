use crate::building_block::field::{Field, FieldElem};
use crate::snarks::equation_parser::Equation;

use super::config::SignalId;

#[derive(Clone)]
pub enum Term {
  Var(String),
  TmpVar(SignalId),
  OutVar,
  Num(FieldElem),
  Sum(Box<Term>, Box<Term>),  // Sum will not contain Sum
}

impl std::fmt::Debug for Term {
  fn fmt(&self, _f: &mut std::fmt::Formatter) -> std::fmt::Result {
      match self {
        Term::Var(s) => print!("{}", s),
        Term::TmpVar(n) => print!("t{}", n),
        Term::OutVar => print!("out"),
        Term::Num(n) => print!("{:?}", n.n),
        Term::Sum(a, b) => print!("({:?} + {:?})", a, b),
      };
      Ok(())
  }
}

pub struct Statement{
  pub a: Term,
  pub b: Term,
  pub c: Term,
}

impl std::fmt::Debug for Statement {
  fn fmt(&self, _f: &mut std::fmt::Formatter) -> std::fmt::Result {
      print!("{:?} = {:?} * {:?}", self.c, self.a, self.b);
      Ok(())
  }
}

impl Statement {
  // traverse the Equation tree generating statement at each Add/Mul node
  fn traverse_and_build(
    f: &Field, eq: &Equation, stmts: &mut Vec<Statement>
  ) -> Term {
    match eq {
      Equation::Num(n) => Term::Num(n.clone()),
      Equation::Var(s) => Term::Var(s.clone()),

      Equation::Add(signal_id, left, right) => {
        let a = Statement::traverse_and_build(f, left, stmts);
        let b = Statement::traverse_and_build(f, right, stmts);
        let c = Term::TmpVar(*signal_id);
        // a + b = c
        // -> (a + b) * 1 = c
        let one = Term::Num(f.elem(&1u8));
        let sum = Term::Sum(Box::new(a), Box::new(b));
        stmts.push(Statement { a: sum, b: one, c: c.clone() });
        c
      },
      Equation::Mul(signal_id, left, right) => {
        let a = Statement::traverse_and_build(f, left, stmts);
        let b = Statement::traverse_and_build(f, right, stmts);
        let c = Term::TmpVar(*signal_id);
        stmts.push(Statement { a, b, c: c.clone() });
        c
      },
    }
  }

  pub fn build(f: &Field, eq: &Equation) -> Vec<Statement> {
    let mut stmts: Vec<Statement> = vec![];
    let root = Statement::traverse_and_build(f, eq, &mut stmts);

    let one = Term::Num(f.elem(&1u8));
    let out_stmt = Statement { a: root, b: one, c: Term::OutVar };
    stmts.push(out_stmt);
    stmts
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::snarks::equation_parser::Parser;

  #[test]
  fn test_generate_from_simple_equation() {
    let f = &Field::new(&3911u16);
    let input = "3 * x + 4 = 10";
    println!("Equation: {}", input);
    let (_, eq) = Parser::parse(f, input).unwrap();

    let stmts = Statement::build(f, &eq);

    for stmt in stmts {
      println!("{:?}", stmt);
    }
  }
}