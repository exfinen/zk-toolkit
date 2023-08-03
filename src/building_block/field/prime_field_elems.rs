use std::{
  fmt,
  ops,
  ops::{Index, Deref},
};
use crate::building_block::field::prime_field_elem::PrimeFieldElem;

#[derive(Clone)]
pub struct PrimeFieldElems(pub Vec<PrimeFieldElem>);

impl fmt::Debug for PrimeFieldElems {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      write!(f, "{{")?;
      for x in &self.0 {
        write!(f, "{:?},", x)?;
      }
      write!(f, "}}")
  }
}

impl<'a> Index<usize> for PrimeFieldElems {
  type Output = PrimeFieldElem;

  fn index(&self, index: usize) -> &Self::Output {
    let x = &self.0[index];
    x
  }
}

impl<'a> PrimeFieldElems {
  pub fn new(xs: &'a [PrimeFieldElem]) -> Self {
    PrimeFieldElems(xs.to_vec())
  }

  pub fn sum(&self) -> PrimeFieldElem {
    assert!(self.0.len() > 0);
    let xs = &self.0;
    xs.iter().fold(xs[0].f.elem(&0u8), |acc, x| {
      acc + x
    })
  }

  pub fn from(&self, idx: usize) -> PrimeFieldElems {
    if idx >= self.len() {
      panic!("index outside the range is specified");
    } else {
      let mut xs = vec![];
      for i in idx..self.len() {
        xs.push(self[i].clone());
      }
      PrimeFieldElems(xs.to_vec())
    }
  }

  pub fn to(&self, idx: usize) -> PrimeFieldElems {
    if idx > self.len() {
      panic!("index outside the range is specified");
    } else {
      let mut xs = vec![];
      for i in 0..idx {
        xs.push(self[i].clone());
      }
      PrimeFieldElems(xs.to_vec())
    }
  }
}

impl PartialEq for PrimeFieldElems {
  fn eq(&self, other: &Self) -> bool {
    if self.len() != other.len() {
      false
    } else {
      self.iter().zip(other.iter()).fold(true, |acc, (l, r)| {
        acc && l.f == r.f && l.e == r.e
      })
    }
  }
}

impl Eq for PrimeFieldElems {}

impl Deref for PrimeFieldElems {
  type Target = [PrimeFieldElem];

  fn deref(&self) -> &Self::Target {
    &self.0[..]
  }
}

macro_rules! impl_field_elems_plus_field_elems {
  ($rhs: ty, $target: ty) => {
    impl<'a> ops::Add<$rhs> for $target {
      type Output = PrimeFieldElems;

      fn add(self, rhs: $rhs) -> Self::Output {
        assert!(self.len() > 0 && self.len() == rhs.len());

        let mut xs = vec![];
        for i in 0..self.len() {
          xs.push(&self[i] + &rhs[i]);
        }
        PrimeFieldElems(xs)
      }
    }
  };
}
impl_field_elems_plus_field_elems!(PrimeFieldElems, PrimeFieldElems);
impl_field_elems_plus_field_elems!(PrimeFieldElems, &PrimeFieldElems);
impl_field_elems_plus_field_elems!(&PrimeFieldElems, &PrimeFieldElems);

macro_rules! impl_field_elems_minus_field_elems {
  ($rhs: ty, $target: ty) => {
    impl<'a> ops::Sub<$rhs> for $target {
      type Output = PrimeFieldElems;

      fn sub(self, rhs: $rhs) -> Self::Output {
        assert!(self.len() > 0 && self.len() == rhs.len());

        let mut xs = vec![];
        for i in 0..self.len() {
          xs.push(&self[i] - &rhs[i]);
        }
        PrimeFieldElems(xs)
      }
    }
  };
}
impl_field_elems_minus_field_elems!(PrimeFieldElems, &PrimeFieldElems);
impl_field_elems_minus_field_elems!(&PrimeFieldElems, &PrimeFieldElems);

macro_rules! impl_field_elems_times_field_elems {
  ($rhs: ty, $target: ty) => {
    impl<'a> ops::Mul<$rhs> for $target {
      type Output = PrimeFieldElems;

      fn mul(self, rhs: $rhs) -> Self::Output {
        assert!(self.len() > 0 && self.len() == rhs.len());

        let mut xs = vec![];
        for i in 0..self.len() {
          xs.push(&self[i] * &rhs[i]);
        }
        PrimeFieldElems(xs)
      }
    }
  };
}
impl_field_elems_times_field_elems!(PrimeFieldElems, PrimeFieldElems);
impl_field_elems_times_field_elems!(PrimeFieldElems, &PrimeFieldElems);
impl_field_elems_times_field_elems!(&PrimeFieldElems, &PrimeFieldElems);

