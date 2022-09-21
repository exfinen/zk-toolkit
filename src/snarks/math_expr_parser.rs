use nom::{
  IResult,
  branch::alt,
  character::{
    complete::{ char, one_of, multispace0, alpha1 },
  },
  combinator::{ opt, recognize },
  multi::{ many0, many1 },
  sequence::{ tuple, delimited, terminated },
};
use crate::building_block::field::{Field, FieldElem};
use num_bigint::{BigInt, BigUint, Sign};
use std::cell::Cell;

type SignalId = u128;

#[derive(Debug, PartialEq, Clone)]
pub enum MathExpr {
  Num(SignalId, FieldElem),
  Var(SignalId, String),
  Mul(SignalId, Box<MathExpr>, Box<MathExpr>),
  Div(SignalId, Box<MathExpr>, Box<MathExpr>),
  Add(SignalId, Box<MathExpr>, Box<MathExpr>),
  Sub(SignalId, Box<MathExpr>, Box<MathExpr>),
}

pub struct Parser {
  f: Field,
}

macro_rules! set_next_id {
  ($signal_id: expr) => {
    $signal_id.set($signal_id.get() + 1);
  };
}

impl Parser {
  fn num_str_to_field_elem(f: &Field, s: &str) -> FieldElem {
    if s.starts_with("-") {
      let mut n = BigInt::parse_bytes(s.as_bytes(), 10).unwrap();
      if n.sign() == Sign::Minus {
        let order = BigInt::from_biguint(Sign::Plus, (*f.order).clone());
        n = -n;
        n = n % &order;
        n = &order - n;
        let n = n.to_biguint().unwrap();
        f.elem(&n)

      } else {
        let n = n.to_biguint().unwrap();
        f.elem(&n)
      }
    } else { // if positive
      let n = BigUint::parse_bytes(s.as_bytes(), 10).unwrap();
      f.elem(&n)
    }
  }

  fn variable<'a>(&'a self, signal_id: &'a Cell<u128>) -> impl Fn(&str) -> IResult<&str, MathExpr> + 'a {
    |input| {
      let (input, s) =
        delimited(
          multispace0,
          recognize(
            terminated(alpha1, many0(one_of("0123456789")))
          ),
          multispace0
        )(input)?;

      set_next_id!(signal_id);
      Ok((input, MathExpr::Var(signal_id.get(), s.to_string())))
    }
  }

  fn decimal<'a>(&'a self, signal_id: &'a Cell<u128>) -> impl Fn(&str) -> IResult<&str, MathExpr> + 'a {
    |input| {
      let (input, s) =
        delimited(
          multispace0,
          recognize(
            tuple((
              opt(char('-')),
              many1(
                one_of("0123456789")
              ),
            )),
          ),
          multispace0
        )(input)?;

      set_next_id!(signal_id);
      let n = Parser::num_str_to_field_elem(&self.f, s);
      Ok((input, MathExpr::Num(signal_id.get(), n)))
    }
  }

  // <term2> ::= <variable> | <number> | '(' <expr> ')'
  fn term2<'a>(&'a self, signal_id: &'a Cell<u128>) -> impl Fn(&str) -> IResult<&str, MathExpr> + 'a {
    |input| {
      let (input, node) = alt((
        Parser::variable(self, signal_id),
        Parser::decimal(self, signal_id),
        delimited(
          delimited(multispace0, char('('), multispace0),
          Parser::expr(self, signal_id),
          delimited(multispace0, char(')'), multispace0),
        ),
      ))(input)?;

      Ok((input, node))
    }
  }

  // <term1> ::= <term2> [ ('*'|'/') <term2> ]*
  fn term1<'a>(&'a self, signal_id: &'a Cell<u128>) -> impl Fn(&str) -> IResult<&str, MathExpr> + 'a {
    |input| {
      let rhs = tuple((alt((char('*'), char('/'))), Parser::term2(self, signal_id)));
      let (input, (lhs, rhs)) = tuple((
        Parser::term2(self, signal_id),
        many0(rhs),
      ))(input)?;

      if rhs.len() == 0 {
        Ok((input, lhs))
      } else {
        // translate rhs vector to Mul<Mul<..,Mul>>>..
        let rhs_head = &rhs[0];
        let rhs = rhs.iter().skip(1).fold(rhs_head.1.clone(), |acc, x| {
          match x {
            ('*', node) => {
              set_next_id!(signal_id);
              MathExpr::Mul(signal_id.get(), Box::new(acc), Box::new(node.clone()))
            },
            ('/', node) => {
              set_next_id!(signal_id);
              MathExpr::Div(signal_id.get(), Box::new(acc), Box::new(node.clone()))
            },
            (op, _) => panic!("unexpected operator encountered in term1 {}", op),
          }
        });

        set_next_id!(signal_id);
        let node = if rhs_head.0 == '*' {
          MathExpr::Mul(signal_id.get(), Box::new(lhs), Box::new(rhs))
        } else {
          MathExpr::Div(signal_id.get(), Box::new(lhs), Box::new(rhs))
        };
        Ok((input, node))
      }
    }
  }

