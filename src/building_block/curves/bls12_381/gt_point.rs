use crate::building_block::curves::bls12_381::fq12::Fq12;
use std::ops::Mul;

pub struct GTPoint {
  e: Fq12,
}

impl GTPoint {
  pub fn new(e: &Fq12) -> Self {
    GTPoint {
      e: e.clone(),
    }
  }
}

macro_rules! impl_mul {
  ($rhs: ty, $target: ty) => {
    impl Mul<$rhs> for $target {
      type Output = GTPoint;

      fn mul(self, rhs: $rhs) -> Self::Output {
        let e = &self.e * &rhs.e;
        GTPoint::new(&e)
      }
    }
  };
}
impl_mul!(GTPoint, GTPoint);
impl_mul!(GTPoint, &GTPoint);
impl_mul!(&GTPoint, GTPoint);
impl_mul!(&GTPoint, &GTPoint);

impl PartialEq for GTPoint {
  fn eq(&self, other: &Self) -> bool {
    &self.e == &other.e
  }
}

impl Eq for GTPoint {}
