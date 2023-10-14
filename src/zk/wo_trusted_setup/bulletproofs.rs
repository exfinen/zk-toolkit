use crate::building_block::{
  field::{
    prime_field_elem::PrimeFieldElem,
    prime_field_elems::PrimeFieldElems,
  },
  curves::secp256k1::{
    affine_point::AffinePoint,
    affine_points::AffinePoints,
  },
};

// implementation based on https://eprint.iacr.org/2017/1066.pdf

pub struct Bulletproofs();

impl Bulletproofs {
  // P = g^a h^b u^<a,b>
  #[allow(non_snake_case)]
  pub fn inner_product_argument(
    n: &usize,
    gg: &AffinePoints,
    hh: &AffinePoints,
    u: &AffinePoint,
    P: &AffinePoint,
    a: &PrimeFieldElems,
    b: &PrimeFieldElems,
  ) -> bool {
    if n == &1 {
        let c = (a * b).sum();
        let rhs = (gg * a).sum() + (hh * b).sum() + u * c;
        P == &rhs
    }
    else {
      let np = n / 2;

      let cL = (a.to(np) * b.from(np)).sum();
      let cR = (a.from(np) * b.to(np)).sum();

      let L = (gg.from(np) * a.to(np)).sum() + (hh.to(np) * b.from(np)).sum() + u * cL;
      let R = (gg.to(np) * a.from(np)).sum() + (hh.from(np) * b.to(np)).sum() + u * cR;

      let x = &AffinePoint::curve_group().rand_elem(true);

      let ggp = (gg.to(np) * x.inv()) + (gg.from(np) * x);
      let hhp = (hh.to(np) * x) + (hh.from(np) * x.inv());

      let Pp = (L * x.sq()) + P + (R * x.sq().inv());

      let ap = a.to(np) * x + a.from(np) * x.inv();
      let bp = b.to(np) * x.inv() + b.from(np) * x;

      Bulletproofs::inner_product_argument(
        &np, &ggp, &hhp, u, &Pp, &ap, &bp)
    }
  }

  #[allow(non_snake_case)]
  pub fn range_proof(
    n: &usize,
    V: &AffinePoint,
    aL: &PrimeFieldElems,
    gamma: &PrimeFieldElem,
    g: &AffinePoint,
    h: &AffinePoint,
    gg: &AffinePoints,
    hh: &AffinePoints,
    use_inner_product_argument: bool,
  ) -> bool {
    let f_n = AffinePoint::curve_group();

    let one = f_n.elem(&1u8);
    let two = f_n.elem(&2u8);
    let one_n = &one.pow_seq(n);
    let two_n = &two.pow_seq(n);

    let aR = &(aL - one_n);
    let alpha = &f_n.rand_elem(true);
    let A = h * alpha + (gg * aL).sum() + (hh * aR).sum();

    let sL = &f_n.rand_elems(n, true);
    let sR = &f_n.rand_elems(n, true);
    let rho = &f_n.rand_elem(true);
    let S = h * rho + (gg * sL).sum() + (hh * sR).sum();

    let y = &f_n.rand_elem(true);
    let z = &f_n.rand_elem(true);

    let y_n = &y.pow_seq(n);
    let l0 = &(aL - (one_n * z));
    let l1 = sL;
    let r0 = &((y_n * (aR + (one_n * z))) + (two_n * z.sq()));
    let r1 = &(y_n * sR);

    let t0 = &(l0 * r0).sum();
    let t1 = &((l1 * r0).sum() + (l0 * r1).sum());
    let t2 = &(l1 * r1).sum();

    let tau1 = &f_n.rand_elem(true);
    let tau2 = &f_n.rand_elem(true);
    let T1 = g * t1 + h * tau1;
    let T2 = g * t2 + h * tau2;

    let x = &f_n.rand_elem(true);

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
      let u = AffinePoint::rand_point(true);
      let Pp = &(P + h * mu.negate() + &u * (l * r).sum());
      Bulletproofs::inner_product_argument(&n, gg, hhp, &u, Pp, l, r)

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

  // gg^z == gg^(ones * z)
  #[test]
  fn test_gg_ones_times_z() {
    let curve_group = AffinePoint::curve_group();

    let n = 2;
    let z = curve_group.rand_elem(true);
    let gg = AffinePoints::rand_points(true, &n);

    let r1 = &gg * &z;

    let one = curve_group.elem(&1u8);
    let ones = one.repeat(&n);
    let r2 = &gg * &(&ones * &z);

    assert!(r1 == r2);
  }

  #[test]
  fn test_offset_by_negation() {
    let curve_group = AffinePoint::curve_group();
    {
        let z = curve_group.elem(&100u8);
        let basis = curve_group.elem(&12345u16);

        let r1 = &basis - &z;
        let r2 = &basis + &z.negate();

        assert_eq!(r1, r2);
    }
    {
        let z = curve_group.elem(&100u8);
        let basis = curve_group.elem(&12345u16);
        let g = &AffinePoint::g();

        let r1 = g * (&basis - &z);
        let r2 = g * (basis + z.negate());

        assert!(r1 == r2);
    }
  }

  #[test]
  #[allow(non_snake_case)]
  fn test_base_point_field_elem_mul() {
    let curve_group = AffinePoint::curve_group();

    let alpha = &curve_group.rand_elem(true);
    let rho = &curve_group.rand_elem(true);
    let h = &AffinePoint::rand_point(true);

    let a = h * alpha + h * rho;
    let b = h * (alpha + rho);
    assert!(a == b);
  }

  #[test]
  #[allow(non_snake_case)]
  fn test_field_elems_mul_field_elem() {
    let curve_group = AffinePoint::curve_group();

    let x = curve_group.elem(&5u8);
    let sL = PrimeFieldElems(vec![
      curve_group.elem(&2u8),
      curve_group.elem(&3u8),
    ]);

    let exp = PrimeFieldElems(vec![
      curve_group.elem(&10u8),
      curve_group.elem(&15u8),
    ]);
    let act = sL * x;
    assert!(act == exp);
  }

  #[test]
  #[allow(non_snake_case)]
  fn test_mul_field_elem_above_order() {
    let curve_group = AffinePoint::curve_group();
    let g = &AffinePoint::g();
    let gg = &AffinePoints::new(&vec![
      g.clone(),
      g.clone(),
    ]);

    let order_minus_1 = curve_group.order_ref() - &1u8;
    let x = curve_group.elem(&order_minus_1);
    let sL = &curve_group.rand_elems(&gg.len(), true);

    let sLx = &(sL * &x);

    assert!(gg * sLx == (gg * sL) * x);
  }

  #[test]
  #[allow(non_snake_case)]
  fn test_range_proof() {
    let curve_group = &AffinePoint::curve_group();

    let aL = PrimeFieldElems::new(&vec![
      curve_group.elem(&1u8),
      curve_group.elem(&0u8),
      curve_group.elem(&0u8),
      curve_group.elem(&1u8),
    ]);
    let n = aL.len();
    let upsilon = curve_group.elem(&9u8);
    let gamma = curve_group.rand_elem(true);
    let g = AffinePoint::rand_point(true);
    let h = AffinePoint::rand_point(true);
    let gg = AffinePoints::rand_points(true, &n);
    let hh = AffinePoints::rand_points(true, &n);
    let V = (&h * &gamma) + (&g * &upsilon);

    for use_inner_product_argument in [false, true] {
      let res = Bulletproofs::range_proof(
        &n,
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
