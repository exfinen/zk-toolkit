use std::{
  ops,
  ops::{Index, RangeFrom, RangeTo, Deref},
};
use crate::building_block::field::prime_field_elem::PrimeFieldElem;

pub struct PrimeFieldElems(pub Vec<PrimeFieldElem>);

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

  pub fn from(&self, range: RangeFrom<usize>) -> PrimeFieldElems {
    let xs = &self.0[range.start..self.0.len()];
    PrimeFieldElems(xs.to_vec())
  }

  pub fn to(&self, range: RangeTo<usize>) -> PrimeFieldElems {
    let xs = &self.0[0..range.end];
    PrimeFieldElems(xs.to_vec())
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

