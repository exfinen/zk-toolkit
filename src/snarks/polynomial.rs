use crate::building_block::field::{Field, FieldElem};
use std::{
  fmt::{Debug, Formatter},
};
use num_traits::identities::Zero;

pub struct Polynomial {
  pub f: Field,
  pub coeffs: Vec<FieldElem>,
  _private: (),  // to force using new()
}

impl PartialEq<Polynomial> for Polynomial {
  fn eq(&self, rhs: &Polynomial) -> bool {
    let (smaller, larger) = if self.coeffs.len() < rhs.coeffs.len() {
      (&self.coeffs, &rhs.coeffs)
    } else {
      (&rhs.coeffs, &self.coeffs)
    };

    // if larger is superset, it contains other non-zero terms
    if smaller.len() != larger.len() { return false; }

    // check if smaller is a subset of larger
    for i in 0..smaller.len() {
      if &smaller[i] != &larger[i] { return false; }
    }
    true
  }
}

impl Debug for Polynomial {
  fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
    let last_idx = self.coeffs.len() - 1;
    let mut s = String::new();
    for (i, coeff) in self.coeffs.iter().rev().enumerate() {
       // if not the first element, write add op
      if i > 0 {
        s.push_str(" + ");
      }
      // write number
      s.push_str(&format!("{:?}", coeff.n));

      // if not the last one to write, write variable after number
      if i < last_idx {
        s.push_str("x");
        // write exponent if x^2 or higher
        if i < last_idx - 1 {  // second to last corresponds to x^1
          s.push_str(&format!("^{}", self.coeffs.len() - 1 - i));
        }
      }
    }
    write!(f, "{}", s)
  }
}

impl Polynomial {
  pub fn new(f: &Field, coeffs: Vec<FieldElem>) -> Self {
    if coeffs.len() == 0 { panic!("coeffs is empty"); }
    let x = Polynomial { f: f.clone(), coeffs, _private: () };
    x.normalize()
  }

  // trim trailing zero-coeff terms
  fn normalize(&self) -> Polynomial {
    let mut new_len = self.coeffs.len();
    for i in 0..(self.coeffs.len() - 1) {  // seek from end to beg and always keep the 0th element
      let coeff = &self.coeffs[&self.coeffs.len() - 1 - i];
      if !coeff.is_zero() { break; }
      new_len -= 1;
    }

    let mut norm_coeffs = vec![];
    for coeff in &self.coeffs[0..new_len] {
      norm_coeffs.push(coeff.clone());
    }
    Polynomial { f: self.f.clone(), coeffs: norm_coeffs, _private: () }
  }

  pub fn add(&self, rhs: &Polynomial) -> Polynomial {
    let (smaller, larger) = if self.coeffs.len() < rhs.coeffs.len() {
      (&self.coeffs, &rhs.coeffs)
    } else {
      (&rhs.coeffs, &self.coeffs)
    };

    let mut coeffs = vec![];
    for i in 0..larger.len() {
      if i < smaller.len() {
        coeffs.push(&smaller[i] + &larger[i]);
      } else {
        coeffs.push(larger[i].clone());
      }
    }
    let x = Polynomial { f: self.f.clone(), coeffs, _private: () };
    x.normalize()  // normalizing b/c addition can make term coefficect zero
  }

