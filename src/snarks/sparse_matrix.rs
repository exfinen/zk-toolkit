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

  pub fn pretty_print(&self) -> String {
    let mut s = String::new();
    let mut keys: Vec<&FieldElem> = self.rows.keys().collect();
    keys.sort();
    for key in keys {
      s = format!("{}{}\n", s, self.rows.get(key).unwrap().pretty_print());
    }
    s
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

  pub fn get_row(&self, y: &impl ToBigUint) -> SparseVec {
    let y = &self.f.elem(y);
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

  pub fn get_column(&self, x: &impl ToBigUint) -> SparseVec {
    let x = &self.f.elem(x);
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

  pub fn transpose(&self) -> SparseMatrix {
    let mut m = SparseMatrix::new(&self.f, &self.height, &self.width);
    for y in self.rows.keys() {
      let src_row = self.rows.get(&y).unwrap();

      for x in src_row.indices() {
        let v = src_row.get(&x);
        if !v.n.is_zero() {
          m.set(y, &x, v);
        }
      }
    }
    m
  }
}

impl PartialEq for SparseMatrix {
  fn eq(&self, other: &SparseMatrix) -> bool {
    if self.width != other.width || self.height != other.height {
      return false;
    }
    for key in self.rows.keys() {
      if !other.rows.contains_key(key) {
        return false;
      }
      let self_row = &self.rows[key];
      let other_row = &other.rows[key];

      if self_row != other_row {
        return false;
      }
    }
    true
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
        panic!("Can only multiply matrix with height {:?}, but the rhs height is {:?}",
          self.width.n, rhs.height.n);
      }
      let mut res = SparseMatrix::new(&self.f, &rhs.width, &self.height);

      let mut y = self.f.elem(&0u8);
      while y < self.height {
        let mut x = self.f.elem(&0u8);
        while x < rhs.width {
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

  fn gen_test_2x2_identity_matrix() -> SparseMatrix {
    let f = &Field::new(&3911u16);
    let zero = &f.elem(&0u8);
    let one = &f.elem(&1u8);

    let mut v1 = SparseVec::new(f, &2u8);
    let mut v2 = SparseVec::new(f, &2u8);
    v1.set(zero, one);
    v2.set(one, one);
    let vecs = vec![v1, v2];
    SparseMatrix::from(&vecs)
  }

  fn gen_test_2x2_matrix() -> SparseMatrix {
    let f = &Field::new(&3911u16);
    let zero = &f.elem(&0u8);
    let one = &f.elem(&1u8);
    let two = &f.elem(&2u8);
    let three = &f.elem(&3u8);
    let four = &f.elem(&4u8);

    let mut v1 = SparseVec::new(f, &2u8);
    let mut v2 = SparseVec::new(f, &2u8);
    v1.set(zero, one);
    v1.set(one, two);
    v2.set(zero, three);
    v2.set(one, four);
    let vecs = vec![v1, v2];
    SparseMatrix::from(&vecs)
  }

  fn gen_test_2x3_matrix() -> SparseMatrix {
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
    SparseMatrix::from(&vecs)
  }

  #[test]
  fn test_eq() {
    let m1 = gen_test_2x2_identity_matrix();
    let m2 = gen_test_2x2_matrix();
    let m3 = gen_test_3x2_matrix();

    assert!(&m1 == &m1);
    assert!(&m2 == &m2);
    assert!(&m3 == &m3);
  }

  #[test]
  fn test_non_eq() {
    let m1 = gen_test_2x2_identity_matrix();
    let m2 = gen_test_2x2_matrix();
    let m3 = gen_test_3x2_matrix();

    assert!(&m1 != &m2);
    assert!(&m1 != &m3);
    assert!(&m2 != &m1);
    assert!(&m2 != &m3);
    assert!(&m3 != &m1);
    assert!(&m3 != &m2);
  }

  fn gen_test_3x2_matrix() -> SparseMatrix {
    let f = &Field::new(&3911u16);
    let zero = &f.elem(&0u8);
    let one = &f.elem(&1u8);
    let two = &f.elem(&2u8);
    let three = &f.elem(&3u8);

    let mut v1 = SparseVec::new(f, &3u8);
    let mut v2 = SparseVec::new(f, &3u8);
    v1.set(zero, one);
    v1.set(one, two);
    v1.set(two, three);
    v2.set(zero, three);
    v2.set(one, two);
    v2.set(two, one);
    let vecs = vec![v1, v2];
    SparseMatrix::from(&vecs)
  }

  #[test]
  fn test_from_sparse_vecs() {
    let m = gen_test_2x3_matrix();
    let zero = &m.f.elem(&0u8);
    let one = &m.f.elem(&1u8);
    let two = &m.f.elem(&2u8);
    let three = &m.f.elem(&3u8);

    assert_eq!(&m.width, two);
    assert_eq!(&m.height, three);

    assert_eq!(m.get(zero, zero), one);
    assert_eq!(m.get(one, one), two);
    assert_eq!(m.get(zero, two), three);
  }

  #[test]
  #[should_panic]
  fn test_get_row_x_out_of_range() {
    std::panic::set_hook(Box::new(|_| {}));  // suppress stack trace
    let m = gen_test_2x3_matrix();
    let _ = m.get_row(&m.height);
  }

  #[test]
  fn test_get_row_x_within_range() {
    let m = gen_test_2x3_matrix();
    let zero = &m.f.elem(&0u8);
    let one = &m.f.elem(&1u8);
    let two = &m.f.elem(&2u8);
    let three = &m.f.elem(&3u8);

    let r0 = m.get_row(&0u8);
    assert_eq!(r0.get(&0u8), one);
    assert_eq!(r0.get(&1u8), zero);

    let r1 = m.get_row(&1u8);
    assert_eq!(r1.get(&0u8), zero);
    assert_eq!(r1.get(&1u8), two);

    let r2 = m.get_row(&2u8);
    assert_eq!(r2.get(&0u8), three);
    assert_eq!(r2.get(&1u8), zero);
  }

  #[test]
  #[should_panic]
  fn test_get_column_out_of_range() {
    std::panic::set_hook(Box::new(|_| {}));  // suppress stack trace
    let m = gen_test_2x3_matrix();
    let _ = m.get_column(&m.width);
  }

  #[test]
  fn test_get_column_within_range() {
    let m = gen_test_2x3_matrix();
    let zero = &m.f.elem(&0u8);
    let one = &m.f.elem(&1u8);
    let two = &m.f.elem(&2u8);
    let three = &m.f.elem(&3u8);

    let c0 = m.get_column(&0u8);
    assert_eq!(c0.get(&0u8), one);
    assert_eq!(c0.get(&1u8), zero);
    assert_eq!(c0.get(&2u8), three);

    let c1 = m.get_column(&1u8);
    assert_eq!(c1.get(&0u8), zero);
    assert_eq!(c1.get(&1u8), two);
    assert_eq!(c1.get(&2u8), zero);
  }

  #[test]
  #[should_panic]
  fn test_mul_incompatible_matrices() {
    std::panic::set_hook(Box::new(|_| {}));  // suppress stack trace
    let m = gen_test_2x3_matrix();
    let _ = &m * &m;
  }

  #[test]
  fn test_mul_different_sizes() {
    let m1 = gen_test_3x2_matrix();
    let m2 = gen_test_2x3_matrix();
    let m3 = &m1 * &m2;

    let two = &m3.f.elem(&2u8);
    let six = &m3.f.elem(&6u8);
    let four = &m3.f.elem(&4u8);
    let ten = &m3.f.elem(&10u8);

    assert_eq!(&m3.width, two);
    assert_eq!(&m3.height, two);
    assert_eq!(m3.get(&0u8, &0u8), ten);
    assert_eq!(m3.get(&1u8, &0u8), four);
    assert_eq!(m3.get(&0u8, &1u8), six);
    assert_eq!(m3.get(&1u8, &1u8), four);
  }

  #[test]
  fn test_mul_identity() {
    let m1 = gen_test_2x2_matrix();
    let m2 = gen_test_2x2_identity_matrix();
    let m3 = &m1 * &m2;

    let two = &m3.f.elem(&2u8);
    assert_eq!(&m3.width, two);
    assert_eq!(&m3.height, two);

    assert!(&m1 == &m3);
  }

  #[test]
  fn test_transpose() {
    let m = &gen_test_3x2_matrix();
    let mt = &m.transpose();

    assert_eq!(m.height, mt.width);
    assert_eq!(m.width, mt.height);

    let mut x = m.f.elem(&0u8);
    while x < m.width {
      let mut y = m.f.elem(&0u8);
      while y < m.height {
        let m_v = m.get(&x, &y);
        let mt_v = mt.get(&y, &x);
        assert_eq!(m_v, mt_v);
        y.inc();
      }
      x.inc();
    }
  }
}
