use crate::{
  impl_affine_add,
  impl_scalar_mul_point,
  building_block::{
    field::prime_field::PrimeField,
    curves::{
      bls12_381::{
        fq1::Fq1,
        params::Params as P,
        private_key::PrivateKey,
      },
      rational_point::RationalPoint,
      weierstrass_eq::WeierstrassEq,
    },
    zero::Zero,
  },
};
use num_bigint::{
  BigUint,
  RandBigInt,
};
use num_traits::Zero as NumTraitsZero;
use std::{
  fmt,
  ops::{Add, Mul, Neg, AddAssign},
  sync::Arc,
};
use once_cell::sync::Lazy;
use rand::SeedableRng;
use rand_chacha::ChaCha12Rng;

#[derive(Clone)]
pub enum G1Point {
  Rational { x: Fq1, y: Fq1 },
  AtInfinity,
}

static GENERATOR: Lazy<AffinePoint> = Lazy::new(|| {
  let f = P::base_prime_field();
  let x = f.elem(
    &BigUint::parse_bytes(b"17f1d3a73197d7942695638c4fa9ac0fc3688c4f9774b905a14e3a3f171bac586c55e83ff97a1aeffb3af00adb22c6bb", 16).unwrap(),
  );
  let y = f.elem(
    &BigUint::parse_bytes(b"08b3f481e3aaa0f1a09e30ed741d8ae4fcf5e095d5d00af600db18cb2c04b3edd03cc744a2888ae40caa232946c5e7e1", 16).unwrap(),
  );
  G1Point::Rational { x, y, }
});

impl G1Point {
  pub fn new(x: &Fq1, y: &Fq1) -> Self {
    G1Point::Rational {
      x: x.clone(),
      y: y.clone(),
    }
  }

  pub fn g() -> AffinePoint {
    GENERATOR.clone()
  }

  pub fn inv(&self) -> Self {
    match self {
      G1Point::AtInfinity => panic!("No inverse exists for point at infinitty"),
      G1Point::Rational { x, y } => G1Point::new(&x, &y.inv()),
    }
  }

  fn fmt_hex(s: &str) -> String {
    let mut ns = s.to_uppercase();
    if s.len() < 96 {
      ns = "0".repeat(96 - ns.len()) + &ns;
    }
    format!("{} {} {} {} {} {}",
      &ns[0..16],
      &ns[16..32],
      &ns[32..48],
      &ns[48..64],
      &ns[64..80],
      &ns[80..96],
    )
  }

  pub fn get_random_point() -> AffinePoint {
    let mut rng = ChaCha12Rng::from_entropy();
    let subgroup = &P::subgroup();
    let n = rng.gen_biguint_range(&NumTraitsZero::zero(), subgroup.order_ref());
    G1Point::g() * &subgroup.elem(&n)
  }

  // for impl_scalar_mul_point macro
  pub fn curve_group() -> Arc<PrimeField> {
    P::subgroup()
  }
}

