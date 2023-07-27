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
      panic!("Index outside the range is specified");
    } else {
      let mut points = vec![];
      for i in idx..self.len() {
        points.push(self[i].clone());
      }
      AffinePoints::new(&self.curve, &points)
    }
  }

  pub fn to(&self, idx: usize) -> Self {
    if idx > self.points.len() {
      panic!("Index outside the range is specified");
    }
    let mut points = vec![];
    for i in 0..idx {
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
  use crate::building_block::curves::secp256k1::{
    affine_points::AffinePoints,
    secp256k1::Secp256k1,
  };
  use std::rc::Rc;

  #[test]
  fn test_from() {
    let curve = Rc::new(Secp256k1::new());
    let g = &curve.g();
    let gs_vec = vec![
      g.clone(),
      g + g,
      g + g + g,
      g + g + g + g,
    ];
    let elems = AffinePoints::new(&curve, &gs_vec);
    {
      let res = &elems.from(0);
      assert_eq!(res.len(), 4);
      assert_eq!(res.as_slice(), gs_vec.as_slice());
    }
    {
      let res = &elems.from(1);
      assert_eq!(res.len(), 3);
      assert_eq!(&res[0], &gs_vec[1]);
      assert_eq!(&res[1], &gs_vec[2]);
      assert_eq!(&res[2], &gs_vec[3]);
    }
    {
      let res = &elems.from(2);
      assert_eq!(res.len(), 2);
      assert_eq!(&res[0], &gs_vec[2]);
      assert_eq!(&res[1], &gs_vec[3]);
    }
    {
      let res = &elems.from(3);
      assert_eq!(res.len(), 1);
      assert_eq!(&res[0], &gs_vec[3]);
    }
    // TODO test elem.from(4) and confirm it panics
  }

  #[test]
  fn test_to() {
    let curve = Rc::new(Secp256k1::new());
    let g = &curve.g();
    let gs_vec = vec![
      g.clone(),
      g + g,
      g + g + g,
      g + g + g + g,
    ];
    let elems = AffinePoints::new(&curve, &gs_vec);
    {
      let res = &elems.to(0);
      assert_eq!(res.len(), 0);
    }
    {
      let res = &elems.to(1);
      assert_eq!(res.len(), 1);
      assert_eq!(&res[0], &gs_vec[0]);
    }
    {
      let res = &elems.to(2);
      assert_eq!(res.len(), 2);
      assert_eq!(&res[0], &gs_vec[0]);
      assert_eq!(&res[1], &gs_vec[1]);
    }
    {
      let res = &elems.to(3);
      assert_eq!(res.len(), 3);
      assert_eq!(&res[0], &gs_vec[0]);
      assert_eq!(&res[1], &gs_vec[1]);
      assert_eq!(&res[2], &gs_vec[2]);
    }
    {
      let res = &elems.to(4);
      assert_eq!(res.len(), 4);
      assert_eq!(&res[0], &gs_vec[0]);
      assert_eq!(&res[1], &gs_vec[1]);
      assert_eq!(&res[2], &gs_vec[2]);
      assert_eq!(&res[3], &gs_vec[3]);
    }
    // TODO test elem.to(5) and confirm it panics
  }

  #[test]
  fn test_sum() {
    let curve = Rc::new(Secp256k1::new());
    let g = &curve.g();
    let gs_vec = vec![
      g.clone(),
      g + g,
    ];
    let elems = AffinePoints::new(&curve, &gs_vec);
    let act = &elems.sum();
    let exp = g + g + g;
    assert_eq!(act, &exp);
  }
}