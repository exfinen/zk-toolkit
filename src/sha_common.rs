#![allow(non_snake_case)]
use num_traits::{int::PrimInt, WrappingAdd};
use std::fmt::Debug;

// implementation based on: https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.180-4.pdf

#[derive(Clone)]
pub struct HashValue<U> {
  pub h: [U; 8],
}

// block consists of sixteen words
#[derive(Debug)]
pub struct Block<'a> {
  pub data: &'a[u8],
}

impl<'a> Block<'a> {
  pub fn of(msg: &'a[u8], from: usize, block_size: usize) -> Block<'a> {
    let beg = from * block_size;
    let end = (from + 1) * block_size;
    Block::<'a> {
      data: &msg[beg..end],
    }
  }

  // convert msg whose length is a multiple of BLOCK_SIZE into blocks
  // consisting of 16 words
  pub fn parse_padded_msg(msg: &'a Vec<u8>, block_size: usize) -> Vec<Block<'a>> {
    let mut blocks: Vec<Block<'a>> = vec![];
    let num_blocks = msg.len() / block_size;
    for i in 0..num_blocks {
      let block: Block<'a> = Block::of(&msg, i, block_size);
      blocks.push(block);
    }
    blocks
  }
}

pub trait MessageSchedule<U: PrimInt> {
  fn message_schedule(&self, idx: usize) -> U;
}

pub trait CoreLogic<
  'a,
  U: PrimInt + WrappingAdd + Debug + Default,
  const MSG_SCHE_LEN: usize,
  const BLOCK_SIZE: usize,
  const LENGTH_PART_LEN: usize,
  const L0_1: u8, const L0_2: u8, const L0_3: u8,
  const L1_1: u8, const L1_2: u8, const L1_3: u8,
  const U0_1: u8, const U0_2: u8, const U0_3: u8,
  const U1_1: u8, const U1_2: u8, const U1_3: u8,
> where Block<'a>: MessageSchedule<U>
{
  // lower case sigma function 0
  fn lc_sigma_0(x: U) -> U {
    x.rotate_right(L0_1.into()) ^ x.rotate_right(L0_2.into()) ^ x.shr(L0_3.into())
  }

  // lower case sigma function 1
  fn lc_sigma_1(x: U) -> U {
    x.rotate_right(L1_1.into()) ^ x.rotate_right(L1_2.into()) ^ x.shr(L1_3.into())
  }

  // upper case sigma function 0
  fn uc_sigma_0(x: U) -> U {
    x.rotate_right(U0_1.into()) ^ x.rotate_right(U0_2.into()) ^ x.rotate_right(U0_3.into())
  }

  // upper case sigma function 1
  fn uc_sigma_1(x: U) -> U {
    x.rotate_right(U1_1.into()) ^ x.rotate_right(U1_2.into()) ^ x.rotate_right(U1_3.into())
  }

  fn ch(x: U, y: U, z: U) -> U {
    (x & y) ^ (!x & z)
  }

  fn maj(x: U, y: U, z: U) -> U {
    (x & y) ^ (x & z) ^ (y & z)
  }

  fn get_K() -> [U; MSG_SCHE_LEN];

  fn get_initial_hash_value() -> HashValue<U>;

  // using the same parameter names as the spec
  // m = Block, w = Message Schedule
  // using wrapping_add to perform addition in modulo 2^[U bit size]
  fn prepare_message_schedules(&self, block: &Block<'a>) -> [U; MSG_SCHE_LEN] {
    let mut W: Vec<U> = vec![];
    for t in 0..16 {
      W.push(block.message_schedule(t));
    }
    for t in 16..MSG_SCHE_LEN {
      let x= Self::lc_sigma_1(W[t-2])
        .wrapping_add(&W[t-7])
        .wrapping_add(&Self::lc_sigma_0(W[t-15]))
        .wrapping_add(&W[t-16]);
      W.push(x);
    }
    W.try_into().unwrap()
  }

  // using wrapping_add to perform addition in modulo 2^[U bit size]
  fn compute_hash(&self, blocks: &Vec<Block<'a>>) -> HashValue<U> {
    let mut tmp = [U::default(); 8];
    let mut hash_value = Self::get_initial_hash_value();
    let K = Self::get_K();
    for block in blocks {
      let mut a: U = hash_value.h[0]; 
      let mut b: U = hash_value.h[1]; 
      let mut c: U = hash_value.h[2]; 
      let mut d: U = hash_value.h[3]; 
      let mut e: U = hash_value.h[4]; 
      let mut f: U = hash_value.h[5]; 
      let mut g: U = hash_value.h[6]; 
      let mut h: U = hash_value.h[7]; 

      let W = self.prepare_message_schedules(block);

      for t in 0..MSG_SCHE_LEN {
        let t1 = h.wrapping_add(&Self::uc_sigma_1(e))
          .wrapping_add(&Self::ch(e, f, g))
          .wrapping_add(&K[t])
          .wrapping_add(&W[t]);
        let t2 = Self::uc_sigma_0(a).wrapping_add(&Self::maj(a, b, c));
        h = g;
        g = f;
        f = e;
        e = d.wrapping_add(&t1);
        d = c;
        c = b;
        b = a;
        a = t1.wrapping_add(&t2);
      }
      
      tmp[0] = a.wrapping_add(&hash_value.h[0]);
      tmp[1] = b.wrapping_add(&hash_value.h[1]);
      tmp[2] = c.wrapping_add(&hash_value.h[2]);
      tmp[3] = d.wrapping_add(&hash_value.h[3]);
      tmp[4] = e.wrapping_add(&hash_value.h[4]);
      tmp[5] = f.wrapping_add(&hash_value.h[5]);
      tmp[6] = g.wrapping_add(&hash_value.h[6]);
      tmp[7] = h.wrapping_add(&hash_value.h[7]);
      hash_value = HashValue { h: tmp };
    }
    hash_value
  }

  // Append the bit 1 to the end of the message, followed by k zero bits, 
  // where k is the smallest, non-negative solution to the equation:
  // l + 1 + k = (PADDED_MSG_SIZE_BITS - LENGTH_PART_LEN_BITS) mod PADDED_MSG_SIZE_BITS 
  // i.e. l + 1 + k = 448 mod 512 for sha256 or l + 1 + k = 871 mod 1024 for sha512
  // resulting msg will have a length that is a multiple of 512 bits
  fn pad_msg(&self, msg: &[u8]) -> Vec<u8> {
    let last_block_max_v_len = BLOCK_SIZE - LENGTH_PART_LEN; 
    let mut v = msg.to_vec();

    // add bit-1 at the end of msg
    v.push(0b1000_0000u8);

    let last_block_v_len = v.len() % BLOCK_SIZE; 

    // if the last block has room to add length part after msg
    if last_block_v_len <= last_block_max_v_len {
      let k = last_block_max_v_len - last_block_v_len;
      v.extend(vec![0u8; k]);

    } else { // otherwise another block needs to be created to store length part 
      // # of bytes remaining in the current last block after msg
      let rest = BLOCK_SIZE - last_block_v_len;
      // fill the current last block w/ rest, and create another block consisting of 0s and length part
      let k = rest + last_block_max_v_len;
      v.extend(vec![0u8; k]);
    }
    // append length part to the end 
    let msg_len: usize = (msg.len() * 8).try_into().unwrap();
    let msg_len_be = msg_len.to_be_bytes();

    // write msg_len to length part
    let mut length_part = [0u8; LENGTH_PART_LEN];
    length_part[LENGTH_PART_LEN - msg_len_be.len()..].copy_from_slice(&msg_len_be);

    v.extend_from_slice(&length_part);
    v
  }
}
