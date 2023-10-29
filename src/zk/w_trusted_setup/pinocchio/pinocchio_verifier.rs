use crate::{
  building_block::curves::bls12_381::pairing::Pairing,
  zk::w_trusted_setup::pinocchio::{
    crs::CRS,
    pinocchio_proof::PinocchioProof,
    sparse_vec::SparseVec,
  },
};

pub struct PinocchioVerifier {
  pairing: Pairing,
}

impl PinocchioVerifier {
  pub fn new() -> Self {
    let pairing = Pairing::new();

    PinocchioVerifier {
      pairing,
    }
  }

  pub fn verify(
    &self,
    proof: &PinocchioProof,
    crs: &CRS,
    witness_io: &SparseVec,
  ) -> bool {
    println!("--> Verifying Pinnochio proof...");
    let e = |a, b| self.pairing.tate(a, b);

    macro_rules! fail_if_ne { ($a:expr, $b:expr) => { if ($a != $b) { return false; } }}
    let (p, vk) = (&proof, &crs.vk); 

    println!("----> Checking if e(E(αh(s)),E(1)) =? e(E(h(s)),E(α))...");
    fail_if_ne!(e(&p.alpha_h, &vk.one_e2), e(&p.h, &vk.e_alpha));

    println!("----> Checking if e(E(βv v_mid(s), E(γ)) =? e(v_mid(s),E(βvγ))..."); 
    fail_if_ne!(e(&p.beta_v_mid, &vk.e_gamma), e(&p.v_mid, &vk.beta_v_gamma));

    println!("----> Checking if e(E(βw w_mid(s)), E(γ)) =? e(w_mid(s),E(βwγ))..."); 
    fail_if_ne!(e(&p.beta_w_mid_e1, &vk.e_gamma), e(&p.w_mid_e1, &vk.beta_w_gamma));

    println!("----> Checking if e(E(βy y_mid(s)), E(γ)) =? e(y_mid(s),E(βyγ))...");
    fail_if_ne!(e(&p.beta_y_mid, &vk.e_gamma), e(&p.y_mid, &vk.beta_y_gamma));
 
    let f = &witness_io.f;

    macro_rules! add_io_x_wit_to_mid {
      ($io_polys:expr, $mid_zk:expr) => {{
        let mut sum = $mid_zk.clone();
        for i in 0..$io_polys.len() {
          let w = &witness_io[&i];
          let p = &$io_polys[i];
          sum = sum + p * w;
        }
        sum
      }};
    }
    let v_e = add_io_x_wit_to_mid!(vk.vi_io, p.v_mid_zk);
    let w_e = add_io_x_wit_to_mid!(vk.wi_io, p.w_mid_e2);
    let y_e = add_io_x_wit_to_mid!(vk.yi_io, p.y_mid_zk);

    println!("----> Checking if e(v_e, w_e)/e(y_e, E(1)) ?= e(E(h(s)), E(t(s)))...");
    let lhs = e(&v_e, &w_e) * e(&y_e, &vk.one_e2).inv();
    let rhs = e(&p.adj_h, &vk.t_e2);

    lhs == rhs
  }
}


