use super::field::{Field, FieldNum};
use num_bigint::BigUint;

// represents: y^2 = x^3 + Ax + B
#[allow(dead_code)]
pub struct WeierstrassEquation<'a> {
  f: &'a Field,
  A: FieldNum<'a>,
  B: FuekdNum<'a>,
}

#[allow(dead_code)]
pub struct EcPoint<'a> {
  f: &'a Field,
  x: FieldNum<'a>,
  y: FieldNum<'a>,
}

impl <'a> WeierstrassEquation <'a> {
  #[allow(dead_code)]
  pub fn new(f: &'a Field, A: BigUint, B: BigUint) -> Self {
    WeierstrassEquation { f, A, B }
  }

  #[allow(dead_code)]
  pub fn add(&self, p1: &'a EcPoint, p2: &'a EcPoint) -> EcPoint<'a> {
    // for now, this code assumes that p1 != p2

    // slope m of the line that intersects the curve at p1 and p2:
    // p2.y - p1.y = m(p2.x - p1.x)
    // m(p2.x - p1.x) = p2.y - p1.y
    // m = (p2.y - p1.y) / (p2.x - p1.x)
    //
    // then the equation of the line is:
    // y = m(x − p1.x) + p1.y  (1)
    //
    // given a curve equation of Wirestrass form:
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
    // replace r and s with the known 2 roots - p1.x and p2.x:
    // m^2 = p1.x + p2. + t
    // t = m^2 - p1.x - p2.x
    //
    // here t is the x coordinate of the p3 we're trying to find:
    // p3.x = m^2 - p1.x - p2.x
    // 
    // find the y-coordinate of the 3rd intersecting point by substituting x of (1):
    // y = m(x − p1.x) + p1.y
    // p3.y = m(p3.x − p1.x) + p1.y
    // p3.y = m((m^2 - p1.x - p2.x) − p1.x) + p1.y
    //
    // then (p3.x, -p3.y) the result of adding p1 and p2

    let t = -1.mul(&(p1.x.add(&p2.x).add(self.A)));
    
    // then substitute x of the line's eq w/ thex-coordinate of the 3rd point 
    // to calculate the y-coordinate and flip the point in terms of the x-axis
    // 
    // y = m(x − x_1) + y_1
    
    // slope of the line intesecting the curve
    
    let m = (p2.y.sub(&p1.y)).div(&p2.x.sub(&p1.x)).unwrap();
    let rhs = m.mul(&p3.x).sub((m.mul(&p1.x))).add(&p1.y)

    let mm = m.mul(&m);

    let x = mm.sub(&p1.x).sub(&p2.x);

    // the 3rd point that the line intersects the curve
    let _y = m.mul(&(mm.sub(&p1.x).sub(&p2.x)).sub(&p1.x).add(&p1.y));

    // reflect the 3rd point across the x-axis 
    let p3y = mm.mul(&p1.x.sub(&(mm.sub(&p1.x)).sub(&p2.x))).sub(&p1.y);

    EcPoint {
      f: self.f,
      x,
      y: p3y,
    } 
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_secp256k1() {
    // a = 0, b = 7
    // E: y^2 = x^3 + 0x + 7
    
    // field order
    let p = BigUint::parse_bytes(b"FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F", 16).unwrap();

    // base point G
    let g_x = BigUint::parse_bytes(b"79BE667EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B16F81798", 16).unwrap();
    let g_y = BigUint::parse_bytes(b"483ADA7726A3C4655DA4FBFC0E1108A8FD17B448A68554199C47D08FFB10D4B8", 16).unwrap(); 

    // order of G
    let n = BigUint::parse_bytes(b"FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141", 16).unwrap();

    println!("p={} n={}", p, n);

  }
}