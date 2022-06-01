use crate::field_elem::FieldElem;
use crate::field::Field;
use crate::ec_point::EcPoint;
use crate::curve::Curve;
use num_bigint::BigUint;
use num_traits::identities::{Zero, One};

// y^2 = x^3 + Ax + B
pub struct WeierstrassEq {
  pub f: Field,
  pub a: FieldElem,
  pub b: FieldElem,
  pub g: EcPoint,
  pub n: BigUint,
  pub zero: BigUint,
  pub one: BigUint,
}

impl WeierstrassEq {
  pub fn new(
    f: Field, 
    a: BigUint, 
    b: BigUint, 
    gx: BigUint, 
    gy: BigUint,
    n: BigUint,
  ) -> Result<Self, String> {
    let a = FieldElem::new(f.clone(), a);
    let b = FieldElem::new(f.clone(), b);
    let g = EcPoint::new(
      FieldElem::new(f.clone(), gx), 
      FieldElem::new(f.clone(), gy),
    ).unwrap();
    let zero = BigUint::zero();
    let one = BigUint::one();

    Ok(WeierstrassEq { f, a, b, g, n, zero, one })
  }

  pub fn secp256k1() -> WeierstrassEq {
    let p = BigUint::parse_bytes(b"FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F", 16).unwrap();
    let f = Field::new(p);

    let a = BigUint::from(0u32);
    let b = BigUint::from(7u32);

    // base point
    let gx = BigUint::parse_bytes(b"79BE667EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B16F81798", 16).unwrap();
    let gy = BigUint::parse_bytes(b"483ADA7726A3C4655DA4FBFC0E1108A8FD17B448A68554199C47D08FFB10D4B8", 16).unwrap();

    // order of base point
    let n = BigUint::parse_bytes(b"FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141", 16).unwrap();

    // curve
    WeierstrassEq::new(f, a, b, gx, gy, n).unwrap()
  }
}

impl Curve for WeierstrassEq {
  fn g(&self) -> EcPoint {
    self.g.clone()
  }
  
  fn n(&self) -> BigUint {
    self.n.clone()
  }

  fn is_on_curve(&self, pt: &EcPoint) -> bool {
    println!("Checking if on curve");
    if pt.is_inf {
      false
    } else {
      let x3 = pt.x.mul(&pt.x).mul(&pt.x);
      let ax = self.a.mul(&pt.x);
      let y2 = pt.y.mul(&pt.y);

      // check if y^2 = x^3 + Ax + B
      y2 == x3.add(&ax).add(&self.b)
    }
  }
}
