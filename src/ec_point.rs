use crate::field_elem::FieldElem;

pub enum EcPoint<'a> {
  Infinity(),
  Affine(AffineCoord<'a>)
}

#[derive(Debug, Clone)]
pub struct AffineCoord<'a> {
  pub x: FieldElem<'a>,
  pub y: FieldElem<'a>,
}

impl <'a> PartialEq for AffineCoord<'a> {
  fn eq(&self, other: &Self) -> bool {
    self.x == other.x && self.y == other.y
  }
}

impl <'a> Eq for AffineCoord<'a> {}

impl <'a> AffineCoord<'a> {
  pub fn new(x: FieldElem<'a>, y: FieldElem<'a>) -> Self {
    AffineCoord { x, y }
  }
}