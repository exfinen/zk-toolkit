use crate::curve::{Curve, AddOps};
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
  pub ops: &'a dyn AddOps,
  pub f_n: Rc<Field>,
}

#[derive(Debug, Clone)]
pub struct Signature {
  pub r: FieldElem,   // mod n
  pub s: FieldElem,   // mod n
}

impl<'a> Ecdsa<'a> {
  pub fn new(curve: &'a dyn Curve, ops: &'a dyn AddOps) -> Self {
    let f_n = Field::new(curve.n());
    Ecdsa { curve, ops, f_n }
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
      // z = e's uppermost Ln bits (Ln = order of n = 256 bits)
      let z = BigUint::from_bytes_be(&Hash::sha256(message));

      // p = kG (k != 0)
      let p: EcPoint = self.ops.scalar_mul(&self.curve.g(), &k.v);

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
    else if !self.ops.scalar_mul(pub_key, &n).is_inf { 
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
      // z = e's uppermost Ln bits (Ln = order of n = 256 bits)
      let z = BigUint::from_bytes_be(&Hash::sha256(message));
      let z_fe = FieldElem::new(self.f_n.clone(), BigUint::from(z));  // mod n
      let w = sig.s.inv().unwrap();  // mod n
      let u1 = z_fe.mul(&w);  // mod n
      let u2 = sig.r.mul(&w);  // mod n

      // (x, y) = u1 * G + u2 * PubKey
      let p1 = self.ops.scalar_mul(&self.curve.g(), &u1.v);
      let p2 = self.ops.scalar_mul(&pub_key, &u2.v);
      let p3 = self.ops.add(&p1, &p2);
      sig.r.v == (p3.x.v % &n)
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::weierstrass_eq::WeierstrassEq;
  use crate::weierstrass_add_ops::JacobianAddOps;

  #[test]
  // TODO create separate tests for not-on-curve and pub_key-not-order-n cases
  fn test_sign_verify_bad_pub_key() {
    let weier = WeierstrassEq::secp256k1();
    let ops = JacobianAddOps::new();
    let mut ecdsa = Ecdsa::new(&weier, &ops);

    let message = vec![1u8, 2, 3];

    // sign with newly generated private key
    let priv_key = ecdsa.gen_random_number_order_n();
    let sig = ecdsa.sign(&priv_key, &message).unwrap();

    let good_pub_key = ops.scalar_mul(&weier.g(), &priv_key.v);
    let bad_pub_key = EcPoint {
      x: good_pub_key.x.clone(),
      y: good_pub_key.x.clone(),
      is_inf: false,
    };
    let is_verified = ecdsa.verify(&sig, &bad_pub_key, &message);
    assert_eq!(is_verified, false);
  }

  #[test]
  fn test_sign_verify_inf_pub_key() {
    let weier = WeierstrassEq::secp256k1();
    let ops = JacobianAddOps::new();
    let mut ecdsa = Ecdsa::new(&weier, &ops);

    let message = vec![1u8, 2, 3];

    // sign with newly generated private key
    let priv_key = ecdsa.gen_random_number_order_n();
    let sig = ecdsa.sign(&priv_key, &message).unwrap();

    // use inf public key for verifying
    let pub_key = EcPoint::inf();
    let is_verified = ecdsa.verify(&sig, &pub_key, &message);
    assert_eq!(is_verified, false);
  }

  #[test]
  fn test_sign_verify_sig_r_out_of_range() {
    let weier = WeierstrassEq::secp256k1();
    let ops = JacobianAddOps::new();
    let mut ecdsa = Ecdsa::new(&weier, &ops);

    let message = vec![1u8, 2, 3];

    // sign with newly generated private key
    let priv_key = ecdsa.gen_random_number_order_n();

    // create public key from the private key used for signing for verifying
    let pub_key = ops.scalar_mul(&weier.g(), &priv_key.v);

    let sig = ecdsa.sign(&priv_key, &message).unwrap();

    let sig_r_too_large = Signature {
      r: sig.clone().s.new_elem(weier.n()),
      s: sig.clone().s,
    };
    let is_verified = ecdsa.verify(&sig_r_too_large, &pub_key, &message);
    assert_eq!(is_verified, false);

    let sig_r_too_small = Signature {
      r: sig.r.new_elem(BigUint::zero()),
      s: sig.s,
    };
    let is_verified = ecdsa.verify(&sig_r_too_small, &pub_key, &message);
    assert_eq!(is_verified, false);
  }

  #[test]
  fn test_sign_verify_sig_s_out_of_range() {
    let weier = WeierstrassEq::secp256k1();
    let ops = JacobianAddOps::new();
    let mut ecdsa = Ecdsa::new(&weier, &ops);

    let message = vec![1u8, 2, 3];

    // sign with newly generated private key
    let priv_key = ecdsa.gen_random_number_order_n();

    // create public key from the private key used for signing for verifying
    let pub_key = ops.scalar_mul(&weier.g(), &priv_key.v);

    let sig = ecdsa.sign(&priv_key, &message).unwrap();

    let sig_s_too_large = Signature {
      r: sig.clone().r,
      s: sig.clone().s.new_elem(weier.n()),
    };
    let is_verified = ecdsa.verify(&sig_s_too_large, &pub_key, &message);
    assert_eq!(is_verified, false);

    let sig_s_too_small = Signature {
      r: sig.r,
      s: sig.s.new_elem(BigUint::zero()),
    };
    let is_verified = ecdsa.verify(&sig_s_too_small, &pub_key, &message);
    assert_eq!(is_verified, false);
  }

  #[test]
  fn test_sign_verify_all_good() {
    let weier = WeierstrassEq::secp256k1();
    let ops = JacobianAddOps::new();
    let mut ecdsa = Ecdsa::new(&weier, &ops);

    let message = vec![1u8, 2, 3];

    // sign with newly generated private key
    let priv_key = ecdsa.gen_random_number_order_n();
    let sig = ecdsa.sign(&priv_key, &message).unwrap();

    // create public key from the private key used for signing for verifying
    let pub_key = ops.scalar_mul(&weier.g(), &priv_key.v);
    let is_verified = ecdsa.verify(&sig, &pub_key, &message);
    assert_eq!(is_verified, true);
  }

  #[test]
  fn test_sign_verify_bad_priv_key() {
    let weier = WeierstrassEq::secp256k1();
    let ops = JacobianAddOps::new();
    let mut ecdsa = Ecdsa::new(&weier, &ops);

    let message = vec![1u8, 2, 3];

    // sign with newly generated private key
    let priv_key = ecdsa.gen_random_number_order_n();
    let sig = ecdsa.sign(&priv_key, &message).unwrap();

    // change private key and create public key from it
    let priv_key = ecdsa.gen_random_number_order_n();
    let pub_key = ops.scalar_mul(&weier.g(), &priv_key.v);

    let is_verified = ecdsa.verify(&sig, &pub_key, &message);
    assert_eq!(is_verified, false);
  }

  #[test]
  fn test_sign_verify_different_message() {
    let weier = WeierstrassEq::secp256k1();
    let ops = JacobianAddOps::new();
    let mut ecdsa = Ecdsa::new(&weier, &ops);

    let message = vec![1u8, 2, 3];

    // sign with newly generated private key
    let priv_key = ecdsa.gen_random_number_order_n();
    let sig = ecdsa.sign(&priv_key, &message).unwrap();

    // create public key from the private key used for signing for verifying
    let pub_key = ops.scalar_mul(&weier.g(), &priv_key.v);

    // change message and verify
    let message = vec![1u8, 2, 3, 4];
    let is_verified = ecdsa.verify(&sig, &pub_key, &message);
    assert_eq!(is_verified, false);
  }
}