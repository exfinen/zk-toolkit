use crate::{
  building_block::{
    curves::bls12_381::g1_point::G1Point,
    field::{
      prime_field::PrimeField,
      prime_field_elem::PrimeFieldElem,
    },
  },
  zk::w_trusted_setup::pinocchio::pinocchio_prover::PinocchioProver,
};

#[allow(dead_code)]
pub struct CRS {
  // Evaluation keys
  h_si: Vec<G1Point>,
  h_alpha_si: Vec<G1Point>,
  h_vi_mid: Vec<G1Point>,
  h_wi_mid: Vec<G1Point>,
  h_yi_mid: Vec<G1Point>,
  h_beta_vi_mid: Vec<G1Point>,
  h_beta_wi_mid: Vec<G1Point>,
  h_beta_yi_mid: Vec<G1Point>,

  // Verification keys
  h_one: G1Point,
  h_alpha: G1Point,
  h_gamma: G1Point,
  h_beta_v_gamma: G1Point,
  h_beta_w_gamma: G1Point,
  h_beta_y_gamma: G1Point,
  h_t: G1Point,
  h_v0: G1Point,
  h_w0: G1Point,
  h_y0: G1Point,
  h_vi_io: Vec<G1Point>,
  h_wi_io: Vec<G1Point>,
  h_yi_io: Vec<G1Point>,
}

impl CRS {
  #[allow(non_snake_case)]
  pub fn new(f: &PrimeField, p: &PinocchioProver) -> Self {
    let g1 = &G1Point::g();
    //let g2 = &G2Point::g();
    let E1 = |n: &PrimeFieldElem| -> G1Point { g1 * n };
    //let E2 = |n: &PrimeFieldElem| -> G2Point { g2 * n };

    let s = &f.rand_elem(true);
    let alpha = &f.rand_elem(true);
    let beta_v = &f.rand_elem(true);
    let beta_w = &f.rand_elem(true);
    let beta_y = &f.rand_elem(true);
    let gamma = &f.rand_elem(true);

    let s_pows = &s.pow_seq(&p.max_degree);
    let mid: &Vec<usize> = &(*&p.mid_beg..=*&p.max_degree).collect();
    let io: &Vec<usize> = &(1usize..*&p.mid_beg).collect();  // TODO is 0 handled separately?

    // Evaluation keys

    // E(s^i), E(alpha * s^i)
    let h_si: Vec<G1Point> = s_pows.iter().map(|pow| { E1(pow) }).collect();
    let h_alpha_si: Vec<G1Point> = s_pows.iter().map(|pow| {
      E1(&(alpha * pow))
    }).collect();

    // E(vi(s)), E(wi(x), E(yi(x))
    let h_vi_mid: Vec<G1Point> = mid.iter().map(|i| { E1(&p.vi[i - 1].eval_at(s)) }).collect();
    let h_wi_mid: Vec<G1Point> = mid.iter().map(|i| { E1(&p.wi[i - 1].eval_at(s)) }).collect();
    let h_yi_mid: Vec<G1Point> = mid.iter().map(|i| { E1(&p.yi[i - 1].eval_at(s)) }).collect();

    // E(beta_v * vi(s)), E(beta_w * wi(s)), E(beta_y * yi(s))
    let h_beta_vi_mid: Vec<G1Point> = mid.iter().map(|i| { E1(&(beta_v * p.vi[i - 1].eval_at(s))) }).collect();
    let h_beta_wi_mid: Vec<G1Point> = mid.iter().map(|i| { E1(&(beta_w * p.wi[i - 1].eval_at(s))) }).collect();
    let h_beta_yi_mid: Vec<G1Point> = mid.iter().map(|i| { E1(&(beta_v * p.yi[i - 1].eval_at(s))) }).collect();

    // Verification keys
    let h_one = E1(&f.elem(&1u8));  
    let h_alpha = E1(alpha);
    let h_gamma = E1(gamma);
    let h_beta_v_gamma = E1(&(beta_v * gamma)); 
    let h_beta_w_gamma = E1(&(beta_w * gamma)); 
    let h_beta_y_gamma = E1(&(beta_y * gamma)); 

    let h_t = E1(&p.t.eval_at(s));
    let h_v0 = E1(&p.vi[0].eval_at(s));
    let h_w0 = E1(&p.wi[0].eval_at(s));
    let h_y0 = E1(&p.yi[0].eval_at(s));

    let h_vi_io: Vec<G1Point> = io.iter().map(|i| { E1(&p.vi[*i].eval_at(s)) }).collect();
    let h_wi_io: Vec<G1Point> = io.iter().map(|i| { E1(&p.wi[*i].eval_at(s)) }).collect();
    let h_yi_io: Vec<G1Point> = io.iter().map(|i| { E1(&p.yi[*i].eval_at(s)) }).collect();

    CRS {
      // Evaluation keys
      h_si,
      h_alpha_si,
      h_vi_mid,
      h_wi_mid,
      h_yi_mid,
      h_beta_vi_mid,
      h_beta_wi_mid,
      h_beta_yi_mid,

      // Verification keys
      h_one,
      h_alpha,
      h_gamma,
      h_beta_v_gamma,
      h_beta_w_gamma,
      h_beta_y_gamma,
      h_t,
      h_v0,
      h_w0,
      h_y0,
      h_vi_io,
      h_wi_io,
      h_yi_io,
    }
  }
}
