use crate::building_block::curves::mcl::mcl_fr::MclFr;

#[derive(Debug, PartialEq, Clone)]
pub enum ArithCircuit {
  Leaf(MclFr),
  Mul(Box<ArithCircuit>, Box<ArithCircuit>),
  Add(Box<ArithCircuit>, Box<ArithCircuit>),
  Sub(Box<ArithCircuit>, Box<ArithCircuit>),
  Div(Box<ArithCircuit>, Box<ArithCircuit>),
}

pub struct Processor();
