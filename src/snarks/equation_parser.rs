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
use crate::snarks::config::SignalId;
use num_bigint::{BigInt, BigUint};
use std::cell::Cell;

#[derive(Debug, PartialEq, Clone)]
pub enum Equation {
  Num(FieldElem),
  Var(String),
  Mul(SignalId, Box<Equation>, Box<Equation>),
  Add(SignalId, Box<Equation>, Box<Equation>),
  Div(SignalId, Box<Equation>, Box<Equation>),
  Sub(SignalId, Box<Equation>, Box<Equation>),
}

pub struct Parser();

macro_rules! set_next_id {
  ($signal_id: expr) => {
    $signal_id.set($signal_id.get() + 1);
  };
}

impl Parser {
  fn num_str_to_field_elem(f: &Field, s: &str) -> FieldElem {
    if s.starts_with("-") {
      let n = BigInt::parse_bytes(s.as_bytes(), 10).unwrap();
      f.elem_from_signed(&n)
    } else { // if positive
      let n = BigUint::parse_bytes(s.as_bytes(), 10).unwrap();
      f.elem(&n)
    }
  }

  fn variable<'a>() -> impl Fn(&str) -> IResult<&str, Equation> + 'a {
    |input| {
      let (input, s) =
        delimited(
          multispace0,
          recognize(
            terminated(alpha1, many0(one_of("0123456789")))
          ),
          multispace0
        )(input)?;

      Ok((input, Equation::Var(s.to_string())))
    }
  }

