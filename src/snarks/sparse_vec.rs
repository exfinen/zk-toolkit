use std::{
  collections::HashMap,
  convert::From,
  ops::Mul,
};
use crate::building_block::field::{Field, FieldElem};
use num_traits::Zero;
use core::ops::{Index, IndexMut};
use crate::building_block::to_biguint::ToBigUint;

#[derive(Clone)]
pub struct SparseVec {
  pub f: Field,
  pub size: FieldElem,
  zero: FieldElem,
  elems: HashMap<FieldElem, FieldElem>,  // HashMap<index, value>
}

impl std::fmt::Debug for SparseVec {
  fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
    let mut keys: Vec<FieldElem> = self.elems.keys().cloned().collect();
    keys.sort();
    let xs: Vec<String> = keys.iter().map(|k| { format!("{:?}->{:?}", k.n, self.elems.get(k).unwrap().n) }).collect();
    write!(fmt, "[{}]", xs.join(","))
  }
}

impl SparseVec {
  pub fn new(f: &Field, size: &impl ToBigUint) -> Self {
    let size = f.elem(size);
    if size == f.elem(&0u8) {
      panic!("Size must be greater than 0");
    }
    SparseVec {
      f: f.clone(),
      zero: f.elem(&0u8),
      size,
      elems: HashMap::<FieldElem, FieldElem>::new(),
    }
  }

  pub fn set(&mut self, index: &impl ToBigUint, n: &impl ToBigUint) {
    let index = &self.f.elem(index);
    if index >= &self.size {
      panic!("Index {:?} is out of range. The size of vector is {:?}", index.n, self.size.n);
    }
    let n = self.f.elem(n);
    if !n.is_zero() {
      self.elems.insert(index.clone(), n);
    }
  }

  pub fn get(&self, index: &impl ToBigUint) -> &FieldElem {
    let index = &self.f.elem(index);
    if index >= &self.size {
      panic!("Index {:?} is out of range. The size of vector is {:?}", index.n, self.size.n);
    }
    if self.elems.contains_key(index) {
      self.elems.get(index).unwrap()
    } else {
      &self.zero
    }
  }

