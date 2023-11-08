use crate::building_block::curves::mcl::{
  mcl_fr::MclFr,
  qap::gates::arith_circuit::ArithCircuit,
};

#[derive(Debug, PartialEq, Clone)]
pub enum BoolCircuit {
  Leaf(bool),
  And(Box<BoolCircuit>, Box<BoolCircuit>),
  Xor(Box<BoolCircuit>, Box<BoolCircuit>),
  Or(Box<BoolCircuit>, Box<BoolCircuit>),
}

pub struct Processor();

impl Processor {
  pub fn eval(root: &BoolCircuit) -> bool {
    match root {
      BoolCircuit::Leaf(x) => *x,
      BoolCircuit::And(a, b) => Processor::eval(&a) && Processor::eval(&b),
      BoolCircuit::Xor(a, b) => {
        let a = Processor::eval(&a);
        let b = Processor::eval(&b);
        !(a && b) && (a || b)
      }
      BoolCircuit::Or(a, b) => Processor::eval(&a) || Processor::eval(&b),
    }
  }

  pub fn to_arith_circuit(root: BoolCircuit) -> ArithCircuit {
    match root {
      BoolCircuit::Leaf(x) => ArithCircuit::Leaf(MclFr::from(x)),
      BoolCircuit::And(a, b) => {
        let a = Processor::eval(&a);
        let b = Processor::eval(&b);
        let a = ArithCircuit::Leaf(MclFr::from(a));
        let b = ArithCircuit::Leaf(MclFr::from(b));
        // AND(a, b) = ab
        ArithCircuit::Mul(Box::new(a), Box::new(b))
      },
      BoolCircuit::Xor(a, b) => {
        let a = Processor::eval(&a);
        let b = Processor::eval(&b);
        let a = ArithCircuit::Leaf(MclFr::from(a));
        let b = ArithCircuit::Leaf(MclFr::from(b));

        // XOR(a, b) = (a + b) - 2 ab
        let t1 = ArithCircuit::Add(
          Box::new(a.clone()),
          Box::new(b.clone()),
        );

        let two = ArithCircuit::Leaf(MclFr::from(2));
        let t2 = ArithCircuit::Mul(Box::new(a), Box::new(b));
        let t2 = ArithCircuit::Mul(Box::new(two), Box::new(t2));
        ArithCircuit::Add(
          Box::new(t1),
          Box::new(t2),
        )
      },
      BoolCircuit::Or(a, b) => {
        let a = Processor::eval(&a);
        let b = Processor::eval(&b);
        let a = ArithCircuit::Leaf(MclFr::from(a));
        let b = ArithCircuit::Leaf(MclFr::from(b));
        // Or(a, b) = a + b - a * b
        let t1 = ArithCircuit::Add(Box::new(a.clone()), Box::new(b.clone()));
        let t2 = ArithCircuit::Mul(Box::new(a.clone()), Box::new(b.clone()));
        ArithCircuit::Sub(
          Box::new(t1),
          Box::new(t2),
        )
      },
    }
  }
}
