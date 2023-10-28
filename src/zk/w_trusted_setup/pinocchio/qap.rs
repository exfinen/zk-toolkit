use std::ops::Mul;

use crate::building_block::{
  field::prime_field::PrimeField,
  to_biguint::ToBigUint,
};
use crate::zk::w_trusted_setup::pinocchio::{
  r1cs::R1CS,
  polynomial::{
    Polynomial,
    DivResult,
  },
  sparse_vec::SparseVec,
};
use num_traits::Zero;
use num_bigint::BigUint;

#[derive(Clone)]
pub struct QAP {
  pub f: PrimeField,
  pub vi: Vec<Polynomial>,
  pub wi: Vec<Polynomial>,
  pub yi: Vec<Polynomial>,
  pub num_constraints: BigUint,
}

impl QAP {
  // build a polynomial that evaluates to target_val at x == index
  // and zero for x != index for each target value.
  // e.g.
  // (x - 2) * (x - 3) * 3 / ((1 - 2) * (1 - 3))
  // where x in [1, 2, 3]; evaluates to 3 if x == 1 and 0 if x != 1
  fn build_polynomial(
    f: &PrimeField,
    target_vals: &SparseVec,
  ) -> Polynomial {
    let mut target_val_polys = vec![];

    let one = f.elem(&1u8);
    let mut target_x = f.elem(&1u8);
    while target_x <= target_vals.size {
      let target_val = target_vals.get(&(&target_x - &one));

      // if target val is zero, simply add 0x^0
      if target_val.e.is_zero() {
        target_val_polys.push(Polynomial::new(f, &vec![f.elem(&0u8)]));
        target_x.inc();
        continue;
      }

      let mut numerator_polys = vec![
        Polynomial::new(f, &vec![target_val.clone()]),
      ];
      let mut denominator = f.elem(&1u8);

      let mut i = f.elem(&1u8);
      while i <= target_vals.size {
        if i == target_x {
          i.inc();
          continue;
        }
        // (x - i) to let the polynomal evaluate to zero at x = i
        let numerator_poly = Polynomial::new(f, &vec![
          -f.elem(&i),
          f.elem(&1u8),
        ]);
        numerator_polys.push(numerator_poly);

        // (target_idx - i) to cancel out the corresponding
        // numerator_poly at x = target_idx
        denominator = denominator * (f.elem(&target_x) - f.elem(&i));

        i.inc();
      }

      // merge denominator polynomial to numerator polynomial vector
      let denominator_poly = Polynomial::new(f, &vec![denominator.inv()]);
      let mut polys = numerator_polys;
      polys.push(denominator_poly);

      // aggregate numerator polynomial vector
      let mut acc_poly = Polynomial::new(f, &vec![f.elem(&1u8)]);
      for poly in polys {
        acc_poly = acc_poly.mul(&poly);
      }
      target_val_polys.push(acc_poly);

      target_x.inc();
    }

    // aggregate polynomials for all target values
    let mut res = target_val_polys[0].clone();
    for x in &target_val_polys[1..] {
      res = res + x;
    }
    res
  }

  pub fn build_p(&self, witness: &SparseVec) -> Polynomial {
    let zero = &Polynomial::zero(&self.f);
    let (mut v, mut w, mut y) =
      (zero.clone(), zero.clone(), zero.clone());

    for i in 0..witness.size_in_usize() {
      let wit = &witness[&self.f.elem(&i)];
      v = &v + &(&self.vi[i] * wit);
      w = &w + &(&self.wi[i] * wit);
      y = &y + &(&self.yi[i] * wit);
    };
    
    (v * &w) - &y
  }

  // build polynomial (x-1)(x-2)..(x-num_constraints)
  pub fn build_t(f: &PrimeField, num_constraints: &impl ToBigUint) -> Polynomial {
    let num_constraints = f.elem(num_constraints);
    let mut i = f.elem(&1u8);
    let mut polys = vec![];

    // create (x-i) polynomials
    while i <= num_constraints {
      let poly = Polynomial::new(f, &vec![
        -f.elem(&i),
        f.elem(&1u8),
      ]);
      polys.push(poly);
      i.inc();
    }
    // aggregate (x-i) polynomial into a single polynomial
    let mut acc_poly = Polynomial::new(&f, &vec![f.elem(&1u8)]);
    for poly in polys {
      acc_poly = acc_poly.mul(&poly);
    }
    acc_poly
  }

