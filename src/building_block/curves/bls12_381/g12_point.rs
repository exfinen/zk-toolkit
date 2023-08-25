use crate::building_block::{
  curves::bls12_381::{
    fq2::Fq2,
    fq6::Fq6,
    fq12::Fq12,
    g1_point::G1Point,
    g2_point::G2Point,
  },
  to_biguint::ToBigUint,
  zero::Zero,
};
use std::convert::From;

#[derive(Clone, Debug)]
pub enum G12Point {
  Rational { x: Fq12, y: Fq12 },
  AtInfinity,
}

impl G12Point {
  pub fn new(x: &Fq12, y: &Fq12) -> Self {
    G12Point::Rational {
      x: x.clone(),
      y: y.clone(),
    }
  }
}

impl From<&G1Point> for G12Point {
  fn from(p: &G1Point) -> Self {
    match p {
      G1Point::AtInfinity => G12Point::AtInfinity,
      G1Point::Rational { x, y } => {
        let x12 = Fq12::from(x as &dyn ToBigUint);
        let y12 = Fq12::from(y as &dyn ToBigUint);

        G12Point::Rational {
          x: x12,
          y: y12,
        }
      },
    }
  }
}

// Untwist operation
impl From<&G2Point> for G12Point {
  fn from(p: &G2Point) -> Self {
    match p {
      G2Point::AtInfinity => G12Point::AtInfinity,
      G2Point::Rational { x, y } => {
        let one = &Fq2::from(&1u8 as &dyn ToBigUint);
        let root = &Fq6::new(&Fq2::zero(), one, &Fq2::zero());

        let x6_w0 = Fq6::new(&Fq2::zero(), &Fq2::zero(), &x);
        let y6_w0 = Fq6::new(&Fq2::zero(), &Fq2::zero(), &y);

        let x12 = Fq12::new(&Fq6::zero(), &x6_w0) * Fq12::new(&Fq6::zero(), root).inv();
        let y12 = Fq12::new(&Fq6::zero(), &y6_w0) * Fq12::new(root, &Fq6::zero()).inv();

        G12Point::Rational {
          x: x12,
          y: y12,
        }
      },
    }
  }
}