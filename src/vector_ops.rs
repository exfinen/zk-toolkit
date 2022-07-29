use crate::elliptic_curve::AddOps;
use crate::field::FieldElem;
use crate::ec_point::EcPoint;
use std::ops;
use std::ops::{Range, RangeFrom, RangeTo, Deref};

pub trait GenericRange {
  fn start(&self) -> usize;
  fn end(&self) -> Option<usize>;
}

impl GenericRange for Range<usize> {
  fn start(&self) -> usize { self.start }
  fn end(&self) -> Option<usize> { Some(self.end) }
}

impl GenericRange for RangeFrom<usize> {
  fn start(&self) -> usize { self.start }
  fn end(&self) -> Option<usize> { None }
}

impl GenericRange for RangeTo<usize> {
  fn start(&self) -> usize { 0 }
  fn end(&self) -> Option<usize> { Some(self.end) }
}

pub struct FieldElems(pub Vec<FieldElem>);

impl<'a> FieldElems {
  pub fn slice(&self, range: &'a dyn GenericRange) -> FieldElems {
    let end = match range.end() {
      Some(x) => x,
      None => self.0.len(),
    };
    let range = Range { start: range.start(), end };

    let mut nxs: Vec<FieldElem> = vec![];
    for i in range {
      nxs.push(self.0[i].clone());
    }
    FieldElems(nxs)
  }

  pub fn sum(&self) -> FieldElem {
    assert!(self.0.len() > 0);
    let xs = self.0;
    xs.iter().fold(xs[0].f.elem(&0u8), |acc, x| {
      acc + x
    })
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

// returns inner product
impl<'a> ops::Mul<&'a[FieldElem]> for FieldElems {
  type Output = FieldElem;

  fn mul(self, rhs: &'a[FieldElem]) -> Self::Output {
    assert!(self.len() == rhs.len() && self.len() > 0);

    let zero = self[0].f.elem(&0u8);
    self.iter().zip(rhs.iter())
      .fold(zero, |acc, (a_i, b_i)| acc + &(a_i * b_i))
  }
}

// returns Hadamard product
impl<'a> ops::Mul<&FieldElems> for FieldElems {
  type Output = FieldElems;

  fn mul(self, rhs: &FieldElems) -> Self::Output {
    assert!(self.len() == rhs.len() && self.len() > 0);

    let xs = self.iter().zip(rhs.iter()).map(|(l, r)| {
      l * r
    }).collect();
    FieldElems(xs)
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

impl<'a> ops::Sub<&'a [FieldElem]> for FieldElems {
  type Output = FieldElems;

  fn sub(self, rhs: &'a[FieldElem]) -> Self::Output {
    assert!(self.len() > 0);

    let xs: Vec<FieldElem> = self.iter().zip(rhs.iter()).map(|(l, r)| {
      l - r 
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

impl<'a> ops::Add<&EcPoint1<'a>> for EcPoint1<'a> {
  type Output = EcPoint;

  fn add(self, rhs: &EcPoint1<'a>) -> Self::Output {
    let (ops, lhs) = self.0;
    ops.add(&lhs, &rhs.0.1)
  }
}

pub struct EcPoints<'a>(pub (&'a dyn AddOps, Vec<EcPoint>));

impl<'a> EcPoints<'a> {
  pub fn slice(&self, range: &'a dyn GenericRange) -> EcPoints<'a> {
    let end = match range.end() {
      Some(x) => x,
      None => self.0.1.len(),
    };
    let range = Range { start: range.start(), end };

    let (ops, xs) = &self.0;

    let mut nxs: Vec<EcPoint> = vec![];
    for i in range {
      nxs.push(xs[i].clone());
    }
    EcPoints((*ops, nxs))
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

impl<'a> ops::Sub<&'a[EcPoint]> for EcPoints<'a> {
  type Output = EcPoints<'a>;

  fn sub(self, rhs: &'a[EcPoint]) -> Self::Output {
    let (ops, lhs) = self.0;
    assert!(lhs.len() == rhs.len() && lhs.len() > 0);

    let xs = lhs.iter().zip(rhs.iter()).map(|(g_i, h_i)| {
      ops.add(g_i, &ops.inv(&h_i))
    }).collect();

    EcPoints((ops, xs))
  }
}



