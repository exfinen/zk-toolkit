use crate::building_block::{
  additive_identity::AdditiveIdentity,
  field::field_elem::NewFieldElem,
  elliptic_curve::{
    affine_point::AffinePoint,
    jacobian_point::JacobianPoint,
    new_affine_point::NewAffinePoint,
  },
  zero::Zero,
};

#[derive(Debug, Clone)]
pub struct EcPoint<E> {
  pub x: E,
  pub y: E,
  pub is_inf: bool,
}

impl<E> From<JacobianPoint<EcPoint<E>>> for EcPoint<E> {
  fn from(pt: JacobianPoint<EcPoint<E>>) -> Self {
    if pt.z.is_zero() {
      panic!("z is not expected to be zero");
    } else {
      let z2 = pt.z.sq();
      let z3 = &z2 * &pt.z;
      let x = &pt.x / z2;
      let y = &pt.y / z3;
      EcPoint { x, y, is_inf: false }
    }
  }
}

impl<F, E> Zero<EcPoint<E>> for EcPoint<E>
  where
    F: NewFieldElem<E>,
{
  fn get_zero(f: &F) -> EcPoint<E> {
      EcPoint {
        x: f.elem(&0u8),
        y: f.elem(&0u8),
        is_inf: true,
      }
  }

  fn is_zero(&self) -> bool {
      self.is_inf
  }
}

impl<E> AffinePoint<EcPoint<E>, E> for EcPoint<E> {
  fn x(&self) -> Self::E {
    self.x
  }
  fn y(&self) -> Self::E {
    self.y
  }
}

impl<E> NewAffinePoint<EcPoint<E>, E> for EcPoint<E>
  where E: Clone,
{
  fn new(x: &E, y: &E) -> Self {
    EcPoint {
      x: x.clone(),
      y: y.clone(),
      is_inf: false,
    }
  }
}

impl<E> PartialEq for EcPoint<E> {
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

impl<E> Eq for EcPoint<E> {}

// impl<Op, E> From<EcPointWithOps<Op, E>> for EcPoint<E>
//   where Op: EllipticCurveField + EllipticCurvePointAdd<EcPoint<E>, E> + ElllipticCurvePointInv<EcPoint<E>, E> {

//   fn from(x: EcPointWithOps<Op>) -> EcPoint<E> {
//     x.0.1
//   }
// }

impl<E> AdditiveIdentity<E> for EcPoint<E> {
  fn get_additive_identity() -> E {
    EcPoint {
      x: E::elem(&0u8),
      y: E::elem(&0u8),
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