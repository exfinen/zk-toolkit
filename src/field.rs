use num_bigint::BigUint;

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

  pub fn inverse(&self) -> Result<FieldNum, String> {
    if (self.v == self.f.zero) {
      return Err("division by zero".to_string());
    }
    let mut t = self.f.zero.clone();
    let mut r = self.f.order.clone();
    let mut nt = self.f.one.clone();
    let mut nr = self.v.clone() % self.f.order.clone();

    while nr != self.f.zero {
      let q = r / nr.clone();
      t = nt.clone();
      nt = t.clone() - q.clone() * nt;
      r = nr.clone();
      nr = r.clone() - q * nr;
    }

    if t < self.f.zero {
      let v = t + self.f.order.clone();
      Ok(FieldNum { f: self.f, v })
    } else {
      Ok(FieldNum { f: self.f, v: t })
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
  fn test_inverse() -> Result<(), String> {
    let f = Field::new(BigUint::from(11u32));
    let a = f.gen_element(BigUint::from(3u32));
    assert_eq!(a.inverse()?.v, BigUint::from(0u32));
    Ok(())
  }
}
