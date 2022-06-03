#![allow(non_snake_case)]
use crate::hasher::Hasher;
use crate::sha512::Sha512;
use crate::field::{Field, FieldElem};
use crate::ec_point::EcPoint;
use crate::elliptic_curve::AddOps;
use num_bigint::BigUint;
use core::ops::Sub;
use num_traits::{Zero};

// implementation based on:
// - https://ed25519.cr.yp.to/ed25519-20110926.pdf
// - https://datatracker.ietf.org/doc/html/rfc8032

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

pub struct Ed25519Sha512 {
  q: BigUint,
  f: Field,
  d: FieldElem,
  one: FieldElem,
  zero: FieldElem,
}

impl AddOps for Ed25519Sha512 {
  fn get_zero_point(&self) -> EcPoint {
      EcPoint::new(&self.zero, &self.one)
  } 
  // Edwards Addition Law
  // (x1,y1) + (x2,y2) = ((x1y2 + x2y1) / (1 + d x1x2 y1y2), (y1y2 + x1x2) / (1 - d x1x2 y1y2))
  fn add(&self, p1: &EcPoint, p2: &EcPoint) -> EcPoint {
    let x1y2 = &p1.x * &p2.y;
    let x2y1 = &p2.x * &p1.y;
    let x1x2y1y2 = &x1y2 * &x2y1;
    let y1y2 = &p1.y * &p2.y;
    let x1x2 = &p1.x * &p2.x;
    let x = &(&x1y2 + &x2y1) / &(self.f.elem(&1u8) + &(&self.d * &x1x2y1y2));
    let y = (&y1y2 + &x1x2) / &(self.f.elem(&1u8) - &(&self.d * &x1x2y1y2));
    EcPoint::new(&x, &y)
  }
}


impl Ed25519Sha512 {
  pub fn new() -> Self {
    // order of base field is 2^255 - 19
    let q = BigUint::from(2u8).pow(255u32).sub(19u8);
    let f = Field::new(&q);

    // d = -121665 / 121666
    let d = -f.elem(&121665u32) / &121666u32;

    let one = f.elem(&1u8);
    let zero = f.elem(&0u8);

    Ed25519Sha512 { q, f, d, one, zero }
  }

  fn get_parity(e: &FieldElem) -> Parity {
    if (&e.n % 2u8).is_zero() { Parity::Even } else { Parity::Odd }
  }

  fn recover_x(&self, y: &FieldElem, x_parity: Parity) -> FieldElem {
    // xx = x^2 = (y^2 - 1) / (1 + d*y^2)
    let xx = (&y.sq() - &1u8) / &(&(&self.d * &y.sq()) + &1u8);

    // calculate the square root of xx assuming a^((q-1)/4) = 1 mod q
    let mut x = (&xx).pow(&((&self.q + &3u8) / &8u8));

    // if that that's match, calculate the square root of xx again 
    // assuming a^((q-1)/4) = -1 mod q
    if &x.sq().n != &xx.n {
      let I = self.f.elem(&2u8).pow(&((&self.q - &1u8) / &4u8));
      x = &x * &I;
    }
    let root_parity = Self::get_parity(&x);
    if root_parity != x_parity {
      x = -&x;
    }
    x
  }

  fn encode_point(&self, pt: &EcPoint) -> [u8; 32] {
    // get parity of x
    let x_parity = if (&pt.x.n & &self.one.n) == self.zero.n { Parity::Even } else { Parity::Odd };

    // interpret y as little-endian integer and write to 32-byte buffer
    let y_bytes_le = pt.y.n.to_bytes_le();
    assert!(y_bytes_le.len() <= 32);
    let mut buf = [0u8; 32];
    buf[32 - y_bytes_le.len()..].copy_from_slice(&y_bytes_le);

    // the most significant bit of the last octet (=parity bit) should be 0
    assert_eq!(buf[31] & 0b1000_0000, 0);

    // set the parity bit if parity is odd
    if x_parity == Parity::Odd {
      buf[31] |= 0b1000_0000;
    }
    buf
  }

  fn decode_point(&self, pt_buf: &[u8; 32]) -> EcPoint {
    let mut pt_buf = pt_buf.clone();

    // get parity of x
    let x_parity = if pt_buf[31] & 0b1000_0000 == 0 { Parity::Even } else { Parity::Odd };

    // clear parity bit
    pt_buf[31] &= 0b0111_1111;

    let y = self.f.elem(&BigUint::from_bytes_le(&pt_buf));
    let x = self.recover_x(&y, x_parity);

    EcPoint::new(&x, &y)
  }

  fn prune_32_byte_buf(buf: &mut [u8; 32]) {
    buf[31] &= 0b0111_1111;  // clear most significant bit
    buf[31] |= 0b0100_0000;  // set 2nd most significant bit
    buf[0] &= 0b1111_1000;  // clear least significant 3 bits
  }

  pub fn gen_pub_key(&self, prv_key: &[u8; 32]) -> [u8; 32] {
    let H = Sha512();
    let digest = H.get_digest(prv_key);

    // private key is the last 32 bytes of the digest pruned
    let mut buf = [0u8; 32];
    buf[..].copy_from_slice(&digest[0..32]);
    Self::prune_32_byte_buf(&mut buf);
    let s = BigUint::from_bytes_le(&buf);

    // base point is (+x, 4/5)
    let bp_y = self.f.elem(&4u8) / &5u8;
    let bp_x = self.recover_x(&bp_y, Parity::Even);  // get positive x
    let base_point = EcPoint::new(&bp_x, &bp_y);
    
    // multiply a w/ base point to get A
    let pub_key_pt = self.scalar_mul(&base_point, &s);
    let pub_key = self.encode_point(&pub_key_pt);
    pub_key
  }

  pub fn sign(&self, msg: &[u8], prv_key: &[u8; 32]) -> [u8; 32] {
    [0u8; 32]
  }

  pub fn verify(&self, _msg: &[u8], _pub_key: u32, _sig: [u8;32]) -> bool {
    true
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn run_rfc8032_test(prv_key: &str, exp_pub_key: &str, msg: &[u8], exp_sig: &str) {
    let ed25519 = Ed25519Sha512::new();

    let prv_key: [u8; 32] = hex::decode(prv_key).unwrap().try_into().unwrap();
    let pub_key = ed25519.gen_pub_key(&prv_key);

    let exp_pub_key = hex::decode(exp_pub_key).unwrap();
    assert_eq!(exp_pub_key, pub_key);

    let sig = ed25519.sign(&msg, &prv_key);
    let exp_sig = hex::decode(exp_sig).unwrap();
    assert_eq!(sig, exp_sig);
  }

  #[test]
  fn rfc8032_test1() {
    let secret = "9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60";
    let exp_pub_key = "d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a";
    let msg = [72u8];
    let exp_sig = "92a009a9f0d4cab8720e820b5f642540a2b27b5416503f8fb3762223ebdb69da085ac1e43e15996e458f3613d0f11d8c387b2eaeb4302aeeb00d291612bb0c00";
    run_rfc8032_test(secret, exp_pub_key, &msg, exp_sig);
  }
}