use crate::field_elem::FieldElem;
use crate::field::Field;
use crate::ec_point::{EcPoint, Coord2};
use num_bigint::BigUint;
use num_traits::identities::{Zero, One};
use std::ops::{BitAnd, ShrAssign};
use std::rc::Rc;

// represents: y^2 = x^3 + Ax + B
#[allow(dead_code)]
pub struct WeierstrassEq {
  pub f: Rc<Field>,
  pub a: FieldElem,
  pub b: FieldElem,
  pub g: EcPoint,
  pub zero: BigUint,
  pub one: BigUint,
}

#[allow(dead_code)]
impl WeierstrassEq {
  pub fn new(
    f: Rc<Field>, 
    a: BigUint, 
    b: BigUint, 
    gx: BigUint, 
    gy: BigUint,
  ) -> Result<Self, String> {
    let a = FieldElem::new(f.clone(), a);
    let b = FieldElem::new(f.clone(), b);
    let g = EcPoint::Affine(Coord2::new(
      FieldElem::new(f.clone(), gx), 
      FieldElem::new(f.clone(), gy),
    ).unwrap());
    let zero = BigUint::zero();
    let one = BigUint::one();

    Ok(WeierstrassEq { f, a, b, g, zero, one })
  }

  pub fn secp256k1() -> WeierstrassEq {
    let p = BigUint::parse_bytes(b"FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F", 16).unwrap();
    let f = Field::new(p);

    let a = BigUint::from(0u32);
    let b = BigUint::from(7u32);

    // base point
    let gx = BigUint::parse_bytes(b"79BE667EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B16F81798", 16).unwrap();
    let gy = BigUint::parse_bytes(b"483ADA7726A3C4655DA4FBFC0E1108A8FD17B448A68554199C47D08FFB10D4B8", 16).unwrap();

    // curve
    WeierstrassEq::new(f, a, b, gx, gy).unwrap()
  }

  pub fn scalar_mul(&self, multiplier: &BigUint) -> EcPoint {
    let mut n = multiplier.clone();
    let mut res = EcPoint::Infinity();
    let mut factor = self.g.clone();
    let two = BigUint::from(2u32);

    while !n.is_zero() {
      if n.clone().bitand(&two).is_one() {
        res = self.add(&res, &factor);
        factor = self.add(&factor, &factor);
      }
      n.shr_assign(1usize);
    }
    res
  }

