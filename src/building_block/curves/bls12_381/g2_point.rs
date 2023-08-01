use crate::building_block::{
  field::prime_field::PrimeField,
  curves::{
    bls12_381::{
      fq2::Fq2,
    },
    rational_point::RationalPoint,
  },
  zero::Zero,
};
use num_bigint::BigUint;
use std::ops::Add;

#[derive(Clone)]
pub enum G2Point {
  Rational { x: Fq2, y: Fq2 },
  AtInfinity,
}

impl G2Point {
  pub fn base_field() -> PrimeField {
    let q = BigUint::parse_bytes(b"", 16).unwrap();
    PrimeField::new(&q)
  }

  pub fn curve_group() -> PrimeField {
    // order of the base point
    let r = BigUint::parse_bytes(b"73eda753299d7d483339d80809a1d80553bda402fffe5bfeffffffff00000001", 16).unwrap();
    PrimeField::new(&r)
  }

  pub fn base_point() -> Self {
    // let f_q = G1Point::base_field();

    // let gx = f_q.elem(
    //   &BigUint::parse_bytes(b"17f1d3a73197d7942695638c4fa9ac0fc3688c4f9774b905a14e3a3f171bac586c55e83ff97a1aeffb3af00adb22c6bb", 16).unwrap(),
    // );

    // let gy = f_q.elem(
    //   &BigUint::parse_bytes(b"08b3f481e3aaa0f1a09e30ed741d8ae4fcf5e095d5d00af600db18cb2c04b3edd03cc744a2888ae40caa232946c5e7e1", 16).unwrap(),
    // );

    G2Point::Rational {
      x: gx,
      y: gy,
    }
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
