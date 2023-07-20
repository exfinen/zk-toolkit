pub trait CurveEquation<P> {
  fn is_rational_point(&self, pt: &P) -> bool;
}
