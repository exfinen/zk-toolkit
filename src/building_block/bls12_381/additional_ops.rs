pub trait AdditionalOps {
  fn inv(n: &Self) -> Self;
  fn reduce(n: &Self) -> Self;
  fn zero() -> Self;
}
