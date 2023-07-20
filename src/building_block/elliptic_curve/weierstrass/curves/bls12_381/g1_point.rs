use crate::building_block::{
  elliptic_curve::{
    affine_point::AffinePoint,
    weierstrass::curves::bls12_381::fq1::Fq1,
    new_affine_point::NewAffinePoint,
  },
  zero::Zero,
};

#[derive(Clone)]
pub struct G1Point<E> {
  pub x: Fq1<E>,
  pub y: Fq1<E>,
}

impl<E, T> Zero<G1Point<E>> for G1Point<E> {
  fn get_zero(f: &T) -> G1Point<E> {
    G1Point {
      x: Fq1::zero(),
      y: Fq1::zero(),
    }
  }

  fn is_zero(&self) -> bool {
     self.x.is_zero() && self.y.is_zero()
  }
}

impl<E> AffinePoint<G1Point<E>, Fq1<E>> for G1Point<E> {
  fn x(&self) -> Fq1<E> {
    self.x
  }

  fn y(&self) -> Fq1<E> {
    self.y
  }
}

impl<E> NewAffinePoint<G1Point<E>, Fq1<E>> for G1Point<E> {
  fn new(x: &Fq1<E>, y: &Fq1<E>) -> Self {
    G1Point {
      x: x.clone(),
      y: y.clone(),
    }
  }
}
