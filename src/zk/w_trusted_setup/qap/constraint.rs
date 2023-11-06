use crate::building_block::field::sparse_vec::SparseVec;

#[derive(Clone)]
pub struct Constraint {
  pub a: SparseVec,
  pub b: SparseVec,
  pub c: SparseVec,
}

impl Constraint {
  pub fn new(a: &SparseVec, b: &SparseVec, c: &SparseVec) -> Self {
    let a = a.clone();
    let b = b.clone();
    let c = c.clone();
    Constraint { a, b, c }
  }
}

impl std::fmt::Debug for Constraint {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "{:?} . w * {:?} . w - {:?} . w = 0", self.a, self.b, self.c)
  }
}
