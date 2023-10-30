use crate::{
  building_block::{
    curves::bls12_381::g1_point::G1Point,
    curves::bls12_381::g2_point::G2Point,
    field::{
      prime_field::PrimeField,
      prime_field_elem::PrimeFieldElem,
    },
  },
  zk::w_trusted_setup::pinocchio::pinocchio_prover::PinocchioProver,
};

pub struct EvaluationKeys {
  pub si: Vec<G1Point>,
  pub alpha_si: Vec<G1Point>,
  pub vi_mid: Vec<G1Point>,
  pub wi_mid: Vec<G1Point>,
  pub yi_mid: Vec<G1Point>,
  pub beta_vi_mid: Vec<G1Point>,
  pub beta_wi_mid: Vec<G1Point>,
  pub beta_yi_mid: Vec<G1Point>,
}

pub struct VerificationKeys {
  pub one_e1: G1Point,
  pub one_e2: G2Point,
  pub e_alpha: G2Point,
  pub e_gamma: G2Point,
  pub beta_v_gamma: G2Point,
  pub beta_w_gamma: G2Point,
  pub beta_y_gamma: G2Point,
  pub t_e1: G1Point,
  pub t_e2: G2Point,
  pub vi_io: Vec<G1Point>,
  pub wi_io: Vec<G2Point>,
  pub wi_io_e1: Vec<G1Point>,
  pub yi_io: Vec<G1Point>,
  pub wi_mid: Vec<G2Point>,
}

pub struct CRS {
  pub ek: EvaluationKeys,
  pub vk: VerificationKeys,
}

impl CRS {
  #[allow(non_snake_case)]
  pub fn new(
    f: &PrimeField,
    p: &PinocchioProver,
    s: &PrimeFieldElem,
  ) -> Self {
    println!("--> Building CRS...");
    let g1 = &G1Point::g();
    let g2 = &G2Point::g();
    let E1 = |n: &PrimeFieldElem| -> G1Point { g1 * n };
    let E2 = |n: &PrimeFieldElem| -> G2Point { g2 * n };

    let alpha = &f.rand_elem(true);
    let beta_v = &f.rand_elem(true);
    let beta_w = &f.rand_elem(true);
    let beta_y = &f.rand_elem(true);
    let gamma = &f.rand_elem(true);

    let s_pows = &s.pow_seq(&p.max_degree);
    let mid_beg: usize = (&p.witness.mid_beg.e).try_into().unwrap();

    let mid: Vec<usize> = {
      let end: usize = (&p.witness.end.e).try_into().unwrap();
      (mid_beg..=end).collect()
    };
    let io: &Vec<usize> = &(0usize..mid_beg).collect();

    // Evaluation keys
    println!("----> Computing evaluation keys...");

    // E(s^i), E(alpha * s^i)
    let si: Vec<G1Point> = s_pows.iter().map(|pow| { E1(pow) }).collect();
    let alpha_si: Vec<G1Point> = s_pows.iter().map(|pow| {
      E1(pow) * alpha
    }).collect();

    // mid contains witness 1 as well
    let vi_mid: Vec<G1Point> = mid.iter().map(|i| { E1(&p.vi[*i].eval_at(s)) }).collect();
    let wi_mid_e1: Vec<G1Point> = mid.iter().map(|i| { E1(&p.wi[*i].eval_at(s)) }).collect();
    let wi_mid_e2: Vec<G2Point> = mid.iter().map(|i| { E2(&p.wi[*i].eval_at(s)) }).collect();
    let yi_mid: Vec<G1Point> = mid.iter().map(|i| { E1(&p.yi[*i].eval_at(s)) }).collect();

    let beta_vi_mid: Vec<G1Point> = mid.iter().map(|i| { E1(&p.vi[*i].eval_at(s)) * beta_v }).collect();
    let beta_wi_mid: Vec<G1Point> = mid.iter().map(|i| { E1(&p.wi[*i].eval_at(s)) * beta_w }).collect();
    let beta_yi_mid: Vec<G1Point> = mid.iter().map(|i| { E1(&p.yi[*i].eval_at(s)) * beta_y }).collect();

    // Verification keys
    println!("----> Computing verification keys...");

    let one_e1 = E1(&f.elem(&1u8));
    let one_e2 = E2(&f.elem(&1u8));  
    let e_alpha = E2(alpha);
    let e_gamma = E2(gamma);
    let beta_v_gamma = E2(gamma) * beta_v; 
    let beta_w_gamma = E2(gamma) * beta_w; 
    let beta_y_gamma = E2(gamma) * beta_y; 

    let t_e1 = E1(&p.t.eval_at(s));
    let t_e2 = E2(&p.t.eval_at(s));

    let vi_io: Vec<G1Point> = io.iter().map(|i| { E1(&p.vi[*i].eval_at(s)) }).collect();
    let wi_io: Vec<G2Point> = io.iter().map(|i| { E2(&p.wi[*i].eval_at(s)) }).collect();
    let wi_io_e1: Vec<G1Point> = io.iter().map(|i| { E1(&p.wi[*i].eval_at(s)) }).collect();
    let yi_io: Vec<G1Point> = io.iter().map(|i| { E1(&p.yi[*i].eval_at(s)) }).collect();

    let ek = EvaluationKeys {
      si,
      alpha_si,
      vi_mid,
      wi_mid: wi_mid_e1,
      yi_mid,
      beta_vi_mid,
      beta_wi_mid,
      beta_yi_mid,
    };

    let vk = VerificationKeys {
      one_e1,
      one_e2,
      e_alpha,
      e_gamma,
      beta_v_gamma,
      beta_w_gamma,
      beta_y_gamma,
      t_e1,
      t_e2,
      vi_io,
      wi_io,
      wi_io_e1,
      yi_io,
      wi_mid: wi_mid_e2,
    };

    CRS {
      ek,
      vk,
    }
  }
}

