use std::{
  fmt,
  convert::From,
  ops::{Add, Sub, Mul, Neg},
};
use crate::building_block::{
  curves::bls12_381::{
    reduce::Reduce,
    fq2::Fq2,
  },
  to_biguint::ToBigUint,
  zero::Zero,
};

#[derive(Debug, Clone)]
pub struct Fq6 {
  pub v2: Fq2,
  pub v1: Fq2,
  pub v0: Fq2,
}

impl Fq6 {
  pub fn inv(&self) -> Self {
    let t0 = &self.v0 * &self.v0 - Fq2::reduce(&(&self.v1 * &self.v2));
    let t1 = Fq2::reduce(&(&self.v2 * &self.v2)) - &self.v0 * &self.v1;
    let t2 = &self.v1 * &self.v1 - &self.v0 * &self.v2;
    let factor = Fq2::inv(&(
      &self.v0 * &t0
      + Fq2::reduce(&(&self.v2 * &t1))
      + Fq2::reduce(&(&self.v1 * &t2))
    ));
    Self {
      v2: &t2 * &factor,
      v1: &t1 * &factor,
      v0: &t0 * &factor,
    }
  }
}

impl Zero<Fq6> for Fq6 {
  fn zero() -> Self {
    Self {
      v2: Fq2::zero(),
      v1: Fq2::zero(),
      v0: Fq2::zero(),
    }
  }

  fn is_zero(&self) -> bool {
    true
  }
}

impl Reduce for Fq6 {
  fn reduce(&self) -> Self {
    Self {
      v2: self.v1.clone(),
      v1: self.v0.clone(),
      v0: Fq2::reduce(&self.v2.clone()),
    }
  }
}

impl Fq6 {
  pub fn new(v2: &Fq2, v1: &Fq2, v0: &Fq2) -> Self {
    Fq6 {
      v2: v2.clone(),
      v1: v1.clone(),
      v0: v0.clone(),
    }
  }
}

impl Neg for Fq6 {
  type Output = Fq6;

  fn neg(self) -> Self::Output {
    Fq6::zero() - self
  }
}

impl From<&dyn ToBigUint> for Fq6 {
  fn from(n: &dyn ToBigUint) -> Self {
    Fq6::new(
      &Fq2::zero(),
      &Fq2::zero(),
      &Fq2::from(n),
    )
  }
}

impl PartialEq for Fq6 {
  fn eq(&self, other: &Self) -> bool {
    self.v2 == other.v2 &&
    self.v1 == other.v1 &&
    self.v0 == other.v0  
  }
}

impl Eq for Fq6 {}

impl fmt::Display for Fq6 {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}\n{}\n{}", self.v0, self.v1, self.v2)
  }
}

macro_rules! impl_add {
  ($rhs: ty, $target: ty) => {
    impl Add<$rhs> for $target {
      type Output = Fq6;

      fn add(self, rhs: $rhs) -> Self::Output {
        Fq6 {
          v2: &self.v2 + &rhs.v2,
          v1: &self.v1 + &rhs.v1,
          v0: &self.v0 + &rhs.v0,
        }
      }
    }
  };
}
impl_add!(Fq6, Fq6);
impl_add!(Fq6, &Fq6);
impl_add!(&Fq6, Fq6);
impl_add!(&Fq6, &Fq6);

macro_rules! impl_sub {
  ($rhs: ty, $target: ty) => {
    impl Sub<$rhs> for $target {
      type Output = Fq6;

      fn sub(self, rhs: $rhs) -> Self::Output {
        Fq6 {
          v2: &self.v2 - &rhs.v2,
          v1: &self.v1 - &rhs.v1,
          v0: &self.v0 - &rhs.v0,
        }
      }
    }
  };
}
impl_sub!(Fq6, Fq6);
impl_sub!(Fq6, &Fq6);
impl_sub!(&Fq6, Fq6);
impl_sub!(&Fq6, &Fq6);

macro_rules! impl_mul {
  ($rhs: ty, $target: ty) => {
    impl Mul<$rhs> for $target {
      type Output = Fq6;

      fn mul(self, rhs: $rhs) -> Self::Output {
        let t0 = &self.v0 * &rhs.v0;
        let t1 = &self.v0 * &rhs.v1 + &self.v1 * &rhs.v0;
        let t2 = &self.v0 * &rhs.v2 + &self.v1 * &rhs.v1 + &self.v2 * &rhs.v0;
        let t3 = Fq2::reduce(&(&self.v1 * &rhs.v2 + &self.v2 * &rhs.v1));
        let t4 = Fq2::reduce(&(&self.v2 * &rhs.v2));
        Fq6 {
          v2: t2,
          v1: t1 + t4,
          v0: t0 + t3,
        }
      }
    }
  };
}
impl_mul!(Fq6, Fq6);
impl_mul!(Fq6, &Fq6);
impl_mul!(&Fq6, Fq6);
impl_mul!(&Fq6, &Fq6);

#[cfg(test)]
mod tests {
  use super::*;
  use crate::building_block::curves::bls12_381::fq_test_helper::get_fq2_values;

  fn to_strs(x: &Fq6) -> [String; 6] {
    [
      x.v2.u1.e.to_string(),
      x.v2.u0.e.to_string(),
      x.v1.u1.e.to_string(),
      x.v1.u0.e.to_string(),
      x.v0.u1.e.to_string(),
      x.v0.u0.e.to_string(),
    ]
  }

