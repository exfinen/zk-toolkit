use num_traits::Zero;

use crate::building_block::field::{
  prime_field::PrimeField,
  prime_field_elem::PrimeFieldElem,
};
use crate::building_block::{
  field::{
    polynomial::Polynomial,
    sparse_vec::SparseVec,
  },
  to_biguint::ToBigUint,
};
use std::{
  collections::HashMap,
  convert::From,
  fmt,
  ops::Mul,
};

pub struct SparseMatrix {
  pub width: PrimeFieldElem,
  pub height: PrimeFieldElem,
  f: PrimeField,
  rows: HashMap<PrimeFieldElem, SparseVec>,
  zero: PrimeFieldElem,
}

impl SparseMatrix {
  pub fn new(f: &PrimeField, width: &impl ToBigUint, height: &impl ToBigUint) -> Self {
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
    let empty_row = &SparseVec::new(&self.f, &self.width);

    let mut y = self.f.elem(&0u8);
    while y < self.height {
      match self.rows.get(&y) {
        Some(row) => {
          s = format!("{}{}\n", s, row.pretty_print());
        },
        None => {
          s = format!("{}{}\n", s, empty_row.pretty_print());
        }
      }
      y.inc();
    }
    s
  }

  pub fn multiply_column(&self, col: &SparseVec) -> Self {
    if col.size != self.height {
      panic!("column size is expected to be {:?}, but got {:?}",
        self.height.e, col.size.e)
    }
    let mut m = SparseMatrix::new(&self.f, &self.width, &self.height);

    let mut y = self.f.elem(&0u8);
    while y < col.size {
      let mut x = self.f.elem(&0u8);
      let multiplier = col.get(&y);
      while x < self.width {
        let v = self.get(&x, &y) * multiplier;
        m.set(&x, &y, &v);
        x.inc();
      }
      y.inc();
    }
    m
  }

  pub fn flatten_rows(&self) -> SparseVec {
    let mut vec = SparseVec::new(&self.f, &self.width);

    let mut y = self.f.elem(&0u8);
    while y < self.height {
      let mut x = self.f.elem(&0u8);
      while x < self.width {
        let v = vec.get(&x) + self.get(&x, &y);
        vec.set(&x, &v);
        x.inc();
      }
      y.inc();
    }
    vec
  }

  pub fn set(&mut self, x: &impl ToBigUint, y: &impl ToBigUint, v: &impl ToBigUint) -> () {
    let v = self.f.elem(v);
    let x = self.f.elem(x);
    let y = self.f.elem(y);
    if x >= self.width || y >= self.height {
      panic!("For {:?} x {:?} matrix, ({:?}, {:?}) is out of range",
        self.width.e, self.height.e, x.e, y.e);
    }
    if v.e.is_zero() {  // don't set if zero
      return;
    }

    if !self.rows.contains_key(&y) {
      let vec = SparseVec::new(&self.f, &self.width);
      self.rows.insert(y.clone(), vec);
    }
    self.rows.get_mut(&y).unwrap().set(&x, &v);
  }

