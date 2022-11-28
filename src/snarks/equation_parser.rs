use nom::{
  IResult,
  branch::alt,
  bytes::complete::tag,
  character::{
    complete::{ alpha1, char, multispace0, one_of },
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
pub enum MathExpr {
  Equation(Box<MathExpr>, Box<MathExpr>),
  Num(FieldElem),
  Var(String),
  Mul(SignalId, Box<MathExpr>, Box<MathExpr>),
  Add(SignalId, Box<MathExpr>, Box<MathExpr>),
  Div(SignalId, Box<MathExpr>, Box<MathExpr>),
  Sub(SignalId, Box<MathExpr>, Box<MathExpr>),
}

#[derive(Debug)]
pub struct Equation {
  pub lhs: MathExpr,
  pub rhs: FieldElem,
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

  fn variable<'a>() -> impl Fn(&str) -> IResult<&str, MathExpr> + 'a {
    |input| {
      let (input, s) =
        delimited(
          multispace0,
          recognize(
            terminated(alpha1, many0(one_of("0123456789")))
          ),
          multispace0
        )(input)?;

      Ok((input, MathExpr::Var(s.to_string())))
    }
  }

  fn decimal<'a>(f: &'a Field) -> impl Fn(&str) -> IResult<&str, MathExpr> + 'a {
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
      Ok((input, MathExpr::Num(n)))
    }
  }

  // <term2> ::= <variable> | <number> | '(' <expr> ')'
  fn term2<'a>(f: &'a Field, signal_id: &'a Cell<u128>) -> impl Fn(&str) -> IResult<&str, MathExpr> + 'a {
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
  fn term1<'a>(f: &'a Field, signal_id: &'a Cell<u128>) -> impl Fn(&str) -> IResult<&str, MathExpr> + 'a {
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
  fn expr<'a>(f: &'a Field, signal_id: &'a Cell<u128>) -> impl Fn(&str) -> IResult<&str, MathExpr> + 'a {
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

  // <equation> ::= <expr> '=' <number>
  fn equation<'a>(f: &'a Field, signal_id: &'a Cell<u128>) -> impl Fn(&str) -> IResult<&str, MathExpr> + 'a {
    |input| {
      let (input, out) =
        tuple((
          multispace0,
          Parser::expr(f, signal_id),
          multispace0,
          tag("=="),
          multispace0,
          Parser::decimal(f),
          multispace0,
        ))(input)?;

      let lhs = out.1;
      let rhs = out.5;
      Ok((input, MathExpr::Equation(Box::new(lhs), Box::new(rhs))))
    }
  }
  // <term1> ::= <term2> [ ('*'|'/') <term2> ]*
  // <term2> ::= <variable> | <number> | '(' <expr> ')'
  // <expr> ::= <term1> [ ('+'|'-') <term1> ]*
  // <equation> ::= <expr> '==' <number>
  pub fn parse<'a>(f: &'a Field, input: &'a str) -> Result<Equation, String> {
    let signal_id = Cell::new(0);
    let expr = Parser::equation(f, &signal_id);
    match expr(input) {
      Ok((_, expr)) => {
        match expr {
          MathExpr::Equation(lhs, rhs) => {
            if let MathExpr::Num(n) = *rhs {
              Ok(Equation { lhs: *lhs, rhs: n })
            } else {
              Err(format!("Equation has unexpected RHS: {:?}", rhs))
            }
          },
          _ => Err(format!("Unexpected parse result: {:?}", expr))
        }

      },
      Err(x) => Err(x.to_string()),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_decimal() {
    let f = &Field::new(&3911u16);
    match Parser::parse(f, "123 == 123") {
      Ok(eq) => {
        assert_eq!(eq.lhs, MathExpr::Num(f.elem(&123u8)));
        assert_eq!(eq.rhs, f.elem(&123u8));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_decimal_with_spaces() {
    let f = &Field::new(&3911u16);
    match Parser::parse(f, " 123 == 123 ") {
      Ok(eq) => {
        assert_eq!(eq.lhs, MathExpr::Num(f.elem(&123u8)));
        assert_eq!(eq.rhs, f.elem(&123u8));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_neg_decimal_below_order() {
    let f = &Field::new(&3911u16);
    match Parser::parse(f, "-123 == -123") {
      Ok(eq) => {
        assert_eq!(eq.lhs, MathExpr::Num(f.elem_from_signed(&-123)));
        assert_eq!(eq.rhs, f.elem_from_signed(&-123));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_neg_decimal_above_order() {
    let f = &Field::new(&11u16);
    match Parser::parse(f, "-123 == -123") {
      Ok(eq) => {
        assert_eq!(eq.lhs, MathExpr::Num(f.elem_from_signed(&-123)));
        assert_eq!(eq.rhs, f.elem_from_signed(&-123));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_1_char_variable() {
    let f = &Field::new(&3911u16);
    for s in vec!["x", "x1", "x0", "xy", "xy1"] {
      match Parser::parse(f, &format!("{} == 123", s)) {
        Ok(eq) => {
          assert_eq!(eq.lhs, MathExpr::Var(s.to_string()));
          assert_eq!(eq.rhs, f.elem(&123u8));
        },
        Err(_) => panic!(),
      }
    }
  }

  #[test]
  fn test_1_char_variable_with_spaces() {
    let f = &Field::new(&3911u16);
    for s in vec!["x", "x1", "x0", "xy", "xy1"] {
      match Parser::parse(f, &format!("  {} == 123  ", s)) {
        Ok(eq) => {
          assert_eq!(eq.lhs, MathExpr::Var(s.to_string()));
          assert_eq!(eq.rhs, f.elem(&123u8));
        },
        Err(_) => panic!(),
      }
    }
  }

  #[test]
  fn test_simple_add_expr() {
    let f = &Field::new(&3911u16);
    match Parser::parse(f, "123+456==123") {
      Ok(eq) => {
        assert_eq!(eq.lhs, MathExpr::Add(1,
          Box::new(MathExpr::Num(f.elem(&123u16))),
          Box::new(MathExpr::Num(f.elem(&456u16))),
        ));
        assert_eq!(eq.rhs, f.elem(&123u8));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_simple_add_expr_with_1_var() {
    let f = &Field::new(&3911u16);
    for s in vec!["x", "x1", "x0", "xy", "xy1"] {
      match Parser::parse(f, &format!("{}+456==123", s)) {
        Ok(eq) => {
          assert_eq!(eq.lhs, MathExpr::Add(1,
            Box::new(MathExpr::Var(s.to_string())),
            Box::new(MathExpr::Num(f.elem(&456u16))),
          ));
          assert_eq!(eq.rhs, f.elem(&123u8));
        },
        Err(_) => panic!(),
      }
    }
  }

  #[test]
  fn test_simple_add_expr_with_2_vars() {
    let f = &Field::new(&3911u16);
    for (a,b) in vec![("x", "y"), ("x1", "y1"), ("xxx1123", "yyy123443")] {
      match Parser::parse(f, &format!("{}+{}==123", a, b)) {
        Ok(eq) => {
          assert_eq!(eq.lhs, MathExpr::Add(1,
            Box::new(MathExpr::Var(a.to_string())),
            Box::new(MathExpr::Var(b.to_string())),
          ));
          assert_eq!(eq.rhs, f.elem(&123u8));
        },
        Err(_) => panic!(),
      }
    }
  }

  #[test]
  fn test_simple_add_expr_incl_neg() {
    let f = &Field::new(&3911u16);
    match Parser::parse(f, "123+-456==123") {
      Ok(eq) => {
        assert_eq!(eq.lhs, MathExpr::Add(1,
          Box::new(MathExpr::Num(f.elem(&123u16))),
          Box::new(MathExpr::Num(f.elem_from_signed(&-456))),
        ));
        assert_eq!(eq.rhs, f.elem(&123u8));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_simple_sub_expr() {
    let f = &Field::new(&3911u16);
    match Parser::parse(f, "123-456==123") {
      Ok(eq) => {
        assert_eq!(eq.lhs, MathExpr::Sub(1,
          Box::new(MathExpr::Num(f.elem(&123u16))),
          Box::new(MathExpr::Num(f.elem(&456u16))),
        ));
        assert_eq!(eq.rhs, f.elem(&123u8));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_simple_sub_expr_1_var() {
    let f = &Field::new(&3911u16);
    for s in vec!["x", "x1", "x0", "xy", "xy1"] {
      match Parser::parse(f, &format!("123-{}==123", s)) {
        Ok(eq) => {
          assert_eq!(eq.lhs, MathExpr::Sub(1,
            Box::new(MathExpr::Num(f.elem(&123u16))),
            Box::new(MathExpr::Var(s.to_string())),
          ));
          assert_eq!(eq.rhs, f.elem(&123u8));
        },
        Err(_) => panic!(),
      }
    }
  }

  #[test]
  fn test_simple_sub_expr_incl_neg1() {
    let f = &Field::new(&3911u16);
    match Parser::parse(f, "-123-456==123") {
      Ok(eq) => {
        assert_eq!(eq.lhs, MathExpr::Sub(1,
          Box::new(MathExpr::Num(f.elem_from_signed(&-123))),
          Box::new(MathExpr::Num(f.elem(&456u16))),
        ));
        assert_eq!(eq.rhs, f.elem(&123u8));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_simple_sub_expr_incl_neg1_1_var() {
    let f = &Field::new(&3911u16);
    for s in vec!["x", "x1", "x0", "xy", "xy1"] {
      match Parser::parse(f, &format!("-123-{}==123", s)) {
        Ok(eq) => {
          assert_eq!(eq.lhs, MathExpr::Sub(1,
            Box::new(MathExpr::Num(f.elem_from_signed(&-123))),
            Box::new(MathExpr::Var(s.to_string())),
          ));
          assert_eq!(eq.rhs, f.elem(&123u8));
        },
        Err(_) => panic!(),
      }
    }
  }

  #[test]
  fn test_simple_sub_expr_incl_neg2() {
    let f = &Field::new(&3911u16);
    match Parser::parse(f, "123--456==123") {
      Ok(eq) => {
        assert_eq!(eq.lhs, MathExpr::Sub(1,
          Box::new(MathExpr::Num(f.elem(&123u16))),
          Box::new(MathExpr::Num(f.elem_from_signed(&-456))),
        ));
        assert_eq!(eq.rhs, f.elem(&123u8));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_simple_sub_expr_incl_neg2_with_spaces() {
    let f = &Field::new(&3911u16);
    match Parser::parse(f, "123 - -456 == 123") {
      Ok(eq) => {
        assert_eq!(eq.lhs, MathExpr::Sub(1,
          Box::new(MathExpr::Num(f.elem(&123u16))),
          Box::new(MathExpr::Num(f.elem_from_signed(&-456))),
        ));
        assert_eq!(eq.rhs, f.elem(&123u8));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_simple_sub_expr_incl_neg2_with_spaces_1_var() {
    let f = &Field::new(&3911u16);
    match Parser::parse(f, "x - -456 == 123") {
      Ok(eq) => {
        assert_eq!(eq.lhs, MathExpr::Sub(1,
          Box::new(MathExpr::Var("x".to_string())),
          Box::new(MathExpr::Num(f.elem_from_signed(&-456))),
        ));
        assert_eq!(eq.rhs, f.elem(&123u8));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_simple_mul_expr() {
    let f = &Field::new(&3911u16);
    match Parser::parse(f, "123*456==123") {
      Ok(eq) => {
        assert_eq!(eq.lhs, MathExpr::Mul(1,
          Box::new(MathExpr::Num(f.elem(&123u16))),
          Box::new(MathExpr::Num(f.elem(&456u16))),
        ));
        assert_eq!(eq.rhs, f.elem(&123u8));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_simple_mul_expr_incl_neg1() {
    let f = &Field::new(&3911u16);
    match Parser::parse(f, "123*-456==123") {
      Ok(eq) => {
        assert_eq!(eq.lhs, MathExpr::Mul(1,
          Box::new(MathExpr::Num(f.elem(&123u16))),
          Box::new(MathExpr::Num(f.elem_from_signed(&-456)),
        )));
        assert_eq!(eq.rhs, f.elem(&123u8));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_simple_mul_expr_incl_neg2() {
    let f = &Field::new(&3911u16);
    match Parser::parse(f, "-123*456==123") {
      Ok(eq) => {
        assert_eq!(eq.lhs, MathExpr::Mul(1,
          Box::new(MathExpr::Num(f.elem_from_signed(&-123))),
          Box::new(MathExpr::Num(f.elem(&456u16))),
        ));
        assert_eq!(eq.rhs, f.elem(&123u8));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_simple_mul_expr_incl_neg() {
    let f = &Field::new(&3911u16);
    match Parser::parse(f, "123*-456==123") {
      Ok(eq) => {
        assert_eq!(eq.lhs, MathExpr::Mul(1,
          Box::new(MathExpr::Num(f.elem(&123u16))),
          Box::new(MathExpr::Num(f.elem_from_signed(&-456))),
        ));
        assert_eq!(eq.rhs, f.elem(&123u8));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_simple_div_expr() {
    let f = &Field::new(&3911u16);
    match Parser::parse(f, "123/456==123") {
      Ok(eq) => {
        assert_eq!(eq.lhs, MathExpr::Div(1,
          Box::new(MathExpr::Num(f.elem(&123u16))),
          Box::new(MathExpr::Num(f.elem(&456u16))),
        ));
        assert_eq!(eq.rhs, f.elem(&123u8));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_add_and_mul_expr() {
    let f = &Field::new(&3911u16);
    match Parser::parse(f, "123+456*789==123") {
      Ok(eq) => {
        assert_eq!(eq.lhs, MathExpr::Add(2,
          Box::new(MathExpr::Num(f.elem(&123u16))),
          Box::new(MathExpr::Mul(1,
            Box::new(MathExpr::Num(f.elem(&456u16))),
            Box::new(MathExpr::Num(f.elem(&789u16)))
          )),
        ));
        assert_eq!(eq.rhs, f.elem(&123u8));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_add_mul_div_expr() {
    let f = &Field::new(&3911u16);
    match Parser::parse(f, "111/222+333*444==123") {
      Ok(eq) => {
        assert_eq!(eq.lhs, MathExpr::Add(3,
          Box::new(MathExpr::Div(1,
            Box::new(MathExpr::Num(f.elem(&111u16))),
            Box::new(MathExpr::Num(f.elem(&222u16))),
          )),
          Box::new(MathExpr::Mul(2,
            Box::new(MathExpr::Num(f.elem(&333u16))),
            Box::new(MathExpr::Num(f.elem(&444u16))),
          )),
        ));
        assert_eq!(eq.rhs, f.elem(&123u8));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_paren_add_and_mul_expr() {
    let f = &Field::new(&3911u16);
    match Parser::parse(f, "(123+456)*789==123") {
      Ok(eq) => {
        assert_eq!(eq.lhs, MathExpr::Mul(2,
          Box::new(MathExpr::Add(1,
            Box::new(MathExpr::Num(f.elem(&123u16))),
            Box::new(MathExpr::Num(f.elem(&456u16))),
          )),
          Box::new(MathExpr::Num(f.elem(&789u16))),
        ));
        assert_eq!(eq.rhs, f.elem(&123u8));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_paren_add_and_mul_expr_with_spaces() {
    let f = &Field::new(&3911u16);
    match Parser::parse(f, " (123 + 456) * 789 == 123") {
      Ok(eq) => {
        assert_eq!(eq.lhs, MathExpr::Mul(2,
          Box::new(MathExpr::Add(1,
            Box::new(MathExpr::Num(f.elem(&123u16))),
            Box::new(MathExpr::Num(f.elem(&456u16))),
          )),
          Box::new(MathExpr::Num(f.elem(&789u16))),
        ));
        assert_eq!(eq.rhs, f.elem(&123u8));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_paren_add_mul_sub_expr() {
    let f = &Field::new(&3911u16);
    match Parser::parse(f, "(111+222)*(333-444)==123") {
      Ok(eq) => {
        assert_eq!(eq.lhs, MathExpr::Mul(3,
          Box::new(MathExpr::Add(1,
            Box::new(MathExpr::Num(f.elem(&111u16))),
            Box::new(MathExpr::Num(f.elem(&222u16))),
          )),
          Box::new(MathExpr::Sub(2,
            Box::new(MathExpr::Num(f.elem(&333u16))),
            Box::new(MathExpr::Num(f.elem(&444u16))),
          )),
        ));
        assert_eq!(eq.rhs, f.elem(&123u8));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_multiple_paren() {
    let f = &Field::new(&3911u16);
    match Parser::parse(f, "((111+222))==123") {
      Ok(eq) => {
        assert_eq!(eq.lhs, MathExpr::Add(1,
          Box::new(MathExpr::Num(f.elem(&111u8))),
          Box::new(MathExpr::Num(f.elem(&222u8))),
        ));
        assert_eq!(eq.rhs, f.elem(&123u8));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_multiple_paren_with_spaces() {
    let f = &Field::new(&3911u16);
    match Parser::parse(f, " ( (111+222) ) == 123") {
      Ok(eq) => {
        assert_eq!(eq.lhs, MathExpr::Add(1,
          Box::new(MathExpr::Num(f.elem(&111u8))),
          Box::new(MathExpr::Num(f.elem(&222u8))),
        ));
        assert_eq!(eq.rhs, f.elem(&123u8));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn blog_post_1_sample_1() {
    let f = &Field::new(&37u8);
    let expr = "(x * x * x) + x + 5 == 35";
    match Parser::parse(f, expr) {
      Ok(eq) => {
        println!("{} -> {:?}", expr, eq);
      },
      Err(_) => panic!(),
    }
  }
}