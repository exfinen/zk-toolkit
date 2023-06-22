#![allow(non_snake_case)]
use crate::building_block::{
  ec_additive_group_ops::EcAdditiveGroupOps,
  ec_point::EcPoint,
  field::{Field, FieldElem},
  hasher::Hasher,
  sha512::Sha512,
};
use num_bigint::BigUint;
use core::ops::{Add, Sub, Rem};
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
  H: Sha512,
  f: Field,
  l: BigUint,
  B: EcPoint,
  d: FieldElem,
  one: FieldElem,
  zero: FieldElem,
}

impl EcAdditiveGroupOps for Ed25519Sha512 {
  // Edwards Addition Law
  // (x1,y1) + (x2,y2) = ((x1y2 + x2y1) / (1 + d x1x2 y1y2), (y1y2 + x1x2) / (1 - d x1x2 y1y2))
  fn add(&self, p1: &EcPoint, p2: &EcPoint) -> EcPoint {
    if p1.is_inf {
      p2.clone()
    }
    else if p2.is_inf {
      p1.clone()
    } else {
      let x1y2 = &p1.x * &p2.y;
      let x2y1 = &p2.x * &p1.y;
      let x1x2y1y2 = &x1y2 * &x2y1;
      let y1y2 = &p1.y * &p2.y;
      let x1x2 = &p1.x * &p2.x;
      let x = (x1y2 + x2y1) / (self.f.elem(&1u8) + (&self.d * &x1x2y1y2));
      let y = (y1y2 + x1x2) / (self.f.elem(&1u8) - (&self.d * x1x2y1y2));
      EcPoint::new(&x, &y)
    }
  }

  fn inv(&self, _p: &EcPoint) -> EcPoint {
    panic!("not implemented");
  }
}

impl Ed25519Sha512 {
  pub fn new() -> Self {
    let H = Sha512();
    let two = BigUint::from(2u8);

    // order of base field q: 2^255 - 19
    let q = two.pow(255u32).sub(19u8);
    let f = Field::new(&q);

    // order of base point l: 2^252 + 27742317777372353535851937790883648493
    let l = two.pow(252u32).add(27742317777372353535851937790883648493u128);

    // d = -121665 / 121666
    let d = -f.elem(&121665u32) / 121666u32;

    // base point is (+x, 4/5)
    let B_y = f.elem(&4u8) / 5u8;
    let B_x = Self::recover_x(&d, &B_y, Parity::Even);  // get positive x
    let B = EcPoint::new(&B_x, &B_y);

    let one = f.elem(&1u8);
    let zero = f.elem(&0u8);

    Ed25519Sha512 { H, f, l, B, d, one, zero }
  }

  fn get_parity(e: &FieldElem) -> Parity {
    if (&e.n % 2u8).is_zero() { Parity::Even } else { Parity::Odd }
  }

  // d is passed to allow new() to call this function. ideally d should be replaced by &self
  fn recover_x(d: &FieldElem, y: &FieldElem, x_parity: Parity) -> FieldElem {
    let f = &d.f;
    let q = &*d.f.order;

    // xx = x^2 = (y^2 - 1) / (1 + d*y^2)
    let xx = (y.sq() - 1u8) / ((d * y.sq()) + 1u8);

    // calculate the square root of xx assuming a^((q-1)/4) = 1 mod q
    let mut x = (&xx).pow(&((q + &3u8) / &8u8));

    // if that doesn't match, calculate the square root of xx again 
    // assuming a^((q-1)/4) = -1 mod q
    if &x.sq().n != &xx.n {
      let I = f.elem(&2u8).pow(&((q - &1u8) / &4u8));
      x = &x * &I;
    }
    let root_parity = Self::get_parity(&x);
    if root_parity != x_parity {
      x = -&x;
    }
    x
  }

  fn write_biguint_to_32_byte_buf_as_le_integer(n: &BigUint) -> [u8; 32] {
    // serialize n to little-endian integer
    let bytes_le = n.to_bytes_le();
    assert!(bytes_le.len() <= 32);

    // then write to 32-byte buffer w/ 0 padding on higher index side
    // e.g. 0xab in little-endian in 4 byte buffer is ab 00 00 00 
    let mut buf = [0u8; 32];
    buf[0..bytes_le.len()].copy_from_slice(&bytes_le);
    buf
  }

