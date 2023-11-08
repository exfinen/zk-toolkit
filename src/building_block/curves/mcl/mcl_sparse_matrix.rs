use num_traits::Zero;

use crate::building_block::curves::mcl::{
  mcl_fr::MclFr,
  mcl_sparse_vec::MclSparseVec,
  polynomial::Polynomial,
};
use std::{
  collections::HashMap,
  convert::From,
  fmt,
  ops::Mul,
};

pub struct MclSparseMatrix {
  pub width: MclFr,
  pub height: MclFr,
  rows: HashMap<MclFr, MclSparseVec>,
  zero: MclFr,
}

impl MclSparseMatrix {
  pub fn new(width: &MclFr, height: &MclFr) -> Self {
    let zero = MclFr::zero();
    let rows = HashMap::new();
    Self {
      width: width.clone(),
      height: height.clone(),
      rows,
      zero,
    }
  }

  pub fn pretty_print(&self) -> String {
    let mut s = String::new();
    let empty_row = &MclSparseVec::new(&self.width);

    let mut y = MclFr::zero();
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

  pub fn multiply_column(&self, col: &MclSparseVec) -> Self {
    if col.size != self.height {
      panic!("column size is expected to be {:?}, but got {:?}",
        self.height, col.size)
    }
    let mut m = MclSparseMatrix::new(&self.width, &self.height);

    let mut y = MclFr::from(0);
    while y < col.size {
      let mut x = MclFr::from(0);
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

  pub fn flatten_rows(&self) -> MclSparseVec {
    let mut vec = MclSparseVec::new(&self.width);

    let mut y = MclFr::from(0);
    while y < self.height {
      println!("y={:?}", &y);
      let mut x = MclFr::from(0);
      while x < self.width {
        println!("x={:?}", &x);
        let v = vec.get(&x) + self.get(&x, &y);
        vec.set(&x, &v);
        x.inc();
      }
      y.inc();
    }
    vec
  }

  pub fn set(&mut self, x: &MclFr, y: &MclFr, v: &MclFr) -> () {
    if x >= &self.width || y >= &self.height {
      panic!("For {:?} x {:?} matrix, ({:?}, {:?}) is out of range",
        self.width, self.height, x, y);
    }
    if v.is_zero() {  // don't set if zero
      return;
    }

    if !self.rows.contains_key(&y) {
      let vec = MclSparseVec::new(&self.width);
      self.rows.insert(y.clone(), vec);
    }
    self.rows.get_mut(&y).unwrap().set(&x, &v);
  }

  pub fn get(&self, x: &MclFr, y: &MclFr) -> &MclFr {
    if x >= &self.width || y >= &self.height {
      panic!("For {:?} x {:?} matrix, ({:?}, {:?}) is out of range",
        self.width, self.height, x, y);
    }
    if !self.rows.contains_key(&y) {
      &self.zero
    } else {
      self.rows.get(&y).unwrap().get(&x)
    }
  }

  pub fn get_row(&self, y: &MclFr) -> MclSparseVec {
    assert!(y < &self.height);
    let mut row = MclSparseVec::new(&self.width);

    if !self.rows.contains_key(y) {
      return row;
    }
    let src_row = self.rows.get(y).unwrap();
    for x in src_row.indices() {
      let v = src_row.get(&x);
      if !v.is_zero() {
        row.set(&x, v);
      }
    }
    row
  }

  pub fn get_column(&self, x: &MclFr) -> MclSparseVec {
    assert!(x < &self.width);
    let mut col = MclSparseVec::new(&self.height);

    for y in self.rows.keys() {
      let src_row = self.rows.get(&y).unwrap();
      let v = src_row.get(x);
      if !v.is_zero() {
        col.set(y, v);
      }
    }
    col
  }

  pub fn transpose(&self) -> MclSparseMatrix {
    let mut m = MclSparseMatrix::new(&self.height, &self.width);
    for y in self.rows.keys() {
      let src_row = self.rows.get(&y).unwrap();

      for x in src_row.indices() {
        let v = src_row.get(&x);
        if !v.is_zero() {
          m.set(y, &x, v);
        }
      }
    }
    m
  }

  // remove empty rows
  pub fn normalize(&self) -> MclSparseMatrix {
    let mut m = MclSparseMatrix::new(&self.width, &self.height);
    for row_key in self.rows.keys() {
      let row = self.rows.get(row_key).unwrap();
      if !row.is_empty() {
        m.rows.insert(row_key.clone(), row.clone());
      }
    }
    m
  }

  pub fn row_transform(&self, transform: Box<dyn Fn(&MclSparseVec) -> MclSparseVec>) -> MclSparseMatrix {
    let mut m = MclSparseMatrix::new(&self.width, &self.height);

    let mut y = MclFr::zero();
    while y < self.height {
      let in_row = self.get_row(&y);
      let out_row = transform(&in_row);

      let mut x = MclFr::zero();
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

impl PartialEq for MclSparseMatrix {
  fn eq(&self, other: &MclSparseMatrix) -> bool {
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

impl Into<Vec<Polynomial>> for MclSparseMatrix {
    fn into(self) -> Vec<Polynomial> {
      let mut vec = vec![];
      let mut i = MclFr::zero();
      while &i < &self.height {
        let p = Polynomial::from(&self.get_row(&i));
        vec.push(p);
        i.inc();
      }
      vec
    }
}

// coverts rows of vectors to a matrix
impl From<&Vec<MclSparseVec>> for MclSparseMatrix {
  fn from(rows: &Vec<MclSparseVec>) -> Self {
    assert!(rows.len() != 0, "Cannot build matrix from empty vector");
    let width = &rows[0].size;
    let height = rows.len();

    for i in 1..height {
      if width != &rows[i].size {
        panic!("different row sizes found; size is {:?} at 0, but {:?} at {}",
          width, &rows[i].size, i)
      }
    }
    let mut m = MclSparseMatrix::new(width, &MclFr::from(height));

    for (y, row) in rows.iter().enumerate() {
      for x in row.indices() {
        let v = row.get(&x);
        if !v.is_zero() {
          m.set(&x, &MclFr::from(y), v);
        }
      }
    }
    m.normalize()
  }
}

impl Mul<&MclSparseMatrix> for &MclSparseMatrix {
    type Output = MclSparseMatrix;

    fn mul(self, rhs: &MclSparseMatrix) -> Self::Output {
      if self.width != rhs.height {
        panic!("Can only multiply matrix with height {:?}, but the rhs height is {:?}",
          self.width, rhs.height);
      }
      let mut res = MclSparseMatrix::new(&rhs.width, &self.height);

      let mut y = MclFr::zero();
      while y < self.height {
        let mut x = MclFr::zero();
        while x < rhs.width {
          let lhs = &self.get_row(&y);
          let rhs = &rhs.get_column(&x);
          let v = (lhs * rhs).sum();
          if !v.is_zero() {
            res.set(&x, &y, &v);
          }
          x.inc();
        }
        y.inc();
      }
      res.normalize()
    }
}

impl fmt::Debug for MclSparseMatrix {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", &self.pretty_print())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::building_block::curves::mcl::mcl_initializer::MclInitializer;

  #[test]
  fn test_size() {
    MclInitializer::init();
    let m = MclSparseMatrix::new(&MclFr::from(2), &MclFr::from(3));
    assert_eq!(m.width, MclFr::from(2));
    assert_eq!(m.height, MclFr::from(3));
  }

  #[test]
  #[should_panic]
  fn test_get_x_out_of_range() {
    MclInitializer::init();
    std::panic::set_hook(Box::new(|_| {}));  // suppress stack trace

    let m = MclSparseMatrix::new(&MclFr::from(2), &MclFr::from(3));
    m.get(&MclFr::from(2), &MclFr::from(1));
  }

  #[test]
  #[should_panic]
  fn test_get_y_out_of_range() {
    MclInitializer::init();
    std::panic::set_hook(Box::new(|_| {}));  // suppress stack trace

    let m = MclSparseMatrix::new(&MclFr::from(2), &MclFr::from(3));
    m.get(&MclFr::from(1), &MclFr::from(3));
  }

  #[test]
  #[should_panic]
  fn test_set_x_out_of_range() {
    MclInitializer::init();
    std::panic::set_hook(Box::new(|_| {}));  // suppress stack trace

    let mut m = MclSparseMatrix::new(&MclFr::from(2), &MclFr::from(3));
    m.set(&MclFr::from(2), &MclFr::from(1), &MclFr::from(12));
  }

  #[test]
  #[should_panic]
  fn test_set_y_out_of_range() {
    MclInitializer::init();
    std::panic::set_hook(Box::new(|_| {}));  // suppress stack trace

    let mut m = MclSparseMatrix::new(&MclFr::from(2), &MclFr::from(3));
    m.set(&MclFr::from(1), &MclFr::from(3), &MclFr::from(12));
  }

  #[test]
  fn test_get_empty() {
    MclInitializer::init();
    let zero = &MclFr::from(0);

    let m = MclSparseMatrix::new(&MclFr::from(2), &MclFr::from(3));
    for x in 0..2 {
      for y in 0..3 {
        assert_eq!(m.get(&MclFr::from(x), &MclFr::from(y)), zero);
      }
    }
  }

  #[test]
  fn test_mul() {
    MclInitializer::init();
    let zero = &MclFr::from(0);

    let m = MclSparseMatrix::new(&MclFr::from(2), &MclFr::from(3));
    for x in 0..2 {
      for y in 0..3 {
        assert_eq!(m.get(&MclFr::from(x), &MclFr::from(y)), zero);
      }
    }
  }

  #[test]
  fn test_get_existing_and_non_existing_cells() {
    MclInitializer::init();
    let zero = &MclFr::from(0);
    let one = &MclFr::from(1);
    let two = &MclFr::from(2);
    let three = &MclFr::from(3);
    let eight = &MclFr::from(8);
    let nine = &MclFr::from(9);

    let mut m = MclSparseMatrix::new(two, three);
    m.set(zero, two, nine);
    m.set(one, one, eight);

    assert_eq!(m.get(zero, two), nine);
    assert_eq!(m.get(one, one), eight);
    assert_eq!(m.get(one, two), zero);
  }

  #[test]
  #[should_panic]
  fn test_from_empty_vec() {
    MclInitializer::init();
    std::panic::set_hook(Box::new(|_| {}));  // suppress stack trace
    let _ = MclSparseMatrix::from(&vec![]);
  }

  // |1 0|
  // |0 1|
  fn gen_test_2x2_identity_matrix() -> MclSparseMatrix {
    MclInitializer::init();
    let zero = &MclFr::from(0);
    let one = &MclFr::from(1);

    let mut v1 = MclSparseVec::new(&MclFr::from(2));
    let mut v2 = MclSparseVec::new(&MclFr::from(2));
    v1.set(zero, one);
    v2.set(one, one);
    let vecs = vec![v1, v2];
    MclSparseMatrix::from(&vecs)
  }

  // |1 2|
  // |3 4|
  fn gen_test_2x2_matrix() -> MclSparseMatrix {
    MclInitializer::init();
    let zero = &MclFr::from(0);
    let one = &MclFr::from(1);
    let two = &MclFr::from(2);
    let three = &MclFr::from(3);
    let four = &MclFr::from(4);

    let mut v1 = MclSparseVec::new(&MclFr::from(2));
    let mut v2 = MclSparseVec::new(&MclFr::from(2));
    v1.set(zero, one);
    v1.set(one, two);
    v2.set(zero, three);
    v2.set(one, four);
    let vecs = vec![v1, v2];
    MclSparseMatrix::from(&vecs)
  }

  // |1 0|
  // |0 2|
  // |3 0|
  fn gen_test_2x3_matrix() -> MclSparseMatrix {
    MclInitializer::init();
    let zero = &MclFr::from(0);
    let one = &MclFr::from(1);
    let two = &MclFr::from(2);
    let three = &MclFr::from(3);

    let mut v1 = MclSparseVec::new(&MclFr::from(2));
    let mut v2 = MclSparseVec::new(&MclFr::from(2));
    let mut v3 = MclSparseVec::new(&MclFr::from(2));
    v1.set(zero, one);
    v2.set(one, two);
    v3.set(zero, three);
    let vecs = vec![v1, v2, v3];
    MclSparseMatrix::from(&vecs)
  }

  // |1 2 3|
  // |3 2 1|
  fn gen_test_3x2_matrix() -> MclSparseMatrix {
    MclInitializer::init();
    let zero = &MclFr::from(0);
    let one = &MclFr::from(1);
    let two = &MclFr::from(2);
    let three = &MclFr::from(3);

    let mut v1 = MclSparseVec::new(&MclFr::from(3));
    let mut v2 = MclSparseVec::new(&MclFr::from(3));
    v1.set(zero, one);
    v1.set(one, two);
    v1.set(two, three);
    v2.set(zero, three);
    v2.set(one, two);
    v2.set(two, one);
    let vecs = vec![v1, v2];
    MclSparseMatrix::from(&vecs)
  }

  // |1 2|
  // |0 0|
  fn gen_test_2x2_redundant_matrix(use_empty_row: bool) -> MclSparseMatrix {
    MclInitializer::init();
    let zero = &MclFr::from(0);
    let one = &MclFr::from(1);
    let two = &MclFr::from(2);

    let mut v1 = MclSparseVec::new(&MclFr::from(2));
    v1.set(zero, one);
    v1.set(one, two);

    if use_empty_row {
      let mut rows = HashMap::<MclFr, MclSparseVec>::new();
      rows.insert(zero.clone(), v1.clone());
      rows.insert(one.clone(), MclSparseVec::new(&MclFr::from(2)));

      MclSparseMatrix {
        width: two.clone(),
        height: two.clone(),
        rows,
        zero: zero.clone(),
      }
    } else {
      let mut rows = HashMap::<MclFr, MclSparseVec>::new();
      rows.insert(zero.clone(), v1.clone());

      MclSparseMatrix {
        width: two.clone(),
        height: two.clone(),
        rows,
        zero: zero.clone(),
      }
    }
  }

  #[test]
  fn test_eq() {
    MclInitializer::init();
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
    MclInitializer::init();
    let m1 = gen_test_2x2_redundant_matrix(true);
    let m2 = gen_test_2x2_redundant_matrix(false);

    assert!(&m1 != &m2);
    assert!(&m2 != &m1);
  }

  #[test]
  fn test_non_eq() {
    MclInitializer::init();
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
    MclInitializer::init();
    let m1 = gen_test_2x2_redundant_matrix(true);
    let m2 = gen_test_2x2_redundant_matrix(false);
    let m1 = m1.normalize();

    assert!(&m1 == &m2);
    assert!(&m2 == &m1);
  }

  #[test]
  fn test_from_sparse_vecs() {
    MclInitializer::init();
    let m = gen_test_2x3_matrix();
    let zero = &MclFr::from(0);
    let one = &MclFr::from(1);
    let two = &MclFr::from(2);
    let three = &MclFr::from(3);

    assert_eq!(&m.width, two);
    assert_eq!(&m.height, three);

    assert_eq!(m.get(zero, zero), one);
    assert_eq!(m.get(one, one), two);
    assert_eq!(m.get(zero, two), three);
  }

  #[test]
  #[should_panic]
  fn test_get_row_x_out_of_range() {
    MclInitializer::init();
    std::panic::set_hook(Box::new(|_| {}));  // suppress stack trace
    let m = gen_test_2x3_matrix();
    let _ = m.get_row(&m.height);
  }

  #[test]
  fn test_get_row_x_within_range() {
    MclInitializer::init();
    let m = gen_test_2x3_matrix();
    let zero = &MclFr::from(0);
    let one = &MclFr::from(1);
    let two = &MclFr::from(2);
    let three = &MclFr::from(3);

    let r0 = m.get_row(zero);
    assert_eq!(r0.get(zero), one);
    assert_eq!(r0.get(one), zero);

    let r1 = m.get_row(one);
    assert_eq!(r1.get(zero), zero);
    assert_eq!(r1.get(one), two);

    let r2 = m.get_row(two);
    assert_eq!(r2.get(zero), three);
    assert_eq!(r2.get(one), zero);
  }

  #[test]
  #[should_panic]
  fn test_get_column_out_of_range() {
    MclInitializer::init();
    std::panic::set_hook(Box::new(|_| {}));  // suppress stack trace
    let m = gen_test_2x3_matrix();
    let _ = m.get_column(&m.width);
  }

  #[test]
  fn test_get_column_within_range() {
    MclInitializer::init();
    let m = gen_test_2x3_matrix();
    let zero = &MclFr::from(0);
    let one = &MclFr::from(1);
    let two = &MclFr::from(2);
    let three = &MclFr::from(3);

    let c0 = m.get_column(zero);
    assert_eq!(c0.get(zero), one);
    assert_eq!(c0.get(one), zero);
    assert_eq!(c0.get(two), three);

    let c1 = m.get_column(one);
    assert_eq!(c1.get(zero), zero);
    assert_eq!(c1.get(one), two);
    assert_eq!(c1.get(two), zero);
  }

  #[test]
  #[should_panic]
  fn test_mul_incompatible_matrices() {
    MclInitializer::init();
    std::panic::set_hook(Box::new(|_| {}));  // suppress stack trace
    let m = gen_test_2x3_matrix();
    let _ = &m * &m;
  }

  #[test]
  fn test_mul_different_sizes() {
    MclInitializer::init();
    let m1 = gen_test_3x2_matrix();
    let m2 = gen_test_2x3_matrix();
    let m3 = &m1 * &m2;

    let zero = &MclFr::from(0);
    let one = &MclFr::from(1);
    let two = &MclFr::from(2);
    let six = &MclFr::from(6);
    let four = &MclFr::from(4);
    let ten = &MclFr::from(10);

    assert_eq!(&m3.width, two);
    assert_eq!(&m3.height, two);
    assert_eq!(m3.get(zero, zero), ten);
    assert_eq!(m3.get(one, zero), four);
    assert_eq!(m3.get(zero, one), six);
    assert_eq!(m3.get(one, one), four);
  }

  #[test]
  fn test_mul_identity() {
    MclInitializer::init();
    let m1 = gen_test_2x2_matrix();
    let m2 = gen_test_2x2_identity_matrix();
    let m3 = &m1 * &m2;

    let two = &MclFr::from(2);
    assert_eq!(&m3.width, two);
    assert_eq!(&m3.height, two);

    assert!(&m1 == &m3);
  }

  #[test]
  fn test_transpose() {
    MclInitializer::init();
    let m = &gen_test_3x2_matrix();
    let mt = &m.transpose();

    assert_eq!(m.height, mt.width);
    assert_eq!(m.width, mt.height);

    let mut x = MclFr::zero();
    while x < m.width {
      let mut y = MclFr::zero();
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
    MclInitializer::init();
    // |1 2 3|
    // |3 2 1|
    let m = gen_test_3x2_matrix();

    let transform = move |in_vec: &MclSparseVec| {
      let one = MclFr::from(1);
      let mut out_vec = MclSparseVec::new(&in_vec.size);
      let mut i = MclFr::from(0);
      while i < in_vec.size {
        let v = in_vec.get(&i) + &one;
        out_vec.set(&i, &v);
        i.inc();
      }
      out_vec
    };
    let m = m.row_transform(Box::new(transform));

    {
      let zero = &MclFr::from(0);
      let one = &MclFr::from(1);
      let two = &MclFr::from(2);
      let three = &MclFr::from(3);
      let four = &MclFr::from(4);

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
    MclInitializer::init();
    // |1 2 3|
    // |3 2 1|
    let m = gen_test_3x2_matrix();

    let zero = &MclFr::from(0);
    let one = &MclFr::from(1);
    let two = &MclFr::from(2);
    let three = &MclFr::from(3);
    let four = &MclFr::from(4);
    let six = &MclFr::from(6);
    let nine = &MclFr::from(9);

    // |2|
    // |3|
    let mut col = MclSparseVec::new(&m.height);
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
    MclInitializer::init();
    // |1 2 3|
    // |3 2 1|
    let m = gen_test_3x2_matrix();

    let zero = &MclFr::from(0);
    let one = &MclFr::from(1);
    let two = &MclFr::from(2);
    let four = &MclFr::from(4);

    let row = m.flatten_rows();

    // |4 2 4|
    assert_eq!(row.get(zero), four);
    assert_eq!(row.get(one), four);
    assert_eq!(row.get(two), four);
  }
}
