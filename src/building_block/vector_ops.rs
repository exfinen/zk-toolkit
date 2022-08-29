use crate::building_block::elliptic_curve::AddOps;
use crate::building_block::field::{FieldElem, FieldElems};
use crate::building_block::ec_point::EcPoint;
use std::ops;
use std::ops::{Index, RangeFrom, RangeTo, Deref};

///////////////
// EcPoint1

#[derive(Clone)]
pub struct EcPoint1<'a>(pub (&'a dyn AddOps, EcPoint));

impl<'a> Deref for EcPoint1<'a> {
  type Target = EcPoint;

  fn deref(&self) -> &Self::Target {
    &self.0.1
  }
}

impl<'a> PartialEq for EcPoint1<'a> {
  fn eq(&self, other: &Self) -> bool {
    self.0.1 == other.0.1
  }
}

impl<'a> Eq for EcPoint1<'a> {}

macro_rules! impl_ec_point1_times_field_elem {
  ($rhs: ty, $target: ty) => {
    impl<'a> ops::Mul<$rhs> for $target {
      type Output = EcPoint1<'a>;

      fn mul(self, rhs: $rhs) -> Self::Output {
        let (ops, lhs) = &self.0;
        let x = ops.scalar_mul(&lhs, &rhs.n);
        EcPoint1((*ops, x))
      }
    }
  };
}
impl_ec_point1_times_field_elem!(&FieldElem, &EcPoint1<'a>);
impl_ec_point1_times_field_elem!(FieldElem, &EcPoint1<'a>);
impl_ec_point1_times_field_elem!(FieldElem, EcPoint1<'a>);
impl_ec_point1_times_field_elem!(&FieldElem, EcPoint1<'a>);

macro_rules! impl_ec_point1_plus_ec_point1 {
  ($rhs: ty, $target: ty) => {
    impl<'a> ops::Add<$rhs> for $target {
      type Output = EcPoint1<'a>;

      fn add(self, rhs: $rhs) -> Self::Output {
        let (ops, lhs) = &self.0;
        let x = ops.add(&lhs, &rhs.0.1);
        EcPoint1((*ops, x))
      }
    }
  };
}
impl_ec_point1_plus_ec_point1!(EcPoint1<'a>, &EcPoint1<'a>);
impl_ec_point1_plus_ec_point1!(&EcPoint1<'a>, EcPoint1<'a>);
impl_ec_point1_plus_ec_point1!(EcPoint1<'a>, EcPoint1<'a>);
impl_ec_point1_plus_ec_point1!(&EcPoint1<'a>, &EcPoint1<'a>);

///////////////
// EcPoints

#[derive(Clone)]
pub struct EcPoints<'a>(pub (&'a dyn AddOps, Vec<EcPoint1<'a>>));

impl<'a> Deref for EcPoints<'a> {
  type Target = [EcPoint1<'a>];

  fn deref(&self) -> &Self::Target {
    &self.0.1[..]
  }
}

impl<'a> PartialEq for EcPoints<'a> {
  fn eq(&self, other: &Self) -> bool {
    self.0.1 == other.0.1
  }
}

impl<'a> Index<usize> for EcPoints<'a> {
  type Output = EcPoint1<'a>;

  fn index(&self, index: usize) -> &Self::Output {
    &self.0.1[index]
  }
}

impl<'a> EcPoints<'a> {
  pub fn from(&self, range: RangeFrom<usize>) -> EcPoints<'a> {
    let (ops, _) = self.0;
    let xs: Vec<EcPoint1<'a>> = (range.start..self.len()).map(|i| {
      let x: &EcPoint1<'a> = &self[i];
      x.clone()
    }).collect::<Vec<EcPoint1<'a>>>();
    EcPoints((ops, xs))
  }

  pub fn to(&self, range: RangeTo<usize>) -> EcPoints<'a> {
    let (ops, _) = self.0;

    if self.len() < range.end {
      return EcPoints((ops, self.0.1.clone()));
    }
    let xs: Vec<EcPoint1<'a>> = (0..range.end).map(|i| {
      let x: &EcPoint1<'a> = &self[i];
      x.clone()
    }).collect::<Vec<EcPoint1<'a>>>();
    EcPoints((ops, xs))
  }

  pub fn sum(&self) -> EcPoint1<'a> {
    assert!(self.len() > 0);
    let (ops, _) = self.0;

    let head = self[0].0.1.clone();
    let tail = &self.from(1..);
    let x: EcPoint = tail.iter().fold(head, |acc, pt| ops.add(&acc, pt));
    EcPoint1((ops, x))
  }

}

