use crate::building_block::{
  elliptic_curve::{
    affine_point::AffinePoint,
    curve::Curve,
    curve_equation::CurveEquation,
    elliptic_curve_point_ops::EllipticCurvePointOps,
    ec_point::EcPoint,
    new_affine_point::NewAffinePoint,
    weierstrass::adder::point_adder::PointAdder,
  },
  field::field::Field,
  hasher::{
    hasher::Hasher,
    sha256::Sha256,
  },
  zero::Zero, additive_identity::AdditiveIdentity,
};
use num_bigint::BigUint;

pub struct Ecdsa<const HASHER_OUT_SIZE: usize, Op, Eq, P, E, F, A>
where
  F: Field<F>,
  E: Zero<E> + AdditiveIdentity<E>,
  P: Zero<P> + AdditiveIdentity<E> + NewAffinePoint<P, E> + AffinePoint<P, E> + Clone,
  A: PointAdder<P, F>,
  Op: EllipticCurvePointOps<P, E, F, Adder = A>,
  Eq: CurveEquation<P> {
  pub curve: Box<dyn Curve<Op, Eq, P, E, F>>,
  pub hasher: Box<dyn Hasher<HASHER_OUT_SIZE>>,
}

#[derive(Debug, Clone)]
pub struct Signature<E> {
  pub r: E,
  pub s: E,
}

