use crate::building_block::elliptic_curve::weierstrass::curves::bls12_381::fq2::Fq2;

#[derive(Clone)]
pub struct G2Point {
  pub x: Fq2,
  pub y: Fq2,
}
