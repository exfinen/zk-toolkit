use crate::building_block::field::{Field, FieldElem};
use num_bigint::BigUint;
use num_traits::One;
use std::{
  fmt::{Debug, Formatter},
  ops::Deref,
};
use num_traits::identities::Zero;

#[derive(Clone)]
pub struct Polynomial {
  pub f: Field,
  pub coeffs: Vec<FieldElem>,
  _private: (),  // to force using new()
}

impl Deref for Polynomial {
  type Target = Vec<FieldElem>;

  fn deref(&self) -> &Self::Target {
    &self.coeffs
  }
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
    let zero = BigUint::zero();
    let one = BigUint::one();

    let mut terms = vec![];
    let last_idx = self.coeffs.len() - 1;

    for (i, coeff) in self.coeffs.iter().rev().enumerate() {
      if coeff.n != zero {
        let mut s = String::new();
        // write number
        if coeff.n != one || i == self.coeffs.len() - 1 {
          s.push_str(&format!("{:?}", coeff.n));
        }

        // if not the constant term, write variable after number
        if i < last_idx {
          s.push_str("x");
          // write exponent if x^2 or higher
          if i < last_idx - 1 {  // second to last corresponds to x^1
            s.push_str(&format!("^{}", self.coeffs.len() - 1 - i));
          }
        }
        terms.push(s);
      }
    }

    let expr = terms.iter().map(|x| format!("{}", x)).collect::<Vec<String>>().join(" + ");
    write!(f, "{}", expr)
  }
}

#[derive(Debug)]
pub enum DivResult {
  Quotient(Polynomial),
  QuotientRemainder((Polynomial, Polynomial)),
}

impl Polynomial {
  pub fn new(f: &Field, coeffs: Vec<FieldElem>) -> Self {
    if coeffs.len() == 0 { panic!("coeffs is empty"); }
    let x = Polynomial { f: f.clone(), coeffs, _private: () };
    x.normalize()
  }

