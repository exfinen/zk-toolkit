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
use std::collections::HashMap;

pub struct PinocchioProver {
  pub f: PrimeField,
  pub max_degree: usize,
  pub num_constraints: usize,
  pub witness: Witness,
  pub t: Polynomial,
  pub p: Polynomial,
  pub vi: Vec<Polynomial>,
  pub wi: Vec<Polynomial>,
  pub yi: Vec<Polynomial>,
}

impl PinocchioProver {
  fn print_debug_info(
    f: &PrimeField,
    gates: &Vec<Gate>,
    r1cs: &R1CS,
    qap: &QAP,
    s: &PrimeFieldElem,
  ) {
    println!("s = {:?}\n", s);
    println!("witness {:?}\n", &r1cs.witness);

    for (i, gate) in gates.iter().enumerate() {
      println!("{}: {:?}", i+1 , gate);
    }
    println!("");

    let mut v = f.elem(&0u8);
    let mut w = f.elem(&0u8);
    let mut y = f.elem(&0u8);
    
    for i in 0..r1cs.witness.size_in_usize() {
      let vi = &qap.vi[i].eval_at(s);
      let wi = &qap.wi[i].eval_at(s);
      let yi = &qap.yi[i].eval_at(s);

      v = &v + vi * &r1cs.witness[&i];
      w = &w + wi * &r1cs.witness[&i];
      y = &y + yi * &r1cs.witness[&i];
    }
    println!("{:?} * {:?} = {:?}\n", v, w, y);
  }
 
  pub fn new(
    f: &PrimeField,
    expr: &str,
    witness_map: &HashMap<Term, PrimeFieldElem>,
    s: &PrimeFieldElem,
  ) -> Self {
    let eq = EquationParser::parse(f, expr).unwrap();

    let gates = &Gate::build(f, &eq);
    let tmpl = &R1CSTmpl::new(f, gates);

    let r1cs = R1CS::from_tmpl(f, tmpl, &witness_map).unwrap();
    r1cs.validate().unwrap();

    let qap = QAP::build(f, &r1cs);

    let t = QAP::build_t(f, &tmpl.constraints.len());
    let p = qap.build_p(&r1cs.witness);

    let max_degree: usize = {
      let xs = vec![&qap.vi[..], &qap.wi[..], &qap.yi[..]].concat();
      xs.iter().map(|x| x.degree()).max().unwrap()
    }.e.try_into().unwrap();

    let witness = Witness::new(&r1cs.witness.clone(), &tmpl.mid_beg);
    let num_constraints = tmpl.constraints.len();

    Self::print_debug_info(f, gates, &r1cs, &qap, s);

    PinocchioProver {
      f: f.clone(),
      max_degree,
      num_constraints,
      witness,
      t,
      p,
      vi: qap.vi.clone(),
      wi: qap.wi.clone(),
      yi: qap.yi.clone(),
    }
  }

