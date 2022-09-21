use nom::{
  IResult,
  branch::alt,
  character::complete::{ char, one_of, multispace0 },
  combinator::{ opt, recognize },
  multi::{ many0, many1 },
  sequence::{ tuple, delimited, terminated },
};

#[derive(Debug, PartialEq, Clone)]
pub enum Ast {
  Num(i32),
  Mul(Box<Ast>, Box<Ast>),
  Div(Box<Ast>, Box<Ast>),
  Add(Box<Ast>, Box<Ast>),
  Sub(Box<Ast>, Box<Ast>),
}

pub struct Parser();

impl Parser {
  fn decimal(input: &str) -> IResult<&str, Ast> {
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

    let n: i32 = s.parse().unwrap();
    Ok((input, Ast::Num(n)))
  }

  // <expr> ::= <term1> [ ('+'|'-') <term1> ]*
  fn expr(input: &str) -> IResult<&str, Ast> {
    let rhs = tuple((alt((char('+'), char('-'))), Parser::term1));
    let (input, (lhs, rhs)) = tuple((
      Parser::term1,
      many0(rhs),
    ))(input)?;

    if rhs.len() == 0 {
      Ok((input, lhs))
    } else {
      // translate rhs vector to Add<Add<..,Add>>>..
      let rhs_head = &rhs[0];
      let rhs = rhs.iter().skip(1).fold(rhs_head.1.clone(), |acc, x| {
        match x {
          ('+', node) => Ast::Add(Box::new(acc), Box::new(node.clone())),
          ('-', node) => Ast::Sub(Box::new(acc), Box::new(node.clone())),
          (op, _) => panic!("unexpected operator encountered in expr: {}", op),
        }
      });

      let node = if rhs_head.0 == '+' {
        Ast::Add(Box::new(lhs), Box::new(rhs))
      } else {
        Ast::Sub(Box::new(lhs), Box::new(rhs))
      };
      Ok((input, node))
    }
  }

  // <term2> ::= <number> | '(' <expr> ')'
  fn term2(input: &str) -> IResult<&str, Ast> {
    let (input, node) = alt((
      Parser::decimal,
      delimited(
        delimited(multispace0, char('('), multispace0),
        Parser::expr,
        delimited(multispace0, char(')'), multispace0),
      ),
    ))(input)?;

    Ok((input, node))
  }

  // <term1> ::= <term2> [ ('*'|'/') <term2> ]*
  fn term1(input: &str) -> IResult<&str, Ast> {
    let rhs = tuple((alt((char('*'), char('/'))), Parser::term2));
    let (input, (lhs, rhs)) = tuple((
      Parser::term2,
      many0(rhs),
    ))(input)?;

    if rhs.len() == 0 {
      Ok((input, lhs))
    } else {
      // translate rhs vector to Mul<Mul<..,Mul>>>..
      let rhs_head = &rhs[0];
      let rhs = rhs.iter().skip(1).fold(rhs_head.1.clone(), |acc, x| {
        match x {
          ('*', node) => Ast::Mul(Box::new(acc), Box::new(node.clone())),
          ('/', node) => Ast::Div(Box::new(acc), Box::new(node.clone())),
          (op, _) => panic!("unexpected operator encountered in term1 {}", op),
        }
      });

      let node = if rhs_head.0 == '*' {
        Ast::Mul(Box::new(lhs), Box::new(rhs))
      } else {
        Ast::Div(Box::new(lhs), Box::new(rhs))
      };
      Ok((input, node))
    }
  }

