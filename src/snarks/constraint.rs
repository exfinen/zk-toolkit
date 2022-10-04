use crate::snarks::sparse_vec::SparseVec;

#[derive(Clone)]
pub struct Constraint {
  pub a: SparseVec,
  pub b: SparseVec,
  pub c: SparseVec,
}

impl std::fmt::Debug for Constraint {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "{:?} * {:?} = {:?}", self.a, self.b, self.c)
  }
}
