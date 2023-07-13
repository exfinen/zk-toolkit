use crate::building_block::{
  field::Field,
  elliptic_curve::{
    curve_equation::CurveEquation,
    ec_point::EcPoint,
    elliptic_curve_point_ops::{
      EllipticCurveField,
      EllipticCurvePointAdd,
      ElllipticCurvePointInv,
    },
  },
};

pub trait Curve<T, U>
where T: EllipticCurveField + EllipticCurvePointAdd + ElllipticCurvePointInv, U: CurveEquation {
  fn get_generator(&self) -> EcPoint;
  fn get_curve_group(&self) -> Field;
  fn get_point_ops(&self) -> Box<T>;
  fn get_equation(&self) -> Box<U>;
}
