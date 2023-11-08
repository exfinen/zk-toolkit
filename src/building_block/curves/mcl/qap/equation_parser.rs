use nom::{
  IResult,
  branch::alt,
  bytes::complete::tag,
  character::complete::{ alpha1, char, multispace0, one_of },
  combinator::{ opt, recognize },
  multi::{ many0, many1 },
  sequence::{ tuple, delimited, terminated },
};
use crate::building_block::curves::mcl::mcl_fr::MclFr;
use crate::zk::w_trusted_setup::qap::config::SignalId;
use std::cell::Cell;

#[derive(Debug, PartialEq, Clone)]
pub enum MathExpr {
  Equation(Box<MathExpr>, Box<MathExpr>),
  Num(MclFr),
  Var(String),
  Mul(SignalId, Box<MathExpr>, Box<MathExpr>),
  Add(SignalId, Box<MathExpr>, Box<MathExpr>),
  Div(SignalId, Box<MathExpr>, Box<MathExpr>),
  Sub(SignalId, Box<MathExpr>, Box<MathExpr>),
}

#[derive(Debug)]
pub struct Equation {
  pub lhs: MathExpr,
  pub rhs: MclFr,
}

pub struct EquationParser();

macro_rules! set_next_id {
  ($signal_id: expr) => {
    $signal_id.set($signal_id.get() + 1);
  };
}

impl EquationParser {
  fn num_str_to_field_elem(s: &str) -> MclFr {
    MclFr::from(s)
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

  fn decimal() -> impl Fn(&str) -> IResult<&str, MathExpr> {
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

      let n = EquationParser::num_str_to_field_elem(s);
      Ok((input, MathExpr::Num(n)))
    }
  }

