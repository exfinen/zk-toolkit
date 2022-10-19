use crate::snarks::{
  r1cs::R1CS,
  polynomial::Polynomial,
};

pub struct QAP {
  pub polys: Vec<Polynomial>,
}

impl QAP {
  fn lagrange_interpolation() {
    // first need to know:
    // 1. # of points
    // 2. expected value for each point
  }

  pub fn build(_r1cs: R1CS) -> QAP {
    QAP::lagrange_interpolation();
    QAP { polys: vec![] }
  }
}

#[cfg(test)]
mod tests {
  // use super::*;
  // use crate::snarks::equation_parser::Parser;

  /*
  Witness
  [1, 3, 35, 9, 27, 30]

  A
  [0, 1, 0, 0, 0, 0]
  [0, 0, 0, 1, 0, 0]
  [0, 1, 0, 0, 1, 0]
  [5, 0, 0, 0, 0, 1]
  B
  [0, 1, 0, 0, 0, 0]
  [0, 1, 0, 0, 0, 0]
  [1, 0, 0, 0, 0, 0]
  [1, 0, 0, 0, 0, 0]
  C
  [0, 0, 0, 1, 0, 0]
  [0, 0, 0, 0, 1, 0]
  [0, 0, 0, 0, 0, 1]
  [0, 0, 1, 0, 0, 0]


  */
  #[test]
  fn test1() {
  }
}