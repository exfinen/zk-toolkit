use crate::building_block::field::{Field, FieldElem};
use crate::snarks::{
  r1cs::R1CS,
  polynomial::Polynomial,
};

pub struct QAP {
  pub f: Field,
  pub a_polys: Vec<Polynomial>,
  pub b_polys: Vec<Polynomial>,
  pub c_polys: Vec<Polynomial>,
}

impl QAP {
  // build a polynomial that returns target_val at x == index
  // and zero for x != index.
  // e.g. (x - 2) * (x - 3) * 3 / ((1 - 2) * (1 - 3))
  fn build_polynomial_for_target_values(
    f: &Field,
    target_vals: Vec<FieldElem>,
  ) -> Polynomial {
    let mut target_val_polys = vec![];
    let num_target_vals = target_vals.len();
    for (target_idx, target_val) in target_vals.iter().enumerate() {

      let mut numerator_polys = vec![
        Polynomial::new(f, vec![target_val.clone()]),
      ];
      let mut denominator = f.elem(&1u8);

      for i in 0..num_target_vals {
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
      let mut polys = numerator_polys;
      polys.push(denominator_poly);

      // aggregate polynomials
      let mut acc_poly = Polynomial::new(f, vec![f.elem(&1u8)]);
      for poly in polys {
        acc_poly = acc_poly.mul(&poly);
      }
      target_val_polys.push(acc_poly);
    }
    // aggregate polynomials for all target values
    let mut res = target_val_polys[0].clone();
    for x in &target_val_polys[1..] {
      res = res.add(x);
    }
    res
  }

  pub fn build(f: &Field, r1cs: R1CS) -> QAP {
    let mut a_polys = vec![];
    let mut b_polys = vec![];
    let mut c_polys = vec![];

    /*
                    a1 a2
    a1 [0 3 0 0] -> |0 0|
    a2 [0 0 0 2]    |3 0| <- need polynomial that returns
    +------^        |0 0|    3 at x=1 and 0 at x=2
    r1cs selector * |0 2| <- here polynomail that retuns
    witness        x=1 x=2   0 at x=1 and 2 at x=2
    */
    for row in 0..r1cs.witness.size {
      let a_target_vals = r1cs.constraints.iter().map(|constraint| {
        (&constraint.a * &r1cs.witness).get(&row)
      }).collect();
      let b_target_vals = r1cs.constraints.iter().map(|constraint| {
        (&constraint.b * &r1cs.witness).get(&row)
      }).collect();
      let c_target_vals = r1cs.constraints.iter().map(|constraint| {
        (&constraint.c * &r1cs.witness).get(&row)
      }).collect();

      a_polys.push(QAP::build_polynomial_for_target_values(f, a_target_vals));
      b_polys.push(QAP::build_polynomial_for_target_values(f, b_target_vals));
      c_polys.push(QAP::build_polynomial_for_target_values(f, c_target_vals));
    }

    QAP { f: f.clone(), a_polys, b_polys, c_polys }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::snarks::{
    r1cs_tmpl::R1CSTmpl,
    term::Term,
  };
  use std::collections::HashMap;

  /*
  Witness
      x  out t1 y   t2
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
    let f = &Field::new(&3911u16);

    let term_values = HashMap::<Term, FieldElem>::from([
      (Term::Var("x".to_string()), f.elem(&3u8)),
      (Term::Out, f.elem(&35u8)),
      (Term::TmpVar(1), f.elem(&9u8)),
      (Term::Var("y".to_string()), f.elem(&27u8)),
      (Term::TmpVar(2), f.elem(&30u8)),
    ]);

    let mut tmpl = R1CSTmpl::new(f);
    for term in term_values.keys() {
      tmpl.add_term(term);
    }

    let r1cs = R1CS::new(f, &tmpl, &term_values).unwrap();
    let _qap = QAP::build(f, r1cs);
  }
}