use crate::building_block::field::{Field, FieldElem};
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
  fn build_polynomial_for_target_values(
    f: &Field,
    target_vals: Vec<FieldElem>,
  ) -> Polynomial {
    let target_val_polys = vec![];

    for (target_idx, target_val) in target_vals.into_iter().enumerate() {

      let numerator_polys = vec![
        Polynomial::new(f, vec![target_val]),
      ];
      let denominator = f.elem(&1u8);

      for i in 0..target_vals.len() {
        if i == target_idx {
          continue;
        }
        // (x - i) to m`ake the polynomal zero at x = i
        let numerator_poly = Polynomial::new(f, vec![
          f.elem(&i),
          f.elem(&1u8),
        ]);
        numerator_polys.push(numerator_poly);

        // (target_idx - i) to cancel out the corresponding
        // numerator_poly at x = target_idx
        denominator = denominator * (target_val - f.elem(&i));
      }
      // merge numerator and denominator
      let denominator_poly = Polynomial::new(f, vec![denominator.inv()]);
      let polys = numerator_polys;
      polys.push(denominator_poly);

      // aggregate polynomials
      let acc_poly = Polynomial::new(f, vec![f.elem(&1u8)]);
      for poly in polys {
        acc_poly = acc_poly.mul(&poly);
      }
      target_val_polys.push(acc_poly);
    }
    // aggregate polynomials for all target values
    let mut res = target_val_polys[0];
    for x in &target_val_polys[1..] {
      res = res.add(x);
    }
    res
  }

  pub fn build(f: &Field, r1cs: R1CS) -> QAP {
    let num_gates = r1cs.constraints.len();

    // columns
    let mut qap_a_cols = vec![];
    let mut qap_b_cols = vec![];
    let mut qap_c_cols = vec![];

    // vertical
    for row in 0..r1cs.witness.size {
      // row selects a witness element

      let target_vals = r1cs.constraints.iter().map(|constraint| {
        (&constraint.a * &r1cs.witness).get(&row)
      }).collect();
      qap_a_cols.push(QAP::build_polynomial_for_target_values(f, target_vals));

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