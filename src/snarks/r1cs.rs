use crate::building_block::field::{Field, FieldElem};
use crate::snarks::equation_parser::Equation;

#[derive(Clone)]
pub enum R1CSValue {
  Var(String),
  Num(FieldElem),
  Sum(Box<R1CSValue>, Box<R1CSValue>),  // Sum will not contain Sum
}

impl std::fmt::Debug for R1CSValue {
  fn fmt(&self, _f: &mut std::fmt::Formatter) -> std::fmt::Result {
      match self {
        R1CSValue::Var(s) => print!("{}", s),
        R1CSValue::Num(n) => print!("{:?}", n.n),
        R1CSValue::Sum(a, b) => print!("({:?} + {:?})", a, b),
      };
      Ok(())
  }
}

#[derive(Clone)]
pub struct R1CS {
  pub a: R1CSValue,
  pub b: R1CSValue,
  pub c: R1CSValue,
}

impl std::fmt::Debug for R1CS {
  fn fmt(&self, _f: &mut std::fmt::Formatter) -> std::fmt::Result {
      print!("{:?} = {:?} * {:?}", self.c, self.a, self.b);
      Ok(())
  }
}

impl R1CS {
  pub fn generate(f: &Field, eq: &Equation, lines: &mut Vec<R1CS>) -> R1CSValue {
    match eq {
      Equation::Num(n) => R1CSValue::Num(n.clone()),
      Equation::Var(s) => R1CSValue::Var(s.clone()),

      Equation::Add(signal_id, eq_left, eq_right)  => {
        let a = R1CS::generate(f, eq_left, lines);
        let b = R1CS::generate(f, eq_right, lines);
        let c = R1CSValue::Var(format!("t{}", signal_id));
        // a + b = c
        // -> (a + b) * 1 = c
        let one = R1CSValue::Num(f.elem(&1u8));
        let sum = R1CSValue::Sum(Box::new(a), Box::new(b));
        lines.push(R1CS { a: sum, b: one, c: c.clone() });
        c
      },
      Equation::Mul(signal_id, eq_left, eq_right)  => {
        let a = R1CS::generate(f, eq_left, lines);
        let b = R1CS::generate(f, eq_right, lines);
        let c = R1CSValue::Var(format!("t{}", signal_id));
        lines.push(R1CS { a, b, c: c.clone() });
        c
      },
    }

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

    let mut lines: Vec<R1CS> = vec![];
    let res = R1CS::generate(f, &eq, &mut lines);

    for line in lines {
      println!("{:?}", line);
    }
    println!("out = {:?}", res);
  }

}