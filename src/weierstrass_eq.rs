use crate::field_elem::FieldElem;
use crate::field::Field;
use crate::ec_point::{EcPoint, Coord2};
use crate::curve::Curve;
use num_bigint::BigUint;
use num_traits::identities::{Zero, One};
use std::ops::{BitAnd, ShrAssign};
use std::rc::Rc;

// represents: y^2 = x^3 + Ax + B
pub struct WeierstrassEq {
  pub f: Rc<Field>,
  pub a: FieldElem,
  pub b: FieldElem,
  pub g: EcPoint,
  pub n: BigUint,
  pub zero: BigUint,
  pub one: BigUint,
}

impl WeierstrassEq {
  pub fn new(
    f: Rc<Field>, 
    a: BigUint, 
    b: BigUint, 
    gx: BigUint, 
    gy: BigUint,
    n: BigUint,
  ) -> Result<Self, String> {
    let a = FieldElem::new(f.clone(), a);
    let b = FieldElem::new(f.clone(), b);
    let g = EcPoint::Affine(Coord2::new(
      FieldElem::new(f.clone(), gx), 
      FieldElem::new(f.clone(), gy),
    ).unwrap());
    let zero = BigUint::zero();
    let one = BigUint::one();

    Ok(WeierstrassEq { f, a, b, g, n, zero, one })
  }

  pub fn secp256k1() -> WeierstrassEq {
    let p = BigUint::parse_bytes(b"FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F", 16).unwrap();
    let f = Field::new(p);

    let a = BigUint::from(0u32);
    let b = BigUint::from(7u32);

    // base point
    let gx = BigUint::parse_bytes(b"79BE667EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B16F81798", 16).unwrap();
    let gy = BigUint::parse_bytes(b"483ADA7726A3C4655DA4FBFC0E1108A8FD17B448A68554199C47D08FFB10D4B8", 16).unwrap();

    // order of base point
    let n = BigUint::parse_bytes(b"FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141", 16).unwrap();

    // curve
    WeierstrassEq::new(f, a, b, gx, gy, n).unwrap()
  }
}

impl Curve for WeierstrassEq {
  fn g(&self) -> EcPoint {
    self.g.clone()
  }
  
  fn n(&self) -> BigUint {
    self.n.clone()
  }

  fn scalar_mul(&self, multiplier: &BigUint) -> EcPoint {
    let mut n = multiplier.clone();
    let mut res = EcPoint::Infinity();
    let mut g_pow_n = self.g.clone();
    let one = BigUint::one();

    while !n.is_zero() {
      if n.clone().bitand(&one).is_one() {
        res = self.add(&res, &g_pow_n);
      }
      g_pow_n = self.add(&g_pow_n, &g_pow_n);
      n.shr_assign(1usize);
    }
    res
  }

