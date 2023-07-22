use std::ops::{Add, Mul, Sub, Div};

pub trait AffinePoint {
  type Element: Add<Self::Element> + Sub<Self::Element> + Mul<Self::Element> + Div<Self::Element>;

  fn x(&self) -> Self::Element;
  fn y(&self) -> Self::Element;
}