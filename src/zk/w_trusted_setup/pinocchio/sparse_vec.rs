use std::{
  collections::HashMap,
  convert::From,
  ops::Mul,
};
use crate::building_block::field::{
  prime_field::PrimeField,
  prime_field_elem::PrimeFieldElem,
};
use num_traits::Zero;
use core::ops::{Index, IndexMut};
use crate::building_block::to_biguint::ToBigUint;

#[derive(Clone)]
pub struct SparseVec {
  pub f: PrimeField,
  pub size: PrimeFieldElem,
  zero: PrimeFieldElem,
  elems: HashMap<PrimeFieldElem, PrimeFieldElem>,  // HashMap<index, value>
}

impl std::fmt::Debug for SparseVec {
  fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
    // let mut keys: Vec<FieldElem> = self.elems.keys().cloned().collect();
    // keys.sort();
    // let xs: Vec<String> = keys.iter().map(|k| { format!("{:?}->{:?}", k.n, self.elems.get(k).unwrap().n) }).collect();
    let s = self.pretty_print();
    //write!(fmt, "[{}]", xs.join(","))
    write!(fmt, "{}", s)
  }
}

pub struct SparseVecIterator<'a> {
  sv: &'a SparseVec,
  i: PrimeFieldElem,
}

impl<'a> Iterator for SparseVecIterator<'a> {
  type Item = PrimeFieldElem;

  fn next(&mut self) -> Option<Self::Item> {
    if &self.sv.size == &self.i {
      None
    } else {
      let elem = self.sv[&self.i].clone();
      self.i.inc();
      Some(elem)
    }
  }
}

impl SparseVec {
  pub fn new(f: &PrimeField, size: &impl ToBigUint) -> Self {
    let size = f.elem(size);
    if size == f.elem(&0u8) {
      panic!("Size must be greater than 0");
    }
    SparseVec {
      f: f.clone(),
      zero: f.elem(&0u8),
      size,
      elems: HashMap::<PrimeFieldElem, PrimeFieldElem>::new(),
    }
  }

  pub fn iter(&self) -> SparseVecIterator {
    SparseVecIterator { sv: self, i: self.f.elem(&0u8) }
  }

  pub fn set(&mut self, index: &impl ToBigUint, n: &impl ToBigUint) {
    let index = &self.f.elem(index);
    if index >= &self.size {
      panic!("Index {:?} is out of range. The size of vector is {:?}", index.e, self.size.e);
    }
    let n = self.f.elem(n);
    if !n.is_zero() {
      self.elems.insert(index.clone(), n);
    }
  }

  pub fn get(&self, index: &impl ToBigUint) -> &PrimeFieldElem {
    let index = &self.f.elem(index);
    if index >= &self.size {
      panic!("Index {:?} is out of range. The size of vector is {:?}", index.e, self.size.e);
    }
    if self.elems.contains_key(index) {
      self.elems.get(index).unwrap()
    } else {
      &self.zero
    }
  }

  pub fn indices(&self) -> Vec<PrimeFieldElem> {
    let mut vec = vec![];
    for x in self.elems.keys() {
      vec.push(x.clone());
    }
    vec
  }