impl RationalPoint for G1Point {
  fn is_rational_point(&self) -> bool {
    match self {
      G1Point::AtInfinity => false,
      G1Point::Rational { x, y } => {
        let f = P::base_prime_field();
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

impl fmt::Debug for G1Point {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      G1Point::AtInfinity => {
        write!(f, "Point at infinity")

      },
      G1Point::Rational { x, y } => {
        write!(f, "{}\n{}",
          G1Point::fmt_hex(&x.e.to_str_radix(16)),
          G1Point::fmt_hex(&y.e.to_str_radix(16)),
        )
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

impl<'a> Mul<&'a PrivateKey> for &G1Point {
  type Output = G1Point;

  fn mul(self, rhs: &PrivateKey) -> Self::Output {
    let rhs = P::subgroup().elem(&rhs.value);
    self * rhs
  }
}

type AffinePoint = G1Point;
impl_affine_add!(G1Point);
impl_scalar_mul_point!(Fq1, G1Point);

impl AddAssign<G1Point> for G1Point {
  fn add_assign(&mut self, rhs: G1Point) {
    *self = &*self + rhs
  }
}

impl PartialEq for G1Point {
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (G1Point::AtInfinity, G1Point::AtInfinity) => true,
      (G1Point::AtInfinity, G1Point::Rational { x: _x, y: _y }) => false,
      (G1Point::Rational { x: _x, y: _y }, G1Point::AtInfinity) => false,
      (G1Point::Rational { x: x1, y: y1 }, G1Point::Rational { x: x2, y: y2 }) => {
        x1 == x2 && y1 == y2
      },
    }
  }
}

impl Eq for AffinePoint {}

macro_rules! impl_neg {
  ($target: ty) => {
    impl Neg for $target {
      type Output = G1Point;

      fn neg(self) -> Self::Output {
        match self {
          G1Point::AtInfinity => G1Point::AtInfinity,
          G1Point::Rational { x, y } => {
            G1Point::new(&x, &y.neg())
          }
        }
      }
    }
  }
}
impl_neg!(G1Point);
impl_neg!(&G1Point);

#[cfg(test)]
mod tests {
  use super::*;
  use num_bigint::BigUint;
  use std::rc::Rc;

  #[test]
  fn scalar_mul() {
    let g = &G1Point::g();
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
    let g = &G1Point::g();
    let g2 = g + g;
    let exp_x = BigUint::parse_bytes(b"838589206289216005799424730305866328161735431124665289961769162861615689790485775997575391185127590486775437397838", 10).unwrap();
    let exp_y = BigUint::parse_bytes(b"3450209970729243429733164009999191867485184320918914219895632678707687208996709678363578245114137957452475385814312", 10).unwrap();

    match g2 {
      G1Point::AtInfinity => panic!("expected rational point, but got point at infinity"),
      G1Point::Rational { x, y } => {
        assert_eq!(x.e, exp_x);
        assert_eq!(y.e, exp_y);
      },
    }
  }

  #[test]
  fn negate() {
    let g = &G1Point::g();
    let res = g + -g;

    match res {
      G1Point::AtInfinity => {},
      _ => panic!("expected point at infinity, but got rational point"),
    }
  }

  #[test]
  fn add_vertical_line() {
    let g = &G1Point::g();
    match g {
      G1Point::AtInfinity => panic!("expected rational point, but got point at infinity"),
      G1Point::Rational { x, y } => {
        let a = G1Point::new(&x, &y);
        let b = G1Point::new(&x, &-y);
        let exp = G1Point::AtInfinity;
        let act = &a + &b;
        assert_eq!(act, exp);
      },
    }
  }

  #[test]
  fn add_inf_and_affine() {
    let g = &G1Point::g();
    match g {
      G1Point::AtInfinity => panic!("expected rational point, but got point at infinity"),
      g => {
        let inf_plus_g = g + G1Point::AtInfinity;
        assert_eq!(g, &inf_plus_g);
      },
    }
  }

  #[test]
  fn add_affine_and_inf() {
    let g = &G1Point::g();
    match g {
      G1Point::AtInfinity => panic!("expected rational point, but got point at infinity"),
      g => {
        let g_plus_inf = G1Point::AtInfinity + g;
        assert_eq!(g, &g_plus_inf);
      },
    }
  }

  #[test]
  fn add_inf_and_inf() {
    let res = G1Point::AtInfinity + G1Point::AtInfinity;
    match res {
      G1Point::AtInfinity => (),
      _ => panic!("rational point not expected"),
    }
  }

  struct Xy<'a> {
    x: &'a [u8],
    y: &'a [u8],
  }

  impl<'a> Into<G1Point> for &Xy<'a> {
    fn into(self) -> G1Point {
      let f = P::base_prime_field();
      let gx = BigUint::parse_bytes(self.x, 10).unwrap();
      let gy = BigUint::parse_bytes(self.y, 10).unwrap();
      G1Point::new(
        &Fq1::new(&Rc::new(f.clone()), &gx),
        &Fq1::new(&Rc::new(f.clone()), &gy),
      )
    }
  }

  fn get_g_multiples<'a>() -> Vec<G1Point> {
    let ps = vec![
      Xy { x: b"3685416753713387016781088315183077757961620795782546409894578378688607592378376318836054947676345821548104185464507", y: b"1339506544944476473020471379941921221584933875938349620426543736416511423956333506472724655353366534992391756441569" },
      Xy { x: b"838589206289216005799424730305866328161735431124665289961769162861615689790485775997575391185127590486775437397838", y: b"3450209970729243429733164009999191867485184320918914219895632678707687208996709678363578245114137957452475385814312" },
      Xy { x: b"1527649530533633684281386512094328299672026648504329745640827351945739272160755686119065091946435084697047221031460", y: b"487897572011753812113448064805964756454529228648704488481988876974355015977479905373670519228592356747638779818193" },
      Xy { x: b"1940386630589042756063486004190165295069185078823041184114464346315715796897349637805088429212932248421554491972448", y: b"3114296198575988357603748217414299058449850389627766458803888308926785677746356789015548793127623382888106859156543" },
      Xy { x: b"2601793266141653880357945339922727723793268013331457916525213050197274797722760296318099993752923714935161798464476", y: b"3498096627312022583321348410616510759186251088555060790999813363211667535344132702692445545590448314959259020805858" },
      Xy { x: b"1063080548659463434646774310890803636667161539235054707411467714858983518890075240133758563865893724012200489498889", y: b"3669927104170827068533340245967707139563249539898402807511810342954528074138727808893798913182606104785795124774780" },
      Xy { x: b"3872473689207892378470335395114902631176541028916158626161662840934315241539439160301564344905260612642783644023991", y: b"2547806390474846378491145127515427451279430889101277169890334737406180277792171092197824251632631671609860505999900" },
      Xy { x: b"1285966557826138051526586543856222272686311322783612095733044979083626020901131273426798483907031822448028171106767", y: b"3987260880502909363629828380003774205546872765117326777338153310890683319206585652257079113564419659352890089363873" },
      Xy { x: b"3971675556538908004130084773503021351583407620890695272226385332452194486153316625183061567093226342405194446632851", y: b"1120750640227410374130508113691552487207139112596221955734902008063040284119210871734388578113045163251615428544022" },
      Xy { x: b"2386781901035473772144341182407687860118005925033428055218509614629770831545237878364312588177396809142590665502445", y: b"2721985711015193199868848835229056819857651383925471979786755635273858421658233285328399263507021600622741844499993" },
    ];
    let mut gs: Vec<G1Point> = vec![];
    for p in &ps {
      gs.push(p.into());
    }
    gs
  }

  #[test]
  fn scalar_mul_smaller_nums() {
    let f = P::subgroup();
    let g = &G1Point::g();
    let gs = get_g_multiples();

    for n in 1usize..=10 {
      let res = g * f.elem(&n);
      assert_eq!(&res, &gs[n - 1]);
    }
  }

  struct ScalarMulTest<'a> {
    p: &'a Xy<'a>,
    multiple: &'a [u8],
  }

  #[test]
  fn scalar_mul_gen_pubkey() {
    let test_cases = vec![
      ScalarMulTest { p: &Xy { x: b"798884334436478365854495527626954795652599920080455843008127451971226059259762238341774216896615589251979273848351", y: b"34212239898044597553446744222440174147395448914054998171267839489169096766468496771229540528596150009462706183351" }, multiple: b"12345" },
      ScalarMulTest { p: &Xy { x: b"2692774828344496329302857221881558456687930215547239128764741076961478364983749016898548084741404528993590529237160", y: b"2854663322886649950013803173936538691128363050644696522278545158974587049233023478599735149127378986005097616945785" }, multiple: b"1234567" },
      ScalarMulTest { p: &Xy { x: b"578595450910910825442973115404547363724456959235221058156522221418740269234033541648608473036490657833890475117414", y: b"918683632106062236182220132688464791275434112564658306544011664985161543677680961900104005427653283147211155443604" }, multiple: b"1234567890123456789" },
      ScalarMulTest { p: &Xy { x: b"233428720546585353387591766649253720217324789024335982603773308838219411095446257324273777614133447118484569237158", y: b"739589369061541979943645640116032571136884145267397816113677291549794526953649687502393808128626557292504917003358" }, multiple: b"123456789012345678901234567890" },
    ];

    let f = P::base_prime_field();
    let g = &G1Point::g();

    for t in &test_cases {
      let p: G1Point = t.p.into();
      let multiple = BigUint::parse_bytes(t.multiple, 10).unwrap();

      let gk = g * f.elem(&multiple);
      assert_eq!(p, gk);
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