  // <term2> ::= <variable> | <number> | '(' <expr> ')'
  fn term2<'a>(signal_id: &'a Cell<u128>) -> impl Fn(&str) -> IResult<&str, MathExpr> + 'a {
    |input| {
      let (input, node) = alt((
        EquationParser::variable(),
        EquationParser::decimal(),
        delimited(
          delimited(multispace0, char('('), multispace0),
          EquationParser::expr(signal_id),
          delimited(multispace0, char(')'), multispace0),
        ),
      ))(input)?;

      Ok((input, node))
    }
  }

  // <term1> ::= <term2> [ ('*'|'/') <term2> ]*
  fn term1<'a>(signal_id: &'a Cell<u128>) -> impl Fn(&str) -> IResult<&str, MathExpr> + 'a {
    |input| {
      let rhs = tuple((alt((char('*'), char('/'))), EquationParser::term2(signal_id)));
      let (input, (lhs, rhs)) = tuple((
        EquationParser::term2(signal_id),
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
  fn expr<'a>(signal_id: &'a Cell<u128>) -> impl Fn(&str) -> IResult<&str, MathExpr> + 'a {
    |input| {
      let rhs = tuple((alt((char('+'), char('-'))), EquationParser::term1(signal_id)));
      let (input, (lhs, rhs)) = tuple((
        EquationParser::term1(signal_id),
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
  fn equation<'a>(signal_id: &'a Cell<u128>) -> impl Fn(&str) -> IResult<&str, MathExpr> + 'a {
    |input| {
      let (input, out) =
        tuple((
          multispace0,
          EquationParser::expr(signal_id),
          multispace0,
          tag("=="),
          multispace0,
          EquationParser::decimal(),
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
  pub fn parse<'a>(input: &'a str) -> Result<Equation, String> {
    let signal_id = Cell::new(0);
    let expr = EquationParser::equation(&signal_id);
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
  use crate::building_block::curves::mcl::mcl_initializer::MclInitializer;

  #[test]
  fn test_decimal() {
    MclInitializer::init();
    match EquationParser::parse("123 == 123") {
      Ok(eq) => {
        assert_eq!(eq.lhs, MathExpr::Num(MclFr::from(123)));
        assert_eq!(eq.rhs, MclFr::from(123));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_decimal_with_spaces() {
    MclInitializer::init();
    match EquationParser::parse(" 123 == 123 ") {
      Ok(eq) => {
        assert_eq!(eq.lhs, MathExpr::Num(MclFr::from(123)));
        assert_eq!(eq.rhs, MclFr::from(123));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_neg_decimal_below_order() {
    MclInitializer::init();
    match EquationParser::parse("-123 == -123") {
      Ok(eq) => {
        assert_eq!(eq.lhs, MathExpr::Num(MclFr::from(-123)));
        assert_eq!(eq.rhs, MclFr::from(-123));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_neg_decimal_above_order() {
    MclInitializer::init();
    match EquationParser::parse("-123 == -123") {
      Ok(eq) => {
        assert_eq!(eq.lhs, MathExpr::Num(MclFr::from(-123)));
        assert_eq!(eq.rhs, MclFr::from(-123));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_1_char_variable() {
    MclInitializer::init();
    for s in vec!["x", "x1", "x0", "xy", "xy1"] {
      match EquationParser::parse(&format!("{} == 123", s)) {
        Ok(eq) => {
          assert_eq!(eq.lhs, MathExpr::Var(s.to_string()));
          assert_eq!(eq.rhs, MclFr::from(123));
        },
        Err(_) => panic!(),
      }
    }
  }

  #[test]
  fn test_1_char_variable_with_spaces() {
    MclInitializer::init();
    for s in vec!["x", "x1", "x0", "xy", "xy1"] {
      match EquationParser::parse(&format!("  {} == 123  ", s)) {
        Ok(eq) => {
          assert_eq!(eq.lhs, MathExpr::Var(s.to_string()));
          assert_eq!(eq.rhs, MclFr::from(123));
        },
        Err(_) => panic!(),
      }
    }
  }

  #[test]
  fn test_simple_add_expr() {
    MclInitializer::init();
    match EquationParser::parse("123+456==123") {
      Ok(eq) => {
        assert_eq!(eq.lhs, MathExpr::Add(1,
          Box::new(MathExpr::Num(MclFr::from(123))),
          Box::new(MathExpr::Num(MclFr::from(456))),
        ));
        assert_eq!(eq.rhs, MclFr::from(123));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_simple_add_expr_with_1_var() {
    MclInitializer::init();
    for s in vec!["x", "x1", "x0", "xy", "xy1"] {
      match EquationParser::parse(&format!("{}+456==123", s)) {
        Ok(eq) => {
          assert_eq!(eq.lhs, MathExpr::Add(1,
            Box::new(MathExpr::Var(s.to_string())),
            Box::new(MathExpr::Num(MclFr::from(456))),
          ));
          assert_eq!(eq.rhs, MclFr::from(123));
        },
        Err(_) => panic!(),
      }
    }
  }

  #[test]
  fn test_simple_add_expr_with_2_vars() {
    MclInitializer::init();
    for (a,b) in vec![("x", "y"), ("x1", "y1"), ("xxx1123", "yyy123443")] {
      match EquationParser::parse(&format!("{}+{}==123", a, b)) {
        Ok(eq) => {
          assert_eq!(eq.lhs, MathExpr::Add(1,
            Box::new(MathExpr::Var(a.to_string())),
            Box::new(MathExpr::Var(b.to_string())),
          ));
          assert_eq!(eq.rhs, MclFr::from(123));
        },
        Err(_) => panic!(),
      }
    }
  }

  #[test]
  fn test_simple_add_expr_incl_neg() {
    MclInitializer::init();
    match EquationParser::parse("123+-456==123") {
      Ok(eq) => {
        assert_eq!(eq.lhs, MathExpr::Add(1,
          Box::new(MathExpr::Num(MclFr::from(123))),
          Box::new(MathExpr::Num(MclFr::from(-456))),
        ));
        assert_eq!(eq.rhs, MclFr::from(123));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_simple_sub_expr() {
    MclInitializer::init();
    match EquationParser::parse("123-456==123") {
      Ok(eq) => {
        assert_eq!(eq.lhs, MathExpr::Sub(1,
          Box::new(MathExpr::Num(MclFr::from(123))),
          Box::new(MathExpr::Num(MclFr::from(456))),
        ));
        assert_eq!(eq.rhs, MclFr::from(123));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_simple_sub_expr_1_var() {
    MclInitializer::init();
    for s in vec!["x", "x1", "x0", "xy", "xy1"] {
      match EquationParser::parse(&format!("123-{}==123", s)) {
        Ok(eq) => {
          assert_eq!(eq.lhs, MathExpr::Sub(1,
            Box::new(MathExpr::Num(MclFr::from(123))),
            Box::new(MathExpr::Var(s.to_string())),
          ));
          assert_eq!(eq.rhs, MclFr::from(123));
        },
        Err(_) => panic!(),
      }
    }
  }

  #[test]
  fn test_simple_sub_expr_incl_neg1() {
    MclInitializer::init();
    match EquationParser::parse("-123-456==123") {
      Ok(eq) => {
        assert_eq!(eq.lhs, MathExpr::Sub(1,
          Box::new(MathExpr::Num(MclFr::from(-123))),
          Box::new(MathExpr::Num(MclFr::from(456))),
        ));
        assert_eq!(eq.rhs, MclFr::from(123));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_simple_sub_expr_incl_neg1_1_var() {
    MclInitializer::init();
    for s in vec!["x", "x1", "x0", "xy", "xy1"] {
      match EquationParser::parse(&format!("-123-{}==123", s)) {
        Ok(eq) => {
          assert_eq!(eq.lhs, MathExpr::Sub(1,
            Box::new(MathExpr::Num(MclFr::from(-123))),
            Box::new(MathExpr::Var(s.to_string())),
          ));
          assert_eq!(eq.rhs, MclFr::from(123));
        },
        Err(_) => panic!(),
      }
    }
  }

  #[test]
  fn test_simple_sub_expr_incl_neg2() {
    MclInitializer::init();
    match EquationParser::parse("123--456==123") {
      Ok(eq) => {
        assert_eq!(eq.lhs, MathExpr::Sub(1,
          Box::new(MathExpr::Num(MclFr::from(123))),
          Box::new(MathExpr::Num(MclFr::from(-456))),
        ));
        assert_eq!(eq.rhs, MclFr::from(123));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_simple_sub_expr_incl_neg2_with_spaces() {
    MclInitializer::init();
    match EquationParser::parse("123 - -456 == 123") {
      Ok(eq) => {
        assert_eq!(eq.lhs, MathExpr::Sub(1,
          Box::new(MathExpr::Num(MclFr::from(123))),
          Box::new(MathExpr::Num(MclFr::from(-456))),
        ));
        assert_eq!(eq.rhs, MclFr::from(123));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_simple_sub_expr_incl_neg2_with_spaces_1_var() {
    MclInitializer::init();
    match EquationParser::parse("x - -456 == 123") {
      Ok(eq) => {
        assert_eq!(eq.lhs, MathExpr::Sub(1,
          Box::new(MathExpr::Var("x".to_string())),
          Box::new(MathExpr::Num(MclFr::from(-456))),
        ));
        assert_eq!(eq.rhs, MclFr::from(123));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_simple_mul_expr() {
    MclInitializer::init();
    match EquationParser::parse("123*456==123") {
      Ok(eq) => {
        assert_eq!(eq.lhs, MathExpr::Mul(1,
          Box::new(MathExpr::Num(MclFr::from(123))),
          Box::new(MathExpr::Num(MclFr::from(456))),
        ));
        assert_eq!(eq.rhs, MclFr::from(123));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_simple_mul_expr_incl_neg1() {
    MclInitializer::init();
    match EquationParser::parse("123*-456==123") {
      Ok(eq) => {
        assert_eq!(eq.lhs, MathExpr::Mul(1,
          Box::new(MathExpr::Num(MclFr::from(123))),
          Box::new(MathExpr::Num(MclFr::from(-456))),
        ));
        assert_eq!(eq.rhs, MclFr::from(123));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_simple_mul_expr_incl_neg2() {
    MclInitializer::init();
    match EquationParser::parse("-123*456==123") {
      Ok(eq) => {
        assert_eq!(eq.lhs, MathExpr::Mul(1,
          Box::new(MathExpr::Num(MclFr::from(-123))),
          Box::new(MathExpr::Num(MclFr::from(456))),
        ));
        assert_eq!(eq.rhs, MclFr::from(123));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_simple_mul_expr_incl_neg() {
    MclInitializer::init();
    match EquationParser::parse("123*-456==123") {
      Ok(eq) => {
        assert_eq!(eq.lhs, MathExpr::Mul(1,
          Box::new(MathExpr::Num(MclFr::from(123))),
          Box::new(MathExpr::Num(MclFr::from(-456))),
        ));
        assert_eq!(eq.rhs, MclFr::from(123));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_simple_div_expr() {
    MclInitializer::init();
    match EquationParser::parse("123/456==123") {
      Ok(eq) => {
        assert_eq!(eq.lhs, MathExpr::Div(1,
          Box::new(MathExpr::Num(MclFr::from(123))),
          Box::new(MathExpr::Num(MclFr::from(456))),
        ));
        assert_eq!(eq.rhs, MclFr::from(123));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_add_and_mul_expr() {
    MclInitializer::init();
    match EquationParser::parse("123+456*789==123") {
      Ok(eq) => {
        assert_eq!(eq.lhs, MathExpr::Add(2,
          Box::new(MathExpr::Num(MclFr::from(123))),
          Box::new(MathExpr::Mul(1,
            Box::new(MathExpr::Num(MclFr::from(456))),
            Box::new(MathExpr::Num(MclFr::from(789))),
          )),
        ));
        assert_eq!(eq.rhs, MclFr::from(123));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_add_mul_div_expr() {
    MclInitializer::init();
    match EquationParser::parse("111/222+333*444==123") {
      Ok(eq) => {
        assert_eq!(eq.lhs, MathExpr::Add(3,
          Box::new(MathExpr::Div(1,
            Box::new(MathExpr::Num(MclFr::from(111))),
            Box::new(MathExpr::Num(MclFr::from(222))),
          )),
          Box::new(MathExpr::Mul(2,
            Box::new(MathExpr::Num(MclFr::from(333))),
            Box::new(MathExpr::Num(MclFr::from(444))),
          )),
        ));
        assert_eq!(eq.rhs, MclFr::from(123));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_paren_add_and_mul_expr() {
    MclInitializer::init();
    match EquationParser::parse("(123+456)*789==123") {
      Ok(eq) => {
        assert_eq!(eq.lhs, MathExpr::Mul(2,
          Box::new(MathExpr::Add(1,
            Box::new(MathExpr::Num(MclFr::from(123))),
            Box::new(MathExpr::Num(MclFr::from(456))),
          )),
          Box::new(MathExpr::Num(MclFr::from(789))),
        ));
        assert_eq!(eq.rhs, MclFr::from(123));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_paren_add_and_mul_expr_with_spaces() {
    MclInitializer::init();
    match EquationParser::parse(" (123 + 456) * 789 == 123") {
      Ok(eq) => {
        assert_eq!(eq.lhs, MathExpr::Mul(2,
          Box::new(MathExpr::Add(1,
            Box::new(MathExpr::Num(MclFr::from(123))),
            Box::new(MathExpr::Num(MclFr::from(456))),
          )),
          Box::new(MathExpr::Num(MclFr::from(789))),
        ));
        assert_eq!(eq.rhs, MclFr::from(123));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_paren_add_mul_sub_expr() {
    MclInitializer::init();
    match EquationParser::parse("(111+222)*(333-444)==123") {
      Ok(eq) => {
        assert_eq!(eq.lhs, MathExpr::Mul(3,
          Box::new(MathExpr::Add(1,
            Box::new(MathExpr::Num(MclFr::from(111))),
            Box::new(MathExpr::Num(MclFr::from(222))),
          )),
          Box::new(MathExpr::Sub(2,
            Box::new(MathExpr::Num(MclFr::from(333))),
            Box::new(MathExpr::Num(MclFr::from(444))),
          )),
        ));
        assert_eq!(eq.rhs, MclFr::from(123));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_multiple_paren() {
    MclInitializer::init();
    match EquationParser::parse("((111+222))==123") {
      Ok(eq) => {
        assert_eq!(eq.lhs, MathExpr::Add(1,
          Box::new(MathExpr::Num(MclFr::from(111))),
          Box::new(MathExpr::Num(MclFr::from(222))),
        ));
        assert_eq!(eq.rhs, MclFr::from(123));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_multiple_paren_with_spaces() {
    MclInitializer::init();
    match EquationParser::parse(" ( (111+222) ) == 123") {
      Ok(eq) => {
        assert_eq!(eq.lhs, MathExpr::Add(1,
          Box::new(MathExpr::Num(MclFr::from(111))),
          Box::new(MathExpr::Num(MclFr::from(222))),
        ));
        assert_eq!(eq.rhs, MclFr::from(123));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn blog_post_1_example_1() {
    MclInitializer::init();
    let expr = "(x * x * x) + x + 5 == 35";
    match EquationParser::parse(expr) {
      Ok(eq) => {
        println!("{} -> {:?}", expr, eq);
      },
      Err(_) => panic!(),
    }
  }
}