  // <expr> ::= <term1> [ ('+'|'-') <term1> ]*
  // <term1> ::= <term2> [ ('*'|'/') <term2> ]*
  // <term2> ::= <number> | '(' <expr> ')'
  pub fn parse(input: &str) -> IResult<&str, Ast> {
    Parser::expr(input)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_decimal() {
    match Parser::parse("123") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, Ast::Num(123));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_decimal_with_spaces() {
    match Parser::parse(" 123 ") {
      Ok((input, x)) => { 
        assert_eq!(input, "");
        assert_eq!(x, Ast::Num(123)); 
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_neg_decimal() {
    match Parser::parse("-123") {
      Ok((input, x)) => { 
        assert_eq!(input, "");
        assert_eq!(x, Ast::Num(-123)); 
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_simple_add_expr() {
    match Parser::parse("123+456") {
      Ok((input, x)) => { 
        assert_eq!(input, "");
        assert_eq!(x, Ast::Add(
          Box::new(Ast::Num(123)), 
          Box::new(Ast::Num(456)),
        )); 
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_simple_add_expr_incl_neg() {
    match Parser::parse("123+-456") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, Ast::Add(
          Box::new(Ast::Num(123)),
          Box::new(Ast::Num(-456)),
        ));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_simple_sub_expr() {
    match Parser::parse("123-456") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, Ast::Sub(
          Box::new(Ast::Num(123)),
          Box::new(Ast::Num(456)),
        ));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_simple_sub_expr_incl_neg1() {
    match Parser::parse("-123-456") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, Ast::Sub(
          Box::new(Ast::Num(-123)),
          Box::new(Ast::Num(456)),
        ));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_simple_sub_expr_incl_neg2() {
    match Parser::parse("123--456") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, Ast::Sub(
          Box::new(Ast::Num(123)),
          Box::new(Ast::Num(-456)),
        ));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_simple_sub_expr_incl_neg2_with_spaces() {
    match Parser::parse("123 - -456") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, Ast::Sub(
          Box::new(Ast::Num(123)),
          Box::new(Ast::Num(-456)),
        ));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_simple_mul_expr() {
    match Parser::parse("123*456") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, Ast::Mul(
          Box::new(Ast::Num(123)),
          Box::new(Ast::Num(456)),
        ));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_simple_mul_expr_incl_neg1() {
    match Parser::parse("123*-456") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, Ast::Mul(
          Box::new(Ast::Num(123)),
          Box::new(Ast::Num(-456)),
        ));
      },
      Err(_) => panic!(),
    }
  }


  #[test]
  fn test_simple_mul_expr_incl_neg2() {
    match Parser::parse("-123*456") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, Ast::Mul(
          Box::new(Ast::Num(-123)),
          Box::new(Ast::Num(456)),
        ));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_simple_mul_expr_incl_neg() {
    match Parser::parse("123*-456") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, Ast::Mul(
          Box::new(Ast::Num(123)),
          Box::new(Ast::Num(-456)),
        ));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_simple_div_expr() {
    match Parser::parse("123/456") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, Ast::Div(
          Box::new(Ast::Num(123)),
          Box::new(Ast::Num(456)),
        ));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_add_and_mul_expr() {
    match Parser::parse("123+456*789") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, Ast::Add(
          Box::new(Ast::Num(123)),
          Box::new(Ast::Mul(
            Box::new(Ast::Num(456)),
            Box::new(Ast::Num(789))
          )),
        ));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_add_mul_div_expr() {
    match Parser::parse("111/222+333*444") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, Ast::Add(
          Box::new(Ast::Div(
            Box::new(Ast::Num(111)),
            Box::new(Ast::Num(222))
          )),
          Box::new(Ast::Mul(
            Box::new(Ast::Num(333)),
            Box::new(Ast::Num(444))
          )),
        ));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_paren_add_and_mul_expr() {
    match Parser::parse("(123+456)*789") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, Ast::Mul(
          Box::new(Ast::Add(
            Box::new(Ast::Num(123)),
            Box::new(Ast::Num(456))
          )),
          Box::new(Ast::Num(789)),
        ));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_paren_add_and_mul_expr_with_spaces() {
    match Parser::parse(" (123 + 456) * 789") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, Ast::Mul(
          Box::new(Ast::Add(
            Box::new(Ast::Num(123)),
            Box::new(Ast::Num(456))
          )),
          Box::new(Ast::Num(789)),
        ));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_paren_add_mul_sub_expr() {
    match Parser::parse("(111+222)*(333-444)") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, Ast::Mul(
          Box::new(Ast::Add(
            Box::new(Ast::Num(111)),
            Box::new(Ast::Num(222))
          )),
          Box::new(Ast::Sub(
            Box::new(Ast::Num(333)),
            Box::new(Ast::Num(444))
          )),
        ));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_multiple_paren() {
    match Parser::parse("((111+222))") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, Ast::Add(
          Box::new(Ast::Num(111)),
          Box::new(Ast::Num(222)),
        ));
      },
      Err(_) => panic!(),
    }
  }

  #[test]
  fn test_multiple_paren_with_spaces() {
    match Parser::parse(" ( (111+222) ) ") {
      Ok((input, x)) => {
        assert_eq!(input, "");
        assert_eq!(x, Ast::Add(
          Box::new(Ast::Num(111)),
          Box::new(Ast::Num(222)),
        ));
      },
      Err(_) => panic!(),
    }
  }
}