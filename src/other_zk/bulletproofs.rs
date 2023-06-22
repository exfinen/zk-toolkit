use crate::building_block::{
  ec_additive_group_ops::EcAdditiveGroupOps,
  ec_cyclic_additive_group::EcCyclicAdditiveGroup,
  ec_point::EcPoint,
  field::{FieldElem, FieldElems},
  vector_ops::{EcPointsWithOps, EcPointWithOps},
};
// implementation based on https://eprint.iacr.org/2017/1066.pdf

pub struct Bulletproofs<'a, const N: usize> {
  group: EcCyclicAdditiveGroup,
  ops: &'a dyn EcAdditiveGroupOps,
}

impl<'a, const N: usize> Bulletproofs<'a, N> {
  pub fn new(
    group: EcCyclicAdditiveGroup,
    ops: &'a dyn EcAdditiveGroupOps,
  ) -> Self {
    Bulletproofs { group, ops }
  }

  pub fn ec_points(&self, ec_points: &'a [EcPoint]) -> EcPointsWithOps<'a> {
    assert!(ec_points.len() > 0);
    let xs = ec_points.iter().map(|x| EcPointWithOps((self.ops, x.clone()))).collect::<_>();
    EcPointsWithOps((self.ops, xs))
  }

  pub fn ec_point1(&self, ec_point: &'a EcPoint) -> EcPointWithOps<'a> {
    EcPointWithOps((self.ops, ec_point.clone()))
  }

  pub fn rand_point(&self) -> EcPointWithOps<'a> {
    let fe = self.group.f_n.rand_elem(true);
    let p = self.ops.scalar_mul(&self.group.g, &fe);
    EcPointWithOps((self.ops, p))
  }

  pub fn rand_points(&self, n: usize) -> Vec<EcPoint> {
    let mut xs = vec![];
    for _ in 0..n {
      xs.push(self.rand_point().into());
    }
    xs
  }

  pub fn scalar_mul(&self, pt: &EcPointWithOps<'a>, fe: &FieldElem) -> EcPointWithOps<'a> {
    let (ops, _) = pt.0;
    let x = self.ops.scalar_mul(&pt, &fe);
    EcPointWithOps((ops, x))
  }

  // P = g^a h^b u^<a,b>
  #[allow(non_snake_case)]
  pub fn inner_product_argument(&self,
    n: usize,
    gg: &EcPointsWithOps,
    hh: &EcPointsWithOps,
    u: &EcPointWithOps,
    P: &EcPointWithOps,
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

      let x = &self.group.f_n.rand_elem(true);

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
    V: &EcPointWithOps,
    aL: &FieldElems,
    gamma: &FieldElem,
    g: &EcPointWithOps,
    h: &EcPointWithOps,
    gg: &EcPointsWithOps,
    hh: &EcPointsWithOps,
    use_inner_product_argument: bool,
  ) -> bool {
    let f = &self.group.f_n;  // prime field of order n

    let one = f.elem(&1u8);
    let two = f.elem(&2u8);
    let one_n = &one.pow_seq(n);
    let two_n = &two.pow_seq(n);

    let aR = &(aL - one_n);
    let alpha = &f.rand_elem(true);
    let A = h * alpha + (gg * aL).sum() + (hh * aR).sum();

    let sL = &f.rand_elems(n, true);
    let sR = &f.rand_elems(n, true);
    let rho = &f.rand_elem(true);
    let S = h * rho + (gg * sL).sum() + (hh * sR).sum();

    let y = &f.rand_elem(true);
    let z = &f.rand_elem(true);

    let y_n = &y.pow_seq(n);
    let l0 = &(aL - (one_n * z));
    let l1 = sL;
    let r0 = &((y_n * (aR + (one_n * z))) + (two_n * z.sq()));
    let r1 = &(y_n * sR);

    let t0 = &(l0 * r0).sum();
    let t1 = &((l1 * r0).sum() + (l0 * r1).sum());
    let t2 = &(l1 * r1).sum();

    let tau1 = &f.rand_elem(true);
    let tau2 = &f.rand_elem(true);
    let T1 = g * t1 + h * tau1;
    let T2 = g * t2 + h * tau2;

    let x = &f.rand_elem(true);

    let t_hat = &(t0 + (t1 * x) + (t2 * x.sq()));
    let tau_x = &(tau2 * x.sq() + (tau1 * x) + (z.sq() * gamma));
    let mu = alpha + (rho * x);

    // (64)
    let hhp = &(hh * &y.inv().pow_seq(n));

    // (65)
    let delta_yz = &((z - z.sq()) * (one_n * y_n).sum()) - (z.cube() * (one_n * two_n).sum());

    let lhs_65 = (g * t_hat) + (h * tau_x);
    let rhs_65 = V * z.sq() + g * delta_yz + T1 * x + T2 * x.sq();
    if lhs_65 != rhs_65 {
      return false;
    }

    // (66), (67)
    let l = &((aL - (one_n * z)) + (sL * x));
    let r = &((y_n * ((aR + (one_n * z)) + (sR * x))) + (two_n * z.sq()));

    let P =
      A
      + S * x
      + (gg * (one_n * z.negate())).sum()
      + (hhp * ((y_n * z) + (two_n * z.sq()))).sum();

    if use_inner_product_argument {
      let u = &self.rand_point();
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
  use crate::building_block::weierstrass_add_ops::JacobianAddOps;

  // gg^z == gg^(ones * z)
  #[test]
  fn test_gg_ones_times_z() {
    let group = EcCyclicAdditiveGroup::secp256k1();
    let ops = JacobianAddOps::new(&group.f);
    let bp: Bulletproofs<2> = Bulletproofs::new(group, &ops);

    let n = 2;
    let z = bp.group.f_n.rand_elem(true);
    let gg = bp.rand_points(n);
    let gg = bp.ec_points(&gg);

    let r1 = &gg * &z;

    let one = bp.group.f_n.elem(&1u8);
    let ones = one.repeat(n);
    let r2 = &gg * &(&ones * &z);

    assert!(r1 == r2);
  }

  #[test]
  fn test_offset_by_negation() {
    let group = EcCyclicAdditiveGroup::secp256k1();
    let ops = JacobianAddOps::new(&group.f);
    let bp: Bulletproofs<2> = Bulletproofs::new(group, &ops);
    let f_n = &bp.group.f_n;
    {
        let z = f_n.elem(&100u8);
        let basis = f_n.elem(&12345u16);

        let r1 = &basis - &z;
        let r2 = &basis + &z.negate();

        assert_eq!(r1, r2);
    }
    {
        let z = f_n.elem(&100u8);
        let basis = f_n.elem(&12345u16);
        let g = bp.ec_point1(&bp.group.g);

        let r1 = bp.scalar_mul(&g, &(&basis - &z));
        let r2 = g * (basis + z.negate());

        assert!(r1 == r2);
    }
  }

  #[test]
  #[allow(non_snake_case)]
  fn test_base_point_field_elem_mul() {
    let group = EcCyclicAdditiveGroup::secp256k1();
    let ops = JacobianAddOps::new(&group.f);
    let bp: Bulletproofs<2> = Bulletproofs::new(group, &ops);
    let f_n = &bp.group.f_n;

    let alpha = &f_n.rand_elem(true);
    let rho = &f_n.rand_elem(true);
    let h = &bp.rand_point();

    let a = h * alpha + h * rho;
    let b = h * (alpha + rho);
    assert!(a == b);
  }

  #[test]
  #[allow(non_snake_case)]
  fn test_field_elems_mul_field_elem() {
    let group = EcCyclicAdditiveGroup::secp256k1();
    let f_n = &group.f_n;

    let x = f_n.elem(&5u8);
    let sL = FieldElems(vec![
      f_n.elem(&2u8),
      f_n.elem(&3u8),
    ]);

    let exp = FieldElems(vec![
      f_n.elem(&10u8),
      f_n.elem(&15u8),
    ]);
    let act = sL * x;
    assert!(act == exp);
  }

  #[test]
  #[allow(non_snake_case)]
  fn test_mul_field_elem_above_order() {
    let group = EcCyclicAdditiveGroup::secp256k1();
    let ops = JacobianAddOps::new(&group.f);
    let bp: Bulletproofs<2> = Bulletproofs::new(group, &ops);
    let f_n = &bp.group.f_n;

    let gg = vec![
      bp.group.g.clone(),
      bp.group.g.clone(),
    ];
    let gg = &bp.ec_points(&gg);

    let order_minus_1 = bp.group.n - &1u8;
    let x = f_n.elem(&order_minus_1);
    let sL = &f_n.rand_elems(gg.len(), true);

    let sLx = &(sL * &x);

    assert!(gg * sLx == (gg * sL) * x);
  }

  #[test]
  #[allow(non_snake_case)]
  fn test_range_proof() {
    let group = EcCyclicAdditiveGroup::secp256k1();
    let ops = JacobianAddOps::new(&group.f);
    let bp: Bulletproofs<2> = Bulletproofs::new(group, &ops);
    let f_n = &bp.group.f_n;

    let aL = FieldElems::new(&vec![
      f_n.elem(&1u8),
      f_n.elem(&0u8),
      f_n.elem(&0u8),
      f_n.elem(&1u8),
    ]);
    let n = aL.len();
    let upsilon = f_n.elem(&9u8);
    let gamma = f_n.rand_elem(true);
    let g = bp.rand_point();
    let h = bp.rand_point();
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