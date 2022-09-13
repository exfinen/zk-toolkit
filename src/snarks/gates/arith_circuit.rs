use crate::building_block::field::{Field, FieldElem};
use crate::snarks::r1cs::R1CS;

#[derive(Debug, PartialEq, Clone)]
pub enum ArithCircuit {
  Leaf(FieldElem),
  Mul(Box<ArithCircuit>, Box<ArithCircuit>),
  Add(Box<ArithCircuit>, Box<ArithCircuit>),
  Sub(Box<ArithCircuit>, Box<ArithCircuit>),
  Div(Box<ArithCircuit>, Box<ArithCircuit>),
}

pub struct Processor();

impl Processor {
  pub fn to_r1cs(f: Field) -> R1CS {
    R1CS::Leaf(f.elem(&0u8))
  }
}