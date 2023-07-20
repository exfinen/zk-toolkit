use std::ops::Add;

pub trait PointAdder<P>
  where
    P: Add<P>,
{
  type Element;

  fn add(&self, p1: &P, p2: &P) -> P;
}
