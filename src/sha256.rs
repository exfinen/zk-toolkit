#![allow(non_snake_case)]

//use sha2::{Digest, Sha256};
use hex::FromHex;
use crate::hasher::Hasher;
use std::ops::Shr;

// implementation based on: https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.180-4.pdf

macro_rules! hex_to_u32 {
  ($x:expr) => {
    u32::from_be_bytes(<[u8; 4]>::from_hex($x).unwrap())
  };
}

// block consists of sixteen 32-bit words
struct Block<'a> {
  data: &'a[u8],
}

impl<'a> Block<'a> {
  pub fn of(msg: &'a[u8], from: usize) -> Block<'a> {
    let beg = from * 64;
    let end = (from + 1) * 64;
    Block {
      data: &msg[beg..end],
    }
  }

  pub fn at(&self, i: usize) -> u32 {
    let beg = i * 4;
    let end = (i + 1) * 4;
    let buf: [u8; 4] = self.data[beg..end].try_into().unwrap();
    u32::from_be_bytes(buf)
  }
}

#[derive(Clone)]
struct HashValue {
  h: [u32; 8],
}

impl HashValue {
  pub fn consolidate(&self) -> [u8; 32] {
    let mut x = [0u8; 32];
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

pub struct Sha256 {
  K256: [u32; 64],
  initial_hash_value: HashValue,
}

impl Sha256 {
  pub fn new() -> Self {
    Sha256 {
      K256: Self::K256(),
      initial_hash_value: Self::initial_hash_value(),
    }
  }
 
  fn K256() -> [u32; 64] { 
    let K256: Vec<u32> = [
      "428a2f98", "71374491", "b5c0fbcf", "e9b5dba5", "3956c25b", "59f111f1", "923f82a4", "ab1c5ed5",
      "d807aa98", "12835b01", "243185be", "550c7dc3", "72be5d74", "80deb1fe", "9bdc06a7", "c19bf174",
      "e49b69c1", "efbe4786", "0fc19dc6", "240ca1cc", "2de92c6f", "4a7484aa", "5cb0a9dc", "76f988da",
      "983e5152", "a831c66d", "b00327c8", "bf597fc7", "c6e00bf3", "d5a79147", "06ca6351", "14292967",
      "27b70a85", "2e1b2138", "4d2c6dfc", "53380d13", "650a7354", "766a0abb", "81c2c92e", "92722c85",
      "a2bfe8a1", "a81a664b", "c24b8b70", "c76c51a3", "d192e819", "d6990624", "f40e3585", "106aa070",
      "19a4c116", "1e376c08", "2748774c", "34b0bcb5", "391c0cb3", "4ed8aa4a", "5b9cca4f", "682e6ff3",
      "748f82ee", "78a5636f", "84c87814", "8cc70208", "90befffa", "a4506ceb", "bef9a3f7", "c67178f2",
    ].iter().map(|x| hex_to_u32!(x)).collect();
    K256.try_into().unwrap()
  }

  fn initial_hash_value() -> HashValue {
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

  // pub fn get_digest(message: &[u8]) -> [u8; 32] {
  //   let digest: [u8; 32] = Sha256::digest(message).into();
  //   digest
  // }

  // Append the bit 1 to the end of the message, followed by k zero bits, 
  // where k is the smallest, non-negative solution to the equation:
  // l + 1 + k = 448 mod 512 
  // resulting msg will have a length that is a multiple of 512 bits
  fn pad_msg(msg: &[u8]) -> Vec<u8> {
    let mut v = msg.to_vec();

    // add first padding byte w/ leftmost bit 1
    v.push(0b1000_0000u8);

    let v_len = v.len() % 64; 

    // if 512 block containing msg has room to add length part
    if v_len <= 56 {
      let k = 56 - v_len;
      v.extend(vec![0u8; k]);

    } else { // otherwise create another block and store length part there
      // # of bytes remaining in 512 block containing msg
      let rest = 64 - v_len;
      // data part of next 512 block is fully filled by padding
      let k = rest + 56;
      v.extend(vec![0u8; k]);
    }
    // append 8-byte length part to the end 
    let data_bit_len: u64 = (msg.len() * 8).try_into().unwrap();
    v.extend(data_bit_len.to_be_bytes());
    v
  }

  // convert msg whose length is a multiple of 64 into blocks
  // consisting of 16 32-bit words
  fn parse_padded_msg<'a>(msg: &'a Vec<u8>) -> Vec<Block<'a>> {
    let mut blocks = vec![];
    for i in 0..msg.len()/64 {
      let block = Block::of(&msg, i);
      blocks.push(block);
    }
    blocks
  }

  fn sml_sigma_256_0(x: u32) -> u32 {
    x.rotate_right(7) ^ x.rotate_right(18) ^ x.shr(3)
  }

  fn sml_sigma_256_1(x: u32) -> u32 {
    x.rotate_right(17) ^ x.rotate_right(19) ^ x.shr(10)
  }

  fn large_sigma_256_0(x: u32) -> u32 {
    x.rotate_right(2) ^ x.rotate_right(13) ^ x.rotate_right(22)
  }

  fn large_sigma_256_1(x: u32) -> u32 {
    x.rotate_right(6) ^ x.rotate_right(11) ^ x.rotate_right(25)
  }

  fn ch(x: u32, y: u32, z: u32) -> u32 {
    (x & y) ^ (!x & z)
  }

  fn maj(x: u32, y: u32, z: u32) -> u32 {
    (x & y) ^ (x & z) ^ (y & z)
  }

  // using the same parameter names as the spec
  // m = Block, w = Message Schedule
  // using wrapping_add to perform addition in modulo 2^32
  fn prepare_message_schedules<'a>(m: &Block<'a>) -> [u32; 64] {
    let mut W = vec![];
    for t in 0..16 {
      W.push(m.at(t));
    }
    for t in 16..64 {
      let x = Self::sml_sigma_256_1(W[t-2])
        .wrapping_add(W[t-7])
        .wrapping_add(Self::sml_sigma_256_0(W[t-15]))
        .wrapping_add(W[t-16]);
      W.push(x);
    }
    W.try_into().unwrap()
  }