  pub fn build(f: &PrimeField, r1cs: &R1CS) -> QAP {
    /*
              c1 c2 c3 (coeffs for a1, a2, a3)
       for w=( 1, 2, 3),
       at x=1, 3 * 1 (w[3] * a3)
       at x=2, 2 * 3 (w[1] * a1)
       at x=3, 0
       at x=4, 2 * 2 (w[2] * a2)
    */

    //       w1 w2 w3 <- witness e.g. w=(x,y,z,w1,...)
    //  x=1 | 0  0  1 |
    //  x=2 | 3  0  0 |
    //  x=3 | 0  0  0 |
    //  x=4 | 0  2  0 |
    //   ^
    //   +-- constraints
    let constraints = r1cs.to_constraint_matrices();

    //   x=1 2 3 4  <- constraints
    // w1 [0 3 0 0]
    // w2 [0 0 0 2]
    // w3 [1 0 0 0]
    //  ^
    //  +-- witness
    let constraints_v_t = constraints.a.transpose();
    let constraints_w_t = constraints.b.transpose();
    let constraints_y_t = constraints.c.transpose();
    println!("- const v_t\n{:?}", &constraints_v_t);
    println!("- const w_t\n{:?}", &constraints_w_t);
    println!("- const y_t\n{:?}", &constraints_y_t);

    // build polynomials for each wirness variable
    // e.g. vi[0] is a polynomial for the first witness variable
    // and returns 3 at x=2 and 0 at all other x's
    let mut vi = vec![];
    let mut wi = vec![];
    let mut yi = vec![];

    let mut i = f.elem(&0u8);

    let num_witness_values = &r1cs.witness.size;

    while &i < num_witness_values {
      // extract a constraint row
      //   x = 1 2 3 4 
      // wi = [0 3 0 0]
      let v_row = constraints_v_t.get_row(&i);
      let w_row = constraints_w_t.get_row(&i);
      let y_row = constraints_y_t.get_row(&i);

      // convert a constraint row to a polynomial
      let v_poly = QAP::build_polynomial(f, &v_row);
      let w_poly = QAP::build_polynomial(f, &w_row);
      let y_poly = QAP::build_polynomial(f, &y_row);
      
      vi.push(v_poly);
      wi.push(w_poly);
      yi.push(y_poly);

      i.inc();
    }

    let num_constraints = constraints.a.height.e.clone();

    QAP { f: f.clone(), vi, wi, yi, num_constraints }
  }

  pub fn is_valid(
    &self,
    witness: &SparseVec,
    num_constraints: &impl ToBigUint,
  ) -> bool {
    let t = QAP::build_t(&self.f, num_constraints);
    let p = self.build_p(witness);

    match p.divide_by(&t) {
      DivResult::Quotient(_) => true,
      DivResult::QuotientRemainder(_) => false,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    building_block::field::prime_field::PrimeField,
    zk::w_trusted_setup::pinocchio::constraint::Constraint,
  };

  #[test]
  fn test_r1cs_to_polynomial() {
    let f = &PrimeField::new(&3911u16);

    //     x  out t1  y  t2
    //  0  1   2  3   4   5
    // [1, 3, 35, 9, 27, 30]
    let witness = SparseVec::from(&vec![
      f.elem(&1u8),
      f.elem(&3u8),
      f.elem(&35u8),
      f.elem(&9u8),
      f.elem(&27u8),
      f.elem(&30u8),
    ]);
    let witness_size = &witness.size;

    // A
    //  0  1  2  3  4  5
    // [0, 1, 0, 0, 0, 0]
    // [0, 0, 0, 1, 0, 0]
    // [0, 1, 0, 0, 1, 0]
    // [5, 0, 0, 0, 0, 1]
    let mut a1 = SparseVec::new(f, witness_size);
    a1.set(&1u8, &1u8);

    let mut a2 = SparseVec::new(f, witness_size);
    a2.set(&3u8, &1u8);

    let mut a3 = SparseVec::new(f, witness_size);
    a3.set(&1u8, &1u8);
    a3.set(&4u8, &1u8);

    let mut a4 = SparseVec::new(f, witness_size);
    a4.set(&0u8, &5u8);
    a4.set(&5u8, &1u8);

    // B
    //  0  1  2  3  4  5
    // [0, 1, 0, 0, 0, 0]
    // [0, 1, 0, 0, 0, 0]
    // [1, 0, 0, 0, 0, 0]
    // [1, 0, 0, 0, 0, 0]
    let mut b1 = SparseVec::new(f, witness_size);
    b1.set(&1u8, &1u8);

    let mut b2 = SparseVec::new(f, witness_size);
    b2.set(&1u8, &1u8);

    let mut b3 = SparseVec::new(f, witness_size);
    b3.set(&0u8, &1u8);

    let mut b4 = SparseVec::new(f, witness_size);
    b4.set(&0u8, &1u8);

    // C
    //  0  1  2  3  4  5
    // [0, 0, 0, 1, 0, 0]
    // [0, 0, 0, 0, 1, 0]
    // [0, 0, 0, 0, 0, 1]
    // [0, 0, 1, 0, 0, 0]
    let mut c1 = SparseVec::new(f, witness_size);
    c1.set(&3u8, &1u8);

    let mut c2 = SparseVec::new(f, witness_size);
    c2.set(&4u8, &1u8);

    let mut c3 = SparseVec::new(f, witness_size);
    c3.set(&5u8, &1u8);

    let mut c4 = SparseVec::new(f, witness_size);
    c4.set(&2u8, &1u8);
    let constraints = vec![
      Constraint::new(&a1, &b1, &c1),
      Constraint::new(&a2, &b2, &c2),
      Constraint::new(&a3, &b3, &c3),
      Constraint::new(&a4, &b4, &c4),
    ];
    let num_constraints = &constraints.len();
    let mid_beg = f.elem(&3u8);
    let r1cs = R1CS {
      constraints,
      witness: witness.clone(),
      mid_beg,
    };

    let qap = QAP::build(f, &r1cs);
    let is_passed = qap.is_valid(&witness, num_constraints);
    assert!(is_passed);
  }

  #[test]
  fn test_build_t() {
    let f = &PrimeField::new(&3911u16);

    let one = &f.elem(&1u8);
    let two = &f.elem(&2u8);
    let neg_three = &f.elem(&3u8).negate();

    let z = QAP::build_t(f, two);

    assert_eq!(z.len(), 3);
    assert_eq!(&z[0], two);
    assert_eq!(&z[1], neg_three);
    assert_eq!(&z[2], one);
  }
}
