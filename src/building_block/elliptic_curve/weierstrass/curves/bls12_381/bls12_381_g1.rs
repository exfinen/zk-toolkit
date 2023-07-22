use crate::building_block::{
  field::{
    prime_field::PrimeField,
    prime_field_elem::PrimeFieldElem,
  },
  elliptic_curve::{
    curve::Curve,
    elliptic_curve_point_ops::EllipticCurvePointOps,
    weierstrass::{
      curves::bls12_381::{
        fq1::{Fq1, FIELD_ORDER as FQ1_FIELD_ORDER},
        g1_point::G1Point,
      },
      weierstrass_eq::WeierstrassEq, adder::affine_point_adder::AffinePointAdder,
    },
  },
  additive_identity::AdditiveIdentity,
};
use num_bigint::BigUint;
use once_cell::sync::Lazy;

#[derive(Clone)]
#[allow(non_camel_case_types)]
pub struct BLS12_381_G1 {
  pub f: PrimeField,    // base prime field
  pub f_r: PrimeField,  // field of group order r for convenience
  pub g: Option<G1Point>,  // generator point
  pub r: PrimeFieldElem,  // group order of the generator
  pub eq: WeierstrassEq<Fq1>,
}

impl BLS12_381_G1 {
  pub fn new() -> Self {
    let f = PrimeField::new(Lazy::get(&FQ1_FIELD_ORDER).unwrap());

    // base point
    let gx = f.elem(
      &BigUint::parse_bytes(b"17f1d3a73197d7942695638c4fa9ac0fc3688c4f9774b905a14e3a3f171bac586c55e83ff97a1aeffb3af00adb22c6bb", 16).unwrap(),
    );
    let gy = f.elem(
      &BigUint::parse_bytes(b"08b3f481e3aaa0f1a09e30ed741d8ae4fcf5e095d5d00af600db18cb2c04b3edd03cc744a2888ae40caa232946c5e7e1", 16).unwrap(),
    );

    // order of the base point
    let r = PrimeFieldElem::new(&f, &BigUint::parse_bytes(b"73eda753299d7d483339d80809a1d80553bda402fffe5bfeffffffff00000001", 16).unwrap());
    let f_r = PrimeField::new(&r);

    // G1 (F_q): y^2 = x^3 + 4
    let a1 = &f.elem(&0u8);
    let a2 = &f.elem(&0u8);
    let a3 = &f.elem(&0u8);
    let a4 = &f.elem(&0u8);
    let a6 = &f.elem(&4u8);
    let eq = Box::new(WeierstrassEq::new(&f, a1, a2, a3, a4, a6));

    let curve = BLS12_381_G1 {
      f,
      f_r,
      g: None,
      r,
      eq,
    };

    let g = G1Point { curve, x: gx, y: gy };
    curve.g = Some(g);

    curve
  }
}

impl EllipticCurvePointOps<G1Point, Fq1, PrimeField, BLS12_381_G1> for BLS12_381_G1 {
  type Adder = AffinePointAdder;
}

impl Curve<G1Point, Fq1, PrimeField> for BLS12_381_G1 {
  fn eq(&self) -> WeierstrassEq<Fq1> {
    self.eq.clone()
  }

  fn f(&self) -> PrimeField {
    self.f().clone()
  }

  fn f_n(&self) -> PrimeField {
    self.f_r.clone()
  }

  fn g(&self) -> G1Point {
    self.g.clone()
  }

  fn n(&self) -> G1Point {
    self.r.clone()
  }

  fn point_at_infinity(&self) -> G1Point {
    self.g.get_additive_identity()
  }
}