  // <expr> ::= <term1> [ ('+'|'-') <term1> ]*
  fn expr<'a>(&'a self, signal_id: &'a Cell<u128>) -> impl Fn(&str) -> IResult<&str, MathExpr> + 'a {
    |input| {
      let rhs = tuple((alt((char('+'), char('-'))), Parser::term1(self, signal_id)));
      let (input, (lhs, rhs)) = tuple((
        Parser::term1(self, signal_id),
        many0(rhs),
      ))(input)?;

      if rhs.len() == 0 {
        Ok((input, lhs))
      } else {
        // translate rhs vector to Add<Add<..,Add>>>..
        let rhs_head = &rhs[0];
        let rhs = rhs.iter().skip(1).fold(rhs_head.1.clone(), |acc, x| {
          match x {
            ('+', node) => {
              set_next_id!(signal_id);
              MathExpr::Add(signal_id.get(), Box::new(acc), Box::new(node.clone()))
            },
            ('-', node) => {
              set_next_id!(signal_id);
              MathExpr::Sub(signal_id.get(), Box::new(acc), Box::new(node.clone()))
            },
            (op, _) => panic!("unexpected operator encountered in expr: {}", op),
          }
        });

        set_next_id!(signal_id);
        let node = if rhs_head.0 == '+' {
          MathExpr::Add(signal_id.get(), Box::new(lhs), Box::new(rhs))
        } else {
          MathExpr::Sub(signal_id.get(), Box::new(lhs), Box::new(rhs))
        };
        Ok((input, node))
      }
    }
  }

