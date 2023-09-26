use crate::building_block::{
  curves::{
    secp256k1::affine_point::AffinePoint,
    rational_point::RationalPoint,
  },
  field::prime_field_elem::PrimeFieldElem,
  hasher::{
    hasher::Hasher,
    sha256::Sha256,
  },
  zero::Zero,
};
use num_bigint::BigUint;
use num_traits::Zero as NumTraisZero;

#[derive(Debug, Clone)]
pub struct Signature {
  pub r: PrimeFieldElem,
  pub s: PrimeFieldElem,
}

pub struct Ecdsa {
  pub hasher: Sha256,
}

impl Ecdsa {
  pub fn new(hasher: &Sha256) -> Self {
    Ecdsa {
      hasher: hasher.clone(),
    }
  }

  pub fn gen_pub_key(&self, priv_key: &PrimeFieldElem) -> AffinePoint {
    AffinePoint::g() * priv_key
  }

  pub fn sign(&self, priv_key: &PrimeFieldElem, message: &[u8]) -> Result<Signature, String> {
    let f_n = &AffinePoint::curve_group();

    if priv_key.f.order_ref() != f_n.order_ref() {
      panic!("Private key needs to be an element of curve group");
    }
    // n is 32-byte long in secp256k1
    // dA = private key in [1, n-1]
    let n = f_n.order_ref();
    let g = &AffinePoint::g();
    let sha256 = Sha256();

    loop {
      // generate temporary non-zero random number k (mod n)
      let k = f_n.rand_elem(true);

      // e = HASH(message)
      // z = e's uppermost Ln bits (Ln = order of n = 256 bits)
      let z = BigUint::from_bytes_be(&sha256.get_digest(message));

      // p = kG (k != 0)
      let p = g * &k;

      match p {
        AffinePoint::AtInfinity => continue,
        AffinePoint::Rational { x, y: _ } => {
          // r = p.x mod n
          let r = x.e % n;

          // if r is 0, k is bad. repeat the process from the beggining
          if r == BigUint::zero() {
            continue;
          }
          // s = k^-1(z + r * dA) mod n // if s == 0, generate k again
          let k_inv = k.inv();  // mod n
          let r_fe = f_n.elem(&r);  // mod n
          let z_fe = f_n.elem(&z);  // mod n
          let s = k_inv * (priv_key * &r_fe + &z_fe);  // mod n

          // if s is 0, k is bad. repear the process from the beginning
          if s.e == BigUint::zero() {
            continue;
          }

          return Ok(Signature { r: r_fe, s });
        },
      }
    }
  }

