use crate::elliptic_curve::AddOps;
use crate::field::FieldElem;
use crate::ec_point::EcPoint;
use std::ops;
use std::ops::Deref;

pub struct FieldElems(pub Vec<FieldElem>);

impl Deref for FieldElems {
  type Target = Vec<FieldElem>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl<'a> ops::Add<&'a[FieldElem]> for FieldElems {
  type Output = FieldElems;

  fn add(self, rhs: &'a[FieldElem]) -> FieldElems {
    assert!(self.len() > 0);
    let xs: Vec<FieldElem> = self.iter().zip(rhs.iter()).map(|(a, b)| {
      a + b 
    }).collect();
    FieldElems(xs)
  }
}

impl<'a> ops::Mul<&'a[FieldElem]> for FieldElems {
  type Output = FieldElem;

  fn mul(self, rhs: &'a[FieldElem]) -> FieldElem {
    assert!(self.len() == rhs.len() && self.len() > 0);

    let zero = self[0].f.elem(&0u8);
    self.iter().zip(rhs.iter())
      .fold(zero, |acc, (a_i, b_i)| acc + &(a_i * b_i))
  }
}

impl<'a> ops::Mul<&'a FieldElem> for FieldElems {
  type Output = FieldElems;

  fn mul(self, rhs: &'a FieldElem) -> FieldElems {
    assert!(self.len() > 0);

    let xs: Vec<FieldElem> = self.iter().map(|fe| {
      fe * rhs  
    }).collect();

    FieldElems(xs)
  }
}

pub struct EcPoint1<'a>(pub (&'a dyn AddOps, &'a EcPoint));

impl<'a> ops::Mul<&'a FieldElem> for EcPoint1<'a> {
  type Output = EcPoint;

  fn mul(self, rhs: &'a FieldElem) -> EcPoint {
    let (ops, lhs) = self.0;
    ops.scalar_mul(lhs, &rhs.n)
  }
}

pub struct EcPoints<'a>(pub (&'a dyn AddOps, Vec<EcPoint>));

impl<'a> ops::Mul<&'a[FieldElem]> for EcPoints<'a> {
  type Output = EcPoint;

  fn mul(self, rhs: &'a[FieldElem]) -> EcPoint {
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

  fn mul(self, rhs: FieldElem) -> EcPoints<'a> {
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

  fn mul(self, rhs: EcPoints<'a>) -> EcPoints<'a> {
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

  fn mul(self, rhs: &'a[EcPoint]) -> EcPoints<'a> {
    let (ops, lhs) = self.0;
    assert!(lhs.len() == rhs.len() && lhs.len() > 0);

    let xs = lhs.iter().zip(rhs.iter()).map(|(g_i, h_i)| {
      ops.add(g_i, h_i)
    }).collect();

    EcPoints((ops, xs))
  }
}