  // using wrapping_add to perform addition in modulo 2^32
  fn compute_hash<'a>(&self, blocks: &Vec<Block<'a>>) -> HashValue {
    let mut tmp: [u32; 8] = [0u32; 8];
    let mut hash_value = self.initial_hash_value.clone();
    for block in blocks {
      let mut a: u32 = hash_value.h[0]; 
      let mut b: u32 = hash_value.h[1]; 
      let mut c: u32 = hash_value.h[2]; 
      let mut d: u32 = hash_value.h[3]; 
      let mut e: u32 = hash_value.h[4]; 
      let mut f: u32 = hash_value.h[5]; 
      let mut g: u32 = hash_value.h[6]; 
      let mut h: u32 = hash_value.h[7]; 

      let W = Self::prepare_message_schedules(block);

      for t in 0..64 {
        let t1 = h.wrapping_add(Self::large_sigma_256_1(e))
          .wrapping_add(Self::ch(e, f, g))
          .wrapping_add(self.K256[t])
          .wrapping_add(W[t]);
        let t2 = Self::large_sigma_256_0(a).wrapping_add(Self::maj(a, b, c));
        h = g;
        g = f;
        f = e;
        e = d.wrapping_add(t1);
        d = c;
        c = b;
        b = a;
        a = t1.wrapping_add(t2);
      }
      
      tmp[0] = a.wrapping_add(hash_value.h[0]);
      tmp[1] = b.wrapping_add(hash_value.h[1]);
      tmp[2] = c.wrapping_add(hash_value.h[2]);
      tmp[3] = d.wrapping_add(hash_value.h[3]);
      tmp[4] = e.wrapping_add(hash_value.h[4]);
      tmp[5] = f.wrapping_add(hash_value.h[5]);
      tmp[6] = g.wrapping_add(hash_value.h[6]);
      tmp[7] = h.wrapping_add(hash_value.h[7]);
      hash_value = HashValue { h: tmp };
    }
    hash_value
  }
}

impl Hasher<32> for Sha256 {
  fn get_digest(&self, msg: &[u8]) -> [u8; 32] {
    let padded_msg = Self::pad_msg(msg);
    let blocks = Self::parse_padded_msg(&padded_msg);
    let hash_value = self.compute_hash(&blocks);
    hash_value.consolidate()
  } 
}

#[cfg(test)]
mod tests {
  use super::*;
  use hex::ToHex;