/*
use crate::building_block::bls12_381::{
  fq1::Fq1,
  fq2::Fq2,
  g1_point::G1Point,
  g2_point::G2Point,
};
use crate::building_block::field::Field;
use num_bigint::BigUint;
use once_cell::sync::Lazy;

pub static BASE_FIELD: Lazy<Field> = Lazy::new(|| {
  let order = BigUint::parse_bytes(b"1a0111ea397fe69a4b1ba7b6434bacd764774b84f38512bf6730d2a0f6b0f6241eabfffeb153ffffb9feffffffffaaab", 16).unwrap();
  Field::new(&order)
});

pub static G1_GENERATOR: Lazy<G1Point> = Lazy::new(|| {
  let x = Fq1::new(
    &BASE_FIELD,
    &BigUint::parse_bytes(b"17f1d3a73197d7942695638c4fa9ac0fc3688c4f9774b905a14e3a3f171bac586c55e83ff97a1aeffb3af00adb22c6bb", 16).unwrap(),
  );
  let y = Fq1::new(
    &BASE_FIELD,
    &BigUint::parse_bytes(b"08b3f481e3aaa0f1a09e30ed741d8ae4fcf5e095d5d00af600db18cb2c04b3edd03cc744a2888ae40caa232946c5e7e1", 16).unwrap(),
  );
  G1Point { x, y }
});

pub static G2_GENERATOR: Lazy<G2Point> = Lazy::new(|| {
  let x1 = BigUint::parse_bytes(b"13e02b6052719f607dacd3a088274f65596bd0d09920b61ab5da61bbdc7f5049334cf11213945d57e5ac7d055d042b7e", 16).unwrap();
  let x2 = BigUint::parse_bytes(b"024aa2b2f08f0a91260805272dc51051c6e47ad4fa403b02b4510b647ae3d1770bac0326a805bbefd48056c8c121bdb8", 16).unwrap();
  let x = Fq2::new(
    &Fq1::new(&BASE_FIELD, &x1),
    &Fq1::new(&BASE_FIELD, &x2),
  );
  let y1 = BigUint::parse_bytes(b"0606c4a02ea734cc32acd2b02bc28b99cb3e287e85a763af267492ab572e99ab3f370d275cec1da1aaa9075ff05f79be", 16).unwrap();
  let y2 = BigUint::parse_bytes(b"0ce5d527727d6e118cc9cdc6da2e351aadfd9baa8cbdd3a76d429a695160d12c923ac9cc3baca289e193548608b82801", 16).unwrap();
  let y = Fq2::new(
    &Fq1::new(&BASE_FIELD, &y1),
    &Fq1::new(&BASE_FIELD, &y2),
  );
  G2Point { x, y }
});

*/
#[cfg(test)]
mod tests {
  // use super::*;
  // use num_bigint::BigUint;
  // use crate::building_block::elliptic_curve::{
  //   elliptic_curve_point_ops::EllipticCurvePointOps,
  // };

  // #[test]
  // fn add_same_point() {
  //   let params = BLS12_381_G1Params::new();
  //   let ops = WeierstrassAffinePointOps::new(&params.f);

