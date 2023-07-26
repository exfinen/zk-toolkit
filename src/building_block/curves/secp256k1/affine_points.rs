use crate::building_block::{
  field::{
    prime_field_elem::PrimeFieldElem,
    prime_field_elems::PrimeFieldElems,
  },
  curves::secp256k1::{
    affine_point::AffinePoint,
    secp256k1::Secp256k1,
  },
  zero::Zero,
};
use std::{
  ops::{Add, Mul, Deref},
  rc::Rc,
};

pub struct AffinePoints {
  curve: Rc<Secp256k1>,
  points: Vec<AffinePoint>,
}

impl AffinePoints {
  pub fn new(curve: &Rc<Secp256k1>, points: &Vec<AffinePoint>) -> Self {
    AffinePoints {
      curve: curve.clone(),
      points: points.clone(),
    }
  }

  pub fn sum(&self) -> AffinePoint {
    let mut sum = self.curve.g().zero();
    for p in &self.points {
      sum = sum + p;
    }
    sum
  }

  pub fn rand_points(
    curve: &Rc<Secp256k1>,
    exclude_zero: bool,
    length: &usize,
  ) -> Self {
    let g = &curve.g();
    let mut points = vec![];

    while &points.len() < length {
      let p = g.rand_point(exclude_zero);
      points.push(p);
    }
    AffinePoints::new(&curve, &points)
  }

  pub fn from(&self, idx: usize) -> Self {
    if idx >= self.len() {
      AffinePoints::new(&self.curve, &vec![])
    } else {
      let mut points = vec![];
      for i in idx..self.len() {
        points.push(self[i].clone());
      }
      AffinePoints::new(&self.curve, &points)
    }
  }

  pub fn to(&self, idx: usize) -> Self {
    let idx = if idx >= self.points.len()
      { self.points.len() - 1 } else { idx };

    let mut points = vec![];
    for i in idx..self.len() {
      points.push(self[i].clone());
    }
    AffinePoints::new(&self.curve, &points)
  }
}

macro_rules! impl_add {
  ($rhs: ty, $target: ty) => {
    impl<'a> Add<$rhs> for $target {
      type Output = AffinePoints;

      fn add(self, rhs: $rhs) -> Self::Output {
        if self.len() != rhs.len() {
          panic!("Tried to add AffinePoints of diffrent length");
        }
        let mut points = vec![];
        for i in 0..self.len() {
          points.push(self.points[i].clone());
        }
        AffinePoints::new(&self.curve, &points)
      }
    }
  };
}
impl_add!(AffinePoints, &AffinePoints);
impl_add!(&AffinePoints, &AffinePoints);
impl_add!(AffinePoints, AffinePoints);
impl_add!(&AffinePoints, AffinePoints);

macro_rules! impl_scalar_mul {
  ($rhs: ty, $target: ty) => {
    impl<'a> Mul<$rhs> for $target {
      type Output = AffinePoints;

      fn mul(self, rhs: $rhs) -> Self::Output {
        let mut points = vec![];
        for x in &self.points {
          points.push(x * rhs.clone())
        }
        AffinePoints::new(&self.curve, &points)
      }
    }
  };
}
impl_scalar_mul!(PrimeFieldElem, AffinePoints);
impl_scalar_mul!(&PrimeFieldElem, AffinePoints);
impl_scalar_mul!(PrimeFieldElem, &AffinePoints);
impl_scalar_mul!(&PrimeFieldElem, &AffinePoints);

macro_rules! impl_vec_mul {
  ($rhs: ty, $target: ty) => {
    impl<'a> Mul<$rhs> for $target {
      type Output = AffinePoints;

      fn mul(self, rhs: $rhs) -> Self::Output {
        if self.points.len() != rhs.len() {
          panic!("Tried to multiply PrimeFieldElems of different size to AffinePoints");
        }
        let mut points = vec![];
        for i in 0..self.points.len() {
          points.push(&self.points[i] * rhs[i].clone())
        }
        AffinePoints::new(&self.curve, &points)
      }
    }
  };
}
impl_vec_mul!(PrimeFieldElems, AffinePoints);
impl_vec_mul!(&PrimeFieldElems, AffinePoints);
impl_vec_mul!(PrimeFieldElems, &AffinePoints);
impl_vec_mul!(&PrimeFieldElems, &AffinePoints);

impl Deref for AffinePoints {
  type Target = Vec<AffinePoint>;

  fn deref(&self) -> &Self::Target {
    &self.points
  }
}

impl PartialEq for AffinePoints {
  fn eq(&self, rhs: &Self) -> bool {
    if self.points.len() != rhs.points.len() {
      false
    } else {
      for i in 0..self.points.len() {
        if self.points[i] != rhs.points[i] {
          return false;
        }
      }
      true
    }
  }
}

impl Eq for AffinePoints {}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_from() {
  }

  #[test]
  fn test_to() {
  }

  #[test]
  fn test_sum() {
  }
}