use crate::building_block::bls12_381::fq6::Fq6;

pub struct Fq12 {
    w1: Fq6,
    w0: Fq6,
}

impl Fq12 {
    pub fn new(w1: Fq6, w0: Fq6) -> Self {
        Fq12 { w1, w0 }
    }
}
