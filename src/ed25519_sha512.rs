#![allow(non_snake_case)]
use crate::hasher::Hasher;
use crate::sha512::Sha512;
use crate::field::Field;
use crate::field_elem::FieldElem;
use num_bigint::BigUint;
use core::ops::Sub;

// based on https://ed25519.cr.yp.to/ed25519-20110926.pdf

pub struct KeyPair {
  pub prv_key: [u8; 32],
  pub pub_key: [u8; 32],
}

// secret key is 32-byte string
pub fn gen_priv_key(k: [u8; 32]) -> KeyPair {
  let H = Sha512();
  let digest = H.get_digest(&k);

  // private key is the last 32 bytes of the digest
  let mut prv_key = [0u8; 32];
  prv_key[..].copy_from_slice(&digest[32..64]);

  // a is 32-byte string based on the first 32 bytes of the digest w/
  // - its most significant bit cleared
  // - its 2nd most significant bit set
  // - its least significant 3 bits cleared
  let mut a = [0u8; 32];
  a[..].copy_from_slice(&digest[0..32]);
  a[0] &= 0b0111_1111;  // clear most significant bit
  a[0] |= 0b0100_0000;  // set 2nd most significant bit
  a[31] &= 0b1111_1000;  // clear least significant 3 bits

  // q = 2^255 - 19
  let q = BigUint::from(2u8).nth_root(255u32).sub(19u8);
  let F_q = Field::new(q);

  // base point is (x, 4/5) w/ positive x
  let inv5 = FieldElem::new(F_q, BigUint::from(5u8)).inv().unwrap();
  let bp_y = inv5.mul_u32(4u32);

  let n_121666 = FieldElem::new(F_q, BigUint::from(121666));
  let n_minus_121665 = FieldElem::new(F_q, BigUint::from(121665)).neg();
  let d = n_minus_121665.div(n_121666);
  
  // xx = x^2 = (y^2 - 1) / (1 + d*y^2)
  let one = FieldElem::new(F_q, BigUint::from(1u8));
  let xx = bp_y.mul(&bp_y).sub(&one).div(one.add(&d.mul(&bp_y).mul(&bp_y)));

  let two = FieldElem::new(F_q, BigUint::from(2u8));
  let four = FieldElem::new(F_q, BigUint::from(4u8));

  let q_minus_1_over_4 = q.sub(one).div(four);
  let I = two.pow(q_minus_1_over_4);

  let three = FieldElem::new(F_q, BigUint::from(3u8));
  let eight = FieldElem::new(F_q, BigUint::from(8u8));
  let q_plus_3_over_8 = (q.add(three)).div(eight);
  let mut x = xx.pot(q_plus_3_over_8);
  if x.mul(&x).sub(&xx) != 0 { // if x is not the solution, multiply I
    x = x.mul(&I);
  }
  // x should be positive
  // if least significant bit of x is 1, convert it to positive by
  // x = q - x

  // multiply a w/ base point to get A
  // 255-bit encoding of F_q^255-19 is little encoding of {0,1,..., 2^255-20}
  // Edwards Addition Law
  // (x1,y1) + (x2,y2) = ((x1y2 + x2y1) / (1 + d x1x2 y1y2), (y1y2 + x1x2) / (1 - d x1x2 y1y2))
  /*
    y co-ordinate value:  b'5866666666666666666666666666666666666666666666666666666666666666'
    n=1 [0100000000000000000000000000000000000000000000000000000000000000]
    1.G -> Point (y): 5866666666666666666666666666666666666666666666666666666666666666 (46316835694926478169428394003475163141307993866256225615783033603165251855960)
    x point: 15112221349535400772501151409588531511454012693041857206046113283949847762202
    y point: 46316835694926478169428394003475163141307993866256225615783033603165251855960
  */
  
  // encode A

  KeyPair {
    prv_key,
    pub_key: [0u8; 32],
  }
}

pub fn gen_pub_key(priv_key: [u8; 32]) -> u32 {
  0u32
}

pub fn sign(msg: &[u8], priv_key: u32) -> [u8; 32] {
  [0u8; 32]
}

pub fn verify(msg: &[u8], pub_key: u32, sig: [u8;32]) -> bool {
  true
}