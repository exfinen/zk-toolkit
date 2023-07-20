use crate::building_block::additive_identity::AdditiveIdentity;

pub trait Zero<T> where T: AdditiveIdentity {
  fn is_zero(&self) -> bool;
  fn get_zero(t: &T) -> T;
}