  pub fn add(&self, p1: &EcPoint, p2: &EcPoint) -> EcPoint {
    match (p1, p2) {
      // when adding point at infinity to a point
      (EcPoint::Infinity(), EcPoint::Affine(p)) => {
        EcPoint::Affine(p.clone())
      },
      (EcPoint::Affine(p), EcPoint::Infinity()) => {
        EcPoint::Affine(p.clone())
      },
      (EcPoint::Infinity(), EcPoint::Infinity()) => {
        EcPoint::Infinity()
      },
      // when line through p1 and p2 is vertical line
      (EcPoint::Affine(p1), EcPoint::Affine(p2)) if p1.x == p2.x && p1.y != p2.y => {
        EcPoint::Infinity()
      },
      // when p1 and p2 are the same point
      (EcPoint::Affine(p1), EcPoint::Affine(p2)) if p1.x == p2.x && p1.y == p2.y => {
        // special case: if y == 0, the tangent line is vertical
        if p1.y.v == BigUint::zero() {
          return EcPoint::Infinity();
        }
        // differentiate y^2 = x^3 + Ax + B w/ implicit differentiation
        // d/dx(y^2) = d/dx(x^3 + Ax + B)
        // 2y dy/dx = 3x^2 + A
        // dy/dx = (3x^2 + A) / 2y
        //
        // dy/dx is the slope m of the tangent line at the point 
        // m = (3x^2 + A) / 2y
        let m1 = p1.x.sq().mul_u32(3u32);
        let m2 = p1.y.mul_u32(2u32);
        let m = m1.div(&m2).unwrap();

        // equation of intersecting line is
        // y = m(x − p1.x) + p1.y (1)
        //
        // substitute y with (1):
        // (m(x − p1.x) + p1.y)^2 = x^3 + Ax + B
        //
        // moving LHS to RHS, we get:
        // 0 = x^3 - m^2 x^2 + ...  (2)
        //
        // with below equation:
        // (x - r)(x - s)(x - t) = x^3 + (r + s + t)x^2 + (ab + ac + bc)x − abc 
        // 
        // we know that the coefficient of x^2 term is:
        // r + s + t 
        //
        // using (2), the coefficient of x^2 term of the intersecting line is:
        // m^2 = r + s + t
        // 
        // since p1 and p2 are the same point, replace r and s w/ p1.x
        // to get the x-coordinate of the point where (1) intersects the curve
        // x3 = m^2 − 2*p1.x
        let p3x = m.sq().sub(&p1.x.mul_u32(2u32));

        // then get the y-coordinate by substituting x in (1) w/ x3 to get y3
        // y3 = m(x3 − p1.x) + p1.y 
        // 
        // reflecting y3 across the x-axis results in the addition result y-coordinate 
        // result.y = -1 * y3 = m(p1.x - x3) - p1.y
        let p3y_neg = m.mul(&p1.x.sub(&p3x)).sub(&p1.y);
        
        EcPoint::Affine(Coord2 {
          x: p3x,
          y: p3y_neg,
        })
      },
      // when line through p1 and p2 is non-vertical line
      (EcPoint::Affine(p1), EcPoint::Affine(p2)) => {

        // slope m of the line that intersects the curve at p1 and p2:
        // p2.y - p1.y = m(p2.x - p1.x)
        // m(p2.x - p1.x) = p2.y - p1.y
        // m = (p2.y - p1.y) / (p2.x - p1.x)
        let m = (p2.y.sub(&p1.y)).div(&p2.x.sub(&p1.x)).unwrap();

        // then the equation of the line is:
        // y = m(x − p1.x) + p1.y  (1)
        //
        // starting from a curve equation of Weierstrass form:
        // y^2 = x^3 + Ax + B
        //
        // substitute y with (1):
        // (m(x − p1.x) + p1.y)^2 = x^3 + Ax + B
        //
        // moving LHS to RHS, we get:
        // 0 = x^3 - m^2 x^2 + ...  (2)
        //
        // with below equation:
        // (x - r)(x - s)(x - t) = x^3 + (r + s + t)x^2 + (ab + ac + bc)x − abc 
        // 
        // we know that the coefficient of x^2 term is:
        // r + s + t 
        //
        // using (2), the coefficient of x^2 term of the intersecting line is:
        // m^2 = r + s + t
        // 
        // substitute r and s with the known 2 roots - p1.x and p2.x:
        // m^2 = p1.x + p2. + t
        // t = m^2 - p1.x - p2.x
        //
        // here t is the x coordinate of the p3 we're trying to find:
        // p3.x = m^2 - p1.x - p2.x
        let p3x = m.sq().sub(&p1.x).sub(&p2.x);

        // using (1), find the y-coordinate of the 3rd intersecting point and p3x obtained above
        // y = m(x − p1.x) + p1.y
        // p3.y = m(p3.x − p1.x) + p1.y
        let p3y = m.mul(&p3x.sub(&p1.x)).add(&p1.y);
        
        // then (p3.x, -p3.y) is the result of adding p1 and p2
        let p3y_neg = p3y.neg();
        
        EcPoint::Affine(Coord2 {
          x: p3x,
          y: p3y_neg,
        })
      },
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use num_bigint::BigUint;

  #[test]
  fn test_scalar_mul_same_point() {
    let e = WeierstrassEq::secp256k1();
    let g2 = e.add(&e.g, &e.g);
    let exp_x = BigUint::parse_bytes(b"89565891926547004231252920425935692360644145829622209833684329913297188986597", 10).unwrap();
    let exp_y = BigUint::parse_bytes(b"12158399299693830322967808612713398636155367887041628176798871954788371653930", 10).unwrap();
    match g2 {
      EcPoint::Affine(c) => {
        assert_eq!(c.x.v, exp_x);
        assert_eq!(c.y.v, exp_y);
      },
      _ => {
        panic!("Didn't get affine point");
      }
    }
  }

  #[test]
  fn test_secp256k1() {
    // in secp256k1, a = 0, b = 7 i.e. E: y^2 = x^3 + 0x + 7

    // order of base point
    //let n = BigUint::parse_bytes(b"FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141", 16).unwrap();
  }
}
