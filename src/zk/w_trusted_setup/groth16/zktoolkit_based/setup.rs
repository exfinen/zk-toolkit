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
  },
  zk::w_trusted_setup::groth16::zktoolkit_based::prover::Prover,
};

pub struct G1 {
  alpha: G1Point,
  beta: G1Point,
  gamma: G1Point,
  xi: Vec<G1Point>,  // x powers
  si: Vec<G1Point>,  // statement
  wi: Vec<G1Point>,  // witness
  xi_t: Vec<G1Point>,  // x powers * t(x)
}

pub struct G2 {
  beta: G2Point,
  gamma: G2Point,
  delta: G2Point,
  xi: Vec<G2Point>,  // x powers
}

pub struct Sigma {
  g1: G1,
  g2: G2,
}

impl Sigma {
  #[allow(non_snake_case)]
  pub fn new(
    f: &PrimeField,
    n: &usize,
  ) -> Self {
    println!("--> Building setup...");
    let g1 = &G1Point::g();
    let g2 = &G2Point::g();

    // generate random values 
    let alpha = &f.rand_elem(true);

    Setup {
      ek,
      vk,
    }
  }
}

