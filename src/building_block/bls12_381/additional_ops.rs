pub trait AdditionalOps {
  fn inv(n: &Self) -> Self;
  fn apply_reduce_rule(n: &Self) -> Self;
}
