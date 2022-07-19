use crate::elliptic_curve::{EllipticCurve, AddOps};
use crate::field::{Field};

pub struct BulletProofs<'a> {
  pub curve: &'a dyn EllipticCurve,
  pub ops: &'a dyn AddOps,
  pub f: Field,
}

impl<'a> BulletProofs<'a> {
  pub fn new(
    curve: &'a dyn EllipticCurve, 
    ops: &'a dyn AddOps, 
  ) -> Self {
    let f = Field::new(&curve.n());
    BulletProofs { curve, ops, f }
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
    let base_point = curve.g();
    
    let a = (0..n).map(|_| bp.f.rand_elem());
    let b = (0..n).map(|_| bp.f.rand_elem());

    let g = a.map(|a_i| ops.scalar_mul(&base_point, &a_i.n));
    let h = b.map(|b_i| ops.scalar_mul(&base_point, &b_i.n));

    // P = g^a h^b and c = <a,b>

    let u = ops.scalar_mul(&base_point, &bp.f.rand_elem().n);
    bp.prove();
  }
}