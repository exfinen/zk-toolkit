#![allow(non_snake_case)]
use hex::FromHex;
use crate::hasher::Hasher;
use crate::sha_common::{Block, MessageSchedule, HashValue, CoreLogic};

// implementation based on: https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.180-4.pdf

macro_rules! hex_to_u64 {
  ($x:expr) => {
    u64::from_be_bytes(<[u8; 8]>::from_hex($x).unwrap())
  };
}

const BLOCK_SIZE: usize = 128;
const DIGEST_SIZE: usize = 64;

impl HashValue<u64> {
  pub fn to_u8_array(&self) -> [u8; DIGEST_SIZE] {
    let mut x = [0u8; DIGEST_SIZE];
    for i in 0..8 {
      let bytes = self.h[i].to_be_bytes();
      x[i * 8 + 0] = bytes[0];
      x[i * 8 + 1] = bytes[1];
      x[i * 8 + 2] = bytes[2];
      x[i * 8 + 3] = bytes[3];
      x[i * 8 + 4] = bytes[4];
      x[i * 8 + 5] = bytes[5];
      x[i * 8 + 6] = bytes[6];
      x[i * 8 + 7] = bytes[7];
    }
    x
  }
}

impl<'a> MessageSchedule<u64> for Block<'a> {
  fn message_schedule(&self, idx: usize) -> u64 {
    let beg = idx * 8;
    let end = (idx + 1) * 8;
    let buf: [u8; 8] = self.data[beg..end].try_into().unwrap();
    u64::from_be_bytes(buf)
  }
}

pub struct Sha512();

impl<'a> CoreLogic<
  'a,
  u64,
  80,  // message schedule len
  BLOCK_SIZE,
  16,  // length part len
  1, 8, 7,
  19, 61, 6,
  28, 34, 39,
  14, 18, 41,
> for Sha512 {

  fn get_K() -> [u64; 80] { 
    let K: Vec<u64> = [
      "428a2f98d728ae22", "7137449123ef65cd", "b5c0fbcfec4d3b2f", "e9b5dba58189dbbc",
      "3956c25bf348b538", "59f111f1b605d019", "923f82a4af194f9b", "ab1c5ed5da6d8118",
      "d807aa98a3030242", "12835b0145706fbe", "243185be4ee4b28c", "550c7dc3d5ffb4e2",
      "72be5d74f27b896f", "80deb1fe3b1696b1", "9bdc06a725c71235", "c19bf174cf692694",
      "e49b69c19ef14ad2", "efbe4786384f25e3", "0fc19dc68b8cd5b5", "240ca1cc77ac9c65",
      "2de92c6f592b0275", "4a7484aa6ea6e483", "5cb0a9dcbd41fbd4", "76f988da831153b5",
      "983e5152ee66dfab", "a831c66d2db43210", "b00327c898fb213f", "bf597fc7beef0ee4",
      "c6e00bf33da88fc2", "d5a79147930aa725", "06ca6351e003826f", "142929670a0e6e70",
      "27b70a8546d22ffc", "2e1b21385c26c926", "4d2c6dfc5ac42aed", "53380d139d95b3df",
      "650a73548baf63de", "766a0abb3c77b2a8", "81c2c92e47edaee6", "92722c851482353b",
      "a2bfe8a14cf10364", "a81a664bbc423001", "c24b8b70d0f89791", "c76c51a30654be30",
      "d192e819d6ef5218", "d69906245565a910", "f40e35855771202a", "106aa07032bbd1b8",
      "19a4c116b8d2d0c8", "1e376c085141ab53", "2748774cdf8eeb99", "34b0bcb5e19b48a8",
      "391c0cb3c5c95a63", "4ed8aa4ae3418acb", "5b9cca4f7763e373", "682e6ff3d6b2b8a3",
      "748f82ee5defb2fc", "78a5636f43172f60", "84c87814a1f0ab72", "8cc702081a6439ec",
      "90befffa23631e28", "a4506cebde82bde9", "bef9a3f7b2c67915", "c67178f2e372532b",
      "ca273eceea26619c", "d186b8c721c0c207", "eada7dd6cde0eb1e", "f57d4f7fee6ed178",
      "06f067aa72176fba", "0a637dc5a2c898a6", "113f9804bef90dae", "1b710b35131c471b",
      "28db77f523047d84", "32caab7b40c72493", "3c9ebe0a15c9bebc", "431d67c49c100d4c",
      "4cc5d4becb3e42b6", "597f299cfc657e2a", "5fcb6fab3ad6faec", "6c44198c4a475817",
    ].iter().map(|x| hex_to_u64!(x)).collect();
    K.try_into().unwrap()
  }

  fn get_initial_hash_value() -> HashValue<u64> {
    let h: Vec<u64> = [
      "6a09e667f3bcc908",
      "bb67ae8584caa73b",
      "3c6ef372fe94f82b",
      "a54ff53a5f1d36f1",
      "510e527fade682d1",
      "9b05688c2b3e6c1f",
      "1f83d9abfb41bd6b",
      "5be0cd19137e2179",
    ].iter().map(|x| hex_to_u64!(x)).collect();
    HashValue { h: h.try_into().unwrap() }
  }
}

