use mcl_rust::*;
use std::ops::{Add, Mul, Neg, AddAssign};
use num_traits::Zero;
use once_cell::sync::Lazy;
use crate::building_block::curves::mcl::mcl_fr::MclFr;

#[derive(Clone, Debug)]
pub struct MclG1 {
  pub v: G1,
}

static GENERATOR: Lazy<MclG1> = Lazy::new(|| {
  let serialized_g = "1 3685416753713387016781088315183077757961620795782546409894578378688607592378376318836054947676345821548104185464507 1339506544944476473020471379941921221584933875938349620426543736416511423956333506472724655353366534992391756441569";
  let mut v = G1::zero();
  G1::set_str(&mut v, serialized_g, 10);
  MclG1 { v }
});

impl MclG1 {
  pub fn new() -> Self {
    let v = G1::zero();
    MclG1::from(&v)
  }

  pub fn g() -> MclG1 {
    GENERATOR.clone()
  }

  pub fn inv(&self) -> Self {
    let mut v = G1::zero();
    G1::neg(&mut v, &self.v);
    MclG1::from(&v)
  }

  pub fn get_random_point() -> MclG1 {
    let mut v = Fr::zero();
    v.set_by_csprng();
    MclG1::g() * MclFr::from(&v)
  }
}

impl Zero for MclG1 {
  fn zero() -> MclG1 {
    let v = G1::zero();
    MclG1::from(&v)
  }

  fn is_zero(&self) -> bool {
    self.v.is_zero()
  }
}

impl From<&G1> for MclG1 {
  fn from(v: &G1) -> Self {
    MclG1 { v: v.clone() }
  }
}

macro_rules! impl_add {
  ($rhs: ty, $target: ty) => {
    impl Add<$rhs> for $target {
      type Output = MclG1;

      fn add(self, rhs: $rhs) -> Self::Output {
        let mut v = G1::zero();
        G1::add(&mut v, &self.v, &rhs.v);
        MclG1::from(&v)
      }
    }
  };
}
impl_add!(MclG1, MclG1);
impl_add!(&MclG1, MclG1);
impl_add!(MclG1, &MclG1);
impl_add!(&MclG1, &MclG1);

macro_rules! impl_mul {
  ($rhs: ty, $target: ty) => {
    impl Mul<$rhs> for $target {
      type Output = MclG1;

      fn mul(self, rhs: $rhs) -> Self::Output {
        let mut v = G1::zero();
        G1::mul(&mut v, &self.v, &rhs.v);
        MclG1::from(&v)
      }
    }
  };
}
impl_mul!(MclFr, MclG1);
impl_mul!(&MclFr, MclG1);
impl_mul!(MclFr, &MclG1);
impl_mul!(&MclFr, &MclG1);

impl AddAssign<MclG1> for MclG1 {
  fn add_assign(&mut self, rhs: MclG1) {
    *self = &*self + rhs
  }
}

impl PartialEq for MclG1 {
  fn eq(&self, rhs: &Self) -> bool {
    self.v == rhs.v
  }
}

impl Eq for MclG1 {}

macro_rules! impl_neg {
  ($target: ty) => {
    impl Neg for $target {
      type Output = MclG1;

      fn neg(self) -> Self::Output {
        let mut v = G1::zero();
        G1::neg(&mut v, &self.v);
        MclG1::from(&v)
      }
    }
  }
}
impl_neg!(MclG1);
impl_neg!(&MclG1);

#[cfg(test)]
mod tests {
  use super::*;
  use crate::building_block::curves::mcl::mcl_initializer::MclInitializer;

  #[test]
  fn equality() {
    MclInitializer::init();

    let g = MclG1::g();
    let g2 = &g + &g;

    assert_eq!(&g, &g);
    assert_eq!(&g2, &g2);
    assert_ne!(&g, &g2);
  }

  #[test]
  fn add() {
    MclInitializer::init();

    let g = &MclG1::g();
    let g2 = &(g + g);
    let g4 = &(g2 + g2);

    {
      let act = g + g;
      let exp = g2;
      assert_eq!(&act, exp);
    }
    {
      let act = g2 + g2;
      let exp = g4;
      assert_eq!(&act, exp);
    }
  }

  #[test]
  fn scalar_mul() {
    MclInitializer::init();

    let g = &MclG1::g();
    let n4 = MclFr::from(4);
    let act = g * n4;
    let exp = g + g + g + g;

    assert_eq!(act, exp);
  }

  #[test]
  fn neg() {
    MclInitializer::init();

    let g = &MclG1::g();
    let n4 = &MclFr::from(4);
    let g_n4 = g * n4;
    let g_n4_neg = (g * n4).neg();
    let act = g_n4 + g_n4_neg;  
    let exp = MclG1::zero();
    assert_eq!(act, exp);
  }
}
