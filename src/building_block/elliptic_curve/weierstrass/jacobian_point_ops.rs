use crate::building_block::{
  field::{Field, FieldElem},
  elliptic_curve::{
    ec_point::EcPoint,
    elliptic_curve_point_ops::{
      EllipticCurveField,
      EllipticCurvePointAdd,
      ElllipticCurvePointInv,
    },
  },
};
use num_bigint::BigUint;
use num_traits::identities::{One, Zero};

#[derive(Debug, Clone)]
struct JacobianPoint {
  pub x: FieldElem,
  pub y: FieldElem,
  pub z: FieldElem,
}

impl JacobianPoint {
  pub fn from_ec_point(pt: &EcPoint) -> Result<JacobianPoint, String> {
    if pt.is_inf {
      Err("Cannot convert inf to Jacobian point".to_string())
    } else {
      let pt = pt.clone();
      Ok(JacobianPoint {
        x: pt.x.clone(),
        y: pt.y,
        z: pt.x.f.elem(&BigUint::one()),
      })
    }
  }

  fn to_ec_point(&self) -> Result<EcPoint, String> {
    if self.z.n == BigUint::zero() {
      Err("z is not expected to be zero".to_string())
    } else {
      let z2 = self.z.sq();
      let z3 = &z2 * &self.z;
      let x = &self.x / z2;
      let y = &self.y / z3;
      Ok(EcPoint { x, y, is_inf: false })
    }
  }
}

#[derive(Clone)]
pub struct WeierstrassJacobianPointOps {
  f: Field,
}

impl WeierstrassJacobianPointOps {
  pub fn new(f: &Field) -> Self {
    Self { f: f.clone() }
  }
}

impl EllipticCurveField for WeierstrassJacobianPointOps {
  fn get_field(&self) -> &Field {
      &self.f
  }
}

impl EllipticCurvePointAdd for WeierstrassJacobianPointOps {
  fn add(&self, p1: &EcPoint, p2: &EcPoint) -> EcPoint {
    let f = self.get_field();

    if p1.is_inf && p2.is_inf {  // inf + inf is inf
      EcPoint::inf(f)
    } else if p1.is_inf {  // adding p2 to inf is p2
      p2.clone()
    } else if p2.is_inf {  // adding p1 to inf is p1
      p1.clone()
    } else if p1.x == p2.x && p1.y != p2.y {  // if line through p1 and p2 is vertical line
      EcPoint::inf(f)
    } else if p1.x == p2.x && p1.y == p2.y {  // if adding the same point
      // special case: if y == 0, the tangent line is vertical
      if p1.y.n == BigUint::zero() || p2.y.n == BigUint::zero() {
        return EcPoint::inf(f);
      }

      // formula described in: http://hyperelliptic.org/EFD/g1p/auto-shortw-jacobian-0.html#doubling-dbl-2009-l
      // w/ unnecessary computation removed
      let jp = JacobianPoint::from_ec_point(p1).unwrap();

      let a = &jp.x.sq();
      let b = &jp.y.sq();
      let c = &b.sq();
      let d = &(((jp.x + b).sq() - a - c) * 2u8);
      let e = &(a * 3u8);
      let f = &e.sq();
      let x3 = f - (d * 2u8);
      let y3 = e * (d - &x3) - (c * 8u8);
      let z3 = jp.y * 2u8;

      let jp2 = JacobianPoint {
        x: x3,
        y: y3,
        z: z3,
      };
      jp2.to_ec_point().unwrap()

    } else {  // when line through p1 and p2 is non-vertical line
      let jp1 = JacobianPoint::from_ec_point(p1).unwrap();
      let jp2 = JacobianPoint::from_ec_point(p2).unwrap();

      // formula described in: https://hyperelliptic.org/EFD/g1p/auto-shortw-jacobian-3.html#addition-add-2007-bl
      // w/ unnecessary computation removed
      let h = jp2.x - &jp1.x;
      let i = (&h * 2).sq();
      let j = &h * &i;
      let r = (jp2.y - &jp1.y) * 2u8;
      let v = jp1.x * &i;
      let x3 = (r.sq() - &j) - (&v * 2u8);
      let y3 = r * (v - &x3) - (jp1.y * (j * 2u8));
      let z3 = h * 2u8;

      let jp3 = JacobianPoint {
        x: x3,
        y: y3,
        z: z3,
      };
      jp3.to_ec_point().unwrap()
    }
  }
}

impl ElllipticCurvePointInv for WeierstrassJacobianPointOps {}
