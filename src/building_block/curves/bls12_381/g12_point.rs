use crate::building_block::{
  curves::bls12_381::{
    fq12::Fq12,
    g1_point::G1Point,
  },
  to_biguint::ToBigUint,
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

impl From<G1Point> for G12Point {
  fn from(p: G1Point) -> Self {
    match p {
      G1Point::AtInfinity => G12Point::AtInfinity,
      G1Point::Rational { x, y } => {
        let x12 = Fq12::from(&x as &dyn ToBigUint);
        let y12 = Fq12::from(&y as &dyn ToBigUint);

        G12Point::Rational {
          x: x12,
          y: y12,
        }
      },
    }
  }
}