use crate::elliptic_curve::{EllipticCurve, AddOps};
use crate::ec_point::EcPoint;
use crate::field::{Field, FieldElem};
use crate::hasher::Hasher;
use num_bigint::{BigUint};
use num_traits::identities::Zero;
use crate::sha256::Sha256;

pub struct Ecdsa<'a, const HASHER_OUT_SIZE: usize> {
  pub curve: &'a dyn EllipticCurve,
  pub ops: &'a dyn AddOps,
  pub hasher: &'a dyn Hasher<HASHER_OUT_SIZE>,
  pub n: Field,
}

#[derive(Debug, Clone)]
pub struct Signature {
  pub r: FieldElem,   // mod n
  pub s: FieldElem,   // mod n
}

impl<'a, const HASHER_OUT_SIZE: usize> Ecdsa<'a, HASHER_OUT_SIZE> {
  pub fn new(
    curve: &'a dyn EllipticCurve, 
    ops: &'a dyn AddOps, 
    hasher: &'a dyn Hasher<HASHER_OUT_SIZE>,
  ) -> Self {
    let n = curve.n().clone();
    Ecdsa { curve, ops, hasher, n }
  }

  pub fn gen_pub_key(&self, priv_key: &FieldElem) -> EcPoint {
    self.ops.scalar_mul(&self.curve.g(), &priv_key.n)
  }

