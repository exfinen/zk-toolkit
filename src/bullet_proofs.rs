use crate::elliptic_curve::{EllipticCurve, AddOps};
use crate::field::{Field, FieldElem, FieldElems};
use crate::ec_point::EcPoint;
use crate::vector_ops::{EcPoints, EcPoint1};

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
    assert!(ec_points.len() > 0);
    let xs = ec_points.iter().map(|x| EcPoint1((self.ops, x.clone()))).collect::<_>();
    EcPoints((self.ops, xs))
  }

  fn ec_point(&self, ec_point: &'a EcPoint) -> EcPoint1<'a> {
    EcPoint1((self.ops, ec_point.clone()))
  }

  fn n(&self) -> usize {
    N  
  }

  fn field_elem_mul(&self, xs: &[FieldElem], ys: &[FieldElem]) -> FieldElems {
    let zs = xs.iter().zip(ys.iter()).map(|(x, y)| x * y).collect::<Vec<FieldElem>>();
    FieldElems(zs)
  }

  fn vector_add(&self, xs: &[&EcPoint1<'a>]) -> EcPoint1<'a> {
    assert!(xs.len() > 0);
    let head = xs[0];
    let (ops, _) = xs[0].0;
    let tail = &xs[1..];
    let x = tail.iter().fold(head.0.1.clone(), |acc, x| {
      self.ops.add(&acc, x)
    });
    EcPoint1((ops, x))
  }

  fn vector_mul_add(&self, 
    g: &EcPoints, h: &EcPoints, 
    a: &FieldElems, b: &FieldElems
  ) -> EcPoint {
    let ga = (g * a).sum();
    let hb = (h * b).sum();
    self.ops.add(&ga, &hb)
  }

  fn scalar_mul(&self, pt: &EcPoint1<'a>, fe: &FieldElem) -> EcPoint1<'a> {
    let (ops, _) = pt.0;
    let x = self.ops.scalar_mul(&pt, &fe);
    EcPoint1((ops, x))
  }

  // prover and verifier know:
  // g, h, c, P
  //
  // then prover convinces verifier that the prover knows a and b s.t.
  // P = g^a h^b and c = <a,b>
  //
  pub fn perform_simplest_inner_product_argument(&self, 
    g: &EcPoints, h: &EcPoints, 
    c_exp: &FieldElem, p_exp: &EcPoint,
    a: &FieldElems, b: &FieldElems,
  ) -> bool {
    let p_act = self.vector_mul_add(g, h, a, b);
    let c_act = (&FieldElems::new(a) * b).sum(); // self.inner_product_of(a, b);

    p_exp == &p_act && c_exp == &c_act
  }

  // for a given P, prover proves that it has vectors a, b s.t. 
  // P = g^a h^b u^<a,b>
  pub fn perform_improved_inner_product_argument(&self,
    n: usize,
    g: &EcPoints<'a>, h: &EcPoints<'a>,
    u: &EcPoint1<'a>, p: &EcPoint1<'a>,  
    a: &FieldElems, b: &FieldElems, 
  ) -> bool {
    if n == 1 {
      // prover sends a,b to verifier

      // verifier computers c = a*b
      let c = &a[0] * &b[0];

      // verifier accepts if P = g^a h^b u^c holds
      let ga = &g[0] * &a[0];
      let hb = &h[0] * &b[0];
      let uc = u * &c;

      let rhs = self.vector_add(&[&ga , &hb, &uc]);
      p == &rhs 

    } else {
      // prover computes L,R whose length is n/2
      let nn = n / 2;
      let cL = self.field_elem_mul(&a.to(..nn), &b.from(nn..)).sum();
      let cR = self.field_elem_mul(&a.from(nn..), &b.to(..nn)).sum();
      let L = self.vector_add(&vec![
        &(&g.from(nn..) * &a.to(..nn)).sum(),
        &(&h.to(..nn) * &b.from(nn..)).sum(),
        &self.scalar_mul(u, &cL),
      ]);
      let R = self.vector_add(&vec![
        &(&g.to(..nn) * &a.from(nn..)).sum(),
        &(&h.from(nn..) * &b.to(..nn)).sum(),
        &(u * &cR),
      ]);

      // prover passes L,R to verifier

      // verifier chooses x in Z^*_p and sends to prover
      let x = self.f.rand_elem();

      // both prover and verifier compute gg,hh,PP
      let gg = &(&g.to(..nn) * &x.inv()) * &(&g.from(nn..) * &x);
      let hh = &(&h.to(..nn) * &x) * &(&h.from(nn..) * &x.inv());
      
      let pp = self.vector_add(&vec![
        &(&L * &x.sq()),
        p,
        &(&R * &x.sq().inv()),
      ]);

      // prover computes aa, bb
      let aa = &(&a.to(..nn) * &x) + &(&a.from(nn..) * &x.inv());
      let bb = &(&b.to(..nn) * &x.inv()) + &(&b.from(nn..) * &x);
      self.perform_improved_inner_product_argument(
        nn, &gg, &hh, u, &pp, &aa, &bb)
    }
  }

  pub fn perform_inner_product_range_proof(
    &self,
    v: &FieldElem,
    g: &EcPoint1<'a>,
    h: &EcPoint1<'a>,
    gg: &EcPoints<'a>,
    hh: &EcPoints<'a>,
    gamma: &FieldElem,
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
    let aL = FieldElems::new(&vec![0u8, 1, 1, 0].iter().map(|x| self.f.elem(x)).collect::<Vec<FieldElem>>());
    let one = self.f.elem(&1u8);
    let aR = FieldElems::new(&aL.iter().map(|x| x - &one).collect::<Vec<FieldElem>>());
    let alpha = self.f.rand_elem();

    // commitment A to aL,aR
    let A = self.vector_add(&vec![
      &(h * &alpha),
      &(gg * &aL).sum(),
      &(hh * &aR).sum(),
    ]);

    // commitment S to sL,sR
    let sL = self.f.rand_elems(self.n());
    let sR = self.f.rand_elems(self.n());
    let rho = self.f.rand_elem();
    let S = self.vector_add(&vec![
      &(h * &rho),
      &(gg * &sL).sum(),
      &(hh * &sR).sum(),
    ]);

    // prover sends A,S to verifier
  
    let y = self.f.rand_elem();
    let z = self.f.rand_elem();

    // verifier sends y,z to prover

    // prover computes
    let t1 = self.f.rand_elem();
    let t2 = self.f.rand_elem();

    // prover calculate commitments to t1,t2
    let T1 = self.vector_add(&vec![
      &(g * &t1),
      &(h * &t1),
    ]);
    let T2 = self.vector_add(&vec![
      &(g * &t2),
      &(h * &t2),
    ]);

    // prover sends T1,T2 to verifier

    // verifier selects random x and sends to prover
    let x = self.f.rand_elem();

    // prover computes
    let ones = self.f.repeated_elem(&1u8, self.n());
    let l = &(&aL - &ones) + &(&sL * &x);  // (58)
    let y_to_n = self.f.first_n_powers_of_x(&y, self.n());
    let z_n = self.f.repeated_elem(&z, self.n());
    let two_to_n = self.f.first_n_powers_of_x(&2u8, self.n());
    let r = &(&y_to_n * &(&(&aR - &z_n) + &(&sR * &x))) + &(&two_to_n * &z.sq());  // (59)
    let t_hat = &(&l * &r).sum();  // (60) should find a better way than switching by multiplicand type
    let tx = &(&t2 * &x.sq()) + &(&t1 * &x) + &(&z.sq() * gamma);  // (61)
    let mu = &alpha + &(rho * &x);  // (62)

    // prover sends tx,mu,t_hat,l,r to verifier

    let hp_inner = (0..self.n()).map(|i| {  // (64)
      let exp = y.pow(&self.f.elem(&i).inv());
      &hh[i] * &exp
    }).collect::<Vec<EcPoint1<'a>>>();
    let hp = EcPoints((self.ops, hp_inner));

    // (65)
    let lhs_65 = &(g * &t_hat) + &(h * &tx);
    let V = &(h * &gamma) + &(g * &v);
    let z_minus_z2 = &z - &z.sq();
    let z3 = &z * &z.sq();
    let delta_y_z_term1 = y_to_n.sum() * &z_minus_z2;
    let delta_y_z_term2 = two_to_n.sum() * &z3;
    let delta_y_z = delta_y_z_term1 - &delta_y_z_term2;
    let rhs_65 = self.vector_add(&[
      &(&V * &z.sq()),
      &(g * &delta_y_z),
      &(&T1 * &x),
      &(&T2 * &x.sq()),
    ]);
    if lhs_65 != rhs_65 {
      return false
    }

    // (66)
    let hp_exp = &(&y_to_n * &z) + &(&two_to_n * &z.sq());
    let lhs_66_P = self.vector_add(&[  // (66)
      &A,
      &(&S * &x),
      &(gg * &z.inv()).sum(),
      &(&hp * &hp_exp).sum(),
    ]);
    let rhs_66_term1 = &(h * &mu); 
    let rhs_66_term2 = (gg * &l).sum();
    let rhs_66_term3 = (&hp * &r).sum();
    let rhs_66 = rhs_66_term1 + &(&rhs_66_term2 + &rhs_66_term3);   
    if lhs_66_P != rhs_66 {  // (67)
      return false;
    }

    let rhs_68 = &(&l * &r).sum();

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
    let a = FieldElems::new(&(0..n).map(|_| bp.f.rand_elem()).collect::<Vec<FieldElem>>());
    let b = FieldElems::new(&(0..n).map(|_| bp.f.rand_elem()).collect::<Vec<FieldElem>>());

    let g_inner = a.iter().map(|a_i| ops.scalar_mul(&base_point, &a_i.n)).collect::<Vec<EcPoint>>();
    let g = bp.ec_points(&g_inner);
    let h_inner = b.iter().map(|b_i| ops.scalar_mul(&base_point, &b_i.n)).collect::<Vec<EcPoint>>();
    let h = bp.ec_points(&h_inner);

    let c = (&a * &b).sum();
    let p = bp.vector_mul_add(&g, &h, &a, &b); 

    assert!(bp.perform_simplest_inner_product_argument(&g, &h, &c, &p, &a, &b));
  }

}