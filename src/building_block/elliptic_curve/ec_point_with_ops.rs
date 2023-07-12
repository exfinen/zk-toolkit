use crate::building_block::field::{FieldElem, FieldElems};
use super::{
  ec_point::EcPoint,
  elliptic_curve_point_ops::{
    EllipticCurveField,
    EllipticCurvePointAdd,
    ElllipticCurvePointInv,
  },
};
use std::ops;
use std::ops::{Index, RangeFrom, RangeTo, Deref};

///////////////////
// EcPointWithOps

#[derive(Clone)]
pub struct EcPointWithOps<'a, T>(pub (&'a T, EcPoint))
  where T: EllipticCurveField + EllipticCurvePointAdd + ElllipticCurvePointInv;

impl<'a, T> Deref for EcPointWithOps<'a, T>
  where T: EllipticCurveField + EllipticCurvePointAdd + ElllipticCurvePointInv {
  type Target = EcPoint;

  fn deref(&self) -> &Self::Target {
    &self.0.1
  }
}

impl<'a, T> PartialEq for EcPointWithOps<'a, T>
  where T: EllipticCurveField + EllipticCurvePointAdd + ElllipticCurvePointInv {

  fn eq(&self, other: &Self) -> bool {
    self.0.1 == other.0.1
  }
}

impl<'a, T> Eq for EcPointWithOps<'a, T>
  where T: EllipticCurveField + EllipticCurvePointAdd + ElllipticCurvePointInv {}

macro_rules! impl_ec_point1_times_field_elem {
  ($rhs: ty, $target: ty) => {
    impl<'a, T> ops::Mul<$rhs> for $target
      where T: EllipticCurveField + EllipticCurvePointAdd + ElllipticCurvePointInv {

      type Output = EcPointWithOps<'a, T>;

      fn mul(self, rhs: $rhs) -> Self::Output {
        let (ops, lhs) = &self.0;
        let x = ops.scalar_mul(&lhs, &rhs.n);
        EcPointWithOps((*ops, x))
      }
    }
  };
}
impl_ec_point1_times_field_elem!(&FieldElem, &EcPointWithOps<'a, T>);
impl_ec_point1_times_field_elem!(FieldElem, &EcPointWithOps<'a, T>);
impl_ec_point1_times_field_elem!(FieldElem, EcPointWithOps<'a, T>);
impl_ec_point1_times_field_elem!(&FieldElem, EcPointWithOps<'a, T>);

macro_rules! impl_ec_point1_plus_ec_point1 {
  ($rhs: ty, $target: ty) => {
    impl<'a, T> ops::Add<$rhs> for $target
      where T: EllipticCurveField + EllipticCurvePointAdd + ElllipticCurvePointInv {
      type Output = EcPointWithOps<'a, T>;

      fn add(self, rhs: $rhs) -> Self::Output {
        let (ops, lhs) = &self.0;
        let x = ops.add(&lhs, &rhs.0.1);
        EcPointWithOps((*ops, x))
      }
    }
  };
}
impl_ec_point1_plus_ec_point1!(EcPointWithOps<'a, T>, &EcPointWithOps<'a, T>);
impl_ec_point1_plus_ec_point1!(&EcPointWithOps<'a, T>, EcPointWithOps<'a, T>);
impl_ec_point1_plus_ec_point1!(EcPointWithOps<'a, T>, EcPointWithOps<'a, T>);
impl_ec_point1_plus_ec_point1!(&EcPointWithOps<'a, T>, &EcPointWithOps<'a, T>);

////////////////////
// EcPointsWithOps

#[derive(Clone)]
pub struct EcPointsWithOps<'a, T>(pub (&'a T, Vec<EcPointWithOps<'a, T>>))
  where T: EllipticCurveField + EllipticCurvePointAdd + ElllipticCurvePointInv;

impl<'a, T> Deref for EcPointsWithOps<'a, T>
  where T: EllipticCurveField + EllipticCurvePointAdd + ElllipticCurvePointInv {
  type Target = [EcPointWithOps<'a, T>];

  fn deref(&self) -> &Self::Target {
    &self.0.1[..]
  }
}

impl<'a, T> PartialEq for EcPointsWithOps<'a, T>
  where T: EllipticCurveField + EllipticCurvePointAdd + ElllipticCurvePointInv {
  fn eq(&self, other: &Self) -> bool {
    self.0.1 == other.0.1
  }
}

impl<'a, T> Index<usize> for EcPointsWithOps<'a, T>
  where T: EllipticCurveField + EllipticCurvePointAdd + ElllipticCurvePointInv {
  type Output = EcPointWithOps<'a, T>;

  fn index(&self, index: usize) -> &Self::Output {
    &self.0.1[index]
  }
}

impl<'a, T> EcPointsWithOps<'a, T>
  where T: EllipticCurveField + EllipticCurvePointAdd + ElllipticCurvePointInv {

  pub fn from(&self, range: RangeFrom<usize>) -> EcPointsWithOps<'a, T> {
    let (ops, _) = self.0;
    let xs: Vec<EcPointWithOps<'a, T>> = (range.start..self.len()).map(|i| {
      let x: &EcPointWithOps<'a, T> = &self[i];
      x.clone()
    }).collect::<Vec<EcPointWithOps<'a, T>>>();
    EcPointsWithOps((ops, xs))
  }

  pub fn to(&self, range: RangeTo<usize>) -> EcPointsWithOps<'a, T> {
    let (ops, _) = self.0;

    if self.len() < range.end {
      return EcPointsWithOps((ops, self.0.1.clone()));
    }
    let xs: Vec<EcPointWithOps<'a, T>> = (0..range.end).map(|i| {
      let x: &EcPointWithOps<'a, T> = &self[i];
      x.clone()
    }).collect::<Vec<EcPointWithOps<'a, T>>>();
    EcPointsWithOps((ops, xs))
  }

  pub fn sum(&self) -> EcPointWithOps<'a, T> {
    assert!(self.len() > 0);
    let (ops, _) = self.0;

    let head = self[0].0.1.clone();
    let tail = &self.from(1..);
    let x: EcPoint = tail.iter().fold(head, |acc, pt| ops.add(&acc, pt));
    EcPointWithOps((ops, x))
  }

}