  fn encode_point(&self, pt: &EcPoint) -> [u8; 32] {
    // get parity of x
    let x_parity = if (&pt.x.n & &self.one.n) == self.zero.n { Parity::Even } else { Parity::Odd };

    // write y to 32-byte buffer as little-endian integer
    let mut buf = Self::write_biguint_to_32_byte_buf_as_le_integer(&pt.y.n);

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
    let x = Self::recover_x(&self.d, &y, x_parity);

    EcPoint::new(&x, &y)
  }

  fn prune_32_byte_buf(buf: &mut [u8; 32]) {
    buf[31] &= 0b0111_1111;  // clear most significant bit
    buf[31] |= 0b0100_0000;  // set 2nd most significant bit
    buf[0] &= 0b1111_1000;  // clear least significant 3 bits
  }

  // creates s that is the last 32 bytes of the digest w/ pruning applied
  fn gen_s(digest_lower_32_bytes: &[u8; 32]) -> BigUint {
    let mut buf = [0u8; 32];
    buf[..].copy_from_slice(&digest_lower_32_bytes[0..32]);
    Self::prune_32_byte_buf(&mut buf);
    let s = BigUint::from_bytes_le(&buf);
    s
  }

  pub fn gen_pub_key(&self, prv_key: &[u8; 32]) -> [u8; 32] {
    let H = Sha512();
    let digest = H.get_digest(prv_key);

    // multiply B by s to get the public key
    let s = Self::gen_s(&digest[0..32].try_into().unwrap());
    let pub_key_pt = self.scalar_mul(&self.B, &s);
    let pub_key = self.encode_point(&pub_key_pt);
    pub_key
  }

  pub fn sign(&self, msg: &[u8], prv_key: &[u8; 32]) -> [u8; 64] {
    let digest = self.H.get_digest(prv_key);
    let s = Self::gen_s(&digest[0..32].try_into().unwrap());
    let prefix = &digest[32..64];

    let A_pt = self.scalar_mul(&self.B, &s);
    let A = self.encode_point(&A_pt);

    let prefix_msg = [prefix, msg].concat();
    let r = BigUint::from_bytes_le(
      &self.H.get_digest(&prefix_msg)
    );
    let r = r.rem(&self.l);
    let R_pt = self.scalar_mul(&self.B, &r);
    let R = self.encode_point(&R_pt);

    let R_A_msg = [&R, &A, msg].concat();
    let k = BigUint::from_bytes_le(
      &self.H.get_digest(&R_A_msg)
    );
    let k = k.rem(&self.l);

    let S = (r + k * s).rem(&self.l);
    let S32 = Self::write_biguint_to_32_byte_buf_as_le_integer(&S);

    [R, S32].concat().try_into().unwrap()
  }

