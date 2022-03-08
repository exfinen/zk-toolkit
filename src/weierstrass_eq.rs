use crate::field_elem::FieldElem;
use crate::ec_point::{EcPoint, AffineCoord};

// represents: y^2 = x^3 + Ax + B
#[allow(dead_code)]
pub struct WeierstrassEquation<'a> {
  pub a: FieldElem<'a>,
  pub b: FieldElem<'a>,
}

#[allow(dead_code)]
impl <'a> WeierstrassEquation <'a> {
  pub fn new(a: FieldElem<'a>, b: FieldElem<'a>) -> Result<Self, String> {
    if a.f != b.f {
      return Err("Orders of field elements differ".to_string());
    }
    Ok(WeierstrassEquation { a, b })
  }

  pub fn add(&self, p1: &'a EcPoint, p2: &'a EcPoint) -> EcPoint<'a> {
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
        // TO BE IMPLEMENTED
        EcPoint::Infinity()
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
        // starting from a curve equation of Wirestrass form:
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
        let p3x = (m.mul(&m)).sub(&p1.x).sub(&p2.x);

        // using (1), find the y-coordinate of the 3rd intersecting point and p3x obtained above
        // y = m(x − p1.x) + p1.y
        // p3.y = m(p3.x − p1.x) + p1.y
        let p3y = m.mul(&p3x.sub(&p1.x)).add(&p1.y);
        
        // then (p3.x, -p3.y) is the result of adding p1 and p2
        let p3y_neg = p3y.neg();
        
        EcPoint::Affine(AffineCoord {
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
  use crate::field::Field;

  #[test]
  fn test_secp256k1() {
    // in secp256k1, a = 0, b = 7 i.e. E: y^2 = x^3 + 0x + 7

    // field order
    let p = BigUint::parse_bytes(b"FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F", 16).unwrap();
    let f = Field::new(p);
    let a = f.element(BigUint::from(0u32));
    let b = f.element(BigUint::from(7u32));

    // curve
    let e = WeierstrassEquation::new(a, b).unwrap(); 

    // base point
    let gx = BigUint::parse_bytes(b"79BE667EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B16F81798", 16).unwrap();
    let gy = BigUint::parse_bytes(b"483ADA7726A3C4655DA4FBFC0E1108A8FD17B448A68554199C47D08FFB10D4B8", 16).unwrap();
    let g = EcPoint::Affine(
      AffineCoord::new(f.element(gx), f.element(gy))
    );
    //println!("{:?}", &g);

    let _gg = e.add(&g, &g);
    // order of base point
    //let n = BigUint::parse_bytes(b"FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141", 16).unwrap();

    //println!("x={}, y={}", gg.x.v, gg.y.v)
  }
}
