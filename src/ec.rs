use super::field::{Field, FieldNum};
use num_bigint::BigUint;

pub struct Ec<'a> {
  f: &'a Field,
}

pub struct EcPoint<'a> {
  f: &'a Field,
  x: FieldNum<'a>,
  y: FieldNum<'a>,
}

impl <'a> Ec <'a> {
  pub fn new(f: &'a Field, p: BigUint) -> Self {
    Ec {
      f,
    }
  }

  pub fn add(self, p1: &'a EcPoint, p2: &'a EcPoint) -> EcPoint<'a> {
    // for now, assumes that p1 != p2
    let aa = p2.y.clone().sub(p1.y.clone());
    let a1 = p2.x.clone().sub(p1.x.clone());
    let m = aa.div(a1).unwrap();

    // equation of the line that intersects w/ the curve at p1 and p2:
    // y = m(x − x_1) + y_1
    // (m(x − x_1) + y_1)^2 = x3 + Ax + B
    // 0 = x^3 - m^2 x^2 + ...

    // x^3 + ax^2 + bx + c = (x-r)(x-s)(x-t) = x^3 - (r+s+t)x^2 ...
    // r + s + t = -a

    let mm = m.clone().mul(m.clone());

    // // the 3rd point the line intersects w/ the curve
    let mm_sub_p1x = mm.clone().sub(p1.x.clone());
    let x = mm_sub_p1x.sub(p2.x.clone());
    let y = m.clone().mul(x.sub(p1.x.clone())).add(p1.y.clone());

    // // reflect the 3rd point accross the x-axis 
    let cc = mm.clone().sub(p1.x.clone());
    let p3x = cc.sub(p2.x.clone());
    let bb = p1.x.clone().sub(p3x);
    let p3y = m.clone().mul(bb).sub(p1.y.clone());

    EcPoint {
      f: self.f,
      x: m.clone(),
      y: m.clone(),
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