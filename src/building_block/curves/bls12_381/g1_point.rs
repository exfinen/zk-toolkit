use crate::building_block::{
  field::prime_field::PrimeField,
  curves::{
    bls12_381::fq1::Fq1,
    rational_point::RationalPoint,
    weierstrass_eq::WeierstrassEq,
  },
  zero::Zero,
};
use num_bigint::BigUint;

#[derive(Clone)]
pub enum G1Point {
  Rational { x: Fq1, y: Fq1 },
  AtInfinity,
}

impl G1Point {
  pub fn new(x: &Fq1, y: &Fq1) -> Self {
    G1Point::Rational {
      x: x.clone(),
      y: y.clone(),
    }
  }

  pub fn base_field() -> PrimeField {
    let q = BigUint::parse_bytes(b"1a0111ea397fe69a4b1ba7b6434bacd764774b84f38512bf6730d2a0f6b0f6241eabfffeb153ffffb9feffffffffaaab", 16).unwrap();
    PrimeField::new(&q)
  }

  pub fn curve_group() -> PrimeField {
    // order of the base point
    let r = BigUint::parse_bytes(b"73eda753299d7d483339d80809a1d80553bda402fffe5bfeffffffff00000001", 16).unwrap();
    PrimeField::new(&r)
  }

  pub fn base_point() -> Self {
    let f_q = G1Point::base_field();

    let gx = f_q.elem(
      &BigUint::parse_bytes(b"17f1d3a73197d7942695638c4fa9ac0fc3688c4f9774b905a14e3a3f171bac586c55e83ff97a1aeffb3af00adb22c6bb", 16).unwrap(),
    );

    let gy = f_q.elem(
      &BigUint::parse_bytes(b"08b3f481e3aaa0f1a09e30ed741d8ae4fcf5e095d5d00af600db18cb2c04b3edd03cc744a2888ae40caa232946c5e7e1", 16).unwrap(),
    );

    G1Point::Rational {
      x: gx,
      y: gy,
    }
  }

  pub fn inv(&self) -> Self {
    match self {
      G1Point::AtInfinity => panic!("No inverse exists for point at infinitty"),
      G1Point::Rational { x, y } => G1Point::new(&x, &y.inv()),
    }
  }
}

impl RationalPoint for G1Point {
  fn is_rational_point(&self) -> bool {
    match self {
      G1Point::AtInfinity => false,
      G1Point::Rational { x, y } => {
        let f = Self::base_field();
        let a1 = f.elem(&0u8);
        let a2 = f.elem(&0u8);
        let a3 = f.elem(&0u8);
        let a4 = f.elem(&0u8);
        let a6 = f.elem(&4u8);
        let eq = WeierstrassEq::new(&a1, &a2, &a3, &a4, &a6);
        eq.is_rational_point(x, y)
      },
    }
  }
}

impl Zero<G1Point> for G1Point {
  fn zero() -> G1Point {
    G1Point::AtInfinity
  }

  fn is_zero(&self) -> bool {
    match self {
      G1Point::AtInfinity => true,
      _ => false,
    }
  }
}

// impl Add<G1Point> for G1Point {
//   type Output = G1Point;

//   fn add(self, rhs: G1Point) -> Self::Output {

//   }
// }
