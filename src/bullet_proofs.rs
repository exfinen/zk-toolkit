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

  pub fn vector_add(&self, xs: &[EcPoint1<'a>]) -> EcPoint1<'a> {
    assert!(xs.len() > 0);
    let head = &xs[0];
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

  // P = g^a h^b u^<a,b>
  #[allow(non_snake_case)]
  pub fn inner_product_argument(&self,
    n: usize,
    gg: &EcPoints<'a>,
    hh: &EcPoints<'a>,
    u: &EcPoint1<'a>,
    P: &EcPoint1<'a>,  
    a: &FieldElems,
    b: &FieldElems, 
  ) -> bool {
    if n == 1 {
        let c = (a * b).sum();
        let rhs = (gg * a).sum() + (hh * b).sum() + u * c;
        P == &rhs
    }
    else {
      let np = n / 2;

      let cL = (a.to(..np) * b.from(np..)).sum();
      let cR = (a.from(np..) * b.to(..np)).sum();

      let L = (gg.from(np..) * a.to(..np)).sum() + (hh.to(..np) * b.from(np..)).sum() + u * cL;
      let R = (gg.to(..np) * a.from(np..)).sum() + (hh.from(np..) * b.to(..np)).sum() + u * cR;

      let x = &self.curve.n().rand_elem(true);

      let ggp = (gg.to(..np) * x.inv()) + (gg.from(np..) * x);
      let hhp = (hh.to(..np) * x) + (hh.from(np..) * x.inv());

      let Pp = (L * x.sq()) + P + (R * x.sq().inv());

      let ap = a.to(..np) * x + a.from(np..) * x.inv();
      let bp = b.to(..np) * x.inv() + b.from(np..) * x;

      self.inner_product_argument(
        np, &ggp, &hhp, u, &Pp, &ap, &bp)
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
    use_inner_product_argument: bool,
  ) -> bool {
    let co = self.curve.n();

    let one = co.elem(&1u8);
    let two = co.elem(&2u8);
    let one_n = &one.pow_seq(n);
    let two_n = &two.pow_seq(n);

    let aR = &(aL - one_n);
    let alpha = &co.rand_elem(true);
    let A = self.vector_add(&vec![
      h * alpha,
      (gg * aL).sum(),
      (hh * aR).sum(),
    ]);

    let sL = &co.rand_elems(n, true);
    let sR = &co.rand_elems(n, true);
    let rho = &co.rand_elem(true);
    let S = self.vector_add(&vec![
      h * rho,
      (gg * sL).sum(),
      (hh * sR).sum(),
    ]);

    let y = &co.rand_elem(true);
    let z = &co.rand_elem(true);

    let y_n = &y.pow_seq(n);
    let l0 = aL - (one_n * z);
    let l1 = sL;
    let r0 = (y_n * (aR + (one_n * z))) + (two_n * z.sq());
    let r1 = y_n * sR;

    let t0 = &(&l0 * &r0).sum();
    let t1 = &((l1 * &r0).sum() + (&l0 * &r1).sum());
    let t2 = &(l1 * &r1).sum();

    let tau1 = &co.rand_elem(true);
    let tau2 = &co.rand_elem(true);
    let T1 = &self.vector_add(&vec![
      g * t1,
      h * tau1,
    ]);
    let T2 = &self.vector_add(&vec![
      g * t2,
      h * tau2,
    ]);

    let x = &co.rand_elem(true);

    let t_hat = &(t0 + (t1 * x) + (t2 * x.sq()));
    let tau_x = &(tau2 * x.sq() + (tau1 * x) + (z.sq() * gamma));
    let mu = alpha + (rho * x);

    // (64)
    let hhp = &(hh * &y.inv().pow_seq(n));

    // (65)
    let delta_yz = &((z - z.sq()) * (one_n * y_n).sum()) - (z.cube() * (one_n * two_n).sum());

    let lhs_65 = (g * t_hat) + (h * tau_x);
    let rhs_65 = self.vector_add(&vec![
      V * z.sq(),
      g * delta_yz,
      T1 * x,
      T2 * x.sq(),
    ]);
    if lhs_65 != rhs_65 {
      return false;
    }

    // (66), (67)
    let l = &((aL - (one_n * z)) + (sL * x));
    let r = &((y_n * ((aR + (one_n * z)) + (sR * x))) + (two_n * z.sq()));

    let P = self.vector_add(&vec![
      A,
      S * x,
      (gg * (one_n * z.negate())).sum(),
      (hhp * ((y_n * z) + (two_n * z.sq()))).sum(),
    ]);

    if use_inner_product_argument {
      let u = &self.rand_point();
      let u = &self.ec_point(u);
      let Pp = &(P + h * mu.negate() + u * (l * r).sum());
      self.inner_product_argument(n, gg, hhp, u, Pp, l, r)

    } else {
      let rhs_66_67 = ((h * mu) + (gg * l).sum()) + (hhp * r).sum();
      if P != rhs_66_67 {
        return false;
      }

      // (68)
      let rhs_68 = (l * r).sum();

      t_hat == &rhs_68
    }
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
  fn test_mul_field_elem_above_order() {
    use num_bigint::BigUint;

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

    let order_minus_1 = co.order.as_ref() - BigUint::from(1u8);
    let x = co.elem(&order_minus_1);
    let sL = &co.rand_elems(n, true);
    
    let sLx = &(sL * &x);

    assert!(gg * sLx == (gg * sL) * x);
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

    for use_inner_product_argument in [true, false] {
      let res = bp.range_proof(
        n,
        &V,
        &aL,
        &gamma,
        &g,
        &h,
        &gg,
        &hh, 
        use_inner_product_argument,
      );
      assert!(res == true);
    }
  }
}