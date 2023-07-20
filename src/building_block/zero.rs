use crate::building_block::additive_identity::AdditiveIdentity;

pub trait Zero<T> where T: AdditiveIdentity<T> {
  fn is_zero(&self) -> bool;
  fn get_zero(f: &T) -> T;
}
