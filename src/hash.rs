use sha2::{Digest, Sha256};

pub struct Hash;

impl Hash {
  pub fn sha256(message: &[u8]) -> [u8; 32] {
    let digest: [u8; 32] = Sha256::digest(message).into();
    digest
  }
}
