use std::{
  convert::From,
  fmt,
  ops::{Add, Sub, Mul},
};
use crate::building_block::{
  curves::bls12_381::{
    reduce::Reduce,
    fq1::Fq1,
    fq6::Fq6,
  },
  to_biguint::ToBigUint,
  zero::Zero,
};


#[derive(Debug, Clone)]
pub struct Fq12 {
  pub w1: Fq6,
  pub w0: Fq6,
}

impl Fq12 {
  pub fn new(w1: &Fq6, w0: &Fq6) -> Self {
      Fq12 {
        w1: w1.clone(),
        w0: w0.clone(),
      }
  }

  pub fn inv(&self) -> Self {
    let factor = Fq6::inv(&(
      &self.w0 * &self.w0
      - Fq6::reduce(&(&self.w1 * &self.w1))
    ));
    Self {
      w1: -self.w1.clone() * &factor,
      w0: &self.w0 * &factor,
    }
  }
}

impl Zero<Fq12> for Fq12 {
  fn zero() -> Self {
    Self {
      w1: Fq6::zero(),
      w0: Fq6::zero(),
    }
  }

  fn is_zero(&self) -> bool {
    true
  }
}

impl Reduce for Fq12 {
  fn reduce(&self) -> Self {
    panic!("Not implemented");
  }
}

impl From<&dyn ToBigUint> for Fq12 {
  fn from(n: &dyn ToBigUint) -> Self {
    Fq12::new(
      &Fq6::zero(),
      &Fq6::from(n),
    )
  }
}

macro_rules! impl_add {
  ($rhs: ty, $target: ty) => {
    impl Add<$rhs> for $target {
      type Output = Fq12;

      fn add(self, rhs: $rhs) -> Self::Output {
        Fq12 {
          w1: &self.w1 + &rhs.w1,
          w0: &self.w0 + &rhs.w0,
        }
      }
    }
  };
}
impl_add!(Fq12, Fq12);
impl_add!(Fq12, &Fq12);
impl_add!(&Fq12, Fq12);
impl_add!(&Fq12, &Fq12);

macro_rules! impl_sub {
  ($rhs: ty, $target: ty) => {
    impl Sub<$rhs> for $target {
      type Output = Fq12;

      fn sub(self, rhs: $rhs) -> Self::Output {
        Fq12 {
          w1: &self.w1 - &rhs.w1,
          w0: &self.w0 - &rhs.w0,
        }
      }
    }
  };
}
impl_sub!(Fq12, Fq12);
impl_sub!(Fq12, &Fq12);
impl_sub!(&Fq12, Fq12);
impl_sub!(&Fq12, &Fq12);

macro_rules! impl_mul {
  ($rhs: ty, $target: ty) => {
    impl Mul<$rhs> for $target {
      type Output = Fq12;

      fn mul(self, rhs: $rhs) -> Self::Output {
        Fq12 {
          w1: &self.w1 * &rhs.w0 + &self.w0 * &rhs.w1,
          w0: &self.w0 * &rhs.w0 + Fq6::reduce(&(&self.w1 * &rhs.w1))
        }
      }
    }
  };
}
impl_mul!(Fq12, Fq12);
impl_mul!(Fq12, &Fq12);
impl_mul!(&Fq12, Fq12);
impl_mul!(&Fq12, &Fq12);

impl fmt::Display for Fq12 {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{{ w1: {}, w0: {} }}", self.w1, self.w0)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::building_block::curves::bls12_381::fq_test_helper::get_fq6_values;

  fn to_strs(x: &Fq12) -> [String; 12] {
    [
      x.w1.v2.u1.e.to_string(),
      x.w1.v2.u0.e.to_string(),
      x.w1.v1.u1.e.to_string(),
      x.w1.v1.u0.e.to_string(),
      x.w1.v0.u1.e.to_string(),
      x.w1.v0.u0.e.to_string(),

      x.w0.v2.u1.e.to_string(),
      x.w0.v2.u0.e.to_string(),
      x.w0.v1.u1.e.to_string(),
      x.w0.v1.u0.e.to_string(),
      x.w0.v0.u1.e.to_string(),
      x.w0.v0.u0.e.to_string(),
    ]
  }

