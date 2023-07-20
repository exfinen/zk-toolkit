use crate::building_block::to_biguint::ToBigUint;

pub trait NewFieldElem<E> {
  fn elem(n: &dyn ToBigUint) -> E;
}
