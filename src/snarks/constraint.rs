use crate::snarks::sparse_vec::SparseVec;

#[derive(Clone, Debug)]
pub struct Constraint {
  pub a: SparseVec,
  pub b: SparseVec,
  pub c: SparseVec,
}
