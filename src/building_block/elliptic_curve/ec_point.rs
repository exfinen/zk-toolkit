use crate::building_block::{
  additive_identity::AdditiveIdentity,
  field::{
    field_elem_ops::Inverse,
    prime_field_elem::PrimeFieldElem,
    prime_field::PrimeField,
  },
  elliptic_curve::{
    curve::Curve,
    jacobian_point::JacobianPoint,
  },
  zero::Zero,
};
use std::ops::Add;

#[derive(Debug, Clone)]
pub struct EcPoint {
  pub curve: Box<dyn Curve<EcPoint, PrimeFieldElem, PrimeField>>,
  pub x: PrimeFieldElem,
  pub y: PrimeFieldElem,
  pub is_inf: bool,
}

impl From<JacobianPoint> for EcPoint {
  fn from(pt: JacobianPoint) -> Self {
    if pt.z.is_zero() {
      panic!("z is not expected to be zero");
    } else {
      let z2 = pt.z.sq();
      let z3 = &z2 * &pt.z;
      let x = &pt.x / z2;
      let y = &pt.y / z3;
      EcPoint {
        curve: pt.curve,
        x,
        y,
        is_inf: false,
      }
    }
  }
}

impl Add for EcPoint {
  type Output = EcPoint;

  fn add(self, rhs: Self) -> Self::Output {
    rhs   // TODO implement this
  }
}

impl Inverse for EcPoint {
  fn inv(&self) -> Self {
    if self.is_inf {
      panic!("Cannot calculate the inverse of zero");
    }
    EcPoint {
      curve: self.curve,
      x: self.x.clone(),
      y: self.y.inv(),
      is_inf: false,
    }
  }
}

impl Zero<EcPoint> for EcPoint {
  fn get_zero(t: &EcPoint) -> EcPoint {
      EcPoint {
        curve: t.curve,
        x: t.x.get_additive_identity(),
        y: t.x.get_additive_identity(),
        is_inf: true,
      }
  }

  fn is_zero(&self) -> bool {
      self.is_inf
  }
}

impl AdditiveIdentity<PrimeFieldElem> for EcPoint {
  fn get_additive_identity(&self) -> PrimeFieldElem {
    self.x.get_additive_identity()
  }
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

// impl<Op, E> From<EcPointWithOps<Op, E>> for EcPoint<E>
//   where Op: EllipticCurveField + EllipticCurvePointAdd<EcPoint<E>, E> + ElllipticCurvePointInv<EcPoint<E>, E> {

//   fn from(x: EcPointWithOps<Op>) -> EcPoint<E> {
//     x.0.1
//   }
// }

impl AdditiveIdentity<EcPoint> for EcPoint {
  fn get_additive_identity(&self) -> EcPoint {
    EcPoint {
      curve: self.curve,
      x: self.x.get_zero(),
      y: self.x.get_zero(),
      is_inf: true,
    }
  }
}

  // pub fn safe_new(x: &E, y: &E) -> Result<Self, String> where E: Clone {
  //   if x.f != y.f {
  //     return Err("Orders of field elements differ".to_string());
  //   }
  //   Ok(EcPoint::new(x, y))
  // }