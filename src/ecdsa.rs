use crate::curve::Curve;
use crate::ec_point::EcPoint;
use crate::field_elem::FieldElem;
use crate::field::Field;
use crate::random_number::RandomNumber;
use num_bigint::{BigUint};
use rand::RngCore;
use std::rc::Rc;
use num_traits::identities::Zero;
use crate::hash::Hash;

pub struct Ecdsa<'a> {
  pub curve: &'a dyn Curve,
  pub f_n: Rc<Field>,
}

pub struct Signature {
  pub r: FieldElem,   // mod n
  pub s: FieldElem,   // mod n
}

impl<'a> Ecdsa<'a> {
  pub fn new(curve: &'a dyn Curve) -> Self {
    let f_n = Field::new(curve.n());
    Ecdsa { curve, f_n }
  }

  // generate random number from range [1, n-1]
  pub fn gen_random_number_order_n(&self) -> FieldElem {
    loop {
      let mut buf = [0u8; 32];
      let mut rand = RandomNumber::new();
      rand.gen.fill_bytes(&mut buf);
      let x = FieldElem::new(self.f_n.clone(), BigUint::from_bytes_be(&buf));
      if x.v != BigUint::zero() { 
        return x;
      }
    }
  }

  pub fn sign(&mut self, priv_key: &FieldElem, message: &[u8]) -> Result<Signature, String> {
    // n is 32-byte long in secp256k1
    // dA = private key in [1, n-1]
    let n = self.curve.n();

    loop {
      // generate temporary non-zero random number k (mod n) 
      let k = self.gen_random_number_order_n();  // mod n
      
      // e = HASH(message)
      // z = e's leftmost Ln bits (Ln = order of n = 256 bits)
      let z = BigUint::from_bytes_be(&Hash::sha256(message));

      // p = kG (k != 0)
      let p: EcPoint = self.curve.scalar_mul(&self.curve.g(), &k.v);

      // r = p.x mod n
      let r = p.x.v % &n;

      // if r is 0, k is bad. repeat the process from the beggining
      if r == BigUint::zero() {
        continue;
      }
      // s = k^-1(z + r * dA) mod n // if s == 0, generate k again
      let k_inv = k.inv().unwrap();  // mod n
      let r_fe = k.new_elem(r);  // mod n
      let z_fe = k.new_elem(z);  // mod n
      let s = k_inv.mul(&(priv_key.mul(&r_fe).add(&z_fe)));  // mod n
      // if s is 0, k is bad. repear the process from the beginning
      if s.v == BigUint::zero() {
        continue;
      }

      return Ok(Signature { r: r_fe, s });
    }
  }

  // pub key is modulo p. not n which is the order of g
  pub fn verify(&self, sig: &Signature, pub_key: &EcPoint, message: &[u8]) -> bool {
    let n = self.curve.n();

    // confirm pub_key is not inf
    if pub_key.is_inf {
      false
    }
    // confirm pub_key is on the curve
    else if !self.curve.is_on_curve(&pub_key) {
      false
    }
    // confirm n * pub_key is inf
    else if !self.curve.scalar_mul(pub_key, &n).is_inf { 
      false
    }
    // check if r and s are in [1, n-1]
    else if 
      sig.r.v == BigUint::zero()
      || sig.s.v == BigUint::zero()
      || n <= sig.r.v
      || n <= sig.s.v {
      false
    } 
    else {
      // compute e = HASH(m)
      // z = e's leftmost Ln bits (Ln = order of n = 256 bits)
      let z = BigUint::from_bytes_be(&Hash::sha256(message));
      let z_fe = FieldElem::new(self.f_n.clone(), BigUint::from(z));  // mod n
      let w = sig.s.inv().unwrap();  // mod n
      let u1 = z_fe.mul(&w);  // mod n
      let u2 = sig.r.mul(&w);  // mod n

      // (x, y) = u1 * G + u2 * PubKey
      let p1 = self.curve.scalar_mul(&self.curve.g(), &u1.v);
      let p2 = self.curve.scalar_mul(&pub_key, &u2.v);
      let p3 = self.curve.add(&p1, &p2);
      sig.r.v == (p3.x.v % &n)
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::weierstrass_eq::WeierstrassEq;

  #[test]
  fn test_1() {
    let curve = WeierstrassEq::secp256k1();
    let ecdsa = Ecdsa::new(&curve);

    // create private key
    let priv_key = ecdsa.gen_random_number_order_n();
    let pub_key = curve.scalar_mul(&curve.g(), &priv_key.v);

    println!("priv={:?}", priv_key);
    println!("pub={:?}", pub_key);
  }
}