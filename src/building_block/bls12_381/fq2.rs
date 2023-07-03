use crate::building_block::bls12_381::fq1::Fq1;

pub struct Fq2 {
    u1: Fq1,
    u0: Fq1,
}

impl Fq2 {
    pub fn new(u1: Fq1, u0: Fq1) -> Self {
        Fq2 { u1, u0 }
    }
}

impl Add<Fq2> for Fq2 {
  type Output = Fq2;

  fn add(self, rhs: Fq2) -> Self::Output {
    let u1 =
    Fq2 {
    }
  }
}