  pub fn is_zero(&self) -> bool {
    self.coeffs.len() == 1 && self.coeffs[0].is_zero()
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

  // not supporting cases where rhs degree > lhs degree
  pub fn sub(&self, rhs: &Polynomial) -> Polynomial {
    assert!(self.coeffs.len() >= rhs.coeffs.len());
    let mut coeffs = self.coeffs.clone();

    for i in 0..rhs.coeffs.len() {
      coeffs[i] = &coeffs[i] - &rhs.coeffs[i];
    }
    let p = Polynomial { f: self.f.clone(), coeffs, _private: () };
    p.normalize()
  }

  pub fn div(&self, rhs: &Polynomial) -> DivResult {
    let mut dividend = self.clone();
    let divisor = rhs.clone();
    let quotient_degree = dividend.len() - divisor.len();
    let divisor_coeff = &divisor[divisor.len() - 1];
    assert!(!divisor_coeff.n.is_zero(), "found zero coeff at highest index. use Polynomial constructor");

    let mut quotient_coeffs = vec![self.f.elem(&0u8); quotient_degree + 1];

    while !dividend.is_zero() && dividend.len() >= divisor.len() {
      let dividend_coeff = &dividend[dividend.len() - 1];

      // create a term to multiply with divisor
      let term_coeff = dividend_coeff / divisor_coeff;
      let term_degree = dividend.len() - divisor.len();
      let mut term_vec = vec![self.f.elem(&0u8); term_degree + 1];
      term_vec[term_degree] = term_coeff.clone();
      let term_poly = Polynomial::new(&self.f, term_vec);

      // reflect term coeff to result quotient
      quotient_coeffs[term_degree] = term_coeff;

      let poly2subtract = divisor.mul(&term_poly);

      // update dividend for the next round
      dividend = dividend.sub(&poly2subtract);
    }

    if dividend.is_zero() {
      DivResult::Quotient(Polynomial { f: self.f.clone(), coeffs: quotient_coeffs, _private: () })
    } else {
      let quotient = Polynomial { f: self.f.clone(), coeffs: quotient_coeffs, _private: () };
      DivResult::QuotientRemainder((quotient, dividend))
    }
  }

  pub fn eval_at(&self, x: &FieldElem) -> FieldElem {
    let mut multiplier = self.f.elem(&1u8);
    let mut sum = self.f.elem(&0u8);

    for coeff in &self.coeffs {
      sum = sum + coeff * &multiplier;
      multiplier = &multiplier * x;
    }
    sum
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::building_block::field::Field;
  use rand::Rng;
  use super::DivResult::{Quotient, QuotientRemainder};

  #[test]
  fn test_eval_at() {
    let f = &Field::new(&3911u16);
    { // 8
      let zero = &f.elem(&0u8);
      let eight = &f.elem(&8u8);
      let p = Polynomial::new(f, vec![
        eight.clone(),
      ]);
      assert_eq!(&p.eval_at(&zero), eight);
    }
    { // 3x + 8
      let p = &Polynomial::new(f, vec![
        f.elem(&8u8),
        f.elem(&3u8),
      ]);
      assert_eq!(p.eval_at(&f.elem(&0u8)), f.elem(&8u8));
      assert_eq!(p.eval_at(&f.elem(&1u8)), f.elem(&11u8));
      assert_eq!(p.eval_at(&f.elem(&2u8)), f.elem(&14u8));
    }
    { // 2x^2 + 3x + 8
      let p = &Polynomial::new(f, vec![
        f.elem(&8u8),
        f.elem(&3u8),
        f.elem(&2u8),
      ]);
      assert_eq!(p.eval_at(&f.elem(&0u8)), f.elem(&8u8));
      assert_eq!(p.eval_at(&f.elem(&1u8)), f.elem(&13u8));
      assert_eq!(p.eval_at(&f.elem(&2u8)), f.elem(&22u8));
    }
  }

  #[test]
  fn test_div_3_2_no_remainder() {
    let f = &Field::new(&7u8);
    {
      /* in GF(7)
              x +  6
            ______________
        x+2 ) x² +  x +  5
              x² + 2x    // -2 = 5 mod 7
              -------
                   6x +  5  // -1 = 6 mod 7
                   6x +  5  // 12 = 5 mod 7
                   -------
                         0
      */
      let dividend = Polynomial::new(f, vec![
        f.elem(&5u8),
        f.elem(&1u8),
        f.elem(&1u8),
      ]);
      let divisor = Polynomial::new(f, vec![
        f.elem(&2u8),
        f.elem(&1u8),
      ]);
      let quotient = Polynomial::new(f, vec![
        f.elem(&6u8),
        f.elem(&1u8),
      ]);
      let res = dividend.div(&divisor);
      if let QuotientRemainder(x) = res {
        panic!("expected no remainder, but got {:?}", x);
      } else if let Quotient(q) = res {
        assert!(q == quotient);
      } else {
        panic!("should not be visited");
      }
    }
  }

  #[test]
  fn test_div_2_2() {
    let f = &Field::new(&7u8);
    {
      /* in GF(7)
              2
            ______________
        x+7 ) 2x +  3
              2x + 14   // 14 = 0 mod 7
              -------
                    3
      */
      let dividend = Polynomial::new(f, vec![
        f.elem(&3u8),
        f.elem(&2u8),
      ]);
      let divisor = Polynomial::new(f, vec![
        f.elem(&7u8),
        f.elem(&1u8),
      ]);
      let quotient = Polynomial::new(f, vec![
        f.elem(&2u8),
      ]);
      let remainder = Polynomial::new(f, vec![
        f.elem(&3u8),
      ]);
      let res = dividend.div(&divisor);
      println!("result = {:?}", &res);
      if let QuotientRemainder((q, r)) = res {
        assert!(q == quotient);
        assert!(r == remainder);
      } else if let Quotient(q) = res {
        panic!("expected remainder, but got quotient {:?} w/ no remainder", q);
      } else {
        panic!("should not be visited");
      }
    }
  }

  #[test]
  fn test_div_2_2_non_divisible_coeff_with_remainder() {
    let f = &Field::new(&7u8);
    {
      /* in GF(7)
              6        // 6*2=12=5 mod 7
            ______________
       2x+7 ) 5x + 3
              5x + 0   // 42 = 0 mod 7
              -------
                   3
      */
      let dividend = Polynomial::new(f, vec![
        f.elem(&3u8),
        f.elem(&5u8),
      ]);
      let divisor = Polynomial::new(f, vec![
        f.elem(&7u8),
        f.elem(&2u8),
      ]);
      let quotient = Polynomial::new(f, vec![
        f.elem(&6u8),
      ]);
      let remainder = Polynomial::new(f, vec![
        f.elem(&3u8),
      ]);
      let res = dividend.div(&divisor);
      if let QuotientRemainder((q, r)) = res {
        assert!(q == quotient);
        assert!(r == remainder);
      } else if let Quotient(q) = res {
        panic!("expected remainder, but got quotient {:?} w/ no remainder", q);
      } else {
        panic!("should not be visited");
      }
    }
  }

  #[test]
  fn test_div_1_1() {
    let f = &Field::new(&7u8);
    {
      /* in GF(7)
            5     // 2*5=10=3 mod 7
         ________
       2 )  3
            3
           --
            0
      */
      let dividend = Polynomial::new(f, vec![
        f.elem(&3u8),
      ]);
      let divisor = Polynomial::new(f, vec![
        f.elem(&2u8),
      ]);
      let quotient = Polynomial::new(f, vec![
        f.elem(&5u8),
      ]);
      let res = dividend.div(&divisor);
      if let QuotientRemainder(x) = res {
        panic!("expected no remainder, but got {:?}", x);
      } else if let Quotient(q) = res {
        assert!(q == quotient);
      } else {
        panic!("should not be visited");
      }
    }

  }

  #[test]
  fn test_div_5_2() {
    let f = &Field::new(&11u8);
    {
      let dividend = Polynomial::new(f, vec![
        f.elem(&5u8), // 0
        f.elem(&0u8), // 1
        f.elem(&0u8), // 2
        f.elem(&4u8), // 3
        f.elem(&7u8), // 4
        f.elem(&0u8), // 5
        f.elem(&3u8), // 6
      ]);
      let divisor = Polynomial::new(f, vec![
        f.elem(&4u8), // 0
        f.elem(&0u8), // 1
        f.elem(&0u8), // 2
        f.elem(&3u8), // 3
        f.elem(&1u8), // 4
      ]);
      let quotient = Polynomial::new(f, vec![
        f.elem(&1u8), // 0
        f.elem(&2u8), // 1
        f.elem(&3u8), // 2
      ]);
      let remainder = Polynomial::new(f, vec![
        f.elem(&1u8),  // 0
        f.elem(&3u8),  // 1
        f.elem(&10u8), // 2
        f.elem(&1u8),  // 3
      ]);
      let res = dividend.div(&divisor);
      if let QuotientRemainder((q, r)) = res {
        assert!(q == quotient);
        assert!(r == remainder);
      } else if let Quotient(q) = res {
        panic!("expected remainder, but got quotient {:?} w/ no remainder", q);
      } else {
        panic!("should not be visited");
      }
    }
  }

  fn gen_random_polynomial(f: &Field, degree: usize, max_coeff: u32) -> Polynomial {
    let mut coeffs = vec![];

    for _ in 0..degree {
      let coeff: u32 = rand::thread_rng().gen_range(0..max_coeff);
      coeffs.push(f.elem(&coeff));
    }
    Polynomial::new(f, coeffs)
  }

  #[test]
  fn test_div_random_divisible() {
    let f = &Field::new(&11u8);
    let max_coeff = 100;
    let min_divisor_degree = 30;
    let max_divisor_degree = 100;

    for _ in 0..10 {
      let divisor_degree = rand::thread_rng().gen_range(min_divisor_degree..max_divisor_degree);
      let quotient_degree = rand::thread_rng().gen_range(1..divisor_degree);

      let divisor = gen_random_polynomial(f, divisor_degree, max_coeff);
      let quotient = gen_random_polynomial(f, quotient_degree, max_coeff);
      let dividend = divisor.mul(&quotient);

      match &dividend.div(&divisor) {
        Quotient(q) => {
          assert!(q == &quotient);
        },
        QuotientRemainder(x) => {
          panic!("unexpected remainder {:?}", x);
        },
      };
    }
  }

  #[test]
  fn test_is_zero() {
    let f = &Field::new(&11u8);
    {
      let a = Polynomial::new(f, vec![
        f.elem(&12u8),
        f.elem(&7u8),
      ]);
      assert!(!a.is_zero());
    }
    {
      let a = Polynomial::new(f, vec![
        f.elem(&7u8),
      ]);
      assert!(!a.is_zero());
    }
    {
      let a = Polynomial::new(f, vec![
        f.elem(&0u8),
      ]);
      assert!(a.is_zero());
    }
  }

  #[test]
  fn test_sub_2_2() {
    let f = &Field::new(&23u8);
    // subtract small poly
    {
      let a = Polynomial::new(f, vec![
        f.elem(&12u8),
        f.elem(&7u8),
      ]);
      let b = Polynomial::new(f, vec![
        f.elem(&3u8),
        f.elem(&4u8),
      ]);
      let c = Polynomial::new(f, vec![
        f.elem(&9u8),
        f.elem(&3u8),
      ]);
      assert!(a.sub(&b) == c);
    }
    // subtract bigger poly
    {
      let a = Polynomial::new(f, vec![
        f.elem(&12u8),
        f.elem(&7u8),
      ]);
      let b = Polynomial::new(f, vec![
        f.elem(&15u8),
        f.elem(&8u8),
      ]);
      let c = Polynomial::new(f, vec![
        f.elem(&20u8),
        f.elem(&22u8),
      ]);
      assert!(a.sub(&b) == c);
    }
    // subtract the same poly
    {
      let a = Polynomial::new(f, vec![
        f.elem(&12u8),
        f.elem(&7u8),
      ]);
      let c = Polynomial::new(f, vec![
        f.elem(&0u8),
      ]);
      println!("res = {:?}", a.sub(&a));
      assert!(a.sub(&a) == c);
    }
  }

  #[test]
  #[should_panic]
  fn test_bad_sub() {
    std::panic::set_hook(Box::new(|_| {}));
    let f = &Field::new(&3299u16);
    let a = Polynomial::new(f, vec![
      f.elem(&7u8),
    ]);
    let b = Polynomial::new(f, vec![
      f.elem(&3u8),
      f.elem(&4u8),
    ]);
    a.sub(&b);
  }

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