// multiply rhs (scalar) to each element
macro_rules! impl_field_elems_times_field_elem {
  ($rhs: ty, $target: ty) => {
    impl<'a> ops::Mul<$rhs> for $target {
      type Output = PrimeFieldElems;

      fn mul(self, rhs: $rhs) -> Self::Output {
        assert!(self.len() > 0);
        let rhs = rhs.clone();  // TODO find a better way to solve &&rhs issue

        let mut xs = vec![];
        for x in self.iter() {
          xs.push(x * &rhs);
        }
        PrimeFieldElems(xs)
      }
    }
  };
}
impl_field_elems_times_field_elem!(&PrimeFieldElem, &PrimeFieldElems);
impl_field_elems_times_field_elem!(PrimeFieldElem, PrimeFieldElems);
impl_field_elems_times_field_elem!(&PrimeFieldElem, PrimeFieldElems);
impl_field_elems_times_field_elem!(PrimeFieldElem, &PrimeFieldElems);

#[cfg(test)]
mod tests {
  use super::*;
  use std::sync::Arc;
  use crate::building_block::curves::secp256k1::affine_point::AffinePoint;

  #[test]
  fn test_from() {
    let f = &Arc::new(AffinePoint::base_field());
    let one = &PrimeFieldElem::new(f, &1u8);
    let vec = vec![
      one.clone(),
      one * 2u8,
      one * 3u8,
      one * 4u8,
    ];
    let elems = PrimeFieldElems::new(&vec);
    {
      let res = &elems.from(0);
      assert_eq!(res.len(), 4);
      assert_eq!(res.0.as_slice(), vec.as_slice());
    }
    {
      let res = &elems.from(1);
      assert_eq!(res.len(), 3);
      assert_eq!(&res[0], &vec[1]);
      assert_eq!(&res[1], &vec[2]);
      assert_eq!(&res[2], &vec[3]);
    }
    {
      let res = &elems.from(2);
      assert_eq!(res.len(), 2);
      assert_eq!(&res[0], &vec[2]);
      assert_eq!(&res[1], &vec[3]);
    }
    {
      let res = &elems.from(3);
      assert_eq!(res.len(), 1);
      assert_eq!(&res[0], &vec[3]);
    }
    // TODO test elem.from(4) and confirm it panics
  }

  #[test]
  fn test_to() {
    let f = &Arc::new(AffinePoint::base_field());
    let one = &PrimeFieldElem::new(f, &1u8);
    let vec = vec![
      one.clone(),
      one * 2u8,
      one * 3u8,
      one * 4u8,
    ];
    let elems = PrimeFieldElems::new(&vec);
    {
      let res = &elems.to(0);
      assert_eq!(res.len(), 0);
    }
    {
      let res = &elems.to(1);
      assert_eq!(res.len(), 1);
      assert_eq!(&res[0], &vec[0]);
    }
    {
      let res = &elems.to(2);
      assert_eq!(res.len(), 2);
      assert_eq!(&res[0], &vec[0]);
      assert_eq!(&res[1], &vec[1]);
    }
    {
      let res = &elems.to(3);
      assert_eq!(res.len(), 3);
      assert_eq!(&res[0], &vec[0]);
      assert_eq!(&res[1], &vec[1]);
      assert_eq!(&res[2], &vec[2]);
    }
    {
      let res = &elems.to(4);
      assert_eq!(res.len(), 4);
      assert_eq!(&res[0], &vec[0]);
      assert_eq!(&res[1], &vec[1]);
      assert_eq!(&res[2], &vec[2]);
      assert_eq!(&res[3], &vec[3]);
    }
    // TODO test elem.to(5) and confirm it panics
  }

  #[test]
  fn test_sum() {
    let f = &Arc::new(AffinePoint::base_field());
    let one = &PrimeFieldElem::new(f, &1u8);
    let vec = vec![
      one.clone(),
      one * 2u8,
      one * 3u8,
      one * 4u8,
    ];
    let elems = PrimeFieldElems::new(&vec);
    let act = &elems.sum();
    let exp = one * 10u8;
    assert_eq!(act, &exp);
  }
}