  pub fn prove(&self, crs: &CRS) -> PinocchioProof {
    println!("--> Generating proof...");
    let witness_mid = &self.witness.mid();

    macro_rules! calc {
      ($point_type:ty, $points:ident) => {{
        let mut sum = <$point_type>::zero();
        for i in 0..$points.len() {
          sum = sum + &$points[i] * &witness_mid[&i];
        }
        sum
      }};
    }
    let calc_e1 = |points: &Vec<G1Point>| calc!(G1Point, points);
    let calc_e2 = |points: &Vec<G2Point>| calc!(G2Point, points);

    // making only v and y zero-knowledge, excluding w. 
    // the reason is that including w results in having t(s)^2 in the
    // adjusted h, and that seems to make adj_h * t != v * w - y
    //
    // without using delta factors, adj_h(s) * t(s) is:
    // 
    // adj_h * t(s)
    // = (v(s) + t(s)) * (w(s) + t(s)) - (y(s) + t(s))
    // = v(s) * w(s)        + v(s) * t(s) + w(s) * t(s) + t(s)^2 - y(s) - t(s)
    // = v(s) * w(s) - y(s) + v(s) * t(s) + w(s) * t(s) + t(s)^2 - t(s)
    // = h(s) * t(s)        + v(s) * t(s) + w(s) * t(s) + t(s)^2 - t(s)
    // = t(s) * (h(s) + v(s) + w(s) + t(s) - 1)
    // 
    // so, adjusted h is h(s) + v(s) + w(s) + t(s) - 1.
    // but the existence t(s) here seems to make the calculation fail.
    //
    // TODO fix this problem and make w zero-knowledge as well 
    // 
    // so instead, this code uses the following:
    //
    // adj_h * t(s)
    // = (v(s) + t(s)) * w(s) - (y(s) + t(s))
    // = v(s) * w(s) + t(s) * w(s) - y(s) - t(s)
    // = v(s) * w(s) - y(s) + t(s) * w(s) - t(s)
    // = h(s) * t(s)        + t(s) * w(s) - t(s)
    // = t(s) * (h(s) + w(s) - 1)

    let (ek, vk, f) = (&crs.ek, &crs.vk, &self.f);

    let delta_v = &f.rand_elem(true); 
    let delta_y = &f.rand_elem(true); 

    // randomizing v and y only. the reason for not randomizing w is explained below
    //
    // (v(x) + delta_v * t(x)) * w(x) - (y(x) + delta_y * t(x))
    // = v(x) * w(x) + delta_v * t(x) * w(x) - y(x) - delta_y * t(x)
    // = v(x) * w(x) - y(x) + delta_v * t(x) * w(x) - delta_y * t(x)
    // = p(x)               + delta_v * t(x) * w(x) - delta_y * t(x)
    let randomized_p = {
      let mut w = Polynomial::zero(f);
      for wi in &self.wi {
        w = &w + wi
      }
      let delta_y = Polynomial::new(f, &vec![delta_y.clone()]);
      &self.p + &(&(&self.t * &w) * delta_v)  - &(&self.t * delta_y)
    };

    let v_mid = calc_e1(&ek.vi_mid) + &vk.t_e1 * delta_v;
    let beta_v_mid = calc_e1(&ek.beta_vi_mid) + &ek.t_beta_v * delta_v;

    let w_mid_e1 = calc_e1(&ek.wi_mid);
    let beta_w_mid_e1 = calc_e1(&ek.beta_wi_mid);

    let w_mid_e2 = calc_e2(&vk.wi_mid);

    let y_mid = calc_e1(&ek.yi_mid) + &vk.t_e1 * delta_y;
    let beta_y_mid = calc_e1(&ek.beta_yi_mid) + &ek.t_beta_y * delta_y;

    let h = match randomized_p.divide_by(&self.t) {
      DivResult::Quotient(h) => h,
      DivResult::QuotientRemainder(_) => panic!("p must be divisible by t"),
    };

    let h_hiding = h.eval_with_g1_hidings(&ek.si);

    // let adj_h = {
    //   let mut w_e_e1 = w_mid_e1.clone();
    //   for i in 0..vk.wi_io.len() {
    //     let w = &witness_io[&i];
    //     let p = &vk.wi_io_e1[i];
    //     w_e_e1 = w_e_e1 + p * w;
    //   }
    //   &h_hiding + &w_e_e1 * &self.delta_v + -&vk.one_e1 * &self.delta_y
    // };

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
    let f = &G1Point::curve_group();

    let expr = "(x * x * x) + x + 5 == 35";
    println!("Expr: {}\n", expr);
    let eq = EquationParser::parse(f, expr).unwrap();

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
    let s = &f.rand_elem(true);
    let prover = &PinocchioProver::new(f, expr, &witness_map, s);
    let verifier = &PinocchioVerifier::new();
    let crs = CRS::new(f, prover, s);

    let proof = prover.prove(&crs);
    let result = verifier.verify(
      &proof,
      &crs,
      &prover.witness.io(),
    );

    assert!(result);
  }
}

