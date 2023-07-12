use crate::building_block::elliptic_curve::ec_point::EcPoint;

pub trait CurveEquation {
  fn is_rational_point(&self, pt: &EcPoint) -> bool;
}
