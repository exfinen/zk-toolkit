use crate::building_block::elliptic_curve::weierstrass::curves::bls12_381::fq2::Fq2;

#[derive(Clone)]
pub struct G2Point<E> {
  pub x: Fq2<E>,
  pub y: Fq2<E>,
}
