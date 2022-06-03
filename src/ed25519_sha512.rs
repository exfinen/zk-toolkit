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
  fn new() -> Self {
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

  fn prune_prv_key_buf(buf: &mut [u8; 32]) {
    buf[31] &= 0b0111_1111;  // clear most significant bit
    buf[31] |= 0b0100_0000;  // set 2nd most significant bit
    buf[0] &= 0b1111_1000;  // clear least significant 3 bits
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

  pub fn gen_key_pair(&self, k: &[u8; 32]) -> KeyPair {
    let H = Sha512();
    let digest = H.get_digest(k);

    // private key is the last 32 bytes of the digest pruned
    let mut prv_key = [0u8; 32];
    prv_key[..].copy_from_slice(&digest[0..32]);
    Self::prune_prv_key_buf(&mut prv_key);
    let prv_key_biguint = BigUint::from_bytes_le(&prv_key);

    // base point is (+x, 4/5)
    let bp_y = self.f.elem(&4u8) / &5u8;
    let bp_x = self.recover_x(&bp_y, Parity::Even);  // get positive x
    let base_point = EcPoint::new(&bp_x, &bp_y);
    
    // multiply a w/ base point to get A
    let pub_key_pt = self.scalar_mul(&base_point, &prv_key_biguint);
    let pub_key = self.encode_point(&pub_key_pt);

    KeyPair {
      prv_key,
      pub_key,
    }
  }

  pub fn sign(_msg: &[u8], _priv_key: u32) -> [u8; 32] {
    [0u8; 32]
  }

  pub fn verify(_msg: &[u8], _pub_key: u32, _sig: [u8;32]) -> bool {
    true
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test1() {
    let ed25519 = Ed25519Sha512::new();

    let secret = [0u8; 32];
    let keypair = ed25519.gen_key_pair(&secret);

    let exp_pub_key = hex::decode("3b6a27bcceb6a42d62a3a8d02a6f0d73653215771de243a63ac048a18b59da29").unwrap();
    assert_eq!(exp_pub_key, keypair.pub_key);
  }
}