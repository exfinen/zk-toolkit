use num_bigint::BigUint;

pub struct Field {
  order: BigUint,
  zero: BigUint,
}

pub struct FieldNum<'a> {
  f: &'a Field,
  v: BigUint,
}

impl Field {
  pub fn new(order: BigUint) -> Self {
    Field { order, zero: BigUint::from(0u32) }
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
}
