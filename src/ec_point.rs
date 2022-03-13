use crate::field_elem::FieldElem;

#[derive(Debug, Clone)]
pub enum EcPoint {
  Infinity(),
  Affine(Coord2)
}

#[derive(Debug, Clone)]
pub struct Coord2 {
  pub x: FieldElem,
  pub y: FieldElem,
}

impl PartialEq for Coord2 {
  fn eq(&self, other: &Self) -> bool {
    self.x == other.x && self.y == other.y
  }
}

impl Eq for Coord2 {}

impl Coord2 {
  pub fn new(x: FieldElem, y: FieldElem) -> Result<Self, String> {
    if x.f != y.f {
      return Err("Orders of field elements differ".to_string());
    }
    Ok(Coord2 { x, y })
  }
}