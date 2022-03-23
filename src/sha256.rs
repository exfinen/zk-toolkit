//use sha2::{Digest, Sha256};
use hex::FromHex;

// implementation based on: https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.180-4.pdf

macro_rules! hex_to_u32 {
  ($x:expr) => {
    u32::from_be_bytes(<[u8; 4]>::from_hex($x).unwrap())
  };
}

// constants
const w: u32 = 32u32;

pub struct Sha256;

impl Sha256 {
  const fn K256() -> [u32; 64] { 
    let K256: Vec<u32> = [
      "428a2f98", "71374491", "b5c0fbcf", "e9b5dba5", "3956c25b", "59f111f1", "923f82a4", "ab1c5ed5",
      "d807aa98", "12835b01", "243185be", "550c7dc3", "72be5d74", "80deb1fe", "9bdc06a7", "c19bf174",
      "e49b69c1", "efbe4786", "0fc19dc6", "240ca1cc", "2de92c6f", "4a7484aa", "5cb0a9dc", "76f988da",
      "983e5152", "a831c66d", "b00327c8", "bf597fc7", "c6e00bf3", "d5a79147", "06ca6351", "14292967",
      "27b70a85", "2e1b2138", "4d2c6dfc", "53380d13", "650a7354", "766a0abb", "81c2c92e", "92722c85",
      "a2bfe8a1", "a81a664b", "c24b8b70", "c76c51a3", "d192e819", "d6990624", "f40e3585", "106aa070",
      "19a4c116", "1e376c08", "2748774c", "34b0bcb5", "391c0cb3", "4ed8aa4a", "5b9cca4f", "682e6ff3",
      "748f82ee", "78a5636f", "84c87814", "8cc70208", "90befffa", "a4506ceb", "bef9a3f7", "c67178f2",
    ].iter().map(|hex| hex_to_u32!(hex)).collect();
    K256.try_into().unwrap()
  }
  const fn initial_hash_value() -> HashValue {
    let h: Vec<u32> = [
      "6a09e667", "bb67ae85", "3c6ef372", "a54ff53a", "510e527f", "9b05688c", "1f83d9ab", "5be0cd19",
    ].iter().map(|hex| hex_to_u32!(hex)).collect();
    HashValue { h: h.try_into().unwrap() }
  }

}
// block consists of sixteen 32-bit words
struct Block<'a> {
  data: &'a[u8],
}

impl<'a> Block<'a> {
  pub fn of(msg: &'a[u8], from: usize) -> Block<'a> {
    Block {
      data: &msg[from..from+64],
    }
  }

  pub fn at(&self, i: usize) -> u32 {
    let buf: [u8; 4] = self.data[i*32..i*(32+1)].try_into().unwrap();
    u32::from_be_bytes(buf)
  }
}

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

type MessageSchedule = u32;

impl Sha256 {
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
    for i in 0..msg.len()/8 {
      let block = Block::of(&msg, i);
      blocks.push(block);
    }
    blocks
  }

  // rorate right x by n positions
  fn rotr(x: u32, n: u32) -> u32 {
    (x >> n) | (x << (w - n))
  }

  // shift right x by n positions
  fn shr(x: u32, n: u32) -> u32 {
    x >> n
  }

  fn sml_sigma_256_0(x: u32) -> u32 {
    Self::rotr(x, 7) ^ Self::rotr(x, 18) ^ Self::shr(x, 3)
  }

  fn sml_sigma_256_1(x: u32) -> u32 {
    Self::rotr(x, 17) ^ Self::rotr(x, 19) ^ Self::shr(x, 10)
  }

  fn large_sigma_256_0(x: u32) -> u32 {
    Self::rotr(x, 2) ^ Self::rotr(x, 13) ^ Self::rotr(x, 22)
  }

  fn large_sigma_256_1(x: u32) -> u32 {
    Self::rotr(x, 6) ^ Self::rotr(x, 11) ^ Self::rotr(x, 25)
  }

  fn ch(x: u32, y: u32, z: u32) -> u32 {
    (x & y) ^ (!x & z)
  }

  fn maj(x: u32, y: u32, z: u32) -> u32 {
    (x & y) ^ (x & z) ^ (y & z)
  }

  // using the same parameter names as the spec
  // m = Block, w = Message Schedule
  fn prepare_message_schedules<'a>(m: &Block<'a>) -> [u32; 16] {
    let mut W = vec![];
    for t in 0..16 {
      W.push(m.at(t));
    }
    for t in 16..64 {
      W.push(Self::sml_sigma_256_1(W[t-2]) + W[t-7] + Self::sml_sigma_256_0(W[t-15]) + W[t-16]);
    }
    W.try_into().unwrap()
  }

  fn compute_hash<'a>(&self, init_hash_value: HashValue, blocks: &Vec<Block<'a>>) -> HashValue {
    let mut tmp: [u32; 8] = [0u32; 8];
    let mut hash_value = init_hash_value;
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
      let K256 = Self::K256();

      for t in 0..64 {
        let t1 = h + Self::large_sigma_256_1(e) + Self::ch(e, f, g) + K256[t] + W[t];
        let t2 = Self::large_sigma_256_0(a) + Self::maj(a, b, c);
        h = g;
        g = f;
        f = e;
        e = d + t1;
        d = c;
        c = b;
        b = a;
        a = t1 + t2;
      }
      
      tmp[0] = a + hash_value.h[0];
      tmp[1] = b + hash_value.h[1];
      tmp[2] = c + hash_value.h[2];
      tmp[3] = d + hash_value.h[3];
      tmp[4] = e + hash_value.h[4];
      tmp[5] = f + hash_value.h[5];
      tmp[6] = g + hash_value.h[6];
      tmp[7] = h + hash_value.h[7];
      hash_value = HashValue { h: tmp };
    }
    hash_value
  }

  pub fn get_digest(&self, msg: &[u8]) -> [u8; 32] {
    let padded_msg = Self::pad_msg(msg);
    let blocks = Self::parse_padded_msg(&padded_msg);
    let init_hash_value = Self::initial_hash_value();
    let hash_value = self.compute_hash(init_hash_value, &blocks);
    hash_value.consolidate()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

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