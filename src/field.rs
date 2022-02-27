use num_bigint::{BigUint, BigInt, ToBigInt, ToBigUint};

pub struct Field {
  order: BigUint,
  zero: BigUint,
  one: BigUint,
}

pub struct FieldNum<'a> {
  f: &'a Field,
  v: BigUint,
}

impl Field {
  pub fn new(order: BigUint) -> Self {
    Field {
      order,
      zero: BigUint::from(0u32),
      one: BigUint::from(1u32),
    }
  }

  pub fn gen_element(&self, v: BigUint) -> FieldNum {
    FieldNum::new(self, v)
  }
}

impl <'a> FieldNum<'a> {
  pub fn new(f: &'a Field, v: BigUint) -> Self {
    if v >= f.order {
      let v = v % f.order.clone();
      FieldNum { f, v }
    } else {
      FieldNum { f, v }
    }
  }

  pub fn add(&self, other: &FieldNum) -> FieldNum {
    let c = self.v.clone() + other.v.clone();
    if c >= self.f.order {
      let v = c - self.f.order.clone();
      FieldNum { f: self.f, v }
    } else {
      FieldNum { f: self.f, v: c.clone() }
    }
  }

  pub fn sub(&self, other: &FieldNum) -> FieldNum {
    if self.v < other.v {
      let diff = other.v.clone() - self.v.clone();
      let v = self.f.order.clone() - diff;
      FieldNum { f: self.f, v }
    } else {
      let v = self.v.clone() - other.v.clone();
      FieldNum { f: self.f, v }
    }
  }

  pub fn mul(&self, other: &FieldNum) -> FieldNum {
    let v = (self.v.clone() * other.v.clone()) % self.f.order.clone();
    if v >= self.f.zero {
      FieldNum { f: self.f, v }
    } else {
      let v = self.f.order.clone() + v.clone();
      FieldNum { f: self.f, v }
    }
  }

  pub fn inv(&self) -> Result<FieldNum, String> {
    if self.v == self.f.zero {
      return Err("cannot find inverse of zero".to_string());
    }
    let order = self.f.order.to_bigint().unwrap();
    let v = self.v.to_bigint().unwrap();
    let zero = self.f.zero.to_bigint().unwrap();
    let one = self.f.one.to_bigint().unwrap();

    // x0*a + y0*b = a
    // x1*a + y1*b = b
    let mut r0 = v.clone();  // initially equals to a
    let mut r1 = order.clone();  // initially equals to b
    let mut x0 = one.clone();
    let mut y0 = zero.clone();
    let mut x1 = zero.clone();
    let mut y1 = one.clone();

    while r1 != zero {
      // a mod b
      // = a - q*b
      // = (x0*a + y0*b) - q*(x1*a + y1*b)
      // = x0*a - q*x1*a + y0*b - q*y1*b
      // = (x0 - x1*q)*a + (y0 - y1*q)*b
      // = r
      let q = r0.clone() / r1.clone();
      let r2 = r0.clone() % r1.clone();
      // this produces the same result as above r2 using mod
      //let r2 = x2.clone() * order.clone() + y2.clone() * v.clone();
      let x2 = x0 - x1.clone() * q.clone();
      let y2 = y0 - y1.clone() * q.clone();

      // do next calculattion based on new and previous equations
      r0 = r1;
      r1 = r2;
      x0 = x1;
      y0 = y1;
      x1 = x2;
      y1 = y2;
    }

    //println!("x0={}, x1={}", x0, x1);
    // if the result is not field element, convert it to field element
    let mut new_v = x0;
    if new_v < zero.clone() {
      while new_v < zero.clone() {
        new_v += order.clone();
      }
    } else {
      if new_v >= order {
        new_v %= order;
      }
    }
    Ok(FieldNum { f: self.f, v: new_v.to_biguint().unwrap() })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_add_eq_order_result() {
    let f = Field::new(BigUint::from(11u32));
    let a = f.gen_element(BigUint::from(9u32));
    let b = f.gen_element(BigUint::from(2u32));
    let c = a.add(&b);
    assert_eq!(c.v, BigUint::from(0u32));
  }

  #[test]
  fn test_add_below_order_result() {
    let f = Field::new(BigUint::from(11u32));
    let a = f.gen_element(BigUint::from(9u32));
    let b = f.gen_element(BigUint::from(1u32));
    let c = a.add(&b);
    assert_eq!(c.v, BigUint::from(10u32));
  }

  #[test]
  fn test_add_above_order_result() {
    let f = Field::new(BigUint::from(11u32));
    let a = f.gen_element(BigUint::from(9u32));
    let b = f.gen_element(BigUint::from(3u32));
    let c = a.add(&b);
    assert_eq!(c.v, BigUint::from(1u32));
  }

  #[test]
  fn test_sub_smaller_val() {
    let f = Field::new(BigUint::from(11u32));
    let a = f.gen_element(BigUint::from(9u32));
    let b = f.gen_element(BigUint::from(2u32));
    let c = a.sub(&b);
    assert_eq!(c.v, BigUint::from(7u32));
  }

  #[test]
  fn test_sub_eq_val() {
    let f = Field::new(BigUint::from(11u32));
    let a = f.gen_element(BigUint::from(9u32));
    let b = f.gen_element(BigUint::from(9u32));
    let c = a.sub(&b);
    assert_eq!(c.v, f.zero);
  }

  #[test]
  fn test_sub_larger_val() {
    let f = Field::new(BigUint::from(11u32));
    let a = f.gen_element(BigUint::from(9u32));
    let b = f.gen_element(BigUint::from(10u32));
    let c = a.sub(&b);
    assert_eq!(c.v, BigUint::from(10u32));
  }

  #[test]
  fn test_mul_below_order_result() {
    let f = Field::new(BigUint::from(11u32));
    let a = f.gen_element(BigUint::from(2u32));
    let b = f.gen_element(BigUint::from(5u32));
    let c = a.mul(&b);
    assert_eq!(c.v, BigUint::from(10u32));
  }

  #[test]
  fn test_mul_eq_order_result() {
    let f = Field::new(BigUint::from(11u32));
    let a = f.gen_element(BigUint::from(1u32));
    let b = f.gen_element(BigUint::from(11u32));
    let c = a.mul(&b);
    assert_eq!(c.v, BigUint::from(0u32));
  }

  #[test]
  fn test_mul_above_order_result() {
    let f = Field::new(BigUint::from(11u32));
    let a = f.gen_element(BigUint::from(1u32));
    let b = f.gen_element(BigUint::from(11u32));
    let c = a.mul(&b);
    assert_eq!(c.v, BigUint::from(0u32));
  }

  struct InvTestCase {
    order: u32,
    v: u32,
    exp: u32,
  }

  #[test]
  fn test_inv1() -> Result<(), String> {
    let test_cases = [
      InvTestCase { order: 53u32, v: 97u32, exp: 47u32 },
      InvTestCase { order: 13u32, v: 5u32, exp: 8u32 },
    ];

    for x in test_cases {
      let f = Field::new(BigUint::from(x.order));
      let a = f.gen_element(BigUint::from(x.v));
      let inv = a.inv()?;
      //println!("inv of {} (mod {}): exp={}, act={}", x.v, x.order, x.exp, inv.v);
      assert_eq!(inv.v, BigUint::from(x.exp));
    }
    Ok(())
  }
}
