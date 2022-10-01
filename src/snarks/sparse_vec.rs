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
    if size == 0 {
      panic!("Size must be greater than 0");
    }
    SparseVec {
      size,
      elems: HashMap::<Index, Value>::new(),
    }
  }

  pub fn set(&mut self, index: Index, n: Value) {
    if index >= self.size {
      panic!("Index {} is out of range. The size of vector is {}", index, self.size);
    }
    self.elems.insert(index, n);
  }

  pub fn safe_get(&self, index: &Index) -> Option<&Value> {
    if index >= &self.size {
      None
    } else {
      self.elems.get(index)
    }
  }

  pub fn get(&self, index: &Index) -> Value {
    if index >= &self.size {
      panic!("Index {} is out of range. The size of vector is {}", index, self.size);
    }
    self.elems.get(index).unwrap().clone()
  }

  pub fn indices(&self) -> Vec<usize> {
    let mut vec = vec![];
    for x in self.elems.keys() {
      vec.push(*x);
    }
    vec
  }

  // TODO make this cleaner
  pub fn sum(&self) -> Value {
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

// returns Hadamard product
impl std::ops::Mul<&SparseVec> for &SparseVec {
    type Output = SparseVec;

    fn mul(self, rhs: &SparseVec) -> Self::Output {
      if self.size != rhs.size {
        panic!("Expected size of rhs to be {}, but got {}", self.size, rhs.size);
      }

      let mut ret = SparseVec::new(self.size);
      for index in self.elems.keys() {
        match rhs.safe_get(index) {
          Some(r) => {
            let l = self.get(index);
            ret.set(*index, l * r);
          },
          None => (),
        };
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
    SparseVec::new(0);
  }

  #[test]
  #[should_panic]
  fn test_bad_set() {
    std::panic::set_hook(Box::new(|_| {}));  // suppress stack trace

    let mut vec = SparseVec::new(3);
    assert_eq!(vec.elems.len(), 0);

    let f = &Field::new(&3911u16);
    vec.set(3, f.elem(&2u8));
  }

  #[test]
  fn test_good_set() {
    let mut vec = SparseVec::new(3);
    assert_eq!(vec.elems.len(), 0);

    let f = &Field::new(&3911u16);
    vec.set(2, f.elem(&2u8));
    assert_eq!(vec.elems.len(), 1);
    assert_eq!(vec.elems.get(&2).unwrap(), &f.elem(&2u8));

    // setting the same index should overwrite
    vec.set(2, f.elem(&3u8));
    assert_eq!(vec.elems.len(), 1);
    assert_eq!(vec.elems.get(&2).unwrap(), &f.elem(&3u8));
  }

  #[test]
  #[should_panic]
  fn test_bad_get() {
    std::panic::set_hook(Box::new(|_| {}));  // suppress stack trace

    let vec = SparseVec::new(3);
    vec.get(&3);
  }

  #[test]
  #[should_panic]
  fn test_get_non_existing_index() {
    std::panic::set_hook(Box::new(|_| {}));  // suppress stack trace

    let vec = SparseVec::new(3);
    vec.get(&2);
  }

  #[test]
  fn test_safe_get_out_of_range_index() {
    let vec = SparseVec::new(3);
    assert_eq!(vec.safe_get(&3), None);
  }

  #[test]
  fn test_safe_get_non_existing_index() {
    let mut vec = SparseVec::new(3);
    assert_eq!(vec.safe_get(&2), None);

    let f = &Field::new(&3911u16);
    vec.set(1, f.elem(&2u8));

    assert_eq!(vec.safe_get(&2), None);
  }

  #[test]
  fn test_safe_get_existing_index() {
    let mut vec = SparseVec::new(3);
    let f = &Field::new(&3911u16);

    vec.set(1, f.elem(&2u8));
    assert_eq!(vec.safe_get(&1), Some(&f.elem(&2u8)));
  }

  #[test]
  fn test_indices() {
    let mut vec = SparseVec::new(3);
    let f = &Field::new(&3911u16);

    vec.set(1, f.elem(&2u8));
    vec.set(2, f.elem(&4u8));

    let indices = vec.indices();

    assert_eq!(indices.len(), 2);
    assert!(indices.contains(&1));
    assert!(indices.contains(&2));
  }

  #[test]
  fn test_mutiply_no_matching_elems() {
    let f = &Field::new(&3911u16);
    let mut vec_a = SparseVec::new(3);
    let mut vec_b = SparseVec::new(3);

    vec_a.set(1, f.elem(&2u8));
    vec_b.set(2, f.elem(&3u8));

    let vec_c = &vec_a * &vec_b;

    assert_eq!(vec_c.elems.len(), 0);
  }

  #[test]
  fn test_mutiply_elems_fully_matching_1_elem() {
    let f = &Field::new(&3911u16);
    let mut vec_a = SparseVec::new(3);
    let mut vec_b = SparseVec::new(3);

    vec_a.set(1, f.elem(&2u8));
    vec_b.set(1, f.elem(&3u8));

    let vec_c = &vec_a * &vec_b;

    assert_eq!(vec_c.elems.len(), 1);
    assert_eq!(vec_c.get(&1), f.elem(&6u8));
  }

  #[test]
  fn test_mutiply_elems_fully_matching_2_elems() {
    let f = &Field::new(&3911u16);
    let mut vec_a = SparseVec::new(3);
    let mut vec_b = SparseVec::new(3);

    vec_a.set(1, f.elem(&2u8));
    vec_a.set(2, f.elem(&3u8));
    vec_b.set(1, f.elem(&4u8));
    vec_b.set(2, f.elem(&5u8));

    let vec_c = &vec_a * &vec_b;

    assert_eq!(vec_c.elems.len(), 2);
    assert_eq!(vec_c.get(&1), f.elem(&8u8));
    assert_eq!(vec_c.get(&2), f.elem(&15u8));
  }

  #[test]
  fn test_mutiply_elems_partially_matching() {
    let f = &Field::new(&3911u16);
    let mut vec_a = SparseVec::new(3);
    let mut vec_b = SparseVec::new(3);

    vec_a.set(1, f.elem(&2u8));
    vec_a.set(2, f.elem(&5u8));
    vec_b.set(1, f.elem(&3u8));

    let vec_c = &vec_a * &vec_b;

    assert_eq!(vec_c.elems.len(), 1);
    assert_eq!(vec_c.get(&1), f.elem(&6u8));
  }

  #[test]
  fn test_sum() {
    let mut vec = SparseVec::new(3);
    let f = &Field::new(&3911u16);

    vec.set(1, f.elem(&2u8));
    vec.set(2, f.elem(&4u8));

    let sum = vec.sum();
    assert_eq!(sum, f.elem(&6u8));
  }

  #[test]
  fn test_eq_different_sizes() {
    let vec_a = SparseVec::new(3);
    let vec_b = SparseVec::new(4);
    assert_ne!(vec_a, vec_b);
  }

  #[test]
  fn test_eq_empty() {
    let vec_a = SparseVec::new(3);
    let vec_b = SparseVec::new(3);
    assert_eq!(vec_a, vec_b);
  }

  #[test]
  fn test_eq_non_empty() {
    let f = &Field::new(&3911u16);
    let mut vec_a = SparseVec::new(3);
    let mut vec_b = SparseVec::new(3);

    vec_a.set(1, f.elem(&92u8));
    vec_b.set(1, f.elem(&92u8));
    assert_eq!(vec_a, vec_b);
  }

  #[test]
  fn test_not_eq_non_empty() {
    let f = &Field::new(&3911u16);
    let mut vec_a = SparseVec::new(3);
    let mut vec_b = SparseVec::new(3);

    vec_a.set(1, f.elem(&92u8));
    vec_b.set(1, f.elem(&13u8));
    assert_ne!(vec_a, vec_b);
  }
}
