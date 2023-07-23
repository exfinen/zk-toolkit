#![allow(non_snake_case)]
use super::hasher::Hasher;
use super::sha_common::{Block, MessageSchedule, HashValue, CoreLogic};

// implementation based on: https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.180-4.pdf

const BLOCK_SIZE: usize = 64;
const DIGEST_SIZE: usize = 32;

impl HashValue<u32> {
  pub fn to_u8_array(&self) -> [u8; DIGEST_SIZE] {
    let mut x = [0u8; DIGEST_SIZE];
    for i in 0..8 {
      let bytes = self.h[i].to_be_bytes();
      x[i * 4 + 0] = bytes[0];
      x[i * 4 + 1] = bytes[1];
      x[i * 4 + 2] = bytes[2];
      x[i * 4 + 3] = bytes[3];
    }
    x
  }
}

impl<'a> MessageSchedule<u32> for Block<'a> {
  fn message_schedule(&self, idx: usize) -> u32 {
    let beg = idx * 4;
    let end = (idx + 1) * 4;
    let buf: [u8; 4] = self.data[beg..end].try_into().unwrap();
    u32::from_be_bytes(buf)
  }
}

#[derive(Clone)]
pub struct Sha256();

impl<'a> CoreLogic<
  'a,
  u32,
  64,  // message schedule len
  BLOCK_SIZE,
  8,  // length part len
  7, 18, 3,
  17, 19, 10,
  2, 13, 22,
  6, 11, 25,
> for Sha256 {

  fn get_K() -> [u32; 64] {
    [
      0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
      0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
      0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
      0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
      0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
      0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
      0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
      0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
    ]
  }

  fn get_initial_hash_value() -> HashValue<u32> {
    HashValue { h: [
      0x6a09e667,
      0xbb67ae85,
      0x3c6ef372,
      0xa54ff53a,
      0x510e527f,
      0x9b05688c,
      0x1f83d9ab,
      0x5be0cd19,
    ] }
  }
}

impl Hasher<DIGEST_SIZE> for Sha256 {
  fn get_digest(&self, msg: &[u8]) -> [u8; DIGEST_SIZE] {
    let padded_msg = self.pad_msg(msg);
    let blocks = Block::parse_padded_msg(&padded_msg, BLOCK_SIZE);
    let hash_value = self.compute_hash(&blocks);
    hash_value.to_u8_array()
  }

  fn get_block_size(&self) -> usize {
    BLOCK_SIZE
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use hex::ToHex;

  #[test]
  fn hash_empty() {
    let hasher = Sha256();
    let msg = [];
    let digest = hasher.get_digest(&msg);
    assert_eq!(digest.encode_hex::<String>(), "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
    let digest = hasher.get_digest(&digest);
    assert_eq!(digest.encode_hex::<String>(), "5df6e0e2761359d30a8275058e299fcc0381534545f55cf43e41983f5d4c9456");
  }

  #[test]
  fn hash_abc() {
    let hasher = Sha256();
    let msg = [b'a', b'b', b'c'];
    let digest = hasher.get_digest(&msg);
    assert_eq!(digest.encode_hex::<String>(), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
    let digest = hasher.get_digest(&digest);
    assert_eq!(digest.encode_hex::<String>(), "4f8b42c22dd3729b519ba6f68d2da7cc5b2d606d05daed5ad5128cc03e6c6358");
  }

  #[test]
  fn hash_a_times_1mil() {
    let hasher = Sha256();
    let msg = [b'a'; 1_000_000];
    let digest = hasher.get_digest(&msg);
    assert_eq!(digest.encode_hex::<String>(), "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0");
    let digest = hasher.get_digest(&digest);
    assert_eq!(digest.encode_hex::<String>(), "80d1189477563e1b5206b2749f1afe4807e5705e8bd77887a60187a712156688");
  }

  #[test]
  fn hash_rc4_key_0_stream_first_16() {
    let hasher = Sha256();
    let msg = hex::decode("de188941a3375d3a8a061e67576e926d").unwrap();
    let digest = hasher.get_digest(&msg);
    assert_eq!(digest.encode_hex::<String>(), "067c531269735ca7f541fdaca8f0dc76305d3cada140f89372a410fe5eff6e4d");
    let digest = hasher.get_digest(&digest);
    assert_eq!(digest.encode_hex::<String>(), "2182d3fe9882fd597d25daf6a85e3a574e5a9861dbc75c13ce3f47fe98572246");
  }

  #[test]
  fn hash_rc4_key_0_stream_first_55() {
    let hasher = Sha256();
    let msg = hex::decode("de188941a3375d3a8a061e67576e926dc71a7fa3f0cceb97452b4d3227965f9ea8cc75076d9fb9c5417aa5cb30fc22198b34982dbb629e").unwrap();
    let digest = hasher.get_digest(&msg);
    assert_eq!(digest.encode_hex::<String>(), "038051e9c324393bd1ca1978dd0952c2aa3742ca4f1bd5cd4611cea83892d382");
    let digest = hasher.get_digest(&digest);
    assert_eq!(digest.encode_hex::<String>(), "3b4666a5643de038930566a5930713e65d72888d3f51e20f9545329620485b03");
  }

  #[test]
  fn parse_padded_msg_empty_msg() {
    let hasher = Sha256();
    let m = [0u8; 0];
    let pad_m = hasher.pad_msg(&m);
    let blocks = Block::parse_padded_msg(&pad_m, BLOCK_SIZE);
    assert_eq!(blocks.len(), 1);
  }

  #[test]
  fn add_padding_len_0_msg() {
    let hasher = Sha256();
    let m = [0u8; 0];
    let pad_m = hasher.pad_msg(&m);
    assert_eq!(pad_m.len(), 64);
    assert_eq!(pad_m[0], 0b1000_0000);
    for i in 1..64 {
      assert_eq!(pad_m[i], 0);
    }
  }

  #[test]
  fn add_padding_len_1_msg() {
    let hasher = Sha256();
    let m = [0b1000_0001; 1];
    let pad_m = hasher.pad_msg(&m);
    assert_eq!(pad_m.len(), 64);
    assert_eq!(pad_m[0], 0b1000_0001);
    assert_eq!(pad_m[1], 0b1000_0000);
    for i in 2..56 {
      assert_eq!(pad_m[i], 0);
    }
    assert_eq!(8u64.to_be_bytes(), &pad_m[56..64]);
  }

  #[test]
  fn add_padding_len_55_msg() {
    let hasher = Sha256();
    let m = [0b1000_0001; 55];
    let pad_m = hasher.pad_msg(&m);
    assert_eq!(pad_m.len(), 64);
    for i in 0..55 {
      assert_eq!(pad_m[i], 0b1000_0001);
    }
    assert_eq!(pad_m[55], 0b1000_0000);
    assert_eq!((55 * 8u64).to_be_bytes(), &pad_m[56..64]);
  }

  #[test]
  fn add_padding_len_56_msg() {
    let hasher = Sha256();
    let m = [0b1000_0001; 56];
    let pad_m = hasher.pad_msg(&m);
    assert_eq!(pad_m.len(), 128);
    for i in 0..56 {
      assert_eq!(pad_m[i], 0b1000_0001);
    }
    assert_eq!(pad_m[56], 0b1000_0000);
    for i in 57..122 {
      assert_eq!(pad_m[i], 0);
    }
    assert_eq!((56 * 8u64).to_be_bytes(), &pad_m[120..128]);
  }
}