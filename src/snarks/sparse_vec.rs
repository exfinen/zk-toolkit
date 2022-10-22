use std::collections::HashMap;
use crate::building_block::field::{Field, FieldElem};
use num_traits::Zero;
use core::ops::Index;

type SvIndex = usize;

#[derive(Clone)]
pub struct SparseVec {
  f: Field,
  zero: FieldElem,
  pub size: usize,
  elems: HashMap<SvIndex, FieldElem>,
}

impl std::fmt::Debug for SparseVec {
  fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
    let mut keys: Vec<SvIndex> = self.elems.keys().cloned().collect();
    keys.sort();
    let xs: Vec<String> = keys.iter().map(|k| { format!("{}->{:?}", k, self.elems.get(k).unwrap().n) }).collect();
    write!(fmt, "[{}]", xs.join(","))
  }
}

impl SparseVec {
  pub fn new(f: &Field, size: usize) -> Self {
    if size == 0 {
      panic!("Size must be greater than 0");
    }
    SparseVec {
      f: f.clone(),
      zero: f.elem(&0u8),
      size,
      elems: HashMap::<SvIndex, FieldElem>::new(),
    }
  }

  pub fn set(&mut self, index: &SvIndex, n: FieldElem) {
    if index >= &self.size {
      panic!("Index {} is out of range. The size of vector is {}", index, self.size);
    }
    self.elems.insert(*index, n);
  }

  pub fn get(&self, index: &SvIndex) -> &FieldElem {
    if index >= &self.size {
      panic!("Index {} is out of range. The size of vector is {}", index, self.size);
    }
    if self.elems.contains_key(index) {
      self.elems.get(index).unwrap()
    } else {
      &self.zero
    }
  }

  pub fn indices_with_value(&self) -> Vec<usize> {
    let mut vec = vec![];
    for x in self.elems.keys() {
      vec.push(*x);
    }
    vec
  }

  // TODO clean up
  pub fn sum(&self) -> FieldElem {
    let mut values = vec![];
    for value in self.elems.values() {
      values.push(value);
    }
    let mut sum = values[0].clone();
    for value in &values[1..] {
      sum = sum + *value;
    }
    sum
  }
}

impl PartialEq for SparseVec {
  fn eq(&self, other: &SparseVec) -> bool {
    if self.size != other.size { return false; }
    for index in self.elems.keys() {
      let other_elem = other.get(index);
      let this_elem = self.get(index);
      if this_elem != other_elem { return false; }
    }
    true
  }
}

impl Index<&usize> for SparseVec {
  type Output = FieldElem;

  fn index(&self, index: &usize) -> &Self::Output {
    &self.get(index)
  }
}

// returns Hadamard product
impl std::ops::Mul<&SparseVec> for &SparseVec {
    type Output = SparseVec;

    fn mul(self, rhs: &SparseVec) -> Self::Output {
      if self.size != rhs.size {
        panic!("Expected size of rhs to be {}, but got {}", self.size, rhs.size);
      }

      let mut ret = SparseVec::new(&self.f, self.size);
      for index in self.elems.keys() {
        let l = self.get(index);
        let r = rhs.get(index);
        if !l.n.is_zero() && !r.n.is_zero() {
          ret.set(index, l * r);
        }
      }
      ret
    }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::building_block::field::Field;

  #[test]
  #[should_panic]
  fn test_empty_vec() {
    std::panic::set_hook(Box::new(|_| {}));  // suppress stack trace
    let f = &Field::new(&3911u16);
    SparseVec::new(f, 0);
  }

  #[test]
  #[should_panic]
  fn test_bad_set() {
    std::panic::set_hook(Box::new(|_| {}));  // suppress stack trace

    let f = &Field::new(&3911u16);
    let mut vec = SparseVec::new(f, 3);
    assert_eq!(vec.elems.len(), 0);

    let f = &Field::new(&3911u16);
    vec.set(&3, f.elem(&2u8));
  }

  #[test]
  fn test_good_set() {
    let f = &Field::new(&3911u16);
    let mut vec = SparseVec::new(f, 3);
    assert_eq!(vec.elems.len(), 0);

    let f = &Field::new(&3911u16);
    vec.set(&2, f.elem(&2u8));
    assert_eq!(vec.elems.len(), 1);
    assert_eq!(vec.elems.get(&2).unwrap(), &f.elem(&2u8));

    // setting the same index should overwrite
    vec.set(&2, f.elem(&3u8));
    assert_eq!(vec.elems.len(), 1);
    assert_eq!(vec.elems.get(&2).unwrap(), &f.elem(&3u8));
  }

  #[test]
  fn test_good_get() {
    let f = &Field::new(&3911u16);
    let one = f.elem(&1u8);
    let two = f.elem(&2u8);
    let three = f.elem(&3u8);
    let mut vec = SparseVec::new(f, 3);
    vec.set(&0, one.clone());
    vec.set(&1, two.clone());
    vec.set(&2, one.clone());
    assert_eq!(vec.get(&0), &one);
    assert_eq!(vec.get(&1), &two);
    assert_eq!(vec.get(&2), &three);
  }
  #[test]
  #[should_panic]
  fn test_get_index_out_of_range() {
    std::panic::set_hook(Box::new(|_| {}));  // suppress stack trace

    let f = &Field::new(&3911u16);
    let vec = SparseVec::new(f, 1);
    vec.get(&2);
  }

