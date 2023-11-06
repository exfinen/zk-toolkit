#[derive(Debug)]
pub struct Number {
  pub bits: [bool; 64],
}

impl Number {
  pub fn new(n: i64) -> Self {
    let mut bits = [false; 64];

    if n == 0 {
      Number { bits }

    } else {
      let mut m = n;
      if n < 0 {
        // convert to a positive number w/ the same bit representation
        m = i64::MAX + n + 1;
      }

      let mut x = m;
      let mut i = 0;
      while x > 0 {
        if x & 1 == 1 {
          bits[i] = true;
        }
        i += 1;
        x >>= 1;
      }

      if n < 0 {  // set sign bit if originally a negative value
        bits[63] = true;
      }

      Number { bits }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn zero() {
    let x = Number::new(0);
    for i in 0..64 {
      assert_eq!(x.bits[i], false);
    }
  }

  #[test]
  fn pos_1() {
    let x = Number::new(1);
    for i in 0..64 {
      let exp = i == 0;
      assert_eq!(x.bits[i], exp);
    }
  }

  #[test]
  fn pos_5() {
    let x = Number::new(5);
    assert_eq!(x.bits[0], true);
    assert_eq!(x.bits[1], false);
    assert_eq!(x.bits[2], true);
    for i in 3..64 {
      assert_eq!(x.bits[i], false);
    }
  }

  #[test]
  fn pos_7() {
    let x = Number::new(7);
    assert_eq!(x.bits[0], true);
    assert_eq!(x.bits[1], true);
    assert_eq!(x.bits[2], true);
    for i in 3..64 {
      assert_eq!(x.bits[i], false);
    }
  }

  #[test]
  fn neg_1() {
    let x = Number::new(-1);
    for i in 0..64 {
      assert_eq!(x.bits[i], true);
    }
  }

  #[test]
  fn neg_5() {
    let x = Number::new(-5);
    assert_eq!(x.bits[0], true);
    assert_eq!(x.bits[1], true);
    assert_eq!(x.bits[2], false);
    for i in 3..64 {
      assert_eq!(x.bits[i], true);
    }
  }

  #[test]
  fn neg_7() {
    let x = Number::new(-7);
    assert_eq!(x.bits[0], true);
    assert_eq!(x.bits[1], false);
    assert_eq!(x.bits[2], false);
    for i in 3..64 {
      assert_eq!(x.bits[i], true);
    }
  }
}  