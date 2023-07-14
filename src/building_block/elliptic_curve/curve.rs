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
  fn g(&self) -> EcPoint;
  fn group(&self) -> Field;
  fn ops(&self) -> Box<T>;
  fn equation(&self) -> Box<U>;
}