  #[test]
  #[should_panic]
  fn test_get_index_without_value() {
    std::panic::set_hook(Box::new(|_| {}));  // suppress stack trace

    let f = &Field::new(&3911u16);
    let zero = &f.elem(&0u8);
    let vec = SparseVec::new(f, 3);
    assert_eq!(vec.get(&0), zero);
    assert_eq!(vec.get(&1), zero);
    assert_eq!(vec.get(&2), zero);
  }

  #[test]
  fn test_indices() {
    let f = &Field::new(&3911u16);
    let mut vec = SparseVec::new(f, 3);
    let f = &Field::new(&3911u16);

    vec.set(&1, f.elem(&2u8));
    vec.set(&2, f.elem(&4u8));

    let indices = vec.indices_with_value();

    assert_eq!(indices.len(), 2);
    assert!(indices.contains(&1));
    assert!(indices.contains(&2));
  }

  #[test]
  fn test_mutiply_no_matching_elems() {
    let f = &Field::new(&3911u16);
    let mut vec_a = SparseVec::new(f, 3);
    let mut vec_b = SparseVec::new(f, 3);

    vec_a.set(&1, f.elem(&2u8));
    vec_b.set(&2, f.elem(&3u8));

    let vec_c = &vec_a * &vec_b;

    assert_eq!(vec_c.elems.len(), 0);
  }

  #[test]
  fn test_mutiply_elems_fully_matching_1_elem() {
    let f = &Field::new(&3911u16);
    let mut vec_a = SparseVec::new(f, 3);
    let mut vec_b = SparseVec::new(f, 3);

    vec_a.set(&1, f.elem(&2u8));
    vec_b.set(&1, f.elem(&3u8));

    let vec_c = &vec_a * &vec_b;

    assert_eq!(vec_c.elems.len(), 1);
    assert_eq!(vec_c.get(&1), &f.elem(&6u8));
  }

  #[test]
  fn test_mutiply_elems_fully_matching_2_elems() {
    let f = &Field::new(&3911u16);
    let mut vec_a = SparseVec::new(f, 3);
    let mut vec_b = SparseVec::new(f, 3);

    vec_a.set(&1, f.elem(&2u8));
    vec_a.set(&2, f.elem(&3u8));
    vec_b.set(&1, f.elem(&4u8));
    vec_b.set(&2, f.elem(&5u8));

    let vec_c = &vec_a * &vec_b;

    assert_eq!(vec_c.elems.len(), 2);
    assert_eq!(vec_c.get(&1), &f.elem(&8u8));
    assert_eq!(vec_c.get(&2), &f.elem(&15u8));
  }

  #[test]
  fn test_mutiply_elems_partially_matching() {
    let f = &Field::new(&3911u16);
    let mut vec_a = SparseVec::new(f, 3);
    let mut vec_b = SparseVec::new(f, 3);

    vec_a.set(&1, f.elem(&2u8));
    vec_a.set(&2, f.elem(&5u8));
    vec_b.set(&1, f.elem(&3u8));

    let vec_c = &vec_a * &vec_b;

    assert_eq!(vec_c.elems.len(), 1);
    assert_eq!(vec_c.get(&1), &f.elem(&6u8));
  }

  #[test]
  fn test_sum() {
    let f = &Field::new(&3911u16);
    let mut vec = SparseVec::new(f, 3);

    vec.set(&1, f.elem(&2u8));
    vec.set(&2, f.elem(&4u8));

    let sum = vec.sum();
    assert_eq!(sum, f.elem(&6u8));
  }

  #[test]
  fn test_eq_different_sizes() {
    let f = &Field::new(&3911u16);
    let vec_a = SparseVec::new(f, 3);
    let vec_b = SparseVec::new(f, 4);
    assert_ne!(vec_a, vec_b);
  }

  #[test]
  fn test_eq_empty() {
    let f = &Field::new(&3911u16);
    let vec_a = SparseVec::new(f, 3);
    let vec_b = SparseVec::new(f, 3);
    assert_eq!(vec_a, vec_b);
  }

  #[test]
  fn test_eq_non_empty() {
    let f = &Field::new(&3911u16);
    let mut vec_a = SparseVec::new(f, 3);
    let mut vec_b = SparseVec::new(f, 3);

    vec_a.set(&1, f.elem(&92u8));
    vec_b.set(&1, f.elem(&92u8));
    assert_eq!(vec_a, vec_b);
  }

  #[test]
  fn test_not_eq_non_empty() {
    let f = &Field::new(&3911u16);
    let mut vec_a = SparseVec::new(f, 3);
    let mut vec_b = SparseVec::new(f, 3);

    vec_a.set(&1, f.elem(&92u8));
    vec_b.set(&1, f.elem(&13u8));
    assert_ne!(vec_a, vec_b);
  }
}
