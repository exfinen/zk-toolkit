use crate::{
  building_block::{
    curves::bls12_381::g1_point::G1Point,
    curves::bls12_381::g2_point::G2Point,
    field::prime_field::PrimeField,
  },
  zk::w_trusted_setup::pinocchio::pinocchio_prover::PinocchioProver,
};

pub struct EvaluationKeys {
  pub vk_mid: Vec<G1Point>,
  pub g1_wk_mid: Vec<G1Point>,
  pub g2_wk_mid: Vec<G2Point>,
  pub yk_mid: Vec<G1Point>,
  pub alpha_vk_mid: Vec<G1Point>,
  pub alpha_wk_mid: Vec<G1Point>,
  pub alpha_yk_mid: Vec<G1Point>,
  pub si: Vec<G2Point>,
  pub beta_vwy_k_mid: Vec<G1Point>,
}

pub struct VerificationKeys {
  pub one_g1: G1Point,
  pub one_g2: G2Point,
  pub alpha_v: G2Point,
  pub alpha_w: G1Point,
  pub alpha_y: G2Point,
  pub gamma: G2Point,
  pub beta_gamma: G2Point,
  pub t: G1Point,
  pub vk_io: Vec<G1Point>,
  pub wk_io: Vec<G2Point>,
  pub yk_io: Vec<G1Point>,
  pub alpha_v_t: G1Point,
  pub alpha_y_t: G1Point,
  pub beta_t: G1Point,
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
  ) -> Self {
    println!("--> Building CRS...");
    let g1 = &G1Point::g();
    let g2 = &G2Point::g();

    // generate random values 
    let r_v = &f.rand_elem(true);
    let r_w = &f.rand_elem(true);
    let alpha_v = &f.rand_elem(true);
    let alpha_w = &f.rand_elem(true);
    let alpha_y = &f.rand_elem(true);
    let beta = &f.rand_elem(true);
    let gamma = &f.rand_elem(true);

    // derive values from random values
    let r_y = &(r_v * r_w);
    let g1_v = &(g1 * r_v);
    let g1_w = &(g1 * r_w);
    let g2_w = &(g2 * r_w);
    let g_y = &(g1 * r_y);

    // build indices
    let (mid, io) = {
      let mid_beg: usize = (&p.witness.mid_beg.e).try_into().unwrap();
      let mid: Vec<usize> = {
        let end: usize = (&p.witness.end.e).try_into().unwrap();
        (mid_beg..=end).collect()
      };
      let io = (0..mid_beg).collect::<Vec<usize>>();
      (mid, io)
    };
    let s = &f.rand_elem(true);

    // compute evaluation keys
    println!("----> Computing evaluation keys...");
    let vk_mid: Vec<G1Point> = mid.iter().map(|i| { g1_v * &p.vi[*i].eval_at(s) }).collect();
    let g1_wk_mid: Vec<G1Point> = mid.iter().map(|i| { g1_w * &p.wi[*i].eval_at(s) }).collect();
    let g2_wk_mid: Vec<G2Point> = mid.iter().map(|i| { g2_w * &p.wi[*i].eval_at(s) }).collect();
    let yk_mid: Vec<G1Point> = mid.iter().map(|i| { g_y * &p.yi[*i].eval_at(s) }).collect();

    let alpha_vk_mid: Vec<G1Point> = mid.iter().map(|i| { g1_v * alpha_v * &p.vi[*i].eval_at(s) }).collect();
    let alpha_wk_mid: Vec<G1Point> = mid.iter().map(|i| { g1_w * alpha_w * &p.wi[*i].eval_at(s) }).collect();
    let alpha_yk_mid: Vec<G1Point> = mid.iter().map(|i| { g_y * alpha_y * &p.yi[*i].eval_at(s) }).collect();

    let s_pows = &s.pow_seq(&p.max_degree);
    let si: Vec<G2Point> = s_pows.iter().map(|pow| { g2 * pow }).collect();

    let beta_vwy_k_mid: Vec<G1Point> = {
      mid.iter().map(|i| {
        g1_v * beta * &p.vi[*i].eval_at(s)
        + g1_w * beta * &p.wi[*i].eval_at(s)
        + g_y * beta * &p.yi[*i].eval_at(s)
      }).collect()
    };

    // compute verification keys
    println!("----> Computing verification keys...");
    let one_g1 = g1 * f.elem(&1u8);
    let one_g2 = g2 * f.elem(&1u8);
    let alpha_v_pt = g2 * alpha_v;
    let alpha_w = g1 * alpha_w;
    let alpha_y_pt = g2 * alpha_y;
    let gamma_pt = g2 * gamma;
    let beta_gamma = g2 * gamma * beta;

    let t = g_y * p.t.eval_at(s);

    let vk_io: Vec<G1Point> = io.iter().map(|i| { g1_v * &p.vi[*i].eval_at(s) }).collect();
    let wk_io: Vec<G2Point> = io.iter().map(|i| { g2_w * &p.wi[*i].eval_at(s) }).collect();
    let yk_io: Vec<G1Point> = io.iter().map(|i| { g_y * &p.yi[*i].eval_at(s) }).collect();

    let ek = EvaluationKeys {
      vk_mid,
      g1_wk_mid,
      g2_wk_mid,
      yk_mid,
      alpha_vk_mid,
      alpha_wk_mid,
      alpha_yk_mid,
      si,
      beta_vwy_k_mid,
    };

    let alpha_v_t: G1Point = &t * alpha_v;
    let alpha_y_t: G1Point = &t * alpha_y;
    let beta_t = &t * beta;

    let vk = VerificationKeys {
      one_g1,
      one_g2,
      alpha_v: alpha_v_pt,
      alpha_w,
      alpha_y: alpha_y_pt,
      gamma: gamma_pt,
      beta_gamma,
      t,
      vk_io,
      wk_io,
      yk_io,
      alpha_v_t,
      alpha_y_t,
      beta_t,
    };

    CRS {
      ek,
      vk,
    }
  }
}

