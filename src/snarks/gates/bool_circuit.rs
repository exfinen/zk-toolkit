use crate::snarks::gates::{
  arith_circuit::ArithCircuit,
  number::Number,
};
use crate::building_block::field::Field;

#[derive(Debug, PartialEq, Clone)]
pub enum BoolCircuit {
  Leaf(bool),
  And(Box<BoolCircuit>, Box<BoolCircuit>),
  Xor(Box<BoolCircuit>, Box<BoolCircuit>),
}

pub struct HalfAdder();

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
  pub fn add(_a: &Number, _b: &Number) -> Number {
    Number::new(0)
  }
}

pub struct Executor();

impl Executor {
  pub fn eval(root: &BoolCircuit) -> bool {
    match root {
      BoolCircuit::Leaf(x) => *x,
      BoolCircuit::And(a, b) => Executor::eval(&a) && Executor::eval(&b),
      BoolCircuit::Xor(a, b) => {
        let a = Executor::eval(&a);
        let b = Executor::eval(&b);
        !(a && b) && (a || b)
      }
    }
  }

  pub fn to_arith_circuit(f: Field, root: BoolCircuit) -> ArithCircuit {
    match root {
      BoolCircuit::Leaf(x) => ArithCircuit::Leaf(f.elem(&x)),
      BoolCircuit::And(a, b) => {
        let a = Executor::eval(&a);
        let b = Executor::eval(&b);
        let a = ArithCircuit::Leaf(f.elem(&a));
        let b = ArithCircuit::Leaf(f.elem(&b));
        // AND(a, b) = ab
        ArithCircuit::Mul(Box::new(a), Box::new(b))
      },
      BoolCircuit::Xor(a, b) => {
        let a = Executor::eval(&a);
        let b = Executor::eval(&b);
        let a = ArithCircuit::Leaf(f.elem(&a));
        let b = ArithCircuit::Leaf(f.elem(&b));

        // XOR(a, b) = (a + b) - 2 ab
        let t1 = ArithCircuit::Add(
          Box::new(a.clone()), 
          Box::new(b.clone()),
        );

        let two = ArithCircuit::Leaf(f.elem(&2u8));
        let t2 = ArithCircuit::Mul(Box::new(a), Box::new(b));
        let t2 = ArithCircuit::Mul(Box::new(two), Box::new(t2));
        ArithCircuit::Add(
          Box::new(t1),
          Box::new(t2),
        )
      }
    }
  }
}
