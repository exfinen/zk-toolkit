use num_bigint::{BigInt, BigUint, Sign};
use crate::building_block::field::{Field, FieldElem};
use crate::snarks::math_expr_parser::MathExpr;

type SignalId = u128;

#[derive(Debug, PartialEq, Clone)]
pub enum ArithCircuit {
  Num(SignalId, FieldElem),
  Mul(SignalId, Box<ArithCircuit>, Box<ArithCircuit>),
  Add(SignalId, Box<ArithCircuit>, Box<ArithCircuit>),
}

pub struct ArithCircuitParser {
  f: Field,
}

impl ArithCircuitParser {
  pub fn new(f: Field) -> ArithCircuitParser {
    ArithCircuitParser { f }
  }

  pub fn num_str_to_field_elem(&self, s: &str) -> FieldElem {
    if s.starts_with("-") {
      let mut n = BigInt::parse_bytes(s.as_bytes(), 10).unwrap();
      if n.sign() == Sign::Minus {
        let order = BigInt::from_biguint(Sign::Plus, (*self.f.order).clone());
        n = -n;
        n = n % &order;
        n = &order - n;
        let n = n.to_biguint().unwrap();
        self.f.elem(&n)

      } else {
        let n = n.to_biguint().unwrap();
        self.f.elem(&n)
      }
    } else { // if positive
      let n = BigUint::parse_bytes(s.as_bytes(), 10).unwrap();
      self.f.elem(&n)
    }
  }

  pub fn parse(&self, _curr_signal_id: u128, _expr: MathExpr) -> ArithCircuit {
    ArithCircuit::Num(1u128, self.f.elem(&0u8))
  }
}



#[cfg(test)]
mod tests {
  //use super::*;

  #[test]
  fn test_num_str_to_field_elem_zero() {
  }

  #[test]
  fn test_num_str_to_field_elem_positive() {
  }

  #[test]
  fn test_num_str_to_field_elem_negative() {
  }
}