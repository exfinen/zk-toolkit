use crate::{
  building_block::{
    curves::bls12_381::g1_point::G1Point,
    curves::bls12_381::g2_point::G2Point,
    field::prime_field::PrimeField,
  },
  zk::w_trusted_setup::pinocchio::pinocchio_prover::PinocchioProver,
};

pub struct EvaluationKeys {
  pub g_v_v_k_mid: Vec<G1Point>,
  pub g1_w_w_k_mid: Vec<G1Point>,
  pub g2_w_w_k_mid: Vec<G2Point>,
  pub g_y_y_k_mid: Vec<G1Point>,
  pub g_v_alpha_v_k_mid: Vec<G1Point>,
  pub g_w_alpha_w_k_mid: Vec<G1Point>,
  pub g_y_alpha_y_k_mid: Vec<G1Point>,
  pub g1_si: Vec<G1Point>,
  pub g2_si: Vec<G2Point>,
  pub g_vwy_beta_vwy_k_mid: Vec<G1Point>,
}

pub struct VerificationKeys {
  pub one_g1: G1Point,
  pub one_g2: G2Point,
  pub g_alpha_v: G2Point,
  pub g2_alpha_w: G2Point,
  pub g2_alpha_y: G2Point,
  pub g_gamma: G2Point,
  pub g_beta_gamma: G2Point,
  pub g_y_t: G1Point,
  pub g_v_v_k_io: Vec<G1Point>,
  pub g_w_w_k_io: Vec<G2Point>,
  pub g_y_y_k_io: Vec<G1Point>,
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
    let g_v_v_k_mid: Vec<G1Point> = mid.iter().map(|i| { g1_v * &p.vi[*i].eval_at(s) }).collect();
    let g1_w_w_k_mid: Vec<G1Point> = mid.iter().map(|i| { g1_w * &p.wi[*i].eval_at(s) }).collect();
    let g2_w_w_k_mid: Vec<G2Point> = mid.iter().map(|i| { g2_w * &p.wi[*i].eval_at(s) }).collect();
    let g_y_y_k_mid: Vec<G1Point> = mid.iter().map(|i| { g_y * &p.yi[*i].eval_at(s) }).collect();

    let g_v_alpha_v_k_mid: Vec<G1Point> = mid.iter().map(|i| { g1_v * alpha_v * &p.vi[*i].eval_at(s) }).collect();
    let g_w_alpha_w_k_mid: Vec<G1Point> = mid.iter().map(|i| { g1_w * alpha_w * &p.wi[*i].eval_at(s) }).collect();
    let g_y_alpha_y_k_mid: Vec<G1Point> = mid.iter().map(|i| { g_y * alpha_y * &p.yi[*i].eval_at(s) }).collect();

    let s_pows = &s.pow_seq(&p.max_degree);
    let g1_si: Vec<G1Point> = s_pows.iter().map(|pow| { g1 * pow }).collect();
    let g2_si: Vec<G2Point> = s_pows.iter().map(|pow| { g2 * pow }).collect();

    let g_vwy_beta_vwy_k_mid: Vec<G1Point> = {
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
    let g_alpha_v = g2 * alpha_v;
    let g2_alpha_w = g2 * alpha_w;
    let g2_alpha_y = g2 * alpha_y;
    let g_gamma = g2 * gamma;
    let g_beta_gamma = g2 * beta * gamma;

    let g_y_t = g_y * p.t.eval_at(s);

    let g_v_v_k_io: Vec<G1Point> = io.iter().map(|i| { g1_v * &p.vi[*i].eval_at(s) }).collect();
    let g_w_w_k_io: Vec<G2Point> = io.iter().map(|i| { g2_w * &p.wi[*i].eval_at(s) }).collect();
    let g_y_y_k_io: Vec<G1Point> = io.iter().map(|i| { g_y * &p.yi[*i].eval_at(s) }).collect();

    let ek = EvaluationKeys {
      g_v_v_k_mid,
      g1_w_w_k_mid,
      g2_w_w_k_mid,
      g_y_y_k_mid,
      g_v_alpha_v_k_mid,
      g_w_alpha_w_k_mid,
      g_y_alpha_y_k_mid,
      g1_si,
      g2_si,
      g_vwy_beta_vwy_k_mid,
    };

    let vk = VerificationKeys {
      one_g1,
      one_g2,
      g_alpha_v,
      g2_alpha_w,
      g2_alpha_y,
      g_gamma,
      g_beta_gamma,
      g_y_t,
      g_v_v_k_io,
      g_w_w_k_io,
      g_y_y_k_io,
    };

    CRS {
      ek,
      vk,
    }
  }
}

