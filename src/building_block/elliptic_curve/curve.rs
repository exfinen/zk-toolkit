use crate::building_block::{
  field::Field,
  elliptic_curve::{
    ec_point::EcPoint,
    elliptic_curve_point_ops::{
      EllipticCurveField,
      EllipticCurvePointAdd,
      ElllipticCurvePointInv,
    }
  },
};

pub trait Curve<T>
where T: EllipticCurveField + EllipticCurvePointAdd + ElllipticCurvePointInv {
  fn get_generator(&self) -> EcPoint;
  fn get_curve_group(&self) -> Field;
  fn get_point_ops(&self) -> Box<T>;
}
