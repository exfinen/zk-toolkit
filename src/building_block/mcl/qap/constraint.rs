use crate::building_block::mcl::mcl_sparse_vec::MclSparseVec;

#[derive(Clone)]
pub struct Constraint {
  pub a: MclSparseVec,
  pub b: MclSparseVec,
  pub c: MclSparseVec,
}

impl Constraint {
  pub fn new(
    a: &MclSparseVec,
    b: &MclSparseVec,
    c: &MclSparseVec,
  ) -> Self {
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

