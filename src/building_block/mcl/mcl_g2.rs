use mcl_rust::*;
use std::ops::{Add, Mul, Neg, AddAssign};
use num_traits::Zero;
use once_cell::sync::Lazy;
use crate::building_block::mcl::mcl_fr::MclFr;

#[derive(Clone, Debug)]
pub struct MclG2 {
  pub v: G2,
}

static GENERATOR: Lazy<MclG2> = Lazy::new(|| {
  let serialized_g = "1 352701069587466618187139116011060144890029952792775240219908644239793785735715026873347600343865175952761926303160 3059144344244213709971259814753781636986470325476647558659373206291635324768958432433509563104347017837885763365758 1985150602287291935568054521177171638300868978215655730859378665066344726373823718423869104263333984641494340347905 927553665492332455747201965776037880757740193453592970025027978793976877002675564980949289727957565575433344219582"; 
  let mut v = G2::zero();
  G2::set_str(&mut v, serialized_g, 10);
  MclG2 { v }
});

impl MclG2 {
  pub fn new() -> Self {
    let v = G2::zero();
    MclG2::from(&v)
  }

  pub fn g() -> MclG2 {
    GENERATOR.clone()
  }

  pub fn inv(&self) -> Self {
    let mut v = G2::zero();
    G2::neg(&mut v, &self.v);
    MclG2::from(&v)
  }

  pub fn get_random_point() -> MclG2 {
    let mut v = Fr::zero();
    v.set_by_csprng();
    MclG2::g() * MclFr::from(&v)
  }

  pub fn hash_and_map(buf: &Vec<u8>) -> MclG2 {
    let mut v = G2::zero();
    G2::set_hash_of(&mut v, buf);
    MclG2::from(&v)
  }
}

impl Zero for MclG2 {
  fn zero() -> MclG2 {
    let v = G2::zero();
    MclG2::from(&v)
  }

  fn is_zero(&self) -> bool {
    self.v.is_zero()
  }
}

impl From<&G2> for MclG2 {
  fn from(v: &G2) -> Self {
    MclG2 { v: v.clone() }
  }
}

macro_rules! impl_add {
  ($rhs: ty, $target: ty) => {
    impl Add<$rhs> for $target {
      type Output = MclG2;

      fn add(self, rhs: $rhs) -> Self::Output {
        let mut v = G2::zero();
        G2::add(&mut v, &self.v, &rhs.v);
        MclG2::from(&v)
      }
    }
  };
}
impl_add!(MclG2, MclG2);
impl_add!(&MclG2, MclG2);
impl_add!(MclG2, &MclG2);
impl_add!(&MclG2, &MclG2);

macro_rules! impl_mul {
  ($rhs: ty, $target: ty) => {
    impl Mul<$rhs> for $target {
      type Output = MclG2;

      fn mul(self, rhs: $rhs) -> Self::Output {
        let mut v = G2::zero();
        G2::mul(&mut v, &self.v, &rhs.v);
        MclG2::from(&v)
      }
    }
  };
}
impl_mul!(MclFr, MclG2);
impl_mul!(&MclFr, MclG2);
impl_mul!(MclFr, &MclG2);
impl_mul!(&MclFr, &MclG2);

impl AddAssign<MclG2> for MclG2 {
  fn add_assign(&mut self, rhs: MclG2) {
    *self = &*self + rhs
  }
}

impl PartialEq for MclG2 {
  fn eq(&self, rhs: &Self) -> bool {
    self.v == rhs.v
  }
}

impl Eq for MclG2 {}

macro_rules! impl_neg {
  ($target: ty) => {
    impl Neg for $target {
      type Output = MclG2;

      fn neg(self) -> Self::Output {
        let mut v = G2::zero();
        G2::neg(&mut v, &self.v);
        MclG2::from(&v)
      }
    }
  }
}
impl_neg!(MclG2);
impl_neg!(&MclG2);

#[cfg(test)]
mod tests {
  use super::*;
  use crate::building_block::mcl::mcl_initializer::MclInitializer;

  #[test]
  fn equality() {
    MclInitializer::init();

    let g = MclG2::g();
    println!("g {:?}", &g);
    let g2 = &g + &g;

    assert_eq!(&g, &g);
    assert_eq!(&g2, &g2);
    assert_ne!(&g, &g2);
  }

  #[test]
  fn add() {
    MclInitializer::init();

    let g = &MclG2::g();
    println!("g is {:?}", &g);
    let g2 = &(g + g);
    let g4 = &(g2 + g2);

    {
      let act = g + g;
      let exp = g2;
      assert_eq!(&act, exp);
    }
    {
      let act = g + g;
      let exp = g * MclFr::from(2);
      assert_eq!(&act, &exp);
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

    let g = &MclG2::g();
    let n4 = MclFr::from(4);
    let act = g * n4;
    let exp = g + g + g + g;

    assert_eq!(act, exp);
  }

  #[test]
  fn neg() {
    MclInitializer::init();

    let g = &MclG2::g();
    let n4 = &MclFr::from(4);
    let g_n4 = g * n4;
    let g_n4_neg = (g * n4).neg();
    let act = g_n4 + g_n4_neg;  
    let exp = MclG2::zero();
    assert_eq!(act, exp);
  }
}
