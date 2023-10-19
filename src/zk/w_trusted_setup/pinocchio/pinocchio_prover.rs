use crate::{
  building_block::{
    curves::bls12_381::{
      g1_point::G1Point,
      g2_point::G2Point,
    },
    field::{
      prime_field::PrimeField,
      prime_field_elem::PrimeFieldElem,
    },
    zero::Zero,
  },
  zk::w_trusted_setup::pinocchio::{
    crs::CRS,
    equation_parser::EquationParser,
    gate::Gate,
    qap::QAP,
    polynomial::{Polynomial, DivResult},
    pinocchio_proof::PinocchioProof,
    r1cs::R1CS,
    r1cs_tmpl::R1CSTmpl,
    term::Term,
    witness::Witness,
  },
};
use std::{
  collections::HashMap,
  cmp,
};

pub struct PinocchioProver {
  pub f: PrimeField,
  pub max_degree: usize,  // TODO use PrimeFieldElem or BigUint
  pub mid_beg: usize,
  pub num_constraints: usize,
  pub witness: Witness,
  pub p: Polynomial,
  pub t: Polynomial,
  pub vi: Vec<Polynomial>,
  pub wi: Vec<Polynomial>,
  pub yi: Vec<Polynomial>,
}

impl PinocchioProver {
  pub fn new(
    f: &PrimeField,
    expr: &str,
    witness_map: &HashMap<Term, PrimeFieldElem>,
  ) -> Self {
    let eq = EquationParser::parse(f, expr).unwrap();

    let gates = &Gate::build(f, &eq);
    let tmpl = &R1CSTmpl::new(f, gates);

    let r1cs = R1CS::from_tmpl(f, tmpl, &witness_map).unwrap();
    r1cs.validate().unwrap();

    let qap = QAP::build(f, &r1cs);

    let t = QAP::build_t(f, &tmpl.constraints.len());
    let p = qap.build_p(&r1cs.witness);

    let max_degree = {
      let vi = qap.vi.iter().map(|x| x.degree()).max().unwrap();
      let wi = qap.wi.iter().map(|x| x.degree()).max().unwrap();
      let yi = qap.yi.iter().map(|x| x.degree()).max().unwrap();
      cmp::max(cmp::max(vi, wi), yi)
    };

    let witness = Witness::new(&r1cs.witness.clone(), &tmpl.mid_beg);

    let num_constraints = tmpl.constraints.len();

    //// EXPERIMENT ZONE
    {
      use crate::zk::w_trusted_setup::pinocchio::sparse_vec::SparseVec;

      let p_div_t = &p.divide_by(&t);
      let h = match &p_div_t {
        DivResult::Quotient(h) => h,
        DivResult::QuotientRemainder(_) => panic!("p must be divisible by t"),
      };

      let s = &f.elem(&11u8);

      let eval = |ps: &[Polynomial], ws: &SparseVec| -> PrimeFieldElem {
        let mut sum = f.elem(&0u8);
        for i in 0..ps.len() {
          let p = ps[i].eval_at(s);
          let w = &ws[&f.elem(&i)];
          sum = sum + p * w;
        }
        sum
      };

      let v_0 = &qap.vi[0].eval_at(s) * witness.const_witness();
      let w_0 = &qap.wi[0].eval_at(s) * witness.const_witness();
      let y_0 = &qap.yi[0].eval_at(s) * witness.const_witness();

      let mid_beg: usize = (&tmpl.mid_beg.e).try_into().unwrap();
      let v_io = eval(&qap.vi[1..mid_beg], &witness.io());
      let w_io = eval(&qap.wi[1..mid_beg], &witness.io());
      let y_io = eval(&qap.yi[1..mid_beg], &witness.io());

      let v_mid = eval(&qap.vi[mid_beg..], &witness.mid());
      let w_mid = eval(&qap.vi[mid_beg..], &witness.mid());
      let y_mid = eval(&qap.vi[mid_beg..], &witness.mid());

      let v = v_0 + v_io + v_mid;
      let w = w_0 + w_io + w_mid;
      let y = y_0 + y_io + y_mid;

      let lhs = v * w - y; 
      let rhs = &h.eval_at(s) * &t.eval_at(s);

      assert!(lhs == rhs);
    }
    //// EXPERIMENT ZONE

    PinocchioProver {
      f: f.clone(),
      max_degree: (&max_degree.e).try_into().unwrap(),
      mid_beg: (&tmpl.mid_beg.e).try_into().unwrap(),
      num_constraints,
      witness,
      p,
      t,
      vi: qap.vi.clone(),
      wi: qap.wi.clone(),
      yi: qap.yi.clone(),
    }
  }

