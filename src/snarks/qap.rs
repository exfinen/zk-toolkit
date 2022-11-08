use crate::building_block::field::Field;
use crate::snarks::{
  r1cs::R1CS,
  polynomial::Polynomial,
  sparse_vec::SparseVec,
  sparse_matrix::SparseMatrix,
};

pub struct QAP {
  pub f: Field,
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
    f: &Field,
    target_vals: &SparseVec,
  ) -> Polynomial {
    let mut target_val_polys = vec![];

    let one = f.elem(&1u8);
    let mut target_x = f.elem(&1u8);
    while target_x <= target_vals.size {
      let target_val = target_vals.get(&(&target_x - &one));
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
      res = res.add(x);
    }
    res
  }

  pub fn build(f: &Field, r1cs: R1CS) -> QAP {
    /*
                      a^t
           a         a1 a2
    a1 [0 3 0 0] ->  |0 0|
    a2 [0 0 0 2]     |3 0| <- need polynomial that returns
    +------^         |0 0|    3 at x=1 and 0 at x=2
    r1cs selector *  |0 2| <- here polynomial that retuns
    witness         x=1 x=2   0 at x=1 and 2 at x=2
                    x-th col corresponds to x-th constraint
    */
    let r1cs = r1cs.to_constraint_by_witness_matrices();
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
println!("a_coeffs");
for x in &a_coeffs {
  println!("{}", x.pretty_print());
}
    let a_polys = SparseMatrix::from(&a_coeffs);
    let b_polys = SparseMatrix::from(&b_coeffs);
    let c_polys = SparseMatrix::from(&c_coeffs);

    QAP { f: f.clone(), a_polys, b_polys, c_polys }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::snarks::{
    sparse_vec::SparseVec,
    constraint::Constraint, sparse_matrix::SparseMatrix,
  };

  #[test]
  fn test_r1cs_to_polynomial() {
    let f = &Field::new(&3911u16);

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
    let r1cs = R1CS { constraints, witness: witness.clone() };
    let qap = QAP::build(f, r1cs);

    // check A
    {
      let exp = SparseMatrix::from(&vec![
        &a1 * &witness,
        &a2 * &witness,
        &a3 * &witness,
        &a4 * &witness,
      ]).transpose();

      let act = qap.a_polys.row_transform(Box::new(|vec| {
        let p = Polynomial::from(vec);
        p.eval_from_1_to_n(&vec.size)
      }));
      println!("exp:\n{}", exp.pretty_print());
      println!("act:\n{}", act.pretty_print());
      assert!(exp == act);
    }

    // check B
    {
      let exp = SparseMatrix::from(&vec![
        &b1 * &witness,
        &b2 * &witness,
        &b3 * &witness,
        &b4 * &witness,
      ]).transpose();

      let act = qap.b_polys.row_transform(Box::new(|vec| {
        let p = Polynomial::from(vec);
        p.eval_from_1_to_n(&vec.size)
      }));
      println!("exp:\n{}", exp.pretty_print());
      println!("act:\n{}", act.pretty_print());
      assert!(exp == act);
    }

    // check C
    {
      let exp = SparseMatrix::from(&vec![
        &c1 * &witness,
        &c2 * &witness,
        &c3 * &witness,
        &c4 * &witness,
      ]).transpose();
      let act = qap.c_polys.row_transform(Box::new(|vec| {
        let p = Polynomial::from(vec);
        p.eval_from_1_to_n(&vec.size)
      }));
      println!("exp:\n{}", exp.pretty_print());
      println!("act:\n{}", act.pretty_print());
      assert!(exp == act);
    }
  }
}