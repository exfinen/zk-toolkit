use std::{
  collections::HashMap,
  convert::From,
  ops::Mul,
};
use crate::building_block::curves::mcl::mcl_fr::MclFr;
use num_traits::Zero;
use core::ops::{Index, IndexMut};

#[derive(Clone)]
pub struct MclSparseVec {
  pub size: MclFr,
  zero: MclFr,
  elems: HashMap<MclFr, MclFr>, // HashMap<index, value>
}

impl std::fmt::Debug for MclSparseVec {
  fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
    let s = self.pretty_print();
    write!(fmt, "{}", s)
  }
}

pub struct MclSparseVecIterator<'a> {
  sv: &'a MclSparseVec,
  i: MclFr,
}

impl<'a> Iterator for MclSparseVecIterator<'a> {
  type Item = MclFr;

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

impl MclSparseVec {
  pub fn new(size: &MclFr) -> Self {
    if size.is_zero() {
      panic!("Size must be greater than 0");
    }
    MclSparseVec {
      zero: MclFr::zero(),
      size: size.clone(),
      elems: HashMap::<MclFr, MclFr>::new(),
    }
  }

  pub fn iter(&self) -> MclSparseVecIterator {
    MclSparseVecIterator { sv: self, i: MclFr::zero() }
  }

  pub fn set(&mut self, index: &MclFr, n: &MclFr) {
    if index >= &self.size {
      panic!("Index {:?} is out of range. The size of vector is {:?}", index, self.size);
    }
    if !n.is_zero() {
      self.elems.insert(index.clone(), n.clone());
    }
  }

  pub fn get(&self, index: &MclFr) -> &MclFr {
    if index >= &self.size {
      panic!("Index {:?} is out of range. The size of vector is {:?}", index, self.size);
    }
    if self.elems.contains_key(index) {
      self.elems.get(index).unwrap()
    } else {
      &self.zero
    }
  }

  pub fn indices(&self) -> Vec<MclFr> {
    let mut vec = vec![];
    for x in self.elems.keys() {
      vec.push(x.clone());
    }
    vec
  }

  // TODO clean up
  pub fn sum(&self) -> MclFr {
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
      if !value.is_zero() {
        return false;
      }
    }
    true
  }

  pub fn pretty_print(&self) -> String {
    let mut s = "[".to_string();
    let mut i = MclFr::zero();
    let one = &MclFr::from(1);

    while &i < &self.size {
      s += &format!("{:?}", self.get(&i));
      if &i < &(&self.size - one) {
        s += ",";
      }
      i.inc();
    }
    s += "]";
    s
  }

  // returns a vector of range [from..to)
  pub fn slice(&self, from: &MclFr, to: &MclFr) -> Self {
    let size = to - from;
    let mut new_sv = MclSparseVec::new(&size);

    let mut i = from.clone();
    while &i < &to {
      new_sv.set(&(&i - from), &self[&i]);
      i.inc();    
    } 
    new_sv
  }

  pub fn concat(&self, other: &MclSparseVec) -> MclSparseVec {
    let size = &self.size + &other.size;
    let mut sv = MclSparseVec::new(&size);

    let mut i = MclFr::zero(); 
    // copy self to new sv
    {
      let mut j = MclFr::zero(); 
      while &j < &self.size {
        sv[&i] = self[&j].clone();
        j.inc();
        i.inc();
      }
    }
    // copy other to new sv
    {
      let mut j = MclFr::zero();
      while &j < &other.size {
        sv[&i] = other[&j].clone();
        j.inc();
        i.inc();
      }
    }
    sv
  }
}

