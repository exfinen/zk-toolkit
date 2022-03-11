use crate::field_elem::FieldElem;

#[derive(Debug, Clone)]
pub enum EcPoint {
  Infinity(),
  Affine(AffineCoord)
}

#[derive(Debug, Clone)]
pub struct AffineCoord {
  pub x: FieldElem,
  pub y: FieldElem,
}

impl PartialEq for AffineCoord {
  fn eq(&self, other: &Self) -> bool {
    self.x == other.x && self.y == other.y
  }
}

impl Eq for AffineCoord {}

impl AffineCoord {
  pub fn new(x: FieldElem, y: FieldElem) -> Result<Self, String> {
    if x.f != y.f {
      return Err("Orders of field elements differ".to_string());
    }
    Ok(AffineCoord { x, y })
  }
}