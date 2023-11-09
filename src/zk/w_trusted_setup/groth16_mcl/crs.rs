use crate::{
  building_block::mcl::{
    mcl_fr::MclFr,
    mcl_g1::MclG1,
    mcl_g2::MclG2,
    mcl_gt::MclGT,
    pairing::Pairing,
    qap::qap::QAP,
  },
  zk::w_trusted_setup::groth16_mcl::prover::Prover,
};

pub struct G1 {
  pub alpha: MclG1,
  pub beta: MclG1,
  pub delta: MclG1,
  pub xi: Vec<MclG1>,  // x powers
  pub uvw_stmt: Vec<MclG1>,  // beta*u(x) + alpha*v(x) + w(x) / div (statement)
  pub uvw_wit: Vec<MclG1>,   // beta*u(x) + alpha*v(x) + w(x) / div (witness)
  pub xt_by_delta: Vec<MclG1>,
}

pub struct G2 {
  pub beta: MclG2,
  pub gamma: MclG2,
  pub delta: MclG2,
  pub xi: Vec<MclG2>,  // x powers
}

pub struct GT {
  pub alpha_beta: MclGT,
}

#[allow(non_snake_case)]
pub struct CRS {
  pub g1: G1,
  pub g2: G2,
  pub gt: GT,
}

impl CRS {
  // 0, 1, .., l, l+1, .., m
  // +---------+  +--------+
  //  statement    witness
  pub fn new(
    prover: &Prover,
    pairing: &Pairing,
  ) -> Self {
    println!("--> Building sigma...");
    let g = &MclG1::g();
    let h = &MclG2::g();

    // sample random non-zero field element
    let alpha = &MclFr::rand(true);
    let beta = &MclFr::rand(true);
    let gamma = &MclFr::rand(true);
    let delta = &MclFr::rand(true);
    let x = &MclFr::rand(true);

    macro_rules! calc_uvw_div {
      ($from: expr, $to: expr, $div_factor: expr) => {
        {
          let mut ys: Vec<MclG1> = vec![];
          let mut i = $from.clone();

          while &i <= $to {
            let ui = beta * &prover.ui[i].eval_at(x);
            let vi = alpha * &prover.vi[i].eval_at(x);
            let wi = &prover.wi[i].eval_at(x);
            let y = (ui + vi + wi) * $div_factor;

            ys.push(g * y);
            i += 1;
          } 
          ys
        }
      }
    }

    let uvw_stmt = calc_uvw_div!(0, &prover.l.to_usize(), &gamma.inv());
    let uvw_wit = calc_uvw_div!(&prover.l.to_usize() + 1, &prover.m.to_usize(), &delta.inv());

    macro_rules! calc_n_pows {
      ($point_type: ty, $x: expr) => {
        {
          let generator = &<$point_type>::g();
          let mut ys: Vec<$point_type> = vec![];
          let mut x_pow = MclFr::from(1);

          for _ in 0..prover.n.to_usize(){
            ys.push(generator * &x_pow);
            x_pow = x_pow * x;
          }
          ys
        }
      }
    }

    let xi_g1 = calc_n_pows!(MclG1, x);
      
    let xt_by_delta = {
      let t = &QAP::build_t(&prover.n).eval_at(x);
      let mut xs = vec![]; 

      let mut x_pow = MclFr::from(1);
      for _ in 0..prover.n.to_usize() {
        xs.push(g * (&x_pow * t * delta.inv()));
        x_pow = &x_pow * x;
      } 
      xs
    };

    let g1 = G1 {
      alpha: g * alpha,
      beta: g * beta,
      delta: g * delta,
      xi: xi_g1,    
      uvw_stmt,
      uvw_wit,
      xt_by_delta,
    };

    let xi_g2 = calc_n_pows!(MclG2, x);

    let g2 = G2 {
      beta: h * beta,
      gamma: h * gamma,
      delta: h * delta,
      xi: xi_g2,
    };

    let gt = GT {
      alpha_beta: pairing.e(&g1.alpha, &g2.beta),
    };

    CRS {
      g1,
      g2,
      gt,
    }
  }
}

