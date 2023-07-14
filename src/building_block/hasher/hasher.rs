pub trait Hasher<const N: usize> {
  fn get_digest(&self, msg: &[u8]) -> [u8; N];
  fn get_block_size(&self) -> usize;
}