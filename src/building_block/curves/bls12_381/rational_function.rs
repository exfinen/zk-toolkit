use crate::building_block::curves::bls12_381::{
  fq1::Fq1,
  fq2::Fq2,
  fq12::Fq12,
  g1_point::G1Point,
  g2_point::G2Point,
};

enum Line {
  Vertical { x: Fq2 },
  NonVertical { x: Fq2, y: Fq2, slope: Fq2 },
}

pub struct RationalFunction {
  line: Line,
}

// y^2 = x^3 + 4: a=0, b
impl RationalFunction {
  pub fn new(p: &G1Point, q: &G1Point) -> Self {
    match (p, q) {
      (G1Point::Rational { x: x1, y: y1 }, G1Point::Rational { x: x2, y: y2 }) => {
        if p == q {
          let two = Fq1::from_to_biguint(&2u8);
          let three = Fq1::from_to_biguint(&3u8);
          let slope = three * x1 * x1 * (two * y1).inv();
          RationalFunction {
            line: Line::NonVertical {
              x: x1.into_fq2(),
              y: y1.into_fq2(),
              slope: slope.into_fq2(),
            },
          }
        } else if q == &-p {
          RationalFunction {
            line: Line::Vertical { x: x1.into_fq2() },
          }
        } else {
          // other lines
          let slope = (y2 - y1) * (x2 - x1).inv();
          RationalFunction {
            line: Line::NonVertical {
              x: x1.into_fq2(),
              y: y1.into_fq2(),
              slope: slope.into_fq2(),
            },
          }
        }
      },
      _ => panic!("Both G1Points are expected to be rational"),
    }
  }

  #[allow(non_snake_case)]
  pub fn eval_at(&self, q: &G2Point) -> Fq12 {
    let elem = match (q, &self.line) {
      (G2Point::Rational { x: X, y: _ }, Line::Vertical { x }) => {
        X + -x
      },
      (G2Point::Rational { x: X, y: Y }, Line::NonVertical { x, y, slope }) => {
        -slope * X + Y + -y + slope * x
      },
      _ => panic!("G2Point must be rational"),
    };
    elem.untwist()
  }
}
