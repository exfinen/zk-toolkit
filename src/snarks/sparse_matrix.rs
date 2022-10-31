use crate::building_block::field::{Field, FieldElem};
use crate::building_block::to_biguint::ToBigUint;
use crate::snarks::sparse_vec::SparseVec;
use std::collections::HashMap;

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

    if !self.rows.contains_key(&y) {
      let vec = SparseVec::new(&self.f, &self.width);
      self.rows.insert(y.clone(), vec);
    }
    self.rows.get_mut(&y).unwrap().set(&x, v);
  }

  pub fn get(&self, x: &impl ToBigUint, y: &impl ToBigUint) -> &FieldElem {
    let y = self.f.elem(y);
    if !self.rows.contains_key(&y) {
      &self.zero
    } else {
      self.rows.get(&y).unwrap().get(x)
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test1() {
    let f = &Field::new(&3911u16);
    let zero = &f.elem(&0u8);

    let m = SparseMatrix::new(f, &2u8, &3u8);
    assert_eq!(m.width, f.elem(&2u8));
    assert_eq!(m.height, f.elem(&3u8));

    for x in 0u8..2 {
      for y in 0u8..3 {
        assert_eq!(m.get(&x, &y), zero);
      }
    }
  }
}
