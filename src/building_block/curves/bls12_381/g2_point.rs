use crate::building_block::{
  field::prime_field::PrimeField,
  curves::{
    bls12_381::{
      fq1::Fq1,
      fq2::Fq2,
    },
    rational_point::RationalPoint,
  },
  zero::Zero,
};
use num_bigint::BigUint;
use std::sync::Arc;
use once_cell::sync::Lazy;

#[derive(Clone)]
pub enum G2Point {
  Rational { x: Fq2, y: Fq2 },
  AtInfinity,
}

static BASE_POINT: Lazy<G2Point> = Lazy::new(|| {
  let x0: Fq1 = Fq1::from(b"024aa2b2f08f0a91260805272dc51051c6e47ad4fa403b02b4510b647ae3d1770bac0326a805bbefd48056c8c121bdb8");
  let x1: Fq1 = Fq1::from(b"13e02b6052719f607dacd3a088274f65596bd0d09920b61ab5da61bbdc7f5049334cf11213945d57e5ac7d055d042b7e");
  let x = Fq2::new(&x1, &x0);

  let y0 = Fq1::from(b"0ce5d527727d6e118cc9cdc6da2e351aadfd9baa8cbdd3a76d429a695160d12c923ac9cc3baca289e193548608b82801");
  let y1 = Fq1::from(b"0606c4a02ea734cc32acd2b02bc28b99cb3e287e85a763af267492ab572e99ab3f370d275cec1da1aaa9075ff05f79be");
  let y = Fq2::new(&y1, &y0);

  G2Point::Rational { x, y }
});

static CURVE_GROUP: Lazy<Arc<PrimeField>> = Lazy::new(|| {
  let r = BigUint::parse_bytes(b"73eda753299d7d483339d80809a1d80553bda402fffe5bfeffffffff00000001", 16).unwrap();
  Arc::new(PrimeField::new(&r))
});

impl G2Point {
  pub fn new(x: &Fq2, y: &Fq2) -> Self {
    G2Point::Rational { x: x.clone(), y: y.clone() }
  }

  pub fn curve_group() -> Arc<PrimeField> {
    CURVE_GROUP.clone()
  }

  pub fn base_point() -> Self {
    BASE_POINT.clone()
  }

  pub fn inv(&self) -> Self {
    match self {
      G2Point::AtInfinity => panic!("No inverse exists for point at infinitty"),
      G2Point::Rational { x, y } => G2Point::new(&x, &y.inv()),
    }
  }
}

impl RationalPoint for G2Point {
  fn is_rational_point(&self) -> bool {
    true
  }
}

impl Zero<G2Point> for G2Point {
  fn zero() -> G2Point {
    G2Point::AtInfinity
  }

  fn is_zero(&self) -> bool {
    match self {
      G2Point::AtInfinity => true,
      _ => false,
    }
  }
}

// impl Add<G2Point> for G2Point {
//   type Output = G2Point;

//   fn add(self, rhs: G2Point) -> Self::Output {

//   }
// }
