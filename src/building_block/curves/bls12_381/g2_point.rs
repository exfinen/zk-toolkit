use crate::{
  impl_affine_add,
  impl_scalar_mul_point,
  building_block::{
    field::prime_field::PrimeField,
    curves::{
      bls12_381::{
        fq1::Fq1,
        fq2::Fq2,
        params::Params as P,
        private_key::PrivateKey,
        reduce::Reduce,
      },
      rational_point::RationalPoint,
    },
    to_biguint::ToBigUint,
    zero::Zero,
  },
};
use num_bigint::{BigUint, RandBigInt};
use num_traits::Zero as NumTraitsZero;
use rand::SeedableRng;
use std::{
  ops::{Add, Mul, Neg, AddAssign},
  sync::Arc,
};
use once_cell::sync::Lazy;
use rand_chacha::ChaCha12Rng;

#[derive(Clone, Debug)]
pub enum G2Point {
  Rational { x: Fq2, y: Fq2 },
  AtInfinity,
}

static BASE_POINT: Lazy<G2Point> = Lazy::new(|| {
  let x1: Fq1 = Fq1::from_u8_slice(b"13e02b6052719f607dacd3a088274f65596bd0d09920b61ab5da61bbdc7f5049334cf11213945d57e5ac7d055d042b7e");
  let x0: Fq1 = Fq1::from_u8_slice(b"024aa2b2f08f0a91260805272dc51051c6e47ad4fa403b02b4510b647ae3d1770bac0326a805bbefd48056c8c121bdb8");
  let x = Fq2::new(&x1, &x0);

  let y1 = Fq1::from_u8_slice(b"0606c4a02ea734cc32acd2b02bc28b99cb3e287e85a763af267492ab572e99ab3f370d275cec1da1aaa9075ff05f79be");
  let y0 = Fq1::from_u8_slice(b"0ce5d527727d6e118cc9cdc6da2e351aadfd9baa8cbdd3a76d429a695160d12c923ac9cc3baca289e193548608b82801");
  let y = Fq2::new(&y1, &y0);

  G2Point::Rational { x, y }
});

impl G2Point {
  pub fn new(x: &Fq2, y: &Fq2) -> Self {
    G2Point::Rational { x: x.clone(), y: y.clone() }
  }

  pub fn g() -> Self {
    BASE_POINT.clone()
  }

  pub fn inv(&self) -> Self {
    match self {
      G2Point::AtInfinity => panic!("No inverse exists for point at infinitty"),
      G2Point::Rational { x, y } => G2Point::new(&x, &y.inv()),
    }
  }

  pub fn get_random_point() -> AffinePoint {
    let mut rng = ChaCha12Rng::from_entropy();
    let subgroup = &P::subgroup();
    let n = rng.gen_biguint_range(&NumTraitsZero::zero(), subgroup.order_ref());
    G2Point::g() * subgroup.elem(&n)
  }

  // for impl_scalar_mul_point macro
  pub fn curve_group() -> Arc<PrimeField> {
    P::subgroup()
  }

  pub fn is_on_curve(x: &Fq2, y: &Fq2) -> bool {
    let lhs = y * y;
    let four = Fq2::from(&4u8 as &dyn ToBigUint);
    let rhs = x * x * x + four.reduce();
    lhs == rhs
  }

  // TODO implement properly with hash-and-check or SWU map
  pub fn hash_to_g2point(buf: &Vec<u8>) -> G2Point {
    let n = BigUint::from_bytes_be(buf);
    let n = P::subgroup().elem(&n);
    G2Point::g() * n
  }
}

macro_rules! impl_neg {
  ($target: ty) => {
    impl Neg for $target {
      type Output = G2Point;

      fn neg(self) -> Self::Output {
        match self {
          G2Point::AtInfinity => G2Point::AtInfinity,
          G2Point::Rational { x, y } => {
            G2Point::new(&x, &y.neg())
          }
        }
      }
    }
  }
}
impl_neg!(G2Point);
impl_neg!(&G2Point);

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

impl<'a> Mul<&'a PrivateKey> for &G2Point {
  type Output = G2Point;

  fn mul(self, rhs: &PrivateKey) -> Self::Output {
    let rhs = P::subgroup().elem(&rhs.value);
    self * rhs
  }
}

type AffinePoint = G2Point;
impl_affine_add!(G2Point);
impl_scalar_mul_point!(Fq1, G2Point);

impl AddAssign<G2Point> for G2Point {
  fn add_assign(&mut self, rhs: G2Point) {
    *self = &*self + rhs
  }
}