  #[test]
  fn test_add() {
    let (a2, b2, c2, d2) = get_fq2_values();
    let a6 = Fq6::new(&a2, &b2, &c2);
    let b6 = Fq6::new(&b2, &c2, &d2);
    let x = a6 + b6;
    let [v2_u1, v2_u0, v1_u1, v1_u0, v0_u1, v0_u0] = to_strs(&x);
    assert_eq!(v2_u1, "4002409555221667393417789825735904156556882819939007885332058136124031650490837864442687629129015664037894272559779");
    assert_eq!(v2_u0, "4002409555221667393417789825735904156556882819939007885332058136124031650490837864442687629129015664037894272559775");
    assert_eq!(v1_u1, "4002409555221667393417789825735904156556882819939007885332058136124031650490837864442687629129015664037894272559775");
    assert_eq!(v1_u0, "4002409555221667393417789825735904156556882819939007885332058136124031650490837864442687629129015664037894272559771");
    assert_eq!(v0_u1, "4002409555221667393417789825735904156556882819939007885332058136124031650490837864442687629129015664037894272559771");
    assert_eq!(v0_u0, "4002409555221667393417789825735904156556882819939007885332058136124031650490837864442687629129015664037894272559775");
  }

  #[test]
  fn test_sub() {
    let (a2, b2, c2, d2) = get_fq2_values();
    let a6 = Fq6::new(&a2, &b2, &c2);
    let b6 = Fq6::new(&b2, &c2, &d2);
    let x = a6 - b6;
    let [v2_u1, v2_u0, v1_u1, v1_u0, v0_u1, v0_u0] = to_strs(&x);
    assert_eq!(v2_u1, "2");
    assert_eq!(v2_u0, "2");
    assert_eq!(v1_u1, "2");
    assert_eq!(v1_u0, "2");
    assert_eq!(v0_u1, "2");
    assert_eq!(v0_u0, "4002409555221667393417789825735904156556882819939007885332058136124031650490837864442687629129015664037894272559781");
  }

  #[test]
  fn test_mul() {
    let (a2, b2, c2, d2) = get_fq2_values();
    let a6 = Fq6::new(&a2, &b2, &c2);
    let b6 = Fq6::new(&b2, &c2, &d2);
    let x = a6 * b6;
    let [v2_u1, v2_u0, v1_u1, v1_u0, v0_u1, v0_u0] = to_strs(&x);
    assert_eq!(v2_u1, "242");
    assert_eq!(v2_u0, "44");
    assert_eq!(v1_u1, "270");
    assert_eq!(v1_u0, "4002409555221667393417789825735904156556882819939007885332058136124031650490837864442687629129015664037894272559769");
    assert_eq!(v0_u1, "282");
    assert_eq!(v0_u0, "4002409555221667393417789825735904156556882819939007885332058136124031650490837864442687629129015664037894272559667");
  }

  #[test]
  fn test_inv() {
    let (a2, b2, c2, d2) = get_fq2_values();
    let a6 = Fq6::new(&a2, &b2, &c2);
    let b6 = Fq6::new(&b2, &c2, &d2);
    {
      let x = Fq6::inv(&a6);
      let [v2_u1, v2_u0, v1_u1, v1_u0, v0_u1, v0_u0] = to_strs(&x);
      assert_eq!(v2_u1, "688711608536765545023681885557136166034180917133277389073094396151295447729546742336123449607265527254827867222198");
      assert_eq!(v2_u0, "2003812238392905611942661335101007273544583728486455899017410569934237090284613467430039416006044773252968021328339");
      assert_eq!(v1_u1, "1384732656315076950784643413767590994207875546830469278737766445091932474503392919437606699976872316001747067940858");
      assert_eq!(v1_u0, "2372233623322050487349756306030560191559020503857093797256052613158418784920969188147209724606157830564869795632468");
      assert_eq!(v0_u1, "3645529390147594929701136395837907020867994011774550777501170281514599817503375988091874900679967756450347235355335");
      assert_eq!(v0_u0, "9703173730005323902704554867270972547775513169640886750222965983511920719625565448752484052932551805290834524543");
    }
    {
      let x = Fq6::inv(&b6);
      let [v2_u1, v2_u0, v1_u1, v1_u0, v0_u1, v0_u0] = to_strs(&x);
      assert_eq!(v2_u1, "3392002946475619471886146574727640271663098634029472440148157218371051887763101585552501551582028166119448731293598");
      assert_eq!(v2_u0, "228466217875481651322428561768669945482162590034752087898328550509215266712199196599713393340779809732559957022809");
      assert_eq!(v1_u1, "731172437583871066247513734403023027638859945898527980483076154302993864736397781362302499134681329972534715571822");
      assert_eq!(v1_u0, "2071095931610344534814189353424681897088300000967209144643543599181364483021457934393054022241416970836467436489378");
      assert_eq!(v0_u1, "2669331993693225396189212829892977465262313676792637153887698068493586989081407652085605543778084823583104065043032");
      assert_eq!(v0_u0, "807596311483464223407464377470177016452713001867849673810601646717748991164998687706272431728894395977864211575535");
    }
  }

  #[test]
  fn test_reduce() {
    let (a2, b2, c2, d2) = get_fq2_values();
    let a6 = Fq6::new(&a2, &b2, &c2);
    let b6 = Fq6::new(&b2, &c2, &d2);

    let x = Fq6::reduce(&(&a6 * &b6));
    let [v2_u1, v2_u0, v1_u1, v1_u0, v0_u1, v0_u0] = to_strs(&x);
    assert_eq!(v2_u1, "270");
    assert_eq!(v2_u0, "4002409555221667393417789825735904156556882819939007885332058136124031650490837864442687629129015664037894272559769");
    assert_eq!(v1_u1, "282");
    assert_eq!(v1_u0, "4002409555221667393417789825735904156556882819939007885332058136124031650490837864442687629129015664037894272559667");
    assert_eq!(v0_u1, "286");
    assert_eq!(v0_u0, "4002409555221667393417789825735904156556882819939007885332058136124031650490837864442687629129015664037894272559589");
  }
}
