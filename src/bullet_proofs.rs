use crate::elliptic_curve::{EllipticCurve, AddOps};
use crate::field::{Field, FieldElem};

pub struct BulletProofs<'a> {
  pub curve: &'a dyn EllipticCurve,
  pub ops: &'a dyn AddOps,
  pub f_n: Field,
}

impl<'a> BulletProofs<'a> {
  pub fn new(
    curve: &'a dyn EllipticCurve, 
    ops: &'a dyn AddOps, 
  ) -> Self {
    let f_n = Field::new(&curve.n());
    BulletProofs { curve, ops, f_n }
  }

  pub fn vector_mul(&self) {

  }

  pub fn prove(&self) {
    println!("Proved");
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::weierstrass_eq::WeierstrassEq;
  use crate::weierstrass_add_ops::JacobianAddOps;

  #[test]
  fn test1() {
    let curve = WeierstrassEq::secp256k1();
    let ops = JacobianAddOps::new();

    let bp = BulletProofs::new(&curve, &ops);

    let n = 4;
    let g = curve.g();
    
    bp.prove();
  }
}