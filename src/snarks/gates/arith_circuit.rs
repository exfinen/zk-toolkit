use crate::building_block::field::FieldElem;

#[derive(Debug, PartialEq, Clone)]
pub enum ArithCircuit {
  Leaf(FieldElem),
  Mul(Box<ArithCircuit>, Box<ArithCircuit>),
  Add(Box<ArithCircuit>, Box<ArithCircuit>),
}