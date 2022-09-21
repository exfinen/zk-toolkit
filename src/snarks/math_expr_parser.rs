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

#[derive(Debug, PartialEq, Clone)]
pub enum MathExpr {
  Num(String),
  Var(String),
  Mul(Box<MathExpr>, Box<MathExpr>),
  Div(Box<MathExpr>, Box<MathExpr>),
  Add(Box<MathExpr>, Box<MathExpr>),
  Sub(Box<MathExpr>, Box<MathExpr>),
}

pub struct MathExprParser();

impl MathExprParser {
  fn variable(input: &str) -> IResult<&str, MathExpr> {
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

    Ok((input, MathExpr::Num(s.to_string())))
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
          ('+', node) => MathExpr::Add(Box::new(acc), Box::new(node.clone())),
          ('-', node) => MathExpr::Sub(Box::new(acc), Box::new(node.clone())),
          (op, _) => panic!("unexpected operator encountered in expr: {}", op),
        }
      });

      let node = if rhs_head.0 == '+' {
        MathExpr::Add(Box::new(lhs), Box::new(rhs))
      } else {
        MathExpr::Sub(Box::new(lhs), Box::new(rhs))
      };
      Ok((input, node))
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
          ('*', node) => MathExpr::Mul(Box::new(acc), Box::new(node.clone())),
          ('/', node) => MathExpr::Div(Box::new(acc), Box::new(node.clone())),
          (op, _) => panic!("unexpected operator encountered in term1 {}", op),
        }
      });

      let node = if rhs_head.0 == '*' {
        MathExpr::Mul(Box::new(lhs), Box::new(rhs))
      } else {
        MathExpr::Div(Box::new(lhs), Box::new(rhs))
      };
      Ok((input, node))
    }
  }

  // <expr> ::= <term1> [ ('+'|'-') <term1> ]*
  // <term1> ::= <term2> [ ('*'|'/') <term2> ]*
  // <term2> ::= <variable> | <number> | '(' <expr> ')'
  pub fn parse(input: &str) -> IResult<&str, MathExpr> {
    MathExprParser::expr(input)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_decimal() {
    match MathExprParser::parse("123") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, MathExpr::Num("123".to_string()));
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