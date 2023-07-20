use crate::building_block::{
  field::field_elem::NewFieldElem,
  additive_identity::AdditiveIdentity,
};

pub trait Zero<T> where T: AdditiveIdentity<T> {
  type Field: NewFieldElem<T>;

  fn is_zero(&self) -> bool;
  fn get_zero(f: &T) -> T;
}
