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

// making static because nom parsers cannot refer to self
static mut OPT_FIELD: Option<Field> = None;
static mut SIGNAL_ID: u128 = 0;

pub struct MathExprParser();

impl MathExprParser {
  fn num_str_to_field_elem(s: &str) -> FieldElem {
    unsafe {
      let field: &Field = OPT_FIELD.as_ref().unwrap();

      if s.starts_with("-") {
        let mut n = BigInt::parse_bytes(s.as_bytes(), 10).unwrap();
        if n.sign() == Sign::Minus {
          let order = BigInt::from_biguint(Sign::Plus, (*field.order).clone());
          n = -n;
          n = n % &order;
          n = &order - n;
          let n = n.to_biguint().unwrap();
          field.elem(&n)

        } else {
          let n = n.to_biguint().unwrap();
          field.elem(&n)
        }
      } else { // if positive
        let n = BigUint::parse_bytes(s.as_bytes(), 10).unwrap();
        field.elem(&n)
      }
    }
  }

  fn variable(input: &str) -> IResult<&str, MathExpr> {
    let (input, s) =
      delimited(
        multispace0,
        recognize(
          terminated(alpha1, many0(one_of("0123456789")))
        ),
        multispace0
      )(input)?;

    unsafe {
      SIGNAL_ID += 1;
      Ok((input, MathExpr::Var(SIGNAL_ID, s.to_string())))
    }
  }

  fn decimal(input: &str) -> IResult<&str, MathExpr> {
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

    let n = MathExprParser::num_str_to_field_elem(s);
    unsafe {
      SIGNAL_ID += 1;
      Ok((input, MathExpr::Num(SIGNAL_ID, n)))
    }
  }

