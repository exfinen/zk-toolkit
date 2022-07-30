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

  pub fn perform_inner_product_range_proof(
    &self,
    v: FieldElem,
    g: EcPoint1<'a>,
    h: EcPoint1<'a>,
    gg: EcPoints<'a>,
    hh: EcPoints<'a>,
    gamma: FieldElem,
  ) -> bool {
    // aL = (a_1, a_2, .., a_n) in {0,1}^n contains bits of v so that <aL, 2^n> = v
    //
    // prover commits to aL using commitment A in G
    // 
    // prover convinces verifier that v is in [0, 2^n-1] 
    // by providing it knows opening of A (aL) and v, gamma in Zp s.t. 
    // - V = h^gamma g^v 
    // - <aL, s^n> = v
    // - aL o aR = 0^n
    // - aR = aL - 1^n

    // prover compute A,S
    let aL: Vec<FieldElem> = vec![0u8, 1, 1, 0].iter().map(|x| self.f.elem(x)).collect();
    let aL = self.field_elems(&aL);
    let one: FieldElem = self.f.elem(&1u8);
    let aR: Vec<FieldElem> = aL.iter().map(|x| x - &one).collect();
    let aR = self.field_elems(&aR);
    let alpha = self.f.rand_elem();

    // commitment A to aL,aR
    let A = self.ops.vector_add(&vec![
      h * &alpha,
      gg * &aL[..],
      hh * &aR[..],
    ]);

    // commitment S to sL,sR
    let sL = self.field_elems(&self.f.rand_elems(self.n()));
    let sR = self.field_elems(&self.f.rand_elems(self.n()));
    let rho = self.f.rand_elem();
    let S = self.ops.vector_add(&vec![
      h * &rho,
      gg * &sL[..],
      hh * &sR[..],
    ]);
    let S = self.ec_point(&S);

    // prover sends A,S to verifier
  
    let y = self.f.rand_elem();
    let z = self.f.rand_elem();

    // verifier sends y,z to prover

    // prover computes
    let t1 = self.f.rand_elem();
    let t2 = self.f.rand_elem();

    // prover calculate commitments to t1,t2
    let T1 = self.ops.vector_add(&vec![
      g * &t1,
      h * &t1,
    ]);
    let T1 = self.ec_point(&T1);
    let T2 = self.ops.vector_add(&vec![
      g * &t2,
      h * &t2,
    ]);
    let T2 = self.ec_point(&T2);

    // prover sends T1,T2 to verifier

    // verifier selects random x and sends to prover
    let x = self.f.rand_elem();

    // prover computes
    let ones = self.f.repeated_elem(&1u8, self.n());
    let l = (aL - &ones[..]) + &(sL * &x);  // (58)
    let y_to_n = self.f.first_n_powers_of_x(&y, self.n());
    let y_to_n = self.field_elems(&y_to_n);
    let z_n = self.f.repeated_elem(&z, self.n());
    let z_n = self.field_elems(&z_n);
    let two_to_n = self.f.first_n_powers_of_x(&2u8, self.n());
    let two_to_n = self.field_elems(&two_to_n);
    let r = (y_to_n * &((aR - &z_n) + &(sR * &x))) + &(two_to_n * &z.sq());  // (59)
    let t_hat: FieldElem = l * &r[..];  // (60) should find a better way than switching by multiplicand type
    let tx = (t2 * &x.sq()) + &(t1 * &x) + &(z.sq() * &gamma);  // (61)
    let mu: FieldElem = alpha + &(rho * &x);  // (62)

    // prover sends tx,mu,t_hat,l,r to verifier

    let hp: Vec<EcPoint> = (0..self.n()).map(|i| {  // (64)
      let exp = y.pow(&self.f.elem(&i).inv());
      hh.at(i) * &exp 
    }).collect();
    let hp = self.ec_points(&hp);

    // (65)
    let lhs_65: EcPoint = self.ec_point(&(g * &t_hat)) + &self.ec_point(&(h * &tx));
    let V: EcPoint1 = self.ec_point(&(self.ec_point(&(h * &gamma)) + &self.ec_point(&(g * &v))));
    let z_minus_z2: FieldElem = z - &z.sq();
    let z3: FieldElem = z * &z.sq();
    let delta_y_z_term1: FieldElem = y_to_n.sum() * &z_minus_z2;
    let delta_y_z_term2: FieldElem = two_to_n.sum() * &z3;
    let delta_y_z: FieldElem = delta_y_z_term1 - &delta_y_z_term2;
    let rhs_65: EcPoint = self.ops.vector_add(&[
      V * &z.sq(),
      g * &delta_y_z,
      T1 * &x,
      T2 * &x.sq(),
    ]);
    if lhs_65 != rhs_65 {
      return false
    }

    // (66)
    let hp_exp = (y_to_n * &z) + &(two_to_n * &z.sq());
    let lhs_66_P: EcPoint = self.ops.vector_add(&[  // (66)
      A,
      S * &x,
      (gg * &z.inv()).sum().0.1,  // TODO add function to extract EcPoint
      hp * &hp_exp[..],
    ]);
    let rhs_66_term1: EcPoint1 = self.ec_point(&(h * &mu)); 
    let rhs_66_term2: EcPoint1 = self.ec_point(&(gg * &l[..]));
    let rhs_66_term3: EcPoint1 = self.ec_point(&(hp * &r[..]));
    let rhs_66: EcPoint = rhs_66_term1 + &(rhs_66_term2 + &rhs_66_term3);   
    if lhs_66_P != rhs_66 {  // (67)
      return false;
    }

    let rhs_68 = (l * &r).sum();

    // (68)
    t_hat == rhs_68
  }

}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::weierstrass_eq::WeierstrassEq;
  use crate::weierstrass_add_ops::JacobianAddOps;

  #[test]
  fn test_perform_inner_product_range_proof() {
    let curve = WeierstrassEq::secp256k1();
    let ops = JacobianAddOps::new();

    let bp: BulletProofs<4> = BulletProofs::new(&curve, &ops);
    //bp.perform_inner_product_range_proof();
  }

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