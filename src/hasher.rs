pub trait Hasher<const N: usize> {
  fn get_digest(&self, msg: &[u8]) -> [u8; N]; 
}