  #[test]
  fn test_add() {
    let (a6, b6, c6, d6) = get_fq6_values();
    let a12 = Fq12::new(&a6, &b6);
    let b12 = Fq12::new(&c6, &d6);
    let x = a12 + b12;
    let [
      w1_v2_u1, w1_v2_u0, w1_v1_u1, w1_v1_u0, w1_v0_u1, w1_v0_u0,
      w0_v2_u1, w0_v2_u0, w0_v1_u1, w0_v1_u0, w0_v0_u1, w0_v0_u0,
    ] = to_strs(&x);
    assert_eq!(w1_v2_u1, "4002409555221667393417789825735904156556882819939007885332058136124031650490837864442687629129015664037894272559777");
    assert_eq!(w1_v2_u0, "4002409555221667393417789825735904156556882819939007885332058136124031650490837864442687629129015664037894272559773");
    assert_eq!(w1_v1_u1, "4002409555221667393417789825735904156556882819939007885332058136124031650490837864442687629129015664037894272559773");
    assert_eq!(w1_v1_u0, "4002409555221667393417789825735904156556882819939007885332058136124031650490837864442687629129015664037894272559777");
    assert_eq!(w1_v0_u1, "4002409555221667393417789825735904156556882819939007885332058136124031650490837864442687629129015664037894272559777");
    assert_eq!(w1_v0_u0, "4002409555221667393417789825735904156556882819939007885332058136124031650490837864442687629129015664037894272559773");

    assert_eq!(w0_v2_u1, "4002409555221667393417789825735904156556882819939007885332058136124031650490837864442687629129015664037894272559773");
    assert_eq!(w0_v2_u0, "4002409555221667393417789825735904156556882819939007885332058136124031650490837864442687629129015664037894272559777");
    assert_eq!(w0_v1_u1, "4002409555221667393417789825735904156556882819939007885332058136124031650490837864442687629129015664037894272559777");
    assert_eq!(w0_v1_u0, "4002409555221667393417789825735904156556882819939007885332058136124031650490837864442687629129015664037894272559773");
    assert_eq!(w0_v0_u1, "4002409555221667393417789825735904156556882819939007885332058136124031650490837864442687629129015664037894272559773");
    assert_eq!(w0_v0_u0, "4002409555221667393417789825735904156556882819939007885332058136124031650490837864442687629129015664037894272559777");
  }

  #[test]
  fn test_sub() {
    let (a6, b6, c6, d6) = get_fq6_values();
    let a12 = Fq12::new(&a6, &b6);
    let b12 = Fq12::new(&c6, &d6);
    let x = a12 - b12;
    let [
      w1_v2_u1, w1_v2_u0, w1_v1_u1, w1_v1_u0, w1_v0_u1, w1_v0_u0,
      w0_v2_u1, w0_v2_u0, w0_v1_u1, w0_v1_u0, w0_v0_u1, w0_v0_u0,
    ] = to_strs(&x);

    assert_eq!(w1_v2_u1, "4");
    assert_eq!(w1_v2_u0, "4");
    assert_eq!(w1_v1_u1, "4");
    assert_eq!(w1_v1_u0, "4002409555221667393417789825735904156556882819939007885332058136124031650490837864442687629129015664037894272559783");
    assert_eq!(w1_v0_u1, "4002409555221667393417789825735904156556882819939007885332058136124031650490837864442687629129015664037894272559783");
    assert_eq!(w1_v0_u0, "4002409555221667393417789825735904156556882819939007885332058136124031650490837864442687629129015664037894272559783");

    assert_eq!(w0_v2_u1, "4");
    assert_eq!(w0_v2_u0, "4002409555221667393417789825735904156556882819939007885332058136124031650490837864442687629129015664037894272559783");
    assert_eq!(w0_v1_u1, "4002409555221667393417789825735904156556882819939007885332058136124031650490837864442687629129015664037894272559783");
    assert_eq!(w0_v1_u0, "4002409555221667393417789825735904156556882819939007885332058136124031650490837864442687629129015664037894272559783");
    assert_eq!(w0_v0_u1, "4002409555221667393417789825735904156556882819939007885332058136124031650490837864442687629129015664037894272559783");
    assert_eq!(w0_v0_u0, "4");
  }

  #[test]
  fn test_mul() {
    let (a6, b6, c6, d6) = get_fq6_values();
    let a12 = Fq12::new(&a6, &b6);
    let b12 = Fq12::new(&c6, &d6);
    let x = a12 * b12;
    let [
      w1_v2_u1, w1_v2_u0, w1_v1_u1, w1_v1_u0, w1_v0_u1, w1_v0_u0,
      w0_v2_u1, w0_v2_u0, w0_v1_u1, w0_v1_u0, w0_v0_u1, w0_v0_u0,
    ] = to_strs(&x);

    assert_eq!(w1_v2_u1, "444");
    assert_eq!(w1_v2_u0, "4002409555221667393417789825735904156556882819939007885332058136124031650490837864442687629129015664037894272559739");
    assert_eq!(w1_v1_u1, "412");
    assert_eq!(w1_v1_u0, "4002409555221667393417789825735904156556882819939007885332058136124031650490837864442687629129015664037894272559655");
    assert_eq!(w1_v0_u1, "460");
    assert_eq!(w1_v0_u0, "4002409555221667393417789825735904156556882819939007885332058136124031650490837864442687629129015664037894272559491");

    assert_eq!(w0_v2_u1, "420");
    assert_eq!(w0_v2_u0, "4002409555221667393417789825735904156556882819939007885332058136124031650490837864442687629129015664037894272559709");
    assert_eq!(w0_v1_u1, "428");
    assert_eq!(w0_v1_u0, "4002409555221667393417789825735904156556882819939007885332058136124031650490837864442687629129015664037894272559593");
    assert_eq!(w0_v0_u1, "468");
    assert_eq!(w0_v0_u0, "4002409555221667393417789825735904156556882819939007885332058136124031650490837864442687629129015664037894272559389");
  }