  pub fn get(&self, x: &impl ToBigUint, y: &impl ToBigUint) -> &PrimeFieldElem {
    let x = self.f.elem(x);
    let y = self.f.elem(y);
    if x >= self.width || y >= self.height {
      panic!("For {:?} x {:?} matrix, ({:?}, {:?}) is out of range",
        self.width.e, self.height.e, x.e, y.e);
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
      if !v.e.is_zero() {
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
      if !v.e.is_zero() {
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
        if !v.e.is_zero() {
          m.set(y, &x, v);
        }
      }
    }
    m
  }

  // remove empty rows
  pub fn normalize(&self) -> SparseMatrix {
    let mut m = SparseMatrix::new(&self.f, &self.width, &self.height);
    for row_key in self.rows.keys() {
      let row = self.rows.get(row_key).unwrap();
      if !row.is_empty() {
        m.rows.insert(row_key.clone(), row.clone());
      }
    }
    m
  }

  pub fn row_transform(&self, transform: Box<dyn Fn(&SparseVec) -> SparseVec>) -> SparseMatrix {
    let mut m = SparseMatrix::new(&self.f, &self.width, &self.height);

    let mut y = self.f.elem(&0u8);
    while y < self.height {
      let in_row = self.get_row(&y);
      let out_row = transform(&in_row);

      let mut x = self.f.elem(&0u8);
      while x < self.width {
        let v = out_row.get(&x);
        m.set(&x, &y, v);
        x.inc();
      }
      y.inc();
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
    for key in other.rows.keys() {
      if !self.rows.contains_key(key) {
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

impl Into<Vec<Polynomial>> for SparseMatrix {
    fn into(self) -> Vec<Polynomial> {
      let mut vec = vec![];
      let mut i = self.height.f.elem(&0u8);
      while &i < &self.height {
        let p = Polynomial::from(&self.get_row(&i));
        vec.push(p);
        i.inc();
      }
      vec
    }
}

// coverts rows of vectors to a matrix
impl From<&Vec<SparseVec>> for SparseMatrix {
  fn from(rows: &Vec<SparseVec>) -> Self {
    assert!(rows.len() != 0, "Cannot build matrix from empty vector");
    let f = &rows[0].f;
    let width = &rows[0].size;
    let height = rows.len();

    for i in 1..height {
      if width != &rows[i].size {
        panic!("different row sizes found; size is {:?} at 0, but {:?} at {}",
          width.e, &rows[i].size.e, i)
      }
    }
    let mut m = SparseMatrix::new(f, width, &height);

    for (y, row) in rows.iter().enumerate() {
      for x in row.indices() {
        let v = row.get(&x);
        if !v.e.is_zero() {
          m.set(&x, &y, v);
        }
      }
    }
    m.normalize()
  }
}

impl Mul<&SparseMatrix> for &SparseMatrix {
    type Output = SparseMatrix;

    fn mul(self, rhs: &SparseMatrix) -> Self::Output {
      if self.width != rhs.height {
        panic!("Can only multiply matrix with height {:?}, but the rhs height is {:?}",
          self.width.e, rhs.height.e);
      }
      let mut res = SparseMatrix::new(&self.f, &rhs.width, &self.height);

      let mut y = self.f.elem(&0u8);
      while y < self.height {
        let mut x = self.f.elem(&0u8);
        while x < rhs.width {
          let lhs = &self.get_row(&y);
          let rhs = &rhs.get_column(&x);
          let v = (lhs * rhs).sum();
          if !v.e.is_zero() {
            res.set(&x, &y, &v);
          }
          x.inc();
        }
        y.inc();
      }
      res.normalize()
    }
}

impl fmt::Debug for SparseMatrix {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", &self.pretty_print())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_size() {
    let f = &PrimeField::new(&3911u16);
    let m = SparseMatrix::new(f, &2u8, &3u8);
    assert_eq!(m.width, f.elem(&2u8));
    assert_eq!(m.height, f.elem(&3u8));
  }

  #[test]
  #[should_panic]
  fn test_get_x_out_of_range() {
    std::panic::set_hook(Box::new(|_| {}));  // suppress stack trace
    let f = &PrimeField::new(&3911u16);

    let m = SparseMatrix::new(f, &2u8, &3u8);
    m.get(&2u8, &1u8);
  }

  #[test]
  #[should_panic]
  fn test_get_y_out_of_range() {
    std::panic::set_hook(Box::new(|_| {}));  // suppress stack trace
    let f = &PrimeField::new(&3911u16);

    let m = SparseMatrix::new(f, &2u8, &3u8);
    m.get(&1u8, &3u8);
  }

  #[test]
  #[should_panic]
  fn test_set_x_out_of_range() {
    std::panic::set_hook(Box::new(|_| {}));  // suppress stack trace
    let f = &PrimeField::new(&3911u16);

    let mut m = SparseMatrix::new(f, &2u8, &3u8);
    m.set(&2u8, &1u8, &f.elem(&12u8));
  }

  #[test]
  #[should_panic]
  fn test_set_y_out_of_range() {
    std::panic::set_hook(Box::new(|_| {}));  // suppress stack trace
    let f = &PrimeField::new(&3911u16);

    let mut m = SparseMatrix::new(f, &2u8, &3u8);
    m.set(&1u8, &3u8, &f.elem(&12u8));
  }

  #[test]
  fn test_get_empty() {
    let f = &PrimeField::new(&3911u16);
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
    let f = &PrimeField::new(&3911u16);
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
    let f = &PrimeField::new(&3911u16);
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

  // |1 0|
  // |0 1|
  fn gen_test_2x2_identity_matrix() -> SparseMatrix {
    let f = &PrimeField::new(&3911u16);
    let zero = &f.elem(&0u8);
    let one = &f.elem(&1u8);

    let mut v1 = SparseVec::new(f, &2u8);
    let mut v2 = SparseVec::new(f, &2u8);
    v1.set(zero, one);
    v2.set(one, one);
    let vecs = vec![v1, v2];
    SparseMatrix::from(&vecs)
  }

  // |1 2|
  // |3 4|
  fn gen_test_2x2_matrix() -> SparseMatrix {
    let f = &PrimeField::new(&3911u16);
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

  // |1 0|
  // |0 2|
  // |3 0|
  fn gen_test_2x3_matrix() -> SparseMatrix {
    let f = &PrimeField::new(&3911u16);
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

  // |1 2 3|
  // |3 2 1|
  fn gen_test_3x2_matrix() -> SparseMatrix {
    let f = &PrimeField::new(&3911u16);
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

  // |1 2|
  // |0 0|
  fn gen_test_2x2_redundant_matrix(use_empty_row: bool) -> SparseMatrix {
    let f = &PrimeField::new(&3911u16);
    let zero = &f.elem(&0u8);
    let one = &f.elem(&1u8);
    let two = &f.elem(&2u8);

    let mut v1 = SparseVec::new(f, &2u8);
    v1.set(zero, one);
    v1.set(one, two);

    if use_empty_row {
      let mut rows = HashMap::<PrimeFieldElem, SparseVec>::new();
      rows.insert(zero.clone(), v1.clone());
      rows.insert(one.clone(), SparseVec::new(f, &2u8));

      SparseMatrix {
        width: two.clone(),
        height: two.clone(),
        f: f.clone(),
        rows,
        zero: zero.clone(),
      }
    } else {
      let mut rows = HashMap::<PrimeFieldElem, SparseVec>::new();
      rows.insert(zero.clone(), v1.clone());

      SparseMatrix {
        width: two.clone(),
        height: two.clone(),
        f: f.clone(),
        rows,
        zero: zero.clone(),
      }
    }
  }

  #[test]
  fn test_eq() {
    {
      let m1 = gen_test_2x2_identity_matrix();
      let m2 = gen_test_2x2_matrix();
      let m3 = gen_test_3x2_matrix();

      assert!(&m1 == &m1);
      assert!(&m2 == &m2);
      assert!(&m3 == &m3);
    }
    {
      let m1 = gen_test_3x2_matrix();
      let m2 = gen_test_3x2_matrix();

      assert!(&m1 == &m2);
      assert!(&m2 == &m1);
    }
  }

  #[test]
  fn test_eq_with_redundant_matrix() {
      let m1 = gen_test_2x2_redundant_matrix(true);
      let m2 = gen_test_2x2_redundant_matrix(false);

      assert!(&m1 != &m2);
      assert!(&m2 != &m1);
  }

  #[test]
  fn test_non_eq() {
    let m1 = gen_test_2x2_identity_matrix();
    let m2 = gen_test_2x2_matrix();
    let m3 = gen_test_3x2_matrix();

    assert!(&m1 != &m2);
    assert!(&m2 != &m1);

    assert!(&m1 != &m3);
    assert!(&m3 != &m1);

    assert!(&m2 != &m1);
    assert!(&m1 != &m2);

    assert!(&m2 != &m3);
    assert!(&m3 != &m2);

    assert!(&m3 != &m1);
    assert!(&m1 != &m3);

    assert!(&m3 != &m2);
    assert!(&m2 != &m3);
  }

  #[test]
  fn test_normalize() {
      let m1 = gen_test_2x2_redundant_matrix(true);
      let m2 = gen_test_2x2_redundant_matrix(false);
      let m1 = m1.normalize();

      assert!(&m1 == &m2);
      assert!(&m2 == &m1);
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

  #[test]
  fn test_row_transform() {
    // |1 2 3|
    // |3 2 1|
    let m = gen_test_3x2_matrix();

    let transform = move |in_vec: &SparseVec| {
      let one = in_vec.f.elem(&1u8);
      let mut out_vec = SparseVec::new(&in_vec.f, &in_vec.size);
      let mut i = in_vec.f.elem(&0u8);
      while i < in_vec.size {
        let v = in_vec.get(&i) + &one;
        out_vec.set(&i, &v);
        i.inc();
      }
      out_vec
    };
    let m = m.row_transform(Box::new(transform));

    {
      let zero = &m.f.elem(&0u8);
      let one = &m.f.elem(&1u8);
      let two = &m.f.elem(&2u8);
      let three = &m.f.elem(&3u8);
      let four = &m.f.elem(&4u8);

      // |2 3 4|
      // |4 3 2|
      assert_eq!(m.get(zero, zero), two);
      assert_eq!(m.get(one, zero), three);
      assert_eq!(m.get(two, zero), four);
      assert_eq!(m.get(zero, one), four);
      assert_eq!(m.get(one, one), three);
      assert_eq!(m.get(two, one), two);
    }
  }

  #[test]
  fn test_multiply_column() {
    // |1 2 3|
    // |3 2 1|
    let m = gen_test_3x2_matrix();

    let zero = &m.f.elem(&0u8);
    let one = &m.f.elem(&1u8);
    let two = &m.f.elem(&2u8);
    let three = &m.f.elem(&3u8);
    let four = &m.f.elem(&4u8);
    let six = &m.f.elem(&6u8);
    let nine = &m.f.elem(&9u8);

    // |2|
    // |3|
    let mut col = SparseVec::new(&m.f, &m.height);
    col.set(zero, two);
    col.set(one, three);

    let m = m.multiply_column(&col);

    // |2 4 6|
    // |9 6 3|
    assert_eq!(m.get(zero, zero), two);
    assert_eq!(m.get(one, zero), four);
    assert_eq!(m.get(two, zero), six);

    assert_eq!(m.get(zero, one), nine);
    assert_eq!(m.get(one, one), six);
    assert_eq!(m.get(two, one), three);
  }

  #[test]
  fn test_flatten_rows() {
    // |1 2 3|
    // |3 2 1|
    let m = gen_test_3x2_matrix();

    let zero = &m.f.elem(&0u8);
    let one = &m.f.elem(&1u8);
    let two = &m.f.elem(&2u8);
    let four = &m.f.elem(&4u8);

    let row = m.flatten_rows();

    // |4 2 4|
    assert_eq!(row.get(zero), four);
    assert_eq!(row.get(one), four);
    assert_eq!(row.get(two), four);
  }
}
