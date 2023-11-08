use std::ops::Mul;

use crate::building_block::curves::mcl::{
  mcl_fr::MclFr,
  mcl_sparse_vec::MclSparseVec,
  polynomial::{
    DivResult,
    Polynomial,
  },
  qap::r1cs::R1CS,
};
use num_traits::Zero;

#[derive(Clone)]
pub struct QAP {
  pub vi: Vec<Polynomial>,
  pub wi: Vec<Polynomial>,
  pub yi: Vec<Polynomial>,
  pub num_constraints: MclFr,
}

impl QAP {
  // build a polynomial that evaluates to target_val at x == index
  // and zero for x != index for each target value.
  // e.g.
  // (x - 2) * (x - 3) * 3 / ((1 - 2) * (1 - 3))
  // where x in [1, 2, 3]; evaluates to 3 if x == 1 and 0 if x != 1
  fn build_polynomial(target_vals: &MclSparseVec) -> Polynomial {
    let mut target_val_polys = vec![];

    let one = MclFr::from(1);
    let mut target_x = MclFr::from(1);
    while target_x <= target_vals.size {
      let target_val = target_vals.get(&(&target_x - &one));

      // if target val is zero, simply add 0x^0
      if target_val.is_zero() {
        target_val_polys.push(Polynomial::new(&vec![MclFr::from(0)]));
        target_x.inc();
        continue;
      }

      let mut numerator_polys = vec![
        Polynomial::new(&vec![target_val.clone()]),
      ];
      let mut denominator = MclFr::from(1);

      let mut i = MclFr::from(1);
      while &i <= &target_vals.size {
        if &i == &target_x {
          i.inc();
          continue;
        }
        // (x - i) to let the polynomal evaluate to zero at x = i
        let numerator_poly = Polynomial::new(&vec![
          -&i,
          MclFr::from(1),
        ]);
        numerator_polys.push(numerator_poly);

        // (target_idx - i) to cancel out the corresponding
        // numerator_poly at x = target_idx
        denominator = &denominator * (&target_x - &i);

        i.inc();
      }

      // merge denominator polynomial to numerator polynomial vector
      let denominator_poly = Polynomial::new(&vec![denominator.inv()]);
      let mut polys = numerator_polys;
      polys.push(denominator_poly);

      // aggregate numerator polynomial vector
      let mut acc_poly = Polynomial::new(&vec![MclFr::from(1)]);
      for poly in polys {
        acc_poly = acc_poly.mul(&poly);
      }
      target_val_polys.push(acc_poly);

      target_x.inc();
    }

    // aggregate polynomials for all target values
    let mut res = target_val_polys[0].clone();
    for x in &target_val_polys[1..] {
      res += x;
    }
    res
  }

  pub fn build_p(&self, witness: &MclSparseVec) -> Polynomial {
    let zero = &Polynomial::zero();
    let (mut v, mut w, mut y) =
      (zero.clone(), zero.clone(), zero.clone());

    for i in 0..witness.size.to_usize() {
      let wit = &witness[&MclFr::from(i)];
      v += &self.vi[i] * wit;
      w += &self.wi[i] * wit;
      y += &self.yi[i] * wit;
    };
    
    (v * &w) - &y
  }

  // build polynomial (x-1)(x-2)..(x-num_constraints)
  pub fn build_t(num_constraints: &MclFr) -> Polynomial {
    let mut i = MclFr::from(1);
    let mut polys = vec![];

    // create (x-i) polynomials
    while &i <= &num_constraints {
      let poly = Polynomial::new(&vec![
        -&i,
        MclFr::from(1),
      ]);
      polys.push(poly);
      i.inc();
    }
    // aggregate (x-i) polynomial into a single polynomial
    let mut acc_poly = Polynomial::new(&vec![MclFr::from(1)]);
    for poly in polys {
      acc_poly = acc_poly.mul(&poly);
    }
    acc_poly
  }

  pub fn build(r1cs: &R1CS) -> QAP {
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

    let mut i = MclFr::from(0);

    let num_witness_values = &r1cs.witness.size;

    while &i < num_witness_values {
      // extract a constraint row
      //   x = 1 2 3 4 
      // wi = [0 3 0 0]
      let v_row = constraints_v_t.get_row(&i);
      let w_row = constraints_w_t.get_row(&i);
      let y_row = constraints_y_t.get_row(&i);

      // convert a constraint row to a polynomial
      let v_poly = QAP::build_polynomial(&v_row);
      let w_poly = QAP::build_polynomial(&w_row);
      let y_poly = QAP::build_polynomial(&y_row);
      
      vi.push(v_poly);
      wi.push(w_poly);
      yi.push(y_poly);

      i.inc();
    }

    let num_constraints = constraints.a.height.clone();

    QAP { vi, wi, yi, num_constraints }
  }

