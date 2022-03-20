use sha2::{Digest, Sha256};

pub struct Hash;

impl Hash {
  pub fn sha256(message: &[u8]) -> [u8; 32] {
    let digest: [u8; 32] = Sha256::digest(message).into();
    digest
  }

  pub fn sha256_myimpl(message: &[u8]) -> [u8; 32] {
    // convert the message to bit array

    // sha-256's block size is 512 bits
    // add padding to bit array so that the array size becomes a mutiple of 512

    // create 32x64 message schedule from each of the 512-bit chunks

    // (msg sche, pre-defined 64 constants, 8 default values) -> 32-bit x 8

    // combine 32-bit x 8 chunks

  }
}