impl PartialEq for MclSparseVec {
  fn eq(&self, other: &MclSparseVec) -> bool {
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

impl Index<&MclFr> for MclSparseVec {
  type Output = MclFr;

  fn index(&self, index: &MclFr) -> &Self::Output {
    &self.get(index)
  }
}

impl IndexMut<&MclFr> for MclSparseVec {
  fn index_mut(&mut self, index: &MclFr) -> &mut Self::Output {
    if !self.elems.contains_key(index) {
      self.elems.insert(index.clone(), MclFr::from(0));
    }
    self.elems.get_mut(index).unwrap()
  }
}

impl From<&Vec<MclFr>> for MclSparseVec {
  fn from(elems: &Vec<MclFr>) -> Self {
    assert!(elems.len() != 0, "Cannot build vector from empty element list");
    let size = MclFr::from(elems.len());
    let mut vec = MclSparseVec::new(&size);

    for (i, v) in elems.iter().enumerate() {
      if !v.is_zero() {
        vec.set(&MclFr::from(i), v);
      }
    }
    vec
  }
}

// returns Hadamard product
impl Mul<&MclSparseVec> for &MclSparseVec {
    type Output = MclSparseVec;

    fn mul(self, rhs: &MclSparseVec) -> Self::Output {
      if self.size != rhs.size {
        panic!("Expected size of rhs to be {:?}, but got {:?}", self.size, rhs.size);
      }

      let mut ret = MclSparseVec::new(&self.size);
      for index in self.elems.keys() {
        let l = self.get(index);
        let r = rhs.get(index);
        if !l.is_zero() && !r.is_zero() {
          ret.set(index, &(l * r));
        }
      }
      ret
    }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::building_block::curves::mcl::mcl_initializer::MclInitializer;

  #[test]
  #[should_panic]
  fn test_from_empty_list() {
    MclInitializer::init();
    std::panic::set_hook(Box::new(|_| {}));  // suppress stack trace
    let _ = MclSparseVec::from(&vec![]);
  }

  #[test]
  fn test_slice() {
    MclInitializer::init();
    let zero = &MclFr::zero();
    let one = &MclFr::from(1);
    let two = &MclFr::from(2);
    let three = &MclFr::from(3);
    let elems = vec![
      zero.clone(),
      one.clone(),
      two.clone(),
      three.clone(),
    ];
    let sv = MclSparseVec::from(&elems);
    let sv2 = sv.slice(one, three);

    assert_eq!(&sv2.size, two);
    assert_eq!(&sv2[zero], one);
    assert_eq!(&sv2[one], two);
  }

  #[test]
  fn test_from_non_empty_list() {
    MclInitializer::init();
    let zero = &MclFr::zero();
    let one = &MclFr::from(1);
    let two = &MclFr::from(2);
    let elems = vec![one.clone(), two.clone()];
    let vec = MclSparseVec::from(&elems);
    assert_eq!(&vec.size, two);
    assert_eq!(&vec[zero], one);
    assert_eq!(&vec[one], two);
  }

  #[test]
  fn test_from_non_empty_zero_only_list() {
    MclInitializer::init();
    let zero = &MclFr::zero();
    let two = &MclFr::from(2);
    let elems = vec![zero.clone(), zero.clone()];
    let vec = MclSparseVec::from(&elems);
    assert_eq!(&vec.size, two);
    assert_eq!(vec.elems.len(), 0);
  }

  #[test]
  #[should_panic]
  fn test_new_empty_vec() {
    MclInitializer::init();
    std::panic::set_hook(Box::new(|_| {}));  // suppress stack trace
    MclSparseVec::new(&MclFr::zero());
  }

  #[test]
  #[should_panic]
  fn test_bad_set() {
    MclInitializer::init();
    std::panic::set_hook(Box::new(|_| {}));  // suppress stack trace

    let mut vec = MclSparseVec::new(&MclFr::from(3));
    assert_eq!(vec.elems.len(), 0);

    vec.set(&MclFr::from(3), &MclFr::from(2));
  }

  #[test]
  fn test_good_set() {
    MclInitializer::init();
    let mut vec = MclSparseVec::new(&MclFr::from(3));
    assert_eq!(vec.elems.len(), 0);

    let two = &MclFr::from(2);
    vec.set(&MclFr::from(2), two);
    assert_eq!(vec.elems.len(), 1);
    assert_eq!(vec.elems.get(two).unwrap(), &MclFr::from(2));

    // setting the same index should overwrite
    vec.set(&MclFr::from(2), &MclFr::from(3));
    assert_eq!(vec.elems.len(), 1);
    assert_eq!(vec.elems.get(two).unwrap(), &MclFr::from(3));

    // setting 0 should do nothing
    vec.set(&MclFr::from(2), &MclFr::zero());
    assert_eq!(vec.elems.len(), 1);
    assert_eq!(vec.elems.get(two).unwrap(), &MclFr::from(3));
  }

  #[test]
  fn test_assign() {
    MclInitializer::init();
    let mut vec = MclSparseVec::new(&MclFr::from(3));
    assert_eq!(vec.elems.len(), 0);

    vec.set(&MclFr::from(2), &MclFr::from(2));
    assert_eq!(vec.elems.len(), 1);
    assert_eq!(vec.elems.get(&MclFr::from(2)).unwrap(), &MclFr::from(2));

    let indices = vec.indices();
    assert_eq!(indices.len(), 1);
    assert_eq!(indices[0], MclFr::from(2));

    // setting the same index should overwrite
    vec.set(&MclFr::from(2), &MclFr::from(3));
    assert_eq!(vec.elems.len(), 1);
    assert_eq!(vec.elems.get(&MclFr::from(2)).unwrap(), &MclFr::from(3));
  }

  #[test]
  fn test_good_get() {
    MclInitializer::init();
    let zero = &MclFr::zero();
    let one = &MclFr::from(1);
    let two = &MclFr::from(2);
    let three = &MclFr::from(3);
    let mut vec = MclSparseVec::new(&MclFr::from(3));
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
    MclInitializer::init();
    std::panic::set_hook(Box::new(|_| {}));  // suppress stack trace

    let vec = MclSparseVec::new(&MclFr::from(1));
    vec.get(&MclFr::from(2));
  }

  #[test]
  fn test_get_index_without_value() {
    MclInitializer::init();
    std::panic::set_hook(Box::new(|_| {}));  // suppress stack trace

    let zero = &MclFr::zero();
    let vec = MclSparseVec::new(&MclFr::from(3));
    assert_eq!(vec.get(&MclFr::from(0)), zero);
    assert_eq!(vec.get(&MclFr::from(1)), zero);
    assert_eq!(vec.get(&MclFr::from(2)), zero);
  }

  #[test]
  fn test_indices() {
    MclInitializer::init();
    let mut vec = MclSparseVec::new(&MclFr::from(3));

    vec.set(&MclFr::from(1), &MclFr::from(2));
    vec.set(&MclFr::from(2), &MclFr::from(4));

    let indices = vec.indices();

    assert_eq!(indices.len(), 2);
    assert!(indices.contains(&MclFr::from(1)));
    assert!(indices.contains(&MclFr::from(2)));
  }

  #[test]
  fn test_mutiply_no_matching_elems() {
    MclInitializer::init();
    let mut vec_a = MclSparseVec::new(&MclFr::from(3));
    let mut vec_b = MclSparseVec::new(&MclFr::from(3));

    vec_a.set(&MclFr::from(1), &MclFr::from(2));
    vec_b.set(&MclFr::from(2), &MclFr::from(3));

    let vec_c = &vec_a * &vec_b;

    assert_eq!(vec_c.elems.len(), 0);
  }

  #[test]
  fn test_mutiply_elems_fully_matching_1_elem() {
    MclInitializer::init();
    let mut vec_a = MclSparseVec::new(&MclFr::from(3));
    let mut vec_b = MclSparseVec::new(&MclFr::from(3));

    vec_a.set(&MclFr::from(1), &MclFr::from(2));
    vec_b.set(&MclFr::from(1), &MclFr::from(3));

    let vec_c = &vec_a * &vec_b;

    assert_eq!(vec_c.elems.len(), 1);
    assert_eq!(vec_c.get(&MclFr::from(1)), &MclFr::from(6));
  }

  #[test]
  fn test_mutiply_elems_fully_matching_2_elems() {
    MclInitializer::init();
    let mut vec_a = MclSparseVec::new(&MclFr::from(3));
    let mut vec_b = MclSparseVec::new(&MclFr::from(3));

    vec_a.set(&MclFr::from(1), &MclFr::from(2));
    vec_a.set(&MclFr::from(2), &MclFr::from(3));
    vec_b.set(&MclFr::from(1), &MclFr::from(4));
    vec_b.set(&MclFr::from(2), &MclFr::from(5));

    let vec_c = &vec_a * &vec_b;

    assert_eq!(vec_c.elems.len(), 2);
    assert_eq!(vec_c.get(&MclFr::from(1)), &MclFr::from(8));
    assert_eq!(vec_c.get(&MclFr::from(2)), &MclFr::from(15));
  }

  #[test]
  fn test_mutiply_elems_partially_matching() {
    MclInitializer::init();
    let mut vec_a = MclSparseVec::new(&MclFr::from(3));
    let mut vec_b = MclSparseVec::new(&MclFr::from(3));

    vec_a.set(&MclFr::from(1), &MclFr::from(2));
    vec_a.set(&MclFr::from(2), &MclFr::from(5));
    vec_b.set(&MclFr::from(1), &MclFr::from(3));

    let vec_c = &vec_a * &vec_b;

    assert_eq!(vec_c.elems.len(), 1);
    assert_eq!(vec_c.get(&MclFr::from(1)), &MclFr::from(6));
  }

  #[test]
  fn test_sum() {
    MclInitializer::init();
    let mut vec = MclSparseVec::new(&MclFr::from(3));

    vec.set(&MclFr::from(1), &MclFr::from(2));
    vec.set(&MclFr::from(2), &MclFr::from(4));

    let sum = vec.sum();
    assert_eq!(sum, MclFr::from(6));
  }

  #[test]
  fn test_eq_different_sizes() {
    MclInitializer::init();
    let vec_a = MclSparseVec::new(&MclFr::from(3));
    let vec_b = MclSparseVec::new(&MclFr::from(4));
    assert_ne!(vec_a, vec_b);
    assert_ne!(vec_b, vec_a);
  }

  #[test]
  fn test_eq_empty() {
    MclInitializer::init();
    let vec_a = MclSparseVec::new(&MclFr::from(3));
    let vec_b = MclSparseVec::new(&MclFr::from(3));
    assert_eq!(vec_a, vec_b);
    assert_eq!(vec_b, vec_a);
  }

  #[test]
  fn test_eq_non_empty() {
    MclInitializer::init();
    let mut vec_a = MclSparseVec::new(&MclFr::from(3));
    let mut vec_b = MclSparseVec::new(&MclFr::from(3));

    vec_a.set(&MclFr::from(1), &MclFr::from(92));
    vec_b.set(&MclFr::from(1), &MclFr::from(92));
    assert_eq!(vec_a, vec_b);
    assert_eq!(vec_b, vec_a);
  }

  #[test]
  fn test_not_eq_non_empty() {
    MclInitializer::init();
    let mut vec_a = MclSparseVec::new(&MclFr::from(3));
    let mut vec_b = MclSparseVec::new(&MclFr::from(3));

    vec_a.set(&MclFr::from(1), &MclFr::from(92));
    vec_b.set(&MclFr::from(1), &MclFr::from(13));
    assert_ne!(vec_a, vec_b);
    assert_ne!(vec_b, vec_a);
  }

  #[test]
  fn test_iterator() {
    MclInitializer::init();
    let mut sv = MclSparseVec::new(&MclFr::from(3));
    sv.set(&MclFr::from(0), &MclFr::from(1));
    sv.set(&MclFr::from(1), &MclFr::from(2));
    sv.set(&MclFr::from(2), &MclFr::from(3));

    let it = &mut sv.iter();
    assert!(&it.next().unwrap() == &MclFr::from(1));
    assert!(&it.next().unwrap() == &MclFr::from(2));
    assert!(&it.next().unwrap() == &MclFr::from(3));
    assert!(it.next() == None);
  }

  #[test]
  fn test_concat() {
    MclInitializer::init();
    let mut sv1 = MclSparseVec::new(&MclFr::from(2));
    sv1.set(&MclFr::from(0), &MclFr::from(1));
    sv1.set(&MclFr::from(1), &MclFr::from(2));

    let mut sv2 = MclSparseVec::new(&MclFr::from(2));
    sv2.set(&MclFr::from(0), &MclFr::from(3));
    sv2.set(&MclFr::from(1), &MclFr::from(4));

    let sv3 = sv1.concat(&sv2);
    assert!(sv3.get(&MclFr::from(0)) == &MclFr::from(1));
    assert!(sv3.get(&MclFr::from(1)) == &MclFr::from(2));
    assert!(sv3.get(&MclFr::from(2)) == &MclFr::from(3));
    assert!(sv3.get(&MclFr::from(3)) == &MclFr::from(4));
  }
}
