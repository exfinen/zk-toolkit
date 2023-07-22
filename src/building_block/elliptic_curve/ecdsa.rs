use crate::building_block::{
  elliptic_curve::{
    curve::Curve,
    curve_equation::CurveEquation,
    elliptic_curve_point_ops::EllipticCurvePointOps,
    ec_point::EcPoint,
    weierstrass::adder::point_adder::PointAdder,
  },
  field::{
    field_elem_ops::Inverse,
    prime_field_elem::PrimeFieldElem,
  },
  hasher::{
    hasher::Hasher,
    sha256::Sha256,
  },
  zero::Zero,
};
use num_bigint::BigUint;
use std::ops::Add;
use num_traits::Zero as NumTraitsZero;

use super::weierstrass::curves::secp256k1::Secp256k1;

pub struct Ecdsa {
  pub curve: Box<Secp256k1>,
  pub hasher: Box<Sha256>,
}

#[derive(Debug, Clone)]
pub struct Signature {
  pub r: PrimeFieldElem,
  pub s: PrimeFieldElem,
}

impl Ecdsa {
  pub fn new(curve: Box<Secp256k1>, hasher: Box<Sha256>) -> Self {
    Ecdsa { curve, hasher }
  }

  pub fn gen_pub_key(&self, priv_key: &PrimeFieldElem) -> EcPoint {
    let g = &self.curve.g.unwrap();
    self.curve.scalar_mul(g, &priv_key)
  }

  pub fn sign(&self, priv_key: &PrimeFieldElem, message: &[u8]) -> Result<Signature, String> {
    // n is 32-byte long in secp256k1
    // dA = private key in [1, n-1]
    let f_n = &self.curve.f_n;
    let n = &f_n.order;
    let g = &self.curve.g.unwrap();
    let sha256 = Sha256();

    loop {
      // generate temporary non-zero random number k (mod n)
      let k = f_n.rand_elem(true);

      // e = HASH(message)
      // z = e's uppermost Ln bits (Ln = order of n = 256 bits)
      let z = BigUint::from_bytes_be(&sha256.get_digest(message));

      // p = kG (k != 0)
      let p: EcPoint = self.curve.scalar_mul(g, &k);

      // r = p.x mod n
      let r = p.x.e % n;

      // if r is 0, k is bad. repeat the process from the beggining
      if r == BigUint::zero() {
        continue;
      }
      // s = k^-1(z + r * dA) mod n // if s == 0, generate k again
      let k_inv = k.inv();  // mod n
      let r_fe = f_n.elem(&r);  // mod n
      let z_fe = f_n.elem(&z);  // mod n
      let s = k_inv * (priv_key * &r_fe + z_fe);  // mod n
      // if s is 0, k is bad. repear the process from the beginning
      if s.e == BigUint::zero() {
        continue;
      }

      return Ok(Signature { r: r_fe, s });
    }
  }

