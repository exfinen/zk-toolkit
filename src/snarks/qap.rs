use crate::building_block::field::Field;
use crate::snarks::{
  r1cs::R1CS,
  polynomial::Polynomial,
  sparse_vec::SparseVec,
};

pub struct QAP {
  pub f: Field,
  pub polys: Vec<Polynomial>,
}

impl QAP {
  fn lagrange_interpolation() {
    // first need to know:
    // 1. # of points
    // 2. expected value for each point
  }

  // build a polynomial that returns target_val at x == index
  // and zero for x != index.
  // e.g. (x - 2) * (x - 3) * 3 / ((1 - 2) * (1 - 3))
  fn build_polynomial_with_lagrange(
    f: &Field,
    constraint_vec: &SparseVec,
    constraint_idx: usize,
    num_constraints: usize,
    witness: &SparseVec,
  ) -> Polynomial {
    // get the target value
    let target_val = (constraint_vec * witness).sum();

    let numerator_polys = vec![
      Polynomial::new(f, vec![target_val]),
    ];
    let denominator = f.elem(&1u8);

    for i in 0..num_constraints {
      if i == constraint_idx {
        continue;
      }
      let numerator_poly = Polynomial::new(f, vec![
        f.elem(&i),
        f.elem(&1u8),
      ]);
      numerator_polys.push(numerator_poly);

      denominator = denominator * (target_val - f.elem(&i));
    }
    let denominator_poly = Polynomial::new(f, vec![denominator.inv()]);

    let polys = numerator_polys;
    polys.push(denominator_poly);

    // aggregate polynomials
    let acc_poly = Polynomial::new(f, vec![f.elem(&1u8)]);
    for poly in polys {
      acc_poly = acc_poly.mul(&poly);
    }
    acc_poly
  }

  pub fn build(f: &Field, r1cs: R1CS) -> QAP {
    let num_gates = r1cs.constraints.len();

    // columns
    let mut qap_a_cols = vec![];
    let mut qap_b_cols = vec![];
    let mut qap_c_cols = vec![];

    // there will be a polynomial for each witness for a, b, c
    // 1 constrant to 6 rows of a,b,c
    for row in 0..r1cs.witness.size {

    }
    for (col_idx, constraint) in r1cs.constraints.iter().enumerate() {
      qap_a_cols.push(QAP::build_polynomial_with_lagrange(f, &constraint.a, col_idx, r1cs.constraints.len(), &r1cs.witness));
      qap_b_cols.push(QAP::build_polynomial_with_lagrange(f, &constraint.b, col_idx, r1cs.constraints.len(), &r1cs.witness));
      qap_c_cols.push(QAP::build_polynomial_with_lagrange(f, &constraint.c, col_idx, r1cs.constraints.len(), &r1cs.witness));
    }

    // flatten qap_polys

    // a has witness # of polys

    QAP { polys: vec![] }
  }
}

#[cfg(test)]
mod tests {
  // use super::*;
  // use crate::snarks::equation_parser::Parser;

  /*
  Witness
  [1, 3, 35, 9, 27, 30]

  A
  [0, 1, 0, 0, 0, 0]
  [0, 0, 0, 1, 0, 0]
  [0, 1, 0, 0, 1, 0]
  [5, 0, 0, 0, 0, 1]
  B
  [0, 1, 0, 0, 0, 0]
  [0, 1, 0, 0, 0, 0]
  [1, 0, 0, 0, 0, 0]
  [1, 0, 0, 0, 0, 0]
  C
  [0, 0, 0, 1, 0, 0]
  [0, 0, 0, 0, 1, 0]
  [0, 0, 0, 0, 0, 1]
  [0, 0, 1, 0, 0, 0]
  */

  #[test]
  fn test1() {
  }
}