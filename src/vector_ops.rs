use crate::elliptic_curve::AddOps;
use crate::field::FieldElem;
use crate::ec_point::EcPoint;
use std::ops;
use std::ops::{Range, RangeFrom, RangeTo, Deref};

struct Helper();

impl<'a> Helper {
  pub fn ec_points_slice(ec_points: &EcPoints<'a>, range: Range<usize>) -> EcPoints<'a> {
    let (ops, xs) = &ec_points.0;

    let mut nxs: Vec<EcPoint> = vec![];
    for i in range {
      nxs.push(xs[i].clone());
    }
    EcPoints((*ops, nxs))
  }

  pub fn field_elems_slice(field_elems: &FieldElems, range: Range<usize>) -> FieldElems {
    let mut nxs: Vec<FieldElem> = vec![];
    for i in range {
      nxs.push(field_elems[i].clone());
    }
    FieldElems(nxs)
  }
}

pub struct FieldElems(pub Vec<FieldElem>);

impl<'a> FieldElems {
  pub fn from(&self, range_from: RangeFrom<usize>) -> FieldElems {
    let range = Range { start: range_from.start, end: self.0.len() };
    Helper::field_elems_slice(self, range)
  }

  pub fn to(&self, range_to: RangeTo<usize>) -> FieldElems {
    let range = Range { start: 0, end: range_to.end };
    Helper::field_elems_slice(self, range)
  }
}

impl Deref for FieldElems {
  type Target = Vec<FieldElem>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl<'a> ops::Add<&'a[FieldElem]> for FieldElems {
  type Output = FieldElems;

  fn add(self, rhs: &'a[FieldElem]) -> Self::Output {
    assert!(self.len() > 0);
    let xs: Vec<FieldElem> = self.iter().zip(rhs.iter()).map(|(a, b)| {
      a + b 
    }).collect();
    FieldElems(xs)
  }
}

impl<'a> ops::Mul<&'a[FieldElem]> for FieldElems {
  type Output = FieldElem;

  fn mul(self, rhs: &'a[FieldElem]) -> Self::Output {
    assert!(self.len() == rhs.len() && self.len() > 0);

    let zero = self[0].f.elem(&0u8);
    self.iter().zip(rhs.iter())
      .fold(zero, |acc, (a_i, b_i)| acc + &(a_i * b_i))
  }
}

impl<'a> ops::Mul<&'a FieldElem> for FieldElems {
  type Output = FieldElems;

  fn mul(self, rhs: &'a FieldElem) -> Self::Output {
    assert!(self.len() > 0);

    let xs: Vec<FieldElem> = self.iter().map(|fe| {
      fe * rhs  
    }).collect();

    FieldElems(xs)
  }
}

pub struct EcPoint1<'a>(pub (&'a dyn AddOps, EcPoint));

impl<'a> ops::Mul<&'a FieldElem> for EcPoint1<'a> {
  type Output = EcPoint;

  fn mul(self, rhs: &'a FieldElem) -> Self::Output {
    let (ops, lhs) = self.0;
    ops.scalar_mul(&lhs, &rhs.n)
  }
}

pub struct EcPoints<'a>(pub (&'a dyn AddOps, Vec<EcPoint>));

impl<'a> EcPoints<'a> {
  pub fn from(&self, range_from: RangeFrom<usize>) -> EcPoints<'a> {
    let range = Range { start: range_from.start, end: self.0.1.len() };
    Helper::ec_points_slice(self, range)
  }

  pub fn to(&self, range_to: RangeTo<usize>) -> EcPoints<'a> {
    let range = Range { start: 0, end: range_to.end };
    Helper::ec_points_slice(self, range)
  }

  pub fn at(&self, i: usize) -> EcPoint1<'a> {
    let (ops, xs) = &self.0;
    EcPoint1((*ops, xs[i].clone()))
  }
}

impl<'a> ops::Mul<&'a[FieldElem]> for EcPoints<'a> {
  type Output = EcPoint;

  fn mul(self, rhs: &'a[FieldElem]) -> Self::Output {
    let (ops, lhs) = self.0;
    assert!(lhs.len() == rhs.len() && lhs.len() > 0);

    lhs.iter().zip(rhs.iter()).fold(None::<EcPoint>, |acc, (pt, fe)| {
      let x = ops.scalar_mul(pt, &fe.n);
      match acc {
        None => Some(x),
        Some(y) => Some(ops.add(&x, &y)),
      }
    }).unwrap()
  }
}

impl<'a> ops::Mul<FieldElem> for EcPoints<'a> {
  type Output = EcPoints<'a>;

  fn mul(self, rhs: FieldElem) -> Self::Output {
    let (ops, lhs) = self.0;
    assert!(lhs.len() > 0);

    let xs = lhs.iter().map(|pt| {
      ops.scalar_mul(pt, &rhs.n)
    }).collect();

    EcPoints((ops, xs))
  }
}

// TODO this is almost identical to Mul<FieldElem> for EcPoints<'a>
impl<'a> ops::Mul<&'a FieldElem> for EcPoints<'a> {
  type Output = EcPoints<'a>;

  fn mul(self, rhs: &'a FieldElem) -> Self::Output {
    let (ops, lhs) = self.0;
    assert!(lhs.len() > 0);

    let xs = lhs.iter().map(|pt| {
      ops.scalar_mul(pt, &rhs.n)
    }).collect();

    EcPoints((ops, xs))
  }
}


// returns Hadamard product
impl<'a> ops::Mul<EcPoints<'a>> for EcPoints<'a> {
  type Output = EcPoints<'a>;

  fn mul(self, rhs: EcPoints<'a>) -> Self::Output {
    let (ops, lhs) = self.0;
    let (_, rhs) = rhs.0;
    assert!(lhs.len() > 0);

    let xs = lhs.iter().zip(rhs.iter()).map(|(l, r)| {
      ops.add(l, r)
    }).collect();

    EcPoints((ops, xs))
  }
}

impl<'a> ops::Mul<&'a[EcPoint]> for EcPoints<'a> {
  type Output = EcPoints<'a>;

  fn mul(self, rhs: &'a[EcPoint]) -> Self::Output {
    let (ops, lhs) = self.0;
    assert!(lhs.len() == rhs.len() && lhs.len() > 0);

    let xs = lhs.iter().zip(rhs.iter()).map(|(g_i, h_i)| {
      ops.add(g_i, h_i)
    }).collect();

    EcPoints((ops, xs))
  }
}


