use crate::field::{Field, FieldElem};
use crate::vector_ops::EcPoint1;

#[derive(Debug, Clone)]
pub struct EcPoint {
  pub x: FieldElem,
  pub y: FieldElem,
  pub is_inf: bool,
}

impl PartialEq for EcPoint {
  fn eq(&self, other: &Self) -> bool {
    if self.is_inf != other.is_inf {
      false
    } else if self.is_inf {  // both is_inf's are true 
      true
    } else {  // both is_inf's are false 
      self.x == other.x && self.y == other.y
    }
  }
}

impl Eq for EcPoint {}

impl<'a> From<EcPoint1<'a>> for EcPoint {
  fn from(x: EcPoint1<'a>) -> EcPoint {
    x.0.1  
  }
}

impl EcPoint {
  pub fn inf() -> Self {
    let f = Field::new(&1u8);
    EcPoint {
      x: FieldElem::new(&f, &0u8),
      y: FieldElem::new(&f, &0u8),
      is_inf: true,
    }
  }

  pub fn new(x: &FieldElem, y: &FieldElem) -> Self {
    EcPoint { x: x.clone(), y: y.clone(), is_inf: false }
  }

  pub fn safe_new(x: &FieldElem, y: &FieldElem) -> Result<Self, String> {
    if x.f != y.f {
      return Err("Orders of field elements differ".to_string());
    }
    Ok(EcPoint { x: x.clone(), y: y.clone(), is_inf: false })
  }
}