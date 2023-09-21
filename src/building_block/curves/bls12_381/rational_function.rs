use crate::building_block::{
  curves::bls12_381::{
    fq12::Fq12,
    g1_point::G1Point,
    g2_point::G2Point,
    g12_point::G12Point,
  },
  to_biguint::ToBigUint,
};

enum Line {
  Vertical { x: Fq12 },
  NonVertical { x: Fq12, y: Fq12, slope: Fq12 },
}

pub struct RationalFunction {
  line: Line,
}

macro_rules! impl_new {
  ($point: ty, $func: ident) => {
    impl RationalFunction {
      pub fn $func(p: &$point, q: &$point) -> Self {
        match (p.into(), q.into()) {
          (G12Point::Rational { x: x1_12, y: y1_12 }, G12Point::Rational { x: x2_12, y: y2_12 }) => {
            if p == q {
              Self::handle_tangent(&x1_12, &y1_12)
            }
            else if q == &-p {
              Self::handle_vertical(&x1_12)
            }
            else {
              Self::handle_others(&x1_12, &y1_12, &x2_12, &y2_12)
            }
          },
          _ => panic!("Both points need to be rational"),
        }
      }
    }
  };
}
impl_new!(G1Point, new_g1);
impl_new!(G2Point, new_g2);

macro_rules! impl_eval_with {
  ($point: ty, $func: ident) => {
    impl RationalFunction {
      #[allow(non_snake_case)]
      pub fn $func(&self, q: &$point) -> Fq12 {
        let q12: G12Point = q.into();

        match (q12, &self.line) {
          (G12Point::Rational { x: X, y: _ }, Line::Vertical { x }) => {
            X + -x
          },
          (G12Point::Rational { x: X, y: Y }, Line::NonVertical { x, y, slope }) => {
            -slope * X + Y + -y + slope * x
          },
          _ => panic!("cannot evaluate with point at infinity"),
        }
      }
    }
  };
}
impl_eval_with!(G1Point, eval_with_g1);
impl_eval_with!(G2Point, eval_with_g2);

// y^2 = x^3 + 4: a=0, b
impl RationalFunction {
  fn handle_tangent(x: &Fq12, y: &Fq12) -> Self {
    let two = Fq12::from(&2u8 as &dyn ToBigUint);
    let three = Fq12::from(&3u8 as &dyn ToBigUint);

    let slope = three * x * x * (two * y).inv();

    RationalFunction {
      line: Line::NonVertical {
        x: x.clone(),
        y: y.clone(),
        slope,
      },
    }
  }

  fn handle_vertical(x: &Fq12) -> Self {
    RationalFunction {
      line: Line::Vertical { x: x.clone() },
    }
  }

  fn handle_others(x1: &Fq12, y1: &Fq12, x2: &Fq12, y2: &Fq12) -> Self {
    let slope = (y2 - y1) * (x2 - x1).inv();

    RationalFunction {
      line: Line::NonVertical {
        x: x1.clone(),
        y: y1.clone(),
        slope,
      },
    }
  }
}