  // TODO clean up
  pub fn sum(&self) -> PrimeFieldElem {
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

  // empty if containing only zeros
  pub fn is_empty(&self) -> bool {
    for value in self.elems.values() {
      if !value.e.is_zero() {
        return false;
      }
    }
    true
  }

  pub fn pretty_print(&self) -> String {
    let one = self.f.elem(&1u8);
    let mut s = "[".to_string();
    let mut i = self.f.elem(&0u8);

    while i < self.size {
      s += &format!("{:?}", self.get(&i).e);
      if i < &self.size - &one {
        s += ",";
      }
      i = &i + &one;
    }
    s += "]";
    s
  }

  // returns a vector of range [from..to)
  pub fn slice(&self, from: &PrimeFieldElem, to: &PrimeFieldElem) -> Self {
    let size = &self.f.elem(&(to - from));
    let mut new_sv = SparseVec::new(&self.f, size);

    let mut i = from.clone();
    while &i < to {
      new_sv.set(&(&i - from), &self[&i]);
      i.inc();    
    } 
    new_sv
  }

  pub fn concat(&self, other: &SparseVec) -> SparseVec {
    let f = &self.f;
    let size = &self.size + &other.size;
    println!("size {:?}", &size);
    let mut sv = SparseVec::new(f, &size);

    let mut i = f.elem(&0u8); 
    // copy self to new sv
    {
      let mut j = f.elem(&0u8); 
      while &j < &self.size {
        sv[&i] = self[&j].clone();
        j.inc();
        i.inc();
      }
    }
    // copy other to new sv
    {
      let mut j = f.elem(&0u8); 
      while &j < &other.size {
        sv[&i] = other[&j].clone();
        j.inc();
        i.inc();
      }
    }
    sv
  }

  // this panics if the size is above usize maximum
  pub fn size_in_usize(&self) -> usize {
    (&self.size.e).try_into().unwrap()
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

impl Index<&PrimeFieldElem> for SparseVec {
  type Output = PrimeFieldElem;

  fn index(&self, index: &PrimeFieldElem) -> &Self::Output {
    &self.get(index)
  }
}

// this fails if index is above usize maximum
impl Index<&usize> for SparseVec {
  type Output = PrimeFieldElem;

  fn index(&self, index: &usize) -> &Self::Output {
    let index = self.size.f.elem(index);
    &self.get(&index)
  }
}

impl IndexMut<&PrimeFieldElem> for SparseVec {
  fn index_mut(&mut self, index: &PrimeFieldElem) -> &mut Self::Output {
    if !self.elems.contains_key(index) {
      self.elems.insert(index.clone(), self.f.elem(&0u8));
    }
    self.elems.get_mut(index).unwrap()
  }
}

// this fails if index is above usize maximum
impl IndexMut<&usize> for SparseVec {
  fn index_mut(&mut self, index: &usize) -> &mut Self::Output {
    let index = &self.size.f.elem(index);

    if !self.elems.contains_key(index) {
      self.elems.insert(index.clone(), self.f.elem(&0u8));
    }
    self.elems.get_mut(index).unwrap()
  }
}

impl From<&Vec<PrimeFieldElem>> for SparseVec {
  fn from(elems: &Vec<PrimeFieldElem>) -> Self {
    let size = &elems.len();
    assert!(elems.len() != 0, "Cannot build vector from empty element list");
    let f = &elems[0].f;
    let mut vec = SparseVec::new(f, size);

    for (i, v) in elems.iter().enumerate() {
      if !v.e.is_zero() {
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
        panic!("Expected size of rhs to be {:?}, but got {:?}", self.size.e, rhs.size.e);
      }

      let mut ret = SparseVec::new(&self.f, &self.size);
      for index in self.elems.keys() {
        let l = self.get(index);
        let r = rhs.get(index);
        if !l.e.is_zero() && !r.e.is_zero() {
          ret.set(index, &(l * r));
        }
      }
      ret
    }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::building_block::field::prime_field::PrimeField;

  #[test]
  #[should_panic]
  fn test_from_empty_list() {
    std::panic::set_hook(Box::new(|_| {}));  // suppress stack trace
    let _ = SparseVec::from(&vec![]);
  }

  #[test]
  fn test_slice() {
    let f = &PrimeField::new(&3911u16);
    let zero = &f.elem(&0u8);
    let one = &f.elem(&1u8);
    let two = &f.elem(&2u8);
    let three = &f.elem(&3u8);
    let elems = vec![
      zero.clone(),
      one.clone(),
      two.clone(),
      three.clone(),
    ];
    let sv = SparseVec::from(&elems);
    let sv2 = sv.slice(one, three);

    assert_eq!(&sv2.size, two);
    assert_eq!(&sv2[zero], one);
    assert_eq!(&sv2[one], two);
  }

  #[test]
  fn test_from_non_empty_list() {
    let f = &PrimeField::new(&3911u16);
    let zero = &f.elem(&0u8);
    let one = &f.elem(&1u8);
    let two = &f.elem(&2u8);
    let elems = vec![one.clone(), two.clone()];
    let vec = SparseVec::from(&elems);
    assert_eq!(&vec.size, two);
    assert_eq!(&vec[zero], one);
    assert_eq!(&vec[one], two);
  }

  #[test]
  fn test_from_non_empty_zero_only_list() {
    let f = &PrimeField::new(&3911u16);
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
    let f = &PrimeField::new(&3911u16);
    SparseVec::new(f, &0u8);
  }

  #[test]
  #[should_panic]
  fn test_bad_set() {
    std::panic::set_hook(Box::new(|_| {}));  // suppress stack trace

    let f = &PrimeField::new(&3911u16);
    let mut vec = SparseVec::new(f, &3u8);
    assert_eq!(vec.elems.len(), 0);

    let f = &PrimeField::new(&3911u16);
    vec.set(&3u8, &f.elem(&2u8));
  }

  #[test]
  fn test_good_set() {
    let f = &PrimeField::new(&3911u16);
    let mut vec = SparseVec::new(f, &3u8);
    assert_eq!(vec.elems.len(), 0);

    let f = &PrimeField::new(&3911u16);
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
    let f = &PrimeField::new(&3911u16);
    let mut vec = SparseVec::new(f, &3u8);
    assert_eq!(vec.elems.len(), 0);

    let f = &PrimeField::new(&3911u16);
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
    let f = &PrimeField::new(&3911u16);
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

    let f = &PrimeField::new(&3911u16);
    let vec = SparseVec::new(f, &1u8);
    vec.get(&2u8);
  }

  #[test]
  fn test_get_index_without_value() {
    std::panic::set_hook(Box::new(|_| {}));  // suppress stack trace

    let f = &PrimeField::new(&3911u16);
    let zero = &f.elem(&0u8);
    let vec = SparseVec::new(f, &3u8);
    assert_eq!(vec.get(&0u8), zero);
    assert_eq!(vec.get(&1u8), zero);
    assert_eq!(vec.get(&2u8), zero);
  }

  #[test]
  fn test_indices() {
    let f = &PrimeField::new(&3911u16);
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
    let f = &PrimeField::new(&3911u16);
    let mut vec_a = SparseVec::new(f, &3u8);
    let mut vec_b = SparseVec::new(f, &3u8);

    vec_a.set(&1u8, &2u8);
    vec_b.set(&2u8, &3u8);

    let vec_c = &vec_a * &vec_b;

    assert_eq!(vec_c.elems.len(), 0);
  }

  #[test]
  fn test_mutiply_elems_fully_matching_1_elem() {
    let f = &PrimeField::new(&3911u16);
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
    let f = &PrimeField::new(&3911u16);
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
    let f = &PrimeField::new(&3911u16);
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
    let f = &PrimeField::new(&3911u16);
    let mut vec = SparseVec::new(f, &3u8);

    vec.set(&1u8, &2u8);
    vec.set(&2u8, &4u8);

    let sum = vec.sum();
    assert_eq!(sum, f.elem(&6u8));
  }

  #[test]
  fn test_eq_different_sizes() {
    let f = &PrimeField::new(&3911u16);
    let vec_a = SparseVec::new(f, &3u8);
    let vec_b = SparseVec::new(f, &4u8);
    assert_ne!(vec_a, vec_b);
    assert_ne!(vec_b, vec_a);
  }

  #[test]
  fn test_eq_empty() {
    let f = &PrimeField::new(&3911u16);
    let vec_a = SparseVec::new(f, &3u8);
    let vec_b = SparseVec::new(f, &3u8);
    assert_eq!(vec_a, vec_b);
    assert_eq!(vec_b, vec_a);
  }

  #[test]
  fn test_eq_non_empty() {
    let f = &PrimeField::new(&3911u16);
    let mut vec_a = SparseVec::new(f, &3u8);
    let mut vec_b = SparseVec::new(f, &3u8);

    vec_a.set(&1u8, &92u8);
    vec_b.set(&1u8, &92u8);
    assert_eq!(vec_a, vec_b);
    assert_eq!(vec_b, vec_a);
  }

  #[test]
  fn test_not_eq_non_empty() {
    let f = &PrimeField::new(&3911u16);
    let mut vec_a = SparseVec::new(f, &3u8);
    let mut vec_b = SparseVec::new(f, &3u8);

    vec_a.set(&1u8, &92u8);
    vec_b.set(&1u8, &13u8);
    assert_ne!(vec_a, vec_b);
    assert_ne!(vec_b, vec_a);
  }

  #[test]
  fn test_iterator() {
    let f = &PrimeField::new(&3911u16);
    let mut sv = SparseVec::new(f, &3u8);
    sv.set(&0u8, &f.elem(&1u8));
    sv.set(&1u8, &f.elem(&2u8));
    sv.set(&2u8, &f.elem(&3u8));

    let it = &mut sv.iter();
    assert!(it.next().unwrap() == f.elem(&1u8));
    assert!(it.next().unwrap() == f.elem(&2u8));
    assert!(it.next().unwrap() == f.elem(&3u8));
    assert!(it.next() == None);
  }

  #[test]
  fn test_concat() {
    let f = &PrimeField::new(&3911u16);
    let mut sv1 = SparseVec::new(f, &2u8);
    sv1.set(&0u8, &f.elem(&1u8));
    sv1.set(&1u8, &f.elem(&2u8));

    let mut sv2 = SparseVec::new(f, &2u8);
    sv2.set(&0u8, &f.elem(&3u8));
    sv2.set(&1u8, &f.elem(&4u8));

    let sv3 = sv1.concat(&sv2);
    assert!(sv3.get(&0u8) == &f.elem(&1u8));
    assert!(sv3.get(&1u8) == &f.elem(&2u8));
    assert!(sv3.get(&2u8) == &f.elem(&3u8));
    assert!(sv3.get(&3u8) == &f.elem(&4u8));
  }
}
