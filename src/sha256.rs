//use sha2::{Digest, Sha256};
use std::ops::Index;
use hex::FromHex;

pub struct Sha256;

// represents sixteen 32-bit words
struct Block<'a> {
  data: &'a[u8],
}

impl<'a> Block<'a> {
  pub fn of(msg: &'a[u8], from: usize) -> Block<'a> {
    Block {
      data: &msg[from..from+64],
    }
  }
}

impl<'a> Index<usize> for Block<'a> {
  type Output = [u8];

  fn index(&self, i: usize) -> &'a [u8] {
    println!("Indexing => {}", i);
    &self.data[i*64..i*(64+1)]
  }
}

struct HashValue {
  h0: u32,
  h1: u32,
  h2: u32,
  h3: u32,
  h4: u32,
  h5: u32,
  h6: u32,
  h7: u32,
}

impl HashValue {
  pub fn consolidate(&self) -> [u8; 32] {
    [0u8; 32]
  }
}

macro_rules! hex_to_u32 {
  ($x:expr) => {
    u32::from_be_bytes(<[u8; 4]>::from_hex($x).unwrap())
  };
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

  fn get_initial_hash_value() -> HashValue {
    HashValue {
      h0: hex_to_u32!("6a09e667"),
      h1: hex_to_u32!("bb67ae85"),
      h2: hex_to_u32!("3c6ef372"),
      h3: hex_to_u32!("a54ff53a"),
      h4: hex_to_u32!("510e527f"),
      h5: hex_to_u32!("9b05688c"),
      h6: hex_to_u32!("1f83d9ab"),
      h7: hex_to_u32!("5be0cd19"),
    }
  }

  fn prepare_message_schedules<'a>(_block: &Block<'a>) -> Vec<MessageSchedule> {
    vec![]
  }

  fn compute_hash<'a>(init_hash_value: HashValue, blocks: &Vec<Block<'a>>) -> HashValue {
    let mut hash_value = init_hash_value;
    for block in blocks {
      let _mss = Self::prepare_message_schedules(block);

      let a: u32 = hash_value.h0; 
      let b: u32 = hash_value.h1; 
      let c: u32 = hash_value.h2; 
      let d: u32 = hash_value.h3; 
      let e: u32 = hash_value.h4; 
      let f: u32 = hash_value.h5; 
      let g: u32 = hash_value.h6; 
      let h: u32 = hash_value.h7; 

      for _i in [0..64] {

      }
      
      hash_value = HashValue {
        h0: a + hash_value.h0,
        h1: b + hash_value.h1,
        h2: c + hash_value.h2,
        h3: d + hash_value.h3,
        h4: e + hash_value.h4,
        h5: f + hash_value.h5,
        h6: g + hash_value.h6,
        h7: h + hash_value.h7,
      }
    }
    hash_value
  }

  pub fn get_digest(msg: &[u8]) -> [u8; 32] {
    let padded_msg = Self::pad_msg(msg);
    let blocks = Self::parse_padded_msg(&padded_msg);
    let init_hash_value = Self::get_initial_hash_value();
    let hash_value = Self::compute_hash(init_hash_value, &blocks);
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