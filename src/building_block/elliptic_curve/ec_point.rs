use crate::building_block::{
  additive_identity::AdditiveIdentity,
  field::{
    prime_field_elem::PrimeFieldElem,
  },
  elliptic_curve::{
    affine_point::AffinePoint,
    jacobian_point::JacobianPoint,
    new_affine_point::NewAffinePoint,
  },
  zero::Zero,
};

#[derive(Debug, Clone)]
pub struct EcPoint {
  pub x: PrimeFieldElem,
  pub y: PrimeFieldElem,
  pub is_inf: bool,
}

impl From<JacobianPoint<EcPoint>> for EcPoint {
  fn from(pt: JacobianPoint<EcPoint>) -> Self {
    if pt.z.is_zero() {
      panic!("z is not expected to be zero");
    } else {
      let z2 = pt.z.sq();
      let z3 = &z2 * &pt.z;
      let x = &pt.x / z2;
      let y = &pt.y / z3;
      EcPoint {
        x,
        y,
        is_inf: false,
      }
    }
  }
}

impl Zero<EcPoint> for EcPoint {
  fn get_zero(t: &EcPoint) -> EcPoint {
      EcPoint {
        x: t.x.get_additive_identity(),
        y: t.x.get_additive_identity(),
        is_inf: true,
      }
  }

  fn is_zero(&self) -> bool {
      self.is_inf
  }
}

impl AffinePoint<EcPoint, PrimeFieldElem> for EcPoint {
  fn x(&self) -> PrimeFieldElem {
    self.x
  }
  fn y(&self) -> PrimeFieldElem {
    self.y
  }
}

impl NewAffinePoint<EcPoint, PrimeFieldElem> for EcPoint {
  fn new(x: &PrimeFieldElem, y: &PrimeFieldElem) -> Self {
    EcPoint {
      x: x.clone(),
      y: y.clone(),
      is_inf: false,
    }
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

impl AdditiveIdentity for EcPoint {
  fn get_additive_identity(&self) -> Self {
    EcPoint {
      x: self.get_zero(),
      y: self.get_zero(),
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