  // pub key is modulo p. not n which is the order of g
  pub fn verify(&self, sig: &Signature, pub_key: &EcPoint, message: &[u8]) -> bool {
    let f_n = &self.curve.f_n;
    let n = &f_n.order;
    let g = &self.curve.g;

    // confirm pub_key is not inf
    if pub_key.is_zero() {
      false
    }
    // confirm pub_key is on the curve
    else if !self.curve.eq().is_rational_point(&pub_key) {
      false
    }
    // confirm n * pub_key is inf
    else if !self.curve.scalar_mul(pub_key, n).is_inf {
      false
    }
    // check if r and s are in [1, n-1]
    else if
      *&sig.r.is_zero()
      || *&sig.s.is_zero()
      || n <= &sig.r.e
      || n <= &sig.s.e {
      false
    }
    else {
      // compute e = HASH(m)
      // z = e's uppermost Ln bits (Ln = order of n = 256 bits)
      let z = BigUint::from_bytes_be(&self.hasher.get_digest(message));
      let z_fe = f_n.elem(&z);  // mod n
      let w = sig.s.inv();  // mod n
      let u1 = z_fe * &w;  // mod n
      let u2 = &sig.r * w;  // mod n

      // (x, y) = u1 * G + u2 * PubKey
      let p1 = self.curve.scalar_mul(&g, &u1.n);
      let p2 = self.curve.scalar_mul(&pub_key, &u2.n);
      let p3 = self.curve.add(&p1, &p2);
      sig.r.n == (p3.x.n % n)
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::building_block::{
    elliptic_curve::{
      elliptic_curve_point_ops::EllipticCurvePointOps,
      weierstrass::{
        adder::affine_point_adder::AffinePointAdder,
        curves::secp256k1::Secp256k1,
      },
    },
    field::{
      prime_field::PrimeField,
      prime_field_elem::PrimeFieldElem,
    },
  };

  #[derive(Clone)]
  struct AffineOps();

  impl EllipticCurvePointOps<EcPoint, PrimeFieldElem, PrimeField, Secp256k1> for AffineOps {
    type Adder = AffinePointAdder;
  }

  #[test]
  // TODO create separate tests for not-on-curve and pub_key-not-order-n cases
  fn sign_verify_bad_pub_key() {
    let curve = Box::new(Secp256k1::new());
    let hasher = Box::new(Sha256());
    let ecdsa = Ecdsa::new(curve.clone(), hasher);

    let curve_group = &curve.f;
    let g = &ecdsa.curve.g;
    let message = vec![1u8, 2, 3];

    // sign with newly generated private key
    let priv_key = curve_group.rand_elem(true);
    let sig = ecdsa.sign(&priv_key, &message).unwrap();

    let good_pub_key = curve.scalar_mul(g, &priv_key.n);
    let bad_pub_key = EcPoint {
      curve,
      x: good_pub_key.x.clone(),
      y: good_pub_key.x.clone(),
      is_inf: false,
    };
    let is_verified = ecdsa.verify(&sig, &bad_pub_key, &message);
    assert_eq!(is_verified, false);
  }

  #[test]
  fn sign_verify_inf_pub_key() {
    let curve = Box::new(Secp256k1::new());
    let hasher = Box::new(Sha256());
    let ecdsa = Ecdsa::new(curve.clone(), hasher);

    let message = vec![1u8, 2, 3];

    // sign with newly generated private key
    let priv_key = ecdsa.curve.f_n().rand_elem(true);
    let sig = ecdsa.sign(&priv_key, &message).unwrap();

    // use inf public key for verifying
    let pub_key = EcPoint::get_zero(&curve.g());
    let is_verified = ecdsa.verify(&sig, &pub_key, &message);
    assert_eq!(is_verified, false);
  }

  #[test]
  fn sign_verify_sig_r_out_of_range() {
    let curve = Box::new(Secp256k1::new());
    let hasher = Box::new(Sha256());
    let ecdsa = Ecdsa::new(curve.clone(), hasher);
    let group = &curve.g;

    let message = vec![1u8, 2, 3];

    // sign with newly generated private key
    let priv_key = group.rand_elem(true);

    // create public key from the private key used for signing for verifying
    let pub_key = curve.scalar_mul(&curve.g, &priv_key.n);

    let sig = ecdsa.sign(&priv_key, &message).unwrap();

    let sig_r_too_large = Signature {
      r: sig.clone().s.f.elem(&curve.n),
      s: sig.clone().s,
    };
    let is_verified = ecdsa.verify(&sig_r_too_large, &pub_key, &message);
    assert_eq!(is_verified, false);

    let sig_r_too_small = Signature {
      r: sig.r.f.elem(&BigUint::zero()),
      s: sig.s,
    };
    let is_verified = ecdsa.verify(&sig_r_too_small, &pub_key, &message);
    assert_eq!(is_verified, false);
  }

  #[test]
  fn sign_verify_sig_s_out_of_range() {
    let curve = Box::new(Secp256k1::new());
    let hasher = Box::new(Sha256());
    let ecdsa = Ecdsa::new(curve.clone(), hasher);
    let group = &curve.f;

    let message = vec![1u8, 2, 3];

    // sign with newly generated private key
    let priv_key = group.rand_elem(true);

    // create public key from the private key used for signing for verifying
    let pub_key = curve.scalar_mul(&curve.g, &priv_key.n);

    let sig = ecdsa.sign(&priv_key, &message).unwrap();

    let sig_s_too_large = Signature {
      r: sig.clone().r,
      s: sig.clone().s.f.elem(&curve.n),
    };
    let is_verified = ecdsa.verify(&sig_s_too_large, &pub_key, &message);
    assert_eq!(is_verified, false);

    let sig_s_too_small = Signature {
      r: sig.r,
      s: sig.s.f.elem(&BigUint::zero()),
    };
    let is_verified = ecdsa.verify(&sig_s_too_small, &pub_key, &message);
    assert_eq!(is_verified, false);
  }

  #[test]
  fn sign_verify_all_good() {
    let curve = Box::new(Secp256k1::new());
    let hasher = Box::new(Sha256());
    let ecdsa = Ecdsa::new(curve, hasher);
    let group = &curve.f;

    let message = vec![1u8, 2, 3];

    // sign with newly generated private key
    let priv_key = group.rand_elem(true);
    let sig = ecdsa.sign(&priv_key, &message).unwrap();

    // create public key from the private key used for signing for verifying
    let pub_key = ecdsa.gen_pub_key(&priv_key);
    let is_verified = ecdsa.verify(&sig, &pub_key, &message);
    assert_eq!(is_verified, true);
  }

  #[test]
  fn sign_verify_bad_priv_key() {
    let curve = Box::new(Secp256k1::new());
    let hasher = Box::new(Sha256());
    let ecdsa = Ecdsa::new(curve, hasher);

    let message = vec![1u8, 2, 3];

    // sign with newly generated private key
    let priv_key = curve.f_n.rand_elem(true);
    let sig = ecdsa.sign(&priv_key, &message).unwrap();

    // change private key and create public key from it
    let priv_key = curve.f_n.rand_elem(true);
    let pub_key = curve.scalar_mul(&curve.g, &priv_key.n);

    let is_verified = ecdsa.verify(&sig, &pub_key, &message);
    assert_eq!(is_verified, false);
  }

  #[test]
  fn sign_verify_different_message() {
    let curve = Box::new(Secp256k1::new());
    let hasher = Box::new(Sha256());
    let ecdsa = Ecdsa::new(curve, hasher);
    let group = &ecdsa.curve.f_n;

    let message = vec![1u8, 2, 3];

    // sign with newly generated private key
    let priv_key = group.rand_elem(true);
    let sig = ecdsa.sign(&priv_key, &message).unwrap();

    // create public key from the private key used for signing for verifying
    let pub_key = curve.scalar_mul(&curve.g, &priv_key.n);

    // change message and verify
    let message = vec![1u8, 2, 3, 4];
    let is_verified = ecdsa.verify(&sig, &pub_key, &message);
    assert_eq!(is_verified, false);
  }
}