use crate::snarks::gates::bool_circuit::{BoolCircuit, Executor};

pub struct HalfAdder();

#[derive(Debug)]
pub struct AdderResult {
  pub sum: bool,
  pub carry: bool,
}

impl HalfAdder {
  // (augend, addend) -> (sum, carry)
  pub fn add(augend: bool, addend: bool) -> AdderResult {
    let sum = BoolCircuit::Xor(
      Box::new(BoolCircuit::Leaf(augend)),
      Box::new(BoolCircuit::Leaf(addend)),
    );
    let carry = BoolCircuit::And(
      Box::new(BoolCircuit::Leaf(augend)),
      Box::new(BoolCircuit::Leaf(addend)),
    );

    let sum = Executor::eval(&sum);
    let carry = Executor::eval(&carry);

    AdderResult { sum, carry }
  }
}

pub struct FullAdder();

impl FullAdder {
  pub fn add(augend: bool, addend: bool, carry: bool) -> AdderResult {
    let res1 = HalfAdder::add(augend, addend);
    let res2 = HalfAdder::add(res1.sum, carry);
    let carry = BoolCircuit::Or(
      Box::new(BoolCircuit::Leaf(res1.carry)),
      Box::new(BoolCircuit::Leaf(res2.carry)),
    );
    let carry = Executor::eval(&carry);
    AdderResult { sum: res2.sum, carry }
  }
}

#[cfg(test)]
mod half_adder_tests {
  use super::*;

  #[test]
  fn add_0_0() {
    let res = HalfAdder::add(false, false);
    assert_eq!(res.sum, false);
    assert_eq!(res.carry, false);
  }

  #[test]
  fn add_1_0_or_0_1() {
    let res = HalfAdder::add(true, false);
    assert_eq!(res.sum, true);
    assert_eq!(res.carry, false);

    let res = HalfAdder::add(false, true);
    assert_eq!(res.sum, true);
    assert_eq!(res.carry, false);
  }

  #[test]
  fn add_1_1() {
    let res = HalfAdder::add(true, true);
    assert_eq!(res.sum, false);
    assert_eq!(res.carry, true);
  }
}

#[cfg(test)]
mod full_adder_tests {
  use super::*;

  #[test]
  fn single_inst_add_0_0_0() {
    let res = FullAdder::add(false, false, false);
    assert_eq!(res.sum, false);
    assert_eq!(res.carry, false);
  }

  #[test]
  fn single_inst_add_1_0_0_or_0_1_0_or_0_0_1() {
    let res = FullAdder::add(true, false, false);
    assert_eq!(res.sum, true);
    assert_eq!(res.carry, false);

    let res = FullAdder::add(false, true, false);
    assert_eq!(res.sum, true);
    assert_eq!(res.carry, false);

    let res = FullAdder::add(false, false, true);
    assert_eq!(res.sum, true);
    assert_eq!(res.carry, false);
  }

  #[test]
  fn single_inst_add_1_1_0_or_1_0_1_or_0_1_1() {
    let res = FullAdder::add(true, true, false);
    assert_eq!(res.sum, false);
    assert_eq!(res.carry, true);

    let res = FullAdder::add(true, false, true);
    assert_eq!(res.sum, false);
    assert_eq!(res.carry, true);

    let res = FullAdder::add(false, true, true);
    assert_eq!(res.sum, false);
    assert_eq!(res.carry, true);
  }

  #[test]
  fn single_inst_add_1_1_1() {
    let res = FullAdder::add(true, true, true);
    assert_eq!(res.sum, true);
    assert_eq!(res.carry, true);
  }
}
