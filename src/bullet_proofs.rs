use crate::elliptic_curve::{EllipticCurve, AddOps};
use crate::field::{Field, FieldElem};
use crate::ec_point::EcPoint;
use crate::vector_ops::{FieldElems, EcPoints, EcPoint1};

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

  fn ec_points(&self, ec_points: &'a [EcPoint]) -> EcPoints<'a> {
    
    EcPoints((self.ops, ec_points.to_vec()))
  }

  fn ec_point(&self, ec_point: &'a EcPoint) -> EcPoint1<'a> {
    EcPoint1((self.ops, ec_point))
  }

  fn field_elems(&self, field_elems: &'a [FieldElem]) -> FieldElems {
    FieldElems(field_elems.to_vec())
  }

  fn n(&self) -> usize {
    N  
  }

  fn vector_mul_add(&self, g: &[EcPoint], h: &[EcPoint], a: &[FieldElem], b: &[FieldElem]) -> EcPoint {
    let ga = self.ec_points(g) * a;
    let hb = self.ec_points(h) * b;
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
    let c_act = self.field_elems(a) * b; // self.inner_product_of(a, b);
    println!("Calculated c");

    p_exp == &p_act && c_exp == &c_act
  }

  // length of a, aa, b, bb are the half of the length of n
  pub fn H(&self, 
    n: usize,
    g: &[EcPoint], u: &EcPoint,   
    a1: &[FieldElem], a2: &[FieldElem], 
    b1: &[FieldElem], b2: &[FieldElem], 
    c: &FieldElem,
  ) -> EcPoint {
    let nn = n / 2;
    let ga1 = self.ec_points(&g[0..nn]) * a1;
    let ga2 = self.ec_points(&g[nn..]) * a2;
    let hb1 = self.ec_points(&g[0..nn]) * b1;
    let hb2 = self.ec_points(&g[nn..]) * b2;
    let uc = self.ops.scalar_mul(u, &c.n);

    self.ops.vector_add(&vec![ga1, ga2, hb1, hb2, uc])
  }

  // for a given P, prover proves that it has vectors a, b s.t. 
  // P = g^a h^b u^<a,b>
  pub fn perform_improved_inner_product_argument(&self,
    n: usize,
    g: EcPoints<'a>, h: EcPoints<'a>,
    u: EcPoint, p: EcPoint,  
    a: FieldElems, b: FieldElems, 
  ) -> bool {
    if n == 1 {
      // prover sends a,b to verifier

      // verifier computers c = a*b
      let c = &a[0] * &b[0];

      // verifier accepts if P = g^a h^b u^c holds
      let ga = self.ec_point(&g.0.1[0]) * &a[0];
      let hb = self.ec_point(&h.0.1[0]) * &b[0];
      let uc = self.ec_point(&u) * &c;

      let rhs = self.ops.vector_add(&vec![ga , hb, uc]);
      p == rhs 

    } else {
      // prover computes L,R whose length is n/2
      let nn = n / 2;
      let cL = self.field_elems(&a[..nn]) * &b[nn..];
      let cR = self.field_elems(&a[nn..]) * &b[..nn];
      let L = self.ops.vector_add(&vec![
        self.ec_points(&g.0.1[nn..]) * &a[..nn],
        self.ec_points(&h.0.1[..nn]) * &b[nn..],
        self.ops.scalar_mul(&u, &cL.n),
      ]);
      let R = self.ops.vector_add(&vec![
        self.ec_points(&g.0.1[..nn]) * &a[nn..],
        self.ec_points(&h.0.1[nn..]) * &b[..nn],
        self.ec_point(&u) * &cR,
      ]);

      // prover passes L,R to verifier

      // verifier chooses x in Z^*_p and sends to prover
      let x = self.f.rand_elem();

      // both prover and verifier compute gg,hh,PP
      let gg = (self.ec_points(&g.0.1[..nn]) * x.inv()) * (self.ec_points(&g.0.1[nn..]) * x.clone());
      let hh = (self.ec_points(&h.0.1[..nn]) * x.clone()) * (self.ec_points(&h.0.1[nn..]) * x.inv());
      
      let pp = self.ops.vector_add(&vec![
        self.ec_point(&L) * &x.sq(),
        p.clone(),
        self.ec_point(&R) * &x.sq().inv(),
      ]);

      // prover computes aa, bb
      let aa = (self.field_elems(&a[..nn]) * &x) + &(self.field_elems(&a[nn..]) * &x.inv());
      let bb = (self.field_elems(&b[..nn]) * &x.inv()) + &(self.field_elems(&b[nn..]) * &x);
      self.perform_improved_inner_product_argument(nn, gg, hh, u, pp, aa, bb)
    }

  }

}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::weierstrass_eq::WeierstrassEq;
  use crate::weierstrass_add_ops::JacobianAddOps;

  #[test]
  fn test_perform_simplest_inner_product_argument() {
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

    let c = bp.field_elems(&a) * b.as_slice();
    let p = bp.vector_mul_add(&g, &h, &a, &b); 
    println!("Created c, p");

    println!("Running argument");
    assert!(bp.perform_simplest_inner_product_argument(&g, &h, &c, &p, &a, &b));
  }

}