use crate::building_block::field::prime_field_elem::PrimeFieldElem;

#[derive(Debug, PartialEq, Clone)]
pub enum ArithCircuit {
  Leaf(PrimeFieldElem),
  Mul(Box<ArithCircuit>, Box<ArithCircuit>),
  Add(Box<ArithCircuit>, Box<ArithCircuit>),
  Sub(Box<ArithCircuit>, Box<ArithCircuit>),
  Div(Box<ArithCircuit>, Box<ArithCircuit>),
}

pub struct Processor();
