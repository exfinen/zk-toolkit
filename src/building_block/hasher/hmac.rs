use super::hasher::Hasher;

// based on: https://datatracker.ietf.org/doc/html/rfc2104

pub struct Hmac<const N: usize> {
  hasher: Box<dyn Hasher<N>>,
}

impl <const N: usize> Hmac<N> {
  pub fn new(hasher: Box<dyn Hasher<N>>) -> Self {
    Hmac { hasher }
  }

  pub fn get_digest(&self, key: &[u8], text: &[u8]) -> [u8; N] {
    let block_size: usize = self.hasher.get_block_size();  // B
    let output_size: usize = N;  // hasher output size L

    // k is of block_size length w/ key expended w/ 0 paddings at the end
    // first hash key if key is longer than block_size
    let mut k = vec![0u8; block_size];

    if key.len() > block_size {
      let key_digest = self.hasher.get_digest(key);
      k[0..key_digest.len()].copy_from_slice(&key_digest[..]);
    } else {
      k[0..key.len()].copy_from_slice(&key[..]);
    }

    let mut r = vec![0u8; block_size + text.len()];
    let mut s = vec![0u8; block_size + output_size];

    for i in 0..block_size {
      r[i] = k[i] ^ 0x36;  // r = k XOR ipad(0x36)
      s[i] = k[i] ^ 0x5c;  // s = k XOR opad(0x5c)
    }

    // digest = H(k XOR opad || H(k XOR ipad || text))
    r[block_size..].copy_from_slice(&text[..]);
    let inner_digest = self.hasher.get_digest(&r);

    s[block_size..].copy_from_slice(&inner_digest);
    let digest = self.hasher.get_digest(&s);

    digest
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use super::super::sha256::Sha256;
  use hex::ToHex;

  #[test]
  fn hmac_empty_key_empty_text() {
    let hasher = Sha256();
    let hmac = Hmac::new(Box::new(hasher));

    let key = [];
    let text = [];
    let digest = hmac.get_digest(&key, &text);
    assert_eq!(digest.encode_hex::<String>(), "b613679a0814d9ec772f95d778c35fc5ff1697c493715653c6c712144292c5ad");
  }

  #[test]
  fn hmac_non_empty_key_non_empty_text() {
    let hasher = Sha256();
    let hmac = Hmac::new(Box::new(hasher));

    let key = b"key foo";
    let text = b"some text";
    let digest = hmac.get_digest(key, text);
    assert_eq!(digest.encode_hex::<String>(), "570b8926badb58b7652a00954f8ff36c872003b47c442419c342c5ebf5117d33");
  }

  #[test]
  fn hmac_non_empty_key_long_text() {
    let hasher = Sha256();
    let hmac = Hmac::new(Box::new(hasher));

    let key = b"fx502p";
    let text = b"The identity of the longest word in the English language depends upon the definition of what constitutes a word in the English language, as well as how length should be compared.";
    let digest = hmac.get_digest(key, text);
    assert_eq!(digest.encode_hex::<String>(), "7767617394b05a76be1959b0720891a152536ef407315e8eeb9209957d07c38e");
  }
}