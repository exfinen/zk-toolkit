pub trait Zero<T> {
  fn zero() -> T;
  fn is_zero(&self) -> bool;
}
