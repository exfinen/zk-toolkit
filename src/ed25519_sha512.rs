#![allow(non_snake_case)]
use crate::hasher::Hasher;
use crate::sha512::Sha512;
use crate::field::{Field, FieldElem};
use num_bigint::BigUint;
use core::ops::Sub;
use num_traits::Zero;

// implementation based on https://ed25519.cr.yp.to/ed25519-20110926.pdf

#[derive(Debug)]
pub struct KeyPair {
  pub prv_key: [u8; 32],
  pub pub_key: [u8; 32],
}

#[derive(PartialEq)]
enum Parity {
  Even,
  Odd,
}

fn get_parity(e: &FieldElem) -> Parity {
  if (&e.n % 2u8).is_zero() { Parity::Even } else { Parity::Odd }
}

fn recover_x(y: &FieldElem, x_parity: Parity) -> FieldElem {
  let f = &y.f;

  // d = -121665 / 121666
  let d = -f.elem(&121665u32) / &121666u32;
  
  // xx = x^2 = (y^2 - 1) / (1 + d*y^2)
  let xx = (&y.sq() - &1u8) / &(&(&d * &y.sq()) + &1u8);

  // calculate the square root of xx assuming a^((q-1)/4) = 1 mod q
  let mut x = (&xx).pow(&((&*f.order + &3u8) / &8u8));

  // if that that's match, calculate the square root of xx again 
  // assuming a^((q-1)/4) = -1 mod q
  if &x.sq().n != &xx.n {
    let I = y.f.elem(&2u8).pow(&((&*f.order - &1u8) / &4u8));
    x = &x * &I;
  }
  let root_parity = get_parity(&x);
  if root_parity != x_parity {
    x = -&x;
  }
  x
}

// secret key is 32-byte string
pub fn gen_priv_key(k: &[u8; 32]) -> KeyPair {
  let H = Sha512();
  let digest = H.get_digest(k);

  // private key is the last 32 bytes of the digest
  let mut prv_key = [0u8; 32];
  prv_key[..].copy_from_slice(&digest[32..64]);

  println!("Priv key: {}", hex::encode(prv_key));
  // a is 32-byte string based on the first 32 bytes of the digest w/
  // - its most significant bit cleared
  // - its 2nd most significant bit set
  // - its least significant 3 bits cleared
  let mut a = [0u8; 32];
  a[..].copy_from_slice(&digest[0..32]);
  a[0] &= 0b0111_1111;  // clear most significant bit
  a[0] |= 0b0100_0000;  // set 2nd most significant bit
  a[31] &= 0b1111_1000;  // clear least significant 3 bits

  // order of base field is 2^255 - 19
  let q = BigUint::from(2u8).pow(255u32).sub(19u8);
  let F_q = Field::new(&q);

  // base point is (+x, 4/5)
  let bp_y = F_q.elem(&4u8) / &5u8;
  let bp_x = recover_x(&bp_y, Parity::Even); // get positive x
  
  println!("Base point x={:?}", bp_x.n);
  println!("Base point y={:?}", bp_y.n);

  // multiply a w/ base point to get A
  // Edwards Addition Law
  // (x1,y1) + (x2,y2) = ((x1y2 + x2y1) / (1 + d x1x2 y1y2), (y1y2 + x1x2) / (1 - d x1x2 y1y2))
  
  KeyPair {
    prv_key,
    pub_key: [0u8; 32],
  }
}

pub fn gen_pub_key(_priv_key: [u8; 32]) -> u32 {
  0u32
}

pub fn sign(_msg: &[u8], _priv_key: u32) -> [u8; 32] {
  [0u8; 32]
}

pub fn verify(_msg: &[u8], _pub_key: u32, _sig: [u8;32]) -> bool {
  true
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test1() {
    let secret = [0u8; 32];
    gen_priv_key(&secret);
  }
}