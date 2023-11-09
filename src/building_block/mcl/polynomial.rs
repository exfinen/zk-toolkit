use crate::building_block::mcl::{
  mcl_fr::MclFr,
  mcl_g1::MclG1,
  mcl_g2::MclG2,
  mcl_sparse_vec::MclSparseVec,
};
use num_traits::Zero;
use std::{
  fmt::{Debug, Formatter},
  ops::{
    Add,
    Deref,
    Mul,
    Sub,
    AddAssign, MulAssign,
  },
  convert::From,
};

// TODO use SparseVec instead of Vec to hold coeffs
#[derive(Clone)]
pub struct Polynomial {
  pub coeffs: Vec<MclFr>,  // e.g. 2x^3 + 5x + 9 -> [9, 5, 0, 2]
  _private: (),  // to force using new()
}

impl Deref for Polynomial {
  type Target = Vec<MclFr>;

  fn deref(&self) -> &Self::Target {
    &self.coeffs
  }
}

impl From<&MclSparseVec> for Polynomial {
  fn from(vec: &MclSparseVec) -> Self {
    let mut i = MclFr::zero();
    let mut coeffs = vec![];
    while i < vec.size {
      let v = vec.get(&i);
      coeffs.push(v.clone());
      i.inc();
    }
    let p = Polynomial::new(&coeffs);
    p.normalize()
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
    let one = &MclFr::from(1);
    let mut terms = vec![];
    let last_idx = self.coeffs.len() - 1;

    for (i, coeff) in self.coeffs.iter().rev().enumerate() {
      if !coeff.is_zero() {
        let mut s = String::new();
        // write number
        if coeff != one || i == self.coeffs.len() - 1 {
          s.push_str(&format!("{:?}", coeff));
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

impl Zero for Polynomial {
  fn zero() -> Self {
    let coeffs = &vec![MclFr::zero()];
    Polynomial::new(coeffs)
  }

  fn is_zero(&self) -> bool {
    self.coeffs.len() == 1 && self.coeffs[0].is_zero()
  }
}

impl Polynomial {
  pub fn new(coeffs: &Vec<MclFr>) -> Self {
    if coeffs.len() == 0 { panic!("coeffs is empty"); }
    let x = Polynomial {
      coeffs: coeffs.clone(),
      _private: ()
    };
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
    Polynomial { coeffs: norm_coeffs, _private: () }
  }

  pub fn plus(&self, rhs: &Polynomial) -> Polynomial {
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
    let x = Polynomial { coeffs, _private: () };
    x.normalize()  // normalizing b/c addition can make term coefficect zero
  }

  pub fn multiply_by(&self, rhs: &Polynomial) -> Polynomial {
    // degree of polynomial is coeffs.len - 1
    let self_degree = self.coeffs.len() - 1;
    let rhs_degree = rhs.coeffs.len() - 1;

    // coeffs len of the mul result is sum of self and rhs degrees + 1
    let new_len = self_degree + rhs_degree + 1;
    let mut coeffs = vec![MclFr::zero(); new_len];

    for i in 0..self.coeffs.len() {
      for j in 0..rhs.coeffs.len() {
        let coeff = &self.coeffs[i] * &rhs.coeffs[j];
        let degree = i + j;
        coeffs[degree] = &coeffs[degree] + coeff;
      }
    }
    Polynomial { coeffs, _private: () }
  }

  // not supporting cases where rhs degree > lhs degree
  pub fn minus(&self, rhs: &Polynomial) -> Polynomial {
    assert!(self.coeffs.len() >= rhs.coeffs.len());
    let mut coeffs = self.coeffs.clone();

    for i in 0..rhs.coeffs.len() {
      coeffs[i] = &coeffs[i] - &rhs.coeffs[i];
    }
    let p = Polynomial { coeffs, _private: () };
    p.normalize()
  }

  pub fn divide_by(&self, rhs: &Polynomial) -> DivResult {
    let mut dividend = self.clone();

    let divisor = rhs;
    let quotient_degree = dividend.len() - divisor.len();
    let divisor_coeff = &divisor[divisor.len() - 1];
    assert!(!divisor_coeff.is_zero(), "found zero coeff at highest index. use Polynomial constructor");

    let mut quotient_coeffs = vec![MclFr::zero(); quotient_degree + 1];

    while !dividend.is_zero() && dividend.len() >= divisor.len() {
      let dividend_coeff = &dividend[dividend.len() - 1];

      // create a term to multiply with divisor
      let term_coeff = dividend_coeff * divisor_coeff.inv();
      let term_degree = dividend.len() - divisor.len();
      let mut term_vec = vec![MclFr::zero(); term_degree + 1];
      term_vec[term_degree] = term_coeff.clone();
      let term_poly = Polynomial::new(&term_vec);

      // reflect term coeff to result quotient
      quotient_coeffs[term_degree] = term_coeff;

      let poly2subtract = divisor.multiply_by(&term_poly);

      // update dividend for the next round
      dividend = dividend.sub(&poly2subtract);
    }

    if dividend.is_zero() {
      DivResult::Quotient(Polynomial { coeffs: quotient_coeffs, _private: () })
    } else {
      let quotient = Polynomial { coeffs: quotient_coeffs, _private: () };
      DivResult::QuotientRemainder((quotient, dividend))
    }
  }

  pub fn eval_at(&self, x: &MclFr) -> MclFr {
    let mut multiplier = MclFr::from(1);
    let mut sum = MclFr::zero();

    for coeff in &self.coeffs {
      sum = sum + coeff * &multiplier;
      multiplier = &multiplier * x;
    }
    sum
  }

  pub fn eval_from_1_to_n(&self, n: &MclFr) -> MclSparseVec {
    let one = &MclFr::from(1);

    let mut vec = MclSparseVec::new(n);
    let mut i = MclFr::zero();
    while &i < n {
      i.inc();
      let res = self.eval_at(&i);
      vec.set(&(&i - one), &res);
    }
    vec
  }

  pub fn degree(&self) -> MclFr {
    if self.coeffs.len() == 0 {
      panic!("should have at least 1 coeff. check code");
    }
    MclFr::from(self.coeffs.len() - 1)
  }

  #[allow(non_snake_case)]
  pub fn eval_with_g1_hidings(
    &self,
    powers: &[MclG1],
  ) -> MclG1 {
    let mut sum = MclG1::zero();
    for i in 0..self.coeffs.len() {
      sum = sum + (&powers[i] * &self.coeffs[i]);
    }
    sum
  }

  #[allow(non_snake_case)]
  pub fn eval_with_g2_hidings(
    &self,
    powers: &[MclG2],
  ) -> MclG2 {
    let mut sum = MclG2::zero();
    for i in 0..self.coeffs.len() {
      sum = sum + (&powers[i] * &self.coeffs[i]);
    }
    sum
  }

  pub fn to_sparse_vec(&self, size: usize) -> MclSparseVec {
    let size = MclFr::from(size);
    let mut vec = MclSparseVec::new(&size);

    for (i, coeff) in self.coeffs.iter().enumerate() {
      let i = MclFr::from(i);
      vec.set(&i, coeff);
    }
    vec
  }
}

macro_rules! impl_add {
  ($rhs: ty, $target: ty) => {
    impl<'a> Add<$rhs> for $target {
      type Output = Polynomial;

      fn add(self, rhs: $rhs) -> Self::Output {
        self.plus(&rhs)
      }
    }
  };
}
impl_add!(Polynomial, Polynomial);
impl_add!(&Polynomial, Polynomial);
impl_add!(Polynomial, &Polynomial);
impl_add!(&Polynomial, &Polynomial);

impl AddAssign<Polynomial> for Polynomial {
  fn add_assign(&mut self, rhs: Polynomial) {
    *self = &*self + &rhs;
  }
}

impl AddAssign<&Polynomial> for Polynomial {
  fn add_assign(&mut self, rhs: &Polynomial) {
    *self = &*self + rhs;
  }
}

macro_rules! impl_mul {
  ($rhs: ty, $target: ty) => {
    impl<'a> Mul<$rhs> for $target {
      type Output = Polynomial;

      fn mul(self, rhs: $rhs) -> Self::Output {
        self.multiply_by(&rhs)
      }
    }
  };
}
impl_mul!(Polynomial, Polynomial);
impl_mul!(Polynomial, &Polynomial);
impl_mul!(&Polynomial, Polynomial);
impl_mul!(&Polynomial, &Polynomial);

impl<'a> Mul<&MclFr> for &Polynomial {
  type Output = Polynomial;

  fn mul(self, rhs: &MclFr) -> Self::Output {
    Polynomial {
      coeffs: self.coeffs.iter().map(|coeff| coeff * rhs).collect(),
      _private: (),
    }
  }
}

impl MulAssign<Polynomial> for Polynomial {
  fn mul_assign(&mut self, rhs: Polynomial) {
    *self = &*self * &rhs;
  }
}

impl MulAssign<&Polynomial> for Polynomial {
  fn mul_assign(&mut self, rhs: &Polynomial) {
    *self = &*self * rhs;
  }
}

impl<'a> Sub<&Polynomial> for Polynomial {
  type Output = Polynomial;

  fn sub(self, rhs: &Polynomial) -> Self::Output {
    self.minus(rhs)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use rand::Rng;
  use super::DivResult::{Quotient, QuotientRemainder};
  use crate::building_block::mcl::mcl_initializer::MclInitializer;

  #[test]
  fn test_to_sparse_vec() {
    MclInitializer::init();

    let zero = &MclFr::zero();
    let one = &MclFr::from(1);
    let two = &MclFr::from(2);
    let three = &MclFr::from(3);
    let four = &MclFr::from(4);

    // 2x + 3
    let coeffs = vec![
      two.clone(),
      three.clone(),
    ];
    let p = Polynomial::new(&coeffs);
    let vec = p.to_sparse_vec(four.to_usize());

    assert_eq!(&vec.size, four);
    assert_eq!(vec.get(zero), two);
    assert_eq!(vec.get(one), three);
    assert_eq!(vec.get(two), zero);
    assert_eq!(vec.get(three), zero);
  }

  #[test]
  fn test_degree() {
    MclInitializer::init();

    // degree 0
    {
      let p = Polynomial::new(&vec![
        MclFr::from(2),
      ]);
      assert_eq!(p.degree(), MclFr::zero());

      // 0 coeff case
      let p = p.normalize();
      assert_eq!(p.degree(), MclFr::zero());
    }
    // degree 1
    {
      let p = Polynomial::new(&vec![
        MclFr::from(2),
        MclFr::from(3),
      ]);
      assert_eq!(p.degree(), MclFr::from(1));
    }
  }

  #[test]
  fn test_from_sparse_vec() {
    MclInitializer::init();

    let zero = &MclFr::from(0);
    let one = &MclFr::from(1);
    let two = &MclFr::from(2);
    let three = &MclFr::from(3);
    {
      let mut vec = MclSparseVec::new(&MclFr::from(2));
      vec.set(zero, two);
      vec.set(one, three);

      let p = Polynomial::from(&vec);
      assert_eq!(&p.degree(), one);
      assert_eq!(&p.coeffs[0], two);
      assert_eq!(&p.coeffs[1], three);
    }
    {
      let mut vec = MclSparseVec::new(&MclFr::from(2));
      vec.set(zero, two);
      vec.set(one, zero);

      let p = Polynomial::from(&vec);
      assert_eq!(&p.degree(), zero);
      assert_eq!(&p.coeffs[0], two);
    }
  }

  #[test]
  fn test_eval_at() {
    MclInitializer::init();

    let zero = MclFr::from(0);
    let one = MclFr::from(1);
    let two = MclFr::from(2);
    { // 8
      let zero = MclFr::from(0);
      let eight = MclFr::from(8);
      let p = Polynomial::new(&vec![
        eight.clone(),
      ]);
      assert_eq!(&p.eval_at(&zero), &eight);
    }
    { // 3x + 8
      let p = &Polynomial::new(&vec![
        MclFr::from(8),
        MclFr::from(3),
      ]);
      assert_eq!(p.eval_at(&zero), MclFr::from(8));
      assert_eq!(p.eval_at(&one), MclFr::from(11));
      assert_eq!(p.eval_at(&two), MclFr::from(14));
    }
    { // 2x^2 + 3x + 8
      let p = &Polynomial::new(&vec![
        MclFr::from(8),
        MclFr::from(3),
        MclFr::from(2),
      ]);
      assert_eq!(p.eval_at(&zero), MclFr::from(8));
      assert_eq!(p.eval_at(&one), MclFr::from(13));
      assert_eq!(p.eval_at(&two), MclFr::from(22));
    }
  }

  #[test]
  fn test_div_3_2_no_remainder() {
    MclInitializer::init();
    {
      /*
            _____________
        x+2 ) x² + x + 5
      */
      let dividend = Polynomial::new(&vec![
        MclFr::from(5),
        MclFr::from(1),
        MclFr::from(1),
      ]);
      let divisor = Polynomial::new(&vec![
        MclFr::from(2),
        MclFr::from(1),
      ]);
      let res = dividend.divide_by(&divisor);
      if let QuotientRemainder((q, r)) = res {
        assert_eq!(dividend, divisor * q + r);
      } else if let Quotient(q) = res {
        panic!("expected remainder, but got quotient {:?} w/ no remainder", q);
      } else {
        panic!("should not be visited");
      }
    }
  }

  #[test]
  fn test_div_2_2() {
    MclInitializer::init();
    {
      /*
            _________
        x+7 ) 2x + 3
      */
      let dividend = Polynomial::new(&vec![
        MclFr::from(3),
        MclFr::from(2),
      ]);
      let divisor = Polynomial::new(&vec![
        MclFr::from(7),
        MclFr::from(1),
      ]);
      let quotient = Polynomial::new(&vec![
        MclFr::from(2),
      ]);
      let remainder = Polynomial::new(&vec![
        -MclFr::from(11),
      ]);
      let res = dividend.divide_by(&divisor);
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
    MclInitializer::init();
    {
      /*
             (32 * 10^-1)
             31461525105075714287668644304911579502614331500316582693562195219963148710711
          ________
       10 )  32
             32
             --
              0
      */
      let dividend = Polynomial::new(&vec![
        MclFr::from(32),
      ]);
      let divisor = Polynomial::new(&vec![
        MclFr::from(10),
      ]);
      let quotient = Polynomial::new(&vec![
        MclFr::from(32) * MclFr::from(10).inv(),
      ]);
      let res = dividend.divide_by(&divisor);
      if let QuotientRemainder((q, r)) = res {
        panic!("no remainder expected, but got q={:?}, r={:?})", q, r);
      } else if let Quotient(q) = res {
        assert!(q == quotient);
      } else {
        panic!("should not be visited");
      }
    }
  }

  #[test]
  fn test_div_5_2() {
    MclInitializer::init();
    {
      let dividend = Polynomial::new(&vec![
        MclFr::from(5), // 0
        MclFr::from(0), // 1
        MclFr::from(0), // 2
        MclFr::from(4), // 3
        MclFr::from(7), // 4
        MclFr::from(0), // 5
        MclFr::from(3), // 6
      ]);
      let divisor = Polynomial::new(&vec![
        MclFr::from(4), // 0
        MclFr::from(0), // 1
        MclFr::from(0), // 2
        MclFr::from(3), // 3
        MclFr::from(1), // 4
      ]);
      let res = &dividend.divide_by(&divisor);
      if let QuotientRemainder((q, r)) = res {
        assert_eq!(dividend, divisor * q + r);
      } else if let Quotient(q) = res {
        panic!("expected remainder, but got quotient {:?} w/ no remainder", q);
      } else {
        panic!("should not be visited");
      }
    }
  }

  fn gen_random_polynomial(degree: usize, max_coeff: u32) -> Polynomial {
    let mut coeffs = vec![];

    for _ in 0..degree {
      let coeff: u32 = rand::thread_rng().gen_range(0..max_coeff);
      coeffs.push(MclFr::from(coeff as i32));
    }
    Polynomial::new(&coeffs)
  }

  #[test]
  fn test_div_random_divisible() {
    MclInitializer::init();

    let max_coeff = 100;
    let min_divisor_degree = 30;
    let max_divisor_degree = 100;

    for _ in 0..10 {
      let divisor_degree = rand::thread_rng().gen_range(min_divisor_degree..max_divisor_degree);
      let quotient_degree = rand::thread_rng().gen_range(1..divisor_degree);

      let divisor = &gen_random_polynomial(divisor_degree, max_coeff);
      let quotient = &gen_random_polynomial(quotient_degree, max_coeff);
      let dividend = divisor.multiply_by(quotient);

      match &&dividend.divide_by(divisor) {
        Quotient(q) => {
          assert!(q == quotient);
        },
        QuotientRemainder(x) => {
          panic!("unexpected remainder {:?}", x);
        },
      };
    }
  }

  #[test]
  fn test_is_zero() {
    MclInitializer::init();
    {
      let a = Polynomial::new(&vec![
        MclFr::from(12),
        MclFr::from(7),
      ]);
      assert!(!a.is_zero());
    }
    {
      let a = Polynomial::new(&vec![
        MclFr::from(7),
      ]);
      assert!(!a.is_zero());
    }
    {
      let a = Polynomial::new(&vec![
        MclFr::from(0),
      ]);
      assert!(a.is_zero());
    }
  }

  #[test]
  fn test_sub_2_2() {
    MclInitializer::init();

    // subtract small poly
    {
      let a = Polynomial::new(&vec![
        MclFr::from(12),
        MclFr::from(7),
      ]);
      let b = Polynomial::new(&vec![
        MclFr::from(3),
        MclFr::from(4),
      ]);
      let c = Polynomial::new(&vec![
        MclFr::from(9),
        MclFr::from(3),
      ]);
      assert!(a.sub(&b) == c);
    }
    // subtract bigger poly
    {
      let a = Polynomial::new(&vec![
        MclFr::from(12),
        MclFr::from(7),
      ]);
      let b = Polynomial::new(&vec![
        MclFr::from(15),
        MclFr::from(8),
      ]);
      let c = Polynomial::new(&vec![
        -MclFr::from(3),
        -MclFr::from(1),
      ]);
      assert!(a.sub(&b) == c);
    }
    // subtract the same poly
    {
      let a = &Polynomial::new(&vec![
        MclFr::from(12),
        MclFr::from(7),
      ]);
      let c = Polynomial::new(&vec![
        MclFr::from(0),
      ]);
      println!("res = {:?}", a.minus(a));
      assert!(a.minus(&a) == c);
    }
  }

  #[test]
  #[should_panic]
  fn test_bad_sub() {
    MclInitializer::init();

    std::panic::set_hook(Box::new(|_| {}));
    let a = Polynomial::new(&vec![
      MclFr::from(7),
    ]);
    let b = Polynomial::new(&vec![
      MclFr::from(3),
      MclFr::from(4),
    ]);
    let _ = a.sub(&b);
  }

  #[test]
  fn test_debug_print() {
    MclInitializer::init();
    {
      let a = Polynomial::new(&vec![
        MclFr::from(12),
        MclFr::from(45),
        MclFr::from(67),
      ]);
      println!("{:?}", a);
    }
    {
      let a = Polynomial::new(&vec![
        MclFr::from(12),
        MclFr::from(45),
      ]);
      println!("{:?}", a);
    }
    {
      let a = Polynomial::new(&vec![
        MclFr::from(12),
      ]);
      println!("{:?}", a);
    }
  }

  #[test]
  fn test_new_non_empty_vec() {
    MclInitializer::init();

    let p = Polynomial::new(&vec![
      MclFr::from(12),
    ]);
    assert!(p.coeffs.len() == 1);
    assert!(p.coeffs[0] == MclFr::from(12));
  }

  #[test]
  #[should_panic]
  fn test_new_empty_vec() {
    MclInitializer::init();

    std::panic::set_hook(Box::new(|_| {}));
    Polynomial::new(&vec![]);
  }

  #[test]
  fn test_normalize() {
    MclInitializer::init();
    {
      let a = Polynomial::new(&vec![
        MclFr::from(0),
      ]);
      let b = Polynomial::new(&vec![
        MclFr::from(0),
      ]);
      assert!(&a == &b);
      assert!(a.coeffs.len() == 1);
      assert!(&a.coeffs[0] == &MclFr::from(0));
    }
    {
      let a = Polynomial::new(&vec![
        MclFr::from(1),
      ]);
      let b = Polynomial::new(&vec![
        MclFr::from(1),
      ]);
      assert!(&a == &b);
      assert!(a.coeffs.len() == 1);
      assert!(&a.coeffs[0] == &MclFr::from(1));
    }
    {
      let a = Polynomial::new(&vec![
        MclFr::from(1),
        MclFr::from(0),
      ]);
      let b = Polynomial::new(&vec![
        MclFr::from(1),
      ]);
      assert!(&a == &b);
      assert!(a.coeffs.len() == 1);
      assert!(&a.coeffs[0] == &MclFr::from(1));
    }
    {
      let a = Polynomial::new(&vec![
        MclFr::from(1),
        MclFr::from(0),
        MclFr::from(0),
      ]);
      let b = Polynomial::new(&vec![
        MclFr::from(1),
      ]);
      assert!(&a == &b);
      assert!(a.coeffs.len() == 1);
      assert!(&a.coeffs[0] == &MclFr::from(1));
    }
    {
      let a = Polynomial::new(&vec![
        MclFr::from(1),
        MclFr::from(0),
        MclFr::from(0),
        MclFr::from(1),
      ]);
      let b = Polynomial::new(&vec![
        MclFr::from(1),
        MclFr::from(0),
        MclFr::from(0),
        MclFr::from(1),
      ]);
      assert!(&a == &b);
      assert!(a.coeffs.len() == 4);
      assert!(&a.coeffs[0] == &MclFr::from(1));
      assert!(&a.coeffs[1] == &MclFr::from(0));
      assert!(&a.coeffs[2] == &MclFr::from(0));
      assert!(&a.coeffs[3] == &MclFr::from(1));
    }
  }

  #[test]
  fn test_eq() {
    MclInitializer::init();
    {
      let a = Polynomial::new(&vec![
        MclFr::from(0),
      ]);
      let b = Polynomial::new(&vec![
        MclFr::from(0),
      ]);
      assert!(&a == &b);
    }
    {
      let a = Polynomial::new(&vec![
        MclFr::from(0),
      ]);
      let b = Polynomial::new(&vec![
        MclFr::from(1),
      ]);
      assert!(&a != &b);
    }
    {
      let a = Polynomial::new(&vec![
        MclFr::from(2),
        MclFr::from(1),
      ]);
      let b = Polynomial::new(&vec![
        MclFr::from(2),
        MclFr::from(1),
      ]);
      assert!(&a == &b);
    }
    {
      let a = Polynomial::new(&vec![
        MclFr::from(2),
        MclFr::from(1),
      ]);
      let b = Polynomial::new(&vec![
        MclFr::from(1),
        MclFr::from(2),
      ]);
      assert!(&a != &b);
    }
    {
      let a = Polynomial::new(&vec![
        MclFr::from(2),
        MclFr::from(1),
      ]);
      let b = Polynomial::new(&vec![
        MclFr::from(2),
        MclFr::from(1),
        MclFr::from(0),
      ]);
      assert!(&a == &b);
    }
    {
      let a = Polynomial::new(&vec![
        MclFr::from(2),
        MclFr::from(1),
      ]);
      let b = Polynomial::new(&vec![
        MclFr::from(2),
        MclFr::from(1),
        MclFr::from(1),
      ]);
      assert!(&a != &b);
    }
  }

  #[test]
  fn test_add() {
    MclInitializer::init();
    // zero + zero
    {
      let a = Polynomial::new(&vec![
        MclFr::from(0),
      ]);
      let b = Polynomial::new(&vec![
        MclFr::from(0),
      ]);
      let c = Polynomial::new(&vec![
        MclFr::from(0),
      ]);
      let res = a.add(&b);
      assert!(&res == &c);
    }
    // zero + non-zero
    {
      let a = Polynomial::new(&vec![
        MclFr::from(0),
      ]);
      let b = Polynomial::new(&vec![
        MclFr::from(12),
      ]);
      let c = Polynomial::new(&vec![
        MclFr::from(12),
      ]);
      let res = a.add(&b);
      assert!(&res == &c);
    }
    // non-zero + non-zero
    {
      let a = Polynomial::new(&vec![
        MclFr::from(100),
      ]);
      let b = Polynomial::new(&vec![
        MclFr::from(12),
      ]);
      let c = Polynomial::new(&vec![
        MclFr::from(112),
      ]);
      let res = a.add(&b);
      assert!(&res == &c);
    }
  }

  #[test]
  fn test_add_zero_term() {
    MclInitializer::init();
    {
      let a = Polynomial::new(&vec![
        MclFr::from(3),
      ]);
      let b = Polynomial::new(&vec![
        MclFr::from(0),
      ]);
      let c = Polynomial::new(&vec![
        MclFr::from(3),
      ]);
      let res = a.add(&b);
      assert!(&res == &c);
    }
  }

  #[test]
  fn test_mul_deg_0_0() {
    MclInitializer::init();
    {
      // 0 * 0
      let a = Polynomial::new(&vec![
        MclFr::from(0),
      ]);
      let b = Polynomial::new(&vec![
        MclFr::from(0),
      ]);
      let c = Polynomial::new(&vec![
        MclFr::from(0),
      ]);
      let res = a.mul(&b);
      assert!(&res == &c);
    }
    {
      // 1 * 0
      let a = Polynomial::new(&vec![
        MclFr::from(1),
      ]);
      let b = Polynomial::new(&vec![
        MclFr::from(0),
      ]);
      let c = Polynomial::new(&vec![
        MclFr::from(0),
      ]);
      let res = a.mul(&b);
      assert!(&res == &c);
    }
    {
      // 0 * 1
      let a = Polynomial::new(&vec![
        MclFr::from(0),
      ]);
      let b = Polynomial::new(&vec![
        MclFr::from(1),
      ]);
      let c = Polynomial::new(&vec![
        MclFr::from(0),
      ]);
      let res = a.mul(&b);
      assert!(&res == &c);
    }
    {
      // 2 * 3
      let a = Polynomial::new(&vec![
        MclFr::from(2),
      ]);
      let b = Polynomial::new(&vec![
        MclFr::from(3),
      ]);
      let c = Polynomial::new(&vec![
        MclFr::from(6),
      ]);
      let res = a.mul(&b);
      println!("res={:?}", res);
      assert!(&res == &c);
    }
  }

  #[test]
  fn test_mul_deg_1_0() {
    MclInitializer::init();
    {
      // (2x - 3) * 4
      let a = Polynomial::new(&vec![
        MclFr::from(3),
        MclFr::from(2),
      ]);
      let b = Polynomial::new(&vec![
        MclFr::from(4),
      ]);
      let c = Polynomial::new(&vec![
        MclFr::from(12),
        MclFr::from(8),
      ]);
      let res = a.mul(&b);
      assert!(&res == &c);
    }
  }

  #[test]
  fn test_mul_deg_1_1() {
    MclInitializer::init();
    {
      // 2x + 3
      let a = &Polynomial::new(&vec![
        MclFr::from(3),
        MclFr::from(2),
      ]);
      // 5x + 4
      let b = &Polynomial::new(&vec![
        MclFr::from(4),
        MclFr::from(5),
      ]);
      // 10x^2 + 23x + 12
      let c = &Polynomial::new(&vec![
        MclFr::from(12),
        MclFr::from(23),
        MclFr::from(10),
      ]);
      let res = a.multiply_by(&b);
      println!("({:?})({:?}) = {:?}", a, b, res);
      assert!(&res == c);
    }
  }

  #[test]
  fn test_mul_const() {
    MclInitializer::init();
    {
      // 2x + 3
      let a = Polynomial::new(&vec![
        MclFr::from(3),
        MclFr::from(2),
      ]);
      let ten = MclFr::from(10);

      // 20x + 30
      let exp = &Polynomial::new(&vec![
        MclFr::from(30),
        MclFr::from(20),
      ]);

      let act = &a * &ten;

      println!("({:?}) * {:?} = {:?}", a, &ten, &act);
      assert!(&act == exp);
    }
  }

  #[test]
  fn test_eval_from_1_to_n() {
    MclInitializer::init();
    {
      // evaluating for the same degree as the polynomial degree
      let p = Polynomial::new(&vec![
          MclFr::from(2),
          MclFr::from(3),
          MclFr::from(5),
      ]);
      let three = &MclFr::from(3);
      let vec = p.eval_from_1_to_n(three);
      assert_eq!(&vec.size, three);
      assert_eq!(vec.get(&MclFr::zero()), &MclFr::from(10));
      assert_eq!(vec.get(&MclFr::from(1)), &MclFr::from(28));
      assert_eq!(vec.get(&MclFr::from(2)), &MclFr::from(56));
    }
    {
      // evaluating for larger degree than the polynomial degree
      let zero = &MclFr::from(0);
      let three = &MclFr::from(3);

      let p = Polynomial::new(&vec![
          zero.clone(),
      ]);
      let vec = p.eval_from_1_to_n(three);
      assert_eq!(&vec.size, three);
      assert_eq!(vec.get(zero), zero);
      assert_eq!(vec.get(&MclFr::from(1)), zero);
      assert_eq!(vec.get(&MclFr::from(2)), zero);
    }
  }

  #[test]
  fn test_eval_with_g1_hidings_1() {
    MclInitializer::init();

    let s = MclFr::from(3);
    let s0g = &MclG1::g();
    let s1g = s0g * &s;
    let s2g = s0g * &s * &s;
    let s3g = s0g * &s * &s * &s;
    let pows = vec![
      s0g.clone(),
      s1g.clone(),
      s2g.clone(),
      s3g.clone(),
    ];
    let two = MclFr::from(2);
    let three = MclFr::from(3);
    let four = MclFr::from(4);
    let five = MclFr::from(5);

    let exp = 
      s0g * &two
      + &s1g * &three
      + &s2g * &four
      + &s3g * &five
    ;
    // 5x^3 + 4x^2 + 3x + 2
    let p = Polynomial::new(&vec![
      two,
      three,
      four,
      five,
    ]);
    let act = p.eval_with_g1_hidings(&pows);

    assert!(act == exp);
  }

  #[test]
  fn test_eval_with_g1_order() {
    MclInitializer::init();

    let s = MclFr::from(3);

    let e1549 = MclFr::from(1549);
    let e3361 = MclFr::from(3361);
    let e3607 = MclFr::from(3607);
    let e822 = MclFr::from(822);
    let e1990 = MclFr::from(1990);
    let e496 = MclFr::from(496);
    let e1698 = MclFr::from(1698);
    let e2362 = MclFr::from(2362);
    let e3670 = MclFr::from(3670);

    // 3670x^8 + 2362x^7 + 1698x^6 + 496x^5 + 1990x^4 + 822x^3 + 3607x^2 + 3361x + 1549
    let p = Polynomial::new(&vec![
      e1549,
      e3361,
      e3607,
      e822,
      e1990,
      e496,
      e1698,
      e2362,
      e3670,
    ]);
    assert!(p.eval_at(&s) == MclFr::from(30830413));

  }
}