  fn add(&self, p1: &EcPoint, p2: &EcPoint) -> EcPoint {
    match (p1, p2) {
      // when adding point at infinity to a point
      (EcPoint::Infinity(), EcPoint::Affine(p)) => {
        EcPoint::Affine(p.clone())
      },
      (EcPoint::Affine(p), EcPoint::Infinity()) => {
        EcPoint::Affine(p.clone())
      },
      (EcPoint::Infinity(), EcPoint::Infinity()) => {
        EcPoint::Infinity()
      },
      // when line through p1 and p2 is vertical line
      (EcPoint::Affine(p1), EcPoint::Affine(p2)) if p1.x == p2.x && p1.y != p2.y => {
        EcPoint::Infinity()
      },
      // when p1 and p2 are the same point
      (EcPoint::Affine(p1), EcPoint::Affine(p2)) if p1.x == p2.x && p1.y == p2.y => {
        // special case: if y == 0, the tangent line is vertical
        if p1.y.v == BigUint::zero() || p2.y.v == BigUint::zero() {
          return EcPoint::Infinity();
        }
        // differentiate y^2 = x^3 + Ax + B w/ implicit differentiation
        // d/dx(y^2) = d/dx(x^3 + Ax + B)
        // 2y dy/dx = 3x^2 + A
        // dy/dx = (3x^2 + A) / 2y
        //
        // dy/dx is the slope m of the tangent line at the point 
        // m = (3x^2 + A) / 2y
        let m1 = p1.x.sq().mul_u32(3u32);
        let m2 = p1.y.mul_u32(2u32);
        let m = m1.div(&m2).unwrap();

        // equation of intersecting line is
        // y = m(x − p1.x) + p1.y (1)
        //
        // substitute y with (1):
        // (m(x − p1.x) + p1.y)^2 = x^3 + Ax + B
        //
        // moving LHS to RHS, we get:
        // 0 = x^3 - m^2 x^2 + ...  (2)
        //
        // with below equation:
        // (x - r)(x - s)(x - t) = x^3 + (r + s + t)x^2 + (ab + ac + bc)x − abc 
        // 
        // we know that the coefficient of x^2 term is:
        // r + s + t 
        //
        // using (2), the coefficient of x^2 term of the intersecting line is:
        // m^2 = r + s + t
        // 
        // since p1 and p2 are the same point, replace r and s w/ p1.x
        // to get the x-coordinate of the point where (1) intersects the curve
        // x3 = m^2 − 2*p1.x
        let p3x = m.sq().sub(&p1.x.mul_u32(2u32));

        // then get the y-coordinate by substituting x in (1) w/ x3 to get y3
        // y3 = m(x3 − p1.x) + p1.y 
        // 
        // reflecting y3 across the x-axis results in the addition result y-coordinate 
        // result.y = -1 * y3 = m(p1.x - x3) - p1.y
        let p3y_neg = m.mul(&p1.x.sub(&p3x)).sub(&p1.y);
        
        EcPoint::Affine(Coord2 {
          x: p3x,
          y: p3y_neg,
        })
      },
      // when line through p1 and p2 is non-vertical line
      (EcPoint::Affine(p1), EcPoint::Affine(p2)) => {

        // slope m of the line that intersects the curve at p1 and p2:
        // p2.y - p1.y = m(p2.x - p1.x)
        // m(p2.x - p1.x) = p2.y - p1.y
        // m = (p2.y - p1.y) / (p2.x - p1.x)
        let m = (p2.y.sub(&p1.y)).div(&p2.x.sub(&p1.x)).unwrap();

        // then the equation of the line is:
        // y = m(x − p1.x) + p1.y  (1)
        //
        // starting from a curve equation of Weierstrass form:
        // y^2 = x^3 + Ax + B
        //
        // substitute y with (1):
        // (m(x − p1.x) + p1.y)^2 = x^3 + Ax + B
        //
        // moving LHS to RHS, we get:
        // 0 = x^3 - m^2 x^2 + ...  (2)
        //
        // with below equation:
        // (x - r)(x - s)(x - t) = x^3 + (r + s + t)x^2 + (ab + ac + bc)x − abc 
        // 
        // we know that the coefficient of x^2 term is:
        // r + s + t 
        //
        // using (2), the coefficient of x^2 term of the intersecting line is:
        // m^2 = r + s + t
        // 
        // substitute r and s with the known 2 roots - p1.x and p2.x:
        // m^2 = p1.x + p2. + t
        // t = m^2 - p1.x - p2.x
        //
        // here t is the x coordinate of the p3 we're trying to find:
        // p3.x = m^2 - p1.x - p2.x
        let p3x = m.sq().sub(&p1.x).sub(&p2.x);

        // using (1), find the y-coordinate of the 3rd intersecting point and p3x obtained above
        // y = m(x − p1.x) + p1.y
        // p3.y = m(p3.x − p1.x) + p1.y
        let p3y = m.mul(&p3x.sub(&p1.x)).add(&p1.y);
        
        // then (p3.x, -p3.y) is the result of adding p1 and p2
        let p3y_neg = p3y.neg();
        
        EcPoint::Affine(Coord2 {
          x: p3x,
          y: p3y_neg,
        })
      },
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use num_bigint::BigUint;

  #[test]
  fn test_scalar_mul_same_point() {
    let e = WeierstrassEq::secp256k1();
    let g2 = e.add(&e.g, &e.g);
    let exp_x = BigUint::parse_bytes(b"89565891926547004231252920425935692360644145829622209833684329913297188986597", 10).unwrap();
    let exp_y = BigUint::parse_bytes(b"12158399299693830322967808612713398636155367887041628176798871954788371653930", 10).unwrap();
    match g2 {
      EcPoint::Affine(c) => {
        assert_eq!(c.x.v, exp_x);
        assert_eq!(c.y.v, exp_y);
      },
      _ => {
        panic!("Expected affine point");
      }
    }
  }

  #[test]
  fn test_scalar_mul_y_eq_0() {
    // TODO implement this. need to find the x-coord when y is zero
  }

  #[test]
  fn test_scalar_mul_vertical_line() {
    let e = WeierstrassEq::secp256k1();
    let a = e.g.clone();
    match e.g.clone() {
      EcPoint::Affine(c) => {
      let b = EcPoint::Affine(Coord2::new(c.x, c.y.neg()).unwrap());
        match e.add(&a, &b) {
          EcPoint::Affine(_) => {
            panic!("Expected point at infinity");
          },
          EcPoint::Infinity() => {},
        }
      },
      _ => {}
    }
  }

  #[test]
  fn test_scalar_mul_inf_affine() {
    let e = WeierstrassEq::secp256k1();
    let inf = EcPoint::Infinity();
    let inf_plus_g = e.add(&e.g, &inf);
    match (e.g, inf_plus_g) {
      (EcPoint::Affine(c1), EcPoint::Affine(c2)) => {
        assert_eq!(c1, c2);
      },
      _ => {
        panic!("Expected inf+g to be g");
      }
    }
  }

  #[test]
  fn test_scalar_mul_affine_inf() {
    let e = WeierstrassEq::secp256k1();
    let inf = EcPoint::Infinity();
    let g_plus_inf = e.add(&inf, &e.g);
    match (e.g, g_plus_inf) {
      (EcPoint::Affine(c1), EcPoint::Affine(c2)) => {
        assert_eq!(c1, c2);
      },
      _ => {
        panic!("Expected g+inf to be g");
      }
    }
  }

  #[test]
  fn test_scalar_mul_inf_inf() {
    let e = WeierstrassEq::secp256k1();
    let inf = EcPoint::Infinity();
    let g2 = e.add(&inf, &inf);
    match g2 {
      EcPoint::Infinity() => {
      },
      _ => {
        panic!("Expected inf+inf to be inf");
      }
    }
  }

  struct Xy<'a> {
    _n: &'a str,
    x: &'a [u8; 64],
    y: &'a [u8; 64],
  }

  impl<'a> Xy<'a> {
    fn to_ec_point(&'a self, f: Rc<Field>) -> EcPoint {
      let gx = BigUint::parse_bytes(self.x, 16).unwrap();
      let gy = BigUint::parse_bytes(self.y, 16).unwrap();
      EcPoint::Affine(Coord2::new(
        FieldElem::new(f.clone(), gx), 
        FieldElem::new(f, gy),
      ).unwrap())
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

  fn get_g_multiples<'a>(e: &WeierstrassEq) -> Vec<EcPoint> {
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
    let mut gs = vec![e.g.clone()];  // gs[0] is used to match index and g's n and will not be actually used
    for p in ps {
      gs.push(p.to_ec_point(e.f.clone()));
    }
    gs
  }

  #[test]
  fn test_scalar_mul_smaller_nums() {
    let e = WeierstrassEq::secp256k1();
    let gs = get_g_multiples(&e);

    for n in 1usize..=10 {
      let res = e.scalar_mul(&BigUint::from(n));
      match (&res, &gs[n]) {
        (EcPoint::Affine(c1), EcPoint::Affine(c2)) => {
          assert_eq!(c1, c2);
        },
        _ => {
          panic!("Expected g * {} to be g{}", n, n);
        }
      }
    }
  }

  struct ScalarMulTest<'a> {
    k: &'a [u8; 64],
    x: &'a [u8; 64],
    y: &'a [u8; 64],
  }

  #[test]
  fn test_scalar_mul_gen_pubkey() {
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
    let e = WeierstrassEq::secp256k1();

    for t in test_cases {
      let k = BigUint::parse_bytes(t.k, 16).unwrap();
      let x = BigUint::parse_bytes(t.x, 16).unwrap();
      let y = BigUint::parse_bytes(t.y, 16).unwrap();
      let p = EcPoint::Affine(Coord2::new(
        FieldElem::new(e.f.clone(), x), 
        FieldElem::new(e.f.clone(), y),
      ).unwrap());

      let beg = Instant::now();
      let gk = e.scalar_mul(&k);
      let end = beg.elapsed();
      println!("Large number scalar mul done in {}.{:03} sec", end.as_secs(), end.subsec_nanos() / 1_000_000);

      match (&p, &gk) {
        (EcPoint::Affine(c1), EcPoint::Affine(c2)) => {
          assert_eq!(c1, c2);
        },
        _ => {
          panic!("Expected gk to be p");
        }
      }
    }
  }

  #[test]
  fn test_add_different_points() {
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

    let e = WeierstrassEq::secp256k1();
    let gs = get_g_multiples(&e);

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
      let res = e.add(&gs[tc.a], &gs[tc.b]);
      match (&res, &gs[tc.c]) {
        (EcPoint::Affine(c1), EcPoint::Affine(c2)) => {
          assert_eq!(c1, c2);
        },
        _ => {
          panic!("Expected g{} + g{} to be g{}", tc.a, tc.b, tc.c);
        }
      }
    }

    let l1 = large_1.to_ec_point(e.f.clone());
    let l2 = large_2.to_ec_point(e.f.clone());
    let l3 = large_3.to_ec_point(e.f.clone());

    let l1_plus_l2 = e.add(&l1, &l2);
    match (&l1_plus_l2, &l3) {
      (EcPoint::Affine(c1), EcPoint::Affine(c2)) => {
        assert_eq!(c1, c2);
      },
      _ => {
        panic!("Expected l1 + l2 to be l3");
      }
    }
  }
}
