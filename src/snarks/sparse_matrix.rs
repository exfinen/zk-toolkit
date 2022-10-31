use num_traits::Zero;

use crate::building_block::field::{Field, FieldElem};
use crate::building_block::to_biguint::ToBigUint;
use crate::snarks::sparse_vec::SparseVec;
use std::{
  collections::HashMap,
  convert::From,
  ops::Mul,
};

pub struct SparseMatrix {
  pub width: FieldElem,
  pub height: FieldElem,
  f: Field,
  rows: HashMap<FieldElem, SparseVec>,
  zero: FieldElem,
}

impl SparseMatrix {
  pub fn new(f: &Field, width: &impl ToBigUint, height: &impl ToBigUint) -> Self {
    let zero = f.elem(&0u8);
    let rows = HashMap::new();
    SparseMatrix {
      f: f.clone(),
      width: f.elem(width),
      height: f.elem(height),
      rows,
      zero,
    }
  }

  pub fn set(&mut self, x: &impl ToBigUint, y: &impl ToBigUint, v: &impl ToBigUint) -> () {
    let v = self.f.elem(v);
    let x = self.f.elem(x);
    let y = self.f.elem(y);
    if x >= self.width || y >= self.height {
      panic!("For {:?} x {:?} matrix, ({:?}, {:?}) is out of range",
        self.width.n, self.height.n, x, y);
    }

    if !self.rows.contains_key(&y) {
      let vec = SparseVec::new(&self.f, &self.width);
      self.rows.insert(y.clone(), vec);
    }
    self.rows.get_mut(&y).unwrap().set(&x, &v);
  }

  pub fn get(&self, x: &impl ToBigUint, y: &impl ToBigUint) -> &FieldElem {
    let x = self.f.elem(x);
    let y = self.f.elem(y);
    if x >= self.width || y >= self.height {
      panic!("For {:?} x {:?} matrix, ({:?}, {:?}) is out of range",
        self.width.n, self.height.n, x, y);
    }
    if !self.rows.contains_key(&y) {
      &self.zero
    } else {
      self.rows.get(&y).unwrap().get(&x)
    }
  }

  pub fn get_row(&self, y: &FieldElem) -> SparseVec {
    assert!(y < &self.height);
    let mut row = SparseVec::new(&self.f, &self.width);

    if !self.rows.contains_key(y) {
      return row;
    }
    let src_row = self.rows.get(y).unwrap();
    for x in src_row.indices() {
      let v = src_row.get(&x);
      if !v.n.is_zero() {
        row.set(&x, v);
      }
    }
    row
  }

  pub fn get_column(&self, x: &FieldElem) -> SparseVec {
    assert!(x < &self.width);
    let mut col = SparseVec::new(&self.f, &self.height);

    for y in self.rows.keys() {
      let src_row = self.rows.get(&y).unwrap();
      let v = src_row.get(x);
      if !v.n.is_zero() {
        col.set(y, v);
      }
    }
    col
  }
}

// coverts rows of vectors to a matrix
impl From<&Vec<SparseVec>> for SparseMatrix {
  fn from(rows: &Vec<SparseVec>) -> Self {
    assert!(rows.len() != 0, "Cannot build matrix from empty vector");
    let f = &rows[0].f;
    let width = &rows[0].size;
    let height = &rows.len();
    let mut m = SparseMatrix::new(f, width, height);

    for (y, row) in rows.iter().enumerate() {
      for x in row.indices() {
        let v = row.get(&x);
        m.set(&x, &y, v);
      }
    }
    m
  }
}

impl Mul<&SparseMatrix> for &SparseMatrix {
    type Output = SparseMatrix;

    fn mul(self, rhs: &SparseMatrix) -> Self::Output {
      if self.width != rhs.height {
        panic!("Can only multiply matrix with height {:?}, but the height is {:?}",
          self.width.n, rhs.height.n);
      }
      let mut res = SparseMatrix::new(&self.f, &rhs.height, &self.height);

      let mut y = self.f.elem(&0u8);
      while y < self.height {
        let mut x = self.f.elem(&0u8);
        while x < self.height {
          let lhs = &self.get_row(&y);
          let rhs = &rhs.get_column(&x);
          let v = (lhs * rhs).sum();
          if !v.n.is_zero() {
            res.set(&x, &y, &v);
          }
          x.inc();
        }
        y.inc();
      }

      res
    }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_size() {
    let f = &Field::new(&3911u16);
    let m = SparseMatrix::new(f, &2u8, &3u8);
    assert_eq!(m.width, f.elem(&2u8));
    assert_eq!(m.height, f.elem(&3u8));
  }