impl PartialEq for G2Point {
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (G2Point::AtInfinity, G2Point::AtInfinity) => true,
      (G2Point::AtInfinity, G2Point::Rational { x: _x, y: _y }) => false,
      (G2Point::Rational { x: _x, y: _y }, G2Point::AtInfinity) => false,
      (G2Point::Rational { x: x1, y: y1 }, G2Point::Rational { x: x2, y: y2 }) => {
        x1.u1 == x2.u1
        && x1.u0 == x2.u0
        && y1.u1 == y2.u1
        && y1.u0 == y2.u0
      },
    }
  }
}

impl Eq for AffinePoint {}

#[cfg(test)]
mod tests {
  use super::*;
  use num_bigint::BigUint;
  use std::rc::Rc;

  #[test]
  fn hash_to_g2point() {
    let buf = b"robert kiyosaki".to_vec();
    G2Point::hash_to_g2point(&buf);
    //assert!(G2Point::is_on_curve(&p.x, &p.y));
  }

  #[test]
  fn scalar_mul() {
    let g = &G2Point::g();
    let f = P::subgroup();
    {
      let act = g * f.elem(&1u8);
      assert_eq!(&act, g);
    }
    {
      let act = g * f.elem(&2u8);
      let exp = g + g;
      assert_eq!(act, exp);
    }
    {
      let act = g * f.elem(&3u8);
      let exp = g + g + g;
      assert_eq!(act, exp);
    }
  }

  #[test]
  fn add_same_point() {
    let g = &G2Point::g();
    let g2 = g + g;

    let exp_x_u1 = BigUint::parse_bytes(
      b"1586560233067062236092888871453626466803933380746149805590083683748120990227823365075019078675272292060187343402359",
      10
    ).unwrap();
    let exp_x_u0 = BigUint::parse_bytes(
      b"3419974069068927546093595533691935972093267703063689549934039433172037728172434967174817854768758291501458544631891",
      10
    ).unwrap();
    let exp_y_u1 = BigUint::parse_bytes(
      b"2374407843478705782611042739236452317510200146460567463070514850492917978226342495167066333366894448569891658583283",
      10
    ).unwrap();
    let exp_y_u0 = BigUint::parse_bytes(
      b"678774053046495337979740195232911687527971909891867263302465188023833943429943242788645503130663197220262587963545",
      10
    ).unwrap();

    match g2 {
      G2Point::AtInfinity => panic!("expected rational point, but got point at infinity"),
      G2Point::Rational { x, y } => {
        assert_eq!(x.u1.e, exp_x_u1);
        assert_eq!(x.u0.e, exp_x_u0);
        assert_eq!(y.u1.e, exp_y_u1);
        assert_eq!(y.u0.e, exp_y_u0);
      },
    }
  }

  #[test]
  fn negate() {
    let g = &G2Point::g();
    let res = g + -g;

    match res {
      G2Point::AtInfinity => {},
      _ => panic!("expected point at infinity, but got rational point"),
    }
  }

  #[test]
  fn add_vertical_line() {
    let g = &G2Point::g();
    match g {
      G2Point::AtInfinity => panic!("expected rational point, but got point at infinity"),
      G2Point::Rational { x, y } => {
        let a = G2Point::new(&x, &y);
        let b = G2Point::new(&x, &-y);
        let exp = G2Point::AtInfinity;
        let act = &a + &b;
        assert_eq!(act, exp);
      },
    }
  }

  #[test]
  fn add_inf_and_affine() {
    let g = &G2Point::g();
    match g {
      G2Point::AtInfinity => panic!("expected rational point, but got point at infinity"),
      g => {
        let inf_plus_g = g + G2Point::AtInfinity;
        assert_eq!(g, &inf_plus_g);
      },
    }
  }

  #[test]
  fn add_affine_and_inf() {
    let g = &G2Point::g();
    match g {
      G2Point::AtInfinity => panic!("expected rational point, but got point at infinity"),
      g => {
        let g_plus_inf = G2Point::AtInfinity + g;
        assert_eq!(g, &g_plus_inf);
      },
    }
  }

  #[test]
  fn add_inf_and_inf() {
    let res = G2Point::AtInfinity + G2Point::AtInfinity;
    match res {
      G2Point::AtInfinity => (),
      _ => panic!("rational point not expected"),
    }
  }

