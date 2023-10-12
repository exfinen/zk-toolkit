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
  sparse_matrix::SparseMatrix,
};
use num_traits::Zero;

pub enum ApplyWitness {
  Beginning,
  End,
}

pub struct QAP {
  pub f: PrimeField,
  pub a_polys: SparseMatrix,
  pub b_polys: SparseMatrix,
  pub c_polys: SparseMatrix,
}

impl QAP {
  // build a polynomial that evaluates to target_val at x == index
  // and zero for x != index.
  // e.g.
  // (x - 2) * (x - 3) * 3 / ((1 - 2) * (1 - 3))
  // where x in [1, 2, 3]; evaluates to 3 if x == 1 and 0 if x != 1
  fn build_polynomial_for_target_values(
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
        target_val_polys.push(Polynomial::new(f, vec![f.elem(&0u8)]));
        target_x.inc();
        continue;
      }

      let mut numerator_polys = vec![
        Polynomial::new(f, vec![target_val.clone()]),
      ];
      let mut denominator = f.elem(&1u8);

      let mut i = f.elem(&1u8);
      while i <= target_vals.size {
        if i == target_x {
          i.inc();
          continue;
        }
        // (x - i) to let the polynomal evaluate to zero at x = i
        let numerator_poly = Polynomial::new(f, vec![
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
      let denominator_poly = Polynomial::new(f, vec![denominator.inv()]);
      let mut polys = numerator_polys;
      polys.push(denominator_poly);

      // aggregate numerator polynomial vector
      let mut acc_poly = Polynomial::new(f, vec![f.elem(&1u8)]);
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

  pub fn build(
    f: &PrimeField,
    r1cs: &R1CS,
    apply_witness: &ApplyWitness,
  ) -> QAP {
    /*
                      a^t
           a         a1 a2
    a1 [0 3 0 0] ->  |0 0|
    a2 [0 0 0 2]     |3 0| <- need polynomial that returns
    +------^         |0 0|    3 at x=1 and 0 at x=2
    r1cs selector *  |0 2| <- here polynomial that returns
    witness         x=1 x=2   0 at x=1 and 2 at x=2
                    x-th col corresponds to x-th constraint
    */
    let r1cs = match apply_witness {
      ApplyWitness::Beginning => r1cs.to_constraint_by_witness_matrices(),
      ApplyWitness::End => r1cs.to_constraint_matrices(),
    };
    let a_t = r1cs.a.transpose();
    let b_t = r1cs.b.transpose();
    let c_t = r1cs.c.transpose();

    let mut a_coeffs: Vec<SparseVec> = vec![];
    let mut b_coeffs: Vec<SparseVec> = vec![];
    let mut c_coeffs: Vec<SparseVec> = vec![];

    let mut y = f.elem(&0u8);
    let height = &a_t.height;  // a_t, b_t and c_t are of the same dimention
    let width = &a_t.width;

    while &y < height {
      let a_row = a_t.get_row(&y);
      let b_row = b_t.get_row(&y);
      let c_row = c_t.get_row(&y);

      a_coeffs.push(QAP::build_polynomial_for_target_values(f, &a_row).to_sparse_vec(width));
      b_coeffs.push(QAP::build_polynomial_for_target_values(f, &b_row).to_sparse_vec(width));
      c_coeffs.push(QAP::build_polynomial_for_target_values(f, &c_row).to_sparse_vec(width));

      y.inc();
    }
    let a_polys = SparseMatrix::from(&a_coeffs);
    let b_polys = SparseMatrix::from(&b_coeffs);
    let c_polys = SparseMatrix::from(&c_coeffs);

    QAP { f: f.clone(), a_polys, b_polys, c_polys }
  }

  // build polynomial (x-1)(x-2)..(x-num_constraints)
  pub fn build_t(f: &PrimeField, num_constraints: &impl ToBigUint) -> Polynomial {
    let num_constraints = f.elem(num_constraints);
    let mut i = f.elem(&1u8);
    let mut polys = vec![];

    // create (x-i) polynomials
    while i <= num_constraints {
      let poly = Polynomial::new(f, vec![
        -f.elem(&i),
        f.elem(&1u8),
      ]);
      polys.push(poly);
      i.inc();
    }
    // aggregate (x-i) polynomial into a single polynomial
    let mut acc_poly = Polynomial::new(f, vec![f.elem(&1u8)]);
    for poly in polys {
      acc_poly = acc_poly.mul(&poly);
    }
    acc_poly
  }

  pub fn check_constraints(
    &self,
    witness: &SparseVec,
    num_constraints: &impl ToBigUint,
    apply_witness: &ApplyWitness,
  ) -> bool {
    // aggregate polynomials by calculating dot products with witness
    let a_poly: Polynomial = match apply_witness {
      ApplyWitness::Beginning => (&self.a_polys.flatten_rows()).into(),
      ApplyWitness::End => (&self.a_polys.multiply_column(witness).flatten_rows()).into(),
    };
    let b_poly: Polynomial = match apply_witness {
      ApplyWitness::Beginning => (&self.b_polys.flatten_rows()).into(),
      ApplyWitness::End => (&self.b_polys.multiply_column(witness).flatten_rows()).into(),
    };
    let c_poly: Polynomial = match apply_witness {
      ApplyWitness::Beginning => (&self.c_polys.flatten_rows()).into(),
      ApplyWitness::End => (&self.c_polys.multiply_column(witness).flatten_rows()).into(),
    };

    let t = a_poly * &b_poly - &c_poly;
    let num_constraints = self.f.elem(num_constraints);
    let z = QAP::build_t(&self.f, &num_constraints);
    match t.divide_by(&z) {
      DivResult::Quotient(_) => true,
      DivResult::QuotientRemainder(_) => false,
    }
  }

  pub fn get_flattened_polys(&self) -> (Polynomial, Polynomial, Polynomial) {
    let a_poly: Polynomial = (&self.a_polys.flatten_rows()).into();
    let b_poly: Polynomial = (&self.b_polys.flatten_rows()).into();
    let c_poly: Polynomial = (&self.c_polys.flatten_rows()).into();
    (a_poly, b_poly, c_poly)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    building_block::field::{
      prime_field::PrimeField,
      prime_field_elem::PrimeFieldElem,
    },
    zk::w_trusted_setup::pinocchio::{
      constraint::Constraint,
      gate::Gate,
      equation_parser::Parser,
      r1cs_tmpl::R1CSTmpl,
      sparse_vec::SparseVec,
      term::Term,
    }
  };
  use std::collections::HashMap;

  #[test]
  fn test_r1cs_to_polynomial() {
    let f = &PrimeField::new(&3911u16);

    //     x  out t1 y   t2
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
    let r1cs = R1CS { constraints, witness: witness.clone() };

    for apply_witness in vec![ApplyWitness::Beginning, ApplyWitness::End] {
      let qap = QAP::build(f, &r1cs, &apply_witness);
      let is_passed = qap.check_constraints(&witness, num_constraints, &apply_witness);
      assert!(is_passed);
    }
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

  #[test]
  fn blog_post_1_example_1() {
    let f = &PrimeField::new(&37u8);
    let expr = "(x * x * x) + x + 5 == 35";
    let eq = Parser::parse(f, expr).unwrap();
    let gates = &Gate::build(f, &eq);
    let r1cs_tmpl = R1CSTmpl::from_gates(f, gates);

    // build witness
    /*
      x = 3
      t1 = x(3) * x(3) = 9
      t2 = t1(9) * x(3) = 27
      t3 = x(3) + 5 = 8
      t4 = t2(27) + t2(8) = 35
      out = t4
    */
    let witness = {
      use crate::zk::w_trusted_setup::pinocchio::term::Term::*;
      HashMap::<Term, PrimeFieldElem>::from([
        (Term::var("x"), f.elem(&3u8)),
        (TmpVar(1), f.elem(&9u8)),
        (TmpVar(2), f.elem(&27u8)),
        (TmpVar(3), f.elem(&8u8)),
        (TmpVar(4), f.elem(&35u8)),
        (Out, eq.rhs),
      ])
    };

    let r1cs = R1CS::from_tmpl(f, &r1cs_tmpl, &witness).unwrap();
    let qap = QAP::build(f, &r1cs, &ApplyWitness::Beginning);

    println!("a:\n{}", qap.a_polys.pretty_print());
    println!("b:\n{}", qap.b_polys.pretty_print());
    println!("c:\n{}", qap.c_polys.pretty_print());
  }

  #[test]
  fn blog_post_1_example_2() {
    let f = &PrimeField::new(&37u8);
    let expr = "(x * x * x) + x + 5 == 35";
    let eq = Parser::parse(f, expr).unwrap();
    let gates = &Gate::build(f, &eq);
    let r1cs_tmpl = R1CSTmpl::from_gates(f, gates);

    // build witness
    let good_witness = {
      use crate::zk::w_trusted_setup::pinocchio::term::Term::*;
      HashMap::<Term, PrimeFieldElem>::from([
        (Term::var("x"), f.elem(&3u8)),
        (TmpVar(1), f.elem(&9u8)),
        (TmpVar(2), f.elem(&27u8)),
        (TmpVar(3), f.elem(&8u8)),
        (TmpVar(4), f.elem(&35u8)),
        (Out, eq.rhs.clone()),
      ])
    };
    let bad_witness = {
      use crate::zk::w_trusted_setup::pinocchio::term::Term::*;
      HashMap::<Term, PrimeFieldElem>::from([
        (Term::var("x"), f.elem(&4u8)),  // replaced 3 with 4
        (TmpVar(1), f.elem(&9u8)),
        (TmpVar(2), f.elem(&27u8)),
        (TmpVar(3), f.elem(&8u8)),
        (TmpVar(4), f.elem(&35u8)),
        (Out, eq.rhs),
      ])
    };

    for test_case in vec![("good", good_witness), ("bad", bad_witness)] {
      let (name, witness) = test_case;
      let r1cs = R1CS::from_tmpl(f, &r1cs_tmpl, &witness).unwrap();

      let qap = QAP::build(f, &r1cs, &ApplyWitness::Beginning);
      let (a, b, c) = qap.get_flattened_polys();
      let t = a * &b - &c;

      let num_constraints = f.elem(&gates.len());
      let z = QAP::build_t(f, &num_constraints);

      let is_witness_valid = match t.divide_by(&z) {
        DivResult::Quotient(_) => true,
        DivResult::QuotientRemainder(_) => false,
      };
      println!("is {} witness valid? -> {}", name, is_witness_valid);
    }
  }
}
