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

// returns scalar multiple of self
impl<'a> ops::Mul<&FieldElem> for &EcPoint1<'a> {
  type Output = EcPoint1<'a>;

  fn mul(self, rhs: &FieldElem) -> Self::Output {
    let (ops, lhs) = &self.0;
    let x = ops.scalar_mul(&lhs, &rhs.n);
    EcPoint1((*ops, x))
  }
}

// point addition
impl<'a> ops::Add<&EcPoint1<'a>> for &EcPoint1<'a> {
  type Output = EcPoint1<'a>;

  fn add(self, rhs: &EcPoint1<'a>) -> Self::Output {
    let (ops, lhs) = &self.0;
    let x = ops.add(&lhs, &rhs.0.1);
    EcPoint1((*ops, x))
  }
}

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
impl<'a> ops::Mul<&FieldElems> for &EcPoints<'a> {
  type Output = EcPoints<'a>;

  fn mul(self, rhs: &FieldElems) -> Self::Output {
    assert!(self.len() > 0 && self.len() == rhs.len());
    let (ops, lhs) = &self.0;

    let xs: Vec<EcPoint1<'a>> = lhs.iter().zip(rhs.iter()).map(|(pt, fe)| {
      let x = ops.scalar_mul(pt, &fe.n);
      EcPoint1((*ops, x))
    }).collect();

    EcPoints((*ops, xs))
  }
}

// returns Hadamard product (TODO almost the same as ops::Mul<&FieldElems>)
impl<'a> ops::Mul<&[FieldElem]> for &EcPoints<'a> {
  type Output = EcPoints<'a>;

  fn mul(self, rhs: &[FieldElem]) -> Self::Output {
    assert!(self.len() > 0 && self.len() == rhs.len());
    let (ops, lhs) = &self.0;

    let xs: Vec<EcPoint1<'a>> = lhs.iter().zip(rhs.iter()).map(|(pt, fe)| {
      let x = ops.scalar_mul(pt, &fe.n);
      EcPoint1((*ops, x))
    }).collect();

    EcPoints((*ops, xs))
  }
}

// returns Hadamard product
impl<'a> ops::Mul<&EcPoints<'a>> for &EcPoints<'a> {
  type Output = EcPoints<'a>;

  fn mul(self, rhs: &EcPoints) -> Self::Output {
    assert!(self.len() > 0 && self.len() == rhs.len());
    let (ops, lhs) = &self.0;
    let (_, rhs) = &rhs.0;

    let xs = lhs.iter().zip(rhs.iter()).map(|(l, r)| {
      let x = ops.add(l, r);
      EcPoint1((*ops, x))
    }).collect();

    EcPoints((*ops, xs))
  }
}

// multiply rhs (scalar) to each element 
impl<'a> ops::Mul<&FieldElem> for &EcPoints<'a> {
  type Output = EcPoints<'a>;

  fn mul(self, rhs: &FieldElem) -> Self::Output {
    assert!(self.len() > 0);
    let (ops, lhs) = &self.0;

    let xs = lhs.iter().map(|pt| {
      let x = ops.scalar_mul(pt, &rhs.n);
      EcPoint1((*ops, x))
    }).collect();

    EcPoints((*ops, xs))
  }
}

impl<'a> ops::Sub<&EcPoints<'a>> for &EcPoints<'a> {
  type Output = EcPoints<'a>;

  fn sub(self, rhs: &EcPoints<'a>) -> Self::Output {
    assert!(self.len() > 0 && self.len() == self.len());
    let (ops, lhs) = &self.0;

    let xs = lhs.iter().zip(rhs.iter()).map(|(g_i, h_i)| {
      let x = ops.add(g_i, &ops.inv(&h_i));
      EcPoint1((*ops, x))
    }).collect();

    EcPoints((*ops, xs))
  }
}
