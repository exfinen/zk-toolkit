use std::ops::Add;
use crate::building_block::field::field::Field;

pub trait PointAdder<P, F>
  where
    F: Field<F>,
    P: Add<P>,
{
    fn add(f: &F, p1: &P, p2: &P) -> P;
}