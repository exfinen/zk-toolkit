use std::collections::HashMap;
use crate::building_block::field::FieldElem;

type Index = usize;
type Value = FieldElem;

#[derive(Debug)]
pub struct SparseVec {
  pub size: usize,
  elems: HashMap<Index, Value>,
}

impl SparseVec {
  pub fn new(size: usize) -> Self {
    SparseVec {
      size,
      elems: HashMap::<Index, Value>::new(),
    }
  }
  pub fn set(&mut self, index: Index, n: Value) {
    self.elems.insert(index, n);
  }

  pub fn indices(&self) -> Vec<usize> {
    let mut vec = vec![];
    for x in self.elems.keys() {
      vec.push(*x);
    }
    vec
  }
}