  #[test]
  fn test_inv() {
    let (a6, b6, c6, d6) = get_fq6_values();
    let a12 = Fq12::new(&a6, &b6);
    let b12 = Fq12::new(&c6, &d6);
    {
      let x = Fq12::inv(&a12);
      let [
        w1_v2_u1, w1_v2_u0, w1_v1_u1, w1_v1_u0, w1_v0_u1, w1_v0_u0,
        w0_v2_u1, w0_v2_u0, w0_v1_u1, w0_v1_u0, w0_v0_u1, w0_v0_u0,
      ] = to_strs(&x);
      assert_eq!(w1_v2_u1, "3098505419743608422343793785504422428717775169252111988990844334159729886446050791768601352995003841275882837991531");
      assert_eq!(w1_v2_u0, "2076491679423606188134486077726045418717620003339170785401381327924469611526758053948834153806157806069997654501659");
      assert_eq!(w1_v1_u1, "136651450238200480235822125397195489650501674765117362125810051518711063477932833979335305727513201470942198973964");
      assert_eq!(w1_v1_u0, "986021327812298143182761856176801052406609326921050610929669634405702210817960479083269858109659107242886085038502");
      assert_eq!(w1_v0_u1, "2361991256477500073090234150653452955387304731412829397779018324591132456688011685857929106418430974433096130304408");
      assert_eq!(w1_v0_u0, "3301913711066851237860386808398456824548985219709125502085597096709687367261755381032760516947013394179429914317166");

      assert_eq!(w0_v2_u1, "2735120901175551405725271898737038209743302486406160522481313863439924023674265853754387530906732613436036097562128");
      assert_eq!(w0_v2_u0, "2041151484916299799993345987297775565682434754689797926629688706062605509137272992342105696386963697577548495681728");
      assert_eq!(w0_v1_u1, "1066964309639494305965857493216329383621929664870705281019471753053551847754213147761827050582781219781473586872225");
      assert_eq!(w0_v1_u0, "3614967602572825056482502955271252417649129707111454921801816342139715436020101631284535392463927375230344418096303");
      assert_eq!(w0_v0_u1, "1136311109186095407653514236545320987393832306751419325363915933403041926113951049413993578612033751156354074602106");
      assert_eq!(w0_v0_u0, "2933466746795546952319734955495670198600278612090745499825161686702957100983663163319418628075072620235327963680617");
    }
    {
      let x = Fq12::inv(&b12);
      let [
        w1_v2_u1, w1_v2_u0, w1_v1_u1, w1_v1_u0, w1_v0_u1, w1_v0_u0,
        w0_v2_u1, w0_v2_u0, w0_v1_u1, w0_v1_u0, w0_v0_u1, w0_v0_u0,
      ] = to_strs(&x);
      assert_eq!(w1_v2_u1, "2214913490991914559310045130883528362288262902918037901495283151650297147332899794331935074215191670712184504494075");
      assert_eq!(w1_v2_u0, "168082411235750587545946012096931645426879193116452342116176814689575333689029657031257656735641267771712714738938");
      assert_eq!(w1_v1_u1, "2773859331476829282540242452166539256262949035890482290348858662545280594736413848171277177981370850102253517490876");
      assert_eq!(w1_v1_u0, "238928992206688770472384912413913779153031130311883835843793666256412855109522774440984534136563764879133913802614");
      assert_eq!(w1_v0_u1, "127518151698247185636724953250392108083866968207265005924796069887021952400804018547495003234839062094779252576231");
      assert_eq!(w1_v0_u0, "1355620439567130908019886969836306387337017770829254582675654118149872324625873963474721578171448162976322572459147");

      assert_eq!(w0_v2_u1, "3700407954026359777693870921798779542881236849196746802718799794231946193381636472627170948911366705994920724128040");
      assert_eq!(w0_v2_u0, "247736457514033285744909859652987438402751928954458259804679648138720706288338278587363031034812055829407648138719");
      assert_eq!(w0_v1_u1, "3183340088589082597865579398799195559513407104734944229960386259873960048510982093357871603614284353795487273975318");
      assert_eq!(w0_v1_u0, "1367966003894733147864707780944922318207773962105052173630678072555981852039367163173267529068007747320116096853333");
      assert_eq!(w0_v0_u1, "2612903860988610859801596132693961538603256869092748567354952881913115729740383996647152330276625278540609481079644");
      assert_eq!(w0_v0_u0, "3420341227371213438340276582737529650673194075953347974574175355309696854464626210382743124271864669941561008888202");
    }
  }
}