  #[test]
  fn hash_empty() {
    let hasher = Sha256::new();
    let msg = [];
    let digest = hasher.get_digest(&msg);
    assert_eq!(digest.encode_hex::<String>(), "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
    let digest = hasher.get_digest(&digest);
    assert_eq!(digest.encode_hex::<String>(), "5df6e0e2761359d30a8275058e299fcc0381534545f55cf43e41983f5d4c9456");
  }

  #[test]
  fn hash_abc() {
    let hasher = Sha256::new();
    let msg = [b'a', b'b', b'c'];
    let digest = hasher.get_digest(&msg);
    assert_eq!(digest.encode_hex::<String>(), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
    let digest = hasher.get_digest(&digest);
    assert_eq!(digest.encode_hex::<String>(), "4f8b42c22dd3729b519ba6f68d2da7cc5b2d606d05daed5ad5128cc03e6c6358");
  }

  #[test]
  fn hash_a_times_1mil() {
    let hasher = Sha256::new();
    let msg = [b'a'; 1_000_000];
    let digest = hasher.get_digest(&msg);
    assert_eq!(digest.encode_hex::<String>(), "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0");
    let digest = hasher.get_digest(&digest);
    assert_eq!(digest.encode_hex::<String>(), "80d1189477563e1b5206b2749f1afe4807e5705e8bd77887a60187a712156688");
  }

  #[test]
  fn hash_rc4_key_0_stream_first_16() {
    let hasher = Sha256::new();
    let msg = hex::decode("de188941a3375d3a8a061e67576e926d").unwrap();
    let digest = hasher.get_digest(&msg);
    assert_eq!(digest.encode_hex::<String>(), "067c531269735ca7f541fdaca8f0dc76305d3cada140f89372a410fe5eff6e4d");
    let digest = hasher.get_digest(&digest);
    assert_eq!(digest.encode_hex::<String>(), "2182d3fe9882fd597d25daf6a85e3a574e5a9861dbc75c13ce3f47fe98572246");
  }

  #[test]
  fn hash_rc4_key_0_stream_first_55() {
    let hasher = Sha256::new();
    let msg = hex::decode("de188941a3375d3a8a061e67576e926dc71a7fa3f0cceb97452b4d3227965f9ea8cc75076d9fb9c5417aa5cb30fc22198b34982dbb629e").unwrap();
    let digest = hasher.get_digest(&msg);
    assert_eq!(digest.encode_hex::<String>(), "038051e9c324393bd1ca1978dd0952c2aa3742ca4f1bd5cd4611cea83892d382");
    let digest = hasher.get_digest(&digest);
    assert_eq!(digest.encode_hex::<String>(), "3b4666a5643de038930566a5930713e65d72888d3f51e20f9545329620485b03");
  }

  #[test]
  fn wrapping_add() {
    let x: u32 = 0b1111_1111_1111_1111_1111_1111_1111_1111;
    let y: u32 = 0b0000_0000_0000_0000_0000_0000_0000_0001;
    assert_eq!(x.wrapping_add(y), 0);

    let z: u32 = 0b0000_0000_0000_0000_0000_0000_0000_0010;
    assert_eq!(x.wrapping_add(z), 1);
  }

  #[test]
  fn shr() {
    let x: u32 = 0b0000_0000_0000_0000_0000_0000_0000_0101;
    assert_eq!(x.shr(1u32), 0b0000_0000_0000_0000_0000_0000_0000_0010u32);
    assert_eq!(x.shr(2u32), 0b0000_0000_0000_0000_0000_0000_0000_0001u32);
    assert_eq!(x.shr(3u32), 0b0000_0000_0000_0000_0000_0000_0000_0000u32);
    assert_eq!(x.shr(4u32), 0b0000_0000_0000_0000_0000_0000_0000_0000u32);
  }

  #[test]
  fn rotate_right() {
    let x: u32 = 0b0000_0000_0000_0000_0000_0000_0000_0101;
    assert_eq!(x.rotate_right(1), 0b1000_0000_0000_0000_0000_0000_0000_0010u32);
    assert_eq!(x.rotate_right(2), 0b0100_0000_0000_0000_0000_0000_0000_0001u32);
    assert_eq!(x.rotate_right(3), 0b1010_0000_0000_0000_0000_0000_0000_0000u32);
    assert_eq!(x.rotate_right(4), 0b0101_0000_0000_0000_0000_0000_0000_0000u32);
  }
  
  #[test]
  fn parse_padded_msg_empty_msg() {
    let m = [0u8; 0];
    let pad_m = Sha256::pad_msg(&m);
    let blocks = Sha256::parse_padded_msg(&pad_m);
    assert_eq!(blocks.len(), 1);
  }

  #[test]
  fn add_padding_len_0_msg() {
    let m = [0u8; 0];
    let pad_m = Sha256::pad_msg(&m);
    assert_eq!(pad_m.len(), 64);
    assert_eq!(pad_m[0], 0b1000_0000);
    for i in 1..64 {
      assert_eq!(pad_m[i], 0);
    }
  }

  #[test]
  fn add_padding_len_1_msg() {
    let m = [0b1000_0001; 1];
    let pad_m = Sha256::pad_msg(&m);
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
    let m = [0b1000_0001; 55];
    let pad_m = Sha256::pad_msg(&m);
    assert_eq!(pad_m.len(), 64);
    for i in 0..55 {
      assert_eq!(pad_m[i], 0b1000_0001);
    }
    assert_eq!(pad_m[55], 0b1000_0000);
    assert_eq!((55 * 8u64).to_be_bytes(), &pad_m[56..64]);
  }
  
  #[test]
  fn add_padding_len_56_msg() {
    let m = [0b1000_0001; 56];
    let pad_m = Sha256::pad_msg(&m);
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