  // pub key is modulo p. not n which is the order of g
  pub fn verify(&self, sig: &Signature, pub_key: &AffinePoint, message: &[u8]) -> bool {
    let f_q = &AffinePoint::base_field();
    let f_n = &AffinePoint::curve_group();
    let n = f_n.order_ref();

    // confirm pub_key is not inf
    if pub_key.is_zero() {
      false
    }
    // confirm pub_key is on the curve
    else if !pub_key.is_rational_point() {
      false
    }
    // confirm n * pub_key is inf
    else if !(pub_key * f_q.elem(n)).is_zero() {
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
      let g = AffinePoint::g();
      let p1 = g * &u1;
      let p2 = pub_key * &u2;
      let p3 = &p1 + &p2;

      match p3 {
        AffinePoint::AtInfinity => false,
        AffinePoint::Rational { x, y: _ } => {
          sig.r.e == (x.e % n)
        },
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  // TODO create separate tests for not-on-curve and pub_key-not-order-n cases
  fn sign_verify_bad_pub_key() {
    let hasher = Sha256();
    let ecdsa = Ecdsa::new(&hasher);

    let curve_group = AffinePoint::curve_group();
    let g = &AffinePoint::g();
    let message = vec![1u8, 2, 3];

    // sign with newly generated private key
    let priv_key = curve_group.rand_elem(true);
    let sig = ecdsa.sign(&priv_key, &message).unwrap();
    println!("{:?}", sig);

    let good_pub_key = g * &priv_key;
    match good_pub_key {
      AffinePoint::AtInfinity => panic!("Expected rational point, but got point at infinity"),
      AffinePoint::Rational { x, y: _ } => {
        let bad_pub_key = AffinePoint::new(
          &x,
          &x,
        );
        match bad_pub_key {
          AffinePoint::AtInfinity => panic!("Should not be visited"),
          p => {
            let is_verified = ecdsa.verify(&sig, &p, &message);
            assert_eq!(is_verified, false);
          },
        }
      },
    }
  }

  #[test]
  fn sign_verify_inf_pub_key() {
    let hasher = Sha256();
    let ecdsa = Ecdsa::new(&hasher);
    let f_n = &AffinePoint::curve_group();

    let message = vec![1u8, 2, 3];

    // sign with newly generated private key
    let priv_key = f_n.rand_elem(true);
    let sig = ecdsa.sign(&priv_key, &message).unwrap();

    // use inf public key for verifying
    let pub_key = AffinePoint::zero();
    let is_verified = ecdsa.verify(&sig, &pub_key, &message);
    assert_eq!(is_verified, false);
  }

  #[test]
  fn sign_verify_sig_r_out_of_range() {
    let hasher = Sha256();
    let ecdsa = Ecdsa::new(&hasher);
    let curve_group = &AffinePoint::curve_group();
    let g = &AffinePoint::g();

    let message = vec![1u8, 2, 3];

    // sign with newly generated private key
    let priv_key = &curve_group.rand_elem(true);

    // create public key from the private key used for signing for verifying
    let pub_key = g * priv_key;

    let sig = ecdsa.sign(priv_key, &message).unwrap();

    let sig_r_too_large = Signature {
      r: AffinePoint::base_field().elem(curve_group.order_ref()),
      s: sig.s.clone(),
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
    let hasher = Sha256();
    let ecdsa = Ecdsa::new(&hasher);
    let curve_group = &AffinePoint::curve_group();
    let g = &AffinePoint::g();

    let message = vec![1u8, 2, 3];

    // sign with newly generated private key
    let priv_key = curve_group.rand_elem(true);

    // create public key from the private key used for signing for verifying
    let pub_key = g * &priv_key;

    let sig = ecdsa.sign(&priv_key, &message).unwrap();

    let sig_s_too_large = Signature {
      r: sig.r.clone(),
      s: sig.s.clone().f.elem(curve_group.order_ref()),
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
    let hasher = Sha256();
    let ecdsa = Ecdsa::new(&hasher);
    let curve_group = &AffinePoint::curve_group();

    let message = vec![1u8, 2, 3];

    // sign with newly generated private key
    let priv_key = curve_group.elem(&1234u32); // group.rand_elem(true);
    let sig = ecdsa.sign(&priv_key, &message).unwrap();

    // create public key from the private key used for signing for verifying
    let pub_key = ecdsa.gen_pub_key(&priv_key);
    let is_verified = ecdsa.verify(&sig, &pub_key, &message);
    assert_eq!(is_verified, true);
  }

  #[test]
  fn sign_verify_bad_priv_key() {
    let hasher = Sha256();
    let ecdsa = Ecdsa::new(&hasher);
    let curve_group = &AffinePoint::curve_group();
    let g = &AffinePoint::g();

    let message = vec![1u8, 2, 3];

    // sign with newly generated private key
    let priv_key = curve_group.rand_elem(true);
    let sig = ecdsa.sign(&priv_key, &message).unwrap();

    // change private key and create public key from it
    let priv_key = curve_group.rand_elem(true);
    let pub_key = g * &priv_key;

    let is_verified = ecdsa.verify(&sig, &pub_key, &message);
    assert_eq!(is_verified, false);
  }

  #[test]
  fn sign_verify_different_message() {
    let hasher = Sha256();
    let ecdsa = Ecdsa::new(&hasher);
    let curve_group = &AffinePoint::curve_group();
    let g = &AffinePoint::g();

    let message = vec![1u8, 2, 3];

    // sign with newly generated private key
    let priv_key = curve_group.rand_elem(true);
    let sig = ecdsa.sign(&priv_key, &message).unwrap();

    // create public key from the private key used for signing for verifying
    let pub_key = g * &priv_key;

    // change message and verify
    let message = vec![1u8, 2, 3, 4];
    let is_verified = ecdsa.verify(&sig, &pub_key, &message);
    assert_eq!(is_verified, false);
  }
}
