use crate::elliptic_curve::{EllipticCurve, AddOps};
use crate::field::{Field, FieldElem};
use crate::ec_point::EcPoint;

// implementation based on https://eprint.iacr.org/2017/1066.pdf

pub struct BulletProofs<'a, const N: usize> {
  pub curve: &'a dyn EllipticCurve,
  pub ops: &'a dyn AddOps,
  pub f: Field,
}

impl<'a, const N: usize> BulletProofs<'a, N> {
  pub fn new(
    curve: &'a dyn EllipticCurve, 
    ops: &'a dyn AddOps, 
  ) -> Self {
    let f = Field::new(&curve.n());
    BulletProofs { curve, ops, f }
  }

  pub fn n(&self) -> usize {
    N  
  }

  pub fn inner_product_of(&self, a: &[FieldElem], b: &[FieldElem]) -> FieldElem {
    let zero = self.f.elem(&0u8);
    a.iter().zip(b.iter())
      .fold(zero, |acc, (a_i, b_i)| acc + &(a_i * b_i))
  }

  fn vector_mul(&self, ec_points: &[EcPoint], field_elems: &[FieldElem]) -> EcPoint {
    ec_points.iter().zip(field_elems.iter()).fold(None::<EcPoint>, |acc, (pt, fe)| {
      let x = self.ops.scalar_mul(pt, &fe.n);
      match acc {
        None => Some(x),
        Some(y) => Some(self.ops.add(&x, &y)),
      }
    }).unwrap()
  }

  pub fn vector_mul_add(&self, g: &[EcPoint], h: &[EcPoint], a: &[FieldElem], b: &[FieldElem]) -> EcPoint {
    let ga = self.vector_mul(&g, &a);
    let hb = self.vector_mul(&h, &b);
    self.ops.add(&ga, &hb)
  }

  // prover and verifier know:
  // g, h, c, P
  //
  // then prover convinces verifier that the prover knows a and b s.t.
  // P = g^a h^b and c = <a,b>
  //
  pub fn perform_simplest_inner_product_argument(&self, 
    g: &[EcPoint], h: &[EcPoint], c_exp: &FieldElem, p_exp: &EcPoint,
    a: &[FieldElem], b: &[FieldElem],
  ) -> bool {
    let p_act = self.vector_mul_add(g, h, a, b);
    println!("Calculated P");
    let c_act = self.inner_product_of(a, b);
    println!("Calculated c");

    p_exp == &p_act && c_exp == &c_act
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

    let bp: BulletProofs<4> = BulletProofs::new(&curve, &ops);

    let base_point = curve.g();
    
    let n = bp.n();
    let a: Vec<_> = (0..n).map(|_| bp.f.rand_elem()).collect();
    let b: Vec<_> = (0..n).map(|_| bp.f.rand_elem()).collect();
    println!("Created a, b");

    let g: Vec<_> = a.iter().map(|a_i| ops.scalar_mul(&base_point, &a_i.n)).collect();
    let h: Vec<_> = b.iter().map(|b_i| ops.scalar_mul(&base_point, &b_i.n)).collect();
    println!("Created g, h");

    let c = bp.inner_product_of(&a, &b);
    let p = bp.vector_mul_add(&g, &h, &a, &b); 
    println!("Created c, p");

    println!("Running argument");
    assert!(bp.perform_simplest_inner_product_argument(&g, &h, &c, &p, &a, &b));
  }

}