  pub fn mul(&self, rhs: &Polynomial) -> Polynomial {
    // degree of polynomial is coeffs.len - 1
    let self_degree = self.coeffs.len() - 1;
    let rhs_degree = rhs.coeffs.len() - 1;

    // coeffs len of the mul result is sum of self and rhs degrees + 1
    let new_len = self_degree + rhs_degree + 1;
    let mut coeffs = vec![self.f.elem(&0u8); new_len];

    for i in 0..self.coeffs.len() {
      for j in 0..rhs.coeffs.len() {
        let coeff = &self.coeffs[i] * &rhs.coeffs[j];
        let degree = i + j;
        coeffs[degree] = &coeffs[degree] + coeff;
      }
    }
    Polynomial { f: self.f.clone(), coeffs, _private: () }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::building_block::field::Field;

  #[test]
  fn test_debug_print() {
    let f = &Field::new(&3299u16);
    {
      let a = Polynomial::new(f, vec![
        f.elem(&12u8),
        f.elem(&45u8),
        f.elem(&67u8),
      ]);
      println!("{:?}", a);
    }
    {
      let a = Polynomial::new(f, vec![
        f.elem(&12u8),
        f.elem(&45u8),
      ]);
      println!("{:?}", a);
    }
    {
      let a = Polynomial::new(f, vec![
        f.elem(&12u8),
      ]);
      println!("{:?}", a);
    }
  }

  #[test]
  fn test_new_non_empty_vec() {
    let f = &Field::new(&3299u16);
    let p = Polynomial::new(f, vec![
      f.elem(&12u8),
    ]);
    assert!(p.coeffs.len() == 1);
    assert!(p.coeffs[0] == f.elem(&12u8));
  }

  #[test]
  #[should_panic]
  fn test_new_empty_vec() {
    std::panic::set_hook(Box::new(|_| {}));
    let f = &Field::new(&3299u16);
    Polynomial::new(f, vec![]);
  }

  #[test]
  fn test_normalize() {
    let f = &Field::new(&3299u16);
    {
      let a = Polynomial::new(f, vec![
        f.elem(&0u8),
      ]);
      let b = Polynomial::new(f, vec![
        f.elem(&0u8),
      ]);
      assert!(&a == &b);
      assert!(a.coeffs.len() == 1);
      assert!(&a.coeffs[0] == &f.elem(&0u8));
    }
    {
      let a = Polynomial::new(f, vec![
        f.elem(&1u8),
      ]);
      let b = Polynomial::new(f, vec![
        f.elem(&1u8),
      ]);
      assert!(&a == &b);
      assert!(a.coeffs.len() == 1);
      assert!(&a.coeffs[0] == &f.elem(&1u8));
    }
    {
      let a = Polynomial::new(f, vec![
        f.elem(&1u8),
        f.elem(&0u8),
      ]);
      let b = Polynomial::new(f, vec![
        f.elem(&1u8),
      ]);
      assert!(&a == &b);
      assert!(a.coeffs.len() == 1);
      assert!(&a.coeffs[0] == &f.elem(&1u8));
    }
    {
      let a = Polynomial::new(f, vec![
        f.elem(&1u8),
        f.elem(&0u8),
        f.elem(&0u8),
      ]);
      let b = Polynomial::new(f, vec![
        f.elem(&1u8),
      ]);
      assert!(&a == &b);
      assert!(a.coeffs.len() == 1);
      assert!(&a.coeffs[0] == &f.elem(&1u8));
    }
    {
      let a = Polynomial::new(f, vec![
        f.elem(&1u8),
        f.elem(&0u8),
        f.elem(&0u8),
        f.elem(&1u8),
      ]);
      let b = Polynomial::new(f, vec![
        f.elem(&1u8),
        f.elem(&0u8),
        f.elem(&0u8),
        f.elem(&1u8),
      ]);
      assert!(&a == &b);
      assert!(a.coeffs.len() == 4);
      assert!(&a.coeffs[0] == &f.elem(&1u8));
      assert!(&a.coeffs[1] == &f.elem(&0u8));
      assert!(&a.coeffs[2] == &f.elem(&0u8));
      assert!(&a.coeffs[3] == &f.elem(&1u8));
    }
  }

  #[test]
  fn test_eq() {
    let f = &Field::new(&3299u16);
    {
      let a = Polynomial::new(f, vec![
        f.elem(&0u8),
      ]);
      let b = Polynomial::new(f, vec![
        f.elem(&0u8),
      ]);
      assert!(&a == &b);
    }
    {
      let a = Polynomial::new(f, vec![
        f.elem(&0u8),
      ]);
      let b = Polynomial::new(f, vec![
        f.elem(&1u8),
      ]);
      assert!(&a != &b);
    }
    {
      let a = Polynomial::new(f, vec![
        f.elem(&2u8),
        f.elem(&1u8),
      ]);
      let b = Polynomial::new(f, vec![
        f.elem(&2u8),
        f.elem(&1u8),
      ]);
      assert!(&a == &b);
    }
    {
      let a = Polynomial::new(f, vec![
        f.elem(&2u8),
        f.elem(&1u8),
      ]);
      let b = Polynomial::new(f, vec![
        f.elem(&1u8),
        f.elem(&2u8),
      ]);
      assert!(&a != &b);
    }
    {
      let a = Polynomial::new(f, vec![
        f.elem(&2u8),
        f.elem(&1u8),
      ]);
      let b = Polynomial::new(f, vec![
        f.elem(&2u8),
        f.elem(&1u8),
        f.elem(&0u8),
      ]);
      assert!(&a == &b);
    }
    {
      let a = Polynomial::new(f, vec![
        f.elem(&2u8),
        f.elem(&1u8),
      ]);
      let b = Polynomial::new(f, vec![
        f.elem(&2u8),
        f.elem(&1u8),
        f.elem(&1u8),
      ]);
      assert!(&a != &b);
    }
  }

  #[test]
  fn test_add() {
    let f = &Field::new(&3299u16);
    // zero + zero
    {
      let a = Polynomial::new(f, vec![
        f.elem(&0u8),
      ]);
      let b = Polynomial::new(f, vec![
        f.elem(&0u8),
      ]);
      let c = Polynomial::new(f, vec![
        f.elem(&0u8),
      ]);
      let res = a.add(&b);
      assert!(&res == &c);
    }
    // zero + non-zero
    {
      let a = Polynomial::new(f, vec![
        f.elem(&0u8),
      ]);
      let b = Polynomial::new(f, vec![
        f.elem(&12u8),
      ]);
      let c = Polynomial::new(f, vec![
        f.elem(&12u8),
      ]);
      let res = a.add(&b);
      assert!(&res == &c);
    }
    // non-zero + non-zero
    {
      let a = Polynomial::new(f, vec![
        f.elem(&100u8),
      ]);
      let b = Polynomial::new(f, vec![
        f.elem(&12u8),
      ]);
      let c = Polynomial::new(f, vec![
        f.elem(&112u8),
      ]);
      let res = a.add(&b);
      assert!(&res == &c);
    }
  }

  #[test]
  fn test_add_zero_terms() {
    let f = &Field::new(&7u8);
    // 1st term only and it becomes zero
    {
      let a = Polynomial::new(f, vec![
        f.elem(&3u8),
      ]);
      let b = Polynomial::new(f, vec![
        f.elem(&4u8),
      ]);
      let c = Polynomial::new(f, vec![
        f.elem(&0u8),
      ]);
      let res = a.add(&b);
      assert!(&res == &c);
    }
    // adding 2-term polynomials and 2nd term becomes zero
    {
      let a = Polynomial::new(f, vec![
        f.elem(&1u8),
        f.elem(&3u8),
      ]);
      let b = Polynomial::new(f, vec![
        f.elem(&2u8),
        f.elem(&4u8),
      ]);
      let c = Polynomial::new(f, vec![
        f.elem(&3u8),
      ]);
      let res = a.add(&b);
      assert!(&res == &c);
    }
  }

  #[test]
  fn test_mul_deg_0_0() {
    let f = &Field::new(&3299u16);
    {
      // 0 * 0
      let a = Polynomial::new(f, vec![
        f.elem(&0u8),
      ]);
      let b = Polynomial::new(f, vec![
        f.elem(&0u8),
      ]);
      let c = Polynomial::new(f, vec![
        f.elem(&0u8),
      ]);
      let res = a.mul(&b);
      assert!(&res == &c);
    }
    {
      // 1 * 0
      let a = Polynomial::new(f, vec![
        f.elem(&1u8),
      ]);
      let b = Polynomial::new(f, vec![
        f.elem(&0u8),
      ]);
      let c = Polynomial::new(f, vec![
        f.elem(&0u8),
      ]);
      let res = a.mul(&b);
      assert!(&res == &c);
    }
    {
      // 0 * 1
      let a = Polynomial::new(f, vec![
        f.elem(&0u8),
      ]);
      let b = Polynomial::new(f, vec![
        f.elem(&1u8),
      ]);
      let c = Polynomial::new(f, vec![
        f.elem(&0u8),
      ]);
      let res = a.mul(&b);
      assert!(&res == &c);
    }
    {
      // 2 * 3
      let a = Polynomial::new(f, vec![
        f.elem(&2u8),
      ]);
      let b = Polynomial::new(f, vec![
        f.elem(&3u8),
      ]);
      let c = Polynomial::new(f, vec![
        f.elem(&6u8),
      ]);
      let res = a.mul(&b);
      println!("res={:?}", res);
      assert!(&res == &c);
    }
  }

  #[test]
  fn test_mul_deg_1_0() {
    let f = &Field::new(&3299u16);
    {
      // (2x - 3) * 4
      let a = Polynomial::new(f, vec![
        f.elem(&3u8),
        f.elem(&2u8),
      ]);
      let b = Polynomial::new(f, vec![
        f.elem(&4u8),
      ]);
      let c = Polynomial::new(f, vec![
        f.elem(&12u8),
        f.elem(&8u8),
      ]);
      let res = a.mul(&b);
      assert!(&res == &c);
    }
  }
  #[test]
  fn test_mul_deg_1_1() {
    let f = &Field::new(&3299u16);
    {
      // 2x + 3
      let a = Polynomial::new(f, vec![
        f.elem(&3u8),
        f.elem(&2u8),
      ]);
      // 5x + 4
      let b = Polynomial::new(f, vec![
        f.elem(&4u8),
        f.elem(&5u8),
      ]);
      // 10x^2 + 23x + 12
      let c = Polynomial::new(f, vec![
        f.elem(&12u8),
        f.elem(&23u8),
        f.elem(&10u8),
      ]);
      let res = a.mul(&b);
      println!("({:?})({:?}) = {:?}", a, b, res);
      assert!(&res == &c);
    }
  }
}