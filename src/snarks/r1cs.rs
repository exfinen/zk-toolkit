use crate::building_block::field::FieldElem;

pub enum R1CS {
  Leaf(FieldElem),
  Add(Box<R1CS>, Box<R1CS>),
  Mul(Box<R1CS>, Box<R1CS>),
}
