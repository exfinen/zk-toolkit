use crate::snarks::gates::arith_circuit::ArithCircuit;
use crate::building_block::field::Field;

#[derive(Debug, PartialEq, Clone)]
pub enum BoolCircuit {
  Leaf(bool),
  And(Box<BoolCircuit>, Box<BoolCircuit>),
  Xor(Box<BoolCircuit>, Box<BoolCircuit>),
}

pub struct GateHelper();

impl GateHelper {
  pub fn eval(root: &BoolCircuit) -> bool {
    match root {
      BoolCircuit::Leaf(x) => *x,
      BoolCircuit::And(a, b) => GateHelper::eval(&a) && GateHelper::eval(&b),
      BoolCircuit::Xor(a, b) => {
        let a = GateHelper::eval(&a);
        let b = GateHelper::eval(&b);
        !(a && b) && (a || b)
      }
    }
  }

  pub fn to_arith_circuit(f: Field, root: BoolCircuit) -> ArithCircuit {
    match root {
      BoolCircuit::Leaf(x) => ArithCircuit::Leaf(f.elem(&x)),
      BoolCircuit::And(a, b) => {
        let a = GateHelper::eval(&a);
        let b = GateHelper::eval(&b);
        let a = ArithCircuit::Leaf(f.elem(&a));
        let b = ArithCircuit::Leaf(f.elem(&b));
        // AND(a, b) = ab
        ArithCircuit::Mul(Box::new(a), Box::new(b))
      },
      BoolCircuit::Xor(a, b) => {
        let a = GateHelper::eval(&a);
        let b = GateHelper::eval(&b);
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