  fn decimal<'a>(f: &'a Field) -> impl Fn(&str) -> IResult<&str, Equation> + 'a {
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

      let n = Parser::num_str_to_field_elem(f, s);
      Ok((input, Equation::Num(n)))
    }
  }

  // <term2> ::= <variable> | <number> | '(' <expr> ')'
  fn term2<'a>(f: &'a Field, signal_id: &'a Cell<u128>) -> impl Fn(&str) -> IResult<&str, Equation> + 'a {
    |input| {
      let (input, node) = alt((
        Parser::variable(),
        Parser::decimal(f),
        delimited(
          delimited(multispace0, char('('), multispace0),
          Parser::expr(f, signal_id),
          delimited(multispace0, char(')'), multispace0),
        ),
      ))(input)?;

      Ok((input, node))
    }
  }

  // <term1> ::= <term2> [ ('*'|'/') <term2> ]*
  fn term1<'a>(f: &'a Field, signal_id: &'a Cell<u128>) -> impl Fn(&str) -> IResult<&str, Equation> + 'a {
    |input| {
      let rhs = tuple((alt((char('*'), char('/'))), Parser::term2(f, signal_id)));
      let (input, (lhs, rhs)) = tuple((
        Parser::term2(f, signal_id),
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
              Equation::Mul(signal_id.get(), Box::new(acc), Box::new(node.clone()))
            },
            ('/', node) => {
              set_next_id!(signal_id);
              Equation::Div(signal_id.get(), Box::new(acc), Box::new(node.clone()))
            },
            (op, _) => panic!("unexpected operator encountered in term1 {}", op),
          }
        });

        set_next_id!(signal_id);
        let node = if rhs_head.0 == '*' {
          Equation::Mul(signal_id.get(), Box::new(lhs), Box::new(rhs))
        } else {
          Equation::Div(signal_id.get(), Box::new(lhs), Box::new(rhs))
        };
        Ok((input, node))
      }
    }
  }

    // <expr> ::= <term1> [ ('+'|'-') <term1> ]*
  fn expr<'a>(f: &'a Field, signal_id: &'a Cell<u128>) -> impl Fn(&str) -> IResult<&str, Equation> + 'a {
    |input| {
      let rhs = tuple((alt((char('+'), char('-'))), Parser::term1(f, signal_id)));
      let (input, (lhs, rhs)) = tuple((
        Parser::term1(f, signal_id),
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
              Equation::Add(signal_id.get(), Box::new(acc), Box::new(node.clone()))
            },
            ('-', node) => {
              set_next_id!(signal_id);
              Equation::Sub(signal_id.get(), Box::new(acc), Box::new(node.clone()))
            },
            (op, _) => panic!("unexpected operator encountered in expr: {}", op),
          }
        });

        set_next_id!(signal_id);
        let node = if rhs_head.0 == '+' {
          Equation::Add(signal_id.get(), Box::new(lhs), Box::new(rhs))
        } else {
          Equation::Sub(signal_id.get(), Box::new(lhs), Box::new(rhs))
        };
        Ok((input, node))
      }
    }
  }

  // <term1> ::= <term2> [ ('*'|'/') <term2> ]*
  // <term2> ::= <variable> | <number> | '(' <expr> ')'
  // <expr> ::= <term1> [ ('+'|'-') <term1> ]*
  pub fn parse<'a>(f: &'a Field, input: &'a str) -> IResult<&'a str, Equation> {
    let signal_id = Cell::new(0);
    let expr = Parser::expr(f, &signal_id);
    expr(input)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_decimal() {
    let f = &Field::new(&3911u16);
    match Parser::parse(f, "123") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, Equation::Num(f.elem(&123u8)));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_decimal_with_spaces() {
    let f = &Field::new(&3911u16);
    match Parser::parse(f, " 123 ") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, Equation::Num(f.elem(&123u8)));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_neg_decimal_below_order() {
    let f = &Field::new(&3911u16);
    match Parser::parse(f, "-123") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, Equation::Num(f.elem_from_signed(&-123)));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_neg_decimal_above_order() {
    let f = &Field::new(&3911u16);
    match Parser::parse(f, "-123") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, Equation::Num(f.elem_from_signed(&-123)));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_1_char_variable() {
    let f = &Field::new(&3911u16);
    for s in vec!["x", "x1", "x0", "xy", "xy1"] {
      match Parser::parse(f, s) {
        Ok((input, x)) => {
          assert_eq!(input, "");
          assert_eq!(x, Equation::Var(s.to_string()));
        },
        Err(_) => panic!(),
      }
    }
  }

  #[test]
  fn test_1_char_variable_with_spaces() {
    let f = &Field::new(&3911u16);
    for s in vec!["x", "x1", "x0", "xy", "xy1"] {
      match Parser::parse(f, &format!("  {}  ", s)) {
        Ok((input, x)) => {
          assert_eq!(input, "");
          assert_eq!(x, Equation::Var(s.to_string()));
        },
        Err(_) => panic!(),
      }
    }
  }

  #[test]
  fn test_simple_add_expr() {
    let f = &Field::new(&3911u16);
    match Parser::parse(f, "123+456") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, Equation::Add(1,
          Box::new(Equation::Num(f.elem(&123u16))),
          Box::new(Equation::Num(f.elem(&456u16))),
        ));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_simple_add_expr_with_1_var() {
    let f = &Field::new(&3911u16);
    for s in vec!["x", "x1", "x0", "xy", "xy1"] {
      match Parser::parse(f, &format!("{}+456", s)) {
        Ok((input, x)) => {
          assert_eq!(input, "");
          assert_eq!(x, Equation::Add(1,
            Box::new(Equation::Var(s.to_string())),
            Box::new(Equation::Num(f.elem(&456u16))),
          ));
        },
        Err(_) => panic!(),
      }
    }
  }

  #[test]
  fn test_simple_add_expr_with_2_vars() {
    let f = &Field::new(&3911u16);
    for (a,b) in vec![("x", "y"), ("x1", "y1"), ("xxx1123", "yyy123443")] {
      match Parser::parse(f, &format!("{}+{}", a, b)) {
        Ok((input, x)) => {
          assert_eq!(input, "");
          assert_eq!(x, Equation::Add(1,
            Box::new(Equation::Var(a.to_string())),
            Box::new(Equation::Var(b.to_string())),
          ));
        },
        Err(_) => panic!(),
      }
    }
  }

  #[test]
  fn test_simple_add_expr_incl_neg() {
    let f = &Field::new(&3911u16);
    match Parser::parse(f, "123+-456") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, Equation::Add(1,
          Box::new(Equation::Num(f.elem(&123u16))),
          Box::new(Equation::Num(f.elem_from_signed(&-456))),
        ));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_simple_sub_expr() {
    let f = &Field::new(&3911u16);
    match Parser::parse(f, "123-456") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, Equation::Sub(1,
          Box::new(Equation::Num(f.elem(&123u16))),
          Box::new(Equation::Num(f.elem(&456u16))),
        ));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_simple_sub_expr_1_var() {
    let f = &Field::new(&3911u16);
    for s in vec!["x", "x1", "x0", "xy", "xy1"] {
      match Parser::parse(f, &format!("123-{}", s)) {
        Ok((input, x)) => {
          assert_eq!(input, "");
          assert_eq!(x, Equation::Sub(1,
            Box::new(Equation::Num(f.elem(&123u16))),
            Box::new(Equation::Var(s.to_string())),
          ));
        },
        Err(_) => panic!(),
      }
    }
  }

  #[test]
  fn test_simple_sub_expr_incl_neg1() {
    let f = &Field::new(&3911u16);
    match Parser::parse(f, "-123-456") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, Equation::Sub(1,
          Box::new(Equation::Num(f.elem_from_signed(&-123))),
          Box::new(Equation::Num(f.elem(&456u16))),
        ));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_simple_sub_expr_incl_neg1_1_var() {
    let f = &Field::new(&3911u16);
    for s in vec!["x", "x1", "x0", "xy", "xy1"] {
      match Parser::parse(f, &format!("-123-{}", s)) {
        Ok((input, x)) => {
          assert_eq!(input, "");
          assert_eq!(x, Equation::Sub(1,
            Box::new(Equation::Num(f.elem_from_signed(&-123))),
            Box::new(Equation::Var(s.to_string())),
          ));
        },
        Err(_) => panic!(),
      }
    }
  }
  #[test]
  fn test_simple_sub_expr_incl_neg2() {
    let f = &Field::new(&3911u16);
    match Parser::parse(f, "123--456") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, Equation::Sub(1,
          Box::new(Equation::Num(f.elem(&123u16))),
          Box::new(Equation::Num(f.elem_from_signed(&-456))),
        ));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_simple_sub_expr_incl_neg2_with_spaces() {
    let f = &Field::new(&3911u16);
    match Parser::parse(f, "123 - -456") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, Equation::Sub(1,
          Box::new(Equation::Num(f.elem(&123u16))),
          Box::new(Equation::Num(f.elem_from_signed(&-456))),
        ));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_simple_sub_expr_incl_neg2_with_spaces_1_var() {
    let f = &Field::new(&3911u16);
    match Parser::parse(f, "x - -456") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, Equation::Sub(1,
          Box::new(Equation::Var("x".to_string())),
          Box::new(Equation::Num(f.elem_from_signed(&-456))),
        ));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_simple_mul_expr() {
    let f = &Field::new(&3911u16);
    match Parser::parse(f, "123*456") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, Equation::Mul(1,
          Box::new(Equation::Num(f.elem(&123u16))),
          Box::new(Equation::Num(f.elem(&456u16))),
        ));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_simple_mul_expr_incl_neg1() {
    let f = &Field::new(&3911u16);
    match Parser::parse(f, "123*-456") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, Equation::Mul(1,
          Box::new(Equation::Num(f.elem(&123u16))),
          Box::new(Equation::Num(f.elem_from_signed(&-456)),
        )));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_simple_mul_expr_incl_neg2() {
    let f = &Field::new(&3911u16);
    match Parser::parse(f, "-123*456") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, Equation::Mul(1,
          Box::new(Equation::Num(f.elem_from_signed(&-123))),
          Box::new(Equation::Num(f.elem(&456u16))),
        ));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_simple_mul_expr_incl_neg() {
    let f = &Field::new(&3911u16);
    match Parser::parse(f, "123*-456") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, Equation::Mul(1,
          Box::new(Equation::Num(f.elem(&123u16))),
          Box::new(Equation::Num(f.elem_from_signed(&-456))),
        ));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_simple_div_expr() {
    let f = &Field::new(&3911u16);
    match Parser::parse(f, "123/456") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, Equation::Div(1,
          Box::new(Equation::Num(f.elem(&123u16))),
          Box::new(Equation::Num(f.elem(&456u16))),
        ));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_add_and_mul_expr() {
    let f = &Field::new(&3911u16);
    match Parser::parse(f, "123+456*789") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, Equation::Add(2,
          Box::new(Equation::Num(f.elem(&123u16))),
          Box::new(Equation::Mul(1,
            Box::new(Equation::Num(f.elem(&456u16))),
            Box::new(Equation::Num(f.elem(&789u16)))
          )),
        ));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_add_mul_div_expr() {
    let f = &Field::new(&3911u16);
    match Parser::parse(f, "111/222+333*444") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, Equation::Add(3,
          Box::new(Equation::Div(1,
            Box::new(Equation::Num(f.elem(&111u16))),
            Box::new(Equation::Num(f.elem(&222u16))),
          )),
          Box::new(Equation::Mul(2,
            Box::new(Equation::Num(f.elem(&333u16))),
            Box::new(Equation::Num(f.elem(&444u16))),
          )),
        ));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_paren_add_and_mul_expr() {
    let f = &Field::new(&3911u16);
    match Parser::parse(f, "(123+456)*789") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, Equation::Mul(2,
          Box::new(Equation::Add(1,
            Box::new(Equation::Num(f.elem(&123u16))),
            Box::new(Equation::Num(f.elem(&456u16))),
          )),
          Box::new(Equation::Num(f.elem(&789u16))),
        ));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_paren_add_and_mul_expr_with_spaces() {
    let f = &Field::new(&3911u16);
    match Parser::parse(f, " (123 + 456) * 789") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, Equation::Mul(2,
          Box::new(Equation::Add(1,
            Box::new(Equation::Num(f.elem(&123u16))),
            Box::new(Equation::Num(f.elem(&456u16))),
          )),
          Box::new(Equation::Num(f.elem(&789u16))),
        ));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_paren_add_mul_sub_expr() {
    let f = &Field::new(&3911u16);
    match Parser::parse(f, "(111+222)*(333-444)") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, Equation::Mul(3,
          Box::new(Equation::Add(1,
            Box::new(Equation::Num(f.elem(&111u16))),
            Box::new(Equation::Num(f.elem(&222u16))),
          )),
          Box::new(Equation::Sub(2,
            Box::new(Equation::Num(f.elem(&333u16))),
            Box::new(Equation::Num(f.elem(&444u16))),
          )),
        ));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_multiple_paren() {
    let f = &Field::new(&3911u16);
    match Parser::parse(f, "((111+222))") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, Equation::Add(1,
          Box::new(Equation::Num(f.elem(&111u8))),
          Box::new(Equation::Num(f.elem(&222u8))),
        ));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_multiple_paren_with_spaces() {
    let f = &Field::new(&3911u16);
    match Parser::parse(f, " ( (111+222) ) ") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, Equation::Add(1,
          Box::new(Equation::Num(f.elem(&111u8))),
          Box::new(Equation::Num(f.elem(&222u8))),
        ));
      },
      Err(_) => panic!(),
    }
  }
}