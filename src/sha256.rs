#![allow(non_snake_case)]
use hex::FromHex;
use crate::hasher::Hasher;
use crate::sha::{Block, MessageSchedule, HashValue, ShaFunctions};

// implementation based on: https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.180-4.pdf

macro_rules! hex_to_u32 {
  ($x:expr) => {
    u32::from_be_bytes(<[u8; 4]>::from_hex($x).unwrap())
  };
}

const BLOCK_SIZE: usize = 64;
const DIGEST_SIZE: usize = 32;

impl HashValue<u32> {
  pub fn consolidate(&self) -> [u8; DIGEST_SIZE] {
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

pub struct Sha256();

impl<'a> ShaFunctions<
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
    let K: Vec<u32> = [
      "428a2f98", "71374491", "b5c0fbcf", "e9b5dba5", "3956c25b", "59f111f1", "923f82a4", "ab1c5ed5",
      "d807aa98", "12835b01", "243185be", "550c7dc3", "72be5d74", "80deb1fe", "9bdc06a7", "c19bf174",
      "e49b69c1", "efbe4786", "0fc19dc6", "240ca1cc", "2de92c6f", "4a7484aa", "5cb0a9dc", "76f988da",
      "983e5152", "a831c66d", "b00327c8", "bf597fc7", "c6e00bf3", "d5a79147", "06ca6351", "14292967",
      "27b70a85", "2e1b2138", "4d2c6dfc", "53380d13", "650a7354", "766a0abb", "81c2c92e", "92722c85",
      "a2bfe8a1", "a81a664b", "c24b8b70", "c76c51a3", "d192e819", "d6990624", "f40e3585", "106aa070",
      "19a4c116", "1e376c08", "2748774c", "34b0bcb5", "391c0cb3", "4ed8aa4a", "5b9cca4f", "682e6ff3",
      "748f82ee", "78a5636f", "84c87814", "8cc70208", "90befffa", "a4506ceb", "bef9a3f7", "c67178f2",
    ].iter().map(|x| hex_to_u32!(x)).collect();
    K.try_into().unwrap()
  }

  fn get_initial_hash_value() -> HashValue<u32> {
    let h: Vec<u32> = [
      "6a09e667", 
      "bb67ae85", 
      "3c6ef372", 
      "a54ff53a", 
      "510e527f", 
      "9b05688c", 
      "1f83d9ab", 
      "5be0cd19",
    ].iter().map(|x| hex_to_u32!(x)).collect();
    HashValue { h: h.try_into().unwrap() }
  }
}

impl Hasher<DIGEST_SIZE> for Sha256 {
  fn get_digest(&self, msg: &[u8]) -> [u8; DIGEST_SIZE] {
    let padded_msg = self.pad_msg(msg);
    let blocks = Block::parse_padded_msg(&padded_msg, BLOCK_SIZE);
    let hash_value = self.compute_hash(&blocks);
    hash_value.consolidate()
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