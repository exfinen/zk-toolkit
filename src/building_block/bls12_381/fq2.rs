use std::ops::{Add, Sub, Mul};
use crate::building_block::bls12_381::additional_ops::AdditionalOps;
use crate::building_block::bls12_381::fq1::Fq1;

#[derive(Debug, Clone)]
pub struct Fq2 {
    u1: Fq1,
    u0: Fq1,
}

impl Fq2 {
    pub fn new(u1: &Fq1, u0: &Fq1) -> Self {
        Fq2 { u1: u1.clone(), u0: u0.clone() }
    }
}

impl AdditionalOps for Fq2 {
  fn apply_reduce_rule(n: &Self) -> Self {
    Self {
      u1: &n.u1 + &n.u0,
      u0: &n.u0 - &n.u1,
    }
  }

  fn inv(n: &Self) -> Self {
    let factor = &(&n.u1 * &n.u1 + &n.u0 * &n.u0).inv();
    Self {
      u1: n.u1.negate() * factor,
      u0: &n.u0 * factor,
    }
  }

  fn zero() -> Self {
      Self {
        u1: Fq1::zero(),
        u0: Fq1::zero(),
      }
  }
}

macro_rules! impl_add {
  ($rhs: ty, $target: ty) => {
    impl<'a> Add<$rhs> for $target {
      type Output = Fq2;

      fn add(self, rhs: $rhs) -> Self::Output {
        Fq2 {
          u1: &self.u1 + &rhs.u1,
          u0: &self.u0 + &rhs.u0,
        }
      }
    }
  };
}
impl_add!(Fq2, Fq2);
impl_add!(Fq2, &Fq2);
impl_add!(&Fq2, Fq2);
impl_add!(&Fq2, &Fq2);

macro_rules! impl_sub {
  ($rhs: ty, $target: ty) => {
    impl<'a> Sub<$rhs> for $target {
      type Output = Fq2;

      fn sub(self, rhs: $rhs) -> Self::Output {
        Fq2 {
          u1: &self.u1 - &rhs.u1,
          u0: &self.u0 - &rhs.u0,
        }
      }
    }
  };
}
impl_sub!(Fq2, Fq2);
impl_sub!(Fq2, &Fq2);
impl_sub!(&Fq2, Fq2);
impl_sub!(&Fq2, &Fq2);

macro_rules! impl_mul {
  ($rhs: ty, $target: ty) => {
    impl<'a> Mul<$rhs> for $target {
      type Output = Fq2;

      fn mul(self, rhs: $rhs) -> Self::Output {
        Fq2 {
          u1: &self.u1 * &rhs.u0 + &self.u0 * &rhs.u1,
          u0: &self.u0 * &rhs.u0 - &self.u1 * &rhs.u1,
        }
      }
    }
  };
}
impl_mul!(Fq2, Fq2);
impl_mul!(Fq2, &Fq2);
impl_mul!(&Fq2, Fq2);
impl_mul!(&Fq2, &Fq2);

#[cfg(test)]
mod tests {
  use crate::building_block::bls12_381::setup::BASE_FIELD;
  use crate::building_block::bls12_381::fq6::Fq6;
use super::*;

  /*
  let a1 = Fq1 3
  let b1 = Fq1 5
  let c1 = Fq1 7
  let d1 = Fq1 9
  let a1b1 = a1 * b1
  let b1c1 = b1 * c1
  let c1d1 = c1 * d1
  -- let inv_a1b1 = inv a1b1
  -- let inv_b1c1 = inv b1c1
  -- print inv_a1b1
  -- print inv_b1c1

  let a2 = Fq2 a1b1 b1c1
  let b2 = Fq2 b1c1 a1b1
  let c2 = Fq2 b1c1 c1d1
  -- let inv_a2 = inv a2
  -- let inv_b2 = inv b2
  -- print inv_a2
  -- print inv_b2
  let a6 = Fq6 a2 b2 c2
  let b6 = Fq6 b2 c2 a2
  let inv_a6 = inv a6
  print inv_a6
   */
  #[test]
  fn test_misc() {
    let f = &BASE_FIELD;
    let a1 = Fq1::new(f, &f.elem(&3u8));
    let b1 = Fq1::new(f, &f.elem(&5u8));
    let c1 = Fq1::new(f, &f.elem(&7u8));
    let d1 = Fq1::new(f, &f.elem(&9u8));
    let a1b1 = &a1 * &b1;
    let b1c1 = &b1 * &c1;
    let c1d1 = &c1 * &d1;
    // let inv_a1b1 = Fq1::inv(&a1b1);
    // let inv_b1c1 = Fq1::inv(&b1c1);
    // println!("inv_a1b1={:?}", inv_a1b1);
    // println!("inv_b1c1={:?}", inv_b1c1);
    // inv_a1b1:
    // 533654607362888985789038643431453887540917709325201051377607751483204220065445048592358350550535421871719236341305
    // 533654607362888985789038643431453887540917709325201051377607751483204220065445048592358350550535421871719236341305
    let a2 = Fq2::new(&a1b1, &b1c1);
    let b2 = Fq2::new(&b1c1, &a1b1);
    let c2 = Fq2::new(&b1c1, &c1d1);
    // let inv_a2 = Fq2::inv(&a2); let inv_b2 = Fq2::inv(&b2);
    // println!("inv_a2={:?}", &inv_a2);
    // println!("inv_b2={:?}", &inv_b2);
    // Haskell:
    // u1: 2194424549242224536391133042386237106526015063345869840578611184978348387682907656711680458729356864075948928748297
    // u0: 2884495162211477535256269219237255064553063825404319475980690173965250396388224529891454187889531978565241044706881}}
    // Rust:
    // u1: 2194424549242224536391133042386237106526015063345869840578611184978348387682907656711680458729356864075948928748297
    // u0: 2884495162211477535256269219237255064553063825404319475980690173965250396388224529891454187889531978565241044706881 } }

    let a6 = Fq6::new(&a2, &b2, &c2);
    // let b6 = Fq6::new(&b2, &c2, &a2);
    let inv_a6 = Fq6::inv(&a6);
    println!("inv_a6={:?}", &inv_a6);
  }
}