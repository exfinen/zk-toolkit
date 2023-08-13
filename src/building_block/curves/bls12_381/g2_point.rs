use crate::{
  impl_affine_add,
  impl_scalar_mul_point,
  building_block::{
    field::prime_field::PrimeField,
    curves::{
      bls12_381::{
        fq1::Fq1,
        fq2::Fq2,
      },
      rational_point::RationalPoint,
    },
    zero::Zero,
  },
};
use num_bigint::BigUint;
use std::{
  ops::{Add, Mul},
  sync::Arc,
};
use once_cell::sync::Lazy;

#[derive(Clone, Debug)]
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

  pub fn g() -> Self {
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

type AffinePoint = G2Point;
impl_affine_add!(G2Point);
impl_scalar_mul_point!(Fq1, G2Point);

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
  fn scalar_mul() {
    let g = &G2Point::g();
    let f = G2Point::curve_group();
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
    let exp_x_u1 = BigUint::parse_bytes(b"838589206289216005799424730305866328161735431124665289961769162861615689790485775997575391185127590486775437397838", 10).unwrap();
    let exp_x_u0 = BigUint::parse_bytes(b"838589206289216005799424730305866328161735431124665289961769162861615689790485775997575391185127590486775437397838", 10).unwrap();
    let exp_y_u1 = BigUint::parse_bytes(b"3450209970729243429733164009999191867485184320918914219895632678707687208996709678363578245114137957452475385814312", 10).unwrap();
    let exp_y_u0 = BigUint::parse_bytes(b"3450209970729243429733164009999191867485184320918914219895632678707687208996709678363578245114137957452475385814312", 10).unwrap();

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
  fn add_same_point_y_eq_0() {
    // TODO implement this. need to find the x-coord when y is zero
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

  struct Xy<'a> {
    x: &'a [u8],
    y: &'a [u8],
  }

  // impl<'a> Into<G2Point> for Xy<'a> {
  //   fn into(self) -> G2Point {
  //     let f = G2Point::base_field();
  //     let gx = BigUint::parse_bytes(self.x, 10).unwrap();
  //     let gy = BigUint::parse_bytes(self.y, 10).unwrap();
  //     G2Point::new(
  //       &Fq1::new(&Rc::new(f.clone()), &gx),
  //       &Fq1::new(&Rc::new(f.clone()), &gy),
  //     )
  //   }
  // }

  // fn get_g_multiples<'a>() -> Vec<G2Point> {
  //   let ps = vec![
  //     Xy { x: b"3685416753713387016781088315183077757961620795782546409894578378688607592378376318836054947676345821548104185464507", y: b"1339506544944476473020471379941921221584933875938349620426543736416511423956333506472724655353366534992391756441569" },
  //     Xy { x: b"838589206289216005799424730305866328161735431124665289961769162861615689790485775997575391185127590486775437397838", y: b"3450209970729243429733164009999191867485184320918914219895632678707687208996709678363578245114137957452475385814312" },
  //     Xy { x: b"1527649530533633684281386512094328299672026648504329745640827351945739272160755686119065091946435084697047221031460", y: b"487897572011753812113448064805964756454529228648704488481988876974355015977479905373670519228592356747638779818193" },
  //     Xy { x: b"1940386630589042756063486004190165295069185078823041184114464346315715796897349637805088429212932248421554491972448", y: b"3114296198575988357603748217414299058449850389627766458803888308926785677746356789015548793127623382888106859156543" },
  //     Xy { x: b"2601793266141653880357945339922727723793268013331457916525213050197274797722760296318099993752923714935161798464476", y: b"3498096627312022583321348410616510759186251088555060790999813363211667535344132702692445545590448314959259020805858" },
  //     Xy { x: b"1063080548659463434646774310890803636667161539235054707411467714858983518890075240133758563865893724012200489498889", y: b"3669927104170827068533340245967707139563249539898402807511810342954528074138727808893798913182606104785795124774780" },
  //     Xy { x: b"3872473689207892378470335395114902631176541028916158626161662840934315241539439160301564344905260612642783644023991", y: b"2547806390474846378491145127515427451279430889101277169890334737406180277792171092197824251632631671609860505999900" },
  //     Xy { x: b"1285966557826138051526586543856222272686311322783612095733044979083626020901131273426798483907031822448028171106767", y: b"3987260880502909363629828380003774205546872765117326777338153310890683319206585652257079113564419659352890089363873" },
  //     Xy { x: b"3971675556538908004130084773503021351583407620890695272226385332452194486153316625183061567093226342405194446632851", y: b"1120750640227410374130508113691552487207139112596221955734902008063040284119210871734388578113045163251615428544022" },
  //     Xy { x: b"2386781901035473772144341182407687860118005925033428055218509614629770831545237878364312588177396809142590665502445", y: b"2721985711015193199868848835229056819857651383925471979786755635273858421658233285328399263507021600622741844499993" },
  //   ];
  //   let mut gs: Vec<G2Point> = vec![];
  //   for p in ps {
  //     gs.push(p.into());
  //   }
  //   gs
  // }

  #[test]
  fn scalar_mul_smaller_nums() {
    let f = G2Point::curve_group();
    let g = &G2Point::g();
    // let gs = get_g_multiples();

    // for n in 1usize..=10 {
    //   let res = g * f.elem(&n);
    //   assert_eq!(&res, &gs[n - 1]);
    // }
  }

  struct ScalarMulTest<'a> {
    x: &'a [u8],
    y: &'a [u8],
    multiple: &'a [u8],
  }

  #[test]
  fn scalar_mul_gen_pubkey() {
    let test_cases = vec![
      ScalarMulTest { x: b"798884334436478365854495527626954795652599920080455843008127451971226059259762238341774216896615589251979273848351", y: b"34212239898044597553446744222440174147395448914054998171267839489169096766468496771229540528596150009462706183351", multiple: b"12345" },
      ScalarMulTest { x: b"2692774828344496329302857221881558456687930215547239128764741076961478364983749016898548084741404528993590529237160", y: b"2854663322886649950013803173936538691128363050644696522278545158974587049233023478599735149127378986005097616945785", multiple: b"1234567" },
      ScalarMulTest { x: b"578595450910910825442973115404547363724456959235221058156522221418740269234033541648608473036490657833890475117414", y: b"918683632106062236182220132688464791275434112564658306544011664985161543677680961900104005427653283147211155443604", multiple: b"1234567890123456789" },
      ScalarMulTest { x: b"233428720546585353387591766649253720217324789024335982603773308838219411095446257324273777614133447118484569237158", y: b"739589369061541979943645640116032571136884145267397816113677291549794526953649687502393808128626557292504917003358", multiple: b"123456789012345678901234567890" },
    ];

    let f = G2Point::g();
    let g = &G2Point::g();

    for t in &test_cases {
      let x = BigUint::parse_bytes(t.x, 10).unwrap();
      let y = BigUint::parse_bytes(t.y, 10).unwrap();

      // let p = G2Point::new(
      //   &Fq1::new(&Rc::new(f.clone()), &x),
      //   &Fq1::new(&Rc::new(f.clone()), &y),
      // );
      // let multiple = BigUint::parse_bytes(t.multiple, 10).unwrap();

      // let gk = g * f.elem(&multiple);
      // assert_eq!(p, gk);
    }
  }

  #[test]
  fn add_different_points() {
    // let gs = get_g_multiples();

    // let test_cases = [
    //   AddTestCase::new(1, 1, 2),
    //   AddTestCase::new(1, 2, 3),
    //   AddTestCase::new(2, 2, 4),
    //   AddTestCase::new(2, 6, 8),
    //   AddTestCase::new(3, 4, 7),
    //   AddTestCase::new(5, 1, 6),
    //   AddTestCase::new(5, 2, 7),
    //   AddTestCase::new(8, 1, 9),
    //   AddTestCase::new(9, 1, 10),
    // ];

    // for tc in test_cases {
    //   let lhs = &gs[tc.a - 1];
    //   let rhs = &gs[tc.b - 1];
    //   let exp = &gs[tc.c - 1];
    //   let act = lhs + rhs;
    //   assert_eq!(exp, &act);
    // }
  }
}
