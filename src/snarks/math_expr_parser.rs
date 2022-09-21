use nom::{
  IResult,
  branch::alt,
  character::complete::{ char, one_of, multispace0 },
  combinator::{ opt, recognize },
  multi::{ many0, many1 },
  sequence::{ tuple, delimited, terminated },
};

#[derive(Debug, PartialEq, Clone)]
pub enum MathExprAst {
  Num(String),
  Mul(Box<MathExprAst>, Box<MathExprAst>),
  Div(Box<MathExprAst>, Box<MathExprAst>),
  Add(Box<MathExprAst>, Box<MathExprAst>),
  Sub(Box<MathExprAst>, Box<MathExprAst>),
}

pub struct MathExprParser();

impl MathExprParser {
  fn decimal(input: &str) -> IResult<&str, MathExprAst> {
    let (input, s) =
      delimited(
        multispace0,
        recognize(
          tuple((
            opt(char('-')),
            many1(
              terminated(one_of("0123456789"), many0(char('_')))
            ),
          )),
        ),
        multispace0
      )(input)?;

    Ok((input, MathExprAst::Num(s.to_string())))
  }

  // <expr> ::= <term1> [ ('+'|'-') <term1> ]*
  fn expr(input: &str) -> IResult<&str, MathExprAst> {
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
          ('+', node) => MathExprAst::Add(Box::new(acc), Box::new(node.clone())),
          ('-', node) => MathExprAst::Sub(Box::new(acc), Box::new(node.clone())),
          (op, _) => panic!("unexpected operator encountered in expr: {}", op),
        }
      });

      let node = if rhs_head.0 == '+' {
        MathExprAst::Add(Box::new(lhs), Box::new(rhs))
      } else {
        MathExprAst::Sub(Box::new(lhs), Box::new(rhs))
      };
      Ok((input, node))
    }
  }

  // <term2> ::= <number> | '(' <expr> ')'
  fn term2(input: &str) -> IResult<&str, MathExprAst> {
    let (input, node) = alt((
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
  fn term1(input: &str) -> IResult<&str, MathExprAst> {
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
          ('*', node) => MathExprAst::Mul(Box::new(acc), Box::new(node.clone())),
          ('/', node) => MathExprAst::Div(Box::new(acc), Box::new(node.clone())),
          (op, _) => panic!("unexpected operator encountered in term1 {}", op),
        }
      });

      let node = if rhs_head.0 == '*' {
        MathExprAst::Mul(Box::new(lhs), Box::new(rhs))
      } else {
        MathExprAst::Div(Box::new(lhs), Box::new(rhs))
      };
      Ok((input, node))
    }
  }

  // <expr> ::= <term1> [ ('+'|'-') <term1> ]*
  // <term1> ::= <term2> [ ('*'|'/') <term2> ]*
  // <term2> ::= <number> | '(' <expr> ')'
  pub fn parse(input: &str) -> IResult<&str, MathExprAst> {
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
        assert_eq!(x, MathExprAst::Num("123".to_string()));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_decimal_with_spaces() {
    match MathExprParser::parse(" 123 ") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, MathExprAst::Num("123".to_string()));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_neg_decimal() {
    match MathExprParser::parse("-123") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, MathExprAst::Num("-123".to_string()));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_simple_add_expr() {
    match MathExprParser::parse("123+456") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, MathExprAst::Add(
          Box::new(MathExprAst::Num("123".to_string())),
          Box::new(MathExprAst::Num("456".to_string())),
        ));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_simple_add_expr_incl_neg() {
    match MathExprParser::parse("123+-456") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, MathExprAst::Add(
          Box::new(MathExprAst::Num("123".to_string())),
          Box::new(MathExprAst::Num("-456".to_string())),
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
        assert_eq!(x, MathExprAst::Sub(
          Box::new(MathExprAst::Num("123".to_string())),
          Box::new(MathExprAst::Num("456".to_string())),
        ));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_simple_sub_expr_incl_neg1() {
    match MathExprParser::parse("-123-456") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, MathExprAst::Sub(
          Box::new(MathExprAst::Num("-123".to_string())),
          Box::new(MathExprAst::Num("456".to_string())),
        ));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_simple_sub_expr_incl_neg2() {
    match MathExprParser::parse("123--456") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, MathExprAst::Sub(
          Box::new(MathExprAst::Num("123".to_string())),
          Box::new(MathExprAst::Num("-456".to_string())),
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
        assert_eq!(x, MathExprAst::Sub(
          Box::new(MathExprAst::Num("123".to_string())),
          Box::new(MathExprAst::Num("-456".to_string())),
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
        assert_eq!(x, MathExprAst::Mul(
          Box::new(MathExprAst::Num("123".to_string())),
          Box::new(MathExprAst::Num("456".to_string())),
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
        assert_eq!(x, MathExprAst::Mul(
          Box::new(MathExprAst::Num("123".to_string())),
          Box::new(MathExprAst::Num("-456".to_string())),
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
        assert_eq!(x, MathExprAst::Mul(
          Box::new(MathExprAst::Num("-123".to_string())),
          Box::new(MathExprAst::Num("456".to_string())),
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
        assert_eq!(x, MathExprAst::Mul(
          Box::new(MathExprAst::Num("123".to_string())),
          Box::new(MathExprAst::Num("-456".to_string())),
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
        assert_eq!(x, MathExprAst::Div(
          Box::new(MathExprAst::Num("123".to_string())),
          Box::new(MathExprAst::Num("456".to_string())),
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
        assert_eq!(x, MathExprAst::Add(
          Box::new(MathExprAst::Num("123".to_string())),
          Box::new(MathExprAst::Mul(
            Box::new(MathExprAst::Num("456".to_string())),
            Box::new(MathExprAst::Num("789".to_string()))
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
        assert_eq!(x, MathExprAst::Add(
          Box::new(MathExprAst::Div(
            Box::new(MathExprAst::Num("111".to_string())),
            Box::new(MathExprAst::Num("222".to_string()))
          )),
          Box::new(MathExprAst::Mul(
            Box::new(MathExprAst::Num("333".to_string())),
            Box::new(MathExprAst::Num("444".to_string()))
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
        assert_eq!(x, MathExprAst::Mul(
          Box::new(MathExprAst::Add(
            Box::new(MathExprAst::Num("123".to_string())),
            Box::new(MathExprAst::Num("456".to_string()))
          )),
          Box::new(MathExprAst::Num("789".to_string())),
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
        assert_eq!(x, MathExprAst::Mul(
          Box::new(MathExprAst::Add(
            Box::new(MathExprAst::Num("123".to_string())),
            Box::new(MathExprAst::Num("456".to_string()))
          )),
          Box::new(MathExprAst::Num("789".to_string())),
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
        assert_eq!(x, MathExprAst::Mul(
          Box::new(MathExprAst::Add(
            Box::new(MathExprAst::Num("111".to_string())),
            Box::new(MathExprAst::Num("222".to_string()))
          )),
          Box::new(MathExprAst::Sub(
            Box::new(MathExprAst::Num("333".to_string())),
            Box::new(MathExprAst::Num("444".to_string()))
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
        assert_eq!(x, MathExprAst::Add(
          Box::new(MathExprAst::Num("111".to_string())),
          Box::new(MathExprAst::Num("222".to_string())),
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
        assert_eq!(x, MathExprAst::Add(
          Box::new(MathExprAst::Num("111".to_string())),
          Box::new(MathExprAst::Num("222".to_string())),
        ));
      },
      Err(_) => panic!(),
    }
  }
}