  struct Xy<'a> {
    x1: &'a [u8],
    x0: &'a [u8],
    y1: &'a [u8],
    y0: &'a [u8],
  }

  impl<'a> Into<G2Point> for &Xy<'a> {
    fn into(self) -> G2Point {
      let f = P::base_prime_field();

      let x1 = BigUint::parse_bytes(self.x1, 10).unwrap();
      let x0 = BigUint::parse_bytes(self.x0, 10).unwrap();
      let y1 = BigUint::parse_bytes(self.y1, 10).unwrap();
      let y0 = BigUint::parse_bytes(self.y0, 10).unwrap();

      G2Point::new(
        &Fq2::new(
          &Fq1::new(&Rc::new(f.clone()), &x1),
          &Fq1::new(&Rc::new(f.clone()), &x0),
        ),
        &Fq2::new(
          &Fq1::new(&Rc::new(f.clone()), &y1),
          &Fq1::new(&Rc::new(f.clone()), &y0),
        ),
      )
    }
  }

  fn get_g_multiples<'a>() -> Vec<G2Point> {
    let ps = vec![
      Xy { x1: b"3059144344244213709971259814753781636986470325476647558659373206291635324768958432433509563104347017837885763365758", x0: b"352701069587466618187139116011060144890029952792775240219908644239793785735715026873347600343865175952761926303160", y1: b"927553665492332455747201965776037880757740193453592970025027978793976877002675564980949289727957565575433344219582", y0: b"1985150602287291935568054521177171638300868978215655730859378665066344726373823718423869104263333984641494340347905", },
      Xy { x1: b"1586560233067062236092888871453626466803933380746149805590083683748120990227823365075019078675272292060187343402359", x0: b"3419974069068927546093595533691935972093267703063689549934039433172037728172434967174817854768758291501458544631891", y1: b"2374407843478705782611042739236452317510200146460567463070514850492917978226342495167066333366894448569891658583283", y0: b"678774053046495337979740195232911687527971909891867263302465188023833943429943242788645503130663197220262587963545", },
      Xy { x1: b"1418901263980595683832511076652430035654903023556505873032297534993731256453342997202098832403658787934376638965468", x0: b"2795155019138475430256695697248607867022196082692926850257941893956680503583886174445899854256891620515274933186478", y1: b"1376945178829045108008380835987620979304438294788415956605678509674588356753313865659068546846109894276784773457993", y0: b"1713408536894110516522969272885192173669900392782465197506312048399987681703463801235485042423756235640603447122066", },
      Xy { x1: b"1078694598252689404244396341813443419551535268687482491181036693120116966808940002268138449409757966646650205799154", x0: b"2228261016665467246533331009551956115370380172154746298363804663874927447724251086262215215805607611831416839456087", y1: b"1156012089965243131925155359058270233117178570263500297221609542882736610397356630132228324008545337607514631821497", y0: b"1078130147839725710638550237346893815189184147577339719832678031476197662920373232750257163212369151522627886861080", },
      Xy { x1: b"151216712330486580381289676720993530468452734725315418939914686037671894984472908062266534423934877412152819881174", x0: b"626266753989782654150694692036924390988881741494970156941802666795495657949695370746846269655547812403809070556808", y1: b"1417335358548100222817200951198539764927940191545220572034594310270669391308385540419375417621016275437603665889670", y0: b"3957221353860521190838035852656308152792962079075169227140436352788803481025497873165648235984294733156170881957140", },
      Xy { x1: b"608866484358724393780037751128594231300217318157170134017274809385436427203952108566113171210510640978002034144337", x0: b"3984640847924757144714972801294669751518204805083279115209564409785869025937099631985667805381121749661329977337231", y1: b"1315309522598195877039488735068285781409574791250756219351720987511623939545034443010380218936398442802560800338945", y0: b"3638085773299732811703058316322950283836045952826105333022656670096922749263292332009255347182392556648551235304444", },
      Xy { x1: b"2002357927014343339248864414634364694493007010346797894329949366020574238568791702800705687329188574611271276704968", x0: b"709940604317203372084363045234008717826848775332345256708783709065481460296552174594695120412283630827121870605628", y1: b"912045267738927660774159947293138338745237549910946144646281482158519356186671009156889035570132788233623423316000", y0: b"1341746576224694386674361975424855739534560887571639474887265245206456367479326365108850910936317989017305100831965", },
      Xy { x1: b"2884924116662489256939453588959376395962356926980616944014706004012485984357534377214124571018891063114820242395361", x0: b"319952164230304133077458442661907403266453869104376212334739706313083480656267037961932179426199435482172341189037", y1: b"1075399299340417723830397866940785443167782131387985455780061730723063380264944719785035587414818747155809437751863", y0: b"3379313718443355656666805651360648422083076370000238420408683761882783147099178834203015601640804592083743639492910", },
      Xy { x1: b"1890785404699189181161569277356497622423785178845737858235714310995835974899880469355250933575450045792782146044819", x0: b"233289878585407360737561818812172281900488265436962145913969074168503452745466655442125797664134009339799716079103", y1: b"2751025411942897795042193940345989612527395984463172615380574492034129474560903255212585680112858672276592527763585", y0: b"1215754321684097939278683023199690844646077558342794977283698289191570128272085945598449054373022460634252133664610", },
      Xy { x1: b"2418374053012673557364841297962259218182001226941422304617523104967782677609763714086135085979083400722225782827409", x0: b"3630611692950647373061270855988056568581211618696406862713007087326556579692942998491068914373409337799654521763668", y1: b"2565033629757685347121048179025111379922439932508562385925537875057987528022635067215030488219761963195393172577303", y0: b"501530643738778464828020827177536510982498088982272277434672929233591213045783384231923436946952102224418164818781", },
    ];
    let mut gs: Vec<G2Point> = vec![];
    for p in ps {
      gs.push((&p).into());
    }
    gs
  }

  #[test]
  fn scalar_mul_smaller_nums() {
    let f = P::subgroup();
    let g = &G2Point::g();
    let gs = get_g_multiples();

    for n in 1usize..=10 {
      let res = g * f.elem(&n);
      assert_eq!(&res, &gs[n - 1]);
    }
  }

  struct ScalarMulTest<'a> {
    multiple: &'a [u8],
    p: &'a Xy<'a>,
  }

  #[test]
  fn scalar_mul_gen_pubkey() {
    let test_cases = vec![
      ScalarMulTest {
        multiple: b"12345",
        p: &Xy { x1: b"710263249623297945205677832422879367136859122336617013558836430937587842487752548203453568692909410481089040535739", x0: b"537981225545542963425487820538681006031270350819931841776188481854991512710445805907224339759290391174516449905299", y1: b"337049862749186367114588866279510224796235667392273765458486416725286123997827520576860003372467871782936277490523", y0: b"711110997734620889999929504948450308326434586997819988962419007364906703059928021844346255898626197228750480567836", },
      },
      // ScalarMulTest {
      //   multiple: b"1234567",
      //   p: &Xy {
      //     x1: b"1",
      //     x0: b"1",
      //     y1: b"1",
      //     y0: b"1",
      //   },
      // },
      // ScalarMulTest {
      //   multiple: b"1234567890123456789",
      //   p: &Xy {
      //     x1: b"1",
      //     x0: b"1",
      //     y1: b"1",
      //     y0: b"1",
      //   },
      // },
      // ScalarMulTest {
      //   multiple: b"123456789012345678901234567890",
      //   p: &Xy {
      //     x1: b"1",
      //     x0: b"1",
      //     y1: b"1",
      //     y0: b"1",
      //   },
      // },
    ];

    let f = P::subgroup();
    let g = &G2Point::g();

    for t in &test_cases {
      let exp: G2Point = t.p.into();
      let multiple = BigUint::parse_bytes(t.multiple, 10).unwrap();

      let act = g * f.elem(&multiple);
      assert_eq!(exp, act);
    }
  }

  // expects a + b = c
  struct AddTestCase {
    a: usize,
    b: usize,
    c: usize,
  }

  impl AddTestCase {
    fn new(a: usize, b: usize, c: usize) -> AddTestCase {
      if a + b != c {
        panic!("Bad add test case: {} + {} = {}", a, b, c);
      }
      AddTestCase { a, b, c }
    }
  }

  #[test]
  fn add_different_points() {
    let gs = get_g_multiples();

    let test_cases = [
      AddTestCase::new(1, 1, 2),
      AddTestCase::new(1, 2, 3),
      AddTestCase::new(2, 2, 4),
      AddTestCase::new(2, 6, 8),
      AddTestCase::new(3, 4, 7),
      AddTestCase::new(5, 1, 6),
      AddTestCase::new(5, 2, 7),
      AddTestCase::new(8, 1, 9),
      AddTestCase::new(9, 1, 10),
    ];

    for tc in test_cases {
      let lhs = &gs[tc.a - 1];
      let rhs = &gs[tc.b - 1];
      let exp = &gs[tc.c - 1];
      let act = lhs + rhs;
      assert_eq!(exp, &act);
    }
  }
}
