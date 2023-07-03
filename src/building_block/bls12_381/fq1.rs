use crate::building_block::field::FieldElem;
use std::ops::Add;

pub struct Fq1(FieldElem);

impl Fq1 {
  pub fn new(n: &FieldElem) -> Self {
      Fq1(n.clone())
  }
}

impl Add<Fq1> for Fq1 {
  type Output = Fq1;

  fn add(self, rhs: Fq1) -> Self::Output {
    Fq1(self.0 + rhs.0)
  }
}