  #[test]
  #[should_panic]
  fn test_get_x_out_of_range() {
    std::panic::set_hook(Box::new(|_| {}));  // suppress stack trace
    let f = &Field::new(&3911u16);

    let m = SparseMatrix::new(f, &2u8, &3u8);
    m.get(&2u8, &1u8);
  }

  #[test]
  #[should_panic]
  fn test_get_y_out_of_range() {
    std::panic::set_hook(Box::new(|_| {}));  // suppress stack trace
    let f = &Field::new(&3911u16);

    let m = SparseMatrix::new(f, &2u8, &3u8);
    m.get(&1u8, &3u8);
  }

  #[test]
  #[should_panic]
  fn test_set_x_out_of_range() {
    std::panic::set_hook(Box::new(|_| {}));  // suppress stack trace
    let f = &Field::new(&3911u16);

    let mut m = SparseMatrix::new(f, &2u8, &3u8);
    m.set(&2u8, &1u8, &f.elem(&12u8));
  }

  #[test]
  #[should_panic]
  fn test_set_y_out_of_range() {
    std::panic::set_hook(Box::new(|_| {}));  // suppress stack trace
    let f = &Field::new(&3911u16);

    let mut m = SparseMatrix::new(f, &2u8, &3u8);
    m.set(&1u8, &3u8, &f.elem(&12u8));
  }

  #[test]
  fn test_get_empty() {
    let f = &Field::new(&3911u16);
    let zero = &f.elem(&0u8);

    let m = SparseMatrix::new(f, &2u8, &3u8);
    for x in 0u8..2 {
      for y in 0u8..3 {
        assert_eq!(m.get(&x, &y), zero);
      }
    }
  }

  #[test]
  fn test_mul() {
    let f = &Field::new(&3911u16);
    let zero = &f.elem(&0u8);

    let m = SparseMatrix::new(f, &2u8, &3u8);
    for x in 0u8..2 {
      for y in 0u8..3 {
        assert_eq!(m.get(&x, &y), zero);
      }
    }
  }

  #[test]
  fn test_get_existing_and_non_existing_cells() {
    let f = &Field::new(&3911u16);
    let zero = &f.elem(&0u8);
    let one = &f.elem(&1u8);
    let two = &f.elem(&2u8);
    let three = &f.elem(&3u8);
    let eight = &f.elem(&8u8);
    let nine = &f.elem(&9u8);

    let mut m = SparseMatrix::new(f, two, three);
    m.set(zero, two, nine);
    m.set(one, one, eight);

    assert_eq!(m.get(zero, two), nine);
    assert_eq!(m.get(one, one), eight);
    assert_eq!(m.get(one, two), zero);
  }

  #[test]
  #[should_panic]
  fn test_from_empty_vec() {
    std::panic::set_hook(Box::new(|_| {}));  // suppress stack trace
    let _ = SparseMatrix::from(&vec![]);
  }

  #[test]
  fn test_from_sparse_vecs() {
    let f = &Field::new(&3911u16);
    let zero = &f.elem(&0u8);
    let one = &f.elem(&1u8);
    let two = &f.elem(&2u8);
    let three = &f.elem(&3u8);

    let mut v1 = SparseVec::new(f, &2u8);
    let mut v2 = SparseVec::new(f, &2u8);
    let mut v3 = SparseVec::new(f, &2u8);
    v1.set(zero, one);
    v2.set(one, two);
    v3.set(zero, three);
    let vecs = vec![v1, v2, v3];
    let m = SparseMatrix::from(&vecs);

    assert_eq!(&m.width, two);
    assert_eq!(&m.height, three);

    assert_eq!(m.get(zero, zero), one);
    assert_eq!(m.get(one, one), two);
    assert_eq!(m.get(zero, two), three);
  }

  #[test]
  fn test_get_row_x_out_of_range() {
    let f = &Field::new(&3911u16);
  }

  #[test]
  fn test_get_row_x_within_range() {
    let f = &Field::new(&3911u16);
  }

  #[test]
  fn test_get_column_out_of_range() {
    let f = &Field::new(&3911u16);
  }

  #[test]
  fn test_get_column_within_range() {
    let f = &Field::new(&3911u16);
  }

  #[test]
  fn test_mul_incompatible_matrices() {
    let f = &Field::new(&3911u16);
  }

  #[test]
  fn test_mul_compatible_matrices() {
    let f = &Field::new(&3911u16);
  }
}
