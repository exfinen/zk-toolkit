pub trait Square<E> {
  fn sq() -> E;
}

pub trait Inverse<E> {
  fn inv(&self) -> E;
}