  pub fn prove(&self, crs: &CRS) -> PinocchioProof {
    let witness_mid = &self.witness.mid();

    let calc_e1 = |points: &Vec<G1Point>| {
      let mut sum = G1Point::zero();
      for i in 0..points.len() {
        sum = sum + &(&points[i] * &witness_mid[&self.f.elem(&i)]);
      }
      sum
    };
    let calc_e2 = |points: &Vec<G2Point>| {
      let mut sum = G2Point::zero();
      for i in 0..points.len() {
        sum = sum + &(&points[i] * &witness_mid[&self.f.elem(&i)]);
      }
      sum
    };

    let v_mid = calc_e1(&crs.ek.vi_mid);
    let beta_v_mid = calc_e1(&crs.ek.beta_vi_mid);

    let w_mid_e1 = calc_e1(&crs.ek.wi_mid);
    let beta_w_mid_e1 = calc_e1(&crs.ek.beta_wi_mid);

    let w_mid_e2 = calc_e2(&crs.vk.wi_mid);

    let y_mid = calc_e1(&crs.ek.yi_mid);
    let beta_y_mid = calc_e1(&crs.ek.beta_yi_mid);

    let h = match self.p.divide_by(&self.t) {
      DivResult::Quotient(h) => h,
      DivResult::QuotientRemainder(_) => panic!("p must be divisible by t"),
    };

    let h_hiding = h.eval_with_g1_hidings(&crs.ek.si);
    let alpha_h = h.eval_with_g1_hidings(&crs.ek.alpha_si);

    PinocchioProof {
      v_mid,
      w_mid_e1,
      w_mid_e2,
      y_mid,
      beta_v_mid,
      beta_w_mid_e1,
      beta_y_mid,
      h: h_hiding,
      alpha_h,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::zk::w_trusted_setup::pinocchio::pinocchio_verifier::PinocchioVerifier;

  #[test]
  fn test_generate_proof_and_verify() {
    let f = &PrimeField::new(&3911u16);

    let expr = "(x * x * x) + x + 5 == 35";
    let eq = EquationParser::parse(f, expr).unwrap();

    /*
      x = 3
      t1 = x(3) * x(3) = 9
      t2 = t1(9) * x(3) = 27
      t3 = x(3) + 5 = 8
      t4 = t2(27) + t2(8) = 35
      out = t4
    */
    let witness_map = {
      use crate::zk::w_trusted_setup::pinocchio::term::Term::*;
      HashMap::<Term, PrimeFieldElem>::from([
        (Term::One, f.elem(&1u8)),
        (Term::var("x"), f.elem(&3u8)),
        (TmpVar(1), f.elem(&9u8)),
        (TmpVar(2), f.elem(&27u8)),
        (TmpVar(3), f.elem(&8u8)),
        (TmpVar(4), f.elem(&35u8)),
        (Out, eq.rhs),
      ])
    };
    let prover = &PinocchioProver::new(f, expr, &witness_map);
    let verifier = &PinocchioVerifier::new();
    let crs = CRS::new(f, prover);

    let proof = prover.prove(&crs);
    let result = verifier.verify(
      &proof,
      &crs,
      &prover.witness.io(),
    );

    assert!(result == true);
  }
}