  pub fn verify(&self, sig: &[u8;64], pub_key: &[u8; 32], msg: &[u8]) -> bool {
    let S = BigUint::from_bytes_le(&sig[32..64]);
    if S >= self.l {
      return false;
    }
    let R_pt = self.decode_point(&sig[0..32].try_into().unwrap());
    let R = self.encode_point(&R_pt);
    let R_pub_key_msg = [&R, pub_key, msg].concat();
    let k = BigUint::from_bytes_le(
      &self.H.get_digest(&R_pub_key_msg)
    );
    let A_pt = self.decode_point(&pub_key);

    let lhs = self.scalar_mul(&self.B, &(S * 8u8));

    let eight = BigUint::from(8u8);
    let rhs_term1 = self.scalar_mul(&R_pt, &eight);
    let rhs_term2 = self.scalar_mul(&A_pt, &(k * &eight));
    let rhs = self.add(&rhs_term1, &rhs_term2);

    lhs == rhs
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn adding_inf_test() {
    let ed25519 = Ed25519Sha512::new();
    let inf = &EcPoint::inf(&ed25519.f);
    let one = &ed25519.f.elem(&1u8);
    let non_inf = &EcPoint::new(one, one);
    {
      let pt = ed25519.add(inf, inf);
      assert!(pt.is_inf);
    }
    {
      let pt = ed25519.add(non_inf, inf);
      assert!(&pt == non_inf);
    }
    {
      let pt = ed25519.add(inf, non_inf);
      assert!(&pt == non_inf);
    }
    {
      let pt = ed25519.add(non_inf, non_inf);
      assert!(pt.is_inf == false);
    }
  }

  fn run_rfc8032_test(prv_key: &str, exp_pub_key: &str, msg: &[u8], exp_sig: &str) {
    let ed25519 = Ed25519Sha512::new();

    let prv_key: [u8; 32] = hex::decode(prv_key).unwrap().try_into().unwrap();
    let pub_key = ed25519.gen_pub_key(&prv_key);

    let exp_pub_key = hex::decode(exp_pub_key).unwrap();
    assert_eq!(exp_pub_key, pub_key);

    let sig = ed25519.sign(&msg, &prv_key);
    let exp_sig: [u8; 64] = hex::decode(exp_sig).unwrap().try_into().unwrap();

    assert_eq!(sig, exp_sig);
    assert!(ed25519.verify(&sig, &pub_key, msg));
  }

  #[test]
  fn rfc8032_test1() {
    let prv_key = "9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60";
    let exp_pub_key = "d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a";
    let msg = [];
    let exp_sig = "e5564300c360ac729086e2cc806e828a84877f1eb8e5d974d873e065224901555fb8821590a33bacc61e39701cf9b46bd25bf5f0595bbe24655141438e7a100b";
    run_rfc8032_test(prv_key, exp_pub_key, &msg, exp_sig);
  }

  #[test]
  fn rfc8032_test2() {
    let prv_key = "4ccd089b28ff96da9db6c346ec114e0f5b8a319f35aba624da8cf6ed4fb8a6fb";
    let exp_pub_key = "3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c";
    let msg = [0x72];
    let exp_sig = "92a009a9f0d4cab8720e820b5f642540a2b27b5416503f8fb3762223ebdb69da085ac1e43e15996e458f3613d0f11d8c387b2eaeb4302aeeb00d291612bb0c00";
    run_rfc8032_test(prv_key, exp_pub_key, &msg, exp_sig);
  }

  #[test]
  fn rfc8032_test3() {
    let prv_key = "c5aa8df43f9f837bedb7442f31dcb7b166d38535076f094b85ce3a2e0b4458f7";
    let exp_pub_key = "fc51cd8e6218a1a38da47ed00230f0580816ed13ba3303ac5deb911548908025";
    let msg = [0xaf, 0x82];
    let exp_sig = "6291d657deec24024827e69c3abe01a30ce548a284743a445e3680d7db5ac3ac18ff9b538d16f290ae67f760984dc6594a7c15e9716ed28dc027beceea1ec40a";
    run_rfc8032_test(prv_key, exp_pub_key, &msg, exp_sig);
  }

  #[test]
  fn rfc8032_test_1024() {
    let prv_key = "f5e5767cf153319517630f226876b86c8160cc583bc013744c6bf255f5cc0ee5";
    let exp_pub_key = "278117fc144c72340f67d0f2316e8386ceffbf2b2428c9c51fef7c597f1d426e";
    let msg = "08b8b2b733424243760fe426a4b54908632110a66c2f6591eabd3345e3e4eb98fa6e264bf09efe12ee50f8f54e9f77b1e355f6c50544e23fb1433ddf73be84d879de7c0046dc4996d9e773f4bc9efe5738829adb26c81b37c93a1b270b20329d658675fc6ea534e0810a4432826bf58c941efb65d57a338bbd2e26640f89ffbc1a858efcb8550ee3a5e1998bd177e93a7363c344fe6b199ee5d02e82d522c4feba15452f80288a821a579116ec6dad2b3b310da903401aa62100ab5d1a36553e06203b33890cc9b832f79ef80560ccb9a39ce767967ed628c6ad573cb116dbefefd75499da96bd68a8a97b928a8bbc103b6621fcde2beca1231d206be6cd9ec7aff6f6c94fcd7204ed3455c68c83f4a41da4af2b74ef5c53f1d8ac70bdcb7ed185ce81bd84359d44254d95629e9855a94a7c1958d1f8ada5d0532ed8a5aa3fb2d17ba70eb6248e594e1a2297acbbb39d502f1a8c6eb6f1ce22b3de1a1f40cc24554119a831a9aad6079cad88425de6bde1a9187ebb6092cf67bf2b13fd65f27088d78b7e883c8759d2c4f5c65adb7553878ad575f9fad878e80a0c9ba63bcbcc2732e69485bbc9c90bfbd62481d9089beccf80cfe2df16a2cf65bd92dd597b0707e0917af48bbb75fed413d238f5555a7a569d80c3414a8d0859dc65a46128bab27af87a71314f318c782b23ebfe808b82b0ce26401d2e22f04d83d1255dc51addd3b75a2b1ae0784504df543af8969be3ea7082ff7fc9888c144da2af58429ec96031dbcad3dad9af0dcbaaaf268cb8fcffead94f3c7ca495e056a9b47acdb751fb73e666c6c655ade8297297d07ad1ba5e43f1bca32301651339e22904cc8c42f58c30c04aafdb038dda0847dd988dcda6f3bfd15c4b4c4525004aa06eeff8ca61783aacec57fb3d1f92b0fe2fd1a85f6724517b65e614ad6808d6f6ee34dff7310fdc82aebfd904b01e1dc54b2927094b2db68d6f903b68401adebf5a7e08d78ff4ef5d63653a65040cf9bfd4aca7984a74d37145986780fc0b16ac451649de6188a7dbdf191f64b5fc5e2ab47b57f7f7276cd419c17a3ca8e1b939ae49e488acba6b965610b5480109c8b17b80e1b7b750dfc7598d5d5011fd2dcc5600a32ef5b52a1ecc820e308aa342721aac0943bf6686b64b2579376504ccc493d97e6aed3fb0f9cd71a43dd497f01f17c0e2cb3797aa2a2f256656168e6c496afc5fb93246f6b1116398a346f1a641f3b041e989f7914f90cc2c7fff357876e506b50d334ba77c225bc307ba537152f3f1610e4eafe595f6d9d90d11faa933a15ef1369546868a7f3a45a96768d40fd9d03412c091c6315cf4fde7cb68606937380db2eaaa707b4c4185c32eddcdd306705e4dc1ffc872eeee475a64dfac86aba41c0618983f8741c5ef68d3a101e8a3b8cac60c905c15fc910840b94c00a0b9d0";
    let exp_sig = "0aab4c900501b3e24d7cdf4663326a3a87df5e4843b2cbdb67cbf6e460fec350aa5371b1508f9f4528ecea23c436d94b5e8fcd4f681e30a6ac00a9704a188a03";
    run_rfc8032_test(prv_key, exp_pub_key, &hex::decode(msg).unwrap(), exp_sig);
  }

  #[test]
  fn rfc8032_test_sha_abc() {
    let prv_key = "833fe62409237b9d62ec77587520911e9a759cec1d19755b7da901b96dca3d42";
    let exp_pub_key = "ec172b93ad5e563bf4932c70e1245034c35467ef2efd4d64ebf819683467e2bf";
    let msg = "ddaf35a193617abacc417349ae20413112e6fa4e89a97ea20a9eeee64b55d39a2192992a274fc1a836ba3c23a3feebbd454d4423643ce80e2a9ac94fa54ca49f"; 
    let exp_sig = "dc2a4459e7369633a52b1bf277839a00201009a3efbf3ecb69bea2186c26b58909351fc9ac90b3ecfdfbc7c66431e0303dca179c138ac17ad9bef1177331a704";
    run_rfc8032_test(prv_key, exp_pub_key, &hex::decode(msg).unwrap(), exp_sig);
  }
}