impl Hasher<DIGEST_SIZE> for Sha512 {
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
  fn hash_abc() {
    let hasher = Sha512();
    let msg = [b'a', b'b', b'c'];
    let digest = hasher.get_digest(&msg);
    assert_eq!(digest.encode_hex::<String>(), "ddaf35a193617abacc417349ae20413112e6fa4e89a97ea20a9eeee64b55d39a2192992a274fc1a836ba3c23a3feebbd454d4423643ce80e2a9ac94fa54ca49f");
  }

  #[test]
  fn hash_empty_msg() {
    let hasher = Sha512();
    let msg = b"";
    let digest = hasher.get_digest(msg);
    assert_eq!(digest.encode_hex::<String>(), "cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e");
  }

  #[test]
  fn hash_448_bit_msg() {
    let hasher = Sha512();
    let msg = b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq";
    let digest = hasher.get_digest(msg);
    assert_eq!(digest.encode_hex::<String>(), "204a8fc6dda82f0a0ced7beb8e08a41657c16ef468b228a8279be331a703c33596fd15c13b1b07f9aa1d3bea57789ca031ad85c7a71dd70354ec631238ca3445");
  }

  #[test]
  fn hash_896_bit_msg() {
    let hasher = Sha512();
    let msg = b"abcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmnhijklmnoijklmnopjklmnopqklmnopqrlmnopqrsmnopqrstnopqrstu";
    let digest = hasher.get_digest(msg);
    assert_eq!(digest.encode_hex::<String>(), "8e959b75dae313da8cf4f72814fc143f8f7779c6eb9f7fa17299aeadb6889018501d289e4900f7e4331b99dec4b5433ac7d329eeb6dd26545e96e55b874be909");
  }

  #[test]
  fn hash_1_mil_a() {
    let hasher = Sha512();
    let msg = [b'a'; 1_000_000];
    let digest = hasher.get_digest(&msg);
    assert_eq!(digest.encode_hex::<String>(), "e718483d0ce769644e2e42c7bc15b4638e1f98b13b2044285632a803afa973ebde0ff244877ea60a4cb0432ce577c31beb009c5c2c49aa2e4eadb217ad8cc09b");
  }
  
  #[test]
  fn parse_padded_msg_empty_msg() {
    let hasher = Sha512();
    let m = [0u8; 0];
    let pad_m = hasher.pad_msg(&m);
    let blocks = Block::parse_padded_msg(&pad_m, BLOCK_SIZE);
    assert_eq!(blocks.len(), 1);
  }

  #[test]
  fn add_padding_len_0_msg() {
    let hasher = Sha512();
    let m = [0u8; 0];
    let pad_m = hasher.pad_msg(&m);
    assert_eq!(pad_m.len(), 128);
    assert_eq!(pad_m[0], 0b1000_0000);
    for i in 1..128 {
      assert_eq!(pad_m[i], 0);
    }
  }

  #[test]
  fn add_padding_len_1_msg() {
    let hasher = Sha512();
    let m = [0b1000_0001; 1];
    let pad_m = hasher.pad_msg(&m);
    assert_eq!(pad_m.len(), 128);
    assert_eq!(pad_m[0], 0b1000_0001);
    assert_eq!(pad_m[1], 0b1000_0000);
    for i in 2..112 {
      assert_eq!(pad_m[i], 0);
    }
    assert_eq!(8u128.to_be_bytes(), &pad_m[112..128]);
  }
  
  #[test]
  fn add_padding_len_111_msg() {
    let hasher = Sha512();
    let m = [0b1000_0001; 111];
    let pad_m = hasher.pad_msg(&m);
    assert_eq!(pad_m.len(), 128);
    for i in 0..111 {
      assert_eq!(pad_m[i], 0b1000_0001);
    }
    assert_eq!(pad_m[111], 0b1000_0000);
    assert_eq!((111 * 8u128).to_be_bytes(), &pad_m[112..128]);
  }
  
  #[test]
  fn add_padding_len_112_msg() {
    let hasher = Sha512();
    let m = [0b1000_0001; 112];
    let pad_m = hasher.pad_msg(&m);
    assert_eq!(pad_m.len(), 256);
    for i in 0..112 {
      assert_eq!(pad_m[i], 0b1000_0001);
    }
    assert_eq!(pad_m[112], 0b1000_0000);
    for i in 113..240 {
      assert_eq!(pad_m[i], 0);
    }
    assert_eq!((112 * 8u128).to_be_bytes(), &pad_m[240..256]);
  }
}