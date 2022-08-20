use crate::elliptic_curve::{EllipticCurve, AddOps};
use crate::field::{FieldElem, FieldElems};
use crate::ec_point::EcPoint;
use crate::vector_ops::{EcPoints, EcPoint1};

// implementation based on https://eprint.iacr.org/2017/1066.pdf

pub struct BulletProofs<'a, const N: usize> {
  pub curve: &'a dyn EllipticCurve,
  pub ops: &'a dyn AddOps,
}

impl<'a, const N: usize> BulletProofs<'a, N> {
  pub fn new(
    curve: &'a dyn EllipticCurve, 
    ops: &'a dyn AddOps, 
  ) -> Self {
    BulletProofs { curve, ops }
  }

  pub fn ec_points(&self, ec_points: &'a [EcPoint]) -> EcPoints<'a> {
    assert!(ec_points.len() > 0);
    let xs = ec_points.iter().map(|x| EcPoint1((self.ops, x.clone()))).collect::<_>();
    EcPoints((self.ops, xs))
  }

  pub fn ec_point(&self, ec_point: &'a EcPoint) -> EcPoint1<'a> {
    EcPoint1((self.ops, ec_point.clone()))
  }

  pub fn rand_point(&self) -> EcPoint {
    let pt = self.curve.g();
    let fe = self.curve.f().rand_elem(true);
    let g = self.ec_point(&pt);
    self.scalar_mul(&g, &fe).into() 
  }

  pub fn rand_points(&self, n: usize) -> Vec<EcPoint> {
    let mut xs = vec![];
    for _ in 0..n {
      xs.push(self.rand_point().into());
    }
    xs
  }

  pub fn n(&self) -> usize {
    N  
  }

  pub fn field_elem_mul(&self, xs: &[FieldElem], ys: &[FieldElem]) -> FieldElems {
    let zs = xs.iter().zip(ys.iter()).map(|(x, y)| x * y).collect::<Vec<FieldElem>>();
    FieldElems(zs)
  }

  pub fn vector_add(&self, xs: &[&EcPoint1<'a>]) -> EcPoint1<'a> {
    assert!(xs.len() > 0);
    let head = xs[0];
    let (ops, _) = xs[0].0;
    let tail = &xs[1..];
    let x = tail.iter().fold(head.0.1.clone(), |acc, x| {
      self.ops.add(&acc, x)
    });
    EcPoint1((ops, x))
  }

  pub fn vector_mul_add(&self, 
    g: &EcPoints, h: &EcPoints, 
    a: &FieldElems, b: &FieldElems
  ) -> EcPoint {
    let ga = (g * a).sum();
    let hb = (h * b).sum();
    self.ops.add(&ga, &hb)
  }

  pub fn scalar_mul(&self, pt: &EcPoint1<'a>, fe: &FieldElem) -> EcPoint1<'a> {
    let (ops, _) = pt.0;
    let x = self.ops.scalar_mul(&pt, &fe);
    EcPoint1((ops, x))
  }

  // for a given P, prover proves that it has vectors a, b s.t. 
  // P = g^a h^b u^<a,b>
  #[allow(non_snake_case)]
  pub fn improved_inner_product_argument(&self,
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
      let x = self.curve.f().rand_elem(true);

      // both prover and verifier compute gg,hh,PP
      let gg = (&g.to(..nn) * &x.inv()) * (&g.from(nn..) * &x);
      let hh = (&h.to(..nn) * &x) * (&h.from(nn..) * &x.inv());
      
      let pp = self.vector_add(&vec![
        &(&L * &x.sq()),
        p,
        &(&R * &x.sq().inv()),
      ]);

      // prover computes aa, bb
      let aa = (&a.to(..nn) * &x) + (&a.from(nn..) * &x.inv());
      let bb = (&b.to(..nn) * &x.inv()) + (&b.from(nn..) * &x);
      self.improved_inner_product_argument(
        nn, &gg, &hh, u, &pp, &aa, &bb)
    }
  }

  #[allow(non_snake_case)]
  pub fn range_proof(
    &self,
    n: usize,
    V: &EcPoint1<'a>,
    aL: &FieldElems,
    gamma: &FieldElem,
    g: &EcPoint1<'a>,
    h: &EcPoint1<'a>,
    gg: &EcPoints<'a>,
    hh: &EcPoints<'a>,
  ) -> bool {
    let co = self.curve.n();

    // on input upsilon, gamma prover computes
    let one = co.elem(&1u8);
    let two = co.elem(&2u8);
    let one_n = one.pow_seq(n);
    let two_n = two.pow_seq(n);

    let aR = aL - &one_n;
    let alpha = co.rand_elem(true);
    let A = self.vector_add(&vec![
      &(h * &alpha),
      &(gg * aL).sum(),
      &(hh * &aR).sum(),
    ]);

    let sL = co.rand_elems(n, true);
    let sR = co.rand_elems(n, true);
    let rho = co.rand_elem(true);
    let S = self.vector_add(&vec![
      &(h * &rho),
      &(gg * &sL).sum(),
      &(hh * &sR).sum(),
    ]);

    // prover sends A,S to verifier
  
    // verifier sends y,z to prover
    let y = co.rand_elem(true);
    let z = co.rand_elem(true);

    // define t(x) = <l(x),r(x)> = t0 + t1 * x + t2 * x^2
    let y_n = &y.pow_seq(n);
    let l0 = aL - (&one_n * &z);
    let l1 = &sL;
    let r0 = (y_n * (&aR + (&one_n * &z))) + (&two_n * &z.sq());
    let r1 = y_n * &sR;

    let t0 = (&l0 * &r0).sum();
    let t1 = (l1 * &r0).sum() + (&l0 * &r1).sum();
    let t2 = (l1 * &r1).sum();

    // prover computes
    let tau1 = co.rand_elem(true);
    let tau2 = co.rand_elem(true);
    let T1 = self.vector_add(&vec![
      &(g * &t1),
      &(h * &tau1),
    ]);
    let T2 = self.vector_add(&vec![
      &(g * &t2),
      &(h * &tau2),
    ]);

    // prover sends T1,T2 to verifier

    // verifier selects random x and sends to prover
    let x = co.rand_elem(true);

    // prover computes

    let t_hat = t0 + (&t1 * &x) + (&t2 * &x.sq());
    let tau_x = &tau2 * &x.sq() + &((&tau1 * &x) + &(&z.sq() * gamma));
    let mu = &alpha + &(&rho * &x);

    // prover sends l, r, t_hat, mu to verifier

    // (64)
    let hhp = hh * &y.inv().pow_seq(n);

    // (65)
    let delta_yz = &((&z - &z.sq()) * &(&one_n * y_n).sum()) - &(&z.cube() * &(&one_n * &two_n).sum());

    let lhs_65 = (g * &t_hat) + (h * &tau_x);
    let rhs_65 = self.vector_add(&vec![
      &(V * &z.sq()),
      &(g * &delta_yz),
      &(&T1 * &x),
      &(&T2 * &x.sq()),
    ]);
    println!("CHECK 1");
    if lhs_65 != rhs_65 {
      return false;
    }
    println!("CHECK 1 PASSED");

    // (66), (67)
    let l = (aL - (&one_n * &z)) + (&sL * &x);
    let r = (y_n * &((aR + (&one_n * &z)) + (&sR * &x))) + (&two_n * &z.sq());

    let P = self.vector_add(&vec![
      &A,
      &(&S * &x),
      &(gg * &(&one_n * &z.negate())).sum(),   // TODO check this
      &(&hhp * &((y_n * &z) + (&two_n * &z.sq()))).sum(),
    ]);

    let rhs_66_67 = ((h * &mu) + (gg * &l).sum()) + (&hhp * &r).sum();
    println!("CHECK 2");
    if P != rhs_66_67 {
      return false;
    }
    println!("CHECK 2 PASSED");

    // (68)
    let rhs_68 = (&l * &r).sum();

    println!("CHECK 3");
    t_hat == rhs_68
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::weierstrass_eq::WeierstrassEq;
  use crate::weierstrass_add_ops::JacobianAddOps;

  // gg^z == gg^(ones * z)
  #[test]
  fn test_gg_ones_times_z() {
    let curve = WeierstrassEq::secp256k1();
    let ops = JacobianAddOps::new();
    let co = curve.n();
    let bp: BulletProofs<2> = BulletProofs::new(&curve, &ops);

    let n = 2;
    let z = co.rand_elem(true);
    let gg = bp.rand_points(n);
    let gg = bp.ec_points(&gg);

    let r1 = &gg * &z;

    let one = co.elem(&1u8);
    let ones = one.repeat(n);
    let r2 = &gg * &(&ones * &z);

    assert!(r1 == r2);
  }

  #[test]
  fn test_offset_by_negation() {
    let curve = WeierstrassEq::secp256k1();
    let ops = JacobianAddOps::new();
    let co = curve.n();
    let bp: BulletProofs<2> = BulletProofs::new(&curve, &ops);

    {
        let z = co.elem(&100u8);
        let basis = co.elem(&12345u16);

        let r1 = &basis - &z;
        let r2 = &basis + &z.negate();
        
        assert_eq!(r1, r2);
    }
    {
        let z = co.elem(&100u8);
        let basis = co.elem(&12345u16);
        let g = curve.g();
        let g = bp.ec_point(&g);

        let r1 = bp.scalar_mul(&g, &(&basis - &z));
        let r2 = g * (basis + z.negate());
        
        assert!(r1 == r2);
    }
  }

  #[test]
  #[allow(non_snake_case)]
  fn test_base_point_field_elem_mul() {
    let curve = WeierstrassEq::secp256k1();
    let ops = JacobianAddOps::new();
    let co = curve.n();
    let bp: BulletProofs<2> = BulletProofs::new(&curve, &ops);

    let alpha = &co.rand_elem(true);
    let rho = &co.rand_elem(true);
    let h = bp.rand_point();
    let h = &bp.ec_point(&h);

    let a = h * alpha + h * rho;
    let b = h * (alpha + rho);
    assert!(a == b);
  }

  #[test]
  #[allow(non_snake_case)]
  fn test_field_elems_mul_field_elem() {
    let curve = WeierstrassEq::secp256k1();
    let co = curve.n();

    let x = co.elem(&5u8);
    let sL = FieldElems(vec![
      co.elem(&2u8),
      co.elem(&3u8),
    ]);

    let exp = FieldElems(vec![
      co.elem(&10u8),
      co.elem(&15u8),
    ]);
    let act = sL * x;
    assert!(act == exp);
  }

  #[test]
  #[allow(non_snake_case)]
  fn test_tmp() {
    let curve = WeierstrassEq::secp256k1();
    let ops = JacobianAddOps::new();
    let co = curve.n();
    let bp: BulletProofs<2> = BulletProofs::new(&curve, &ops);

    let n = 2;
    let gg = vec![
      curve.g(),
      curve.g(),
    ];
    let gg = &bp.ec_points(&gg);

    let x = &co.elem(&2u8);
    let xx = vec![ x.clone(), x.clone() ];
    let xx = &FieldElems(xx);
    let sL = vec![
      co.elem(&3u8),
      co.elem(&7u8),
    ];
    let sL = &FieldElems(sL);
    let sL = &co.rand_elems(n, true);
    println!("sL[0]={0}", sL[0].to_str_radix(16));
    println!("sL[1]={0}", sL[1].to_str_radix(16));
    
    let sLx = &(sL * x);
    println!("ll[0]={0}", sLx[0].to_str_radix(16));
    println!("ll[1]={0}", sLx[1].to_str_radix(16));

    assert!(gg * sLx == (gg * sL) * x);
  }

  #[test]
  #[allow(non_snake_case)]
  fn test_range_proof_66_67_excl_h_prime_experiment() {
    let curve = WeierstrassEq::secp256k1();
    let ops = JacobianAddOps::new();
    let co = curve.n();
    let bp: BulletProofs<2> = BulletProofs::new(&curve, &ops);

    let n = 2;
    let one = co.elem(&1u8);
    let ones = &one.repeat(n);
    let z = &co.rand_elem(true);

    let alpha = &co.rand_elem(true);
    let rho = &co.rand_elem(true);
    let x = &co.rand_elem(true);
    let mu = &(alpha + (rho * x)); 

    let gg = bp.rand_points(n);
    let gg = &bp.ec_points(&gg);

    let aL = &FieldElems(vec![
      co.elem(&1u8),
      co.elem(&1u8),
    ]);
    let sL = &co.rand_elems(n, true);

    let sLx = sL * x;
    let ll = &(/*- (ones * z)*/ sLx);

    let hmu_ggl = (gg * ll).sum();

    let A = (gg * aL).sum();
    let S = (gg * sL).sum();
    let P = S * x; // + (gg * z.negate()).sum();

    assert!(P == hmu_ggl);
  }

  #[test]
  #[allow(non_snake_case)]
  fn test_range_proof_66_67_excl_h_prime() {
    let curve = WeierstrassEq::secp256k1();
    let ops = JacobianAddOps::new();
    let co = curve.n();
    let bp: BulletProofs<2> = BulletProofs::new(&curve, &ops);

    let n = 2;
    let one = co.elem(&1u8);
    let ones = &one.repeat(n);
    let z = &co.rand_elem(true);

    let alpha = &co.rand_elem(true);
    let rho = &co.rand_elem(true);
    let x = &co.rand_elem(true);
    let mu = &(alpha + (rho * x)); 

    let gg = bp.rand_points(n);
    let gg = &bp.ec_points(&gg);
    let h = bp.rand_point();
    let h = &bp.ec_point(&h);

    let aL = &FieldElems(vec![
      co.elem(&1u8),
      co.elem(&1u8),
    ]);
    let sL = &co.rand_elems(n, true);

    let ll = &(aL - (ones * z) + (sL * x));

    let hmu_ggl = (h * mu) + (gg * ll).sum();

    let A = (h * alpha) + (gg * aL).sum();
    let S = (h * rho) + (gg * sL).sum();
    let P = (A + (&S * x)) + (gg * z.negate()).sum();

    assert!(P == hmu_ggl);
  }

  #[test]
  #[allow(non_snake_case)]
  fn test_range_proof() {
    let curve = WeierstrassEq::secp256k1();
    let ops = JacobianAddOps::new();
    let co = curve.n();
    let bp: BulletProofs<2> = BulletProofs::new(&curve, &ops);

    let aL = FieldElems::new(&vec![
      co.elem(&1u8), 
      co.elem(&0u8), 
      co.elem(&0u8), 
      co.elem(&1u8),
    ]);
    let n = aL.len();
    let upsilon = curve.f().elem(&9u8);
    let gamma = bp.curve.f().rand_elem(true);
    let g = bp.rand_point();
    let g = bp.ec_point(&g);
    let h = bp.rand_point();
    let h = bp.ec_point(&h);
    let gg = bp.rand_points(n);
    let gg = bp.ec_points(&gg);
    let hh = bp.rand_points(n);
    let hh = bp.ec_points(&hh);
    let V = (&h * &gamma) + (&g * &upsilon);

    let res = bp.range_proof(
      n,
      &V,
      &aL,
      &gamma,
      &g,
      &h,
      &gg,
      &hh, 
    );
    assert!(res == true);
  }
}