  // <expr> ::= <term1> [ ('+'|'-') <term1> ]*
  fn expr(input: &str) -> IResult<&str, MathExpr> {
    let rhs = tuple((alt((char('+'), char('-'))), MathExprParser::term1));
    let (input, (lhs, rhs)) = tuple((
      MathExprParser::term1,
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
            unsafe {
              SIGNAL_ID += 1;
              MathExpr::Add(SIGNAL_ID, Box::new(acc), Box::new(node.clone()))
            }
          },
          ('-', node) => {
            unsafe {
              SIGNAL_ID += 1;
              MathExpr::Sub(SIGNAL_ID, Box::new(acc), Box::new(node.clone()))
            }
          },
          (op, _) => panic!("unexpected operator encountered in expr: {}", op),
        }
      });

      unsafe {
        SIGNAL_ID += 1;
        let node = if rhs_head.0 == '+' {
          MathExpr::Add(SIGNAL_ID, Box::new(lhs), Box::new(rhs))
        } else {
          MathExpr::Sub(SIGNAL_ID, Box::new(lhs), Box::new(rhs))
        };
        Ok((input, node))
      }
    }
  }

  // <term2> ::= <variable> | <number> | '(' <expr> ')'
  fn term2(input: &str) -> IResult<&str, MathExpr> {
    let (input, node) = alt((
      MathExprParser::variable,
      MathExprParser::decimal,
      delimited(
        delimited(multispace0, char('('), multispace0),
        MathExprParser::expr,
        delimited(multispace0, char(')'), multispace0),
      ),
    ))(input)?;

    Ok((input, node))
  }

  // <term1> ::= <term2> [ ('*'|'/') <term2> ]*
  fn term1(input: &str) -> IResult<&str, MathExpr> {
    let rhs = tuple((alt((char('*'), char('/'))), MathExprParser::term2));
    let (input, (lhs, rhs)) = tuple((
      MathExprParser::term2,
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
            unsafe {
              SIGNAL_ID += 1;
              MathExpr::Mul(SIGNAL_ID, Box::new(acc), Box::new(node.clone()))
            }
          },
          ('/', node) => {
            unsafe {
              SIGNAL_ID += 1;
              MathExpr::Div(SIGNAL_ID, Box::new(acc), Box::new(node.clone()))
            }
          },
          (op, _) => panic!("unexpected operator encountered in term1 {}", op),
        }
      });

      unsafe {
        SIGNAL_ID += 1;
        let node = if rhs_head.0 == '*' {
          MathExpr::Mul(SIGNAL_ID, Box::new(lhs), Box::new(rhs))
        } else {
          MathExpr::Div(SIGNAL_ID, Box::new(lhs), Box::new(rhs))
        };
        Ok((input, node))
      }
    }
  }

  // <expr> ::= <term1> [ ('+'|'-') <term1> ]*
  // <term1> ::= <term2> [ ('*'|'/') <term2> ]*
  // <term2> ::= <variable> | <number> | '(' <expr> ')'
  pub fn parse(input: &str, f: Field) -> IResult<&str, MathExpr> {
    // reset parser
    unsafe {
      OPT_FIELD = Some(f);
      SIGNAL_ID = 0;
    }
    MathExprParser::expr(input)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_decimal() {
    let f = Field::new(&11u8);
    match MathExprParser::parse("123", f) {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, MathExpr::Num(1, f.elem(&123u8)));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_decimal_with_spaces() {
    match MathExprParser::parse(" 123 ") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, MathExpr::Num("123".to_string()));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_neg_decimal() {
    match MathExprParser::parse("-123") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, MathExpr::Num("-123".to_string()));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_1_char_variable() {
    for s in vec!["x", "x1", "x0", "xy", "xy1"] {
      match MathExprParser::parse(s) {
        Ok((input, x)) => {
          assert_eq!(input, "");
          assert_eq!(x, MathExpr::Var(s.to_string()));
        },
        Err(_) => panic!(),
      }
    }
  }

  #[test]
  fn test_1_char_variable_with_spaces() {
    for s in vec!["x", "x1", "x0", "xy", "xy1"] {
      match MathExprParser::parse(&format!("  {}  ", s)) {
        Ok((input, x)) => {
          assert_eq!(input, "");
          assert_eq!(x, MathExpr::Var(s.to_string()));
        },
        Err(_) => panic!(),
      }
    }
  }

  #[test]
  fn test_simple_add_expr() {
    match MathExprParser::parse("123+456") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, MathExpr::Add(
          Box::new(MathExpr::Num("123".to_string())),
          Box::new(MathExpr::Num("456".to_string())),
        ));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_simple_add_expr_with_1_var() {
    for s in vec!["x", "x1", "x0", "xy", "xy1"] {
      match MathExprParser::parse(&format!("{}+456", s)) {
        Ok((input, x)) => {
          assert_eq!(input, "");
          assert_eq!(x, MathExpr::Add(
            Box::new(MathExpr::Var(s.to_string())),
            Box::new(MathExpr::Num("456".to_string())),
          ));
        },
        Err(_) => panic!(),
      }
    }
  }

  #[test]
  fn test_simple_add_expr_with_2_vars() {
    for (a,b) in vec![("x", "y"), ("x1", "y1"), ("xxx1123", "yyy123443")] {
      match MathExprParser::parse(&format!("{}+{}", a, b)) {
        Ok((input, x)) => {
          assert_eq!(input, "");
          assert_eq!(x, MathExpr::Add(
            Box::new(MathExpr::Var(a.to_string())),
            Box::new(MathExpr::Var(b.to_string())),
          ));
        },
        Err(_) => panic!(),
      }
    }
  }

  #[test]
  fn test_simple_add_expr_incl_neg() {
    match MathExprParser::parse("123+-456") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, MathExpr::Add(
          Box::new(MathExpr::Num("123".to_string())),
          Box::new(MathExpr::Num("-456".to_string())),
        ));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_simple_sub_expr() {
    match MathExprParser::parse("123-456") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, MathExpr::Sub(
          Box::new(MathExpr::Num("123".to_string())),
          Box::new(MathExpr::Num("456".to_string())),
        ));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_simple_sub_expr_1_var() {
    for s in vec!["x", "x1", "x0", "xy", "xy1"] {
      match MathExprParser::parse(&format!("123-{}", s)) {
        Ok((input, x)) => {
          assert_eq!(input, "");
          assert_eq!(x, MathExpr::Sub(
            Box::new(MathExpr::Num("123".to_string())),
            Box::new(MathExpr::Var(s.to_string())),
          ));
        },
        Err(_) => panic!(),
      }
    }
  }

  #[test]
  fn test_simple_sub_expr_incl_neg1() {
    match MathExprParser::parse("-123-456") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, MathExpr::Sub(
          Box::new(MathExpr::Num("-123".to_string())),
          Box::new(MathExpr::Num("456".to_string())),
        ));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_simple_sub_expr_incl_neg1_1_var() {
    for s in vec!["x", "x1", "x0", "xy", "xy1"] {
      match MathExprParser::parse(&format!("-123-{}", s)) {
        Ok((input, x)) => {
          assert_eq!(input, "");
          assert_eq!(x, MathExpr::Sub(
            Box::new(MathExpr::Num("-123".to_string())),
            Box::new(MathExpr::Var(s.to_string())),
          ));
        },
        Err(_) => panic!(),
      }
    }
  }
  #[test]
  fn test_simple_sub_expr_incl_neg2() {
    match MathExprParser::parse("123--456") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, MathExpr::Sub(
          Box::new(MathExpr::Num("123".to_string())),
          Box::new(MathExpr::Num("-456".to_string())),
        ));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_simple_sub_expr_incl_neg2_with_spaces() {
    match MathExprParser::parse("123 - -456") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, MathExpr::Sub(
          Box::new(MathExpr::Num("123".to_string())),
          Box::new(MathExpr::Num("-456".to_string())),
        ));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_simple_sub_expr_incl_neg2_with_spaces_1_var() {
    match MathExprParser::parse("x - -456") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, MathExpr::Sub(
          Box::new(MathExpr::Var("x".to_string())),
          Box::new(MathExpr::Num("-456".to_string())),
        ));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_simple_mul_expr() {
    match MathExprParser::parse("123*456") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, MathExpr::Mul(
          Box::new(MathExpr::Num("123".to_string())),
          Box::new(MathExpr::Num("456".to_string())),
        ));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_simple_mul_expr_incl_neg1() {
    match MathExprParser::parse("123*-456") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, MathExpr::Mul(
          Box::new(MathExpr::Num("123".to_string())),
          Box::new(MathExpr::Num("-456".to_string())),
        ));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_simple_mul_expr_incl_neg2() {
    match MathExprParser::parse("-123*456") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, MathExpr::Mul(
          Box::new(MathExpr::Num("-123".to_string())),
          Box::new(MathExpr::Num("456".to_string())),
        ));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_simple_mul_expr_incl_neg() {
    match MathExprParser::parse("123*-456") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, MathExpr::Mul(
          Box::new(MathExpr::Num("123".to_string())),
          Box::new(MathExpr::Num("-456".to_string())),
        ));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_simple_div_expr() {
    match MathExprParser::parse("123/456") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, MathExpr::Div(
          Box::new(MathExpr::Num("123".to_string())),
          Box::new(MathExpr::Num("456".to_string())),
        ));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_add_and_mul_expr() {
    match MathExprParser::parse("123+456*789") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, MathExpr::Add(
          Box::new(MathExpr::Num("123".to_string())),
          Box::new(MathExpr::Mul(
            Box::new(MathExpr::Num("456".to_string())),
            Box::new(MathExpr::Num("789".to_string()))
          )),
        ));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_add_mul_div_expr() {
    match MathExprParser::parse("111/222+333*444") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, MathExpr::Add(
          Box::new(MathExpr::Div(
            Box::new(MathExpr::Num("111".to_string())),
            Box::new(MathExpr::Num("222".to_string()))
          )),
          Box::new(MathExpr::Mul(
            Box::new(MathExpr::Num("333".to_string())),
            Box::new(MathExpr::Num("444".to_string()))
          )),
        ));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_paren_add_and_mul_expr() {
    match MathExprParser::parse("(123+456)*789") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, MathExpr::Mul(
          Box::new(MathExpr::Add(
            Box::new(MathExpr::Num("123".to_string())),
            Box::new(MathExpr::Num("456".to_string()))
          )),
          Box::new(MathExpr::Num("789".to_string())),
        ));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_paren_add_and_mul_expr_with_spaces() {
    match MathExprParser::parse(" (123 + 456) * 789") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, MathExpr::Mul(
          Box::new(MathExpr::Add(
            Box::new(MathExpr::Num("123".to_string())),
            Box::new(MathExpr::Num("456".to_string()))
          )),
          Box::new(MathExpr::Num("789".to_string())),
        ));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_paren_add_mul_sub_expr() {
    match MathExprParser::parse("(111+222)*(333-444)") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, MathExpr::Mul(
          Box::new(MathExpr::Add(
            Box::new(MathExpr::Num("111".to_string())),
            Box::new(MathExpr::Num("222".to_string()))
          )),
          Box::new(MathExpr::Sub(
            Box::new(MathExpr::Num("333".to_string())),
            Box::new(MathExpr::Num("444".to_string()))
          )),
        ));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_multiple_paren() {
    match MathExprParser::parse("((111+222))") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, MathExpr::Add(
          Box::new(MathExpr::Num("111".to_string())),
          Box::new(MathExpr::Num("222".to_string())),
        ));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_multiple_paren_with_spaces() {
    match MathExprParser::parse(" ( (111+222) ) ") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, MathExpr::Add(
          Box::new(MathExpr::Num("111".to_string())),
          Box::new(MathExpr::Num("222".to_string())),
        ));
      },
      Err(_) => panic!(),
    }
  }
}