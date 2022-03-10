use crate::field_elem::FieldElem;

#[derive(Debug, Clone)]
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
  pub fn new(x: FieldElem<'a>, y: FieldElem<'a>) -> Result<Self, String> {
    if x.f != y.f {
      return Err("Orders of field elements differ".to_string());
    }
    Ok(AffineCoord { x, y })
  }
}