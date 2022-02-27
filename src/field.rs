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
    let zero = self.f.zero.to_bigint().unwrap();
    let one = self.f.one.to_bigint().unwrap();

    println!("self.v={}", self.v);

    // x0*a + y0*b = a
    // x1*a + y1*b = b
    // let mut r0 = self.v.to_bigint().unwrap();  // b
    // let mut r1 = self.f.order.to_bigint().unwrap();  // a
    let mut r0 = self.f.order.to_bigint().unwrap();  // a
    let mut r1 = self.v.to_bigint().unwrap();  // b
    let mut x0 = one.clone();
    let mut y0 = zero.clone();
    let mut x1 = zero.clone();
    let mut y1 = one.clone();

    while r1 != zero {
      println!("x0={}, y0={}, r0={}", x0, y0, r0);
      println!("x1={}, y1={}, r1={}", x1, y1, r1);
      // a mod b
      // = a - q*b
      // = (x0*a + y0*b) - q*(x1*a + y1*b)
      // = x0*a - q*x1*a + y0*b - q*y1*b
      // = (x0 - x1*q)*a + (y0 - y1*q)*b
      // = r
      let q = r0.clone() / r1.clone();
      let r2 = r0.clone() % r1.clone();
      let x2 = x0 - x1.clone() * q.clone();
      let y2 = y0 - y1.clone() * q.clone();
      let r2a = x2.clone() * self.f.order.to_bigint().unwrap() + y2.clone() * self.v.to_bigint().unwrap();
      println!("q={}, x2={}, y2={}", q, x2, y2);

      println!("new r={}, calc-r={}", r2, r2a);

      // do next calculattion based on new and previous equations
      r0 = r1;
      r1 = r2;
      x0 = x1;
      y0 = y1;
      x1 = x2;
      y1 = y2;
    }

    println!("result x = {}", x0); // last eq's x when r became 0


    if r1 < zero {
      let v = r1 + self.f.order.to_bigint().unwrap();
      Ok(FieldNum { f: self.f, v: v.to_biguint().unwrap() }) // TODO use loop
    } else {
      Ok(FieldNum { f: self.f, v: r1.to_biguint().unwrap() })
    }
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

  #[test]
  fn test_inv() -> Result<(), String> {
    let f = Field::new(BigUint::from(53u32));
    let a = f.gen_element(BigUint::from(97u32));
    assert_eq!(a.inv()?.v, BigUint::from(0u32));
    Ok(())
  }
}
