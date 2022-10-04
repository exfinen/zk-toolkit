use crate::building_block::field::Field;
use crate::snarks::{
  term::Term,
  equation_parser::{Equation, MathExpr},
};

pub struct Gate {
  pub a: Term,
  pub b: Term,
  pub c: Term,
}

impl std::fmt::Debug for Gate {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
      write!(f, "{:?} = {:?} * {:?}", self.c, self.a, self.b)
  }
}

impl Gate {
  // traverse the Equation tree generating statement at each Add/Mul node
  fn traverse_lhs(
    f: &Field, expr: &MathExpr, gates: &mut Vec<Gate>
  ) -> Term {
    match expr {
      MathExpr::Num(n) => Term::Num(n.clone()),
      MathExpr::Var(s) => Term::Var(s.clone()),

      MathExpr::Add(signal_id, left, right) => {
        let a = Gate::traverse_lhs(f, left, gates);
        let b = Gate::traverse_lhs(f, right, gates);
        let c = Term::TmpVar(*signal_id);
        // a + b = c
        // -> (a + b) * 1 = c
        let sum = Term::Sum(Box::new(a), Box::new(b));
        gates.push(Gate { a: sum, b: Term::One, c: c.clone() });
        c
      },
      MathExpr::Mul(signal_id, left, right) => {
        let a = Gate::traverse_lhs(f, left, gates);
        let b = Gate::traverse_lhs(f, right, gates);
        let c = Term::TmpVar(*signal_id);
        gates.push(Gate { a, b, c: c.clone() });
        c
      },
      MathExpr::Sub(signal_id, left, right) => {
        let a = Gate::traverse_lhs(f, left, gates);
        let b = Gate::traverse_lhs(f, right, gates);
        let c = Term::TmpVar(*signal_id);
        // a - b = c
        // -> b + c = a
        // -> (b + c) * 1 = a
        let sum = Term::Sum(Box::new(b), Box::new(c.clone()));
        gates.push(Gate { a: sum, b: Term::One, c: a.clone() });
        c
      },
      MathExpr::Div(signal_id, left, right) => {
        let a = Gate::traverse_lhs(f, left, gates);
        let b = Gate::traverse_lhs(f, right, gates);
        let c = Term::TmpVar(*signal_id);
        // a / b = c
        // -> b * c = a
        gates.push(Gate { a: b, b: c.clone(), c: a.clone() });
        // send c to next as the original division does
        c
      },
      MathExpr::Equation(_lhs, _rhs) => {
        panic!("should not be visited");
      }
    }
  }

  pub fn build(f: &Field, eq: &Equation) -> Vec<Gate> {
    let mut gates: Vec<Gate> = vec![];
    let lhs = Gate::traverse_lhs(f, &eq.lhs, &mut gates);

    let out_gate = Gate { a: lhs, b: Term::One, c: Term::Out };
    gates.push(out_gate);
    gates
  }
}


#[cfg(test)]
mod tests {
  use super::*;
  use crate::snarks::equation_parser::Parser;

  #[test]
  fn test_build_add() {
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
  fn test_build_sub() {
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
  fn test_build_mul() {
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
  fn test_build_div() {
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
  fn test_build_combined() {
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
}