// returns Hadamard product
macro_rules! impl_ec_points_times_field_elems {
  ($rhs: ty, $target: ty) => {
    impl<'a, T> ops::Mul<$rhs> for $target
      where T: EllipticCurveField + EllipticCurvePointAdd + ElllipticCurvePointInv {
      type Output = EcPointsWithOps<'a, T>;

      fn mul(self, rhs: $rhs) -> Self::Output {
        assert!(self.len() > 0 && self.len() == rhs.len());
        let (ops, lhs) = &self.0;

        let mut xs = vec![];
        for i in 0..self.len() {
          let x = ops.scalar_mul(&lhs[i], &rhs[i]);
          let x = EcPointWithOps((*ops, x));
          xs.push(x);
        }
        EcPointsWithOps((*ops, xs))
      }
    }
  };
}
impl_ec_points_times_field_elems!(&FieldElems, &EcPointsWithOps<'a, T>);
impl_ec_points_times_field_elems!(&FieldElems, EcPointsWithOps<'a, T>);
impl_ec_points_times_field_elems!(FieldElems, &EcPointsWithOps<'a, T>);
impl_ec_points_times_field_elems!(FieldElems, EcPointsWithOps<'a, T>);

// returns Hadamard product
macro_rules! impl_ec_points_times_ec_points {
  ($rhs: ty, $target: ty) => {
    impl<'a, T> ops::Mul<$rhs> for $target
      where T: EllipticCurveField + EllipticCurvePointAdd + ElllipticCurvePointInv {
      type Output = EcPointsWithOps<'a, T>;

      fn mul(self, rhs: $rhs) -> Self::Output {
        assert!(self.len() > 0 && self.len() == rhs.len());
        let (ops, lhs) = &self.0;

        let mut xs = vec![];
        for i in 0..self.len() {
          let x = ops.add(&lhs[i], &rhs[i]);
          let x = EcPointWithOps((*ops, x));
          xs.push(x);
        }
        EcPointsWithOps((*ops, xs))
      }
    }
  };
}
impl_ec_points_times_ec_points!(&EcPointsWithOps<'a, T>, &EcPointsWithOps<'a, T>);
impl_ec_points_times_ec_points!(&EcPointsWithOps<'a, T>, EcPointsWithOps<'a, T>);
impl_ec_points_times_ec_points!(EcPointsWithOps<'a, T>, &EcPointsWithOps<'a, T>);
impl_ec_points_times_ec_points!(EcPointsWithOps<'a, T>, EcPointsWithOps<'a, T>);

// multiply rhs (scalar) to each element
macro_rules! impl_ec_points_times_field_elem {
  ($rhs: ty, $target: ty) => {
    impl<'a, T> ops::Mul<$rhs> for $target
      where T: EllipticCurveField + EllipticCurvePointAdd + ElllipticCurvePointInv {
      type Output = EcPointsWithOps<'a, T>;

      fn mul(self, rhs: $rhs) -> Self::Output {
        assert!(self.len() > 0);
        let (ops, lhs) = &self.0;

        let xs = lhs.iter().map(|pt| {
          let x = ops.scalar_mul(pt, &rhs.n);
          EcPointWithOps((*ops, x))
        }).collect();

        EcPointsWithOps((*ops, xs))
      }
    }
  };
}
impl_ec_points_times_field_elem!(&FieldElem, &EcPointsWithOps<'a, T>);
impl_ec_points_times_field_elem!(&FieldElem, EcPointsWithOps<'a, T>);
impl_ec_points_times_field_elem!(FieldElem, &EcPointsWithOps<'a, T>);
impl_ec_points_times_field_elem!(FieldElem, EcPointsWithOps<'a, T>);

macro_rules! impl_ec_points_plus_ec_points {
  ($rhs: ty, $target: ty) => {
    impl<'a, T> ops::Add<$rhs> for $target
      where T: EllipticCurveField + EllipticCurvePointAdd + ElllipticCurvePointInv {
      type Output = EcPointsWithOps<'a, T>;

      fn add(self, rhs: $rhs) -> Self::Output {
        assert!(self.len() > 0 && self.len() == rhs.len());
        let (ops, lhs) = &self.0;

        let mut xs = vec![];
        for i in 0..self.len() {
          let x = ops.add(&lhs[i], &rhs[i]);
          let x = EcPointWithOps((*ops, x));
          xs.push(x);
        }
        EcPointsWithOps((*ops, xs))
      }
    }
  };
}
impl_ec_points_plus_ec_points!(&EcPointsWithOps<'a, T>, &EcPointsWithOps<'a, T>);
impl_ec_points_plus_ec_points!(&EcPointsWithOps<'a, T>, EcPointsWithOps<'a, T>);
impl_ec_points_plus_ec_points!(EcPointsWithOps<'a, T>, &EcPointsWithOps<'a, T>);
impl_ec_points_plus_ec_points!(EcPointsWithOps<'a, T>, EcPointsWithOps<'a, T>);

macro_rules! impl_ec_points_minus_ec_points {
  ($rhs: ty, $target: ty) => {
    impl<'a, T> ops::Sub<$rhs> for $target
      where T: EllipticCurveField + EllipticCurvePointAdd + ElllipticCurvePointInv {
      type Output = EcPointsWithOps<'a, T>;

      fn sub(self, rhs: $rhs) -> Self::Output {
        assert!(self.len() > 0 && self.len() == rhs.len());
        let (ops, lhs) = &self.0;

        let mut xs = vec![];
        for i in 0..self.len() {
          let x = ops.add(&lhs[i], &ops.inv(&rhs[i]));
          let x = EcPointWithOps((*ops, x));
          xs.push(x);
        }
        EcPointsWithOps((*ops, xs))
      }
    }
  };
}
impl_ec_points_minus_ec_points!(&EcPointsWithOps<'a, T>, &EcPointsWithOps<'a, T>);
impl_ec_points_minus_ec_points!(&EcPointsWithOps<'a, T>, EcPointsWithOps<'a, T>);
impl_ec_points_minus_ec_points!(EcPointsWithOps<'a, T>, &EcPointsWithOps<'a, T>);
impl_ec_points_minus_ec_points!(EcPointsWithOps<'a, T>, EcPointsWithOps<'a, T>);

#[cfg(test)]
mod tests {
  use super::*;
  use crate::building_block::elliptic_curve::weierstrass::{
    curves::secp256k1::{
      Secp256k1,
      Secp256k1Params,
    },
    jacobian_point_ops::WeierstrassJacobianPointOps,
  };

  #[test]
  fn ec_point1_eq() {
    let params = Secp256k1Params::new();
    let ops = WeierstrassJacobianPointOps::new(&params.f);

    let p = EcPointWithOps((&ops, params.g));
    assert!(p == p);

    let q = &p + &p;
    assert!(p != q);
  }

  #[test]
  fn ec_point1_times_field_elem() {
    let params = Secp256k1Params::new();
    let ops = WeierstrassJacobianPointOps::new(&params.f);

    let p = params.g;
    let p = EcPointWithOps((&ops, p));
    let p_plus_p = &p + &p;

    let two = params.f_n.elem(&2u8);
    let p_times_2 = p * two;

    assert!(p_plus_p == p_times_2);
  }

  #[test]
  fn ec_point1_plus_ec_point1() {
    let params = Secp256k1Params::new();
    let ops = WeierstrassJacobianPointOps::new(&params.f);

    let g = params.g;
    let g = EcPointWithOps((&ops, g));

    let g2 = &g + &g;
    let g4 = &g2 + &g2;
    let p4 = &g * params.f.elem(&4u8);
    assert!(&g4 == &p4);

    let p8 = &g * params.f.elem(&8u8);
    let g8 = &g4 + &g4;
    assert!(&g8 == &p8);
  }

  #[test]
  fn ec_points_eq() {
    let params = Secp256k1Params::new();
    let ops = WeierstrassJacobianPointOps::new(&params.f);

    let p1 = EcPointWithOps((&ops, params.g));
    let p2 = &p1 * params.f_n.elem(&2u8);

    let ps1 = EcPointsWithOps((&ops, vec![p1.clone(), p2.clone()]));
    let ps2 = EcPointsWithOps((&ops, vec![p2.clone(), p1.clone()]));
    let ps3 = EcPointsWithOps((&ops, vec![p1.clone(), p2.clone()]));

    assert!(ps1 == ps1);
    assert!(ps1 != ps2);
    assert!(ps1 == ps3);
  }

  #[test]
  fn ec_points_index() {
    let params = Secp256k1Params::new();
    let ops = WeierstrassJacobianPointOps::new(&params.f);

    let p1 = EcPointWithOps((&ops, params.g));
    let p2 = &p1 * params.f.elem(&2u8);
    let p3 = &p1 * params.f.elem(&3u8);

    let ps = EcPointsWithOps((&ops, vec![p1.clone(), p2.clone(), p3.clone()]));
    assert!(ps.len() == 3);
    assert!(ps[0] == p1);
    assert!(ps[1] == p2);
    assert!(ps[2] == p3);
  }

  #[test]
  fn ec_points_from() {
    let params = Secp256k1Params::new();
    let ops = WeierstrassJacobianPointOps::new(&params.f);

    let p1 = EcPointWithOps((&ops, params.g));
    let p2 = &p1 * params.f.elem(&2u8);
    let p3 = &p1 * params.f.elem(&3u8);

    let vec = vec![p1.clone(), p2.clone(), p3.clone()];
    let ps = EcPointsWithOps((&ops, vec));

    {
      let res = ps.from(0..);
      assert!(res.len() == 3);
      assert!(&res[0] == &p1);
      assert!(&res[1] == &p2);
      assert!(&res[2] == &p3);
    }
    {
      let res = ps.from(1..);
      assert!(res.len() == 2);
      assert!(&res[0] == &p2);
      assert!(&res[1] == &p3);
    }
    {
      let res = ps.from(2..);
      assert!(res.len() == 1);
      assert!(&res[0] == &p3);
    }
    {
      let res = ps.from(3..);
      assert!(res.len() == 0);
    }
    {
      let res = ps.from(4..);
      assert!(res.len() == 0);
    }
  }

  #[test]
  fn ec_points_to() {
    let params = Secp256k1Params::new();
    let ops = WeierstrassJacobianPointOps::new(&params.f);

    let p1 = EcPointWithOps((&ops, params.g));
    let p2 = &p1 * params.f.elem(&2u8);
    let p3 = &p1 * params.f.elem(&3u8);

    let vec = vec![p1.clone(), p2.clone(), p3.clone()];
    let ps = EcPointsWithOps((&ops, vec));

    {
      let res = ps.to(..0);
      assert!(res.len() == 0);
    }
    {
      let res = ps.to(..1);
      assert!(res.len() == 1);
      assert!(&res[0] == &p1);
    }
    {
      let res = ps.to(..2);
      assert!(res.len() == 2);
      assert!(&res[0] == &p1);
      assert!(&res[1] == &p2);
    }
    {
      let res = ps.to(..3);
      assert!(res.len() == 3);
      assert!(&res[0] == &p1);
      assert!(&res[1] == &p2);
      assert!(&res[2] == &p3);
    }
    {
      let res = ps.to(..4);
      assert!(res.len() == 3);
      assert!(&res[0] == &p1);
      assert!(&res[1] == &p2);
      assert!(&res[2] == &p3);
    }
  }

  #[test]
  fn ec_points_sum() {
    let params = Secp256k1Params::new();
    let ops = WeierstrassJacobianPointOps::new(&params.f);

    let p1 = EcPointWithOps((&ops, params.g));
    let p2 = &p1 * params.f.elem(&2u8);
    let p3 = &p1 * params.f.elem(&3u8);

    let vec = vec![p1.clone(), p2.clone(), p3.clone()];
    let ps = EcPointsWithOps((&ops, vec));
    let act = ps.sum();
    let exp = &p1 * params.f.elem(&6u8);

    assert!(act == exp);
  }

  #[test]
  fn ec_points_times_field_elem() {
    let params = Secp256k1Params::new();
    let ops = WeierstrassJacobianPointOps::new(&params.f);

    let p1 = EcPointWithOps((&ops, params.g));
    let p2 = &p1 * params.f.elem(&2u8);
    let p4 = &p1 * params.f.elem(&4u8);

    let act = EcPointsWithOps((&ops, vec![p1, p2.clone()])) * params.f.elem(&2u8);
    let exp = EcPointsWithOps((&ops, vec![p2, p4]));

    assert!(act == exp);
  }

  #[test]
  fn ec_points_plus_ec_points() {
    let params = Secp256k1Params::new();
    let ops = WeierstrassJacobianPointOps::new(&params.f);

    let p1 = EcPointWithOps((&ops, params.g));
    let p2 = &p1 * params.f.elem(&2u8);

    let ps = EcPointsWithOps((&ops, vec![p1, p2.clone()]));
    let ps3 = &ps * params.f.elem(&3u8);
    let ps2 = &ps * params.f.elem(&2u8);

    let act = ps + ps2;
    let exp = ps3;

    assert!(act == exp);
  }

  #[test]
  fn ec_points_minus_ec_points() {
    let params = Secp256k1Params::new();
    let ops = WeierstrassJacobianPointOps::new(&params.f);

    let g = &EcPointWithOps((&ops, params.g));
    let g2 = g * params.f.elem(&2u8);
    let zero = &EcPointWithOps((&ops, ops.get_zero(&params.f)));

    let g2s = EcPointsWithOps((&ops, vec![g2.clone(), g2.clone()]));
    let zeros = EcPointsWithOps((&ops, vec![zero.clone(), zero.clone()]));

    let act = &g2s - &g2s;
    let exp = zeros;

    assert!(act == exp);
  }
}
