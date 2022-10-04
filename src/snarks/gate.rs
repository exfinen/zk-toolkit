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
  fn fmt(&self, _f: &mut std::fmt::Formatter) -> std::fmt::Result {
      print!("{:?} = {:?} * {:?}", self.c, self.a, self.b);
      Ok(())
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

