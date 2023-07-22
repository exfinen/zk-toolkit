pub trait Zero<T> {
  fn zero(&self) -> T;
  fn is_zero(&self) -> bool;
}