// returns Hadamard product
macro_rules! impl_ec_points_times_field_elems {
  ($rhs: ty, $target: ty) => {
    impl<'a> ops::Mul<$rhs> for $target {
      type Output = EcPoints<'a>;

      fn mul(self, rhs: $rhs) -> Self::Output {
        assert!(self.len() > 0 && self.len() == rhs.len());
        let (ops, lhs) = &self.0;

        let mut xs = vec![];
        for i in 0..self.len() {
          let x = ops.scalar_mul(&lhs[i], &rhs[i]);
          let x = EcPoint1((*ops, x));
          xs.push(x);
        }
        EcPoints((*ops, xs))
      }
    }
  };
}
impl_ec_points_times_field_elems!(&FieldElems, &EcPoints<'a>);
impl_ec_points_times_field_elems!(&FieldElems, EcPoints<'a>);
impl_ec_points_times_field_elems!(FieldElems, &EcPoints<'a>);
impl_ec_points_times_field_elems!(FieldElems, EcPoints<'a>);

// returns Hadamard product
macro_rules! impl_ec_points_times_ec_points {
  ($rhs: ty, $target: ty) => {
    impl<'a> ops::Mul<$rhs> for $target {
      type Output = EcPoints<'a>;

      fn mul(self, rhs: $rhs) -> Self::Output {
        assert!(self.len() > 0 && self.len() == rhs.len());
        let (ops, lhs) = &self.0;

        let mut xs = vec![];
        for i in 0..self.len() {
          let x = ops.add(&lhs[i], &rhs[i]);
          let x = EcPoint1((*ops, x));
          xs.push(x);
        }
        EcPoints((*ops, xs))
      }
    }
  };
}
impl_ec_points_times_ec_points!(&EcPoints<'a>, &EcPoints<'a>);
impl_ec_points_times_ec_points!(&EcPoints<'a>, EcPoints<'a>);
impl_ec_points_times_ec_points!(EcPoints<'a>, &EcPoints<'a>);
impl_ec_points_times_ec_points!(EcPoints<'a>, EcPoints<'a>);

// multiply rhs (scalar) to each element 
macro_rules! impl_ec_points_times_field_elem {
  ($rhs: ty, $target: ty) => {
    impl<'a> ops::Mul<$rhs> for $target {
      type Output = EcPoints<'a>;

      fn mul(self, rhs: $rhs) -> Self::Output {
        assert!(self.len() > 0);
        let (ops, lhs) = &self.0;

        let xs = lhs.iter().map(|pt| {
          let x = ops.scalar_mul(pt, &rhs.n);
          EcPoint1((*ops, x))
        }).collect();

        EcPoints((*ops, xs))
      }
    }
  };
}
impl_ec_points_times_field_elem!(&FieldElem, &EcPoints<'a>);
impl_ec_points_times_field_elem!(&FieldElem, EcPoints<'a>);
impl_ec_points_times_field_elem!(FieldElem, &EcPoints<'a>);
impl_ec_points_times_field_elem!(FieldElem, EcPoints<'a>);

macro_rules! impl_ec_points_plus_ec_points {
  ($rhs: ty, $target: ty) => {
    impl<'a> ops::Add<$rhs> for $target {
      type Output = EcPoints<'a>;

      fn add(self, rhs: $rhs) -> Self::Output {
        assert!(self.len() > 0 && self.len() == rhs.len());
        let (ops, lhs) = &self.0;

        let mut xs = vec![];
        for i in 0..self.len() {
          let x = ops.add(&lhs[i], &rhs[i]);
          let x = EcPoint1((*ops, x));
          xs.push(x);
        }
        EcPoints((*ops, xs))
      }
    }
  };
}
impl_ec_points_plus_ec_points!(&EcPoints<'a>, &EcPoints<'a>);
impl_ec_points_plus_ec_points!(&EcPoints<'a>, EcPoints<'a>);
impl_ec_points_plus_ec_points!(EcPoints<'a>, &EcPoints<'a>);
impl_ec_points_plus_ec_points!(EcPoints<'a>, EcPoints<'a>);

macro_rules! impl_ec_points_minus_ec_points {
  ($rhs: ty, $target: ty) => {
    impl<'a> ops::Sub<$rhs> for $target {
      type Output = EcPoints<'a>;

      fn sub(self, rhs: $rhs) -> Self::Output {
        assert!(self.len() > 0 && self.len() == rhs.len());
        let (ops, lhs) = &self.0;

        let mut xs = vec![];
        for i in 0..self.len() {
          let x = ops.add(&lhs[i], &ops.inv(&rhs[i]));
          let x = EcPoint1((*ops, x));
          xs.push(x);
        }
        EcPoints((*ops, xs))
      }
    }
  };
}
impl_ec_points_minus_ec_points!(&EcPoints<'a>, &EcPoints<'a>);
impl_ec_points_minus_ec_points!(&EcPoints<'a>, EcPoints<'a>);
impl_ec_points_minus_ec_points!(EcPoints<'a>, &EcPoints<'a>);
impl_ec_points_minus_ec_points!(EcPoints<'a>, EcPoints<'a>);

#[cfg(test)]
mod tests {
  use super::*;
  use crate::building_block::weierstrass_eq::WeierstrassEq;
  use crate::building_block::weierstrass_add_ops::JacobianAddOps;
  use crate::building_block::elliptic_curve::EllipticCurve;

  #[test]
  fn ec_point1_eq() {
    let curve = WeierstrassEq::secp256k1();
    let ops = JacobianAddOps::new();

    let p = curve.g();
    let p = EcPoint1((&ops, p));
    assert!(p == p);

    let q = &p + &p;
    assert!(p != q);
  }
  
  #[test]
  fn ec_point1_times_field_elem() {
    let curve = WeierstrassEq::secp256k1();
    let ops = JacobianAddOps::new();

    let p = curve.g();
    let p = EcPoint1((&ops, p));
    let p_plus_p = &p + &p;

    let two = curve.f.elem(&2u8);
    let p_times_2 = p * two;

    assert!(p_plus_p == p_times_2);
  }
  
  #[test]
  fn ec_point1_plus_ec_point1() {
    let curve = WeierstrassEq::secp256k1();
    let ops = JacobianAddOps::new();

    let g = curve.g();
    let g = EcPoint1((&ops, g));

    let g2 = &g + &g;
    let g4 = &g2 + &g2;
    let p4 = &g * curve.f.elem(&4u8);
    assert!(&g4 == &p4);

    let p8 = &g * curve.f.elem(&8u8);
    let g8 = &g4 + &g4;
    assert!(&g8 == &p8);
  }
  
  #[test]
  fn ec_points_eq() {
    let curve = WeierstrassEq::secp256k1();
    let ops = JacobianAddOps::new();

    let p1 = EcPoint1((&ops, curve.g()));
    let p2 = &p1 * curve.f.elem(&2u8);

    let ps1 = EcPoints((&ops, vec![p1.clone(), p2.clone()]));
    let ps2 = EcPoints((&ops, vec![p2.clone(), p1.clone()]));
    let ps3 = EcPoints((&ops, vec![p1.clone(), p2.clone()]));

    assert!(ps1 == ps1);
    assert!(ps1 != ps2);
    assert!(ps1 == ps3);
  }
  
  #[test]
  fn ec_points_index() {
    let curve = WeierstrassEq::secp256k1();
    let ops = JacobianAddOps::new();

    let p1 = EcPoint1((&ops, curve.g()));
    let p2 = &p1 * curve.f.elem(&2u8);
    let p3 = &p1 * curve.f.elem(&3u8);

    let ps = EcPoints((&ops, vec![p1.clone(), p2.clone(), p3.clone()]));
    assert!(ps.len() == 3);
    assert!(ps[0] == p1);
    assert!(ps[1] == p2);
    assert!(ps[2] == p3);
  }

  #[test]
  fn ec_points_from() {
    let curve = WeierstrassEq::secp256k1();
    let ops = JacobianAddOps::new();

    let p1 = EcPoint1((&ops, curve.g()));
    let p2 = &p1 * curve.f.elem(&2u8);
    let p3 = &p1 * curve.f.elem(&3u8);

    let vec = vec![p1.clone(), p2.clone(), p3.clone()];
    let ps = EcPoints((&ops, vec));

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
    let curve = WeierstrassEq::secp256k1();
    let ops = JacobianAddOps::new();

    let p1 = EcPoint1((&ops, curve.g()));
    let p2 = &p1 * curve.f.elem(&2u8);
    let p3 = &p1 * curve.f.elem(&3u8);

    let vec = vec![p1.clone(), p2.clone(), p3.clone()];
    let ps = EcPoints((&ops, vec));

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
    let curve = WeierstrassEq::secp256k1();
    let ops = JacobianAddOps::new();

    let p1 = EcPoint1((&ops, curve.g()));
    let p2 = &p1 * curve.f.elem(&2u8);
    let p3 = &p1 * curve.f.elem(&3u8);

    let vec = vec![p1.clone(), p2.clone(), p3.clone()];
    let ps = EcPoints((&ops, vec));
    let act = ps.sum();
    let exp = &p1 * curve.f.elem(&6u8);

    assert!(act == exp);
  }

  #[test]
  fn ec_points_times_field_elem() {
    let curve = WeierstrassEq::secp256k1();
    let ops = JacobianAddOps::new();

    let p1 = EcPoint1((&ops, curve.g()));
    let p2 = &p1 * curve.f.elem(&2u8);
    let p4 = &p1 * curve.f.elem(&4u8);

    let act = EcPoints((&ops, vec![p1, p2.clone()])) * curve.f.elem(&2u8);
    let exp = EcPoints((&ops, vec![p2, p4]));

    assert!(act == exp);
  }

  #[test]
  fn ec_points_plus_ec_points() {
    let curve = WeierstrassEq::secp256k1();
    let ops = JacobianAddOps::new();

    let p1 = EcPoint1((&ops, curve.g()));
    let p2 = &p1 * curve.f.elem(&2u8);

    let ps = EcPoints((&ops, vec![p1, p2.clone()]));
    let ps3 = &ps * curve.f.elem(&3u8);
    let ps2 = &ps * curve.f.elem(&2u8);

    let act = ps + ps2;
    let exp = ps3;

    assert!(act == exp);
  }

  #[test]
  fn ec_points_minus_ec_points() {
    let curve = &WeierstrassEq::secp256k1();
    let ops = &JacobianAddOps::new();

    let g = &EcPoint1((ops, curve.g()));
    let g2 = g * curve.f.elem(&2u8);
    let zero = &EcPoint1((ops, ops.get_zero_point()));

    let g2s = EcPoints((ops, vec![g2.clone(), g2.clone()]));
    let zeros = EcPoints((ops, vec![zero.clone(), zero.clone()]));

    let act = &g2s - &g2s;
    let exp = zeros;

    assert!(act == exp);
  }
}