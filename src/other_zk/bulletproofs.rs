use crate::building_block::elliptic_curve::curve_equation::CurveEquation;
use crate::building_block::field::{FieldElem, FieldElems};
use crate::building_block::elliptic_curve::{
  ec_point::EcPoint,
  curve::Curve,
  ec_point_with_ops::{EcPointsWithOps, EcPointWithOps},
  elliptic_curve_point_ops::{
    EllipticCurveField,
    EllipticCurvePointAdd,
    ElllipticCurvePointInv,
  },
};
// implementation based on https://eprint.iacr.org/2017/1066.pdf

pub struct Bulletproofs<const N: usize, T, U>
  where T: EllipticCurveField + EllipticCurvePointAdd + ElllipticCurvePointInv + Clone, U: CurveEquation {
  curve: Box<dyn Curve<T, U>>,
}

impl<'a, T, const N: usize, U> Bulletproofs<N, T, U>
  where T: EllipticCurveField + EllipticCurvePointAdd + ElllipticCurvePointInv + Clone, U: CurveEquation {

  pub fn new(curve: Box<dyn Curve<T, U>>) -> Self {
    Bulletproofs { curve }
  }

  pub fn ec_points(&self, ec_points: &'a [EcPoint]) -> EcPointsWithOps<T> {
    assert!(ec_points.len() > 0);
    let ops = self.curve.ops();
    let xs = ec_points.iter().map(|x| EcPointWithOps((ops.clone(), x.clone()))).collect::<_>();
    EcPointsWithOps((ops, xs))
  }

  pub fn ec_point1(&self, ec_point: &'a EcPoint) -> EcPointWithOps<T> {
    let ops = self.curve.ops();
    EcPointWithOps((ops, ec_point.clone()))
  }

  pub fn rand_point(&self) -> EcPointWithOps<T> {
    let ops = self.curve.ops();
    let group = &self.curve.group();
    let fe = group.rand_elem(true);
    let p = ops.scalar_mul(&self.curve.g(), &fe);
    EcPointWithOps((ops, p))
  }

  pub fn rand_points(&self, n: usize) -> Vec<EcPoint> {
    let mut xs = vec![];
    for _ in 0..n {
      xs.push(self.rand_point().into());
    }
    xs
  }

  pub fn scalar_mul(&self, pt: &EcPointWithOps<T>, fe: &FieldElem) -> EcPointWithOps<T> {
    let ops = self.curve.ops();
    let x = ops.scalar_mul(&pt, &fe);
    EcPointWithOps((ops, x))
  }

  // P = g^a h^b u^<a,b>
  #[allow(non_snake_case)]
  pub fn inner_product_argument(&self,
    n: usize,
    gg: &EcPointsWithOps<T>,
    hh: &EcPointsWithOps<T>,
    u: &EcPointWithOps<T>,
    P: &EcPointWithOps<T>,
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

      let x = &self.curve.group().rand_elem(true);

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
    V: &EcPointWithOps<T>,
    aL: &FieldElems,
    gamma: &FieldElem,
    g: &EcPointWithOps<T>,
    h: &EcPointWithOps<T>,
    gg: &EcPointsWithOps<T>,
    hh: &EcPointsWithOps<T>,
    use_inner_product_argument: bool,
  ) -> bool {
    let f = &self.curve.group();  // prime field of order n

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
  use crate::building_block::elliptic_curve::{
    weierstrass::{
      curves::secp256k1::{Secp256k1, Secp256k1Params},
      jacobian_point_ops::WeierstrassJacobianPointOps,
      equation::WeierstrassEq,
    },
  };

  // gg^z == gg^(ones * z)
  #[test]
  fn test_gg_ones_times_z() {
    let params = Secp256k1Params::new();
    let ops = Box::new(WeierstrassJacobianPointOps::new(&params.f));
    let curve = Box::new(Secp256k1::new(ops, params));
    let bp: Bulletproofs<2, WeierstrassJacobianPointOps, WeierstrassEq> = Bulletproofs::new(curve);
    let group = bp.curve.group();

    let n = 2;
    let z = group.rand_elem(true);
    let gg = bp.rand_points(n);
    let gg = bp.ec_points(&gg);

    let r1 = &gg * &z;

    let one = group.elem(&1u8);
    let ones = one.repeat(n);
    let r2 = &gg * &(&ones * &z);

    assert!(r1 == r2);
  }

  #[test]
  fn test_offset_by_negation() {
    let params = Secp256k1Params::new();
    let ops = Box::new(WeierstrassJacobianPointOps::new(&params.f));
    let curve = Box::new(Secp256k1::new(ops, params.clone()));
    let bp: Bulletproofs<2, WeierstrassJacobianPointOps, WeierstrassEq> = Bulletproofs::new(curve);
    let group = bp.curve.group();
    {
        let z = group.elem(&100u8);
        let basis = group.elem(&12345u16);

        let r1 = &basis - &z;
        let r2 = &basis + &z.negate();

        assert_eq!(r1, r2);
    }
    {
        let z = group.elem(&100u8);
        let basis = group.elem(&12345u16);
        let g = bp.ec_point1(&params.g);

        let r1 = bp.scalar_mul(&g, &(&basis - &z));
        let r2 = g * (basis + z.negate());

        assert!(r1 == r2);
    }
  }

  #[test]
  #[allow(non_snake_case)]
  fn test_base_point_field_elem_mul() {
    let params = Secp256k1Params::new();
    let ops = Box::new(WeierstrassJacobianPointOps::new(&params.f));
    let curve = Box::new(Secp256k1::new(ops, params));
    let bp: Bulletproofs<2, WeierstrassJacobianPointOps, WeierstrassEq> = Bulletproofs::new(curve);
    let group = &bp.curve.group();

    let alpha = &group.rand_elem(true);
    let rho = &group.rand_elem(true);
    let h = &bp.rand_point();

    let a = h * alpha + h * rho;
    let b = h * (alpha + rho);
    assert!(a == b);
  }

  #[test]
  #[allow(non_snake_case)]
  fn test_field_elems_mul_field_elem() {
    let params = Secp256k1Params::new();
    let ops = Box::new(WeierstrassJacobianPointOps::new(&params.f));
    let curve = Box::new(Secp256k1::new(ops, params));
    let bp: Bulletproofs<2, WeierstrassJacobianPointOps, WeierstrassEq> = Bulletproofs::new(curve);
    let group = &bp.curve.group();

    let x = group.elem(&5u8);
    let sL = FieldElems(vec![
      group.elem(&2u8),
      group.elem(&3u8),
    ]);

    let exp = FieldElems(vec![
      group.elem(&10u8),
      group.elem(&15u8),
    ]);
    let act = sL * x;
    assert!(act == exp);
  }

  #[test]
  #[allow(non_snake_case)]
  fn test_mul_field_elem_above_order() {
    let params = Secp256k1Params::new();
    let ops = Box::new(WeierstrassJacobianPointOps::new(&params.f));
    let curve = Box::new(Secp256k1::new(ops, params.clone()));
    let bp: Bulletproofs<2, WeierstrassJacobianPointOps, WeierstrassEq> = Bulletproofs::new(curve);
    let group = &bp.curve.group();

    let gg = vec![
      params.g.clone(),
      params.g.clone(),
    ];
    let gg = &bp.ec_points(&gg);

    let order_minus_1 = params.n - &1u8;
    let x = group.elem(&order_minus_1);
    let sL = &group.rand_elems(gg.len(), true);

    let sLx = &(sL * &x);

    assert!(gg * sLx == (gg * sL) * x);
  }

  #[test]
  #[allow(non_snake_case)]
  fn test_range_proof() {
    let params = Secp256k1Params::new();
    let ops = Box::new(WeierstrassJacobianPointOps::new(&params.f));
    let curve = Box::new(Secp256k1::new(ops, params));
    let bp: Bulletproofs<2, WeierstrassJacobianPointOps, WeierstrassEq> = Bulletproofs::new(curve);
    let group = &bp.curve.group();

    let aL = FieldElems::new(&vec![
      group.elem(&1u8),
      group.elem(&0u8),
      group.elem(&0u8),
      group.elem(&1u8),
    ]);
    let n = aL.len();
    let upsilon = group.elem(&9u8);
    let gamma = group.rand_elem(true);
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