  pub fn indices(&self) -> Vec<FieldElem> {
    let mut vec = vec![];
    for x in self.elems.keys() {
      vec.push(x.clone());
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

  pub fn pretty_print(&self) -> String {
    let one = self.f.elem(&1u8);
    let mut s = "[".to_string();
    let mut i = self.f.elem(&0u8);

    while i < self.size {
      s += &format!("{:?}", self.get(&i).n);
      if i < &self.size - &one {
        s += ",";
      }
      i = &i + &one;
    }
    s += "]";
    s
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
    for index in other.elems.keys() {
      let other_elem = other.get(index);
      let this_elem = self.get(index);
      if this_elem != other_elem { return false; }
    }
    true
  }
}

impl Index<&FieldElem> for SparseVec {
  type Output = FieldElem;

  fn index(&self, index: &FieldElem) -> &Self::Output {
    &self.get(index)
  }
}

impl IndexMut<&FieldElem> for SparseVec {
  fn index_mut(&mut self, index: &FieldElem) -> &mut Self::Output {
    if !self.elems.contains_key(index) {
      self.elems.insert(index.clone(), self.f.elem(&0u8));
    }
    self.elems.get_mut(index).unwrap()
  }
}

impl From<&Vec<FieldElem>> for SparseVec {
  fn from(elems: &Vec<FieldElem>) -> Self {
    let size = &elems.len();
    assert!(elems.len() != 0, "Cannot build vector from empty element list");
    let f = &elems[0].f;
    let mut vec = SparseVec::new(f, size);

    for (i, v) in elems.iter().enumerate() {
      if !v.n.is_zero() {
        vec.set(&i, v);
      }
    }
    vec
  }
}

// returns Hadamard product
impl Mul<&SparseVec> for &SparseVec {
    type Output = SparseVec;

    fn mul(self, rhs: &SparseVec) -> Self::Output {
      if self.size != rhs.size {
        panic!("Expected size of rhs to be {:?}, but got {:?}", self.size.n, rhs.size.n);
      }

      let mut ret = SparseVec::new(&self.f, &self.size);
      for index in self.elems.keys() {
        let l = self.get(index);
        let r = rhs.get(index);
        if !l.n.is_zero() && !r.n.is_zero() {
          ret.set(index, &(l * r));
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
  fn test_from_empty_list() {
    std::panic::set_hook(Box::new(|_| {}));  // suppress stack trace
    let _ = SparseVec::from(&vec![]);
  }

  #[test]
  fn test_from_non_empty_list() {
    let f = &Field::new(&3911u16);
    let zero = &f.elem(&0u8);
    let one = &f.elem(&1u8);
    let two = &f.elem(&2u8);
    let elems = vec![one.clone(), two.clone()];
    let vec = SparseVec::from(&elems);
    assert_eq!(&vec.size, two);
    assert_eq!(&vec[&zero], one);
    assert_eq!(&vec[&one], two);
  }

  #[test]
  fn test_from_non_empty_zero_only_list() {
    let f = &Field::new(&3911u16);
    let zero = &f.elem(&0u8);
    let two = &f.elem(&2u8);
    let elems = vec![zero.clone(), zero.clone()];
    let vec = SparseVec::from(&elems);
    assert_eq!(&vec.size, two);
    assert_eq!(vec.elems.len(), 0);
  }

  #[test]
  #[should_panic]
  fn test_new_empty_vec() {
    std::panic::set_hook(Box::new(|_| {}));  // suppress stack trace
    let f = &Field::new(&3911u16);
    SparseVec::new(f, &0u8);
  }

  #[test]
  #[should_panic]
  fn test_bad_set() {
    std::panic::set_hook(Box::new(|_| {}));  // suppress stack trace

    let f = &Field::new(&3911u16);
    let mut vec = SparseVec::new(f, &3u8);
    assert_eq!(vec.elems.len(), 0);

    let f = &Field::new(&3911u16);
    vec.set(&3u8, &f.elem(&2u8));
  }

  #[test]
  fn test_good_set() {
    let f = &Field::new(&3911u16);
    let mut vec = SparseVec::new(f, &3u8);
    assert_eq!(vec.elems.len(), 0);

    let f = &Field::new(&3911u16);
    let two = &f.elem(&2u8);
    vec.set(&2u8, two);
    assert_eq!(vec.elems.len(), 1);
    assert_eq!(vec.elems.get(two).unwrap(), &f.elem(&2u8));

    // setting the same index should overwrite
    vec.set(&2u8, &f.elem(&3u8));
    assert_eq!(vec.elems.len(), 1);
    assert_eq!(vec.elems.get(two).unwrap(), &f.elem(&3u8));

    // setting 0 should do nothing
    vec.set(&2u8, &f.elem(&0u8));
    assert_eq!(vec.elems.len(), 1);
    assert_eq!(vec.elems.get(two).unwrap(), &f.elem(&3u8));
  }

  #[test]
  fn test_assign() {
    let f = &Field::new(&3911u16);
    let mut vec = SparseVec::new(f, &3u8);
    assert_eq!(vec.elems.len(), 0);

    let f = &Field::new(&3911u16);
    vec.set(&2u8, &f.elem(&2u8));
    assert_eq!(vec.elems.len(), 1);
    assert_eq!(vec.elems.get(&f.elem(&2u8)).unwrap(), &f.elem(&2u8));

    let indices = vec.indices();
    assert_eq!(indices.len(), 1);
    assert_eq!(indices[0], f.elem(&2u8));

    // setting the same index should overwrite
    vec.set(&2u8, &3u8);
    assert_eq!(vec.elems.len(), 1);
    assert_eq!(vec.elems.get(&f.elem(&2u8)).unwrap(), &f.elem(&3u8));
  }

  #[test]
  fn test_good_get() {
    let f = &Field::new(&3911u16);
    let zero = &f.elem(&0u8);
    let one = &f.elem(&1u8);
    let two = &f.elem(&2u8);
    let three = &f.elem(&3u8);
    let mut vec = SparseVec::new(f, &3u8);
    vec.set(zero, one);
    vec.set(one, two);
    vec.set(two, three);
    assert_eq!(vec.get(zero), one);
    assert_eq!(vec.get(one), two);
    assert_eq!(vec.get(two), three);
  }
  #[test]
  #[should_panic]
  fn test_get_index_out_of_range() {
    std::panic::set_hook(Box::new(|_| {}));  // suppress stack trace

    let f = &Field::new(&3911u16);
    let vec = SparseVec::new(f, &1u8);
    vec.get(&2u8);
  }

  #[test]
  fn test_get_index_without_value() {
    std::panic::set_hook(Box::new(|_| {}));  // suppress stack trace

    let f = &Field::new(&3911u16);
    let zero = &f.elem(&0u8);
    let vec = SparseVec::new(f, &3u8);
    assert_eq!(vec.get(&0u8), zero);
    assert_eq!(vec.get(&1u8), zero);
    assert_eq!(vec.get(&2u8), zero);
  }

  #[test]
  fn test_indices() {
    let f = &Field::new(&3911u16);
    let mut vec = SparseVec::new(f, &3u8);

    vec.set(&1u8, &2u8);
    vec.set(&2u8, &4u8);

    let indices = vec.indices();

    assert_eq!(indices.len(), 2);
    assert!(indices.contains(&f.elem(&1u8)));
    assert!(indices.contains(&f.elem(&2u8)));
  }

  #[test]
  fn test_mutiply_no_matching_elems() {
    let f = &Field::new(&3911u16);
    let mut vec_a = SparseVec::new(f, &3u8);
    let mut vec_b = SparseVec::new(f, &3u8);

    vec_a.set(&1u8, &2u8);
    vec_b.set(&2u8, &3u8);

    let vec_c = &vec_a * &vec_b;

    assert_eq!(vec_c.elems.len(), 0);
  }

  #[test]
  fn test_mutiply_elems_fully_matching_1_elem() {
    let f = &Field::new(&3911u16);
    let mut vec_a = SparseVec::new(f, &3u8);
    let mut vec_b = SparseVec::new(f, &3u8);

    vec_a.set(&1u8, &2u8);
    vec_b.set(&1u8, &3u8);

    let vec_c = &vec_a * &vec_b;

    assert_eq!(vec_c.elems.len(), 1);
    assert_eq!(vec_c.get(&1u8), &f.elem(&6u8));
  }

  #[test]
  fn test_mutiply_elems_fully_matching_2_elems() {
    let f = &Field::new(&3911u16);
    let mut vec_a = SparseVec::new(f, &3u8);
    let mut vec_b = SparseVec::new(f, &3u8);

    vec_a.set(&1u8, &2u8);
    vec_a.set(&2u8, &3u8);
    vec_b.set(&1u8, &4u8);
    vec_b.set(&2u8, &5u8);

    let vec_c = &vec_a * &vec_b;

    assert_eq!(vec_c.elems.len(), 2);
    assert_eq!(vec_c.get(&1u8), &f.elem(&8u8));
    assert_eq!(vec_c.get(&2u8), &f.elem(&15u8));
  }

  #[test]
  fn test_mutiply_elems_partially_matching() {
    let f = &Field::new(&3911u16);
    let mut vec_a = SparseVec::new(f, &3u8);
    let mut vec_b = SparseVec::new(f, &3u8);

    vec_a.set(&1u8, &2u8);
    vec_a.set(&2u8, &5u8);
    vec_b.set(&1u8, &3u8);

    let vec_c = &vec_a * &vec_b;

    assert_eq!(vec_c.elems.len(), 1);
    assert_eq!(vec_c.get(&1u8), &f.elem(&6u8));
  }

  #[test]
  fn test_sum() {
    let f = &Field::new(&3911u16);
    let mut vec = SparseVec::new(f, &3u8);

    vec.set(&1u8, &2u8);
    vec.set(&2u8, &4u8);

    let sum = vec.sum();
    assert_eq!(sum, f.elem(&6u8));
  }

  #[test]
  fn test_eq_different_sizes() {
    let f = &Field::new(&3911u16);
    let vec_a = SparseVec::new(f, &3u8);
    let vec_b = SparseVec::new(f, &4u8);
    assert_ne!(vec_a, vec_b);
    assert_ne!(vec_b, vec_a);
  }

  #[test]
  fn test_eq_empty() {
    let f = &Field::new(&3911u16);
    let vec_a = SparseVec::new(f, &3u8);
    let vec_b = SparseVec::new(f, &3u8);
    assert_eq!(vec_a, vec_b);
    assert_eq!(vec_b, vec_a);
  }

  #[test]
  fn test_eq_non_empty() {
    let f = &Field::new(&3911u16);
    let mut vec_a = SparseVec::new(f, &3u8);
    let mut vec_b = SparseVec::new(f, &3u8);

    vec_a.set(&1u8, &92u8);
    vec_b.set(&1u8, &92u8);
    assert_eq!(vec_a, vec_b);
    assert_eq!(vec_b, vec_a);
  }

  #[test]
  fn test_not_eq_non_empty() {
    let f = &Field::new(&3911u16);
    let mut vec_a = SparseVec::new(f, &3u8);
    let mut vec_b = SparseVec::new(f, &3u8);

    vec_a.set(&1u8, &92u8);
    vec_b.set(&1u8, &13u8);
    assert_ne!(vec_a, vec_b);
    assert_ne!(vec_b, vec_a);
  }
}