impl<const HASHER_OUT_SIZE: usize, Op, Eq, P, E, F, A> Ecdsa<HASHER_OUT_SIZE, Op, Eq, P, E, F, A>
  where
    F: Field<F>,
    E: Zero<E> + AdditiveIdentity<E>,
    P: Zero<P> + AdditiveIdentity<E> + NewAffinePoint<P, E> + AffinePoint<P, E> + Clone,
    A: PointAdder<P, F>,
    Op: EllipticCurvePointOps<P, E, F, Adder = A>,
    Eq: CurveEquation<P> {

  pub fn new(
    curve: Box<dyn Curve<Op, Eq, P, E, F>>,
    hasher: Box<dyn Hasher<HASHER_OUT_SIZE>>,
  ) -> Self {
    Ecdsa { curve, hasher }
  }

  pub fn gen_pub_key(&self, priv_key: &E) -> P {
    let g = &self.curve.g();
    self.curve.ops().scalar_mul(g, &priv_key)
  }

  pub fn sign(&mut self, priv_key: &E, message: &[u8]) -> Result<Signature<E>, String> {
    // n is 32-byte long in secp256k1
    // dA = private key in [1, n-1]
    let f_n = self.curve.group();
    let n = &f_n.order;
    let ops = &self.curve.ops();
    let g = &self.curve.g();
    let sha256 = Sha256();

    loop {
      // generate temporary non-zero random number k (mod n)
      let k = f_n.rand_elem(true);

      // e = HASH(message)
      // z = e's uppermost Ln bits (Ln = order of n = 256 bits)
      let z = BigUint::from_bytes_be(&sha256.get_digest(message));

      // p = kG (k != 0)
      let p: EcPoint = ops.scalar_mul(g, &k.n);

      // r = p.x mod n
      let r = p.x.n % n;

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
      if s.n == BigUint::zero() {
        continue;
      }

      return Ok(Signature { r: r_fe, s });
    }
  }

  // pub key is modulo p. not n which is the order of g
  pub fn verify(&self, sig: &Signature<E>, pub_key: &P, message: &[u8]) -> bool {
    let curve_group = &self.curve.group();
    let n = &curve_group.order;
    let g = &self.curve.g();
    let ops = &self.curve.ops();

    // confirm pub_key is not inf
    if pub_key.is_inf {
      false
    }
    // confirm pub_key is on the curve
    else if !self.curve.equation().is_rational_point(&pub_key) {
      false
    }
    // confirm n * pub_key is inf
    else if !ops.scalar_mul(pub_key, n).is_inf {
      false
    }
    // check if r and s are in [1, n-1]
    else if
      *&sig.r.n.is_zero()
      || *&sig.s.n.is_zero()
      || n <= &sig.r.n
      || n <= &sig.s.n {
      false
    }
    else {
      // compute e = HASH(m)
      // z = e's uppermost Ln bits (Ln = order of n = 256 bits)
      let z = BigUint::from_bytes_be(&self.hasher.get_digest(message));
      let z_fe = curve_group.elem(&z);  // mod n
      let w = sig.s.inv();  // mod n
      let u1 = z_fe * &w;  // mod n
      let u2 = &sig.r * w;  // mod n

      // (x, y) = u1 * G + u2 * PubKey
      let p1 = ops.scalar_mul(&g, &u1.n);
      let p2 = ops.scalar_mul(&pub_key, &u2.n);
      let p3 = ops.add(&p1, &p2);
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
        curves::secp256k1::{Secp256k1, Secp256k1Params},
      },
    },
    field::{
      prime_field::PrimeField,
      prime_field_elem::PrimeFieldElem,
    },
  };

  #[derive(Clone)]
  struct AffineOps();

  impl EllipticCurvePointOps<EcPoint, PrimeFieldElem, PrimeField> for AffineOps {
    type Adder = AffinePointAdder;
  }

  #[test]
  // TODO create separate tests for not-on-curve and pub_key-not-order-n cases
  fn sign_verify_bad_pub_key() {
    let params = Secp256k1Params::new();
    let curve = Box::new(Secp256k1::new(AffineOps(), params));
    let hasher = Box::new(Sha256());
    let mut ecdsa = Ecdsa::new(curve.clone(), hasher);

    let curve_group = &curve.group();
    let g = &curve.g();
    let ops = &curve.ops();

    let message = vec![1u8, 2, 3];

    // sign with newly generated private key
    let priv_key = curve_group.rand_elem(true);
    let sig = ecdsa.sign(&priv_key, &message).unwrap();

    let good_pub_key = ops.scalar_mul(g, &priv_key.n);
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
    let params = Secp256k1Params::new();
    let curve = Box::new(Secp256k1::new(AffineOps(), params));
    let hasher = Box::new(Sha256());
    let mut ecdsa = Ecdsa::new(curve, hasher);

    let message = vec![1u8, 2, 3];

    // sign with newly generated private key
    let priv_key = ecdsa.curve.group().rand_elem(true);
    let sig = ecdsa.sign(&priv_key, &message).unwrap();

    // use inf public key for verifying
    let pub_key = EcPoint::inf(&params.f);
    let is_verified = ecdsa.verify(&sig, &pub_key, &message);
    assert_eq!(is_verified, false);
  }

  #[test]
  fn sign_verify_sig_r_out_of_range() {
    let params = Secp256k1Params::new();
    let ops = AffineOps();
    let curve = Box::new(Secp256k1::new(ops.clone(), params));
    let hasher = Box::new(Sha256());
    let mut ecdsa = Ecdsa::new(curve, hasher);
    let group = &ecdsa.curve.group();

    let message = vec![1u8, 2, 3];

    // sign with newly generated private key
    let priv_key = group.rand_elem(true);

    // create public key from the private key used for signing for verifying
    let pub_key = ops.scalar_mul(&params.g, &priv_key.n);

    let sig = ecdsa.sign(&priv_key, &message).unwrap();

    let sig_r_too_large = Signature {
      r: sig.clone().s.f.elem(&params.n),
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
    let params = Secp256k1Params::new();
    let ops = AffineOps();
    let curve = Box::new(Secp256k1::new(ops.clone(), params));
    let hasher = Box::new(Sha256());
    let mut ecdsa = Ecdsa::new(curve, hasher);
    let group = &ecdsa.curve.group();

    let message = vec![1u8, 2, 3];

    // sign with newly generated private key
    let priv_key = group.rand_elem(true);

    // create public key from the private key used for signing for verifying
    let pub_key = ops.scalar_mul(&params.g, &priv_key.n);

    let sig = ecdsa.sign(&priv_key, &message).unwrap();

    let sig_s_too_large = Signature {
      r: sig.clone().r,
      s: sig.clone().s.f.elem(&params.n),
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
    let params = Secp256k1Params::new();
    let ops = AffineOps();
    let curve = Box::new(Secp256k1::new(ops.clone(), params));
    let hasher = Box::new(Sha256());
    let mut ecdsa = Ecdsa::new(curve, hasher);
    let group = &ecdsa.curve.group();

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
    let params = Secp256k1Params::new();
    let ops = AffineOps();
    let curve = Box::new(Secp256k1::new(ops.clone(), params));
    let hasher = Box::new(Sha256());
    let mut ecdsa = Ecdsa::new(curve, hasher);
    let group = &ecdsa.curve.group();

    let message = vec![1u8, 2, 3];

    // sign with newly generated private key
    let priv_key = group.rand_elem(true);
    let sig = ecdsa.sign(&priv_key, &message).unwrap();

    // change private key and create public key from it
    let priv_key = group.rand_elem(true);
    let pub_key = ops.scalar_mul(&params.g, &priv_key.n);

    let is_verified = ecdsa.verify(&sig, &pub_key, &message);
    assert_eq!(is_verified, false);
  }

  #[test]
  fn sign_verify_different_message() {
    let params = Secp256k1Params::new();
    let ops = AffineOps();
    let curve = Box::new(Secp256k1::new(ops.clone(), params));
    let hasher = Box::new(Sha256());
    let mut ecdsa = Ecdsa::new(curve, hasher);
    let group = &ecdsa.curve.group();

    let message = vec![1u8, 2, 3];

    // sign with newly generated private key
    let priv_key = group.rand_elem(true);
    let sig = ecdsa.sign(&priv_key, &message).unwrap();

    // create public key from the private key used for signing for verifying
    let pub_key = ops.scalar_mul(&params.g, &priv_key.n);

    // change message and verify
    let message = vec![1u8, 2, 3, 4];
    let is_verified = ecdsa.verify(&sig, &pub_key, &message);
    assert_eq!(is_verified, false);
  }
}