  pub fn is_valid(
    &self,
    witness: &MclSparseVec,
    num_constraints: &MclFr,
  ) -> bool {
    let t = QAP::build_t(num_constraints);
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
  use crate::building_block::curves::mcl::{
    mcl_fr::MclFr,
    mcl_initializer::MclInitializer,
    qap::constraint::Constraint,
  };

  #[test]
  fn test_r1cs_to_polynomial() {
    MclInitializer::init();
    //     x  out t1  y  t2
    //  0  1   2  3   4   5
    // [1, 3, 35, 9, 27, 30]
    let witness = MclSparseVec::from(&vec![
      MclFr::from(1),
      MclFr::from(3),
      MclFr::from(35),
      MclFr::from(9),
      MclFr::from(27),
      MclFr::from(30),
    ]);
    let witness_size = &witness.size;

    // A
    //  0  1  2  3  4  5
    // [0, 1, 0, 0, 0, 0]
    // [0, 0, 0, 1, 0, 0]
    // [0, 1, 0, 0, 1, 0]
    // [5, 0, 0, 0, 0, 1]
    let mut a1 = MclSparseVec::new(witness_size);
    a1.set(&MclFr::from(1), &MclFr::from(1));

    let mut a2 = MclSparseVec::new(witness_size);
    a2.set(&MclFr::from(3), &MclFr::from(1));

    let mut a3 = MclSparseVec::new(witness_size);
    a3.set(&MclFr::from(1), &MclFr::from(1));
    a3.set(&MclFr::from(4), &MclFr::from(1));

    let mut a4 = MclSparseVec::new(witness_size);
    a4.set(&MclFr::from(0), &MclFr::from(5));
    a4.set(&MclFr::from(5), &MclFr::from(1));

    // B
    //  0  1  2  3  4  5
    // [0, 1, 0, 0, 0, 0]
    // [0, 1, 0, 0, 0, 0]
    // [1, 0, 0, 0, 0, 0]
    // [1, 0, 0, 0, 0, 0]
    let mut b1 = MclSparseVec::new(witness_size);
    b1.set(&MclFr::from(1), &MclFr::from(1));

    let mut b2 = MclSparseVec::new(witness_size);
    b2.set(&MclFr::from(1), &MclFr::from(1));

    let mut b3 = MclSparseVec::new(witness_size);
    b3.set(&MclFr::from(0), &MclFr::from(1));

    let mut b4 = MclSparseVec::new(witness_size);
    b4.set(&MclFr::from(0), &MclFr::from(1));

    // C
    //  0  1  2  3  4  5
    // [0, 0, 0, 1, 0, 0]
    // [0, 0, 0, 0, 1, 0]
    // [0, 0, 0, 0, 0, 1]
    // [0, 0, 1, 0, 0, 0]
    let mut c1 = MclSparseVec::new(witness_size);
    c1.set(&MclFr::from(3), &MclFr::from(1));

    let mut c2 = MclSparseVec::new(witness_size);
    c2.set(&MclFr::from(4), &MclFr::from(1));

    let mut c3 = MclSparseVec::new(witness_size);
    c3.set(&MclFr::from(5), &MclFr::from(1));

    let mut c4 = MclSparseVec::new(witness_size);
    c4.set(&MclFr::from(2), &MclFr::from(1));
    let constraints = vec![
      Constraint::new(&a1, &b1, &c1),
      Constraint::new(&a2, &b2, &c2),
      Constraint::new(&a3, &b3, &c3),
      Constraint::new(&a4, &b4, &c4),
    ];
    let num_constraints = &MclFr::from(constraints.len());
    let mid_beg = MclFr::from(3);
    let r1cs = R1CS {
      constraints,
      witness: witness.clone(),
      mid_beg,
    };

    let qap = QAP::build(&r1cs);
    let is_passed = qap.is_valid(&witness, num_constraints);
    assert!(is_passed);
  }

  #[test]
  fn test_build_t() {
    MclInitializer::init();
    let one = &MclFr::from(1);
    let two = &MclFr::from(2);
    let neg_three = &-MclFr::from(3);

    // (x-1)(x-2) = x^2 - 3x + 2
    let z = QAP::build_t(two);

    // expect [2, -3, 1] 
    assert_eq!(z.len(), 3);
    assert_eq!(&z[0], two);
    assert_eq!(&z[1], neg_three);
    assert_eq!(&z[2], one);
  }
}
