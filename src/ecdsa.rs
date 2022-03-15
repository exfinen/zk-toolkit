use crate::curve::Curve;
use crate::ec_point::EcPoint;
use crate::field_elem::FieldElem;
use crate::field::Field;
use crate::random_number::RandomNumber;
use crate::weierstrass_eq::WeierstrassEq;
use num_bigint::{BigUint};
use rand::RngCore;
use std::rc::Rc;
use num_traits::identities::Zero;
use crate::hash::Hash;

struct Ecdsa<'a> {
  e: &'a dyn Curve,
  g: EcPoint,
  n: BigUint,
  f_n: Rc<Field>,
}

struct Signature {
  r: FieldElem,   // mod n
  s: FieldElem,   // mod n
}

impl<'a> Ecdsa<'a> {
  pub fn new(e: &'a dyn Curve) -> Self {
    let f_n = Field::new(e.n());
    Ecdsa { e, g: e.g(), n: e.n(), f_n }
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

  pub fn sign(&mut self, priv_key: &FieldElem, message: &[u8]) -> Signature {
    // n is 32-byte long in secp256k1
    // dA = private key in [1, n-1]

    // generate temporary random number k (mod n)
    let k = self.gen_random_number_order_n();
    
    // e = HASH(message)
    // z = e's leftmost Ln bits (Ln = order of n = 256 bits)
    let z = Hash::sha256(message);

    // p = kG
    let p = self.e.scalar_mul(&k.v);

    // r = p.x mod n
    let r = p.v % &self.n;
    // ** if r is 0, go back to generating k

    // s = k^-1(z + r * dA) mod n // if s == 0, generate k again
    let s = k.inv().mul(&(priv_key.mul(r).add(z)));
    // ** if s is 0, go back to generating k

    Signature { r, s }
  }

  // pub key is modulo p. not n which is the order of g
  pub fn verify(&self, sig: &Signature, pub_key: FieldElem, message: &[u8]) {
    // confirm pub_key is not inf
    // confirm pub_key is on the curve
    // confirm n * pub_key is inf

    // check if r and s are in [1, n-1]

    // compute e = HASH(m)
    // z = e's leftmost Ln bits (Ln = order of n = 256 bits)

    let z = Hash::sha256(message);
    let w = sig.s.inv();  // mod n
    let u1 = z.mul(&w);  // mod n
    let u2 = r.mul(&w);  // mod n
    let p = self.curve.scalar_mul(u1).add(pub_key.mul(u2));
    p.x % curve.n() == sig.r
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_1() {
    let e = WeierstrassEq::secp256k1();
    let ecdsa = Ecdsa::new(&e);

    // create private key
    let priv_key = ecdsa.gen_random_number_order_n();

    let pub_key = e.scalar_mul(&priv_key.v);

    println!("priv={:?}", priv_key);
    println!("pub={:?}", pub_key);
  }
}