  pub fn sign(&mut self, priv_key: &FieldElem, message: &[u8]) -> Result<Signature, String> {
    // n is 32-byte long in secp256k1
    // dA = private key in [1, n-1]
    let n = self.curve.n();

    let sha256 = Sha256();

    loop {
      // generate temporary non-zero random number k (mod n) 
      let k = self.n.rand_elem(true);  // mod n
      
      // e = HASH(message)
      // z = e's uppermost Ln bits (Ln = order of n = 256 bits)
      let z = BigUint::from_bytes_be(&sha256.get_digest(message));

      // p = kG (k != 0)
      let p: EcPoint = self.ops.scalar_mul(&self.curve.g(), &k.n);

      // r = p.x mod n
      let r = p.x.n % n.order.as_ref();

      // if r is 0, k is bad. repeat the process from the beggining
      if r == BigUint::zero() {
        continue;
      }
      // s = k^-1(z + r * dA) mod n // if s == 0, generate k again
      let k_inv = k.inv();  // mod n
      let r_fe = n.elem(&r);  // mod n
      let z_fe = n.elem(&z);  // mod n
      let s = k_inv * (priv_key * &r_fe + z_fe);  // mod n
      // if s is 0, k is bad. repear the process from the beginning
      if s.n == BigUint::zero() {
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
    else if !self.ops.scalar_mul(pub_key, &n.order.as_ref()).is_inf { 
      false
    }
    // check if r and s are in [1, n-1]
    else if 
      *&sig.r.n.is_zero()
      || *&sig.s.n.is_zero()
      || n.order.as_ref() <= &sig.r.n
      || n.order.as_ref() <= &sig.s.n {
      false
    } 
    else {
      // compute e = HASH(m)
      // z = e's uppermost Ln bits (Ln = order of n = 256 bits)
      let z = BigUint::from_bytes_be(&self.hasher.get_digest(message));
      let z_fe = FieldElem::new(&self.n, &z);  // mod n
      let w = sig.s.inv();  // mod n
      let u1 = z_fe * &w;  // mod n
      let u2 = &sig.r * w;  // mod n

      // (x, y) = u1 * G + u2 * PubKey
      let p1 = self.ops.scalar_mul(&self.curve.g(), &u1.n);
      let p2 = self.ops.scalar_mul(&pub_key, &u2.n);
      let p3 = self.ops.add(&p1, &p2);
      sig.r.n == (p3.x.n % n.order.as_ref())
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
  fn sign_verify_bad_pub_key() {
    let curve = WeierstrassEq::secp256k1();
    let ops = JacobianAddOps::new();
    let hasher = Sha256();
    let mut ecdsa = Ecdsa::new(&curve, &ops, &hasher);

    let message = vec![1u8, 2, 3];

    // sign with newly generated private key
    let priv_key = ecdsa.n.rand_elem(true);
    let sig = ecdsa.sign(&priv_key, &message).unwrap();

    let good_pub_key = ops.scalar_mul(&curve.g(), &priv_key.n);
    let bad_pub_key = EcPoint {
      x: good_pub_key.x.clone(),
      y: good_pub_key.x.clone(),
      is_inf: false,
    };
    let is_verified = ecdsa.verify(&sig, &bad_pub_key, &message);
    assert_eq!(is_verified, false);
  }

  #[test]
  fn sign_verify_inf_pub_key() {
    let curve = WeierstrassEq::secp256k1();
    let ops = JacobianAddOps::new();
    let hasher = Sha256();
    let mut ecdsa = Ecdsa::new(&curve, &ops, &hasher);

    let message = vec![1u8, 2, 3];

    // sign with newly generated private key
    let priv_key = ecdsa.n.rand_elem(true);
    let sig = ecdsa.sign(&priv_key, &message).unwrap();

    // use inf public key for verifying
    let pub_key = EcPoint::inf();
    let is_verified = ecdsa.verify(&sig, &pub_key, &message);
    assert_eq!(is_verified, false);
  }

  #[test]
  fn sign_verify_sig_r_out_of_range() {
    let curve = WeierstrassEq::secp256k1();
    let ops = JacobianAddOps::new();
    let hasher = Sha256();
    let mut ecdsa = Ecdsa::new(&curve, &ops, &hasher);

    let message = vec![1u8, 2, 3];

    // sign with newly generated private key
    let priv_key = ecdsa.n.rand_elem(true);

    // create public key from the private key used for signing for verifying
    let pub_key = ops.scalar_mul(&curve.g(), &priv_key.n);

    let sig = ecdsa.sign(&priv_key, &message).unwrap();

    let sig_r_too_large = Signature {
      r: sig.clone().s.f.elem(curve.n().order.as_ref()),
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
    let curve = WeierstrassEq::secp256k1();
    let ops = JacobianAddOps::new();
    let hasher = Sha256();
    let mut ecdsa = Ecdsa::new(&curve, &ops, &hasher);

    let message = vec![1u8, 2, 3];

    // sign with newly generated private key
    let priv_key = ecdsa.n.rand_elem(true);

    // create public key from the private key used for signing for verifying
    let pub_key = ops.scalar_mul(&curve.g(), &priv_key.n);

    let sig = ecdsa.sign(&priv_key, &message).unwrap();

    let sig_s_too_large = Signature {
      r: sig.clone().r,
      s: sig.clone().s.f.elem(curve.n().order.as_ref()),
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
    let curve = WeierstrassEq::secp256k1();
    let ops = JacobianAddOps::new();
    let hasher = Sha256();
    let mut ecdsa = Ecdsa::new(&curve, &ops, &hasher);

    let message = vec![1u8, 2, 3];

    // sign with newly generated private key
    let priv_key = ecdsa.n.rand_elem(true);
    let sig = ecdsa.sign(&priv_key, &message).unwrap();

    // create public key from the private key used for signing for verifying
    let pub_key = ecdsa.gen_pub_key(&priv_key);
    let is_verified = ecdsa.verify(&sig, &pub_key, &message);
    assert_eq!(is_verified, true);
  }

  #[test]
  fn sign_verify_bad_priv_key() {
    let curve = WeierstrassEq::secp256k1();
    let ops = JacobianAddOps::new();
    let hasher = Sha256();
    let mut ecdsa = Ecdsa::new(&curve, &ops, &hasher);

    let message = vec![1u8, 2, 3];

    // sign with newly generated private key
    let priv_key = ecdsa.n.rand_elem(true);
    let sig = ecdsa.sign(&priv_key, &message).unwrap();

    // change private key and create public key from it
    let priv_key = ecdsa.n.rand_elem(true);
    let pub_key = ops.scalar_mul(&curve.g(), &priv_key.n);

    let is_verified = ecdsa.verify(&sig, &pub_key, &message);
    assert_eq!(is_verified, false);
  }

  #[test]
  fn sign_verify_different_message() {
    let curve = WeierstrassEq::secp256k1();
    let ops = JacobianAddOps::new();
    let hasher = Sha256();
    let mut ecdsa = Ecdsa::new(&curve, &ops, &hasher);

    let message = vec![1u8, 2, 3];

    // sign with newly generated private key
    let priv_key = ecdsa.n.rand_elem(true);
    let sig = ecdsa.sign(&priv_key, &message).unwrap();

    // create public key from the private key used for signing for verifying
    let pub_key = ops.scalar_mul(&curve.g(), &priv_key.n);

    // change message and verify
    let message = vec![1u8, 2, 3, 4];
    let is_verified = ecdsa.verify(&sig, &pub_key, &message);
    assert_eq!(is_verified, false);
  }
}