  // <term1> ::= <term2> [ ('*'|'/') <term2> ]*
  // <term2> ::= <variable> | <number> | '(' <expr> ')'
  // <expr> ::= <term1> [ ('+'|'-') <term1> ]*
  pub fn parse<'a>(&self, input: &'a str) -> IResult<&'a str, MathExpr> {
    let signal_id = Cell::new(0);
    let expr = Parser::expr(self, &signal_id);
    expr(input)
  }

  pub fn new(f: &Field) -> Parser {
    let f = f.clone();
    Parser { f }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_decimal() {
    let f = &Field::new(&11u8);
    let parser = Parser::new(f);
    match parser.parse("123") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, MathExpr::Num(1, f.elem(&123u8)));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_decimal_with_spaces() {
    let f = &Field::new(&11u8);
    let parser = Parser::new(f);
    match parser.parse(" 123 ") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, MathExpr::Num(1, f.elem(&123u8)));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_neg_decimal_below_order() {
    let f = &Field::new(&11u8);
    let parser = Parser::new(f);
    match parser.parse("-123") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, MathExpr::Num(1, f.elem_from_signed_int(-123)));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_neg_decimal_above_order() {
    let f = &Field::new(&11u8);
    let parser = Parser::new(f);
    match parser.parse("-123") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, MathExpr::Num(1, f.elem_from_signed_int(-123)));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_1_char_variable() {
    let f = &Field::new(&11u8);
    for s in vec!["x", "x1", "x0", "xy", "xy1"] {
      let parser = Parser::new(f);
      match parser.parse(s) {
        Ok((input, x)) => {
          assert_eq!(input, "");
          assert_eq!(x, MathExpr::Var(1, s.to_string()));
        },
        Err(_) => panic!(),
      }
    }
  }

  #[test]
  fn test_1_char_variable_with_spaces() {
    let f = &Field::new(&11u8);
    for s in vec!["x", "x1", "x0", "xy", "xy1"] {
      let parser = Parser::new(f);
      match parser.parse(&format!("  {}  ", s)) {
        Ok((input, x)) => {
          assert_eq!(input, "");
          assert_eq!(x, MathExpr::Var(1, s.to_string()));
        },
        Err(_) => panic!(),
      }
    }
  }

  #[test]
  fn test_simple_add_expr() {
    let f = &Field::new(&11u8);
    let parser = Parser::new(f);
    match parser.parse("123+456") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, MathExpr::Add(3,
          Box::new(MathExpr::Num(1, f.elem(&123u16))),
          Box::new(MathExpr::Num(2, f.elem(&456u16))),
        ));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_simple_add_expr_with_1_var() {
    let f = &Field::new(&11u8);
    for s in vec!["x", "x1", "x0", "xy", "xy1"] {
      let parser = Parser::new(f);
      match parser.parse(&format!("{}+456", s)) {
        Ok((input, x)) => {
          assert_eq!(input, "");
          assert_eq!(x, MathExpr::Add(3,
            Box::new(MathExpr::Var(1, s.to_string())),
            Box::new(MathExpr::Num(2, f.elem(&456u16))),
          ));
        },
        Err(_) => panic!(),
      }
    }
  }

  #[test]
  fn test_simple_add_expr_with_2_vars() {
    let f = &Field::new(&11u8);
    for (a,b) in vec![("x", "y"), ("x1", "y1"), ("xxx1123", "yyy123443")] {
      let parser = Parser::new(f);
      match parser.parse(&format!("{}+{}", a, b)) {
        Ok((input, x)) => {
          assert_eq!(input, "");
          assert_eq!(x, MathExpr::Add(3,
            Box::new(MathExpr::Var(1, a.to_string())),
            Box::new(MathExpr::Var(2, b.to_string())),
          ));
        },
        Err(_) => panic!(),
      }
    }
  }

  #[test]
  fn test_simple_add_expr_incl_neg() {
    let f = &Field::new(&11u8);
    let parser = Parser::new(f);
    match parser.parse("123+-456") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, MathExpr::Add(3,
          Box::new(MathExpr::Num(1, f.elem(&123u16))),
          Box::new(MathExpr::Num(2, f.elem_from_signed_int(-456))),
        ));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_simple_sub_expr() {
    let f = &Field::new(&11u8);
    let parser = Parser::new(f);
    match parser.parse("123-456") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, MathExpr::Sub(3,
          Box::new(MathExpr::Num(1, f.elem(&123u16))),
          Box::new(MathExpr::Num(2, f.elem(&456u16))),
        ));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_simple_sub_expr_1_var() {
    let f = &Field::new(&11u8);
    for s in vec!["x", "x1", "x0", "xy", "xy1"] {
      let parser = Parser::new(f);
      match parser.parse(&format!("123-{}", s)) {
        Ok((input, x)) => {
          assert_eq!(input, "");
          assert_eq!(x, MathExpr::Sub(3,
            Box::new(MathExpr::Num(1, f.elem(&123u16))),
            Box::new(MathExpr::Var(2, s.to_string())),
          ));
        },
        Err(_) => panic!(),
      }
    }
  }

  #[test]
  fn test_simple_sub_expr_incl_neg1() {
    let f = &Field::new(&11u8);
    let parser = Parser::new(f);
    match parser.parse("-123-456") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, MathExpr::Sub(3,
          Box::new(MathExpr::Num(1, f.elem_from_signed_int(-123))),
          Box::new(MathExpr::Num(2, f.elem(&456u16))),
        ));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_simple_sub_expr_incl_neg1_1_var() {
    let f = &Field::new(&11u8);
    for s in vec!["x", "x1", "x0", "xy", "xy1"] {
      let parser = Parser::new(f);
      match parser.parse(&format!("-123-{}", s)) {
        Ok((input, x)) => {
          assert_eq!(input, "");
          assert_eq!(x, MathExpr::Sub(3,
            Box::new(MathExpr::Num(1, f.elem_from_signed_int(-123))),
            Box::new(MathExpr::Var(2, s.to_string())),
          ));
        },
        Err(_) => panic!(),
      }
    }
  }
  #[test]
  fn test_simple_sub_expr_incl_neg2() {
    let f = &Field::new(&11u8);
    let parser = Parser::new(f);
    match parser.parse("123--456") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, MathExpr::Sub(3,
          Box::new(MathExpr::Num(1, f.elem(&123u16))),
          Box::new(MathExpr::Num(2, f.elem_from_signed_int(-456))),
        ));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_simple_sub_expr_incl_neg2_with_spaces() {
    let f = &Field::new(&11u8);
    let parser = Parser::new(f);
    match parser.parse("123 - -456") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, MathExpr::Sub(3,
          Box::new(MathExpr::Num(1, f.elem(&123u16))),
          Box::new(MathExpr::Num(2, f.elem_from_signed_int(-456))),
        ));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_simple_sub_expr_incl_neg2_with_spaces_1_var() {
    let f = &Field::new(&11u8);
    let parser = Parser::new(f);
    match parser.parse("x - -456") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, MathExpr::Sub(3,
          Box::new(MathExpr::Var(1, "x".to_string())),
          Box::new(MathExpr::Num(2, f.elem_from_signed_int(-456))),
        ));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_simple_mul_expr() {
    let f = &Field::new(&11u8);
    let parser = Parser::new(f);
    match parser.parse("123*456") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, MathExpr::Mul(3,
          Box::new(MathExpr::Num(1, f.elem(&123u16))),
          Box::new(MathExpr::Num(2, f.elem(&456u16))),
        ));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_simple_mul_expr_incl_neg1() {
    let f = &Field::new(&11u8);
    let parser = Parser::new(f);
    match parser.parse("123*-456") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, MathExpr::Mul(3,
          Box::new(MathExpr::Num(1, f.elem(&123u16))),
          Box::new(MathExpr::Num(2, f.elem_from_signed_int(-456)),
        )));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_simple_mul_expr_incl_neg2() {
    let f = &Field::new(&11u8);
    let parser = Parser::new(f);
    match parser.parse("-123*456") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, MathExpr::Mul(3,
          Box::new(MathExpr::Num(1, f.elem_from_signed_int(-123))),
          Box::new(MathExpr::Num(2, f.elem(&456u16))),
        ));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_simple_mul_expr_incl_neg() {
    let f = &Field::new(&11u8);
    let parser = Parser::new(f);
    match parser.parse("123*-456") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, MathExpr::Mul(3,
          Box::new(MathExpr::Num(1, f.elem(&123u16))),
          Box::new(MathExpr::Num(2, f.elem_from_signed_int(-456))),
        ));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_simple_div_expr() {
    let f = &Field::new(&11u8);
    let parser = Parser::new(f);
    match parser.parse("123/456") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, MathExpr::Div(3,
          Box::new(MathExpr::Num(1, f.elem(&123u16))),
          Box::new(MathExpr::Num(2, f.elem(&456u16))),
        ));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_add_and_mul_expr() {
    let f = &Field::new(&11u8);
    let parser = Parser::new(f);
    match parser.parse("123+456*789") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, MathExpr::Add(5,
          Box::new(MathExpr::Num(1, f.elem(&123u16))),
          Box::new(MathExpr::Mul(4,
            Box::new(MathExpr::Num(2, f.elem(&456u16))),
            Box::new(MathExpr::Num(3, f.elem(&789u16)))
          )),
        ));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_add_mul_div_expr() {
    let f = &Field::new(&11u8);
    let parser = Parser::new(f);
    match parser.parse("111/222+333*444") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, MathExpr::Add(7,
          Box::new(MathExpr::Div(3,
            Box::new(MathExpr::Num(1, f.elem(&111u16))),
            Box::new(MathExpr::Num(2, f.elem(&222u16))),
          )),
          Box::new(MathExpr::Mul(6,
            Box::new(MathExpr::Num(4, f.elem(&333u16))),
            Box::new(MathExpr::Num(5, f.elem(&444u16))),
          )),
        ));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_paren_add_and_mul_expr() {
    let f = &Field::new(&11u8);
    let parser = Parser::new(f);
    match parser.parse("(123+456)*789") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, MathExpr::Mul(5,
          Box::new(MathExpr::Add(3,
            Box::new(MathExpr::Num(1, f.elem(&123u16))),
            Box::new(MathExpr::Num(2, f.elem(&456u16))),
          )),
          Box::new(MathExpr::Num(4, f.elem(&789u16))),
        ));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_paren_add_and_mul_expr_with_spaces() {
    let f = &Field::new(&11u8);
    let parser = Parser::new(f);
    match parser.parse(" (123 + 456) * 789") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, MathExpr::Mul(5,
          Box::new(MathExpr::Add(3,
            Box::new(MathExpr::Num(1, f.elem(&123u16))),
            Box::new(MathExpr::Num(2, f.elem(&456u16))),
          )),
          Box::new(MathExpr::Num(4, f.elem(&789u16))),
        ));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_paren_add_mul_sub_expr() {
    let f = &Field::new(&11u8);
    let parser = Parser::new(f);
    match parser.parse("(111+222)*(333-444)") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, MathExpr::Mul(7,
          Box::new(MathExpr::Add(3,
            Box::new(MathExpr::Num(1, f.elem(&111u16))),
            Box::new(MathExpr::Num(2, f.elem(&222u16))),
          )),
          Box::new(MathExpr::Sub(6,
            Box::new(MathExpr::Num(4, f.elem(&333u16))),
            Box::new(MathExpr::Num(5, f.elem(&444u16))),
          )),
        ));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_multiple_paren() {
    let f = &Field::new(&11u8);
    let parser = Parser::new(f);
    match parser.parse("((111+222))") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, MathExpr::Add(3,
          Box::new(MathExpr::Num(1, f.elem(&111u8))),
          Box::new(MathExpr::Num(2, f.elem(&222u8))),
        ));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_multiple_paren_with_spaces() {
    let f = &Field::new(&11u8);
    let parser = Parser::new(f);
    match parser.parse(" ( (111+222) ) ") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, MathExpr::Add(3,
          Box::new(MathExpr::Num(1, f.elem(&111u8))),
          Box::new(MathExpr::Num(2, f.elem(&222u8))),
        ));
      },
      Err(_) => panic!(),
    }
  }
}