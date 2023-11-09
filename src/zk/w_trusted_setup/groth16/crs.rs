use crate::{
  building_block::{
    curves::bls12_381::{
      g1_point::G1Point,
      g2_point::G2Point,
      fq12::Fq12,
      pairing::Pairing,
    },
    field::prime_field::PrimeField,
  },
  zk::w_trusted_setup::{
    groth16::prover::Prover,
    qap::qap::QAP,
  },
};

pub struct G1 {
  pub alpha: G1Point,
  pub beta: G1Point,
  pub delta: G1Point,
  pub xi: Vec<G1Point>,  // x powers
  pub uvw_stmt: Vec<G1Point>,  // beta*u(x) + alpha*v(x) + w(x) / div (statement)
  pub uvw_wit: Vec<G1Point>,   // beta*u(x) + alpha*v(x) + w(x) / div (witness)
  pub xt_by_delta: Vec<G1Point>,
}

pub struct G2 {
  pub beta: G2Point,
  pub gamma: G2Point,
  pub delta: G2Point,
  pub xi: Vec<G2Point>,  // x powers
}

pub struct GT {
  pub alpha_beta: Fq12,
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
    f: &PrimeField,
    prover: &Prover,
    pairing: &Pairing,
  ) -> Self {
    println!("--> Building sigma...");
    let g = &G1Point::g();
    let h = &G2Point::g();

    // sample random non-zero field element
    let alpha = &f.rand_elem(true);
    let beta = &f.rand_elem(true);
    let gamma = &f.rand_elem(true);
    let delta = &f.rand_elem(true);
    let x = &f.rand_elem(true);

    macro_rules! calc_uvw_div {
      ($from: expr, $to: expr, $div_factor: expr) => {
        {
          let mut ys: Vec<G1Point> = vec![];
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

    let uvw_stmt = calc_uvw_div!(0, &prover.l, &gamma.inv());
    let uvw_wit = calc_uvw_div!(&prover.l + 1, &prover.m, &delta.inv());

    macro_rules! calc_n_pows {
      ($point_type: ty, $x: expr) => {
        {
          let generator = &<$point_type>::g();
          let mut ys: Vec<$point_type> = vec![];
          let mut x_pow = f.elem(&1u8);

          for _ in 0..prover.n {
            ys.push(generator * &x_pow);
            x_pow = x_pow * x;
          }
          ys
        }
      }
    }

    let xi_g1 = calc_n_pows!(G1Point, x);
      
    let xt_by_delta = {
      let t = &QAP::build_t(f, &prover.n).eval_at(x);
      let mut xs = vec![]; 

      let mut x_pow = f.elem(&1u8);
      for _ in 0..prover.n {
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

    let xi_g2 = calc_n_pows!(G2Point, x);

    let g2 = G2 {
      beta: h * beta,
      gamma: h * gamma,
      delta: h * delta,
      xi: xi_g2,
    };

    let gt = GT {
      alpha_beta: pairing.tate(&g1.alpha, &g2.beta),
    };

    CRS {
      g1,
      g2,
      gt,
    }
  }
}

