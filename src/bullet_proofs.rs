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

  fn vector_mul_ec_points_by_field_elem(&self, ec_points: &[EcPoint], field_elem: &FieldElem) -> Vec<EcPoint> {
    ec_points.iter().map(|pt| {
      self.ops.scalar_mul(pt, &field_elem.n)
    }).collect()
  }

  fn vector_mul_field_elems_by_field_elem(&self, vector: &[FieldElem], scalar: &FieldElem) -> Vec<FieldElem> {
    vector.iter().map(|fe| {
      fe * scalar  
    }).collect()
  }

  fn vector_add_field_elem_vectors(&self, a: &[FieldElem], b: &[FieldElem]) -> Vec<FieldElem> {
    a.iter().zip(b.iter()).map(|(a, b)| {
      a + b 
    }).collect()
  }

  fn ec_point_hadamard_product(&self, g: &[EcPoint], h: &[EcPoint]) -> Vec<EcPoint> {
    g.iter().zip(h.iter()).map(|(g_i, h_i)| {
      self.ops.add(g_i, h_i)
    }).collect()
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

  // length of a, aa, b, bb are the half of the length of n
  pub fn H(&self, 
    n: usize,
    g: &[EcPoint], u: &EcPoint,   
    a1: &[FieldElem], a2: &[FieldElem], 
    b1: &[FieldElem], b2: &[FieldElem], 
    c: &FieldElem,
  ) -> EcPoint {
    let nn = n / 2;
    let ga1 = self.vector_mul(&g[0..nn], a1);
    let ga2 = self.vector_mul(&g[nn..], a2);
    let hb1 = self.vector_mul(&g[0..nn], b1);
    let hb2 = self.vector_mul(&g[nn..], b2);
    let uc = self.ops.scalar_mul(u, &c.n);

    self.ops.vector_add(&vec![ga1, ga2, hb1, hb2, uc])
  }

  // for a given P, prover proves that it has vectors a, b s.t. 
  // P = g^a h^b u^<a,b>
  pub fn perform_improved_inner_product_argument(&self,
    n: usize,
    g: &[EcPoint], h: &[EcPoint],
    u: &EcPoint, p: &EcPoint,  
    a: &[FieldElem], b: &[FieldElem], 
  ) -> bool {
    if n == 1 {
      // prover sends a,b to verifier

      // verifier computers c = a*b
      let c = &a[0] * &b[0];

      // verifier accepts if P = g^a h^b u^c holds
      let ga = self.ops.scalar_mul(&g[0], &a[0].n);
      let hb = self.ops.scalar_mul(&h[0], &b[0].n);
      let uc = self.ops.scalar_mul(u, &c.n);

      let rhs = self.ops.vector_add(&vec![ga , hb, uc]);
      P == &rhs 

    } else {
      // prover computes L,R whose length is n/2
      let nn = n / 2;
      let cL = self.inner_product_of(&a[..nn], &b[nn..]);
      let cR = self.inner_product_of(&a[nn..], &b[..nn]);
      let L = self.ops.vector_add(&vec![
        self.vector_mul(&g[nn..], &a[..nn]),
        self.vector_mul(&h[..nn], &b[nn..]),
        self.ops.scalar_mul(u, &cL.n),
      ]);
      let R = self.ops.vector_add(&vec![
        self.vector_mul(&g[..nn], &a[nn..]),
        self.vector_mul(&h[nn..], &b[..nn]),
        self.ops.scalar_mul(u, &cR.n),
      ]);

      // prover passes L,R to verifier

      // verifier chooses x in Z^*_p and sends to prover
      let x = self.f.rand_elem();

      // both prover and verifier compute gg,hh,PP
      let gg = self.ec_point_hadamard_product(
        &self.vector_mul_ec_points_by_field_elem(&g[..nn], &x.inv()),
        &self.vector_mul_ec_points_by_field_elem(&g[nn..], &x),
      );
      let hh = self.ec_point_hadamard_product(
        &self.vector_mul_ec_points_by_field_elem(&h[..nn], &x),
        &self.vector_mul_ec_points_by_field_elem(&h[nn..], &x.inv()),
      );
      let pp = self.ops.vector_add(&vec![
        self.ops.scalar_mul(&L, &x.sq().n),
        p.clone(),
        self.ops.scalar_mul(&R, &x.sq().inv().n),
      ]);

      // prover computes aa, bb
      let aa = self.vector_add_field_elem_vectors(
        &self.vector_mul_field_elems_by_field_elem(&a[..nn], &x),
        &self.vector_mul_field_elems_by_field_elem(&a[nn..], &x.inv()),
      );
      let bb = self.vector_add_field_elem_vectors(
        &self.vector_mul_field_elems_by_field_elem(&b[..nn], &x.inv()),
        &self.vector_mul_field_elems_by_field_elem(&b[nn..], &x),
      );
      self.perform_improved_inner_product_argument(nn, &gg, &hh, u, &pp, &aa, &bb)
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

    let c = bp.inner_product_of(&a, &b);
    let p = bp.vector_mul_add(&g, &h, &a, &b); 
    println!("Created c, p");

    println!("Running argument");
    assert!(bp.perform_simplest_inner_product_argument(&g, &h, &c, &p, &a, &b));
  }

}