  //   let g2 = ops.add(&params.g, &params.g);
  //   let exp_x = BigUint::parse_bytes(b"89565891926547004231252920425935692360644145829622209833684329913297188986597", 10).unwrap();
  //   let exp_y = BigUint::parse_bytes(b"12158399299693830322967808612713398636155367887041628176798871954788371653930", 10).unwrap();
  //   assert_eq!(g2.x.n, exp_x);
  //   assert_eq!(g2.y.n, exp_y);
  // }

/*
  #[test]
  fn add_same_point_y_eq_0() {
    // TODO implement this. need to find the x-coord when y is zero
  }

  #[test]
  fn add_vertical_line() {
    let params = BLS12_381_G1Params::new();
    let g = &params.g;
    for ops in get_ops_list(&params.f) {
      let a = g.clone();
      let b = EcPoint::new(&a.x, &-&a.y);
      let exp = EcPoint::inf(&params.f);
      let act = ops.add(&a, &b);
      assert_eq!(act, exp);
    }
  }

  #[test]
  fn add_inf_and_affine() {
    let params = BLS12_381_G1Params::new();
    let g = &params.g;
    for ops in get_ops_list(&params.f) {
      let inf = EcPoint::inf(&params.f);
      let inf_plus_g = ops.add(g, &inf);
      assert_eq!(g, &inf_plus_g);
    }
  }

  #[test]
  fn add_affine_and_inf() {
    let params = BLS12_381_G1Params::new();
    let g = &params.g;
    for ops in get_ops_list(&params.f) {
      let inf = EcPoint::inf(&params.f);
      let g_plus_inf = ops.add(&inf, g);
      assert_eq!(g, &g_plus_inf);
    }
  }

  #[test]
  fn add_inf_and_inf() {
    let params = BLS12_381_G1Params::new();
    for ops in get_ops_list(&params.f) {
      let inf = EcPoint::inf(&params.f);
      let inf_plus_inf = ops.add(&inf, &inf);
      assert_eq!(inf_plus_inf, inf);
    }
  }

  struct Xy<'a> {
    _n: &'a str,
    x: &'a [u8; 64],
    y: &'a [u8; 64],
  }

  impl<'a> Xy<'a> {
    fn to_ec_point(&'a self, f: &Field) -> EcPoint {
      let gx = BigUint::parse_bytes(self.x, 16).unwrap();
      let gy = BigUint::parse_bytes(self.y, 16).unwrap();
      EcPoint::new(
        &FieldElem::new(f, &gx),
        &FieldElem::new(f, &gy),
      )
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

  fn get_g_multiples<'a, T>(curve: &Secp256k1<T, WeierstrassEq>) -> Vec<EcPoint>
    where T: ?Sized + EllipticCurveField + EllipticCurvePointAdd + ElllipticCurvePointInv {
    let ps = vec![
      Xy { _n: "1", x: b"79BE667EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B16F81798", y: b"483ADA7726A3C4655DA4FBFC0E1108A8FD17B448A68554199C47D08FFB10D4B8" },
      Xy { _n: "2", x: b"C6047F9441ED7D6D3045406E95C07CD85C778E4B8CEF3CA7ABAC09B95C709EE5", y: b"1AE168FEA63DC339A3C58419466CEAEEF7F632653266D0E1236431A950CFE52A" },
      Xy { _n: "3", x: b"F9308A019258C31049344F85F89D5229B531C845836F99B08601F113BCE036F9", y: b"388F7B0F632DE8140FE337E62A37F3566500A99934C2231B6CB9FD7584B8E672" },
      Xy { _n: "4", x: b"E493DBF1C10D80F3581E4904930B1404CC6C13900EE0758474FA94ABE8C4CD13", y: b"51ED993EA0D455B75642E2098EA51448D967AE33BFBDFE40CFE97BDC47739922" },
      Xy { _n: "5", x: b"2F8BDE4D1A07209355B4A7250A5C5128E88B84BDDC619AB7CBA8D569B240EFE4", y: b"D8AC222636E5E3D6D4DBA9DDA6C9C426F788271BAB0D6840DCA87D3AA6AC62D6" },
      Xy { _n: "6", x: b"FFF97BD5755EEEA420453A14355235D382F6472F8568A18B2F057A1460297556", y: b"AE12777AACFBB620F3BE96017F45C560DE80F0F6518FE4A03C870C36B075F297" },
      Xy { _n: "7", x: b"5CBDF0646E5DB4EAA398F365F2EA7A0E3D419B7E0330E39CE92BDDEDCAC4F9BC", y: b"6AEBCA40BA255960A3178D6D861A54DBA813D0B813FDE7B5A5082628087264DA" },
      Xy { _n: "8", x: b"2F01E5E15CCA351DAFF3843FB70F3C2F0A1BDD05E5AF888A67784EF3E10A2A01", y: b"5C4DA8A741539949293D082A132D13B4C2E213D6BA5B7617B5DA2CB76CBDE904" },
      Xy { _n: "9", x: b"ACD484E2F0C7F65309AD178A9F559ABDE09796974C57E714C35F110DFC27CCBE", y: b"CC338921B0A7D9FD64380971763B61E9ADD888A4375F8E0F05CC262AC64F9C37" },
      Xy { _n: "10", x: b"A0434D9E47F3C86235477C7B1AE6AE5D3442D49B1943C2B752A68E2A47E247C7", y: b"893ABA425419BC27A3B6C7E693A24C696F794C2ED877A1593CBEE53B037368D7" },
    ];
    let g = &curve.params.g;
    let mut gs = vec![g.clone()];  // gs[0] is used to match index and g's n and will not be actually used
    for p in ps {
      gs.push(p.to_ec_point(&curve.params.f));
    }
    gs
  }

  #[test]
  fn scalar_mul_smaller_nums() {
    let params = Secp256k1Params::new();
    let g = &params.g;
    for ops in get_ops_list(&params.f) {
      let curve = Secp256k1::new(ops.clone(), params.clone());
      let gs = get_g_multiples(&curve);

      for n in 1usize..=10 {
        let res = ops.scalar_mul(g, &BigUint::from(n));
        assert_eq!(&res, &gs[n]);
      }
    }
  }

  struct ScalarMulTest<'a> {
    k: &'a [u8; 64],
    x: &'a [u8; 64],
    y: &'a [u8; 64],
  }

  #[test]
  fn scalar_mul_gen_pubkey() {
    let test_cases = vec![
      ScalarMulTest {
        k: b"AA5E28D6A97A2479A65527F7290311A3624D4CC0FA1578598EE3C2613BF99522",
        x: b"34F9460F0E4F08393D192B3C5133A6BA099AA0AD9FD54EBCCFACDFA239FF49C6",
        y: b"0B71EA9BD730FD8923F6D25A7A91E7DD7728A960686CB5A901BB419E0F2CA232",
      },
      ScalarMulTest {
        k: b"7E2B897B8CEBC6361663AD410835639826D590F393D90A9538881735256DFAE3",
        x: b"D74BF844B0862475103D96A611CF2D898447E288D34B360BC885CB8CE7C00575",
        y: b"131C670D414C4546B88AC3FF664611B1C38CEB1C21D76369D7A7A0969D61D97D",
      },
      ScalarMulTest {
        k: b"6461E6DF0FE7DFD05329F41BF771B86578143D4DD1F7866FB4CA7E97C5FA945D",
        x: b"E8AECC370AEDD953483719A116711963CE201AC3EB21D3F3257BB48668C6A72F",
        y: b"C25CAF2F0EBA1DDB2F0F3F47866299EF907867B7D27E95B3873BF98397B24EE1",
      },
      ScalarMulTest {
        k: b"376A3A2CDCD12581EFFF13EE4AD44C4044B8A0524C42422A7E1E181E4DEECCEC",
        x: b"14890E61FCD4B0BD92E5B36C81372CA6FED471EF3AA60A3E415EE4FE987DABA1",
        y: b"297B858D9F752AB42D3BCA67EE0EB6DCD1C2B7B0DBE23397E66ADC272263F982",
      },
      ScalarMulTest {
        k: b"1B22644A7BE026548810C378D0B2994EEFA6D2B9881803CB02CEFF865287D1B9",
        x: b"F73C65EAD01C5126F28F442D087689BFA08E12763E0CEC1D35B01751FD735ED3",
        y: b"F449A8376906482A84ED01479BD18882B919C140D638307F0C0934BA12590BDE",
      },
    ];

    use std::time::Instant;
    let params = Secp256k1Params::new();
    let g = &params.g;
    for ops in get_ops_list(&params.f) {
      let curve = Secp256k1::new(ops.clone(), params.clone());
      for t in &test_cases {
        let k = BigUint::parse_bytes(t.k, 16).unwrap();
        let x = BigUint::parse_bytes(t.x, 16).unwrap();
        let y = BigUint::parse_bytes(t.y, 16).unwrap();
        let p = EcPoint::new(
          &FieldElem::new(&curve.params.f, &x),
          &FieldElem::new(&curve.params.f, &y),
        );

        let beg = Instant::now();
        let gk = ops.scalar_mul(g, &k);
        let end = beg.elapsed();
        println!("Large number scalar mul done in {}.{:03} sec", end.as_secs(), end.subsec_nanos() / 1_000_000);
        assert_eq!(p, gk);
      }
    }
  }

  #[test]
  fn add_different_points() {
    let large_1 = Xy {
      _n: "28948022309329048855892746252171976963209391069768726095651290785379540373584",
      x: b"A6B594B38FB3E77C6EDF78161FADE2041F4E09FD8497DB776E546C41567FEB3C",
      y: b"71444009192228730CD8237A490FEBA2AFE3D27D7CC1136BC97E439D13330D55",
    };
    let large_2 = Xy {
      _n: "57896044618658097711785492504343953926418782139537452191302581570759080747168",
      x: b"00000000000000000000003B78CE563F89A0ED9414F5AA28AD0D96D6795F9C63",
      y: b"3F3979BF72AE8202983DC989AEC7F2FF2ED91BDD69CE02FC0700CA100E59DDF3",
    };
    let large_3 = Xy {
      _n: "86844066927987146567678238756515930889628173209306178286953872356138621120752",
      x: b"E24CE4BEEE294AA6350FAA67512B99D388693AE4E7F53D19882A6EA169FC1CE1",
      y: b"8B71E83545FC2B5872589F99D948C03108D36797C4DE363EBD3FF6A9E1A95B10",
    };

    let params = Secp256k1Params::new();
    let f = &params.f;
    for ops in get_ops_list(f) {
      let curve = Secp256k1::new(ops.clone(), params.clone());
      let gs = get_g_multiples(&curve);

      let test_cases = [
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
        let res = ops.add(&gs[tc.a], &gs[tc.b]);
        assert_eq!(&res, &gs[tc.c]);
      }

      let f = &curve.params.f;
      let l1 = large_1.to_ec_point(f);
      let l2 = large_2.to_ec_point(f);
      let l3 = large_3.to_ec_point(f);

      let l1_plus_l2 = ops.add(&l1, &l2);
      assert_eq!(l1_plus_l2, l3);
    }
  }
*/
}
