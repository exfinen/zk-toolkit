#![allow(non_snake_case)]
use super::hasher::Hasher;
use super::sha_common::{Block, MessageSchedule, HashValue, CoreLogic};

// implementation based on: https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.180-4.pdf

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

#[derive(Clone)]
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
    [
      0x428a2f98d728ae22, 0x7137449123ef65cd, 0xb5c0fbcfec4d3b2f, 0xe9b5dba58189dbbc,
      0x3956c25bf348b538, 0x59f111f1b605d019, 0x923f82a4af194f9b, 0xab1c5ed5da6d8118,
      0xd807aa98a3030242, 0x12835b0145706fbe, 0x243185be4ee4b28c, 0x550c7dc3d5ffb4e2,
      0x72be5d74f27b896f, 0x80deb1fe3b1696b1, 0x9bdc06a725c71235, 0xc19bf174cf692694,
      0xe49b69c19ef14ad2, 0xefbe4786384f25e3, 0x0fc19dc68b8cd5b5, 0x240ca1cc77ac9c65,
      0x2de92c6f592b0275, 0x4a7484aa6ea6e483, 0x5cb0a9dcbd41fbd4, 0x76f988da831153b5,
      0x983e5152ee66dfab, 0xa831c66d2db43210, 0xb00327c898fb213f, 0xbf597fc7beef0ee4,
      0xc6e00bf33da88fc2, 0xd5a79147930aa725, 0x06ca6351e003826f, 0x142929670a0e6e70,
      0x27b70a8546d22ffc, 0x2e1b21385c26c926, 0x4d2c6dfc5ac42aed, 0x53380d139d95b3df,
      0x650a73548baf63de, 0x766a0abb3c77b2a8, 0x81c2c92e47edaee6, 0x92722c851482353b,
      0xa2bfe8a14cf10364, 0xa81a664bbc423001, 0xc24b8b70d0f89791, 0xc76c51a30654be30,
      0xd192e819d6ef5218, 0xd69906245565a910, 0xf40e35855771202a, 0x106aa07032bbd1b8,
      0x19a4c116b8d2d0c8, 0x1e376c085141ab53, 0x2748774cdf8eeb99, 0x34b0bcb5e19b48a8,
      0x391c0cb3c5c95a63, 0x4ed8aa4ae3418acb, 0x5b9cca4f7763e373, 0x682e6ff3d6b2b8a3,
      0x748f82ee5defb2fc, 0x78a5636f43172f60, 0x84c87814a1f0ab72, 0x8cc702081a6439ec,
      0x90befffa23631e28, 0xa4506cebde82bde9, 0xbef9a3f7b2c67915, 0xc67178f2e372532b,
      0xca273eceea26619c, 0xd186b8c721c0c207, 0xeada7dd6cde0eb1e, 0xf57d4f7fee6ed178,
      0x06f067aa72176fba, 0x0a637dc5a2c898a6, 0x113f9804bef90dae, 0x1b710b35131c471b,
      0x28db77f523047d84, 0x32caab7b40c72493, 0x3c9ebe0a15c9bebc, 0x431d67c49c100d4c,
      0x4cc5d4becb3e42b6, 0x597f299cfc657e2a, 0x5fcb6fab3ad6faec, 0x6c44198c4a475817,
    ]
  }

  fn get_initial_hash_value() -> HashValue<u64> {
    HashValue { h: [
      0x6a09e667f3bcc908,
      0xbb67ae8584caa73b,
      0x3c6ef372fe94f82b,
      0xa54ff53a5f1d36f1,
      0x510e527fade682d1,
      0x9b05688c2b3e6c1f,
      0x1f83d9abfb41bd6b,
      0x5be0cd19137e2179,
    ] }
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