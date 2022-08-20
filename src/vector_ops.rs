use crate::elliptic_curve::AddOps;
use crate::field::{FieldElem, FieldElems};
use crate::ec_point::EcPoint;
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
impl_ec_point1_plus_ec_point1!(EcPoint1<'a>, EcPoint1<'a>);


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
impl_ec_points_times_field_elem!(FieldElem, &EcPoints<'a>);

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